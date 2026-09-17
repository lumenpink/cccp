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
    case " ${GIT_HOOKS_LIST:-commit-msg post-commit} " in
        *" $current_hook "*)
            command="$current_hook"
            ;;
    esac

    # Hook version audit in git repositories
    if [ -n "${GIT_ROOT:-}" ] && command -v check_hook_version >/dev/null 2>&1; then
        case "$command" in
            "commit"|"commit-msg"|"post-commit")
                check_hook_version || true
                ;;
        esac
    fi

    # Verify required system tools for operational commands
    if command -v check_system_tools >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"changelog"|"config"|"status"|"lint"|"commit-msg"|"post-commit")
                check_system_tools || exit 1
                ;;
        esac
    fi

    # Non-blocking periodic update check on interactive user commands
    if command -v check_auto_update >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"changelog"|"config"|"status"|"lint"|"completion")
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
            commit_msg "$@"
            exit 0
            ;;
        "post-commit")
            post_commit
            exit 0
            ;;
        "update")
            shift || true
            update_script "$@"
            exit 0
            ;;
        "soviet"|"sputnik"|"anthem"|"gosplan")
            show_soviet
            exit 0
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|cz|install|config|status|lint|completion|version|tag|changelog|commit-msg|post-commit|update|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
            exit 1
            ;;
    esac
}
