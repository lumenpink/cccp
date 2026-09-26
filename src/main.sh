#!/bin/sh

# =============================================================================
# Main script entry point & command dispatcher
# =============================================================================

main() {
    # Recursion depth guard to prevent fork bombs
    depth="${CCCP_RECURSION_DEPTH:-0}"
    if [ "$depth" -ge 3 ]; then
        echo "Error: Maximum recursion depth exceeded in cccp." >&2
        return 1
    fi
    export CCCP_RECURSION_DEPTH=$((depth + 1))

    # Global directory override option (-C <path|repo>)
    while [ $# -gt 0 ]; do
        case "$1" in
            -C|--dir)
                shift
                [ $# -gt 0 ] || { echo "Error: $1 requires directory path or repository name" >&2; return 1; }
                _target_repo="$1"
                _resolved_dir=""
                if [ -d "$_target_repo" ]; then
                    _resolved_dir="$_target_repo"
                elif [ -d "${HOME}/src/${_target_repo}" ]; then
                    _resolved_dir="${HOME}/src/${_target_repo}"
                fi

                if [ -z "$_resolved_dir" ] && command -v _catalog_resolve_target_dir >/dev/null 2>&1; then
                    _base_cat=$(_catalog_resolve_target_dir "" 2>/dev/null || echo "")
                    if [ -n "$_base_cat" ] && [ -d "${_base_cat}/${_target_repo}" ]; then
                        _resolved_dir="${_base_cat}/${_target_repo}"
                    fi
                fi

                if [ -n "$_resolved_dir" ]; then
                    cd "$_resolved_dir" || { echo "Error: Failed to change directory to '$_resolved_dir'" >&2; return 1; }
                else
                    echo "Error: Repository directory '$_target_repo' not found" >&2
                    return 1
                fi
                shift
                ;;
            *)
                break
                ;;
        esac
    done

    # Load configuration hierarchy (Defaults < Global < Local < Environment)
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    if [ -n "${1:-}" ]; then
       command="$1"
    else
       command=""
    fi
    current_hook=$(basename "$0")
    
    # Check if running as a hook
    case " ${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction} " in
        *" $current_hook "*)
            command="$current_hook"
            ;;
    esac

    # Hook version audit in git repositories
    if [ -n "${GIT_ROOT:-}" ] && command -v check_hook_version >/dev/null 2>&1; then
        case "$command" in
            "commit"|"commit-msg"|"post-commit"|"pre-push"|"reference-transaction")
                check_hook_version || true
                ;;
        esac
    fi

    # Verify required system tools for operational commands
    if command -v check_system_tools >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"commit-msg"|"post-commit"|"pre-push"|"reference-transaction"|"hooks"|"catalog")
                check_system_tools || exit 1
                ;;
        esac
    fi

    # Non-blocking periodic update check on interactive user commands
    if command -v check_auto_update >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"completion"|"hooks"|"tickets"|"sync-tickets"|"agents"|"update-agents"|"set-av"|"catalog"|"sync-catalog"|"diva"|"divan"|"ticket"|"issue"|"web"|"gost"|"check"|"serve")
                check_auto_update || true
                ;;
        esac
    fi
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "config")
            shift || true
            cmd_config "$@"
            exit 0
            ;;
        "status")
            shift || true
            show_status "$@"
            exit 0
            ;;
        "lint")
            shift || true
            lint_commits "$@"
            exit 0
            ;;
        "cz")
            shift || true
            interactive_commit "$@"
            exit 0
            ;;
        "commit")
            shift || true
            commit "$@"
            exit 0
            ;;
        "install")
            shift || true
            install_cccp "$@"
            exit 0
            ;;
        "completion")
            shift || true
            cmd_completion "$@"
            exit 0
            ;;
        "version")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "version"
                    exit 0
                    ;;
            esac
            generate_version_info
            exit 0
            ;;
        "tag")
            shift || true
            create_tag "$@"
            exit 0
            ;;
        "push")
            shift || true
            cmd_push "$@"
            exit 0
            ;;
        "changelog")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "changelog"
                    exit 0
                    ;;
            esac
            generate_changelog
            exit 0
            ;;
        "commit-msg")
            if [ "${1:-}" = "commit-msg" ]; then
                shift || true
            fi
            commit_msg "$@"
            exit 0
            ;;
        "post-commit")
            post_commit
            exit 0
            ;;
        "pre-push")
            if [ "${1:-}" = "pre-push" ]; then
                shift || true
            fi
            cmd_pre_push "$@"
            exit 0
            ;;
        "reference-transaction")
            if [ "${1:-}" = "reference-transaction" ]; then
                shift || true
            fi
            cmd_reference_transaction "$@"
            exit 0
            ;;
        "update")
            shift || true
            update_script "$@"
            exit 0
            ;;
        "check-update"|"check-updates")
            shift || true
            cmd_check_update "$@"
            exit 0
            ;;
        "hooks")
            shift || true
            cmd_hooks "$@"
            exit 0
            ;;
        "tickets"|"sync-tickets")
            shift || true
            cmd_tickets "$@"
            exit 0
            ;;
        "codeberg")
            shift || true
            cmd_codeberg "$@"
            exit 0
            ;;
        "agent"|"agents"|"update-agent"|"update-agents"|"sync-agent"|"sync-agents")
            shift || true
            cmd_agent "$@"
            exit 0
            ;;
        "set-av"|"set-agent-var")
            shift || true
            cmd_set_av "$@"
            exit 0
            ;;
        "catalog"|"sync-catalog")
            shift || true
            cmd_catalog "$@"
            exit 0
            ;;
        "ticket"|"issue"|"diva"|"divan")
            shift || true
            cmd_ticket "$@"
            exit 0
            ;;
        "web")
            shift || true
            cmd_web "$@"
            exit 0
            ;;
        "comrades"|"camaradas"|"comrade")
            shift || true
            cmd_comrades "$@"
            exit 0
            ;;
        "soviet"|"sputnik"|"anthem"|"gosplan")
            show_soviet
            exit 0
            ;;
        "check")
            shift || true
            cmd_check "$@"
            exit $?
            ;;
        "gost")
            shift || true
            cmd_gost "$@"
            exit $?
            ;;
        "serve")
            shift || true
            cmd_serve "$@"
            exit $?
            ;;
        "repo"|"repos")
            shift || true
            cmd_repo "$@"
            exit $?
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|cz|install|config|status|lint|completion|version|tag|push|tickets|ticket|issue|agent|set-av|catalog|diva|web|comrades|changelog|commit-msg|post-commit|update|check-update|hooks|gost|check|serve|repo|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
            exit 1
            ;;
    esac
}
