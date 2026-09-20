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
    if [ -f "$GIT_ROOT/src/hooks/hooks.sh" ]; then
        . "$GIT_ROOT/src/hooks/hooks.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Execute git push and trigger post-push hook
# -----------------------------------------------------------------------------
cmd_push() {
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    for arg in "$@"; do
        case "$arg" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "push"
                else
                    echo "Usage: cccp push [git push arguments...]"
                fi
                return 0
                ;;
        esac
    done

    # Reset run flag
    export CCCP_POST_PUSH_RAN=0

    # Determine branch and tracking remote before push for fallback hook execution
    curr_branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "main")
    curr_remote=$(git config "branch.${curr_branch}.remote" 2>/dev/null || echo "origin")
    old_remote_sha=$(git rev-parse "refs/remotes/${curr_remote}/${curr_branch}" 2>/dev/null || echo "0000000000000000000000000000000000000000")

    # Execute standard git push
    git push "$@"

    # If reference-transaction hook already ran the post-push hook, finish
    if [ "${CCCP_POST_PUSH_RAN:-0}" = "1" ]; then
        return 0
    fi

    # Fallback execution (e.g. if git hooks are not installed in this repository)
    new_remote_sha=$(git rev-parse "refs/remotes/${curr_remote}/${curr_branch}" 2>/dev/null || git rev-parse HEAD 2>/dev/null || echo "")
    if command -v run_post_push_hook >/dev/null 2>&1; then
        run_post_push_hook "$curr_remote" "$curr_branch" "$old_remote_sha" "$new_remote_sha" || true
    fi
    return 0
}
