#!/bin/sh

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
GIT_HOOKS_DIR=$([ -n "$GIT_ROOT" ] && echo "$GIT_ROOT/.git/hooks" || echo "")

# The file name of the script to be used as a git hook
GIT_HOOK_FILE="cccp.sh"

# URL for updating the script
UPDATE_URL="https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh"

# Commit types following conventional commits specification
COMMIT_TYPES="feat fix perf refactor revert chore build ci docs ops style test merge"

# Allowed scopes for commits
COMMIT_SCOPES="ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build"

# Allowed subscopes for commits
COMMIT_SUBSCOPES="components pages services utils auth models views controllers handlers"

# Allowed types for changelog
CHANGELOG_TYPES="feat fix perf refactor merge"

# Git hooks configuration
GIT_HOOKS_LIST="commit-msg post-commit"

# Source hierarchical config manager if available
[ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/utils/config_manager.sh" ] && . "$GIT_ROOT/src/utils/config_manager.sh"

# Load hierarchical configuration if manager is loaded
command -v load_hierarchical_config >/dev/null 2>&1 && load_hierarchical_config