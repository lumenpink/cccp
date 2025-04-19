#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository"
    exit 1
fi

# Set up paths relative to git root
GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"

# Source the configuration
. "$GIT_ROOT/src/config/config.sh"

# -----------------------------------------------------------------------------
# Commit changes
# -----------------------------------------------------------------------------
commit() {
    message="$1"
    
    # If no message provided, show usage
    if [ -z "$message" ]; then
        echo "Usage: $0 commit <message>"
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