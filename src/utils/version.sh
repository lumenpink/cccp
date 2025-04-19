#!/bin/sh

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
# Generate version information
# -----------------------------------------------------------------------------
generate_version_info() {
    last_tag=$(git describe --tags --abbrev=0 --always)
    commit_count=$(git rev-list --count $last_tag..HEAD)
    current_date=$(date +%Y%m%d)
    # Get the short hash of the second-to-last commit
    second_to_last_commit_hash=$(git log -n 2 --format=%h | tail -n 1)    
    echo "${last_tag}+${commit_count}.${current_date}.${second_to_last_commit_hash}" > VERSION
    echo "Version information written to VERSION file"
} 