#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

# Enable error handling
set -eu

# Verify required tools
for tool in git sed grep date cut tr; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Error: Required tool '$tool' is not installed or not in PATH." >&2
        echo "Please install $tool to use cccp.sh." >&2
        exit 1
    fi
done

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository. This script must be executed within a valid Git repository." >&2
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
UPDATE_URL="https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh"
DEFAULT_BASE_VERSION="0.2.0"
COMMIT_TYPES="feat fix perf refactor revert chore build ci docs ops style test merge"
COMMIT_SCOPES="ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build"
COMMIT_SUBSCOPES="components pages services utils auth models views controllers handlers"
CHANGELOG_TYPES="feat fix perf refactor merge"
DISABLE_SUBSCOPES=${DISABLE_SUBSCOPES:-0}
DISABLE_MULTIPLE_SCOPES=${DISABLE_MULTIPLE_SCOPES:-0}
ALLOW_ANY_SUBSCOPE=${ALLOW_ANY_SUBSCOPE:-1}
ALLOW_ANY_SCOPE=${ALLOW_ANY_SCOPE:-1}
GIT_HOOKS_LIST="commit-msg post-commit"

# =============================================================================
# Validation Functions
# =============================================================================
check_prerequisites() {
    missing_tools=""
    for tool in git sed grep date cut tr; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools="$missing_tools $tool"
        fi
    done

    if [ -n "$missing_tools" ]; then
        echo "Error: Required system tools are missing from PATH:$missing_tools" >&2
        echo "Please install the missing tools and ensure they are accessible in your PATH." >&2
        return 1
    fi

    # Verify that we are inside a Git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    return 0
}

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

    # Check if header contains a colon
    if ! echo "$commit_msg" | grep -q ":"; then
        echo "Error: Commit message must follow format '<type>(<scope>): <subject>' or '<type>: <subject>'"
        return 1
    fi

    # Extract header line before colon
    header_prefix=$(echo "$commit_msg" | sed -E 's/:.*$//')
    subject=$(echo "$commit_msg" | sed -E 's/^[^:]*:[[:space:]]*//')

    # Detect breaking change marker '!'
    is_breaking=0
    if echo "$header_prefix" | grep -q '!$'; then
        is_breaking=1
        header_prefix="${header_prefix%!}"
    fi

    # Check for empty parentheses e.g. feat():
    if echo "$header_prefix" | grep -Fq '()'; then
        echo "Error: Scope cannot be empty"
        return 1
    fi

    # Extract type and scope
    if echo "$header_prefix" | grep -q "^[^(]*([^)]*)$"; then
        type=$(echo "$header_prefix" | sed -E 's/^([^(]+)\(([^)]*)\)$/\1/')
        scope_part=$(echo "$header_prefix" | sed -E 's/^([^(]+)\(([^)]*)\)$/\2/')
    else
        type="$header_prefix"
        scope_part=""
    fi
    
    # Clean up subject
    subject=$(echo "$subject" | sed -E 's/^[[:space:]]+//')
    
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

    if [ -z "$scope_part" ]; then
        return 0
    fi
    
    # Validate scopes
    OLD_IFS="$IFS"
    IFS=','
    scope_count=0
    for scope_item in $scope_part; do
        scope_count=$((scope_count + 1))
        # Trim leading and trailing whitespace
        scope_item=$(echo "$scope_item" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
        
        if echo "$scope_item" | grep -q "/"; then
            if [ "$DISABLE_SUBSCOPES" = "1" ]; then
                echo "Error: Subscopes are disabled"
                IFS="$OLD_IFS"
                return 1
            fi
            
            scope=$(echo "$scope_item" | cut -d'/' -f1 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            subscope=$(echo "$scope_item" | cut -d'/' -f2 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            
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
                IFS="$OLD_IFS"
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
                IFS="$OLD_IFS"
                return 1
            fi
        else
            scope=$(echo "$scope_item" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            
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
                IFS="$OLD_IFS"
                return 1
            fi
        fi
    done
    
    if [ $scope_count -gt 1 ] && [ "$DISABLE_MULTIPLE_SCOPES" = "1" ]; then
        echo "Error: Multiple scopes are disabled"
        IFS="$OLD_IFS"
        return 1
    fi
    
    IFS="$OLD_IFS"
    return 0
} 


# =============================================================================
# Changelog Functions
# =============================================================================
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

    # Get all tags sorted by version
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
calculate_target_version() {
    last_tag=""
    default_base="${DEFAULT_BASE_VERSION:-0.2.0}"

    # Check if the most recent tag is a valid SemVer
    raw_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$raw_tag" ] && echo "$raw_tag" | grep -qE '^v?[0-9]+\.[0-9]+'; then
        last_tag="$raw_tag"
    else
        # Search for the latest SemVer tag in the repository
        for t in $(git tag -l 'v[0-9]*' '[0-9]*' --sort=-v:refname 2>/dev/null); do
            if echo "$t" | grep -qE '^v?[0-9]+\.[0-9]+'; then
                last_tag="$t"
                break
            fi
        done
    fi

    if [ -z "$last_tag" ]; then
        base_version="$default_base"
        commit_range="HEAD"
    else
        base_version="${last_tag#v}"
        commit_range="$last_tag..HEAD"
    fi

    # Extract major, minor, patch numbers ensuring they are valid integers
    major=$(echo "$base_version" | cut -d. -f1 | tr -cd '0-9')
    minor=$(echo "$base_version" | cut -d. -f2 | tr -cd '0-9')
    patch=$(echo "$base_version" | cut -d. -f3 | cut -d- -f1 | cut -d+ -f1 | tr -cd '0-9')

    major=${major:-0}
    minor=${minor:-0}
    patch=${patch:-0}

    # Predict the next version bump based on conventional commits in range
    has_breaking=0
    has_feat=0

    # Check for breaking changes (BREAKING CHANGE: in footer or ! before colon in header)
    if git log "$commit_range" --format="%s%n%b" 2>/dev/null | grep -qE "(^BREAKING[ -]CHANGE:|^[a-zA-Z]+(\([^)]+\))?!:)"; then
        has_breaking=1
    elif git log "$commit_range" --format="%s" 2>/dev/null | grep -qE "^feat(\([^)]+\))?:"; then
        has_feat=1
    fi

    if [ "$has_breaking" -eq 1 ]; then
        next_major=$((major + 1))
        target_version="${next_major}.0.0"
    elif [ "$has_feat" -eq 1 ]; then
        next_minor=$((minor + 1))
        target_version="${major}.${next_minor}.0"
    else
        next_patch=$((patch + 1))
        target_version="${major}.${minor}.${next_patch}"
    fi

    echo "$target_version"
}

generate_version_info() {
    if command -v check_prerequisites >/dev/null 2>&1; then
        check_prerequisites
    fi

    last_tag=""
    default_base="${DEFAULT_BASE_VERSION:-0.2.0}"

    # Check if the most recent tag is a valid SemVer
    raw_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$raw_tag" ] && echo "$raw_tag" | grep -qE '^v?[0-9]+\.[0-9]+'; then
        last_tag="$raw_tag"
    else
        # Search for the latest SemVer tag in the repository
        for t in $(git tag -l 'v[0-9]*' '[0-9]*' --sort=-v:refname 2>/dev/null); do
            if echo "$t" | grep -qE '^v?[0-9]+\.[0-9]+'; then
                last_tag="$t"
                break
            fi
        done
    fi

    if [ -z "$last_tag" ]; then
        base_version="$default_base"
        commit_range="HEAD"
        commit_count=$(git rev-list --count HEAD 2>/dev/null || echo "0")
    else
        base_version="${last_tag#v}"
        commit_range="$last_tag..HEAD"
        commit_count=$(git rev-list --count "$commit_range" 2>/dev/null || echo "0")
    fi

    # If exactly on a tagged release with no new commits
    if [ -n "$last_tag" ] && [ "$commit_count" -eq 0 ]; then
        final_version="$base_version"
    else
        target_version=$(calculate_target_version)
        current_date=$(date +%Y%m%d)
        current_commit_hash=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

        final_version="${target_version}-dev.${commit_count}+${current_date}.${current_commit_hash}"
    fi

    echo "$final_version" > "$GIT_ROOT/VERSION"
    echo "Version information written to VERSION file: $final_version"
} 


# =============================================================================
# Tag Functions
# =============================================================================
create_tag() {
    if command -v check_prerequisites >/dev/null 2>&1; then
        check_prerequisites
    fi

    # Ensure working tree has no uncommitted changes to tracked files
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        echo "Error: Working directory has uncommitted changes. Please commit or stash them before creating a tag." >&2
        return 1
    fi

    target_ver=""
    no_prefix=0
    message=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --no-v|--no-prefix)
                no_prefix=1
                shift
                ;;
            -m|--message)
                if [ $# -lt 2 ]; then
                    echo "Error: -m option requires a message argument" >&2
                    return 1
                fi
                message="$2"
                shift 2
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "tag"
                else
                    echo "Usage: $0 tag [version] [--no-v] [-m \"message\"]"
                fi
                return 0
                ;;
            -*)
                echo "Error: Unknown option '$1'" >&2
                echo "Run '$0 help tag' for usage." >&2
                return 1
                ;;
            *)
                if [ -z "$target_ver" ]; then
                    target_ver="$1"
                else
                    echo "Error: Unexpected argument '$1'" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    # If no version argument was provided, use the predictive version
    if [ -z "$target_ver" ]; then
        if ! command -v calculate_target_version >/dev/null 2>&1; then
            echo "Error: calculate_target_version function not found" >&2
            return 1
        fi
        clean_ver=$(calculate_target_version)
        if [ -z "$clean_ver" ]; then
            echo "Error: Could not calculate predictive version" >&2
            return 1
        fi
    else
        # Normalize provided version argument: accepts 2, 2.1, 2.1.0, v2, v2.1, v2.1.0
        clean_ver=$(echo "$target_ver" | sed -E 's/^[vV]//')
        if echo "$clean_ver" | grep -qE '^[0-9]+$'; then
            clean_ver="${clean_ver}.0.0"
        elif echo "$clean_ver" | grep -qE '^[0-9]+\.[0-9]+$'; then
            clean_ver="${clean_ver}.0"
        elif echo "$clean_ver" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+'; then
            :
        else
            echo "Error: Invalid version format '$target_ver'. Expected: 2, 2.1, 2.1.0 or v2.1.0" >&2
            return 1
        fi
    fi

    # Determine final tag name with or without 'v' prefix
    if [ "$no_prefix" -eq 1 ]; then
        tag_name="$clean_ver"
    else
        tag_name="v$clean_ver"
    fi

    # Verify if tag already exists in Git
    if git rev-parse -q --verify "refs/tags/$tag_name" >/dev/null 2>&1; then
        echo "Error: Tag '$tag_name' already exists." >&2
        return 1
    fi

    # Set default message if not provided
    if [ -z "$message" ]; then
        message="Release $tag_name"
    fi

    # Update VERSION file with clean SemVer version
    echo "$clean_ver" > "$GIT_ROOT/VERSION"

    # Create temporary tag to allow generate_changelog to group commits under this release
    git tag -a "$tag_name" -m "$message"

    if command -v generate_changelog >/dev/null 2>&1; then
        generate_changelog
    fi

    # Commit the release files (VERSION and CHANGELOG.md)
    export HOOK_ACTIVE=1
    git add "$GIT_ROOT/VERSION" "$GIT_ROOT/CHANGELOG.md"
    if ! git commit -m "chore(release): $tag_name"; then
        unset HOOK_ACTIVE
        echo "Error: Failed to create release commit." >&2
        git tag -d "$tag_name" >/dev/null 2>&1 || true
        return 1
    fi
    unset HOOK_ACTIVE

    # Move tag to the release commit
    git tag -f -a "$tag_name" -m "$message"

    current_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")

    echo "Release commit created: chore(release): $tag_name"
    echo "Tag '$tag_name' created successfully on release commit."
    echo "Updated VERSION: $clean_ver"
    echo "Updated CHANGELOG.md"
    echo ""
    echo "To push the release and tag to remote, run:"
    echo "  git push origin $current_branch"
    echo "  git push origin $tag_name"
}


# =============================================================================
# Hooks Functions
# =============================================================================
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
show_help() {
    target="${1:-}"

    case "$target" in
        "version")
            echo "cccp.sh version - Predictive Semantic Versioning"
            echo "==============================================="
            echo ""
            echo "Calculates and outputs predictive Semantic Versioning (SemVer) metadata"
            echo "based on Git commits and release tags, writing the result to 'VERSION'."
            echo ""
            echo "Usage:"
            echo "  $0 version"
            echo ""
            echo "How it works:"
            echo "  1. Baseline Discovery:"
            echo "     - Scans Git history for the most recent valid SemVer tag (e.g. v1.2.0 or 1.2.0)."
            echo "     - If no tag exists, uses DEFAULT_BASE_VERSION (default: 0.2.0)."
            echo ""
            echo "  2. Commit Inspection (range: <last_tag>..HEAD):"
            echo "     - Evaluates all commit messages following Conventional Commits format:"
            echo "       * MAJOR bump (X+1.0.0): Triggered by breaking change indicators:"
            echo "         - 'BREAKING CHANGE:' or 'BREAKING-CHANGE:' in commit body/footer"
            echo "         - '!' before colon in header (e.g. 'feat!:', 'fix(api)!:')"
            echo "       * MINOR bump (X.Y+1.0): Triggered by feature commits ('feat:' or 'feat(...):')"
            echo "       * PATCH bump (X.Y.Z+1): Triggered by any other conventional commit"
            echo "         (e.g. 'fix:', 'perf:', 'refactor:', 'docs:', 'chore:', etc.)"
            echo ""
            echo "  3. Output Format (in VERSION file):"
            echo "     - Clean release (commit is on a tag with 0 new commits):"
            echo "       <major>.<minor>.<patch> (e.g. 1.2.0)"
            echo "     - Development build (has new commits since last tag):"
            echo "       <target_version>-dev.<commit_count>+<YYYYMMDD>.<short_commit_hash>"
            echo "       Example: 1.3.0-dev.4+20260914.7a3c2b1"
            echo ""
            echo "Related commands:"
            echo "  $0 tag          - Create an annotated Git tag using the predicted version"
            echo "  $0 changelog    - Generate CHANGELOG.md based on conventional commits"
            ;;
        "tag")
            echo "cccp.sh tag - Create Release Tag & Synchronize Version/Changelog"
            echo "================================================================"
            echo ""
            echo "Creates an annotated Git tag, writes the clean version into 'VERSION',"
            echo "and regenerates 'CHANGELOG.md' to reflect the new release."
            echo ""
            echo "Usage:"
            echo "  $0 tag [version] [options]"
            echo ""
            echo "Arguments:"
            echo "  [version]         Target version. Optional. If omitted, uses the"
            echo "                    predictive SemVer version calculated from commits."
            echo "                    Flexible formats accepted:"
            echo "                      2       -> normalized to v2.0.0"
            echo "                      2.1     -> normalized to v2.1.0"
            echo "                      2.1.3   -> normalized to v2.1.3"
            echo "                      v2.1.3  -> normalized to v2.1.3"
            echo ""
            echo "Options:"
            echo "  --no-v            Create tag without 'v' prefix (e.g. 2.1.0 instead of v2.1.0)"
            echo "  -m, --message     Custom tag annotation message (default: 'Release <tag>')"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Actions performed:"
            echo "  1. Ensures working directory is clean (no uncommitted changes)."
            echo "  2. Verifies the tag does not already exist."
            echo "  3. Updates VERSION file with clean SemVer version (e.g. 2.1.0)."
            echo "  4. Regenerates CHANGELOG.md including the new release section."
            echo "  5. Creates release commit 'chore(release): <tag>' containing VERSION & CHANGELOG.md."
            echo "  6. Creates annotated Git tag pointing directly to the release commit."
            echo ""
            echo "Examples:"
            echo "  $0 tag                     # Automatically tag with predicted version (e.g. v0.3.0)"
            echo "  $0 tag 2                   # Creates tag v2.0.0"
            echo "  $0 tag 2.1                 # Creates tag v2.1.0"
            echo "  $0 tag 2.1.0               # Creates tag v2.1.0"
            echo "  $0 tag 2.1.0 --no-v        # Creates tag 2.1.0 (without 'v')"
            echo "  $0 tag 1.0.0 -m 'First GA' # Creates tag with custom annotation"
            ;;
        "commit")
            echo "cccp.sh commit - Conventional Commit Creation"
            echo "============================================="
            echo ""
            echo "Validates and creates a Git commit following the Conventional Commits specification."
            echo ""
            echo "Usage:"
            echo "  $0 commit <message>"
            echo ""
            echo "Format:"
            echo "  <type>(<scope>): <subject>"
            echo "  <type>(<scope>/<subscope>): <subject>"
            echo "  <type>!: <subject>  (Breaking Change)"
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
            echo "Examples:"
            echo "  $0 commit 'feat(ui): add dark mode switch'"
            echo "  $0 commit 'fix(api/auth): handle expired tokens'"
            echo "  $0 commit 'feat(core)!: drop support for legacy protocols'"
            ;;
        "changelog")
            echo "cccp.sh changelog - Conventional Changelog Generation"
            echo "===================================================="
            echo ""
            echo "Parses conventional commits in the Git history and generates or updates CHANGELOG.md."
            echo ""
            echo "Usage:"
            echo "  $0 changelog"
            echo ""
            echo "How it works:"
            echo "  1. Groups unreleased commits under '## [Unreleased]'."
            echo "  2. Groups commits between tags under each '### [<tag>]' release header."
            echo "  3. Categorizes commits into:"
            echo "     - Features (feat)"
            echo "     - Bug Fixes (fix)"
            echo "     - Performance Improvements (perf)"
            ;;
        "install")
            echo "cccp.sh install - Install Git Hooks"
            echo "==================================="
            echo ""
            echo "Installs Git hooks in '.git/hooks' to automate validation and release metadata:"
            echo ""
            echo "Usage:"
            echo "  $0 install"
            echo ""
            echo "Hooks installed:"
            echo "  - commit-msg:  Validates commit message format against Conventional Commits."
            echo "  - post-commit: Automatically runs 'changelog' and 'version' after each commit."
            ;;
        "update")
            echo "cccp.sh update - Update Script"
            echo "=============================="
            echo ""
            echo "Downloads and replaces the current cccp.sh script with the latest version"
            echo "from the upstream GitHub repository."
            echo ""
            echo "Usage:"
            echo "  $0 update"
            ;;
        *)
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
            echo "  tag [version]      Create release tag, update VERSION and CHANGELOG"
            echo "  changelog          Generate or update CHANGELOG.md"
            echo "  update             Update the script to the latest version"
            echo "  help [command]     Show this help message or deep help on a command"
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
            echo "  $0 tag 2.1.0"
            echo "  $0 changelog"
            echo "  $0 update"
            echo "  $0 help version"
            echo ""
            echo "Note: After installation, git hooks will automatically validate commit messages"
            echo "and update the changelog and version information after each commit."
            echo ""
            echo "For in-depth help on any command, run:"
            echo "  $0 help <command> (e.g. $0 help version, $0 help tag)"
            ;;
    esac
}


# =============================================================================
# Commit Functions
# =============================================================================
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
# Update Functions
# =============================================================================
update_script() {
    echo "Updating cccp.sh from $UPDATE_URL..."
    
    # Download the new script
    if ! wget -q "$UPDATE_URL" -O "$GIT_ROOT/cccp.sh.new"; then
        echo "Error: Failed to download the new script"
        return 1
    fi
    
    # Make the new script executable
    chmod +x "$GIT_ROOT/cccp.sh.new"
    
    # Backup the current script
    if [ -f "$GIT_ROOT/cccp.sh" ]; then
        mv "$GIT_ROOT/cccp.sh" "$GIT_ROOT/cccp.sh.bak"
    fi
    
    # Replace the current script with the new one
    mv "$GIT_ROOT/cccp.sh.new" "$GIT_ROOT/cccp.sh"
    
    echo "Successfully updated cccp.sh"
    echo "A backup of your previous version was saved as cccp.sh.bak"
    
    return 0
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
            case "${2:-}" in
                "-h"|"--help")
                    show_help "commit"
                    exit 0
                    ;;
            esac
            commit "$2"
            exit 0
            ;;
        "install")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "install"
                    exit 0
                    ;;
            esac
            install_git_hooks
            ;;
        "version")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "version"
                    exit 0
                    ;;
            esac
            generate_version_info
            exit 0
            ;;
        "tag")
            shift || true
            create_tag "$@"
            exit 0
            ;;
        "changelog")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "changelog"
                    exit 0
                    ;;
            esac
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
        "update")
            case "${2:-}" in
                "-h"|"--help")
                    show_help "update"
                    exit 0
                    ;;
            esac
            update_script
            exit 0
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|install|version|tag|changelog|commit-msg|post-commit|update|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
            exit 1
            ;;
    esac
}

# Execute main function
main "$@"
