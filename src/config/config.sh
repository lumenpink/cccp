#!/bin/sh

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository"
    exit 1
fi

# Set up paths relative to git root
GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"

# the file name of the script to be used as a git hook
GIT_HOOK_FILE="cccp.sh"

# URL for updating the script
UPDATE_URL="https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh"

# Default base version when no git tags exist in the repository
DEFAULT_BASE_VERSION="0.2.0"

# Commit types following conventional commits specification
COMMIT_TYPES="feat fix perf refactor revert chore build ci docs ops style test merge"

# Allowed scopes for commits
COMMIT_SCOPES="ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build"

# Allowed subscopes for commits
COMMIT_SUBSCOPES="components pages services utils auth models views controllers handlers"

# Allowed types for changelog
CHANGELOG_TYPES="feat fix perf refactor merge"

# Default behavior flags
DISABLE_SUBSCOPES=${DISABLE_SUBSCOPES:-0}
DISABLE_MULTIPLE_SCOPES=${DISABLE_MULTIPLE_SCOPES:-0}
ALLOW_ANY_SUBSCOPE=${ALLOW_ANY_SUBSCOPE:-1}
ALLOW_ANY_SCOPE=${ALLOW_ANY_SCOPE:-1}

# Git hooks configuration
GIT_HOOKS_LIST="commit-msg post-commit"
 