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
# Post-commit hook
# -----------------------------------------------------------------------------
post_commit() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Check if the hook is active to prevent infinite loops
    if [ -n "${HOOK_ACTIVE:-}" ] && [ "$HOOK_ACTIVE" = "1" ]; then
        return 0
    fi
    generate_changelog
    generate_version_info
    # Set the hook active flag to prevent infinite loops
    export HOOK_ACTIVE=1
    git add VERSION CHANGELOG.md
    git commit --amend --no-edit    
    unset HOOK_ACTIVE
    return 0
}