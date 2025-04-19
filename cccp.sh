#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

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

# =============================================================================
# Configuration
# =============================================================================
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository"
    exit 1
fi
GIT_HOOK_FILE="cccp.sh"
COMMIT_TYPES="feat fix perf refactor revert chore build ci docs ops style test merge"
COMMIT_SCOPES="ui docs api docker db"
COMMIT_SUBSCOPES="components pages services utils auth"
CHANGELOG_TYPES="feat fix perf refactor merge"
DISABLE_SUBSCOPES=${DISABLE_SUBSCOPES:-0}
DISABLE_MULTIPLE_SCOPES=${DISABLE_MULTIPLE_SCOPES:-0}
ALLOW_ANY_SUBSCOPE=${ALLOW_ANY_SUBSCOPE:-0}
ALLOW_ANY_SCOPE=${ALLOW_ANY_SCOPE:-0}
GIT_HOOKS_LIST="commit-msg post-commit" 

# =============================================================================
# Validation Functions
# =============================================================================
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
# Validate commit message format
# -----------------------------------------------------------------------------
validate_commit_message() {
    commit_msg="$1"
    type=""
    scope_part=""
    subject=""
    
    # Check for empty commit message
    if [ -z "$commit_msg" ]; then
        echo "Error: Commit message can't be empty"
        return 1
    fi
    
    # Extract type, scope, and subject from commit message
    if echo "$commit_msg" | grep -q "^[^:]*([^)]*):"; then
        type=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\1/')
        scope_part=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\2/')
        subject=$(echo "$commit_msg" | sed -E 's/^([^(]+)\(([^)]*)\):(.*)$/\3/')
    else
        type=$(echo "$commit_msg" | sed -E 's/^([^:]*):(.*)$/\1/')
        scope_part=""
        subject=$(echo "$commit_msg" | sed -E 's/^([^:]*):(.*)$/\2/')
    fi
    
    # Clean up subject
    subject=$(echo "$subject" | sed -E 's/^[ ]+//')
    
    # Validate type
    valid_type=0
    for t in $COMMIT_TYPES; do
        if [ "$type" = "$t" ]; then
            valid_type=1
            break
        fi
    done
    
    if [ $valid_type -eq 0 ]; then
        echo "Error: Invalid type '$type'. Must be one of: $COMMIT_TYPES"
        return 1
    fi
    
    if [ -z "$subject" ]; then
        echo "Error: Commit message must have a subject"
        return 1
    fi
    
    # Check for empty parentheses
    if echo "$commit_msg" | grep -q "^[^:]*():"; then
        echo "Error: Scope cannot be empty"
        return 1
    fi
    
    if [ -z "$scope_part" ]; then
        return 0
    fi
    
    # Validate scopes
    OLD_IFS="$IFS"
    IFS=','
    scope_count=0
    for scope_item in $scope_part; do
        scope_count=$((scope_count + 1))
        
        if echo "$scope_item" | grep -q "/"; then
            if [ "$DISABLE_SUBSCOPES" = "1" ]; then
                echo "Error: Subscopes are disabled"
                return 1
            fi
            
            scope=$(echo "$scope_item" | cut -d'/' -f1)
            subscope=$(echo "$scope_item" | cut -d'/' -f2)
            
            # Validate scope
            valid_scope=0
            if [ "$ALLOW_ANY_SCOPE" = "1" ]; then
                valid_scope=1
            else
                IFS=" "
                for s in $COMMIT_SCOPES; do
                    if [ "$scope" = "$s" ]; then
                        valid_scope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_scope -eq 0 ]; then
                echo "Error: Invalid scope '$scope'. Must be one of: $COMMIT_SCOPES"
                return 1
            fi
            
            # Validate subscope
            valid_subscope=0
            if [ "$ALLOW_ANY_SUBSCOPE" = "1" ]; then
                valid_subscope=1
            else
                IFS=" "
                for ss in $COMMIT_SUBSCOPES; do
                    if [ "$subscope" = "$ss" ]; then
                        valid_subscope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_subscope -eq 0 ]; then
                echo "Error: Invalid subscope '$subscope'. Must be one of: $COMMIT_SUBSCOPES"
                return 1
            fi
        else
            scope=$scope_item
            
            valid_scope=0
            if [ "$ALLOW_ANY_SCOPE" = "1" ]; then
                valid_scope=1
            else
                IFS=" "
                for s in $COMMIT_SCOPES; do
                    if [ "$scope" = "$s" ]; then
                        valid_scope=1
                        break
                    fi
                done
                IFS=","
            fi
            
            if [ $valid_scope -eq 0 ]; then
                echo "Error: Invalid scope '$scope'. Must be one of: $COMMIT_SCOPES"
                return 1
            fi
        fi
    done
    
    if [ $scope_count -gt 1 ] && [ "$DISABLE_MULTIPLE_SCOPES" = "1" ]; then
        echo "Error: Multiple scopes are disabled"
        return 1
    fi
    
    IFS="$OLD_IFS"
    return 0
} 


# =============================================================================
# Changelog Functions
# =============================================================================
set -eu

# Get the git root directory
GIT_ROOT=$(git rev-parse --show-toplevel)
if [ $? -ne 0 ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Source the configuration file
. "$GIT_ROOT/src/config/config.sh"

# Function to format commit message with scope
format_commit_message() {
    local msg="$1"
    # Extract scope and message, then format with scope in parentheses
    if echo "$msg" | grep -q "("; then
        # Has scope
        local scope=$(echo "$msg" | sed -E 's/^[a-z]+\(([^)]+)\):.*/\1/')
        local message=$(echo "$msg" | sed -E 's/^[a-z]+\([^)]+\): (.*)/\1/')
        echo "  - ($scope) $message"
    else
        # No scope
        echo "$msg" | sed -E 's/^[a-z]+: /  - /'
    fi
}

generate_changelog() {
    local changelog_file="CHANGELOG.md"

    # Get all commits since the last tag
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    local commits
    if [ -n "$last_tag" ]; then
        commits=$(git log "$last_tag"..HEAD --pretty=format:"%s")
    else
        commits=$(git log --pretty=format:"%s")
    fi

    # Extract commits by type
    local feat_commits=$(echo "$commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
    local fix_commits=$(echo "$commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
    local perf_commits=$(echo "$commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")

    # Initialize changelog file with header and unreleased section
    {
        echo "# Changelog"
        echo
        echo "## [Unreleased]"
        echo
        echo "### Features"
        
        # Add feature commits
        if [ -n "$feat_commits" ]; then
            echo "$feat_commits"
        fi
        
        echo
        echo "### Bug Fixes"
        
        # Add bug fix commits
        if [ -n "$fix_commits" ]; then
            echo "$fix_commits"
        fi
        
        echo
        echo "### Performance Improvements"
        
        # Add performance improvement commits
        if [ -n "$perf_commits" ]; then
            echo "$perf_commits"
        fi
        
        echo
        echo "## Previous Releases"
        echo
    } > "$changelog_file"

    local tags=$(git tag -l --sort=-v:refname)
    if [ -n "$tags" ]; then
        local prev_tag=""
        for tag in $tags; do
            if [ -n "$prev_tag" ]; then
                # Get commits between tags
                local tag_commits=$(git log "$tag..$prev_tag" --pretty=format:"%s")
                
                # Extract commits by type for this tag range
                local tag_feat_commits=$(echo "$tag_commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                local tag_fix_commits=$(echo "$tag_commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                local tag_perf_commits=$(echo "$tag_commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")
                
                # Add tag section
                {
                    echo "### [$tag]"
                    echo
                    echo "### Features"
                    
                    # Add feature commits
                    if [ -n "$tag_feat_commits" ]; then
                        echo "$tag_feat_commits"
                    fi
                    
                    echo
                    echo "### Bug Fixes"
                    
                    # Add bug fix commits
                    if [ -n "$tag_fix_commits" ]; then
                        echo "$tag_fix_commits"
                    fi
                    
                    echo
                    echo "### Performance Improvements"
                    
                    # Add performance improvement commits
                    if [ -n "$tag_perf_commits" ]; then
                        echo "$tag_perf_commits"
                    fi
                    
                    echo
                } >> "$changelog_file"

            fi
            prev_tag="$tag"
        done

        # Handle the last tag
        if [ -n "$prev_tag" ]; then
            # Get commits before the first tag
            local first_commits=$(git log "$prev_tag" --pretty=format:"%s")
            
            # Extract commits by type for the first tag
            local first_feat_commits=$(echo "$first_commits" | grep "^feat" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            local first_fix_commits=$(echo "$first_commits" | grep "^fix" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            local first_perf_commits=$(echo "$first_commits" | grep "^perf" | while read -r commit; do format_commit_message "$commit"; done || echo "")
            
            # Add the first tag section
            {
                echo "### [$prev_tag]"
                echo
                echo "### Features"
                
                # Add feature commits
                if [ -n "$first_feat_commits" ]; then
                    echo "$first_feat_commits"
                fi
                
                echo
                echo "### Bug Fixes"
                
                # Add bug fix commits
                if [ -n "$first_fix_commits" ]; then
                    echo "$first_fix_commits"
                fi
                
                echo
                echo "### Performance Improvements"
                
                # Add performance improvement commits
                if [ -n "$first_perf_commits" ]; then
                    echo "$first_perf_commits"
                fi
            } >> "$changelog_file"

        fi
    fi
} 

# =============================================================================
# Version Functions
# =============================================================================
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


# =============================================================================
# Hooks Functions
# =============================================================================
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Source the configuration
. "$GIT_ROOT/src/config/config.sh"

# -----------------------------------------------------------------------------
# Install git hooks
# -----------------------------------------------------------------------------
install_git_hooks() {
    # Create hooks directory if it doesn't exist
    mkdir -p "$GIT_HOOKS_DIR"

    # Configure git to use the local hooks directory
    git config --global core.hooksPath "$GIT_HOOKS_DIR"

    # Process each hook in the list
    for hook in $GIT_HOOKS_LIST; do
        # Check if the hook is already a symlink to our script
        if [ -L "$GIT_HOOKS_DIR/$hook" ] && [ "$(readlink "$GIT_HOOKS_DIR/$hook")" = "$GIT_ROOT/$GIT_HOOK_FILE" ]; then
            echo "Hook already installed: $GIT_HOOKS_DIR/$hook"
            continue
        fi
        
        # Handle existing hook (whether it's a file or a symlink)
        if [ -e "$GIT_HOOKS_DIR/$hook" ] || [ -L "$GIT_HOOKS_DIR/$hook" ]; then
            # Find an available backup name for the existing hook
            backup_name="$GIT_HOOKS_DIR/$hook.old"
            counter=1
            
            while [ -e "$backup_name" ] || [ -L "$backup_name" ]; do
                backup_name="$GIT_HOOKS_DIR/$hook.old.$counter"
                counter=$((counter + 1))
            done
            
            # Backup existing hook
            mv "$GIT_HOOKS_DIR/$hook" "$backup_name"
            echo "Backed up existing hook to: $backup_name"
        fi

        # Create symbolic link to our script        
        ln -s "$GIT_ROOT/$GIT_HOOK_FILE" "$GIT_HOOKS_DIR/$hook"
    done

    echo "Successfully installed git hooks!"
    echo "You can now use 'git cc' to commit your changes."
} 


# =============================================================================
# Commit Message Hook Function
# =============================================================================
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Set up paths relative to git root
GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"

# Source the configuration
. "$GIT_ROOT/src/config/config.sh"

# Function to validate the commit message
commit_msg() {
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


# =============================================================================
# Post Commit Hook Function
# =============================================================================
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
# Post-commit hook
# -----------------------------------------------------------------------------
post_commit() {
    # Check if the hook is active to prevent infinite loops
    # If the hook is active, exit the script
    if [ -n "${HOOK_ACTIVE:-}" ] && [ "$HOOK_ACTIVE" = "1" ]; then
        exit 0
    fi
    generate_changelog
    generate_version_info
    # Set the hook active flag to prevent infinite loops
    export HOOK_ACTIVE=1
    git add VERSION CHANGELOG.md
    git commit --amend --no-edit    
    unset HOOK_ACTIVE
}


# =============================================================================
# Help Functions
# =============================================================================
set -eu

# =============================================================================
# Help Functions
# =============================================================================
# -----------------------------------------------------------------------------
# Display help information
# -----------------------------------------------------------------------------
show_help() {
    echo "Git Conventional Commits Helper Script"
    echo "====================================="
    echo ""
    echo "This script provides tools for managing git commits following conventional commit"
    echo "standards, version management, and changelog generation."
    echo ""
    echo "Usage:"
    echo "  $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  commit <message>    Create a commit with a conventional commit message"
    echo "  install            Install git hooks for commit message validation"
    echo "  version            Generate version information file"
    echo "  changelog          Generate or update CHANGELOG.md"
    echo "  help               Show this help message"
    echo ""
    echo "Git Hooks:"
    echo "  commit-msg         Validates commit messages for conventional commit format"
    echo "  post-commit        Automatically updates changelog and version after commit"
    echo ""
    echo "Commit Message Format:"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo "Types:"
    echo "  feat     - New feature"
    echo "  fix      - Bug fix"
    echo "  perf     - Performance improvement"
    echo "  refactor - Code refactoring"
    echo "  revert   - Revert changes"
    echo "  chore    - Maintenance tasks"
    echo "  build    - Build system changes"
    echo "  ci       - CI configuration changes"
    echo "  docs     - Documentation changes"
    echo "  ops      - Operational changes"
    echo "  style    - Code style changes"
    echo "  test     - Test related changes"
    echo "  merge    - Merge commits"
    echo ""
    echo "Scopes:"
    echo "  ui       - User interface changes"
    echo "  docs     - Documentation changes"
    echo "  api      - API changes"
    echo "  docker   - Docker related changes"
    echo "  db       - Database changes"
    echo ""
    echo "Subscopes:"
    echo "  components - UI components"
    echo "  pages      - Page components"
    echo "  services   - Service layer"
    echo "  utils      - Utility functions"
    echo "  auth       - Authentication related"
    echo ""
    echo "Environment Variables:"
    echo "  DISABLE_SUBSCOPES         - Set to 1 to disable subscopes"
    echo "  DISABLE_MULTIPLE_SCOPES   - Set to 1 to disable multiple scopes"
    echo "  ALLOW_ANY_SUBSCOPE        - Set to 1 to allow any subscope"
    echo "  ALLOW_ANY_SCOPE           - Set to 1 to allow any scope"
    echo ""
    echo "Examples:"
    echo "  $0 commit 'feat(ui): add new button'"
    echo "  $0 commit 'fix(api/auth): resolve login issue'"
    echo "  $0 install"
    echo "  $0 version"
    echo "  $0 changelog"
    echo ""
    echo "Note: After installation, git hooks will automatically validate commit messages"
    echo "and update the changelog and version information after each commit."
} 


# =============================================================================
# Commit Functions
# =============================================================================
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


# =============================================================================
# Main script entry point
# =============================================================================
main() {
    if [ -n "${1:-}" ]; then
       command="$1"
    else
       command=""
    fi
    current_hook=$(basename "$0")
    
    # Check if running as a hook
    case " $GIT_HOOKS_LIST " in
        *" $current_hook "*)
            command="$current_hook"
            ;;
    esac
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "commit")
            commit "$2"
            exit 0
            ;;
        "install")
            install_git_hooks
            ;;
        "version")
            generate_version_info
            exit 0
            ;;
        "changelog")
            generate_changelog
            exit 0
            ;;
        "commit-msg")
            commit_msg "$@"
            exit 0
            ;;
        "post-commit")
            post_commit
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|install|version|changelog|commit-msg|post-commit]"
            exit 1
            ;;
    esac
}

# Execute main function
main "$@"
