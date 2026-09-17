#!/bin/sh

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
GIT_HOOKS_DIR=$([ -n "$GIT_ROOT" ] && echo "$GIT_ROOT/.git/hooks" || echo "")

# CCCP Tool Version (Synchronized from VERSION file)
if [ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/VERSION" ]; then
    CCCP_VERSION="${CCCP_VERSION:-$(head -n 1 "$GIT_ROOT/VERSION" | tr -d ' \r\n')}"
else
    CCCP_VERSION="${CCCP_VERSION:-2.0.0}"
fi
export CCCP_VERSION

# The file name of the script to be used as a git hook
GIT_HOOK_FILE="cccp.sh"

# URL for updating the script
UPDATE_URL="https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh"

# Commit types following conventional commits specification
COMMIT_TYPES="${COMMIT_TYPES:-feat fix perf refactor revert chore build ci docs ops style test merge}"

# Allowed scopes for commits
COMMIT_SCOPES="${COMMIT_SCOPES:-ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build}"

# Allowed subscopes for commits
COMMIT_SUBSCOPES="${COMMIT_SUBSCOPES:-components pages services utils auth models views controllers handlers}"

# Allowed types for changelog
CHANGELOG_TYPES="feat fix perf refactor merge"

# Git hooks configuration
GIT_HOOKS_LIST="commit-msg post-commit"

# Scope and subscope policy flags
DISABLE_SUBSCOPES="${DISABLE_SUBSCOPES:-0}"
DISABLE_MULTIPLE_SCOPES="${DISABLE_MULTIPLE_SCOPES:-0}"
STRICT_SCOPES="${STRICT_SCOPES:-0}"
STRICT_SUBSCOPES="${STRICT_SUBSCOPES:-0}"
STRICT_TYPES="${STRICT_TYPES:-1}"
ALLOW_ANY_SCOPE="${ALLOW_ANY_SCOPE:-1}"
ALLOW_ANY_SUBSCOPE="${ALLOW_ANY_SUBSCOPE:-1}"

# Source hierarchical config manager if available
[ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/utils/config_manager.sh" ] && . "$GIT_ROOT/src/utils/config_manager.sh"

# Load hierarchical configuration if manager is loaded
command -v load_hierarchical_config >/dev/null 2>&1 && load_hierarchical_config || true