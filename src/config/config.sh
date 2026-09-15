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
DEFAULT_BASE_VERSION="${DEFAULT_BASE_VERSION:-0.0.1}"

# Commit types following conventional commits specification
COMMIT_TYPES="feat fix perf refactor revert chore build ci docs ops style test merge"

# Allowed scopes for commits
COMMIT_SCOPES="ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build"

# Allowed subscopes for commits
COMMIT_SUBSCOPES="components pages services utils auth models views controllers handlers"

# Allowed types for changelog
CHANGELOG_TYPES="feat fix perf refactor merge"

# Default behavior flags (Canonical POSIX/UNIX design: all default to 0)
# STRICT_SCOPES=1: Require commit scope to match COMMIT_SCOPES list.
# STRICT_SUBSCOPES=1: Require commit subscope to match COMMIT_SUBSCOPES list.
# DISABLE_SUBSCOPES=1: Prohibit slash-delimited subscopes.
# DISABLE_MULTIPLE_SCOPES=1: Prohibit comma-separated multiple scopes.
STRICT_SCOPES=${STRICT_SCOPES:-0}
STRICT_SUBSCOPES=${STRICT_SUBSCOPES:-0}
DISABLE_SUBSCOPES=${DISABLE_SUBSCOPES:-0}
DISABLE_MULTIPLE_SCOPES=${DISABLE_MULTIPLE_SCOPES:-0}

# Backward Compatibility for deprecated flags (ALLOW_ANY_SCOPE / ALLOW_ANY_SUBSCOPE):
# If legacy ALLOW_ANY_* was set to 0, map to STRICT_*=1.
if [ "${ALLOW_ANY_SCOPE:-1}" = "0" ]; then
    STRICT_SCOPES=1
fi
if [ "${ALLOW_ANY_SUBSCOPE:-1}" = "0" ]; then
    STRICT_SUBSCOPES=1
fi
# Mirror legacy variables for external consumers
ALLOW_ANY_SCOPE=$([ "$STRICT_SCOPES" = "1" ] && echo "0" || echo "1")
ALLOW_ANY_SUBSCOPE=$([ "$STRICT_SUBSCOPES" = "1" ] && echo "0" || echo "1")

# Git hooks configuration
GIT_HOOKS_LIST="commit-msg post-commit"
 