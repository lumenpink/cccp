#!/bin/sh

# Enable error handling
set -eu

# =============================================================================
# CCCP Act Local CI Runner & Pre-Push Gate
# Implements Ticket #37: local CI verification via 'act' with live spinner,
# structured logging to .git/act-logs/, and graceful degradation.
# Zero external runtime dependencies; strict zero-awk compliance.
# =============================================================================

# -----------------------------------------------------------------------------
# Terminal Colors & NO_COLOR Support
# -----------------------------------------------------------------------------
_act_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        ACT_C_RED='\033[0;31m'
        ACT_C_GREEN='\033[0;32m'
        ACT_C_YELLOW='\033[0;33m'
        ACT_C_CYAN='\033[0;36m'
        ACT_C_BOLD='\033[1m'
        ACT_C_RESET='\033[0m'
    else
        ACT_C_RED=''
        ACT_C_GREEN=''
        ACT_C_YELLOW=''
        ACT_C_CYAN=''
        ACT_C_BOLD=''
        ACT_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# Helper: Format elapsed duration in seconds to human-readable string (e.g. 1m 34s)
# -----------------------------------------------------------------------------
_act_format_time() {
    _total_sec="${1:-0}"
    case "$_total_sec" in
        ''|*[!0-9]*) _total_sec=0 ;;
    esac
    _min=$((_total_sec / 60))
    _sec=$((_total_sec % 60))
    printf '%dm %02ds\n' "$_min" "$_sec"
}

# -----------------------------------------------------------------------------
# Helper: Check act and docker daemon prerequisites
# Returns 0 if ready, 1 if act missing, 2 if docker daemon unreachable
# -----------------------------------------------------------------------------
_act_check_prerequisites() {
    _act_bin="${CCCP_ACT_BIN:-act}"
    if ! command -v "$_act_bin" >/dev/null 2>&1; then
        return 1
    fi

    # Check Docker daemon availability
    if command -v docker >/dev/null 2>&1; then
        if docker info >/dev/null 2>&1 || docker ps >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 2
}

# -----------------------------------------------------------------------------
# Helper: Locate workflow file (.forgejo/workflows or .github/workflows)
# -----------------------------------------------------------------------------
_act_find_workflow() {
    _git_root="${1:-}"
    if [ -z "$_git_root" ]; then
        _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    fi
    [ -z "$_git_root" ] && return 1

    # Check Forgejo workflows first, then GitHub
    for _candidate in \
        "$_git_root/.forgejo/workflows/ci.yml" \
        "$_git_root/.forgejo/workflows/ci.yaml" \
        "$_git_root/.forgejo/workflows/test.yml" \
        "$_git_root/.forgejo/workflows/test.yaml" \
        "$_git_root/.github/workflows/ci.yml" \
        "$_git_root/.github/workflows/ci.yaml" \
        "$_git_root/.github/workflows/test.yml" \
        "$_git_root/.github/workflows/test.yaml"; do
        if [ -f "$_candidate" ]; then
            printf '%s\n' "$_candidate"
            return 0
        fi
    done

    # Check any .yml or .yaml in workflow directories
    for _wf_dir in "$_git_root/.forgejo/workflows" "$_git_root/.github/workflows"; do
        if [ -d "$_wf_dir" ]; then
            for _f in "$_wf_dir"/*.yml "$_wf_dir"/*.yaml; do
                if [ -f "$_f" ]; then
                    printf '%s\n' "$_f"
                    return 0
                fi
            done
        fi
    done

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Get or create .git/act-logs directory
# -----------------------------------------------------------------------------
_act_get_log_dir() {
    _git_dir=$(git rev-parse --git-dir 2>/dev/null || true)
    if [ -z "$_git_dir" ]; then
        _git_dir=".git"
    fi
    _target_dir="$_git_dir/act-logs"
    if [ ! -d "$_target_dir" ]; then
        mkdir -p "$_target_dir" 2>/dev/null || true
    fi
    printf '%s\n' "$_target_dir"
}

# -----------------------------------------------------------------------------
# Core: Pre-Push CI Gate Runner
# Called from cmd_pre_push during git push
# -----------------------------------------------------------------------------
act_ci_gate() {
    _remote="${1:-origin}"
    _remote_url="${2:-}"

    _act_colors

    # 1. Conscious bypass check via environment
    if [ -n "${CCCP_ACT_SKIP:-}" ] || [ -n "${ACT_SKIP:-}" ]; then
        printf '%b[pre-push] CI gate bypassed via environment directive.%b\n' "$ACT_C_YELLOW" "$ACT_C_RESET"
        return 0
    fi

    # 2. Check if repository has CI workflow
    _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    if [ -z "$_git_root" ] || ! _act_find_workflow "$_git_root" >/dev/null 2>&1; then
        # No CI workflow found; allow push without running runner
        return 0
    fi

    # 3. Prerequisites check: act & docker (Graceful degradation)
    if ! _act_check_prerequisites; then
        printf '%b[pre-push] Aviso: '\''act'\'' ou Docker indisponível. Pulando CI gate local (degradação graciosa).%b\n' "$ACT_C_YELLOW" "$ACT_C_RESET"
        return 0
    fi

    # 4. Prepare structured log file path (.git/act-logs/<YYYY-MM-DD>_<HH-MM>_<branch>_<short-sha>.log)
    _log_dir=$(_act_get_log_dir)
    _branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "head" | tr '/' '-')
    _sha=$(git rev-parse --short HEAD 2>/dev/null || echo "0000000")
    _ts=$(date '+%Y-%m-%d_%H-%M')
    _logfile="${_log_dir}/${_ts}_${_branch}_${_sha}.log"

    _job_name="${CCCP_ACT_JOB:-test}"
    _act_bin="${CCCP_ACT_BIN:-act}"

    _t_start=$(date +%s 2>/dev/null || echo 0)

    # 5. Execute act with live feedback
    if [ -t 1 ] && [ "${CI:-}" != "true" ] && [ "${SHELLSPEC:-}" != "1" ]; then
        printf '%b[pre-push] Running CI gate locally...%b\n' "$ACT_C_CYAN" "$ACT_C_RESET"
        printf '  Job: %s\n' "$_job_name"
        printf '  Log: %s\n' "$_logfile"

        # Run in background to display interactive spinner and timer
        "$_act_bin" -j "$_job_name" --no-pull > "$_logfile" 2>&1 &
        _act_pid=$!

        _spin_chars='|/-\'
        _spin_idx=0
        while kill -0 "$_act_pid" 2>/dev/null; do
            _t_curr=$(date +%s 2>/dev/null || echo 0)
            _t_diff=$((_t_curr - _t_start))
            [ "$_t_diff" -lt 0 ] && _t_diff=0
            _time_disp=$(_act_format_time "$_t_diff")

            _spin_char=$(printf '%s' "$_spin_chars" | cut -c$(( (_spin_idx % 4) + 1 )))
            _spin_idx=$((_spin_idx + 1))

            _step_name=$(tail -n 10 "$_logfile" 2>/dev/null | grep -E '(⭐ Run|\[.*\])' | tail -n 1 | sed -e 's/^[[:space:]]*//' -e 's/\x1b\[[0-9;]*m//g' | cut -c1-35 || true)
            [ -z "$_step_name" ] && _step_name="Executing workflow..."

            printf '\r  %b%s%b Job: %s › %-35s [%s]   ' "$ACT_C_CYAN" "$_spin_char" "$ACT_C_RESET" "$_job_name" "$_step_name" "$_time_disp"
            sleep 0.2 2>/dev/null || sleep 1
        done

        wait "$_act_pid"
        _exit_code=$?
        printf '\r%80s\r' "" # clear spinner line
    else
        # Non-interactive or test environment
        "$_act_bin" -j "$_job_name" --no-pull > "$_logfile" 2>&1
        _exit_code=$?
    fi

    _t_end=$(date +%s 2>/dev/null || echo 0)
    _elapsed_sec=$((_t_end - _t_start))
    [ "$_elapsed_sec" -lt 0 ] && _elapsed_sec=0
    _elapsed_fmt=$(_act_format_time "$_elapsed_sec")

    # 6. Evaluation & Summary
    if [ "$_exit_code" -eq 0 ]; then
        printf '%b✅ CI gate passed (%s) — pushing to %s/%s%b\n' "$ACT_C_GREEN" "$_elapsed_fmt" "$_remote" "$_branch" "$ACT_C_RESET"
        return 0
    else
        _failed_step=$(grep -iE '(##\[error\]|error:|fatal:|failure:|failed:)' "$_logfile" 2>/dev/null | tail -n 1 | sed -e 's/^[[:space:]]*//' -e 's/\x1b\[[0-9;]*m//g' | cut -c1-70 || true)
        [ -z "$_failed_step" ] && _failed_step="job '${_job_name}' failed with code ${_exit_code}"

        printf '\n%b❌ CI gate failed (%s) — push aborted%b\n' "$ACT_C_RED" "$_elapsed_fmt" "$ACT_C_RESET"
        printf '   → Step failed: %s\n' "$_failed_step"
        printf '   → Log: %s\n' "$_logfile"
        printf '   → Bypass (conscious): %bgit push --no-verify%b\n\n' "$ACT_C_BOLD" "$ACT_C_RESET"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Main Subcommand: cmd_act
# Local CI Runner, log tailing, and status inspection (Ticket #37)
# -----------------------------------------------------------------------------
cmd_act() {
    _act_colors

    _subcmd="${1:-}"
    case "$_subcmd" in
        "log"|"logs")
            shift || true
            _follow=0
            while [ $# -gt 0 ]; do
                case "$1" in
                    "-f"|"--follow")
                        _follow=1
                        shift
                        ;;
                    "-h"|"--help")
                        echo "Usage: cccp act log [-f|--follow]"
                        return 0
                        ;;
                    *)
                        shift
                        ;;
                esac
            done

            _log_dir=$(_act_get_log_dir)
            _latest_log=$(ls -t "$_log_dir"/*.log 2>/dev/null | head -n 1 || true)

            if [ -z "$_latest_log" ] || [ ! -f "$_latest_log" ]; then
                echo "Error: No act CI logs found in $_log_dir" >&2
                return 1
            fi

            printf '%b=== Most Recent CI Log: %s ===%b\n' "$ACT_C_BOLD" "$_latest_log" "$ACT_C_RESET"
            if [ "$_follow" -eq 1 ]; then
                exec tail -f "$_latest_log"
            else
                cat "$_latest_log"
            fi
            return 0
            ;;

        "run")
            shift || true
            _run_job="${CCCP_ACT_JOB:-test}"
            while [ $# -gt 0 ]; do
                case "$1" in
                    "-j"|"--job")
                        shift
                        [ $# -gt 0 ] || { echo "Error: --job requires a job name" >&2; return 1; }
                        _run_job="$1"
                        shift
                        ;;
                    *)
                        _run_job="$1"
                        shift
                        ;;
                esac
            done
            CCCP_ACT_JOB="$_run_job" act_ci_gate "local" "manual"
            return $?
            ;;

        "status")
            echo "========================================================"
            echo " ★ CCCP Act Local CI Runner Status ★"
            echo "========================================================"
            _act_bin="${CCCP_ACT_BIN:-act}"
            if command -v "$_act_bin" >/dev/null 2>&1; then
                _act_loc=$(command -v "$_act_bin")
                echo " Act Runner Binary: $_act_loc (available)"
            else
                echo " Act Runner Binary: not installed (optional, runs in Docker)"
            fi

            if command -v docker >/dev/null 2>&1 && (docker info >/dev/null 2>&1 || docker ps >/dev/null 2>&1); then
                echo " Docker Daemon:     running (operational)"
            else
                echo " Docker Daemon:     not reachable / stopped"
            fi

            _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
            if [ -n "$_git_root" ]; then
                _wf=$(_act_find_workflow "$_git_root" 2>/dev/null || true)
                if [ -n "$_wf" ]; then
                    echo " Active Workflow:   $_wf"
                else
                    echo " Active Workflow:   none found"
                fi
            fi

            _log_dir=$(_act_get_log_dir)
            _log_count=$(ls "$_log_dir"/*.log 2>/dev/null | wc -l | tr -d ' ' || echo 0)
            echo " Recorded CI Logs:  $_log_count logs in $_log_dir"
            echo "========================================================"
            return 0
            ;;

        "-h"|"--help"|"help"|"")
            if command -v show_help >/dev/null 2>&1; then
                show_help "act"
            else
                echo "Usage: cccp act [log|run|status|help] [options]"
            fi
            return 0
            ;;

        *)
            echo "Error: Unknown act subcommand: $_subcmd" >&2
            echo "Run 'cccp act help' for usage." >&2
            return 1
            ;;
    esac
}
