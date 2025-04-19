#!/bin/sh

# Enable error handling
set -eu

# =============================================================================
# Git Conventional Commits Helper Script
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository"
    exit 1
fi

# Source all utility files
. "$GIT_ROOT/src/config/config.sh"
. "$GIT_ROOT/src/utils/validation.sh"
. "$GIT_ROOT/src/utils/changelog.sh"
. "$GIT_ROOT/src/utils/version.sh"
. "$GIT_ROOT/src/hooks/hooks.sh"
. "$GIT_ROOT/src/hooks/commit_msg.sh"
. "$GIT_ROOT/src/hooks/post_commit.sh"
. "$GIT_ROOT/src/utils/help.sh"

# -----------------------------------------------------------------------------
# Main script entry point
# -----------------------------------------------------------------------------
main() {
    if [ -n "${1:-}" ]; then
       command="$1"
    else
       command=""
    fi
    current_hook=$(basename "$0")
    
    # Check if running as a hook
    case " $GIT_HOOKS_LIST " in
        *" $current_hook "*)
            echo "Running hook: $current_hook"
            command="$current_hook"
            ;;
    esac
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "commit")
            commit "$2"
            exit 0
            ;;
        "install")
            install_git_hooks
            ;;
        "version")
            generate_version_info
            exit 0
            ;;
        "changelog")
            generate_changelog
            exit 0
            ;;
        "help")
            show_help
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
        *)
            show_help
            exit 1
            ;;
    esac
}

# Execute main function
main "$@" 