#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    if [ -f "$GIT_ROOT/src/config/config.sh" ]; then
        . "$GIT_ROOT/src/config/config.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/config_manager.sh" ]; then
        . "$GIT_ROOT/src/utils/config_manager.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/repo.sh" ]; then
        . "$GIT_ROOT/src/utils/repo.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/tickets.sh" ]; then
        . "$GIT_ROOT/src/utils/tickets.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/catalog.sh" ]; then
        . "$GIT_ROOT/src/utils/catalog.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Helper: Resolve Gemini API Key (Memory -> ~/.env -> .cccprc)
# -----------------------------------------------------------------------------
_diva_resolve_api_key() {
    # 1. Environment / In-memory variable
    if [ -n "${GEMINI_API_KEY:-}" ]; then
        printf '%s\n' "$GEMINI_API_KEY"
        return 0
    fi

    # 2. Check ~/.env (or local .env)
    for _env_file in "${HOME}/.env" ".env"; do
        if [ -f "$_env_file" ] && command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_env_file" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    done

    # 3. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_local_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    # 4. Check global config (~/.config/cccp/config or ~/.cccprc)
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    fi
    [ -z "$_global_cfg" ] && [ -f "${HOME}/.cccprc" ] && _global_cfg="${HOME}/.cccprc"

    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_global_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Resolve Forge & Token for Active Repository
# -----------------------------------------------------------------------------
_diva_resolve_forge() {
    if command -v detect_repo_info >/dev/null 2>&1; then
        detect_repo_info
    else
        return 1
    fi
}

_diva_resolve_forge_token() {
    _forge="$1"
    if command -v resolve_forge_token >/dev/null 2>&1; then
        resolve_forge_token "$_forge" && return 0
    fi
    if command -v _catalog_resolve_token >/dev/null 2>&1; then
        _catalog_resolve_token "$_forge" && return 0
    fi
    return 1
}

_diva_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

# -----------------------------------------------------------------------------
# Helper: Resolve Text Editor for Ticket Body Editing
# Priority: DIVA_EDITOR -> .cccprc diva_editor -> VISUAL/EDITOR -> kv -> kvant -> vi
# -----------------------------------------------------------------------------
_diva_resolve_editor() {
    [ -n "${DIVA_EDITOR:-}" ] && { printf '%s\n' "$DIVA_EDITOR"; return 0; }

    # 1. Check local .cccprc
    if command -v get_local_config_file >/dev/null 2>&1; then
        _lcfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$_lcfg" ] && [ -f "$_lcfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _e=$(read_file_key "$_lcfg" "diva_editor" 2>/dev/null || true)
            [ -n "$_e" ] && { printf '%s\n' "$_e"; return 0; }
        fi
    fi

    # 2. Check global .cccprc
    if command -v get_global_config_file >/dev/null 2>&1; then
        _gcfg=$(get_global_config_file 2>/dev/null || true)
        if [ -n "$_gcfg" ] && [ -f "$_gcfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _e=$(read_file_key "$_gcfg" "diva_editor" 2>/dev/null || true)
            [ -n "$_e" ] && { printf '%s\n' "$_e"; return 0; }
        fi
    fi

    # 3. Check VISUAL / EDITOR
    if [ -n "${VISUAL:-}" ]; then
        printf '%s\n' "$VISUAL"
        return 0
    fi
    if [ -n "${EDITOR:-}" ]; then
        printf '%s\n' "$EDITOR"
        return 0
    fi

    # 4. Probe dedicated micro-editor with full keyboard navigation: kv / kvant
    if command -v kv >/dev/null 2>&1; then
        printf 'kv\n'
        return 0
    fi
    if command -v kvant >/dev/null 2>&1; then
        printf 'kvant\n'
        return 0
    fi
    if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
        if _cccp_ensure_comrade "kv" 0 >&2; then
            if command -v kv >/dev/null 2>&1; then
                printf 'kv\n'
                return 0
            fi
        fi
    fi

    # 5. Fallback to standard POSIX editor
    if command -v vi >/dev/null 2>&1; then
        printf 'vi\n'
        return 0
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Resolve Custom Repo Input (Support partial name or full owner/repo)
# Outputs: "forge owner repo host"
# Returns: 0 on unique resolution, 1 on ambiguity or unresolvable name
# -----------------------------------------------------------------------------
_diva_resolve_custom_repo() {
    _input_repo="$1"
    [ -n "$_input_repo" ] || return 1

    # 1. If already in owner/repo format, extract and return
    case "$_input_repo" in
        */*)
            _r_owner=$(printf '%s' "$_input_repo" | cut -d'/' -f1)
            _r_repo=$(printf '%s' "$_input_repo" | cut -d'/' -f2)
            _r_forge="forgejo"
            _r_host=""

            # Check if matching entry exists in catalog to infer forge/host
            _cat_files=""
            _d_root=""
            if command -v get_git_root >/dev/null 2>&1; then
                _d_root=$(get_git_root 2>/dev/null || true)
            fi
            [ -z "$_d_root" ] && _d_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
            [ -n "$_d_root" ] && _cat_files="${_d_root}/catalog.json $(dirname "$_d_root")/catalog.json"
            _cat_files="${_cat_files} catalog.json ${HOME}/src/catalog.json ${HOME}/.config/cccp/catalog.json"

            for _cf in $_cat_files; do
                if [ -f "$_cf" ] && command -v _catalog_read_existing_json >/dev/null 2>&1; then
                    while IFS="$(printf '\t')" read -r _cn _cf_forge _c_slug _c_lic _c_up _c_web _c_top _c_desc || [ -n "$_cn" ]; do
                        if [ "$_c_slug" = "$_input_repo" ]; then
                            _r_forge="$_cf_forge"
                            break 2
                        fi
                    done <<EOF
$(_catalog_read_existing_json "$_cf" 2>/dev/null || true)
EOF
                fi
            done

            if [ "$_r_forge" = "codeberg" ]; then
                _r_host="codeberg.org"
            elif [ "$_r_forge" = "tildegit" ]; then
                _r_host="tildegit.org"
            elif [ "$_r_forge" = "github" ]; then
                _r_host="github.com"
            fi

            printf '%s %s %s %s\n' "$_r_forge" "$_r_owner" "$_r_repo" "$_r_host"
            return 0
            ;;
    esac

    # 2. Partial name resolution
    _candidates=""

    # 2a. Search catalog.json
    _cat_files=""
    _d_root=""
    if command -v get_git_root >/dev/null 2>&1; then
        _d_root=$(get_git_root 2>/dev/null || true)
    fi
    [ -z "$_d_root" ] && _d_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    [ -n "$_d_root" ] && _cat_files="${_d_root}/catalog.json $(dirname "$_d_root")/catalog.json"
    _cat_files="${_cat_files} catalog.json ${HOME}/src/catalog.json ${HOME}/.config/cccp/catalog.json"

    for _cf in $_cat_files; do
        if [ -f "$_cf" ] && command -v _catalog_read_existing_json >/dev/null 2>&1; then
            while IFS="$(printf '\t')" read -r _cn _cf_forge _c_slug _c_lic _c_up _c_web _c_top _c_desc || [ -n "$_cn" ]; do
                [ -n "$_c_slug" ] || continue
                _c_r=$(printf '%s' "$_c_slug" | cut -d'/' -f2)
                if [ "$_cn" = "$_input_repo" ] || [ "$_c_r" = "$_input_repo" ]; then
                    _c_host_def=""
                    case "$_cf_forge" in
                        github) _c_host_def="github.com" ;;
                        codeberg) _c_host_def="codeberg.org" ;;
                        tildegit) _c_host_def="tildegit.org" ;;
                    esac
                    _cand="${_c_slug}:${_cf_forge}:${_c_host_def}"
                    case " ${_candidates} " in
                        *" ${_cand} "*) ;;
                        *) _candidates="${_candidates}${_candidates:+ }${_cand}" ;;
                    esac
                fi
            done <<EOF
$(_catalog_read_existing_json "$_cf" 2>/dev/null || true)
EOF
        fi
    done

    # 2b. Search sibling directories
    if [ -n "$_d_root" ]; then
        _parent=$(dirname "$_d_root")
        if [ -d "${_parent}/${_input_repo}/.git" ]; then
            if _sib_info=$(detect_repo_info "${_parent}/${_input_repo}" 2>/dev/null); then
                _s_forge=$(printf '%s' "$_sib_info" | cut -d' ' -f1)
                _s_owner=$(printf '%s' "$_sib_info" | cut -d' ' -f2)
                _s_repo=$(printf '%s' "$_sib_info" | cut -d' ' -f3)
                _s_host=$(printf '%s' "$_sib_info" | cut -d' ' -f4)
                if [ -n "$_s_owner" ] && [ -n "$_s_repo" ]; then
                    _cand="${_s_owner}/${_s_repo}:${_s_forge}:${_s_host}"
                    case " ${_candidates} " in
                        *" ${_cand} "*) ;;
                        *) _candidates="${_candidates}${_candidates:+ }${_cand}" ;;
                    esac
                fi
            fi
        fi
    fi

    # 2c. Check current repo if it matches
    _cur_owner=""
    _cur_forge=""
    _cur_host=""
    if _cur_info=$(detect_repo_info 2>/dev/null); then
        _cur_forge=$(printf '%s' "$_cur_info" | cut -d' ' -f1)
        _cur_owner=$(printf '%s' "$_cur_info" | cut -d' ' -f2)
        _cur_repo=$(printf '%s' "$_cur_info" | cut -d' ' -f3)
        _cur_host=$(printf '%s' "$_cur_info" | cut -d' ' -f4)
        if [ "$_cur_repo" = "$_input_repo" ] && [ -n "$_cur_owner" ]; then
            _cand="${_cur_owner}/${_cur_repo}:${_cur_forge}:${_cur_host}"
            case " ${_candidates} " in
                *" ${_cand} "*) ;;
                *) _candidates="${_candidates}${_candidates:+ }${_cand}" ;;
            esac
        fi
    fi

    # Count unique candidates
    _count=0
    for _c in $_candidates; do
        _count=$((_count + 1))
    done

    if [ "$_count" -eq 1 ]; then
        _c_slug=$(printf '%s' "$_candidates" | cut -d':' -f1)
        _c_forge=$(printf '%s' "$_candidates" | cut -d':' -f2)
        _c_host=$(printf '%s' "$_candidates" | cut -d':' -f3)
        _r_owner=$(printf '%s' "$_c_slug" | cut -d'/' -f1)
        _r_repo=$(printf '%s' "$_c_slug" | cut -d'/' -f2)
        [ -z "$_c_forge" ] && _c_forge="forgejo"
        printf '%s %s %s %s\n' "$_c_forge" "$_r_owner" "$_r_repo" "$_c_host"
        return 0
    elif [ "$_count" -gt 1 ]; then
        printf 'Error: O nome de repositório "%s" é ambíguo. Encontrados múltiplos correspondentes:\n' "$_input_repo" >&2
        for _c in $_candidates; do
            _c_slug=$(printf '%s' "$_c" | cut -d':' -f1)
            _c_forge=$(printf '%s' "$_c" | cut -d':' -f2)
            printf '  - %s (%s)\n' "$_c_slug" "${_c_forge:-desconhecida}" >&2
        done
        printf 'Por favor, informe o nome completo no formato "proprietario/repositorio".\n' >&2
        return 1
    fi

    # Fallback: if no candidates found in catalog or siblings, try active owner
    _fallback_owner=""
    if [ -n "$_cur_owner" ]; then
        _fallback_owner="$_cur_owner"
    elif [ -n "${FORGEJO_OWNER:-}" ]; then
        _fallback_owner="$FORGEJO_OWNER"
    elif [ -n "${CODEBERG_OWNER:-}" ]; then
        _fallback_owner="$CODEBERG_OWNER"
    elif [ -n "${GITHUB_OWNER:-}" ]; then
        _fallback_owner="$GITHUB_OWNER"
    fi

    if [ -n "$_fallback_owner" ]; then
        _fb_forge="${_cur_forge:-forgejo}"
        _fb_host="${_cur_host:-}"
        printf '%s %s %s %s\n' "$_fb_forge" "$_fallback_owner" "$_input_repo" "$_fb_host"
        return 0
    fi

    printf 'Error: Repositório "%s" não encontrado. Especifique no formato "proprietario/repositorio".\n' "$_input_repo" >&2
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Escape String for JSON (Pure POSIX)
# -----------------------------------------------------------------------------
_diva_escape_json() {
    _input="$1"
    _out=""
    _first=1
    while IFS= read -r _l || [ -n "$_l" ]; do
        if command -v escape_json_string >/dev/null 2>&1; then
            _esc_l=$(escape_json_string "$_l")
        else
            _esc_l=$(printf '%s' "$_l" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g')
        fi
        if [ "$_first" -eq 1 ]; then
            _out="$_esc_l"
            _first=0
        else
            _out="${_out}\\n${_esc_l}"
        fi
    done <<EOF
$_input
EOF
    printf '%s' "$_out"
}

# -----------------------------------------------------------------------------
# Helper: Save Ticket Draft to .antigravity/diva_drafts/
# Called whenever publication fails so no generated ticket is silently lost.
# -----------------------------------------------------------------------------
_diva_save_draft() {
    _d_forge="${1:-unknown}"
    _d_owner="${2:-}"
    _d_repo="${3:-}"
    _d_title="${4:-}"
    _d_body="${5:-}"
    _d_labels="${6:-}"

    _d_git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$_d_git_root" ]; then
        printf '[ERRO] Impossível determinar git root para salvar draft.\n' >&2
        return 1
    fi

    _d_dir="${_d_git_root}/.antigravity/diva_drafts"
    mkdir -p "$_d_dir"
    if command -v ensure_gitignored >/dev/null 2>&1; then
        ensure_gitignored ".antigravity/"
    fi

    _d_ts=$(date +%Y%m%d_%H%M%S 2>/dev/null || printf 'unknown')
    _d_slug=$(printf '%s_%s_%s' "$_d_forge" "$_d_owner" "$_d_repo" | tr '/' '_')
    _d_file="${_d_dir}/${_d_ts}_${_d_slug}.json"

    _d_esc_title=$(_diva_escape_json "$_d_title")
    _d_esc_body=$(_diva_escape_json "$_d_body")
    _d_esc_labels=$(_diva_escape_json "$_d_labels")

    printf '{"forge":"%s","owner":"%s","repo":"%s","title":"%s","body":"%s","labels":"%s","saved_at":"%s"}\n' \
        "$_d_forge" "$_d_owner" "$_d_repo" \
        "$_d_esc_title" "$_d_esc_body" "$_d_esc_labels" \
        "$_d_ts" > "$_d_file"

    printf '  ⚠️  Draft salvo em: %s\n' "$_d_file" >&2
}

# -----------------------------------------------------------------------------
# Helper: Call Google Gemini REST API
# -----------------------------------------------------------------------------
_diva_call_gemini() {
    _api_key="$1"
    _model="$2"
    _user_text="$3"
    _repo_context="${4:-}"
    _temperature="${5:-0.2}"
    _refinement="${6:-}"
    _prev_tickets="${7:-}"

    case "$_temperature" in
        [0-9]*.[0-9]*|[0-9]*) ;;
        *) _temperature="0.2" ;;
    esac

    _esc_text=$(_diva_escape_json "$_user_text")
    _esc_ctx=$(_diva_escape_json "$_repo_context")

    _sys_instructions="You are a senior software engineering triage assistant (O Divã do Gosplan). \
Your task is to analyze user reports, bug descriptions, and feature requests, and decompose them into one or more discrete, atomic issue tickets following Conventional Commits format for titles (e.g. feat(scope): title, fix(scope): title, refactor(scope): title). \
Split multi-topic or unrelated problems into separate tickets. \
Return ONLY a valid JSON object matching this schema: \
{\\\"tickets\\\": [{\\\"title\\\": \\\"Conventional Commit title\\\", \\\"rationale\\\": \\\"Why this is a distinct issue\\\", \\\"labels\\\": [\\\"bug\\\"], \\\"body\\\": \\\"## Description\\\\n\\\\n## Steps to Reproduce\\\\n\\\\n## Expected vs Actual\\\\n\\\"}]} \
Write the ticket body and descriptions in the language used by the user report."

    if [ -n "$_refinement" ] || [ -n "$_prev_tickets" ]; then
        _esc_ref=$(_diva_escape_json "$_refinement")
        _esc_prev=$(_diva_escape_json "$_prev_tickets")
        _prompt="Repository context: ${_esc_ctx}\n\nInitial user input:\n${_esc_text}\n\nPrevious proposed tickets JSON:\n${_esc_prev}\n\nUser refinement and parameter adjustment request:\n${_esc_ref}\n\nPlease update and refine the ticket decomposition according to the user's feedback. Return ONLY valid JSON matching the schema."
    else
        _prompt="Repository context: ${_esc_ctx}\n\nUser input to triage:\n${_esc_text}"
    fi

    _req_body=$(cat <<EOF
{
  "system_instruction": {
    "parts": [{ "text": "${_sys_instructions}" }]
  },
  "contents": [
    {
      "parts": [{ "text": "${_prompt}" }]
    }
  ],
  "generationConfig": {
    "responseMimeType": "application/json",
    "temperature": ${_temperature}
  }
}
EOF
)

    _url="https://generativelanguage.googleapis.com/v1beta/models/${_model}:generateContent?key=${_api_key}"

    _res=$(_diva_http -s -f -X POST \
        -H "Content-Type: application/json" \
        -d "$_req_body" \
        "$_url" 2>/dev/null || true)

    if [ -z "$_res" ]; then
        return 1
    fi

    printf '%s\n' "$_res"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Parse Gemini Response and Yield Structured Tickets
# -----------------------------------------------------------------------------
_diva_extract_tickets() {
    _raw_gemini_resp="$1"

    # Extract JSON text part
    _json_text=""
    if command -v jq >/dev/null 2>&1; then
        _json_text=$(printf '%s' "$_raw_gemini_resp" | jq -r '.candidates[0].content.parts[0].text // empty' 2>/dev/null || true)
    else
        # Pure POSIX fallback
        _json_text=$(printf '%s\n' "$_raw_gemini_resp" | sed -n '/[[:space:]]*"text":[[:space:]]*"/,$p' | sed -e '1s/^[[:space:]]*"text":[[:space:]]*"//' -e 's/"[[:space:]]*$//' -e 's/\\n/\n/g' -e 's/\\"/"/g' || true)
    fi

    if [ -z "$_json_text" ]; then
        return 1
    fi

    printf '%s\n' "$_json_text"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Extract Issue HTML URL from Forge Response (Pure POSIX + jq accelerator)
# -----------------------------------------------------------------------------
_diva_extract_html_url() {
    _raw="$1"
    if command -v jq >/dev/null 2>&1; then
        _u=$(printf '%s' "$_raw" | jq -r '.html_url // empty' 2>/dev/null || true)
        if [ -n "$_u" ]; then
            printf '%s\n' "$_u"
            return 0
        fi
    fi
    # Pure POSIX fallback
    printf '%s\n' "$_raw" | sed -n 's/.*"html_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1
}

# -----------------------------------------------------------------------------
# Helper: Format and Print Column-Wrapped Box Row with Closed Right Border
# Width: 79 columns (│ + 1 space + 75 chars content + 1 space + │)
# -----------------------------------------------------------------------------
_diva_box_field() {
    _b_prefix="$1"
    _b_text="$2"
    _b_cont="              "
    _b_first=1

    _trimmed=$(printf '%s' "$_b_text" | tr -d ' \t\n')
    if [ -z "$_trimmed" ]; then
        _spaces=$(printf "%*s" 61 "")
        printf "│ %s%s │\n" "$_b_prefix" "$_spaces"
        return 0
    fi

    printf '%s\n' "$_b_text" | fold -s -w 61 | while IFS= read -r _b_chunk || [ -n "$_b_chunk" ]; do
        _b_chunk=$(printf '%s' "$_b_chunk" | sed 's/[[:space:]]*$//')
        [ -z "$_b_chunk" ] && continue

        if [ "$_b_first" -eq 1 ]; then
            _pfx="$_b_prefix"
            _b_first=0
        else
            _pfx="$_b_cont"
        fi

        _clen=$(printf '%s' "$_b_chunk" | wc -m | tr -d ' ')
        _pad=$((61 - _clen))
        [ "$_pad" -lt 0 ] && _pad=0
        _spaces=""
        [ "$_pad" -gt 0 ] && _spaces=$(printf "%*s" "$_pad" "")
        printf "│ %s%s%s │\n" "$_pfx" "$_b_chunk" "$_spaces"
    done
}

# -----------------------------------------------------------------------------
# Helper: Format and Print Body Preview Lines with Closed Right Border
# -----------------------------------------------------------------------------
_diva_box_body() {
    _bb_body="$1"
    echo "│ 📝 Prévia do Corpo:                                                         │"
    [ -z "$_bb_body" ] && return 0

    printf '%s\n' "$_bb_body" | while IFS= read -r _bb_line || [ -n "$_bb_line" ]; do
        _bb_trimmed=$(printf '%s' "$_bb_line" | tr -d ' \t\n')
        if [ -z "$_bb_trimmed" ]; then
            _bb_spaces=$(printf "%*s" 75 "")
            printf "│ %s │\n" "$_bb_spaces"
        else
            printf '%s\n' "$_bb_line" | fold -s -w 72 | while IFS= read -r _bb_chunk || [ -n "$_bb_chunk" ]; do
                _bb_chunk=$(printf '%s' "$_bb_chunk" | sed 's/[[:space:]]*$//')
                [ -z "$_bb_chunk" ] && continue
                _clen=$(printf '%s' "$_bb_chunk" | wc -m | tr -d ' ')
                _pad=$((72 - _clen))
                [ "$_pad" -lt 0 ] && _pad=0
                _bb_spaces=""
                [ "$_pad" -gt 0 ] && _bb_spaces=$(printf "%*s" "$_pad" "")
                printf "│    %s%s │\n" "$_bb_chunk" "$_bb_spaces"
            done
        fi
    done
}

# -----------------------------------------------------------------------------
# Helper: Display Soviet Ticket Box & Request Interactive Confirmation
# -----------------------------------------------------------------------------
_diva_present_and_dispatch() {
    _tickets_json="$1"
    _forge="$2"
    _owner="$3"
    _repo="$4"
    _dry_run="${5:-0}"
    _host="${6:-}"
    _api_key="${7:-}"
    _model="${8:-gemini-3.6-flash}"
    _user_text="${9:-}"
    _temp="${10:-0.2}"

    _count=0
    if command -v jq >/dev/null 2>&1; then
        _count=$(printf '%s' "$_tickets_json" | jq '.tickets | length' 2>/dev/null || echo 0)
    else
        _count=$(printf '%s' "$_tickets_json" | grep -c '"title":' || echo 0)
    fi

    if [ "$_count" -le 0 ]; then
        echo "Aviso: Nenhum ticket foi identificado pelo modelo."
        return 0
    fi

    echo ""
    echo "★ [ГОСПЛАН] O Divã identificou $_count ticket(s) para abertura ★"
    echo ""

    _created_count=0

    _idx=0
    _should_restart=0
    while [ "$_idx" -lt "$_count" ]; do
        _title=""
        _rationale=""
        _labels=""
        _body=""

        if command -v jq >/dev/null 2>&1; then
            _title=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].title // empty")
            _rationale=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].rationale // empty")
            _labels=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].labels // [] | join(\", \")")
            _body=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].body // empty")
        else
            # Pure POSIX fallback extraction for ticket fields
            _title=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"title"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | sed -n "$((_idx + 1))p")
            [ -z "$_title" ] && _title="Ticket $((_idx + 1))"
            _rationale=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"rationale"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | sed -n "$((_idx + 1))p")
            _labels=""
            _body=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"body"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | sed -n "$((_idx + 1))p" | sed -e 's/\\n/\n/g' -e 's/\\"/"/g')
        fi

        _num=$((_idx + 1))

        if [ "$_dry_run" -eq 1 ]; then
            echo "┌─────────────────────────────────────────────────────────────────────────────┐"
            _hdr=$(printf "[ДИВАН] Ticket %d/%d" "$_num" "$_count")
            _hlen=$(printf '%s' "$_hdr" | wc -m | tr -d ' ')
            _hpad=$((75 - _hlen))
            [ "$_hpad" -lt 0 ] && _hpad=0
            _hspaces=""
            [ "$_hpad" -gt 0 ] && _hspaces=$(printf "%*s" "$_hpad" "")
            printf "│ %s%s │\n" "$_hdr" "$_hspaces"
            echo "├─────────────────────────────────────────────────────────────────────────────┤"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                _diva_box_field "🎯 Destino  : " "${_owner}/${_repo} (${_forge})"
            fi
            _diva_box_field "📌 Título   : " "$_title"
            if [ -n "$_rationale" ]; then
                _diva_box_field "💡 Racional : " "$_rationale"
            fi
            if [ -n "$_labels" ]; then
                _diva_box_field "🏷️ Labels   : " "$_labels"
            fi
            _diva_box_body "$_body"
            echo "└─────────────────────────────────────────────────────────────────────────────┘"
            echo "  [DRY-RUN] Simulação ativa. Ticket não publicado."
            _idx=$((_idx + 1))
            continue
        fi

        while true; do
            echo "┌─────────────────────────────────────────────────────────────────────────────┐"
            _hdr=$(printf "[ДИВАН] Ticket %d/%d" "$_num" "$_count")
            _hlen=$(printf '%s' "$_hdr" | wc -m | tr -d ' ')
            _hpad=$((75 - _hlen))
            [ "$_hpad" -lt 0 ] && _hpad=0
            _hspaces=""
            [ "$_hpad" -gt 0 ] && _hspaces=$(printf "%*s" "$_hpad" "")
            printf "│ %s%s │\n" "$_hdr" "$_hspaces"
            echo "├─────────────────────────────────────────────────────────────────────────────┤"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                _diva_box_field "🎯 Destino  : " "${_owner}/${_repo} (${_forge})"
            fi
            _diva_box_field "📌 Título   : " "$_title"
            if [ -n "$_rationale" ]; then
                _diva_box_field "💡 Racional : " "$_rationale"
            fi
            if [ -n "$_labels" ]; then
                _diva_box_field "🏷️ Labels   : " "$_labels"
            fi
            _diva_box_body "$_body"
            echo "└─────────────────────────────────────────────────────────────────────────────┘"

            # Interactive confirmation
            _choice="y"
            if [ -t 0 ]; then
                if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                    printf "  [y] Criar em %s/%s (%s)  [n] Pular  [t] Editar título  [b] Editar corpo  [r] Rediscutir com IA  [f] Ver corpo completo → " "$_owner" "$_repo" "$_forge"
                else
                    printf "  [y] Criar  [n] Pular  [t] Editar título  [b] Editar corpo  [r] Rediscutir com IA  [f] Ver corpo completo → "
                fi
                read -r _user_choice || _user_choice="n"
                _choice=$(printf '%s' "$_user_choice" | tr '[:upper:]' '[:lower:]')
            else
                if read -r _user_choice; then
                    _choice=$(printf '%s' "$_user_choice" | tr '[:upper:]' '[:lower:]')
                else
                    _choice="y"
                fi
            fi

            case "$_choice" in
                y|yes|"")
                    echo "  -> Publicando ticket na forja ${_forge} (${_owner}/${_repo})..."
                    _issue_url=""

                    case "$_forge" in
                        codeberg)
                            _base_url="https://codeberg.org"

                            _token=$(_diva_resolve_forge_token "$_forge" 2>/dev/null || true)
                            if [ -z "$_token" ]; then
                                printf '  [ERRO] Token para %s não encontrado. Ticket não publicado.\n' "$_forge" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                break
                            fi

                            _esc_t=$(_diva_escape_json "$_title")
                            _esc_b=$(_diva_escape_json "$_body")
                            _issue_payload="{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}"

                            _post_res=$(_diva_http -s -X POST \
                                -H "Authorization: token ${_token}" \
                                -H "Content-Type: application/json" \
                                -H "Accept: application/json" \
                                -d "$_issue_payload" \
                                "${_base_url}/api/v1/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)

                            _issue_url=$(_diva_extract_html_url "$_post_res")
                            if [ -n "$_issue_url" ]; then
                                echo "  ✅ Criado: ${_issue_url}"
                                _created_count=$((_created_count + 1))
                            else
                                printf '  [ERRO] Falha ao publicar em %s. Resposta: %s\n' "$_forge" "$_post_res" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            fi
                            ;;

                        tildegit)
                            _base_url="https://tildegit.org"

                            _token=$(_diva_resolve_forge_token "$_forge" 2>/dev/null || true)
                            if [ -z "$_token" ]; then
                                printf '  [ERRO] Token para %s não encontrado. Ticket não publicado.\n' "$_forge" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                break
                            fi

                            _esc_t=$(_diva_escape_json "$_title")
                            _esc_b=$(_diva_escape_json "$_body")
                            _issue_payload="{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}"

                            _post_res=$(_diva_http -s -X POST \
                                -H "Authorization: token ${_token}" \
                                -H "Content-Type: application/json" \
                                -H "Accept: application/json" \
                                -d "$_issue_payload" \
                                "${_base_url}/api/v1/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)

                            _issue_url=$(_diva_extract_html_url "$_post_res")
                            if [ -n "$_issue_url" ]; then
                                echo "  ✅ Criado: ${_issue_url}"
                                _created_count=$((_created_count + 1))
                            else
                                printf '  [ERRO] Falha ao publicar em %s. Resposta: %s\n' "$_forge" "$_post_res" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            fi
                            ;;

                        forgejo)
                            # Generic Forgejo/Gitea instance — API identical to Codeberg.
                            # Use the host detected from the git remote.
                            if [ -z "$_host" ]; then
                                printf '  [ERRO] Host da instância Forgejo não detectado. Não é possível publicar.\n' >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                break
                            fi
                            _base_url="https://${_host}"

                            _token=$(_diva_resolve_forge_token "$_forge" 2>/dev/null || true)
                            if [ -z "$_token" ]; then
                                printf '  [ERRO] Token para Forgejo (%s) não encontrado. Configure FORGEJO_TOKEN ou forgejo_token no .cccprc.\n' "$_host" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                break
                            fi

                            _esc_t=$(_diva_escape_json "$_title")
                            _esc_b=$(_diva_escape_json "$_body")
                            _issue_payload="{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}"

                            _post_res=$(_diva_http -s -X POST \
                                -H "Authorization: token ${_token}" \
                                -H "Content-Type: application/json" \
                                -H "Accept: application/json" \
                                -d "$_issue_payload" \
                                "${_base_url}/api/v1/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)

                            _issue_url=$(_diva_extract_html_url "$_post_res")
                            if [ -n "$_issue_url" ]; then
                                echo "  ✅ Criado: ${_issue_url}"
                                _created_count=$((_created_count + 1))
                            else
                                printf '  [ERRO] Falha ao publicar em Forgejo (%s). Resposta: %s\n' "$_host" "$_post_res" >&2
                                _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            fi
                            ;;

                        github)
                            if command -v gh >/dev/null 2>&1; then
                                _gh_url=$(gh issue create --repo "${_owner}/${_repo}" --title "$_title" --body "$_body" 2>/dev/null || true)
                                if [ -n "$_gh_url" ]; then
                                    echo "  ✅ Criado: ${_gh_url}"
                                    _created_count=$((_created_count + 1))
                                else
                                    printf '  [ERRO] gh cli falhou ao criar issue no GitHub.\n' >&2
                                    _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                fi
                            else
                                _token=$(_diva_resolve_forge_token "github" 2>/dev/null || true)
                                if [ -n "$_token" ]; then
                                    _esc_t=$(_diva_escape_json "$_title")
                                    _esc_b=$(_diva_escape_json "$_body")
                                    _post_res=$(_diva_http -s -X POST \
                                        -H "Authorization: token ${_token}" \
                                        -H "Content-Type: application/json" \
                                        -H "Accept: application/vnd.github+json" \
                                        -d "{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}" \
                                        "https://api.github.com/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)
                                    _issue_url=$(_diva_extract_html_url "$_post_res")
                                    if [ -n "$_issue_url" ]; then
                                        echo "  ✅ Criado: ${_issue_url}"
                                        _created_count=$((_created_count + 1))
                                    else
                                        printf '  [ERRO] Falha ao publicar no GitHub. Resposta: %s\n' "$_post_res" >&2
                                        _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                    fi
                                else
                                    printf '  [ERRO] Nem "gh" CLI nem GITHUB_TOKEN disponíveis para publicação no GitHub.\n' >&2
                                    _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                fi
                            fi
                            ;;

                        *)
                            printf '  [ERRO] Forja "%s" não suportada para publicação automática.\n' "$_forge" >&2
                            _diva_save_draft "$_forge" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            ;;
                    esac
                    break
                    ;;

                f|full)
                    echo ""
                    echo "--- CORPO COMPLETO ---"
                    printf '%s\n' "$_body"
                    echo "-----------------------"
                    ;;

                t|title|e|edit)
                    printf "  Novo título [%s]: " "$_title"
                    if [ -t 0 ] && [ -e /dev/tty ]; then
                        read -r _new_title < /dev/tty || _new_title=""
                    else
                        read -r _new_title || _new_title=""
                    fi
                    if [ -n "$_new_title" ]; then
                        _title="$_new_title"
                        echo "  ✓ Título atualizado."
                    fi
                    ;;

                b|body)
                    _editor=$(_diva_resolve_editor 2>/dev/null || true)

                    _tmp_body=$(mktemp 2>/dev/null || printf '/tmp/diva_body_%s_%s.md' "$$" "$_idx")
                    printf '%s\n' "$_body" > "$_tmp_body"

                    if [ -n "$_editor" ] && { [ -t 0 ] || [ "$_editor" != "vi" ]; }; then
                        echo "  -> Abrindo editor ($_editor) para editar o corpo do ticket..."
                        if [ -t 0 ] && [ -e /dev/tty ]; then
                            $_editor "$_tmp_body" < /dev/tty > /dev/tty 2>&1 || true
                        else
                            $_editor "$_tmp_body" || true
                        fi
                        if [ -f "$_tmp_body" ]; then
                            _body=$(cat "$_tmp_body")
                            echo "  ✓ Corpo atualizado."
                        fi
                    else
                        printf "  Digite o novo corpo (ou deixe vazio para manter atual): "
                        if [ -t 0 ] && [ -e /dev/tty ]; then
                            read -r _inline_body < /dev/tty || _inline_body=""
                        else
                            read -r _inline_body || _inline_body=""
                        fi
                        if [ -n "$_inline_body" ]; then
                            _body="$_inline_body"
                            echo "  ✓ Corpo atualizado."
                        fi
                    fi
                    rm -f "$_tmp_body" 2>/dev/null || true
                    ;;

                r|refine|rediscutir)
                    if [ -z "$_api_key" ]; then
                        _api_key=$(_diva_resolve_api_key 2>/dev/null || true)
                    fi
                    if [ -z "$_api_key" ]; then
                        printf '  [ERRO] GEMINI_API_KEY não configurada. Não é possível rediscutir com a IA.\n' >&2
                        continue
                    fi

                    printf "  💬 Instruções para a Gemini (ou ':temp <val>', ':model <val>'): "
                    if [ -t 0 ] && [ -e /dev/tty ]; then
                        read -r _refinement_raw < /dev/tty || _refinement_raw=""
                    else
                        read -r _refinement_raw || _refinement_raw=""
                    fi

                    if [ -z "$_refinement_raw" ]; then
                        echo "  Aviso: Nenhuma instrução informada. Mantendo proposta atual."
                        continue
                    fi

                    # Parse inline directives like :temp <val> or :model <val>
                    _clean_instructions=""
                    _next_arg=""
                    for _tok in $_refinement_raw; do
                        if [ "$_next_arg" = "temp" ]; then
                            _temp="$_tok"
                            _next_arg=""
                        elif [ "$_next_arg" = "model" ]; then
                            _model="$_tok"
                            _next_arg=""
                        elif [ "$_tok" = ":temp" ]; then
                            _next_arg="temp"
                        elif [ "$_tok" = ":model" ]; then
                            _next_arg="model"
                        elif [ "${_tok#*:temp=}" != "$_tok" ]; then
                            _temp="${_tok#:temp=}"
                        elif [ "${_tok#*:model=}" != "$_tok" ]; then
                            _model="${_tok#:model=}"
                        else
                            if [ -z "$_clean_instructions" ]; then
                                _clean_instructions="$_tok"
                            else
                                _clean_instructions="${_clean_instructions} ${_tok}"
                            fi
                        fi
                    done

                    case "$_temp" in
                        [0-9]*.[0-9]*|[0-9]*) ;;
                        *) _temp="0.2" ;;
                    esac

                    echo "  -> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) com refinamento..."
                    _repo_ctx=""
                    if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                        _repo_ctx="${_owner}/${_repo} (${_forge})"
                    fi

                    _raw_resp=$(_diva_call_gemini "$_api_key" "$_model" "$_user_text" "$_repo_ctx" "$_temp" "$_clean_instructions" "$_tickets_json" 2>/dev/null || true)
                    _new_tickets=$(_diva_extract_tickets "$_raw_resp" 2>/dev/null || true)

                    _new_count=0
                    if [ -n "$_new_tickets" ]; then
                        if command -v jq >/dev/null 2>&1; then
                            _new_count=$(printf '%s' "$_new_tickets" | jq '.tickets | length' 2>/dev/null || echo 0)
                        else
                            _new_count=$(printf '%s' "$_new_tickets" | grep -c '"title":' || echo 0)
                        fi
                    fi

                    if [ "$_new_count" -gt 0 ]; then
                        echo "  ✓ Proposta refinada com sucesso pela Gemini ($_new_count ticket(s))!"
                        _tickets_json="$_new_tickets"
                        _count="$_new_count"
                        _idx=0
                        _should_restart=1
                        break
                    else
                        printf '  [ERRO] Falha ao rediscutir com a Gemini ou resposta inválida. Mantendo proposta atual.\n' >&2
                    fi
                    ;;

                n|no|*)
                    echo "  ⏭️  Pulado."
                    break
                    ;;
            esac
        done

        if [ "$_should_restart" -eq 1 ]; then
            _should_restart=0
            continue
        fi

        _idx=$((_idx + 1))
    done

    # Re-sync local tickets if tickets were created
    if [ "$_created_count" -gt 0 ]; then
        if command -v cmd_tickets >/dev/null 2>&1; then
            echo ""
            echo "-> Sincronizando .antigravity/tickets.md..."
            cmd_tickets >/dev/null 2>&1 || true
        fi
    fi

    echo ""
    echo "Sessão concluída. $_created_count ticket(s) publicado(s)."
    return 0
}

# -----------------------------------------------------------------------------
# Main Subcommand Dispatcher: cmd_diva
# -----------------------------------------------------------------------------
cmd_diva() {
    _model="${GEMINI_MODEL:-gemini-3.6-flash}"
    _temp="${GEMINI_TEMPERATURE:-0.2}"
    _dry_run=0
    _user_text=""
    _custom_repo=""

    while [ $# -gt 0 ]; do
        case "$1" in
            config)
                shift
                if [ $# -gt 0 ]; then
                    _key="$1"
                    _local_cfg=$(get_local_config_file 2>/dev/null || true)
                    [ -z "$_local_cfg" ] && _local_cfg="${HOME}/.cccprc"
                    write_file_key "$_local_cfg" "gemini_api_key" "$_key"
                    echo "Chave gemini_api_key salva com sucesso em $_local_cfg"
                    return 0
                else
                    echo "Uso: cccp diva config <GEMINI_API_KEY>"
                    return 1
                fi
                ;;
            --dry-run)
                _dry_run=1
                shift
                ;;
            --model|-m)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _model="$1"
                shift
                ;;
            --temp|-t|--temperature)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _temp="$1"
                shift
                ;;
            --repo|-r)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _custom_repo="$1"
                shift
                ;;
            -h|--help|help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "diva"
                else
                    echo "Uso: cccp diva [options] [texto_do_problema]"
                    echo "  --dry-run          Executa triagem sem publicar na forja"
                    echo "  --model <model>    Modelo Gemini (padrão: gemini-3.6-flash)"
                    echo "  --temp <val>       Temperatura de amostragem (padrão: 0.2)"
                    echo "  --repo <slug>      Repositório alvo (padrão: detectado do git remote)"
                    echo "  config <api-key>   Salva GEMINI_API_KEY no .cccprc"
                fi
                return 0
                ;;
            -*)
                echo "Error: Unknown option '$1'" >&2
                return 1
                ;;
            *)
                if [ -z "$_user_text" ]; then
                    _user_text="$1"
                else
                    _user_text="${_user_text} $1"
                fi
                shift
                ;;
        esac
    done

    # 1. Resolve Gemini API Key
    _api_key=$(_diva_resolve_api_key 2>/dev/null || true)
    if [ -z "$_api_key" ]; then
        echo "Error: GEMINI_API_KEY não configurada." >&2
        echo "Configure através de:" >&2
        echo "  export GEMINI_API_KEY=\"sua_chave\"" >&2
        echo "  ou: cccp diva config <sua_chave>" >&2
        return 1
    fi

    # 2. Resolve Forge Info
    _forge="unknown"
    _owner=""
    _repo=""
    _host=""
    if [ -n "$_custom_repo" ]; then
        if _cinfo=$(_diva_resolve_custom_repo "$_custom_repo"); then
            _forge=$(printf '%s' "$_cinfo" | cut -d' ' -f1)
            _owner=$(printf '%s' "$_cinfo" | cut -d' ' -f2)
            _repo=$(printf '%s' "$_cinfo" | cut -d' ' -f3)
            _host=$(printf '%s' "$_cinfo" | cut -d' ' -f4)
        else
            return 1
        fi
    else
        if _finfo=$(_diva_resolve_forge 2>/dev/null); then
            _forge=$(printf '%s' "$_finfo" | cut -d' ' -f1)
            _owner=$(printf '%s' "$_finfo" | cut -d' ' -f2)
            _repo=$(printf '%s' "$_finfo" | cut -d' ' -f3)
            _host=$(printf '%s' "$_finfo" | cut -d' ' -f4)
        fi
    fi

    # 3. Read input from stdin if not provided as argument
    if [ -z "$_user_text" ]; then
        if [ ! -t 0 ]; then
            _user_text=$(cat)
        else
            echo "★ [ГОСПЛАН] O Divã — Triagem Interativa de Tickets ★"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                echo "🎯 Repositório Alvo: ${_owner}/${_repo} (${_forge})"
            fi
            echo "Descreva seu problema, bug ou funcionalidade (digite seu relato e pressione Enter):"
            printf "> "
            read -r _user_text || _user_text=""
        fi
    fi

    if [ -z "$_user_text" ]; then
        echo "Nenhum relato informado. Encerrando sessão."
        return 0
    fi

    case "$_temp" in
        [0-9]*.[0-9]*|[0-9]*) ;;
        *) _temp="0.2" ;;
    esac

    if [ -n "$_owner" ] && [ -n "$_repo" ]; then
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) para ${_owner}/${_repo} (${_forge})..."
    else
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) para decomposição de tickets..."
    fi

    _repo_ctx="${_owner}/${_repo} (${_forge})"
    _raw_resp=$(_diva_call_gemini "$_api_key" "$_model" "$_user_text" "$_repo_ctx" "$_temp" 2>/dev/null || true)

    if [ -z "$_raw_resp" ]; then
        echo "Error: Falha na comunicação com a API do Google Gemini." >&2
        return 1
    fi

    _tickets_json=$(_diva_extract_tickets "$_raw_resp" 2>/dev/null || true)
    if [ -z "$_tickets_json" ]; then
        echo "Error: Resposta do modelo não pôde ser interpretada." >&2
        return 1
    fi

    _diva_present_and_dispatch "$_tickets_json" "$_forge" "$_owner" "$_repo" "$_dry_run" "$_host" "$_api_key" "$_model" "$_user_text" "$_temp"
}

cmd_divan() {
    cmd_diva "$@"
}
