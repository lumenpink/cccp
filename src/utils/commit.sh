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
fi

# -----------------------------------------------------------------------------
# Commit changes
# -----------------------------------------------------------------------------
commit() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    message=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --strict-scopes)
                STRICT_SCOPES=1
                ALLOW_ANY_SCOPE=0
                shift
                ;;
            --allow-any-scope)
                STRICT_SCOPES=0
                ALLOW_ANY_SCOPE=1
                shift
                ;;
            --strict-subscopes)
                STRICT_SUBSCOPES=1
                ALLOW_ANY_SUBSCOPE=0
                shift
                ;;
            --allow-any-subscope)
                STRICT_SUBSCOPES=0
                ALLOW_ANY_SUBSCOPE=1
                shift
                ;;
            --disable-subscopes)
                DISABLE_SUBSCOPES=1
                shift
                ;;
            --enable-subscopes)
                DISABLE_SUBSCOPES=0
                shift
                ;;
            --disable-multiple-scopes)
                DISABLE_MULTIPLE_SCOPES=1
                shift
                ;;
            --enable-multiple-scopes)
                DISABLE_MULTIPLE_SCOPES=0
                shift
                ;;
            -i|--interactive)
                interactive_mode=1
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "commit"
                else
                    echo "Usage: $0 commit [options] <message>"
                fi
                return 0
                ;;
            *)
                if [ -z "$message" ]; then
                    message="$1"
                else
                    echo "Error: Unexpected argument '$1'" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    # If interactive mode requested, launch wizard
    if [ "${interactive_mode:-0}" = "1" ]; then
        if command -v interactive_commit >/dev/null 2>&1; then
            interactive_commit
            return $?
        else
            echo "Error: Interactive wizard is not available" >&2
            return 1
        fi
    fi

    # If no message provided, show usage
    if [ -z "$message" ]; then
        echo "Usage: $0 commit [options] <message>"
        echo "Example: $0 commit 'feat(ui): add new button'"
        return 1
    fi

    # Validate the commit message
    if ! validate_commit_message "$message"; then
        return 1
    fi

    # Commit the changes
    git commit -m "$message"
}