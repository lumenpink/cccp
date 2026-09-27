#!/bin/sh
# =============================================================================
# CCCP Matraca — Mobile & Offline-Tolerant Web REPL for Antigravity CLI (agy)
# Pure POSIX Shell, Filesystem-based FIFO Queue, Zero external database.
# Strict zero-awk, pure POSIX compliant.
# =============================================================================
set -eu

# -----------------------------------------------------------------------------
# Helper: Determine Matraca Data Directory
# Priority: MATRACA_DATA_DIR -> $XDG_DATA_HOME/cccp/matraca -> ~/.local/share/cccp/matraca
# -----------------------------------------------------------------------------
_matraca_get_dir() {
    if [ -n "${MATRACA_DATA_DIR:-}" ]; then
        _mdir="$MATRACA_DATA_DIR"
    elif [ -n "${XDG_DATA_HOME:-}" ]; then
        _mdir="${XDG_DATA_HOME}/cccp/matraca"
    else
        _mdir="${HOME}/.local/share/cccp/matraca"
    fi

    # Ensure queue directory structure exists
    mkdir -p "${_mdir}/clients" \
             "${_mdir}/queue/pending" \
             "${_mdir}/queue/processing" \
             "${_mdir}/queue/done" \
             "${_mdir}/queue/error" 2>/dev/null || true

    printf '%s\n' "$_mdir"
}

# -----------------------------------------------------------------------------
# Helper: Generate short random alphanumeric string (Pure POSIX)
# -----------------------------------------------------------------------------
_matraca_rand() {
    _len="${1:-6}"
    if [ -r /dev/urandom ]; then
        od -vN 16 -An -tx1 /dev/urandom 2>/dev/null | tr -cd 'a-z0-9' | cut -c 1-"$_len"
    else
        printf '%s%s' "$(date +%s)" "$$" | tr -cd 'a-z0-9' | cut -c 1-"$_len"
    fi
}

# -----------------------------------------------------------------------------
# Helper: Escape String for JSON (Pure POSIX, Zero-awk)
# -----------------------------------------------------------------------------
_matraca_escape_json() {
    _raw="$1"
    _out=""
    _first=1
    while IFS= read -r _line || [ -n "$_line" ]; do
        _esc_line=$(printf '%s' "$_line" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g')
        if [ "$_first" -eq 1 ]; then
            _out="$_esc_line"
            _first=0
        else
            _out="${_out}\\n${_esc_line}"
        fi
    done <<EOF
$_raw
EOF
    printf '%s' "$_out"
}

# -----------------------------------------------------------------------------
# Helper: Extract Simple JSON field from file (jq with POSIX sed fallback)
# -----------------------------------------------------------------------------
_matraca_extract_field() {
    _json_file="$1"
    _field_name="$2"
    if [ ! -f "$_json_file" ]; then
        return 1
    fi
    if command -v jq >/dev/null 2>&1; then
        jq -r ".${_field_name} // empty" "$_json_file" 2>/dev/null && return 0
    fi
    sed -n "s/.*\"${_field_name}\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" "$_json_file" 2>/dev/null | head -n 1
}

# -----------------------------------------------------------------------------
# Helper: Trigger Background Worker Asynchronously (Non-blocking)
# -----------------------------------------------------------------------------
_matraca_trigger_worker() {
    if [ "${MATRACA_NO_AUTO_WORKER:-0}" = "1" ]; then
        return 0
    fi
    _bin=$(command -v cccp 2>/dev/null || true)
    if [ -z "$_bin" ] && [ -n "${GIT_ROOT:-}" ] && [ -x "${GIT_ROOT}/bin/cccp" ]; then
        _bin="${GIT_ROOT}/bin/cccp"
    fi
    if [ -z "$_bin" ]; then
        _bin="$0"
    fi

    # Trigger worker run in detached background process
    if [ -x "$_bin" ]; then
        "$_bin" matraca worker --once >/dev/null 2>&1 &
    fi
}

# -----------------------------------------------------------------------------
# Command: cccp matraca send [--client-id <id>] [--workspace <dir>] <prompt>
# Enqueues prompt into FIFO queue with strict idempotency (<10ms)
# -----------------------------------------------------------------------------
cmd_matraca_send() {
    _client_id=""
    _workspace=""
    _prompt=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --client-id)
                shift
                [ $# -gt 0 ] || { echo "Error: --client-id requires an ID argument" >&2; return 1; }
                _client_id="$1"
                ;;
            --workspace)
                shift
                [ $# -gt 0 ] || { echo "Error: --workspace requires a directory argument" >&2; return 1; }
                _workspace="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca send [--client-id <id>] [--workspace <dir>] <prompt...>"
                return 0
                ;;
            *)
                if [ -z "$_prompt" ]; then
                    _prompt="$1"
                else
                    _prompt="${_prompt} $1"
                fi
                ;;
        esac
        shift
    done

    # If prompt is still empty, try reading from stdin if available
    if [ -z "$_prompt" ] && [ ! -t 0 ]; then
        _prompt=$(cat)
    fi

    if [ -z "$_prompt" ]; then
        printf '{"ok": false, "error": "Prompt cannot be empty"}\n' >&2
        return 1
    fi

    _mdir=$(_matraca_get_dir)
    _now=$(date +%s 2>/dev/null || echo 0)

    # 1. Ensure client_id
    if [ -z "$_client_id" ]; then
        _client_id="cli_${_now}_$(_matraca_rand 8)"
    fi

    # 2. Idempotency Check: if client_id already submitted, return existing job
    _client_file="${_mdir}/clients/${_client_id}"
    if [ -f "$_client_file" ]; then
        _existing_job_id=$(head -n 1 "$_client_file" | tr -d ' \t\r\n')
        if [ -n "$_existing_job_id" ]; then
            for _sub in done processing pending error; do
                _candidate="${_mdir}/queue/${_sub}/${_existing_job_id}.json"
                if [ -f "$_candidate" ]; then
                    cat "$_candidate"
                    printf '\n'
                    return 0
                fi
            done
        fi
    fi

    # 3. Resolve Workspace
    if [ -z "$_workspace" ]; then
        _workspace="${PWD:-$(pwd)}"
    fi

    # 4. Generate Monotonic Unique Job ID (timestamp + random suffix + pid)
    _job_id="${_now}_$(_matraca_rand 6)_$$"

    _esc_prompt=$(_matraca_escape_json "$_prompt")
    _esc_workspace=$(_matraca_escape_json "$_workspace")
    _esc_client_id=$(_matraca_escape_json "$_client_id")

    # 5. Write Job File Atomically to Pending Queue
    _tmp_job="${_mdir}/queue/tmp_${_job_id}.json"
    _target_job="${_mdir}/queue/pending/${_job_id}.json"

    cat > "$_tmp_job" << EOF
{
  "id": "${_job_id}",
  "client_id": "${_esc_client_id}",
  "status": "pending",
  "workspace": "${_esc_workspace}",
  "prompt": "${_esc_prompt}",
  "response": "",
  "exit_code": 0,
  "created_at": ${_now},
  "updated_at": ${_now}
}
EOF

    mv "$_tmp_job" "$_target_job"

    # 6. Record client_id mapping for idempotency
    printf '%s\n' "$_job_id" > "$_client_file"

    # 7. Print immediate acknowledgment in <10ms
    printf '{"ok": true, "id": "%s", "client_id": "%s", "status": "pending", "created_at": %s}\n' \
        "$_job_id" "$_client_id" "$_now"

    # 8. Trigger background worker
    _matraca_trigger_worker

    return 0
}

# -----------------------------------------------------------------------------
# Command: cccp matraca poll <job_id|client_id>
# Non-blocking query for job status and response (<1ms)
# -----------------------------------------------------------------------------
cmd_matraca_poll() {
    if [ $# -lt 1 ]; then
        echo "Usage: cccp matraca poll <job_id|client_id>" >&2
        return 1
    fi
    _query_id="$1"
    _mdir=$(_matraca_get_dir)

    _job_id="$_query_id"
    # Check if input is a known client_id
    if [ -f "${_mdir}/clients/${_query_id}" ]; then
        _job_id=$(head -n 1 "${_mdir}/clients/${_query_id}" | tr -d ' \t\r\n')
    fi

    # Search in order of resolution probability: done, processing, pending, error
    for _sub in done processing pending error; do
        _f="${_mdir}/queue/${_sub}/${_job_id}.json"
        if [ -f "$_f" ]; then
            cat "$_f"
            printf '\n'
            return 0
        fi
    done

    printf '{"ok": false, "status": "not_found", "error": "Job '%s' not found"}\n' "$_query_id" >&2
    return 1
}

# -----------------------------------------------------------------------------
# Command: cccp matraca worker [--daemon] [--once] [--job-id <id>]
# Consumes pending jobs from filesystem FIFO queue and dispatches to agy
# -----------------------------------------------------------------------------
cmd_matraca_worker() {
    _daemon=0
    _target_job_id=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --daemon)
                _daemon=1
                ;;
            --once)
                _daemon=0
                ;;
            --job-id)
                shift
                [ $# -gt 0 ] || { echo "Error: --job-id requires an ID argument" >&2; return 1; }
                _target_job_id="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca worker [--daemon] [--once] [--job-id <id>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)

    _process_single_file() {
        _pending_file="$1"
        [ -f "$_pending_file" ] || return 0

        _fname=$(basename "$_pending_file")
        _processing_file="${_mdir}/queue/processing/${_fname}"

        # Atomic Claim via rename()
        if ! mv "$_pending_file" "$_processing_file" 2>/dev/null; then
            # Another worker claimed it first
            return 0
        fi

        _job_id=$(_matraca_extract_field "$_processing_file" "id")
        _client_id=$(_matraca_extract_field "$_processing_file" "client_id")
        _workspace=$(_matraca_extract_field "$_processing_file" "workspace")

        if [ -z "$_workspace" ] || [ ! -d "$_workspace" ]; then
            _workspace="${PWD:-$(pwd)}"
        fi

        # Extract prompt
        _prompt=""
        if command -v jq >/dev/null 2>&1; then
            _prompt=$(jq -r '.prompt // empty' "$_processing_file" 2>/dev/null || true)
        fi
        if [ -z "$_prompt" ]; then
            _prompt=$(_matraca_extract_field "$_processing_file" "prompt")
        fi

        # Execute Agent Runner
        _agent_output=""
        _agent_exit=0
        _start_time=$(date +%s 2>/dev/null || echo 0)

        if [ -n "${MATRACA_RUNNER:-}" ]; then
            # Custom / Mock Runner for hermetic automated tests
            _agent_output=$(eval "$MATRACA_RUNNER" 2>&1) || _agent_exit=$?
        else
            # Official Antigravity CLI invocation in workspace
            _agy_bin=$(command -v agy 2>/dev/null || true)
            if [ -n "$_agy_bin" ]; then
                _agent_output=$( (cd "$_workspace" && "$_agy_bin" --continue --print "$_prompt") 2>&1 ) || _agent_exit=$?
            else
                _agent_output="Erro: Binário 'agy' (Antigravity CLI) não encontrado no PATH."
                _agent_exit=127
            fi
        fi

        _finish_time=$(date +%s 2>/dev/null || echo 0)
        _esc_out=$(_matraca_escape_json "$_agent_output")
        _esc_p=$(_matraca_escape_json "$_prompt")
        _esc_ws=$(_matraca_escape_json "$_workspace")
        _esc_cid=$(_matraca_escape_json "$_client_id")

        if [ "$_agent_exit" -eq 0 ]; then
            _final_status="done"
            _dest_dir="${_mdir}/queue/done"
        else
            _final_status="error"
            _dest_dir="${_mdir}/queue/error"
        fi

        _tmp_dest="${_mdir}/queue/tmp_final_${_fname}"
        cat > "$_tmp_dest" << EOF
{
  "id": "${_job_id}",
  "client_id": "${_esc_cid}",
  "status": "${_final_status}",
  "workspace": "${_esc_ws}",
  "prompt": "${_esc_p}",
  "response": "${_esc_out}",
  "exit_code": ${_agent_exit},
  "created_at": ${_start_time},
  "updated_at": ${_finish_time}
}
EOF

        mv "$_tmp_dest" "${_dest_dir}/${_fname}"
        rm -f "$_processing_file" 2>/dev/null || true
    }

    if [ -n "$_target_job_id" ]; then
        _single_target="${_mdir}/queue/pending/${_target_job_id}.json"
        _process_single_file "$_single_target"
        return 0
    fi

    if [ "$_daemon" -eq 1 ]; then
        echo "[Matraca Worker] Daemon iniciado. Monitorando fila em ${_mdir}/queue/pending..."
        while true; do
            _found=0
            for _f in "${_mdir}"/queue/pending/*.json; do
                if [ -f "$_f" ]; then
                    _found=1
                    _process_single_file "$_f"
                fi
            done
            if [ "$_found" -eq 0 ]; then
                sleep 1
            fi
        done
    else
        # Process once across all current pending files
        for _f in "${_mdir}"/queue/pending/*.json; do
            if [ -f "$_f" ]; then
                _process_single_file "$_f"
            fi
        done
    fi

    return 0
}

# -----------------------------------------------------------------------------
# Command: cccp matraca history [--limit N]
# Returns JSON array of recent finished & active messages
# -----------------------------------------------------------------------------
cmd_matraca_history() {
    _limit=50
    while [ $# -gt 0 ]; do
        case "$1" in
            --limit|-n)
                shift
                [ $# -gt 0 ] || { echo "Error: --limit requires a number" >&2; return 1; }
                _limit="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca history [--limit <N>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)

    # Collect files from done and error (and pending/processing)
    _count=0
    printf '{"ok": true, "messages": ['
    _first=1

    # In pure POSIX, gather and sort descending
    for _sub in done error processing pending; do
        if [ -d "${_mdir}/queue/${_sub}" ]; then
            for _f in "${_mdir}/queue/${_sub}"/*.json; do
                if [ -f "$_f" ]; then
                    _count=$((_count + 1))
                    if [ "$_count" -gt "$_limit" ]; then
                        break
                    fi
                    if [ "$_first" -eq 1 ]; then
                        _first=0
                    else
                        printf ','
                    fi
                    cat "$_f"
                fi
            done
        fi
        if [ "$_count" -ge "$_limit" ]; then
            break
        fi
    done

    printf ']}\n'
    return 0
}

# -----------------------------------------------------------------------------
# Command: cccp matraca status
# Prints health and queue statistics in terminal or JSON
# -----------------------------------------------------------------------------
cmd_matraca_status() {
    _json=0
    case "${1:-}" in
        --json) _json=1 ;;
        -h|--help)
            echo "Usage: cccp matraca status [--json]"
            return 0
            ;;
    esac

    _mdir=$(_matraca_get_dir)

    _count_dir() {
        _d="$1"
        _c=0
        if [ -d "$_d" ]; then
            for _f in "${_d}"/*.json; do
                [ -f "$_f" ] && _c=$((_c + 1))
            done
        fi
        printf '%s' "$_c"
    }

    _pending_count=$(_count_dir "${_mdir}/queue/pending")
    _processing_count=$(_count_dir "${_mdir}/queue/processing")
    _done_count=$(_count_dir "${_mdir}/queue/done")
    _error_count=$(_count_dir "${_mdir}/queue/error")

    _clients_count=0
    if [ -d "${_mdir}/clients" ]; then
        for _cf in "${_mdir}/clients"/*; do
            [ -f "$_cf" ] && _clients_count=$((_clients_count + 1))
        done
    fi

    if [ "$_json" -eq 1 ]; then
        printf '{"ok": true, "pending": %s, "processing": %s, "done": %s, "error": %s, "clients": %s, "data_dir": "%s"}\n' \
            "$_pending_count" "$_processing_count" "$_done_count" "$_error_count" "$_clients_count" "$_mdir"
    else
        echo "🦜 CCCP Matraca — Queue & Spool Status"
        echo "======================================"
        echo "  Diretório : ${_mdir}"
        echo "  Na fila   : ${_pending_count}"
        echo "  Executando: ${_processing_count}"
        echo "  Concluídos: ${_done_count}"
        echo "  Erros     : ${_error_count}"
        echo "  Clientes  : ${_clients_count}"
    fi

    return 0
}

# -----------------------------------------------------------------------------
# Command: cccp matraca clean [--days N]
# Cleans old jobs from done and error directories
# -----------------------------------------------------------------------------
cmd_matraca_clean() {
    _days=7
    while [ $# -gt 0 ]; do
        case "$1" in
            --days|-d)
                shift
                [ $# -gt 0 ] || { echo "Error: --days requires a number" >&2; return 1; }
                _days="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca clean [--days <N>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)
    # Find files older than N days in done and error queues
    find "${_mdir}/queue/done" "${_mdir}/queue/error" -type f -mtime +"$_days" -delete 2>/dev/null || true
    echo "Fila do Matraca limpa com sucesso (arquivos > ${_days} dias removidos)."
    return 0
}

# -----------------------------------------------------------------------------
# Main Subcommand Dispatcher: cmd_matraca
# -----------------------------------------------------------------------------
cmd_matraca() {
    if [ $# -eq 0 ]; then
        # Default behavior when called without arguments: open web REPL cockpit
        if command -v cmd_web >/dev/null 2>&1; then
            cmd_web matraca
            return $?
        else
            echo "Usage: cccp matraca [send|poll|worker|history|status|clean|web] [options]"
            return 1
        fi
    fi

    _sub="$1"
    shift

    case "$_sub" in
        send)
            cmd_matraca_send "$@"
            ;;
        poll)
            cmd_matraca_poll "$@"
            ;;
        worker)
            cmd_matraca_worker "$@"
            ;;
        history)
            cmd_matraca_history "$@"
            ;;
        status)
            cmd_matraca_status "$@"
            ;;
        clean)
            cmd_matraca_clean "$@"
            ;;
        web|serve)
            if command -v cmd_web >/dev/null 2>&1; then
                cmd_web matraca "$@"
            else
                echo "Error: cmd_web is not available in current environment" >&2
                return 1
            fi
            ;;
        -h|--help|help)
            echo "cccp matraca - Mobile & Offline-Tolerant Web REPL for Antigravity (agy)"
            echo "====================================================================="
            echo ""
            echo "Usage:"
            echo "  cccp matraca send [--client-id <id>] [--workspace <dir>] <prompt...>"
            echo "  cccp matraca poll <job_id|client_id>"
            echo "  cccp matraca worker [--daemon] [--once] [--job-id <id>]"
            echo "  cccp matraca history [--limit <N>]"
            echo "  cccp matraca status [--json]"
            echo "  cccp matraca clean [--days <N>]"
            echo "  cccp matraca web [options]"
            return 0
            ;;
        *)
            # If unknown subcommand, check if it was intended as a prompt for send
            cmd_matraca_send "$_sub" "$@"
            ;;
    esac
}
