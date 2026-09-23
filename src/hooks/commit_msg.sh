#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"

# Set up paths relative to git root
GIT_HOOKS_DIR=$([ -n "$GIT_ROOT" ] && echo "$GIT_ROOT/.git/hooks" || echo "")

# Source the configuration if available
[ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/config/config.sh" ] && . "$GIT_ROOT/src/config/config.sh"

# Function to validate the commit message
commit_msg() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi
    # Check if the hook is active to prevent infinite loops
    # If the hook is active, exit the script
    if [ -n "${HOOK_ACTIVE:-}" ] && [ "$HOOK_ACTIVE" = "1" ]; then
        exit 0
    fi
    local message_file="$1"
    local message
    
    if ! read -r message < "$message_file"; then
        echo "Error: Failed to read commit message" >&2
        return 1
    fi
    
    if ! validate_commit_message "$message"; then
        echo "Error: Invalid commit message" >&2
        return 1
    fi
    
    # Unset the variables
    unset message
    unset message_file
}

