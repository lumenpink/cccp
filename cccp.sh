#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

# Enable error handling
set -eu

# Script Version (Single Source of Truth, synchronized from VERSION)
CCCP_VERSION="2.3.0-dev.22+20260920.9241ca0"
export CCCP_VERSION

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
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
else
    GIT_HOOKS_DIR=""
fi

# =============================================================================
# Configuration
# =============================================================================
if [ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/VERSION" ]; then
    CCCP_VERSION="${CCCP_VERSION:-$(head -n 1 "$GIT_ROOT/VERSION" | tr -d ' \r\n')}"
else
    CCCP_VERSION="${CCCP_VERSION:-2.0.0}"
fi
export CCCP_VERSION
GIT_HOOK_FILE="cccp.sh"
UPDATE_URL="https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh"
COMMIT_TYPES="${COMMIT_TYPES:-feat fix perf refactor revert chore build ci docs ops style test merge}"
COMMIT_SCOPES="${COMMIT_SCOPES:-ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build}"
COMMIT_SUBSCOPES="${COMMIT_SUBSCOPES:-components pages services utils auth models views controllers handlers}"
CHANGELOG_TYPES="feat fix perf refactor merge"
GIT_HOOKS_LIST="commit-msg post-commit pre-push reference-transaction"
DISABLE_SUBSCOPES="${DISABLE_SUBSCOPES:-0}"
DISABLE_MULTIPLE_SCOPES="${DISABLE_MULTIPLE_SCOPES:-0}"
STRICT_SCOPES="${STRICT_SCOPES:-0}"
STRICT_SUBSCOPES="${STRICT_SUBSCOPES:-0}"
STRICT_TYPES="${STRICT_TYPES:-1}"
ALLOW_ANY_SCOPE="${ALLOW_ANY_SCOPE:-1}"
ALLOW_ANY_SUBSCOPE="${ALLOW_ANY_SUBSCOPE:-1}"
POST_PUSH_HOOK="${POST_PUSH_HOOK:-}"
POST_PUSH_SYNC="${POST_PUSH_SYNC:-}"
POST_PUSH_SYNC_TICKETS="${POST_PUSH_SYNC_TICKETS:-0}"
POST_PUSH_SYNC_AGENTS="${POST_PUSH_SYNC_AGENTS:-0}"
TICKETS_FILE="${TICKETS_FILE:-}"
AGENTS_MD_URL="${AGENTS_MD_URL:-https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md}"
AGENTS_MD_FILE="${AGENTS_MD_FILE:-}"

# =============================================================================
# Validation Functions
# =============================================================================
check_system_tools() {
    missing_tools=""
    for tool in git sed grep date cut tr; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools="$missing_tools $tool"
        fi
    done

    if [ -n "$missing_tools" ]; then
        echo "Error: Required system tools are missing from PATH:$missing_tools" >&2
        echo "Gosplan Quality Control: Please install the missing tools and ensure they are accessible in your PATH." >&2
        return 1
    fi

    return 0
}

check_git_repo() {
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi
    return 0
}

check_prerequisites() {
    check_system_tools || return 1
    check_git_repo || return 1
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

    # Extract header line (first line of commit message)
    header_line=$(echo "$commit_msg" | head -n 1)

    # Check if header contains a colon
    if ! echo "$header_line" | grep -q ":"; then
        echo "Error: Commit message must follow format '<type>(<scope>): <subject>' or '<type>: <subject>'"
        return 1
    fi

    # Extract header line before colon
    header_prefix=$(echo "$header_line" | sed -E 's/:.*$//')
    subject=$(echo "$header_line" | sed -E 's/^[^:]*:[[:space:]]*//')

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
    if [ "${STRICT_TYPES:-1}" = "0" ]; then
        if echo "$type" | grep -qE '^[a-zA-Z0-9_-]+$'; then
            valid_type=1
        fi
    else
        for t in $COMMIT_TYPES; do
            if [ "$type" = "$t" ]; then
                valid_type=1
                break
            fi
        done
    fi
    
    if [ $valid_type -eq 0 ]; then
        if [ "${STRICT_TYPES:-1}" = "0" ]; then
            echo "Error: Invalid type '$type'. Type must consist of alphanumeric characters, hyphens, or underscores"
        else
            echo "Error: Invalid type '$type'. Must be one of: $COMMIT_TYPES"
        fi
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
            if [ "${DISABLE_SUBSCOPES:-0}" = "1" ]; then
                echo "Error: Subscopes are disabled"
                IFS="$OLD_IFS"
                return 1
            fi
            
            scope=$(echo "$scope_item" | cut -d'/' -f1 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            subscope=$(echo "$scope_item" | cut -d'/' -f2 | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
            
            # Validate scope
            valid_scope=0
            if [ "${STRICT_SCOPES:-0}" = "0" ] && [ "${ALLOW_ANY_SCOPE:-1}" = "1" ]; then
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
            if [ "${STRICT_SUBSCOPES:-0}" = "0" ] && [ "${ALLOW_ANY_SUBSCOPE:-1}" = "1" ]; then
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
            if [ "${STRICT_SCOPES:-0}" = "0" ] && [ "${ALLOW_ANY_SCOPE:-1}" = "1" ]; then
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
    
    if [ $scope_count -gt 1 ] && [ "${DISABLE_MULTIPLE_SCOPES:-0}" = "1" ]; then
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

format_tag_header() {
    local t="$1"
    local subj=$(git tag -l --format="%(contents:subject)" "$t" 2>/dev/null || true)
    if echo "$subj" | grep -qE '^Release [^:]+: .+'; then
        local title=$(echo "$subj" | sed -E 's/^Release [^:]+: //')
        echo "### [$t] - $title"
    elif echo "$subj" | grep -qE '^[^:]+: .+'; then
        local title=$(echo "$subj" | sed -E 's/^[^:]+: //')
        echo "### [$t] - $title"
    else
        echo "### [$t]"
    fi
}

format_changelog_entries() {
    local feat="$1"
    local fix="$2"
    local perf="$3"
    local has_content=0

    if [ -n "$feat" ]; then
        echo "### Features"
        echo "$feat"
        echo
        has_content=1
    fi

    if [ -n "$fix" ]; then
        echo "### Bug Fixes"
        echo "$fix"
        echo
        has_content=1
    fi

    if [ -n "$perf" ]; then
        echo "### Performance Improvements"
        echo "$perf"
        echo
        has_content=1
    fi

    if [ $has_content -eq 0 ]; then
        echo "- Routine maintenance, documentation updates, and operational improvements."
        echo
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
        format_changelog_entries "$feat_commits" "$fix_commits" "$perf_commits"
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
                    format_tag_header "$prev_tag"
                    echo
                    format_changelog_entries "$tag_feat_commits" "$tag_fix_commits" "$tag_perf_commits"
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
                format_tag_header "$prev_tag"
                echo
                format_changelog_entries "$first_feat_commits" "$first_fix_commits" "$first_perf_commits"
            } >> "$changelog_file"
        fi
    fi
} 


# =============================================================================
# Version Functions
# =============================================================================
calculate_target_version() {
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    last_tag=""
    default_base="${DEFAULT_BASE_VERSION:-0.0.1}"

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

    # If the base tag is already a pre-release (e.g. 2.0.0-dev, 2.1.0-dev, 2.0.1-dev)
    if echo "$base_version" | grep -q -- "-"; then
        if [ "$has_breaking" -eq 1 ]; then
            if [ "$minor" -eq 0 ] && [ "$patch" -eq 0 ]; then
                # In an X.0.0 major dev cycle, breaking changes are expected and absorbed
                target_version="${major}.0.0"
            else
                # In a minor/patch pre-release, breaking changes force a new major release
                next_major=$((major + 1))
                target_version="${next_major}.0.0"
            fi
        elif [ "$has_feat" -eq 1 ]; then
            if [ "$patch" -eq 0 ]; then
                # In an X.0.0 or X.Y.0 development cycle, features are absorbed
                target_version="${major}.${minor}.0"
            else
                # In an X.Y.Z patch pre-release, features force a minor bump
                next_minor=$((minor + 1))
                target_version="${major}.${next_minor}.0"
            fi
        else
            target_version="${major}.${minor}.${patch}"
        fi
    elif [ "$has_breaking" -eq 1 ]; then
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
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    if command -v check_prerequisites >/dev/null 2>&1; then
        check_prerequisites
    fi

    last_tag=""
    default_base="${DEFAULT_BASE_VERSION:-0.0.1}"

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
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    if command -v check_prerequisites >/dev/null 2>&1; then
        check_prerequisites
    fi

    # Ensure working tree has no uncommitted changes to tracked files (excluding release artifacts)
    git update-index -q --refresh 2>/dev/null || true
    if ! git diff-index --quiet HEAD -- ':!VERSION' ':!CHANGELOG.md' 2>/dev/null; then
        echo "Error: Working directory has uncommitted changes. Please commit or stash them before creating a tag." >&2
        return 1
    fi

    target_ver=""
    no_prefix=${NO_V:-0}
    message=""
    title=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --no-v|--no-prefix)
                no_prefix=1
                shift
                ;;
            --with-v|--v)
                no_prefix=0
                shift
                ;;
            -t|--title)
                if [ $# -lt 2 ]; then
                    echo "Error: -t option requires a title argument" >&2
                    return 1
                fi
                title="$2"
                shift 2
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
        if [ -n "$title" ]; then
            message="Release $tag_name: $title"
        else
            message="Release $tag_name"
        fi
    fi

    # Update VERSION file with clean SemVer version
    echo "$clean_ver" > "$GIT_ROOT/VERSION"

    export CCCP_TAG_ACTIVE=1
    # Create temporary tag to allow generate_changelog to group commits under this release
    git tag -a "$tag_name" -m "$message"

    if command -v generate_changelog >/dev/null 2>&1; then
        generate_changelog
    fi

    # Commit the release files (VERSION and CHANGELOG.md)
    if [ -n "$title" ]; then
        commit_msg="chore(release): $tag_name - $title"
    else
        commit_msg="chore(release): $tag_name"
    fi

    export HOOK_ACTIVE=1
    git add "$GIT_ROOT/VERSION" "$GIT_ROOT/CHANGELOG.md"
    if ! git commit -m "$commit_msg"; then
        unset HOOK_ACTIVE
        git tag -d "$tag_name" >/dev/null 2>&1 || true
        unset CCCP_TAG_ACTIVE
        echo "Error: Failed to create release commit." >&2
        return 1
    fi
    unset HOOK_ACTIVE

    # Move tag to the release commit
    git tag -f -a "$tag_name" -m "$message"
    unset CCCP_TAG_ACTIVE

    current_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")

    echo "Release commit created: $commit_msg"
    if [ -n "$title" ]; then
        echo "Tag '$tag_name' (\"$title\") created successfully."
    else
        echo "Tag '$tag_name' created successfully."
    fi
    echo "Updated VERSION: $clean_ver"
    echo "Updated CHANGELOG.md"
    echo ""
    echo "To push the release and tag to remote, run:"
    echo "  git push origin $current_branch"
    echo "  git push origin $tag_name"
}


# =============================================================================
# Push Functions
# =============================================================================
cmd_push() {
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    for arg in "$@"; do
        case "$arg" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "push"
                else
                    echo "Usage: cccp push [git push arguments...]"
                fi
                return 0
                ;;
        esac
    done

    # Reset run flag
    export CCCP_POST_PUSH_RAN=0

    # Determine branch and tracking remote before push for fallback hook execution
    curr_branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "main")
    curr_remote=$(git config "branch.${curr_branch}.remote" 2>/dev/null || echo "origin")
    old_remote_sha=$(git rev-parse "refs/remotes/${curr_remote}/${curr_branch}" 2>/dev/null || echo "0000000000000000000000000000000000000000")

    # Execute standard git push
    git push "$@"

    # If reference-transaction hook already ran the post-push hook, finish
    if [ "${CCCP_POST_PUSH_RAN:-0}" = "1" ]; then
        return 0
    fi

    # Fallback execution (e.g. if git hooks are not installed in this repository)
    new_remote_sha=$(git rev-parse "refs/remotes/${curr_remote}/${curr_branch}" 2>/dev/null || git rev-parse HEAD 2>/dev/null || echo "")
    if command -v run_post_push_hook >/dev/null 2>&1; then
        run_post_push_hook "$curr_remote" "$curr_branch" "$old_remote_sha" "$new_remote_sha" || true
    fi
    return 0
}


# =============================================================================
# Tickets Functions
# =============================================================================
parse_codeberg_url() {
    url="$1"
    clean=$(echo "$url" | sed -e 's/\.git$//' -e 's|/*$||')
    case "$clean" in
        *codeberg.org[:/]*)
            repo_path=$(echo "$clean" | sed -e 's|^.*codeberg\.org[:/]||' -e 's|^/*||')
            owner=$(echo "$repo_path" | cut -d'/' -f1)
            repo=$(echo "$repo_path" | cut -d'/' -f2)
            if [ -n "$owner" ] && [ -n "$repo" ]; then
                echo "$owner $repo"
                return 0
            fi
            ;;
    esac
    return 1
}

resolve_codeberg_info() {
    resolved_owner="${CODEBERG_OWNER:-${CODEBERG_USER:-}}"
    resolved_repo="${CODEBERG_REPO:-}"

    if [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        echo "$resolved_owner $resolved_repo"
        return 0
    fi

    # Inspect git remotes if inside a git repository
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        # Check 'origin' first
        origin_url=$(git config remote.origin.url 2>/dev/null || git remote get-url origin 2>/dev/null || true)
        if [ -n "$origin_url" ]; then
            if parsed=$(parse_codeberg_url "$origin_url"); then
                p_owner=$(echo "$parsed" | cut -d' ' -f1)
                p_repo=$(echo "$parsed" | cut -d' ' -f2)
                resolved_owner="${resolved_owner:-$p_owner}"
                resolved_repo="${resolved_repo:-$p_repo}"
                echo "$resolved_owner $resolved_repo"
                return 0
            fi
        fi

        # Check any other configured git remotes
        all_remotes=$(git remote 2>/dev/null || true)
        for rmt in $all_remotes; do
            [ "$rmt" = "origin" ] && continue
            rmt_url=$(git config "remote.${rmt}.url" 2>/dev/null || git remote get-url "$rmt" 2>/dev/null || true)
            if [ -n "$rmt_url" ]; then
                if parsed=$(parse_codeberg_url "$rmt_url"); then
                    p_owner=$(echo "$parsed" | cut -d' ' -f1)
                    p_repo=$(echo "$parsed" | cut -d' ' -f2)
                    resolved_owner="${resolved_owner:-$p_owner}"
                    resolved_repo="${resolved_repo:-$p_repo}"
                    echo "$resolved_owner $resolved_repo"
                    return 0
                fi
            fi
        done
    fi

    if [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        echo "$resolved_owner $resolved_repo"
        return 0
    fi

    return 1
}

ensure_tickets_gitignored() {
    target_file="$1"
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

    [ -z "$git_root" ] && return 0
    [ ! -d "$git_root/.git" ] && return 0

    gitignore_file="$git_root/.gitignore"

    # If git already ignores this path, nothing to do
    if git check-ignore -q "$target_file" 2>/dev/null; then
        return 0
    fi

    # Determine relative path from git root
    rel_path=$(echo "$target_file" | sed "s|^$git_root/||")

    case "$rel_path" in
        .antigravity/*)
            if [ ! -f "$gitignore_file" ] || ! grep -q "^\.antigravity/" "$gitignore_file" 2>/dev/null; then
                echo ".antigravity/" >> "$gitignore_file"
            fi
            ;;
        *)
            escaped=$(echo "$rel_path" | sed 's/[.[\*^$]/\\&/g')
            if [ ! -f "$gitignore_file" ] || ! grep -q "^$escaped$" "$gitignore_file" 2>/dev/null; then
                echo "$rel_path" >> "$gitignore_file"
            fi
            ;;
    esac
    return 0
}

cmd_tickets() {
    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    cli_owner=""
    cli_repo=""
    cli_file=""
    cli_max_bytes=""
    cli_token="${CODEBERG_TOKEN:-}"

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "tickets"
                else
                    echo "Usage: cccp tickets [--file <path>] [--owner <owner>] [--repo <repo>] [--max-bytes <num>]"
                fi
                return 0
                ;;
            -f|--file)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --file" >&2; return 1; }
                cli_file="$1"
                shift
                ;;
            -o|--owner)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --owner" >&2; return 1; }
                cli_owner="$1"
                shift
                ;;
            -r|--repo)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --repo" >&2; return 1; }
                cli_repo="$1"
                shift
                ;;
            -m|--max-bytes)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --max-bytes" >&2; return 1; }
                cli_max_bytes="$1"
                shift
                ;;
            -t|--token)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --token" >&2; return 1; }
                cli_token="$1"
                shift
                ;;
            *)
                echo "Error: Unknown option '$1'" >&2
                return 1
                ;;
        esac
    done

    # Check required tools
    for tool in curl jq; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Warning: Required tool '$tool' is not installed or not in PATH. Skipping tickets sync." >&2
            return 0
        fi
    done

    # Resolve Owner and Repo
    if [ -n "$cli_owner" ] && [ -n "$cli_repo" ]; then
        owner="$cli_owner"
        repo="$cli_repo"
    else
        info=$(resolve_codeberg_info || true)
        if [ -z "$info" ]; then
            echo "Warning: Unable to determine Codeberg repository and owner from git remote or environment." >&2
            return 0
        fi
        owner=$(echo "$info" | cut -d' ' -f1)
        repo=$(echo "$info" | cut -d' ' -f2)
        [ -n "$cli_owner" ] && owner="$cli_owner"
        [ -n "$cli_repo" ] && repo="$cli_repo"
    fi

    if [ -z "$owner" ] || [ -z "$repo" ]; then
        echo "Warning: Unable to determine Codeberg repository and owner from git remote or environment." >&2
        return 0
    fi

    # Resolve Output File
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -n "$cli_file" ]; then
        output_file="$cli_file"
    elif [ -n "${TICKETS_FILE:-}" ]; then
        output_file="$TICKETS_FILE"
    elif [ -n "${CODEBERG_TICKETS_FILE:-}" ]; then
        output_file="$CODEBERG_TICKETS_FILE"
    elif [ -n "$git_root" ]; then
        output_file="$git_root/.antigravity/tickets.md"
    else
        output_file=".antigravity/tickets.md"
    fi

    max_bytes="${cli_max_bytes:-${MAX_BYTES:-10240}}"

    # Create destination directory if necessary
    output_dir="$(dirname "$output_file")"
    [ -n "$output_dir" ] && mkdir -p "$output_dir"

    # Ensure target file is gitignored
    ensure_tickets_gitignored "$output_file"

    api_url="https://codeberg.org/api/v1/repos/${owner}/${repo}/issues?state=open&type=issues&limit=30"

    echo "Fetching open issues from Codeberg (${owner}/${repo})..."
    if [ -n "$cli_token" ]; then
        response=$(curl -s -f -H "Authorization: token ${cli_token}" "$api_url" 2>/dev/null || true)
    else
        response=$(curl -s -f "$api_url" 2>/dev/null || true)
    fi

    if [ -z "$response" ] || ! printf '%s\n' "$response" | jq -e . >/dev/null 2>&1; then
        echo "Warning: Failed to retrieve or parse issues from Codeberg API (${owner}/${repo})." >&2
        return 0
    fi

    if ! printf '%s\n' "$response" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1; then
        echo "Warning: Unexpected response format from Codeberg API." >&2
        return 0
    fi

    temp_file="${output_file}.tmp.$$"
    issues_tmp="${temp_file}.issues"

    # Initialize temporary markdown file
    cat <<EOF > "$temp_file"
# Active Tickets - ${repo}
> Synced at: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
> Ceiling: ${max_bytes} bytes (strictly uncut tickets)

EOF

    current_size=$(wc -c < "$temp_file" | tr -d ' ')
    count=0
    stopped_early=0

    printf '%s\n' "$response" | jq -c '.[]' > "$issues_tmp"

    jq_ticket_filter='
        "## [#" + (.number|tostring) + "] " + .title + "\n" +
        "- **Status**: `" + .state + "`\n" +
        "- **Labels**: " + (if (.labels | length) > 0 then ([.labels[].name] | join(", ")) else "none" end) + "\n" +
        "- **Link**: " + .html_url + "\n\n" +
        "### Description\n" +
        (.body // "_No description provided._") + "\n\n" +
        "---\n"
    '

    while IFS= read -r issue_json || [ -n "$issue_json" ]; do
        [ -z "$issue_json" ] && continue

        ticket_md=$(printf '%s\n' "$issue_json" | jq -r "$jq_ticket_filter")
        entry_size=$(printf '%s\n' "$ticket_md" | wc -c | tr -d ' ')

        if [ $((current_size + entry_size)) -gt "$max_bytes" ]; then
            stopped_early=1
            break
        fi

        printf '%s\n' "$ticket_md" >> "$temp_file"
        current_size=$((current_size + entry_size))
        count=$((count + 1))
    done < "$issues_tmp"

    rm -f "$issues_tmp"
    mv "$temp_file" "$output_file"

    final_size=$(wc -c < "$output_file" | tr -d ' ')
    echo "Done! Synced ${count} complete ticket(s) into ${output_file} (${final_size} / ${max_bytes} bytes)."

    if [ "$stopped_early" -eq 1 ]; then
        echo "Notice: Sync stopped early at ${final_size} bytes to avoid severing the next issue mid-text."
    fi

    return 0
}


# =============================================================================
# Agents Functions
# =============================================================================
fetch_agents_template() {
    url="$1"

    # If it's a local file path
    if [ -f "$url" ]; then
        cat "$url"
        return 0
    fi

    case "$url" in
        file://*)
            local_path="${url#file://}"
            if [ -f "$local_path" ]; then
                cat "$local_path"
                return 0
            fi
            ;;
        http://*|https://*)
            if command -v curl >/dev/null 2>&1; then
                content=$(curl -s -f "$url" 2>/dev/null || true)
                if [ -z "$content" ]; then
                    echo "Warning: Received empty response from '$url'." >&2
                    return 1
                fi
                # Reject responses that look like HTML error pages
                case "$content" in
                    *'<html'*|*'<!DOCTYPE'*|*'<!doctype'*|*'<head'*|*'<body'*)
                        echo "Warning: Response from '$url' does not appear to be a valid Markdown template (looks like HTML). Skipping." >&2
                        return 1
                        ;;
                esac
                printf '%s\n' "$content"
                return 0
            else
                echo "Warning: 'curl' not available; cannot fetch remote template." >&2
                return 1
            fi
            ;;
    esac

    # Fallback: check local repository template if available
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -n "$git_root" ] && [ -f "$git_root/src/base/AGENTS_template.md" ]; then
        cat "$git_root/src/base/AGENTS_template.md"
        return 0
    fi

    return 1
}

fetch_agent_template() {
    fetch_agents_template "$@"
}

extract_template_vars() {
    template_text="$1"
    printf '%s\n' "$template_text" | grep -oE '(\{\{[A-Za-z0-9_]+\}|\$\{[A-Za-z0-9_]+\})' 2>/dev/null | \
        sed -E -e 's/^(\{\{|\$\{)//' -e 's/(\}\}|\})$//' | \
        sort -u || true
}

ensure_agent_gitignored() {
    target_file="$1"
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

    [ -z "$git_root" ] && return 0
    [ ! -d "$git_root/.git" ] && return 0

    gitignore_file="$git_root/.gitignore"

    # If git already ignores this path, nothing to do
    if git check-ignore -q "$target_file" 2>/dev/null; then
        return 0
    fi

    # Determine relative path from git root
    rel_path=$(echo "$target_file" | sed "s|^$git_root/||")

    case "$rel_path" in
        .antigravity/*)
            if [ ! -f "$gitignore_file" ] || ! grep -q "^\.antigravity/" "$gitignore_file" 2>/dev/null; then
                echo ".antigravity/" >> "$gitignore_file"
            fi
            ;;
        *)
            escaped=$(echo "$rel_path" | sed 's/[.[\*^$]/\\&/g')
            if [ ! -f "$gitignore_file" ] || ! grep -q "^$escaped$" "$gitignore_file" 2>/dev/null; then
                echo "$rel_path" >> "$gitignore_file"
            fi
            ;;
    esac
    return 0
}

resolve_agent_var() {
    var_name="$1"
    norm_name=$(echo "$var_name" | tr '[:upper:]' '[:lower:]' | tr '-' '_')
    stripped_name=$(echo "$norm_name" | sed -E 's/^(agents|agent)_//')

    candidates="$norm_name"
    if [ "$stripped_name" != "$norm_name" ]; then
        candidates="$candidates $stripped_name"
    else
        candidates="$candidates agent_${norm_name} agents_${norm_name}"
    fi

    # 1. Environment variables have highest precedence (strictly AGENT_VAR_*)
    for cand in $candidates; do
        up_cand=$(echo "$cand" | tr '[:lower:]' '[:upper:]')
        env_val=$(eval "printf '%s' \"\${AGENT_VAR_${up_cand}:-\${AGENT_VAR_${cand}:-}}\"")
        if [ -n "$env_val" ]; then
            printf '%b\n' "$env_val"
            return 0
        fi
    done

    # 2. Local repository configuration (.cccprc)
    local_file=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        local_file=$(get_local_config_file 2>/dev/null || true)
    fi

    if [ -n "$local_file" ] && [ -f "$local_file" ] && command -v read_file_key >/dev/null 2>&1; then
        for cand in $candidates; do
            up_cand=$(echo "$cand" | tr '[:lower:]' '[:upper:]')
            val=$(read_file_key "$local_file" "agent_var_${cand}" 2>/dev/null || true)
            [ -z "$val" ] && val=$(read_file_key "$local_file" "agent_var_${up_cand}" 2>/dev/null || true)
            [ -z "$val" ] && val=$(read_file_key "$local_file" "${cand}" 2>/dev/null || true)
            [ -z "$val" ] && val=$(read_file_key "$local_file" "${up_cand}" 2>/dev/null || true)
            if [ -n "$val" ]; then
                printf '%b\n' "$val"
                return 0
            fi
        done
    fi

    # 3. Global configuration
    global_file=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        global_file=$(get_global_config_file 2>/dev/null || true)
    fi

    if [ -n "$global_file" ] && [ -f "$global_file" ] && command -v read_file_key >/dev/null 2>&1; then
        for cand in $candidates; do
            up_cand=$(echo "$cand" | tr '[:lower:]' '[:upper:]')
            val=$(read_file_key "$global_file" "agent_var_${cand}" 2>/dev/null || true)
            [ -z "$val" ] && val=$(read_file_key "$global_file" "agent_var_${up_cand}" 2>/dev/null || true)
            if [ -n "$val" ]; then
                printf '%b\n' "$val"
                return 0
            fi
        done
    fi

    return 1
}

cmd_agent() {
    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    action="update"
    target_url=""
    target_file=""
    var_key=""
    var_val=""
    url_from_cli=0

    # Check for subcommands or options
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "agent"
                else
                    echo "Usage: cccp agent [update|check|set|get|unset] [options]"
                fi
                return 0
                ;;
            check|--check|vars|list-vars)
                action="check"
                shift
                ;;
            set|set-var)
                action="set"
                shift
                [ $# -gt 0 ] || { echo "Error: Missing variable name for 'set'" >&2; return 1; }
                var_key="$1"
                shift
                [ $# -gt 0 ] || { echo "Error: Missing value for 'set $var_key'" >&2; return 1; }
                var_val="$1"
                shift
                ;;
            get)
                action="get"
                shift
                [ $# -gt 0 ] || { echo "Error: Missing variable name for 'get'" >&2; return 1; }
                var_key="$1"
                shift
                ;;
            unset)
                action="unset"
                shift
                [ $# -gt 0 ] || { echo "Error: Missing variable name for 'unset'" >&2; return 1; }
                var_key="$1"
                shift
                ;;
            fill|fill-vars)
                action="fill"
                shift
                ;;
            update)
                action="update"
                shift
                ;;
            -u|--url)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --url" >&2; return 1; }
                target_url="$1"
                url_from_cli=1
                shift
                ;;
            -f|--file)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --file" >&2; return 1; }
                target_file="$1"
                shift
                ;;
            *)
                echo "Error: Unknown option or command '$1'" >&2
                return 1
                ;;
        esac
    done

    # Handle variable setting (ergonomic set-av / agent set)
    if [ "$action" = "set" ]; then
        norm_key=$(echo "$var_key" | sed 's/^agent_var_//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -z "$local_cfg" ]; then
            echo "Error: Not inside a git repository. Cannot save to local .cccprc." >&2
            return 1
        fi
        clean_val=$(printf '%s' "$var_val" | awk '{ if (NR > 1) printf "\\n"; printf "%s", $0 }')

        case "$norm_key" in
            url|template_url|agent_url|agents_url|agent_md_url|agents_md_url)
                write_file_key "$local_cfg" "agent_md_url" "$clean_val"
                unset_file_key "$local_cfg" "agent_var_url" 2>/dev/null || true
                echo "Configured agent template URL 'agent_md_url' = '${var_val}' in ${local_cfg}"
                return 0
                ;;
            file|dest|destination|agent_file|agents_file|agent_md_file|agents_md_file)
                write_file_key "$local_cfg" "agent_md_file" "$clean_val"
                echo "Configured agent destination file 'agent_md_file' = '${var_val}' in ${local_cfg}"
                return 0
                ;;
        esac

        write_file_key "$local_cfg" "agent_var_${norm_key}" "$clean_val"
        echo "Configured agent variable '${norm_key}' = '${var_val}' in ${local_cfg}"
        return 0
    fi

    # Handle variable getting
    if [ "$action" = "get" ]; then
        norm_key=$(echo "$var_key" | sed 's/^agent_var_//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
        if val=$(resolve_agent_var "$norm_key"); then
            echo "$val"
            return 0
        else
            echo "Error: Variable '${norm_key}' is not set." >&2
            return 1
        fi
    fi

    # Handle variable unsetting
    if [ "$action" = "unset" ]; then
        norm_key=$(echo "$var_key" | sed 's/^agent_var_//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -z "$local_cfg" ]; then
            echo "Error: Not inside a git repository." >&2
            return 1
        fi
        unset_file_key "$local_cfg" "agent_var_${norm_key}"
        echo "Unset agent variable '${norm_key}' from ${local_cfg}"
        return 0
    fi

    # Resolve template URL: prefer CLI option, then local/global config, then env, then default
    if [ -z "$target_url" ]; then
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -f "$local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_url=$(read_file_key "$local_cfg" "agent_md_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$local_cfg" "agents_md_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$local_cfg" "agent_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$local_cfg" "agent_var_url" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_url" ]; then
        global_cfg=$(get_global_config_file 2>/dev/null || true)
        if [ -n "$global_cfg" ] && [ -f "$global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_url=$(read_file_key "$global_cfg" "agent_md_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$global_cfg" "agents_md_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$global_cfg" "agent_url" 2>/dev/null || true)
            [ -z "$target_url" ] && target_url=$(read_file_key "$global_cfg" "agent_var_url" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_url" ]; then
        target_url="${AGENT_MD_URL:-${AGENTS_MD_URL:-}}"
    fi
    if [ -z "$target_url" ]; then
        target_url="$DEFAULT_AGENT_MD_URL"
    fi

    # Resolve output destination file (default: .antigravity/AGENTS.md)
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$target_file" ]; then
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -f "$local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_file=$(read_file_key "$local_cfg" "agent_md_file" 2>/dev/null || true)
            [ -z "$target_file" ] && target_file=$(read_file_key "$local_cfg" "agents_md_file" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_file" ]; then
        target_file="${AGENT_MD_FILE:-${AGENTS_MD_FILE:-}}"
    fi
    if [ -z "$target_file" ]; then
        if [ -n "$git_root" ]; then
            target_file="$git_root/.antigravity/AGENTS.md"
        else
            target_file=".antigravity/AGENTS.md"
        fi
    fi

    # Persist explicit --url to local config so subsequent runs reuse it
    if [ "$url_from_cli" -eq 1 ] && command -v get_local_config_file >/dev/null 2>&1; then
        _persist_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$_persist_cfg" ] && command -v write_file_key >/dev/null 2>&1; then
            write_file_key "$_persist_cfg" "agent_md_url" "$target_url"
        fi
    fi

    # Fetch template
    echo "Fetching AGENTS template from: $target_url"
    if ! template_content=$(fetch_agents_template "$target_url"); then
        echo "Warning: Skipping AGENTS template generation." >&2
        return 1
    fi

    # Extract variables
    vars=$(extract_template_vars "$template_content")

    # Diagnostic check mode
    if [ "$action" = "check" ]; then
        echo ""
        echo "Template: $target_url"
        echo "Destination: $target_file"
        echo ""
        if [ -z "$vars" ]; then
            echo "No customizable variables detected in template."
            return 0
        fi

        echo "Detected Variables:"
        missing_count=0
        for v in $vars; do
            norm_v=$(echo "$v" | tr '[:upper:]' '[:lower:]')
            stripped_v=$(echo "$norm_v" | sed -E 's/^(agents|agent)_//')
            if val=$(resolve_agent_var "$v"); then
                echo "  [✓] $v = $val"
            else
                echo "  [✗] $v = (missing - configure via: cccp agent set $stripped_v \"<val>\")"
                missing_count=$((missing_count + 1))
            fi
        done
        echo ""
        if [ "$missing_count" -gt 0 ]; then
            echo "Status: $missing_count variable(s) missing. Cannot generate $target_file."
            return 1
        else
            echo "Status: All variables configured! Ready to run 'cccp agent'."
            return 0
        fi
    fi

    # Fill mode: prompt interactively for missing variables, then regenerate
    if [ "$action" = "fill" ]; then
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -z "$local_cfg" ]; then
            echo "Error: Not inside a git repository. Cannot save to local .cccprc." >&2
            return 1
        fi

        if [ -z "$vars" ]; then
            echo "No customizable variables detected in template. Nothing to fill."
            return 0
        fi

        filled_count=0
        for v in $vars; do
            norm_v=$(echo "$v" | tr '[:upper:]' '[:lower:]')
            stripped_v=$(echo "$norm_v" | sed -E 's/^(agents|agent)_//')

            if resolve_agent_var "$v" >/dev/null 2>&1; then
                existing=$(resolve_agent_var "$v")
                echo "  [✓] $stripped_v = $existing  (already set, skipping)"
                continue
            fi

            val=""
            while [ -z "$val" ]; do
                printf "  %s: " "$stripped_v"
                read -r val
                if [ -z "$val" ]; then
                    echo "    ↳ Value cannot be empty."
                fi
            done

            write_file_key "$local_cfg" "agent_var_${stripped_v}" "$val"
            filled_count=$((filled_count + 1))
        done

        if [ "$filled_count" -gt 0 ]; then
            echo ""
            echo "Saved $filled_count variable(s). Regenerating $target_file..."
            # Re-run as update now that vars are set
            cmd_agent update --url "$target_url" --file "$target_file"
        else
            echo "No missing variables — already complete. Run 'cccp agent' to regenerate."
        fi
        return 0
    fi

    # Generate / update mode — write file even when variables are missing
    missing_count=0
    missing_list=""
    for v in $vars; do
        if ! resolve_agent_var "$v" >/dev/null 2>&1; then
            norm_v=$(echo "$v" | tr '[:upper:]' '[:lower:]')
            stripped_v=$(echo "$norm_v" | sed -E 's/^(agents|agent)_//')
            missing_list="${missing_list}\n  - ${stripped_v} (configure with: cccp agent set ${stripped_v} \"<value>\")"
            missing_count=$((missing_count + 1))
        fi
    done

    # Perform interpolation (missing vars leave placeholder intact)
    result="$template_content"
    interpolated_count=0
    for v in $vars; do
        if val=$(resolve_agent_var "$v" 2>/dev/null); then
            REPL_VAL="$val"
            export REPL_VAL
            result=$(printf '%s\n' "$result" | awk -v vname="$v" '
            BEGIN {
                repl = ENVIRON["REPL_VAL"]
                m_tag = "{{" vname "}}"
                s_tag = "${" vname "}"
                m_len = length(m_tag)
                s_len = length(s_tag)
            }
            {
                while ((idx = index($0, m_tag)) > 0) {
                    $0 = substr($0, 1, idx - 1) repl substr($0, idx + m_len)
                }
                while ((idx = index($0, s_tag)) > 0) {
                    $0 = substr($0, 1, idx - 1) repl substr($0, idx + s_len)
                }
                print
            }')
            unset REPL_VAL
            interpolated_count=$((interpolated_count + 1))
        fi
    done

    # Write destination file atomically
    tmp_out="${target_file}.tmp.$$"
    mkdir -p "$(dirname "$target_file")"
    printf '%s\n' "$result" > "$tmp_out"
    mv "$tmp_out" "$target_file"

    # Ensure target file is gitignored
    ensure_agent_gitignored "$target_file"

    echo "Done! Generated $target_file with $interpolated_count variable(s) substituted."
    if [ "$missing_count" -gt 0 ]; then
        printf "Warning: %d variable(s) missing in %s:%b\n" "$missing_count" "$target_file" "$missing_list" >&2
        echo "Run 'cccp agent fill' to interactively provide missing variables." >&2
    fi
    return 0
}

cmd_agents() {
    cmd_agent "$@"
}

cmd_set_av() {
    cmd_agent set "$@"
}


# =============================================================================
# Codeberg Functions
# =============================================================================
_codeberg_enable_sync() {
    cfg="$1"
    target="$2"

    current=$(read_file_key "$cfg" "post_push_sync" 2>/dev/null || true)

    # Check if already enabled (exact word match)
    case " $(echo "$current" | tr ',' ' ') " in
        *" $target "*) return 0 ;;
    esac

    if [ -z "$current" ]; then
        write_file_key "$cfg" "post_push_sync" "$target"
    else
        # Append with comma separator
        write_file_key "$cfg" "post_push_sync" "${current}, ${target}"
    fi
}

_codeberg_setup() {
    template_url=""

    # Parse options
    while [ $# -gt 0 ]; do
        case "$1" in
            -u|--url)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --url" >&2; return 1; }
                template_url="$1"
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "codeberg"
                else
                    echo "Usage: cccp codeberg setup [--url <template-url>]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unknown option '$1'" >&2
                return 1
                ;;
        esac
    done

    # ── 1. Git repository check ──────────────────────────────────────────────
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$git_root" ]; then
        echo "Error: Not inside a git repository." >&2
        return 1
    fi

    local_cfg="$git_root/.cccprc"

    # ── 2. Detect Codeberg remote ────────────────────────────────────────────
    origin_url="$(git config remote.origin.url 2>/dev/null || git remote get-url origin 2>/dev/null || true)"
    if [ -z "$origin_url" ]; then
        echo "Error: No 'origin' remote configured." >&2
        return 1
    fi

    if ! parsed=$(parse_codeberg_url "$origin_url" 2>/dev/null); then
        echo "Error: Remote 'origin' does not appear to be a Codeberg repository." >&2
        echo "  URL: $origin_url" >&2
        return 1
    fi

    cb_owner=$(echo "$parsed" | cut -d' ' -f1)
    cb_repo=$(echo "$parsed" | cut -d' ' -f2)

    echo "✓ Detected Codeberg repository: ${cb_owner}/${cb_repo}"

    # ── 3. Save owner/repo and enable tickets sync ───────────────────────────
    write_file_key "$local_cfg" "codeberg_owner" "$cb_owner"
    write_file_key "$local_cfg" "codeberg_repo"  "$cb_repo"
    echo "✓ Saved codeberg_owner and codeberg_repo to .cccprc"

    _codeberg_enable_sync "$local_cfg" "ticket"
    echo "✓ Enabled post_push_sync = ticket"

    # ── 4. Resolve AGENTS template URL ──────────────────────────────────────
    if [ -z "$template_url" ]; then
        template_url="$(read_file_key "$local_cfg" "agent_md_url" 2>/dev/null || true)"
        [ -z "$template_url" ] && template_url="$(read_file_key "$local_cfg" "agents_md_url" 2>/dev/null || true)"
    fi
    if [ -z "$template_url" ] && command -v get_default_config_value >/dev/null 2>&1; then
        template_url="$(get_default_config_value "agent_md_url" 2>/dev/null || true)"
    fi
    if [ -z "$template_url" ]; then
        template_url="https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md"
    fi

    echo "✓ Using AGENTS template: $template_url"

    # Save the URL to config
    write_file_key "$local_cfg" "agent_md_url" "$template_url"

    # ── 5. Fetch template and extract variables ──────────────────────────────
    echo "  Fetching template..."
    if ! template_content=$(fetch_agents_template "$template_url"); then
        echo "Error: Failed to fetch AGENTS template from '$template_url'." >&2
        return 1
    fi

    vars=$(extract_template_vars "$template_content")

    if [ -z "$vars" ]; then
        echo "✓ Template has no customizable variables. Nothing to configure."
    else
        echo ""
        echo "Configure template variables (press Enter to re-enter; empty values are not accepted):"
        echo ""

        for v in $vars; do
            norm_v=$(echo "$v" | tr '[:upper:]' '[:lower:]')
            stripped_v=$(echo "$norm_v" | sed -E 's/^(agents|agent)_//')

            # Skip if already configured
            if existing=$(resolve_agent_var "$v" 2>/dev/null); then
                echo "  [✓] $v = $existing  (already set, skipping)"
                continue
            fi

            # Interactive prompt — loop until non-empty value provided
            val=""
            while [ -z "$val" ]; do
                printf "  %s: " "$stripped_v"
                read -r val
                if [ -z "$val" ]; then
                    echo "    ↳ Value cannot be empty. Please enter a value."
                fi
            done

            write_file_key "$local_cfg" "agent_var_${stripped_v}" "$val"
            echo "    ✓ Saved agent_var_${stripped_v}"
        done
    fi

    echo ""

    # ── 6. Enable agent sync ─────────────────────────────────────────────────
    _codeberg_enable_sync "$local_cfg" "agent"
    echo "✓ Enabled post_push_sync = agent"

    echo ""
    echo "✓ Codeberg setup complete!"
    echo "  On every 'cccp push':"
    echo "    • Tickets will sync from ${cb_owner}/${cb_repo} → .antigravity/tickets.md"
    echo "    • AGENTS.md will be regenerated from template → .antigravity/AGENTS.md"
    echo ""
    echo "  Run 'cccp push' to trigger the first sync."
}

cmd_codeberg() {
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    subcommand="${1:-}"
    [ $# -gt 0 ] && shift || true

    case "$subcommand" in
        setup|"")
            _codeberg_setup "$@"
            ;;
        -h|--help)
            if command -v show_help >/dev/null 2>&1; then
                show_help "codeberg"
            else
                echo "Usage: cccp codeberg [setup] [options]"
            fi
            ;;
        *)
            echo "Error: Unknown codeberg subcommand '$subcommand'" >&2
            echo "Usage: cccp codeberg setup [--url <template-url>]" >&2
            return 1
            ;;
    esac
}


# =============================================================================
# Hooks Functions
# =============================================================================
install_global_binary() {
    # 1. Determine source file
    source_bin=""
    if [ -n "${1:-}" ] && [ -f "$1" ]; then
        source_bin="$1"
    elif [ -f "${0:-}" ] && [ -s "${0:-}" ]; then
        source_bin="${0:-}"
    elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/cccp.sh" ]; then
        source_bin="$GIT_ROOT/cccp.sh"
    elif [ -f "./cccp.sh" ]; then
        source_bin="./cccp.sh"
    fi

    if [ -z "$source_bin" ] || [ ! -f "$source_bin" ]; then
        tmp_download="$(mktemp)"
        update_url="${UPDATE_URL:-https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh}"
        if command -v curl >/dev/null 2>&1; then
            curl -fsSL "$update_url" -o "$tmp_download" 2>/dev/null || true
        elif command -v wget >/dev/null 2>&1; then
            wget -q "$update_url" -O "$tmp_download" 2>/dev/null || true
        fi

        if [ -s "$tmp_download" ]; then
            source_bin="$tmp_download"
        else
            echo "Error: Could not locate or download cccp.sh to install." >&2
            return 1
        fi
    fi

    # 2. Determine target destination based on EUID / uid
    user_id=$(id -u 2>/dev/null || echo "1000")
    if [ "$user_id" -eq 0 ]; then
        target_dir="/usr/local/bin"
    else
        target_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
    fi

    mkdir -p "$target_dir"
    target_bin="$target_dir/cccp"

    # 3. Copy binary and make executable
    cp "$source_bin" "$target_bin"
    chmod +x "$target_bin"
    echo "Successfully installed cccp to: $target_bin"

    # 4. Check if target_dir is in PATH
    case ":$PATH:" in
        *":$target_dir:"*)
            in_path=1
            ;;
        *)
            in_path=0
            ;;
    esac

    if [ $in_path -eq 0 ]; then
        echo ""
        echo "Notice: '$target_dir' is not currently in your \$PATH."

        # Check ~/.bashrc
        bashrc="$HOME/.bashrc"
        if [ -f "$bashrc" ] || [ ! -f "$HOME/.zshrc" ]; then
            if [ ! -f "$bashrc" ] || ! grep -q "$target_dir" "$bashrc" 2>/dev/null; then
                mkdir -p "$(dirname "$bashrc")"
                printf "\n# Added by cccp installer\nexport PATH=\"%s:\$PATH\"\n" "$target_dir" >> "$bashrc"
                echo "Added '$target_dir' to $bashrc"
            fi
        fi

        # Check ~/.zshrc
        zshrc="$HOME/.zshrc"
        if [ -f "$zshrc" ]; then
            if ! grep -q "$target_dir" "$zshrc" 2>/dev/null; then
                printf "\n# Added by cccp installer\nexport PATH=\"%s:\$PATH\"\n" "$target_dir" >> "$zshrc"
                echo "Added '$target_dir' to $zshrc"
            fi
        fi

        echo ""
        echo "To use 'cccp' immediately in this terminal session, run:"
        echo "    export PATH=\"$target_dir:\$PATH\""
        echo "Or open a new terminal window."
    else
        echo "You can now run 'cccp' from anywhere!"
    fi

    return 0
}

get_canonical_hook_content() {
    hook_name="$1"
    ver="${2:-${CCCP_VERSION:-2.0.0}}"
    root_dir="${3:-${GIT_ROOT:-}}"
    cat <<EOF
#!/bin/sh
# cccp-hook-version: $ver

# Execute cccp from PATH if installed, or fallback to repo root script
if command -v cccp >/dev/null 2>&1; then
    exec cccp $hook_name "\$@"
elif [ -x "$root_dir/cccp.sh" ]; then
    exec "$root_dir/cccp.sh" $hook_name "\$@"
elif [ -x "./cccp.sh" ]; then
    exec ./cccp.sh $hook_name "\$@"
else
    echo "Error: cccp is not installed in PATH or repository root." >&2
    echo "Please install cccp in your PATH or place cccp.sh in repository root." >&2
    exit 1
fi
EOF
}

detect_hook_framework() {
    file="$1"
    if [ ! -f "$file" ]; then
        echo "none"
        return 0
    fi
    if grep -qi "husky" "$file" 2>/dev/null; then
        echo "Husky"
    elif grep -qi "lefthook" "$file" 2>/dev/null; then
        echo "Lefthook"
    elif grep -qi "pre-commit" "$file" 2>/dev/null; then
        echo "pre-commit (Python framework)"
    elif grep -qi "overcommit" "$file" 2>/dev/null; then
        echo "Overcommit"
    else
        first_line=$(head -n 1 "$file" 2>/dev/null || true)
        case "$first_line" in
            *sh*) echo "Custom Shell Script" ;;
            *node*|*js*) echo "Custom Node.js Script" ;;
            *python*) echo "Custom Python Script" ;;
            *) echo "Custom Executable / Script" ;;
        esac
    fi
}

get_hook_backups() {
    hook_path="$1"
    dir=$(dirname "$hook_path")
    base=$(basename "$hook_path")
    backups=""
    for b in "$hook_path.old" "$hook_path.old."*; do
        if [ -f "$b" ] || [ -L "$b" ]; then
            backups="${backups}$(basename "$b") "
        fi
    done
    echo "$backups" | sed 's/[[:space:]]*$//'
}

install_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    GIT_HOOKS_LIST="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"

    # Determine current cccp version to stamp in hook
    current_version="${CCCP_VERSION:-2.0.0}"

    # Create hooks directory if it doesn't exist
    mkdir -p "$GIT_HOOKS_DIR"

    # Process each hook in the list
    for hook in $GIT_HOOKS_LIST; do
        hook_path="$GIT_HOOKS_DIR/$hook"

        # Check if the hook is already an up-to-date cccp wrapper script
        if [ -f "$hook_path" ] && [ ! -L "$hook_path" ] && grep -q "# cccp-hook-version: $current_version" "$hook_path" 2>/dev/null; then
            echo "Hook already installed: $hook_path"
            continue
        fi

        # Handle existing hook (whether regular file or symlink)
        if [ -e "$hook_path" ] || [ -L "$hook_path" ]; then
            backup_name="$hook_path.old"
            counter=1
            while { [ -e "$backup_name" ] || [ -L "$backup_name" ]; } && [ "$counter" -le 100 ]; do
                backup_name="$hook_path.old.$counter"
                counter=$((counter + 1))
            done
            mv "$hook_path" "$backup_name"
            echo "Backed up existing hook to: $backup_name"
        fi

        # Write standalone wrapper script
        get_canonical_hook_content "$hook" "$current_version" > "$hook_path"
        chmod +x "$hook_path"
    done

    echo "Successfully installed git hooks!"
    echo "Hooks configured with cccp version: $current_version"
}

audit_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    hooks_dir="${GIT_HOOKS_DIR:-$GIT_ROOT/.git/hooks}"
    curr_ver="${CCCP_VERSION:-2.0.0}"
    hooks_list="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"

    echo "========================================================"
    echo " ★ CCCP Git Hooks Inspectorate (Komissariat Audit) ★"
    echo "========================================================"
    echo " Repository    : $GIT_ROOT"
    echo " Hooks Dir     : $hooks_dir"
    echo " CCCP Version  : $curr_ver"
    echo ""

    for hook in $hooks_list; do
        hook_path="$hooks_dir/$hook"
        echo " Hook: $hook"
        if [ ! -e "$hook_path" ] && [ ! -L "$hook_path" ]; then
            echo "   Status      : Missing / Not installed"
            echo "   Intervention: Run 'cccp install' to install standard CCCP hook."
            echo ""
            continue
        fi

        # Check if symlink
        if [ -L "$hook_path" ]; then
            link_target=$(ls -l "$hook_path" 2>/dev/null | sed 's/.*-> //')
            echo "   Type        : Symbolic link (-> $link_target)"
        fi

        # Check if CCCP wrapper
        if grep -q "# cccp-hook-version:" "$hook_path" 2>/dev/null; then
            hook_ver=$(sed -n 's/^# cccp-hook-version:[[:space:]]*//p' "$hook_path" | head -n 1)
            if [ "$hook_ver" = "$curr_ver" ]; then
                echo "   Status      : Synchronized (v$hook_ver)"
                echo "   Details     : Up to date with active CCCP version."
            else
                echo "   Status      : Outdated CCCP Wrapper (v$hook_ver vs current v$curr_ver)"
                echo "   Intervention: Run 'cccp install' to upgrade wrapper to v$curr_ver."
                echo "                 Run 'cccp hooks diff $hook' to view changes."
            fi
        else
            framework=$(detect_hook_framework "$hook_path")
            echo "   Status      : Custom / Non-CCCP"
            echo "   Framework   : $framework"
            if grep -q "cccp" "$hook_path" 2>/dev/null; then
                echo "   Chains CCCP : Yes (invokes cccp)"
            else
                echo "   Chains CCCP : No"
            fi
            backups=$(get_hook_backups "$hook_path")
            if [ -n "$backups" ]; then
                echo "   Backups     : $backups"
            fi
            echo "   Intervention:"
            echo "     - To replace with CCCP: Run 'cccp install' (current hook will be backed up)."
            echo "     - To chain CCCP inside this hook: Add 'cccp $hook \"\$@\"' to $hook_path."
            echo "     - To inspect differences: Run 'cccp hooks diff $hook'."
            if [ -n "$backups" ]; then
                first_backup=$(echo "$backups" | cut -d' ' -f1)
                echo "     - To restore prior backup: Run 'mv $hooks_dir/$first_backup $hook_path'."
            fi
        fi
        echo ""
    done

    echo " Optional Extensions:"
    pp_info=$(get_post_push_hook_info 2>/dev/null || echo "status=none")
    pp_status=$(echo "$pp_info" | grep "^status=" | cut -d= -f2-)
    if [ "$pp_status" = "active" ]; then
        pp_source=$(echo "$pp_info" | grep "^source=" | cut -d= -f2-)
        pp_path=$(echo "$pp_info" | grep "^path=" | cut -d= -f2-)
        pp_exec=$(echo "$pp_info" | grep "^executable=" | cut -d= -f2-)
        pp_sync=$(echo "$pp_info" | grep "^sync_actions=" | cut -d= -f2-)
        echo "   Post-Push   : Active"
        echo "   Source      : $pp_source"
        echo "   Script Path : $pp_path"
        echo "   Executable  : $pp_exec"
        [ -n "$pp_sync" ] && echo "   Sync Actions: $pp_sync"
    else
        echo "   Post-Push   : None (Optional - configure 'post_push_hook' or 'post_push_sync' in .cccprc)"
    fi
    echo ""
    echo "========================================================"
    return 0
}

diff_git_hooks() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository. This command must be executed within a valid Git repository." >&2
        return 1
    fi

    hooks_dir="${GIT_HOOKS_DIR:-$GIT_ROOT/.git/hooks}"
    curr_ver="${CCCP_VERSION:-2.0.0}"
    target_hook="${1:-}"

    if [ -n "$target_hook" ]; then
        hooks_to_diff="$target_hook"
    else
        hooks_to_diff="${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}"
    fi

    diff_cmd="diff -u"
    if ! command -v diff >/dev/null 2>&1; then
        echo "Error: 'diff' tool is not available in system PATH." >&2
        return 1
    fi

    for hook in $hooks_to_diff; do
        hook_path="$hooks_dir/$hook"
        echo "=== Diff: $hook ($hook_path vs canonical CCCP v$curr_ver) ==="
        if [ ! -e "$hook_path" ] && [ ! -L "$hook_path" ]; then
            echo "Hook file '$hook_path' does not exist."
            echo ""
            continue
        fi

        tmp_expected="$(mktemp)"
        get_canonical_hook_content "$hook" "$curr_ver" > "$tmp_expected"

        if diff -q "$hook_path" "$tmp_expected" >/dev/null 2>&1; then
            echo "No differences found. Installed hook is identical to canonical CCCP wrapper."
        else
            $diff_cmd "$hook_path" "$tmp_expected" || true
            echo ""
            echo "Intervention Guidance:"
            if grep -q "# cccp-hook-version:" "$hook_path" 2>/dev/null; then
                echo "  - Installed hook is an older CCCP wrapper. Run 'cccp install' to upgrade."
            else
                echo "  - Installed hook is custom/non-CCCP. Run 'cccp install' to replace (with backup),"
                echo "    or manually chain 'cccp $hook \"\$@\"' within your script."
            fi
        fi
        rm -f "$tmp_expected"
        echo ""
    done

    return 0
}

cmd_hooks() {
    if [ $# -gt 0 ]; then
        case "$1" in
            audit|check|status)
                shift
                audit_git_hooks "$@"
                return $?
                ;;
            diff)
                shift
                diff_git_hooks "$@"
                return $?
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "hooks"
                else
                    echo "Usage: cccp hooks [audit|diff [hook]]"
                fi
                return 0
                ;;
            *)
                for h in ${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction}; do
                    if [ "$1" = "$h" ]; then
                        diff_git_hooks "$@"
                        return $?
                    fi
                done
                echo "Error: Unknown hooks action '$1'. Choose 'audit' or 'diff'." >&2
                return 1
                ;;
        esac
    fi
    audit_git_hooks
}

cmd_pre_push() {
    # If stdin is a tty, nothing to validate
    [ -t 0 ] && return 0

    has_error=0
    while read -r local_ref local_sha remote_ref remote_sha || [ -n "$local_ref" ]; do
        # Ignore ref deletions
        if [ "$local_sha" = "0000000000000000000000000000000000000000" ] || [ "$local_ref" = "(delete)" ]; then
            continue
        fi

        case "$remote_ref" in
            refs/tags/*)
                tag_name="${remote_ref#refs/tags/}"
                clean_tag="${tag_name#v}"

                # Only validate SemVer release tags (e.g. v1.0.0, 1.0.0)
                # Ignore non-version utility tags (e.g. nightly, latest, canary)
                if ! echo "$tag_name" | grep -qE '^v?[0-9]'; then
                    continue
                fi

                # Check if VERSION exists in the commit pointed to by local_sha
                ver_in_commit=$(git show "${local_sha}:VERSION" 2>/dev/null || true)
                ver_in_commit=$(echo "$ver_in_commit" | tr -d ' \r\n')

                if [ -z "$ver_in_commit" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' points to a commit without a VERSION file." >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Releases must contain an official VERSION file." >&2
                    echo " Run 'cccp tag' to generate the release commit and changelog." >&2
                    echo "========================================================" >&2
                    has_error=1
                elif echo "$ver_in_commit" | grep -qE -- "-dev|\+"; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' points to a development version: '$ver_in_commit'." >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Pre-release/development versions cannot be pushed as release tags." >&2
                    echo " Run 'cccp tag' to generate clean release commits and synchronize CHANGELOG." >&2
                    echo "========================================================" >&2
                    has_error=1
                elif [ "$ver_in_commit" != "$clean_tag" ] && [ "$ver_in_commit" != "$tag_name" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Push Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Tag '$tag_name' does not match VERSION file in target commit." >&2
                    echo " Expected: '$clean_tag', found in commit: '$ver_in_commit'" >&2
                    echo " Target commit: $local_sha" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Tag name and VERSION file must be strictly synchronized." >&2
                    echo " Run 'cccp tag' to generate clean release commits and synchronize CHANGELOG." >&2
                    echo "========================================================" >&2
                    has_error=1
                fi
                ;;
        esac
    done

    if [ "$has_error" -eq 1 ]; then
        return 1
    fi
    return 0
}

resolve_post_push_hook() {
    # 1. Configured via POST_PUSH_HOOK
    if [ -n "${POST_PUSH_HOOK:-}" ]; then
        echo "$POST_PUSH_HOOK"
        return 0
    fi

    # 2. Shared convention in repository (.cccp/hooks/post-push)
    if [ -n "${GIT_ROOT:-}" ] && [ -x "$GIT_ROOT/.cccp/hooks/post-push" ]; then
        echo "$GIT_ROOT/.cccp/hooks/post-push"
        return 0
    fi

    # 3. Local convention in repository (.git/hooks/post-push.local or .git/hooks/post-push)
    if [ -n "${GIT_HOOKS_DIR:-}" ]; then
        if [ -x "$GIT_HOOKS_DIR/post-push.local" ]; then
            echo "$GIT_HOOKS_DIR/post-push.local"
            return 0
        elif [ -x "$GIT_HOOKS_DIR/post-push" ]; then
            echo "$GIT_HOOKS_DIR/post-push"
            return 0
        fi
    fi

    return 1
}

is_post_push_sync_tickets_enabled() {
    case "${POST_PUSH_SYNC_TICKETS:-0}" in
        1|true|TRUE|yes|YES|on|ON) return 0 ;;
    esac
    for action in $(printf '%s' "${POST_PUSH_SYNC:-}" | tr ',' ' '); do
        case "$action" in
            tickets|sync-tickets) return 0 ;;
        esac
    done
    return 1
}

is_post_push_sync_agent_enabled() {
    case "${POST_PUSH_SYNC_AGENT:-${POST_PUSH_SYNC_AGENTS:-0}}" in
        1|true|TRUE|yes|YES|on|ON) return 0 ;;
    esac
    for action in $(printf '%s' "${POST_PUSH_SYNC:-}" | tr ',' ' '); do
        case "$action" in
            agent|agents|sync-agent|sync-agents) return 0 ;;
        esac
    done
    return 1
}

is_post_push_sync_agents_enabled() {
    is_post_push_sync_agent_enabled
}

get_post_push_hook_info() {
    sync_actions=""
    if is_post_push_sync_tickets_enabled; then
        sync_actions="sync-tickets"
    fi
    if is_post_push_sync_agent_enabled; then
        if [ -n "$sync_actions" ]; then
            sync_actions="$sync_actions, sync-agent"
        else
            sync_actions="sync-agent"
        fi
    fi

    if [ -n "${POST_PUSH_HOOK:-}" ]; then
        echo "status=active"
        echo "source=config (POST_PUSH_HOOK)"
        echo "path=$POST_PUSH_HOOK"
        if [ -f "$POST_PUSH_HOOK" ]; then
            if [ -x "$POST_PUSH_HOOK" ]; then
                echo "executable=yes"
            else
                echo "executable=no (warning: file is not executable)"
            fi
        else
            echo "executable=command"
        fi
        [ -n "$sync_actions" ] && echo "sync_actions=$sync_actions"
        return 0
    elif [ -n "${GIT_ROOT:-}" ] && [ -e "$GIT_ROOT/.cccp/hooks/post-push" ]; then
        echo "status=active"
        echo "source=shared convention (.cccp/hooks/post-push)"
        echo "path=$GIT_ROOT/.cccp/hooks/post-push"
        if [ -x "$GIT_ROOT/.cccp/hooks/post-push" ]; then
            echo "executable=yes"
        else
            echo "executable=no (warning: file is not executable)"
        fi
        [ -n "$sync_actions" ] && echo "sync_actions=$sync_actions"
        return 0
    elif [ -n "${GIT_HOOKS_DIR:-}" ] && [ -e "$GIT_HOOKS_DIR/post-push.local" ]; then
        echo "status=active"
        echo "source=local convention (.git/hooks/post-push.local)"
        echo "path=$GIT_HOOKS_DIR/post-push.local"
        if [ -x "$GIT_HOOKS_DIR/post-push.local" ]; then
            echo "executable=yes"
        else
            echo "executable=no (warning: file is not executable)"
        fi
        [ -n "$sync_actions" ] && echo "sync_actions=$sync_actions"
        return 0
    elif [ -n "${GIT_HOOKS_DIR:-}" ] && [ -e "$GIT_HOOKS_DIR/post-push" ]; then
        echo "status=active"
        echo "source=local convention (.git/hooks/post-push)"
        echo "path=$GIT_HOOKS_DIR/post-push"
        if [ -x "$GIT_HOOKS_DIR/post-push" ]; then
            echo "executable=yes"
        else
            echo "executable=no (warning: file is not executable)"
        fi
        [ -n "$sync_actions" ] && echo "sync_actions=$sync_actions"
        return 0
    elif [ -n "$sync_actions" ]; then
        echo "status=active"
        echo "source=built-in ($sync_actions)"
        echo "path=(built-in sync)"
        echo "executable=yes"
        echo "sync_actions=$sync_actions"
        return 0
    fi

    echo "status=none"
    return 1
}

run_post_push_hook() {
    remote="${1:-}"
    branch="${2:-}"
    old_sha="${3:-}"
    new_sha="${4:-}"

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    hook_cmd=$(resolve_post_push_hook 2>/dev/null || true)
    sync_tickets_active=0
    sync_agent_active=0
    if is_post_push_sync_tickets_enabled; then
        sync_tickets_active=1
    fi
    if is_post_push_sync_agent_enabled; then
        sync_agent_active=1
    fi

    if [ -z "$hook_cmd" ] && [ "$sync_tickets_active" -eq 0 ] && [ "$sync_agent_active" -eq 0 ]; then
        return 0
    fi

    export CCCP_PUSH_REMOTE="$remote"
    export CCCP_PUSH_BRANCH="$branch"
    export CCCP_PUSH_OLD_SHA="$old_sha"
    export CCCP_PUSH_NEW_SHA="$new_sha"

    # Mark as executed for this session to avoid duplicate execution in cccp push
    export CCCP_POST_PUSH_RAN=1

    # 1. Execute custom hook script if configured
    if [ -n "$hook_cmd" ]; then
        if [ -f "$hook_cmd" ] && [ -x "$hook_cmd" ]; then
            "$hook_cmd" "$remote" "$branch" "$old_sha" "$new_sha"
        else
            sh -c "$hook_cmd \"\$@\"" -- "$remote" "$branch" "$old_sha" "$new_sha"
        fi
    fi

    # 2. Execute sync-tickets if enabled
    if [ "$sync_tickets_active" -eq 1 ]; then
        if command -v cmd_tickets >/dev/null 2>&1; then
            cmd_tickets || true
        elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/src/utils/tickets.sh" ]; then
            . "$GIT_ROOT/src/utils/tickets.sh"
            cmd_tickets || true
        elif command -v cccp >/dev/null 2>&1; then
            cccp tickets || true
        fi
    fi

    # 3. Execute sync-agent if enabled
    if [ "$sync_agent_active" -eq 1 ]; then
        if command -v cmd_agent >/dev/null 2>&1; then
            cmd_agent update || true
        elif command -v cmd_agents >/dev/null 2>&1; then
            cmd_agents update || true
        elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/src/utils/agents.sh" ]; then
            . "$GIT_ROOT/src/utils/agents.sh"
            cmd_agent update || true
        elif command -v cccp >/dev/null 2>&1; then
            cccp agent update || true
        fi
    fi

    return 0
}

cmd_reference_transaction() {
    state="${1:-}"

    # Handle post-push execution during 'committed' state
    if [ "$state" = "committed" ]; then
        parent_cmd=""
        if [ -r "/proc/$PPID/cmdline" ]; then
            parent_cmd=$(tr '\0' ' ' < "/proc/$PPID/cmdline" 2>/dev/null || true)
        fi
        if [ -z "$parent_cmd" ] && command -v ps >/dev/null 2>&1; then
            parent_cmd=$(ps -p "$PPID" -o args= 2>/dev/null || ps -p "$PPID" -o command= 2>/dev/null || true)
        fi
        case "$parent_cmd" in
            *push*)
                # Process ref updates from successful push
                if [ ! -t 0 ]; then
                    while read -r old_sha new_sha ref_name || [ -n "$old_sha" ]; do
                        case "$ref_name" in
                            refs/remotes/*)
                                remote_branch="${ref_name#refs/remotes/}"
                                remote="${remote_branch%%/*}"
                                branch="${remote_branch#*/}"
                                run_post_push_hook "$remote" "$branch" "$old_sha" "$new_sha" || true
                                ;;
                        esac
                    done
                fi
                return 0
                ;;
        esac
        return 0
    fi

    # Only enforce validation during 'prepared' state
    if [ "$state" != "prepared" ]; then
        return 0
    fi

    # If cccp tag is orchestrating the release, bypass check
    if [ "${CCCP_TAG_ACTIVE:-0}" -eq 1 ]; then
        return 0
    fi

    # Inspect parent command to bypass remote synchronization operations (fetch, pull, clone)
    parent_cmd=""
    if [ -r "/proc/$PPID/cmdline" ]; then
        parent_cmd=$(tr '\0' ' ' < "/proc/$PPID/cmdline" 2>/dev/null || true)
    fi
    if [ -z "$parent_cmd" ] && command -v ps >/dev/null 2>&1; then
        parent_cmd=$(ps -p "$PPID" -o args= 2>/dev/null || ps -p "$PPID" -o command= 2>/dev/null || true)
    fi
    case "$parent_cmd" in
        *fetch*|*pull*|*clone*)
            return 0
            ;;
    esac

    # If stdin is a tty, nothing to validate
    [ -t 0 ] && return 0

    while read -r old_sha new_sha ref_name || [ -n "$old_sha" ]; do
        # Ignore tag deletion
        if [ "$new_sha" = "0000000000000000000000000000000000000000" ]; then
            continue
        fi

        case "$ref_name" in
            refs/tags/*)
                tag_name="${ref_name#refs/tags/}"
                clean_tag="${tag_name#v}"

                # Only validate SemVer release tags (e.g. v1.0.0, 1.0.0)
                # Ignore non-version utility tags (e.g. nightly, latest, canary)
                if ! echo "$tag_name" | grep -qE '^v?[0-9]'; then
                    continue
                fi

                # Inspect VERSION file in the commit being tagged
                ver_in_commit=$(git show "${new_sha}:VERSION" 2>/dev/null || true)
                ver_in_commit=$(echo "$ver_in_commit" | tr -d ' \r\n')

                if [ -z "$ver_in_commit" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " The target commit ($new_sha) does not have a VERSION file." >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to generate the release commit and synchronize VERSION and CHANGELOG." >&2
                    echo "========================================================" >&2
                    return 1
                elif echo "$ver_in_commit" | grep -qE -- "-dev|\+"; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " The target commit has a development VERSION: '$ver_in_commit'." >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to create official release commits with clean SemVer." >&2
                    echo "========================================================" >&2
                    return 1
                elif [ "$ver_in_commit" != "$clean_tag" ] && [ "$ver_in_commit" != "$tag_name" ]; then
                    echo "========================================================" >&2
                    echo " ★ CCCP Gosplan Tag Quality Control (Intervention) ★" >&2
                    echo "========================================================" >&2
                    echo " Error: Cannot create tag '$tag_name'." >&2
                    echo " Tag version does not match target commit VERSION: '$ver_in_commit'." >&2
                    echo " Expected: '$clean_tag'" >&2
                    echo "" >&2
                    echo " Gosplan Directive: Direct git tag blocked." >&2
                    echo " Use 'cccp tag' to synchronize VERSION, tag, and CHANGELOG." >&2
                    echo "========================================================" >&2
                    return 1
                fi
                ;;
        esac
    done

    return 0
}

install_cccp() {
    is_global=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --global|-g)
                is_global=1
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "install"
                else
                    echo "Usage: cccp install [--global|-g]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unexpected argument '$1'" >&2
                return 1
                ;;
        esac
    done

    if [ $is_global -eq 1 ]; then
        install_global_binary
    else
        install_git_hooks
    fi
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


# =============================================================================
# Help Functions
# =============================================================================
show_help() {
    target="${1:-}"
    ver="${CCCP_VERSION:-2.0.0}"

    echo "★ CCCP v$ver - Conventional Commits Compliance Program ★"
    echo ""

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
            echo "     - If no tag exists, uses DEFAULT_BASE_VERSION (default: 0.0.1)."
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
            echo "  -t, --title       Release title for commit, tag annotation, and CHANGELOG"
            echo "  --no-v            Create tag without 'v' prefix (e.g. 2.1.0 instead of v2.1.0)"
            echo "  -m, --message     Custom tag annotation message (default: 'Release <tag>: <title>')"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Actions performed:"
            echo "  1. Ensures working directory is clean (no uncommitted changes)."
            echo "  2. Verifies the tag does not already exist."
            echo "  3. Updates VERSION file with clean SemVer version (e.g. 2.1.0)."
            echo "  4. Regenerates CHANGELOG.md including release title header."
            echo "  5. Creates release commit 'chore(release): <tag> - <title>' containing VERSION & CHANGELOG.md."
            echo "  6. Creates annotated Git tag pointing directly to the release commit."
            echo ""
            echo "Examples:"
            echo "  $0 tag                                     # Automatically tag with predicted version (e.g. v0.3.0)"
            echo "  $0 tag -t \"Sputnik Protocol\"               # Auto-tag with release title"
            echo "  $0 tag 2.0.0 -t \"Great October Revolution\" # Explicit version with title"
            echo "  $0 tag 2.1.0 --no-v                        # Creates tag 2.1.0 (without 'v')"
            echo "  $0 tag 1.0.0 -m 'First GA'                 # Custom annotation message"
            ;;
        "push")
            echo "cccp.sh push - Execute Git Push and Trigger Optional Post-Push Hook"
            echo "==================================================================="
            echo ""
            echo "Forwards arguments directly to 'git push' and triggers the optional post-push"
            echo "hook upon successful push completion."
            echo ""
            echo "Usage:"
            echo "  $0 push [git push arguments...]"
            echo ""
            echo "Post-Push Hook Discovery Order:"
            echo "  1. Configured via 'post_push_hook' in .cccprc, ~/.config/cccp/config, or POST_PUSH_HOOK env"
            echo "  2. Shared convention: .cccp/hooks/post-push (if executable)"
            echo "  3. Local convention:  .git/hooks/post-push.local or .git/hooks/post-push (if executable)"
            echo "  4. None: If no hook is defined, execution concludes cleanly with zero overhead."
            echo ""
            echo "Parameters passed to the post-push hook:"
            echo "  Positional arguments:"
            echo "    \$1: Remote name (e.g. 'origin')"
            echo "    \$2: Branch name (e.g. 'main')"
            echo "    \$3: Old commit SHA before push"
            echo "    \$4: New commit SHA after push"
            echo "  Exported Environment Variables:"
            echo "    CCCP_PUSH_REMOTE   Remote name"
            echo "    CCCP_PUSH_BRANCH   Branch name"
            echo "    CCCP_PUSH_OLD_SHA  Commit SHA prior to push"
            echo "    CCCP_PUSH_NEW_SHA  Commit SHA following push"
            echo ""
            echo "Automatic Synchronization Actions:"
            echo "  Post-push can automatically run synchronization tasks configured via:"
            echo "    post_push_sync_tickets = 1  (or sync_tickets = 1) -> syncs Codeberg issues"
            echo "    post_push_sync_agent   = 1  (or sync_agent   = 1) -> syncs AGENTS.md directives"
            echo "    post_push_sync         = tickets, agent           -> combined list"
            echo ""
            echo "Note on Native 'git push':"
            echo "  The post-push hook and sync actions also execute transparently on native 'git push'"
            echo "  commands via the 'reference-transaction committed' hook."
            ;;
        "tickets"|"sync-tickets")
            echo "cccp.sh tickets - Synchronize Codeberg Issues into Markdown"
            echo "==========================================================="
            echo ""
            echo "Fetches open issues from Codeberg API dynamically and formats them into a clean"
            echo "markdown document with a strict byte ceiling (10KB default), strictly avoiding"
            echo "truncating tickets mid-text."
            echo ""
            echo "Usage:"
            echo "  $0 tickets [options]"
            echo "  $0 sync-tickets [options]"
            echo ""
            echo "Options:"
            echo "  -f, --file <path>        Output markdown file (default: .antigravity/tickets.md)"
            echo "  -o, --owner <owner>      Codeberg repository owner/organization"
            echo "  -r, --repo <repo>        Codeberg repository name"
            echo "  -m, --max-bytes <num>    Maximum budget in bytes (default: 10240)"
            echo "  -t, --token <token>      Codeberg personal access token (or via CODEBERG_TOKEN)"
            echo ""
            echo "Dynamic Repository Discovery:"
            echo "  1. CLI options (--owner, --repo) and environment variables (CODEBERG_OWNER, CODEBERG_REPO)"
            echo "  2. Git remote URL inspection (origin or other remotes) supporting SSH and HTTPS"
            echo "  3. Graceful fallback: If remote is not Codeberg or network fails, logs to stderr and exits 0"
            echo ""
            echo "Configuration:"
            echo "  The target file can be permanently set via 'cccp config set tickets_file <path>'"
            echo "  and can be chained with 'post_push_hook' to run automatically on every push:"
            echo "    $ cccp config set post_push_hook \"cccp tickets\""
            ;;
        "agent"|"agents"|"update-agent"|"update-agents"|"sync-agent"|"sync-agents"|"set-av"|"set-agent-var")
            echo "cccp.sh agent - Generate and Synchronize AGENTS.md Directives"
            echo "==========================================================="
            echo ""
            echo "Generates or updates AGENTS.md using a shared Markdown skeleton with local variable"
            echo "interpolation from .cccprc, strictly preventing incomplete file generation."
            echo ""
            echo "Usage:"
            echo "  $0 agent [options]                      # Generate/update AGENTS.md"
            echo "  $0 agent check [options]                # Validate template variables status"
            echo "  $0 agent fill [options]                 # Interactively prompt and save missing variables"
            echo "  $0 agent set <var_name> <value>         # Set local agent variable in .cccprc"
            echo "  $0 agent get <var_name>                 # Read agent variable value"
            echo "  $0 agent unset <var_name>               # Remove agent variable from .cccprc"
            echo "  $0 set-av <var_name> <value>            # Fast shortcut to set agent variable"
            echo ""
            echo "Options:"
            echo "  -u, --url <url_or_path>  Template URL or local file path"
            echo "  -f, --file <path>        Output markdown file (default: .antigravity/AGENTS.md)"
            echo ""
            echo "Configuration in .cccprc:"
            echo "  agent_md_url   = <url>    # Template URL or file path"
            echo "  agent_md_file  = <path>   # Target destination path"
            echo ""
            echo "Variable Substitution:"
            echo "  Supports both {{VAR_NAME}} and \${VAR_NAME} in the skeleton. Variables are"
            echo "  resolved from local .cccprc (keys: agent_var_<name>) or environment variables."
            echo "  If variables are missing, the file is still generated with placeholders intact"
            echo "  and a warning is displayed. Run '$0 agent fill' to complete them interactively."
            ;;
        "commit")
            echo "cccp.sh commit - Conventional Commit Creation"
            echo "============================================="
            echo ""
            echo "Validates and creates a Git commit following the Conventional Commits specification."
            echo ""
            echo "Usage:"
            echo "  $0 commit [options] <message>"
            echo "  $0 commit -i | --interactive"
            echo ""
            echo "Options:"
            echo "  -i, --interactive        Launch the interactive terminal commit wizard"
            echo "  --strict-scopes          Enforce known scopes list"
            echo "  --allow-any-scope        Permit arbitrary scopes"
            echo "  --disable-subscopes      Disallow slash-delimited subscopes"
            echo "  --disable-multiple-scopes Disallow comma-separated scopes"
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
            echo "  $0 commit -i"
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
            echo "cccp.sh install - Install cccp Globally or Git Hooks Locally"
            echo "==========================================================="
            echo ""
            echo "Installs cccp in PATH or sets up Git hooks in '.git/hooks':"
            echo ""
            echo "Usage:"
            echo "  $0 install [--global|-g]"
            echo ""
            echo "Options:"
            echo "  --global, -g   Install cccp executable into your PATH"
            echo "                 (/usr/local/bin for root, ~/.local/bin for users)"
            echo "  -h, --help     Show this help message"
            echo ""
            echo "Default (no flags):"
            echo "  Installs portable wrapper Git hooks into '.git/hooks':"
            echo "  - commit-msg:  Validates commit message format against Conventional Commits."
            echo "  - post-commit: Automatically runs 'changelog' and 'version' after each commit."
            ;;
        "update")
            echo "cccp.sh update - Update cccp to Latest Version"
            echo "============================================="
            echo ""
            echo "Downloads and installs the latest cccp release from GitHub or manages version pinning."
            echo ""
            echo "Usage:"
            echo "  $0 update [--channel stable|nightly] [--pin [version]] [--unpin]"
            echo ""
            echo "Options:"
            echo "  --channel, -c    Release channel to update from ('stable' or 'nightly', default: stable)"
            echo "  --pin, -p [ver]  Pin cccp to current version or specified release tag"
            echo "  --unpin, -u      Remove version pin and resume standard release tracking"
            echo "  -h, --help       Show this help message"
            echo ""
            echo "Configuration:"
            echo "  Default channel and interval can be configured via:"
            echo "    $0 config update_channel <stable|nightly>"
            echo "    $0 config update_interval_days <days>"
            echo "    $0 config check_updates <0|1>"
            echo "    $0 config pinned_version <version>"
            ;;
        "check-update"|"check-updates")
            echo "cccp.sh check-update - Inspect Version Status and Upstream Releases"
            echo "=================================================================="
            echo ""
            echo "Queries upstream release telemetry to check for available updates,"
            echo "inspects the active release channel, and verifies Gosplan version pin status."
            echo ""
            echo "Usage:"
            echo "  $0 check-update"
            echo ""
            echo "Information displayed:"
            echo "  - Installed cccp version"
            echo "  - Active release channel (stable or nightly)"
            echo "  - Pinned version status (if locked by Gosplan decree)"
            echo "  - Remote upstream version from GitHub releases"
            echo "  - Alignment status and recommended upgrade actions"
            ;;
        "config")
            echo "cccp.sh config - Manage Hierarchical Configuration"
            echo "=================================================="
            echo ""
            echo "Inspect and modify global and repository-level configuration settings."
            echo ""
            echo "Precedence:"
            echo "  Defaults < Global (~/.config/cccp/config) < Local (.cccprc) < Environment < CLI flags"
            echo ""
            echo "Usage:"
            echo "  $0 config [--global|--local] <key> [value]"
            echo "  $0 config [--global|--local] --list"
            echo "  $0 config [--global|--local] --unset <key>"
            echo ""
            echo "Options:"
            echo "  --global, -g     Target global config file (~/.config/cccp/config)"
            echo "  --local          Target local repo config file (.cccprc)"
            echo "  --list, -l       List configuration key/value pairs"
            echo "  --unset          Remove a configuration key"
            echo "  -h, --help       Show this help message"
            echo ""
            echo "Supported Keys:"
            echo "  Repository & Global:"
            echo "    types                    Allowed commit types (space-separated list)"
            echo "    scopes                   Allowed commit scopes (space-separated list)"
            echo "    subscopes                Allowed commit subscopes (space-separated list)"
            echo "    strict_types             Enforce type matching known types list (0 or 1, default: 1)"
            echo "    strict_scopes            Require scope to match known list (0 or 1, default: 0)"
            echo "    strict_subscopes         Require subscope to match known list (0 or 1, default: 0)"
            echo "    disable_subscopes        Disallow slash-delimited subscopes (0 or 1, default: 0)"
            echo "    disable_multiple_scopes  Disallow comma-separated scopes (0 or 1, default: 0)"
            echo "    default_base_version     Fallback SemVer when no tags exist (default: 0.0.1)"
            echo "    no_v                     Tag without 'v' prefix (0 or 1, default: 0)"
            echo "    type_desc_<type>         Custom description for commit type"
            echo "    scope_desc_<scope>       Custom description for commit scope"
            echo ""
            echo "  Global Only (system update management, saved in ~/.config/cccp/config):"
            echo "    update_channel           Update channel ('stable' or 'nightly', default: stable)"
            echo "    update_interval_days     Days between update checks (default: 30)"
            echo "    check_updates            Enable update checks (0 or 1, default: 1)"
            echo "    pinned_version           Lock cccp to specific version (suppresses updates)"
            echo ""
            echo "Examples:"
            echo "  $0 config types 'feat fix chore docs'"
            echo "  $0 config scopes 'core api cli ui auth'"
            echo "  $0 config type_desc_feat 'State-approved feature addition'"
            echo "  $0 config strict_scopes 1"
            echo "  $0 config --global update_channel nightly"
            echo "  $0 config pinned_version 2.0.0"
            echo "  $0 config default_base_version"
            echo "  $0 config --list"
            echo "  $0 config --unset strict_scopes"
            ;;
        "status")
            echo "cccp.sh status - State Inspection (Gosplan Quality Control)"
            echo "=========================================================="
            echo ""
            echo "Inspects repository health, working tree cleanliness, Git hooks status,"
            echo "predictive SemVer targeting, active policies, and configuration files."
            echo ""
            echo "Usage:"
            echo "  $0 status"
            echo ""
            echo "Metrics inspected:"
            echo "  - Repository root and active branch tracking status"
            echo "  - Working tree state (clean or breakdown of staged/modified/untracked files)"
            echo "  - Current version and predicted next target version"
            echo "  - Commits ahead of latest release tag"
            echo "  - Git hooks installation and version synchronization"
            echo "  - System binary location in PATH"
            echo "  - Active local (.cccprc) and global configuration files"
            echo "  - Active type and scope validation policies"
            echo "  - Update channel and periodic check cadence"
            ;;
        "lint")
            echo "cccp.sh lint - Party Line Compliance (Commit History Linting)"
            echo "============================================================"
            echo ""
            echo "Validates commit messages across a Git revision range to ensure"
            echo "strict compliance with the Conventional Commits specification."
            echo ""
            echo "Usage:"
            echo "  $0 lint [range]"
            echo ""
            echo "Arguments:"
            echo "  range            Git revision range to lint (e.g. 'origin/main..HEAD', 'HEAD~5..HEAD')"
            echo "                   If omitted, automatically defaults to upstream @{u}..HEAD,"
            echo "                   origin/\$GITHUB_BASE_REF..HEAD (in CI), or HEAD~1..HEAD."
            echo ""
            echo "Exit codes:"
            echo "  0                All commits compliant (clean collective compliance)"
            echo "  1                One or more commits violate formatting rules (Order No. 227)"
            echo ""
            echo "Examples:"
            echo "  $0 lint"
            echo "  $0 lint origin/main..HEAD"
            echo "  $0 lint HEAD~10..HEAD"
            ;;
        "cz")
            echo "cccp.sh cz - Soviet Assembly Line (Interactive Commit Wizard)"
            echo "============================================================"
            echo ""
            echo "Launches the interactive terminal wizard to assemble a Conventional"
            echo "Commit with step-by-step prompts for type, scope, subject, breaking change,"
            echo "and body description."
            echo ""
            echo "Usage:"
            echo "  $0 cz"
            echo "  $0 commit -i"
            ;;
        "completion")
            echo "cccp.sh completion - Teleprinter Telegraph (Shell Autocompletion)"
            echo "================================================================"
            echo ""
            echo "Generates tab-completion scripts for Bash, Zsh, and Fish shells."
            echo ""
            echo "Usage:"
            echo "  $0 completion [bash|zsh|fish]"
            echo ""
            echo "Installation:"
            echo "  Bash:"
            echo "    eval \"\$(cccp completion bash)\""
            echo "    # Or persist in ~/.bashrc:"
            echo "    cccp completion bash > ~/.bash_completion.d/cccp"
            echo ""
            echo "  Zsh:"
            echo "    cccp completion zsh > \"\${fpath[1]}/_cccp\""
            echo ""
            echo "  Fish:"
            echo "    cccp completion fish > ~/.config/fish/completions/cccp.fish"
            ;;
        "hooks")
            echo "cccp.sh hooks - Git Hooks Inspectorate and Diffing"
            echo "=================================================="
            echo ""
            echo "Deeply inspects repository Git hooks, diagnoses custom/non-CCCP implementations"
            echo "(Husky, Lefthook, pre-commit, custom scripts), detects backups, and displays"
            echo "unified diffs against the canonical CCCP wrapper."
            echo ""
            echo "Usage:"
            echo "  $0 hooks                  # Run comprehensive audit on all hooks"
            echo "  $0 hooks audit            # Explicit audit of hooks health and frameworks"
            echo "  $0 hooks diff [hook]      # Show unified diff against canonical CCCP wrapper"
            echo ""
            echo "Actions:"
            echo "  audit   Inspect synchronization, detect frameworks (Husky/Lefthook), verify backups"
            echo "  diff    Generate diff against standard wrapper with intervention guidance"
            ;;
        "codeberg")
            echo "cccp.sh codeberg - Codeberg Integration Setup"
            echo "============================================="
            echo ""
            echo "Interactive wizard that detects your Codeberg remote, enables post-push syncing"
            echo "of tickets and AGENTS.md, and walks you through configuring every template variable."
            echo ""
            echo "Usage:"
            echo "  $0 codeberg setup [--url <template-url>]"
            echo ""
            echo "Options:"
            echo "  -u, --url <url>   URL of the AGENTS template to use."
            echo "                    Falls back to agent_md_url in .cccprc, then the built-in default."
            echo "  -h, --help        Show this help message"
            echo ""
            echo "What setup does:"
            echo "  1. Detects the Codeberg owner/repo from the git remote 'origin'."
            echo "     Aborts if the remote is not a Codeberg URL."
            echo "  2. Saves codeberg_owner and codeberg_repo to .cccprc."
            echo "  3. Enables 'ticket' in post_push_sync so issues are synced on every push."
            echo "  4. Fetches the AGENTS template and lists all customizable variables."
            echo "  5. Prompts interactively for each variable not yet configured."
            echo "     Empty values are not accepted."
            echo "  6. Enables 'agent' in post_push_sync so AGENTS.md is regenerated on every push."
            echo ""
            echo "Example:"
            echo "  $0 codeberg setup"
            echo "  $0 codeberg setup --url https://example.com/my-agents-template.md"
            echo ""
            echo "After setup, every 'cccp push' will automatically:"
            echo "  • Sync open Codeberg issues → .antigravity/tickets.md"
            echo "  • Regenerate AGENTS.md from the template → .antigravity/AGENTS.md"
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
            echo "  status             Inspect repository health and Gosplan state diagnostics"
            echo "  lint [range]       Lint commit messages across a git revision range"
            echo "  cz, commit -i      Interactive terminal wizard for creating commits"
            echo "  completion [shell] Generate shell autocompletion (bash, zsh, fish)"
            echo "  commit <message>    Create a commit with a conventional commit message"
            echo "  install            Install git hooks for commit message validation"
            echo "  hooks [audit|diff] Inspect git hooks provenance, detect frameworks, and diff wrappers"
            echo "  config             Manage global or repository configuration"
            echo "  version            Generate version information file"
            echo "  tag [version]      Create release tag, update VERSION and CHANGELOG"
            echo "  push [args...]     Execute git push and trigger optional post-push hook"
            echo "  tickets [options]  Sync Codeberg issues into markdown within strict context limit"
            echo "  agent [options]    Generate/update AGENTS.md directives with local variable interpolation"
            echo "  codeberg setup     Interactive wizard to configure Codeberg integration and AGENTS template"
            echo "  set-av <var> <val> Fast shortcut to configure local agent variable in .cccprc"
            echo "  changelog          Generate or update CHANGELOG.md"
            echo "  update             Update the script to the latest version"
            echo "  check-update       Inspect remote version, release channel, and pin status"
            echo "  help [command]     Show this help message or deep help on a command"
            echo ""
            echo "Git Hooks:"
            echo "  commit-msg            Validates commit messages for conventional commit format"
            echo "  post-commit           Automatically updates version after commit"
            echo "  pre-push              Verifies release tags point to commits with matching VERSION"
            echo "  reference-transaction Intercepts git tag to prevent tagging with unsynchronized VERSION"
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
            echo "Environment Variables:"
            echo "  (All behavior flags default to 0 for inert/standard behavior)"
            echo "  STRICT_SCOPES             - Set to 1 to enforce only listed scopes (legacy: ALLOW_ANY_SCOPE=0)"
            echo "  STRICT_SUBSCOPES          - Set to 1 to enforce only listed subscopes (legacy: ALLOW_ANY_SUBSCOPE=0)"
            echo "  DISABLE_SUBSCOPES         - Set to 1 to disable subscopes"
            echo "  DISABLE_MULTIPLE_SCOPES   - Set to 1 to disable multiple scopes"
            echo "  DEFAULT_BASE_VERSION      - Fallback base version when no tags exist (default: 0.0.1)"
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
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    message=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --strict-scopes)
                STRICT_SCOPES=1
                ALLOW_ANY_SCOPE=0
                shift
                ;;
            --allow-any-scope)
                STRICT_SCOPES=0
                ALLOW_ANY_SCOPE=1
                shift
                ;;
            --strict-subscopes)
                STRICT_SUBSCOPES=1
                ALLOW_ANY_SUBSCOPE=0
                shift
                ;;
            --allow-any-subscope)
                STRICT_SUBSCOPES=0
                ALLOW_ANY_SUBSCOPE=1
                shift
                ;;
            --disable-subscopes)
                DISABLE_SUBSCOPES=1
                shift
                ;;
            --enable-subscopes)
                DISABLE_SUBSCOPES=0
                shift
                ;;
            --disable-multiple-scopes)
                DISABLE_MULTIPLE_SCOPES=1
                shift
                ;;
            --enable-multiple-scopes)
                DISABLE_MULTIPLE_SCOPES=0
                shift
                ;;
            -i|--interactive)
                interactive_mode=1
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "commit"
                else
                    echo "Usage: $0 commit [options] <message>"
                fi
                return 0
                ;;
            *)
                if [ -z "$message" ]; then
                    message="$1"
                else
                    echo "Error: Unexpected argument '$1'" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    # If interactive mode requested, launch wizard
    if [ "${interactive_mode:-0}" = "1" ]; then
        if command -v interactive_commit >/dev/null 2>&1; then
            interactive_commit
            return $?
        else
            echo "Error: Interactive wizard is not available" >&2
            return 1
        fi
    fi

    # If no message provided, show usage
    if [ -z "$message" ]; then
        echo "Usage: $0 commit [options] <message>"
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
# Config Manager Functions
# =============================================================================
get_global_config_dir() {
    echo "${XDG_CONFIG_HOME:-$HOME/.config}/cccp"
}

get_global_config_file() {
    echo "$(get_global_config_dir)/config"
}

get_local_config_file() {
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -n "$git_root" ]; then
        echo "$git_root/.cccprc"
    else
        echo ""
    fi
}

normalize_config_key() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | tr '-' '_'
}

get_default_config_value() {
    case "$1" in
        types|commit_types) echo "feat fix perf refactor revert chore build ci docs ops style test merge" ;;
        scopes|commit_scopes) echo "ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build" ;;
        subscopes|commit_subscopes) echo "components pages services utils auth models views controllers handlers" ;;
        strict_types) echo "1" ;;
        strict_scopes) echo "0" ;;
        strict_subscopes) echo "0" ;;
        disable_subscopes) echo "0" ;;
        disable_multiple_scopes) echo "0" ;;
        default_base_version) echo "0.0.1" ;;
        no_v) echo "0" ;;
        update_channel) echo "stable" ;;
        update_interval_days) echo "30" ;;
        check_updates) echo "1" ;;
        pinned_version|pin_version) echo "" ;;
        post_push_hook) echo "" ;;
        tickets_file) echo ".antigravity/tickets.md" ;;
        agent_md_url|agents_md_url) echo "https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md" ;;
        agent_md_file|agents_md_file) echo ".antigravity/AGENTS.md" ;;
        allow_any_scope) echo "1" ;;
        allow_any_subscope) echo "1" ;;
        *) return 1 ;;
    esac
}

is_global_only_config_key() {
    case "$1" in
        update_channel|update_interval_days|check_updates|pinned_version|pin_version)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

get_type_description() {
    target_type="$1"
    local_file=$(get_local_config_file 2>/dev/null || true)
    global_file=$(get_global_config_file 2>/dev/null || true)
    custom_desc=""
    if [ -n "$local_file" ] && [ -f "$local_file" ]; then
        custom_desc=$(read_file_key "$local_file" "type_desc_${target_type}" 2>/dev/null || true)
    fi
    if [ -z "$custom_desc" ] && [ -n "$global_file" ] && [ -f "$global_file" ]; then
        custom_desc=$(read_file_key "$global_file" "type_desc_${target_type}" 2>/dev/null || true)
    fi
    if [ -n "$custom_desc" ]; then
        echo "$custom_desc"
        return 0
    fi

    case "$target_type" in
        feat) echo "A new feature for the collective" ;;
        fix) echo "A bug fix (correcting imperialist sabotage)" ;;
        docs) echo "Documentation updates or State archives" ;;
        style) echo "Code style/formatting changes (no logic changes)" ;;
        refactor) echo "Code refactoring without changing functionality" ;;
        perf) echo "Performance optimization (Stakhanovite efficiency)" ;;
        test) echo "Adding or correcting tests" ;;
        build) echo "Changes affecting build system or external dependencies" ;;
        ci) echo "Continuous integration / automated factory pipelines" ;;
        chore) echo "Routine maintenance tasks and housekeeping" ;;
        revert) echo "Reverting a previous state decree" ;;
        ops) echo "Operational or infrastructure directives" ;;
        merge) echo "Merging branches into the unified motherland" ;;
        *) echo "Custom action" ;;
    esac
}

get_scope_description() {
    target_scope="$1"
    local_file=$(get_local_config_file 2>/dev/null || true)
    global_file=$(get_global_config_file 2>/dev/null || true)
    custom_desc=""
    if [ -n "$local_file" ] && [ -f "$local_file" ]; then
        custom_desc=$(read_file_key "$local_file" "scope_desc_${target_scope}" 2>/dev/null || true)
    fi
    if [ -z "$custom_desc" ] && [ -n "$global_file" ] && [ -f "$global_file" ]; then
        custom_desc=$(read_file_key "$global_file" "scope_desc_${target_scope}" 2>/dev/null || true)
    fi
    if [ -n "$custom_desc" ]; then
        echo "$custom_desc"
        return 0
    fi

    case "$target_scope" in
        core) echo "Core engine and foundational machinery" ;;
        cli) echo "Command-line interface and terminal protocols" ;;
        config) echo "Configuration, Gosplan directives and rc files" ;;
        ui) echo "User interface components and display" ;;
        api) echo "API interfaces and inter-service communications" ;;
        auth) echo "Authentication, security and clearance checks" ;;
        db) echo "Database and data archives" ;;
        docs) echo "Documentation and user manuals" ;;
        test) echo "Test harnesses, specs and quality verification" ;;
        build) echo "Build manifests, packaging and compilation" ;;
        docker) echo "Containerization and isolation silos" ;;
        updater) echo "Self-updating pipeline and distribution" ;;
        *) echo "Custom scope" ;;
    esac
}

read_file_key() {
    file="$1"
    raw_key="$2"
    key=$(normalize_config_key "$raw_key")

    if [ ! -f "$file" ]; then
        return 1
    fi

    found=0
    result_val=""
    while IFS= read -r line || [ -n "$line" ]; do
        # Ignore comments and empty lines
        case "$line" in
            [#\;]*|"") continue ;;
        esac

        case "$line" in
            *"="*)
                k="${line%%=*}"
                k=$(echo "$k" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
                if [ "$k" = "$key" ]; then
                    val="${line#*=}"
                    val=$(echo "$val" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                    result_val="$val"
                    found=1
                    break
                fi
                ;;
        esac
    done < "$file"

    if [ "$found" -eq 1 ]; then
        echo "$result_val"
        return 0
    fi
    return 1
}

write_file_key() {
    file="$1"
    raw_key="$2"
    value="$3"
    key=$(normalize_config_key "$raw_key")

    dir=$(dirname "$file")
    mkdir -p "$dir"

    if [ ! -f "$file" ]; then
        echo "$key = $value" > "$file"
        return 0
    fi

    tmp_file="${file}.tmp.$$"
    touch "$tmp_file"
    replaced=0

    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            [#\;]*|"")
                echo "$line" >> "$tmp_file"
                continue
                ;;
        esac

        case "$line" in
            *"="*)
                k="${line%%=*}"
                k=$(echo "$k" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
                if [ "$k" = "$key" ] && [ "$replaced" -eq 0 ]; then
                    echo "$key = $value" >> "$tmp_file"
                    replaced=1
                    continue
                fi
                ;;
        esac
        echo "$line" >> "$tmp_file"
    done < "$file"

    if [ "$replaced" -eq 0 ]; then
        echo "$key = $value" >> "$tmp_file"
    fi

    mv "$tmp_file" "$file"
}

unset_file_key() {
    file="$1"
    raw_key="$2"
    key=$(normalize_config_key "$raw_key")

    if [ ! -f "$file" ]; then
        return 0
    fi

    tmp_file="${file}.tmp.$$"
    touch "$tmp_file"
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            [#\;]*|"")
                echo "$line" >> "$tmp_file"
                continue
                ;;
        esac

        case "$line" in
            *"="*)
                k="${line%%=*}"
                k=$(echo "$k" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
                if [ "$k" = "$key" ]; then
                    continue
                fi
                ;;
        esac
        echo "$line" >> "$tmp_file"
    done < "$file"

    mv "$tmp_file" "$file"
}

list_file_keys() {
    file="$1"
    prefix="$2"

    if [ ! -f "$file" ]; then
        return 0
    fi

    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            [#\;]*|"") continue ;;
        esac

        case "$line" in
            *"="*)
                k="${line%%=*}"
                k=$(echo "$k" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
                val="${line#*=}"
                val=$(echo "$val" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                echo "[$prefix] $k = $val"
                ;;
        esac
    done < "$file"
}

load_hierarchical_config() {
    # Capture any pre-existing environment variables
    env_strict_types="${STRICT_TYPES:-}"
    env_types="${CCCP_TYPES:-}"
    env_scopes="${CCCP_SCOPES:-}"
    env_subscopes="${CCCP_SUBSCOPES:-}"
    env_strict_scopes="${STRICT_SCOPES:-}"
    env_strict_subscopes="${STRICT_SUBSCOPES:-}"
    env_disable_subscopes="${DISABLE_SUBSCOPES:-}"
    env_disable_multi="${DISABLE_MULTIPLE_SCOPES:-}"
    env_base_ver="${DEFAULT_BASE_VERSION:-}"
    env_no_v="${NO_V:-}"
    env_channel="${UPDATE_CHANNEL:-}"
    env_interval="${UPDATE_INTERVAL_DAYS:-}"
    env_check="${CHECK_UPDATES:-}"
    env_pinned="${PINNED_VERSION:-${CCCP_PINNED_VERSION:-}}"
    env_post_push_hook="${POST_PUSH_HOOK:-}"
    env_post_push_sync="${POST_PUSH_SYNC:-}"
    env_post_push_sync_tickets="${POST_PUSH_SYNC_TICKETS:-}"
    env_post_push_sync_agent="${POST_PUSH_SYNC_AGENT:-${POST_PUSH_SYNC_AGENTS:-}}"
    env_tickets_file="${TICKETS_FILE:-${CODEBERG_TICKETS_FILE:-}}"
    env_agent_url="${AGENT_MD_URL:-${AGENTS_MD_URL:-}}"
    env_agent_file="${AGENT_MD_FILE:-${AGENTS_MD_FILE:-}}"
    env_allow_any_scope="${ALLOW_ANY_SCOPE:-}"
    env_allow_any_subscope="${ALLOW_ANY_SUBSCOPE:-}"

    # Default internal values
    STRICT_TYPES="1"
    STRICT_SCOPES="0"
    STRICT_SUBSCOPES="0"
    DISABLE_SUBSCOPES="0"
    DISABLE_MULTIPLE_SCOPES="0"
    DEFAULT_BASE_VERSION="0.0.1"
    NO_V="0"
    UPDATE_CHANNEL="stable"
    UPDATE_INTERVAL_DAYS="30"
    CHECK_UPDATES="1"
    PINNED_VERSION=""
    POST_PUSH_HOOK=""
    POST_PUSH_SYNC=""
    POST_PUSH_SYNC_TICKETS="0"
    POST_PUSH_SYNC_AGENT="0"
    POST_PUSH_SYNC_AGENTS="0"
    TICKETS_FILE=""
    AGENT_MD_URL=""
    AGENTS_MD_URL=""
    AGENT_MD_FILE=""
    AGENTS_MD_FILE=""
    COMMIT_TYPES="${COMMIT_TYPES:-feat fix perf refactor revert chore build ci docs ops style test merge}"
    COMMIT_SCOPES="${COMMIT_SCOPES:-ui docs api docker db updater micropub indieauth activitypub microsub twtxt webmention theme feeds cli core config auth test build}"
    COMMIT_SUBSCOPES="${COMMIT_SUBSCOPES:-components pages services utils auth models views controllers handlers}"

    global_file=$(get_global_config_file)
    local_file=$(get_local_config_file)

    # 1. Global config file overrides
    if [ -f "$global_file" ]; then
        g_types=$(read_file_key "$global_file" "types" 2>/dev/null || true)
        [ -z "$g_types" ] && g_types=$(read_file_key "$global_file" "commit_types" 2>/dev/null || true)
        [ -n "$g_types" ] && COMMIT_TYPES=$(echo "$g_types" | tr ',' ' ')

        g_scopes=$(read_file_key "$global_file" "scopes" 2>/dev/null || true)
        [ -z "$g_scopes" ] && g_scopes=$(read_file_key "$global_file" "commit_scopes" 2>/dev/null || true)
        [ -n "$g_scopes" ] && COMMIT_SCOPES=$(echo "$g_scopes" | tr ',' ' ')

        g_subscopes=$(read_file_key "$global_file" "subscopes" 2>/dev/null || true)
        [ -z "$g_subscopes" ] && g_subscopes=$(read_file_key "$global_file" "commit_subscopes" 2>/dev/null || true)
        [ -n "$g_subscopes" ] && COMMIT_SUBSCOPES=$(echo "$g_subscopes" | tr ',' ' ')

        g_strict_types=$(read_file_key "$global_file" "strict_types" 2>/dev/null || true)
        [ -n "$g_strict_types" ] && STRICT_TYPES="$g_strict_types"

        g_strict_scopes=$(read_file_key "$global_file" "strict_scopes" 2>/dev/null || true)
        [ -n "$g_strict_scopes" ] && STRICT_SCOPES="$g_strict_scopes"

        g_strict_subscopes=$(read_file_key "$global_file" "strict_subscopes" 2>/dev/null || true)
        [ -n "$g_strict_subscopes" ] && STRICT_SUBSCOPES="$g_strict_subscopes"

        g_allow_any_scope=$(read_file_key "$global_file" "allow_any_scope" 2>/dev/null || true)
        if [ "$g_allow_any_scope" = "0" ]; then STRICT_SCOPES=1; elif [ "$g_allow_any_scope" = "1" ]; then STRICT_SCOPES=0; fi

        g_allow_any_subscope=$(read_file_key "$global_file" "allow_any_subscope" 2>/dev/null || true)
        if [ "$g_allow_any_subscope" = "0" ]; then STRICT_SUBSCOPES=1; elif [ "$g_allow_any_subscope" = "1" ]; then STRICT_SUBSCOPES=0; fi

        g_disable_subscopes=$(read_file_key "$global_file" "disable_subscopes" 2>/dev/null || true)
        [ -n "$g_disable_subscopes" ] && DISABLE_SUBSCOPES="$g_disable_subscopes"

        g_disable_multi=$(read_file_key "$global_file" "disable_multiple_scopes" 2>/dev/null || true)
        [ -n "$g_disable_multi" ] && DISABLE_MULTIPLE_SCOPES="$g_disable_multi"

        g_base_ver=$(read_file_key "$global_file" "default_base_version" 2>/dev/null || true)
        [ -n "$g_base_ver" ] && DEFAULT_BASE_VERSION="$g_base_ver"

        g_no_v=$(read_file_key "$global_file" "no_v" 2>/dev/null || true)
        [ -n "$g_no_v" ] && NO_V="$g_no_v"

        g_channel=$(read_file_key "$global_file" "update_channel" 2>/dev/null || true)
        [ -n "$g_channel" ] && UPDATE_CHANNEL="$g_channel"

        g_interval=$(read_file_key "$global_file" "update_interval_days" 2>/dev/null || true)
        [ -n "$g_interval" ] && UPDATE_INTERVAL_DAYS="$g_interval"

        g_check=$(read_file_key "$global_file" "check_updates" 2>/dev/null || true)
        [ -n "$g_check" ] && CHECK_UPDATES="$g_check"

        g_pinned=$(read_file_key "$global_file" "pinned_version" 2>/dev/null || true)
        [ -z "$g_pinned" ] && g_pinned=$(read_file_key "$global_file" "pin_version" 2>/dev/null || true)
        [ -n "$g_pinned" ] && PINNED_VERSION="$g_pinned"

        g_post_push=$(read_file_key "$global_file" "post_push_hook" 2>/dev/null || true)
        [ -n "$g_post_push" ] && POST_PUSH_HOOK="$g_post_push"

        g_pp_sync=$(read_file_key "$global_file" "post_push_sync" 2>/dev/null || true)
        [ -n "$g_pp_sync" ] && POST_PUSH_SYNC="$g_pp_sync"

        g_pp_tickets=$(read_file_key "$global_file" "post_push_sync_tickets" 2>/dev/null || true)
        [ -z "$g_pp_tickets" ] && g_pp_tickets=$(read_file_key "$global_file" "sync_tickets" 2>/dev/null || true)
        [ -n "$g_pp_tickets" ] && POST_PUSH_SYNC_TICKETS="$g_pp_tickets"

        g_pp_agents=$(read_file_key "$global_file" "post_push_sync_agent" 2>/dev/null || true)
        [ -z "$g_pp_agents" ] && g_pp_agents=$(read_file_key "$global_file" "post_push_sync_agents" 2>/dev/null || true)
        [ -z "$g_pp_agents" ] && g_pp_agents=$(read_file_key "$global_file" "sync_agent" 2>/dev/null || true)
        [ -z "$g_pp_agents" ] && g_pp_agents=$(read_file_key "$global_file" "sync_agents" 2>/dev/null || true)
        [ -n "$g_pp_agents" ] && POST_PUSH_SYNC_AGENT="$g_pp_agents" && POST_PUSH_SYNC_AGENTS="$g_pp_agents"

        g_tickets=$(read_file_key "$global_file" "tickets_file" 2>/dev/null || true)
        [ -n "$g_tickets" ] && TICKETS_FILE="$g_tickets"

        g_ag_url=$(read_file_key "$global_file" "agent_md_url" 2>/dev/null || true)
        [ -z "$g_ag_url" ] && g_ag_url=$(read_file_key "$global_file" "agents_md_url" 2>/dev/null || true)
        [ -z "$g_ag_url" ] && g_ag_url=$(read_file_key "$global_file" "agent_url" 2>/dev/null || true)
        [ -z "$g_ag_url" ] && g_ag_url=$(read_file_key "$global_file" "agent_var_url" 2>/dev/null || true)
        [ -n "$g_ag_url" ] && AGENT_MD_URL="$g_ag_url" && AGENTS_MD_URL="$g_ag_url"

        g_ag_file=$(read_file_key "$global_file" "agent_md_file" 2>/dev/null || true)
        [ -z "$g_ag_file" ] && g_ag_file=$(read_file_key "$global_file" "agents_md_file" 2>/dev/null || true)
        [ -n "$g_ag_file" ] && AGENT_MD_FILE="$g_ag_file" && AGENTS_MD_FILE="$g_ag_file"
    fi

    # 2. Local repository config (.cccprc) overrides global
    if [ -n "$local_file" ] && [ -f "$local_file" ]; then
        l_types=$(read_file_key "$local_file" "types" 2>/dev/null || true)
        [ -z "$l_types" ] && l_types=$(read_file_key "$local_file" "commit_types" 2>/dev/null || true)
        [ -n "$l_types" ] && COMMIT_TYPES=$(echo "$l_types" | tr ',' ' ')

        l_scopes=$(read_file_key "$local_file" "scopes" 2>/dev/null || true)
        [ -z "$l_scopes" ] && l_scopes=$(read_file_key "$local_file" "commit_scopes" 2>/dev/null || true)
        [ -n "$l_scopes" ] && COMMIT_SCOPES=$(echo "$l_scopes" | tr ',' ' ')

        l_subscopes=$(read_file_key "$local_file" "subscopes" 2>/dev/null || true)
        [ -z "$l_subscopes" ] && l_subscopes=$(read_file_key "$local_file" "commit_subscopes" 2>/dev/null || true)
        [ -n "$l_subscopes" ] && COMMIT_SUBSCOPES=$(echo "$l_subscopes" | tr ',' ' ')

        l_strict_types=$(read_file_key "$local_file" "strict_types" 2>/dev/null || true)
        [ -n "$l_strict_types" ] && STRICT_TYPES="$l_strict_types"

        l_strict_scopes=$(read_file_key "$local_file" "strict_scopes" 2>/dev/null || true)
        [ -n "$l_strict_scopes" ] && STRICT_SCOPES="$l_strict_scopes"

        l_strict_subscopes=$(read_file_key "$local_file" "strict_subscopes" 2>/dev/null || true)
        [ -n "$l_strict_subscopes" ] && STRICT_SUBSCOPES="$l_strict_subscopes"

        l_allow_any_scope=$(read_file_key "$local_file" "allow_any_scope" 2>/dev/null || true)
        if [ "$l_allow_any_scope" = "0" ]; then STRICT_SCOPES=1; elif [ "$l_allow_any_scope" = "1" ]; then STRICT_SCOPES=0; fi

        l_allow_any_subscope=$(read_file_key "$local_file" "allow_any_subscope" 2>/dev/null || true)
        if [ "$l_allow_any_subscope" = "0" ]; then STRICT_SUBSCOPES=1; elif [ "$l_allow_any_subscope" = "1" ]; then STRICT_SUBSCOPES=0; fi

        l_disable_subscopes=$(read_file_key "$local_file" "disable_subscopes" 2>/dev/null || true)
        [ -n "$l_disable_subscopes" ] && DISABLE_SUBSCOPES="$l_disable_subscopes"

        l_disable_multi=$(read_file_key "$local_file" "disable_multiple_scopes" 2>/dev/null || true)
        [ -n "$l_disable_multi" ] && DISABLE_MULTIPLE_SCOPES="$l_disable_multi"

        l_base_ver=$(read_file_key "$local_file" "default_base_version" 2>/dev/null || true)
        [ -n "$l_base_ver" ] && DEFAULT_BASE_VERSION="$l_base_ver"

        l_no_v=$(read_file_key "$local_file" "no_v" 2>/dev/null || true)
        [ -n "$l_no_v" ] && NO_V="$l_no_v"

        l_post_push=$(read_file_key "$local_file" "post_push_hook" 2>/dev/null || true)
        [ -n "$l_post_push" ] && POST_PUSH_HOOK="$l_post_push"

        l_pp_sync=$(read_file_key "$local_file" "post_push_sync" 2>/dev/null || true)
        [ -n "$l_pp_sync" ] && POST_PUSH_SYNC="$l_pp_sync"

        l_pp_tickets=$(read_file_key "$local_file" "post_push_sync_tickets" 2>/dev/null || true)
        [ -z "$l_pp_tickets" ] && l_pp_tickets=$(read_file_key "$local_file" "sync_tickets" 2>/dev/null || true)
        [ -n "$l_pp_tickets" ] && POST_PUSH_SYNC_TICKETS="$l_pp_tickets"

        l_pp_agents=$(read_file_key "$local_file" "post_push_sync_agent" 2>/dev/null || true)
        [ -z "$l_pp_agents" ] && l_pp_agents=$(read_file_key "$local_file" "post_push_sync_agents" 2>/dev/null || true)
        [ -z "$l_pp_agents" ] && l_pp_agents=$(read_file_key "$local_file" "sync_agent" 2>/dev/null || true)
        [ -z "$l_pp_agents" ] && l_pp_agents=$(read_file_key "$local_file" "sync_agents" 2>/dev/null || true)
        [ -n "$l_pp_agents" ] && POST_PUSH_SYNC_AGENT="$l_pp_agents" && POST_PUSH_SYNC_AGENTS="$l_pp_agents"

        l_tickets=$(read_file_key "$local_file" "tickets_file" 2>/dev/null || true)
        [ -n "$l_tickets" ] && TICKETS_FILE="$l_tickets"

        l_ag_url=$(read_file_key "$local_file" "agent_md_url" 2>/dev/null || true)
        [ -z "$l_ag_url" ] && l_ag_url=$(read_file_key "$local_file" "agents_md_url" 2>/dev/null || true)
        [ -z "$l_ag_url" ] && l_ag_url=$(read_file_key "$local_file" "agent_url" 2>/dev/null || true)
        [ -z "$l_ag_url" ] && l_ag_url=$(read_file_key "$local_file" "agent_var_url" 2>/dev/null || true)
        [ -n "$l_ag_url" ] && AGENT_MD_URL="$l_ag_url" && AGENTS_MD_URL="$l_ag_url"

        l_ag_file=$(read_file_key "$local_file" "agent_md_file" 2>/dev/null || true)
        [ -z "$l_ag_file" ] && l_ag_file=$(read_file_key "$local_file" "agents_md_file" 2>/dev/null || true)
        [ -n "$l_ag_file" ] && AGENT_MD_FILE="$l_ag_file" && AGENTS_MD_FILE="$l_ag_file"
    fi

    # 3. Environment variables take highest precedence
    [ -n "$env_strict_types" ] && STRICT_TYPES="$env_strict_types"
    [ -n "$env_types" ] && COMMIT_TYPES="$env_types"
    [ -n "$env_scopes" ] && COMMIT_SCOPES="$env_scopes"
    [ -n "$env_subscopes" ] && COMMIT_SUBSCOPES="$env_subscopes"
    [ -n "$env_strict_scopes" ] && STRICT_SCOPES="$env_strict_scopes"
    [ -n "$env_strict_subscopes" ] && STRICT_SUBSCOPES="$env_strict_subscopes"
    [ -n "$env_disable_subscopes" ] && DISABLE_SUBSCOPES="$env_disable_subscopes"
    [ -n "$env_disable_multi" ] && DISABLE_MULTIPLE_SCOPES="$env_disable_multi"
    [ -n "$env_base_ver" ] && DEFAULT_BASE_VERSION="$env_base_ver"
    [ -n "$env_no_v" ] && NO_V="$env_no_v"
    [ -n "$env_channel" ] && UPDATE_CHANNEL="$env_channel"
    [ -n "$env_interval" ] && UPDATE_INTERVAL_DAYS="$env_interval"
    [ -n "$env_check" ] && CHECK_UPDATES="$env_check"
    [ -n "$env_pinned" ] && PINNED_VERSION="$env_pinned"
    [ -n "$env_post_push_hook" ] && POST_PUSH_HOOK="$env_post_push_hook"
    [ -n "$env_post_push_sync" ] && POST_PUSH_SYNC="$env_post_push_sync"
    [ -n "$env_post_push_sync_tickets" ] && POST_PUSH_SYNC_TICKETS="$env_post_push_sync_tickets"
    [ -n "$env_post_push_sync_agent" ] && POST_PUSH_SYNC_AGENT="$env_post_push_sync_agent" && POST_PUSH_SYNC_AGENTS="$env_post_push_sync_agent"
    [ -n "$env_tickets_file" ] && TICKETS_FILE="$env_tickets_file"
    [ -n "$env_agent_url" ] && AGENT_MD_URL="$env_agent_url" && AGENTS_MD_URL="$env_agent_url"
    [ -n "$env_agent_file" ] && AGENT_MD_FILE="$env_agent_file" && AGENTS_MD_FILE="$env_agent_file"

    # 4. Final defaults if still unset
    if [ -z "$AGENT_MD_URL" ]; then
        AGENT_MD_URL="https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md"
        AGENTS_MD_URL="$AGENT_MD_URL"
    fi
    if [ -z "$AGENT_MD_FILE" ]; then
        AGENT_MD_FILE=".antigravity/AGENTS.md"
        AGENTS_MD_FILE="$AGENT_MD_FILE"
    fi

    export PINNED_VERSION
    export POST_PUSH_HOOK
    export POST_PUSH_SYNC
    export POST_PUSH_SYNC_TICKETS
    export POST_PUSH_SYNC_AGENT
    export POST_PUSH_SYNC_AGENTS
    export TICKETS_FILE
    export AGENT_MD_URL
    export AGENTS_MD_URL
    export AGENT_MD_FILE
    export AGENTS_MD_FILE

    # Legacy environment overrides
    if [ -z "$env_strict_scopes" ]; then
        if [ "$env_allow_any_scope" = "0" ]; then
            STRICT_SCOPES=1
        elif [ "$env_allow_any_scope" = "1" ]; then
            STRICT_SCOPES=0
        fi
    fi

    if [ -z "$env_strict_subscopes" ]; then
        if [ "$env_allow_any_subscope" = "0" ]; then
            STRICT_SUBSCOPES=1
        elif [ "$env_allow_any_subscope" = "1" ]; then
            STRICT_SUBSCOPES=0
        fi
    fi

    ALLOW_ANY_SCOPE=$([ "$STRICT_SCOPES" = "1" ] && echo "0" || echo "1")
    ALLOW_ANY_SUBSCOPE=$([ "$STRICT_SUBSCOPES" = "1" ] && echo "0" || echo "1")
}

cmd_config() {
    is_global=0
    is_local=0
    is_list=0
    is_unset=0
    key=""
    value=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --global|-g)
                is_global=1
                shift
                ;;
            --local)
                is_local=1
                shift
                ;;
            --list|-l)
                is_list=1
                shift
                ;;
            --unset)
                is_unset=1
                shift
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "config"
                else
                    echo "Usage: cccp config [--global|--local] [--list|--unset <key>|<key> [value]]"
                fi
                return 0
                ;;
            *)
                if [ -z "$key" ]; then
                    key="$1"
                elif [ -z "$value" ]; then
                    value="$1"
                else
                    echo "Error: Unexpected argument '$1'" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    global_file=$(get_global_config_file)
    local_file=$(get_local_config_file)

    # 1. Listing configs
    if [ $is_list -eq 1 ]; then
        if [ $is_global -eq 1 ]; then
            list_file_keys "$global_file" "global"
        elif [ $is_local -eq 1 ]; then
            if [ -z "$local_file" ]; then
                echo "Error: Not in a git repository. Cannot list local config." >&2
                return 1
            fi
            list_file_keys "$local_file" "local"
        else
            list_file_keys "$global_file" "global"
            if [ -n "$local_file" ]; then
                list_file_keys "$local_file" "local"
            fi
        fi
        return 0
    fi

    # 2. Key must be provided for get, set, or unset
    if [ -z "$key" ]; then
        echo "Error: No configuration key specified." >&2
        echo "Run '$0 help config' for usage information." >&2
        return 1
    fi

    norm_key=$(normalize_config_key "$key")

    # Update directives are strictly machine/user global
    if is_global_only_config_key "$norm_key"; then
        if [ $is_local -eq 1 ]; then
            echo "Error: Configuration key '$key' governs system tool updates and cannot be set locally. Use --global." >&2
            return 1
        fi
        if [ -n "$value" ] || [ $is_unset -eq 1 ]; then
            is_global=1
        fi
    fi

    # 3. Unsetting a key
    if [ $is_unset -eq 1 ]; then
        if [ $is_global -eq 1 ]; then
            unset_file_key "$global_file" "$norm_key"
            echo "Unset global $norm_key"
        elif [ $is_local -eq 1 ]; then
            if [ -z "$local_file" ]; then
                echo "Error: Not in a git repository." >&2
                return 1
            fi
            unset_file_key "$local_file" "$norm_key"
            echo "Unset local $norm_key"
        else
            if [ -n "$local_file" ] && [ -f "$local_file" ] && grep -qE "^[[:space:]]*${norm_key}[[:space:]]*=" "$local_file" 2>/dev/null; then
                unset_file_key "$local_file" "$norm_key"
                echo "Unset local $norm_key"
            else
                unset_file_key "$global_file" "$norm_key"
                echo "Unset global $norm_key"
            fi
        fi
        return 0
    fi

    # 4. Setting a value
    if [ -n "$value" ]; then
        if [ $is_global -eq 1 ]; then
            write_file_key "$global_file" "$norm_key" "$value"
            echo "Set global $norm_key = $value"
        elif [ $is_local -eq 1 ]; then
            if [ -z "$local_file" ]; then
                echo "Error: Not in a git repository. Cannot set local config." >&2
                return 1
            fi
            write_file_key "$local_file" "$norm_key" "$value"
            echo "Set local $norm_key = $value"
        else
            if [ -z "$local_file" ]; then
                # Outside a git repo, default to global
                write_file_key "$global_file" "$norm_key" "$value"
                echo "Set global $norm_key = $value"
            else
                write_file_key "$local_file" "$norm_key" "$value"
                echo "Set local $norm_key = $value"
            fi
        fi
        return 0
    fi

    # 5. Getting a value
    if [ $is_global -eq 1 ]; then
        val=$(read_file_key "$global_file" "$norm_key" 2>/dev/null || true)
        if [ -n "$val" ]; then
            echo "$val"
            return 0
        else
            echo "Error: Configuration key '$key' not found in global config." >&2
            return 1
        fi
    elif [ $is_local -eq 1 ]; then
        if [ -z "$local_file" ]; then
            echo "Error: Not in a git repository." >&2
            return 1
        fi
        val=$(read_file_key "$local_file" "$norm_key" 2>/dev/null || true)
        if [ -n "$val" ]; then
            echo "$val"
            return 0
        else
            echo "Error: Configuration key '$key' not found in local config." >&2
            return 1
        fi
    else
        # Cascaded get: local first (if not global-only key), then global, then default
        val=""
        if ! is_global_only_config_key "$norm_key" && [ -n "$local_file" ] && [ -f "$local_file" ]; then
            val=$(read_file_key "$local_file" "$norm_key" 2>/dev/null || true)
        fi
        if [ -z "$val" ] && [ -f "$global_file" ]; then
            val=$(read_file_key "$global_file" "$norm_key" 2>/dev/null || true)
        fi
        if [ -z "$val" ]; then
            val=$(get_default_config_value "$norm_key" 2>/dev/null || true)
        fi

        if [ -n "$val" ]; then
            echo "$val"
            return 0
        else
            echo "Error: Configuration key '$key' not found." >&2
            return 1
        fi
    fi
}


# =============================================================================
# Update Functions
# =============================================================================
get_current_version() {
    echo "${CCCP_VERSION:-2.0.0}"
}

get_update_cache_file() {
    if command -v get_global_config_dir >/dev/null 2>&1; then
        echo "$(get_global_config_dir)/update_cache"
    else
        echo "${XDG_CONFIG_HOME:-$HOME/.config}/cccp/update_cache"
    fi
}

check_hook_version() {
    [ -z "${GIT_ROOT:-}" ] && return 0
    hooks_dir="${GIT_HOOKS_DIR:-$GIT_ROOT/.git/hooks}"
    [ ! -d "$hooks_dir" ] && return 0

    curr_ver=$(get_current_version)

    for hook in commit-msg post-commit; do
        hook_file="$hooks_dir/$hook"
        if [ -f "$hook_file" ]; then
            hook_ver=$(sed -n 's/^# cccp-hook-version:[[:space:]]*//p' "$hook_file" 2>/dev/null | head -n 1)
            if [ -n "$hook_ver" ] && [ "$hook_ver" != "$curr_ver" ]; then
                echo "[cccp] Warning: Git hook '$hook' was installed with cccp v$hook_ver (current: v$curr_ver)." >&2
                echo "[cccp] Run 'cccp install' to synchronize git hooks with your current cccp version." >&2
                return 0
            fi
        fi
    done
    return 0
}

semver_is_newer() {
    remote="$1"
    current="$2"

    r="${remote#v}"
    c="${current#v}"

    r_base=$(echo "$r" | cut -d- -f1 | cut -d+ -f1)
    c_base=$(echo "$c" | cut -d- -f1 | cut -d+ -f1)

    r_maj=$(echo "$r_base" | cut -d. -f1 | tr -cd '0-9'); r_maj=${r_maj:-0}
    r_min=$(echo "$r_base" | cut -d. -f2 | tr -cd '0-9'); r_min=${r_min:-0}
    r_pat=$(echo "$r_base" | cut -d. -f3 | tr -cd '0-9'); r_pat=${r_pat:-0}

    c_maj=$(echo "$c_base" | cut -d. -f1 | tr -cd '0-9'); c_maj=${c_maj:-0}
    c_min=$(echo "$c_base" | cut -d. -f2 | tr -cd '0-9'); c_min=${c_min:-0}
    c_pat=$(echo "$c_base" | cut -d. -f3 | tr -cd '0-9'); c_pat=${c_pat:-0}

    if [ "$r_maj" -gt "$c_maj" ]; then return 0; fi
    if [ "$r_maj" -lt "$c_maj" ]; then return 1; fi

    if [ "$r_min" -gt "$c_min" ]; then return 0; fi
    if [ "$r_min" -lt "$c_min" ]; then return 1; fi

    if [ "$r_pat" -gt "$c_pat" ]; then return 0; fi
    if [ "$r_pat" -lt "$c_pat" ]; then return 1; fi

    if echo "$c" | grep -q -- "-" && ! echo "$r" | grep -q -- "-"; then
        return 0
    fi

    return 1
}

check_auto_update() {
    # Check if update checks are enabled or version is pinned
    if [ "${CHECK_UPDATES:-1}" = "0" ] || [ -n "${PINNED_VERSION:-}" ]; then
        return 0
    fi

    # Require curl or wget
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        return 0
    fi

    cache_file=$(get_update_cache_file)
    interval_days="${UPDATE_INTERVAL_DAYS:-30}"
    interval_sec=$(( interval_days * 86400 ))

    now=$(date +%s 2>/dev/null || true)
    [ -z "$now" ] && return 0

    last_check=0
    cached_latest=""

    if [ -f "$cache_file" ]; then
        last_check=$(grep '^last_check_timestamp=' "$cache_file" 2>/dev/null | cut -d= -f2 | tr -d '[:space:]' || true)
        cached_latest=$(grep '^latest_version=' "$cache_file" 2>/dev/null | cut -d= -f2 | tr -d '[:space:]' || true)
    fi
    last_check="${last_check:-0}"

    curr_ver=$(get_current_version)

    # If interval expired, check remote version
    if [ $(( now - last_check )) -ge $interval_sec ]; then
        channel="${UPDATE_CHANNEL:-stable}"
        remote_ver=""

        if [ "$channel" = "nightly" ]; then
            remote_ver="nightly"
        else
            api_url="https://api.github.com/repos/lumenpink/cccp.sh/releases/latest"
            response=""
            if command -v curl >/dev/null 2>&1; then
                response=$(curl -s --max-time 2 "$api_url" 2>/dev/null || true)
            elif command -v wget >/dev/null 2>&1; then
                response=$(wget -q -T 2 -O- "$api_url" 2>/dev/null || true)
            fi

            if [ -n "$response" ]; then
                remote_ver=$(echo "$response" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1 | sed 's/^v//')
            fi
        fi

        if [ -n "$remote_ver" ]; then
            cached_latest="$remote_ver"
            mkdir -p "$(dirname "$cache_file")"
            cat > "$cache_file" <<EOF
last_check_timestamp=$now
latest_version=$cached_latest
EOF
        fi
    fi

    # If cached latest version is newer than current version, notify user
    if [ -n "$cached_latest" ]; then
        if [ "$cached_latest" = "nightly" ]; then
            if [ "${UPDATE_CHANNEL:-stable}" = "nightly" ]; then
                echo "[cccp] Notice: Running on nightly channel. Run 'cccp update' to pull latest changes." >&2
            fi
        elif semver_is_newer "$cached_latest" "$curr_ver"; then
            echo "[cccp] Notice: A newer version of cccp is available ($cached_latest vs current $curr_ver)." >&2
            echo "[cccp] Run 'cccp update' to update to the latest ${UPDATE_CHANNEL:-stable} release." >&2
        fi
    fi

    return 0
}

cmd_check_update() {
    case "${1:-}" in
        -h|--help)
            if command -v show_help >/dev/null 2>&1; then
                show_help "check-update"
            else
                echo "Usage: cccp check-update"
            fi
            return 0
            ;;
    esac

    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    curr_ver=$(get_current_version)
    channel="${UPDATE_CHANNEL:-stable}"
    pinned="${PINNED_VERSION:-}"

    echo "★ CCCP Update Verification Bureau ★"
    echo "Installed Version : $curr_ver"
    echo "Release Channel   : $channel"
    if [ -n "$pinned" ]; then
        echo "Pinned Version    : $pinned (Gosplan Directive Active)"
    else
        echo "Pinned Version    : none (tracking latest $channel releases)"
    fi

    remote_ver=""
    if [ "$channel" = "nightly" ]; then
        remote_ver="nightly"
    else
        api_url="https://api.github.com/repos/lumenpink/cccp.sh/releases/latest"
        response=""
        if command -v curl >/dev/null 2>&1; then
            response=$(curl -s --max-time 3 "$api_url" 2>/dev/null || true)
        elif command -v wget >/dev/null 2>&1; then
            response=$(wget -q -T 3 -O- "$api_url" 2>/dev/null || true)
        fi

        if [ -n "$response" ]; then
            remote_ver=$(echo "$response" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1 | sed 's/^v//')
        fi
    fi

    if [ -z "$remote_ver" ]; then
        echo "Remote Version    : unavailable (unable to reach GitHub API)"
        echo "Status            : Offline or telemetry unreachable. Verify network connection."
        return 0
    fi

    echo "Remote Version    : $remote_ver"

    if [ -n "$pinned" ]; then
        if [ "$pinned" = "$curr_ver" ]; then
            echo "Status            : Pinned to $pinned. Updates are frozen by Gosplan decree."
        else
            echo "Status            : Pinned to $pinned (currently running $curr_ver). Use 'cccp update --pin $pinned' to align or 'cccp update --unpin' to release."
        fi
    elif [ "$channel" = "nightly" ]; then
        echo "Status            : Following nightly stream. Run 'cccp update' to pull latest changes."
    elif semver_is_newer "$remote_ver" "$curr_ver"; then
        echo "Status            : Update available ($curr_ver -> $remote_ver). Execute 'cccp update' to upgrade."
    else
        echo "Status            : Up to date. The collective is operating on the latest standard."
    fi
    return 0
}

update_script() {
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    channel="${UPDATE_CHANNEL:-stable}"
    do_pin=0
    pin_target=""
    do_unpin=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --channel|-c)
                if [ $# -lt 2 ]; then
                    echo "Error: --channel requires an argument (stable or nightly)" >&2
                    return 1
                fi
                channel="$2"
                shift 2
                ;;
            --pin|-p)
                do_pin=1
                if [ $# -ge 2 ] && [ "${2#-}" = "$2" ]; then
                    pin_target="$2"
                    shift 2
                else
                    pin_target="$(get_current_version)"
                    shift 1
                fi
                ;;
            --unpin|-u)
                do_unpin=1
                shift 1
                ;;
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "update"
                else
                    echo "Usage: cccp update [--channel stable|nightly] [--pin [version]] [--unpin]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unexpected argument '$1'" >&2
                return 1
                ;;
        esac
    done

    # Handle unpinning
    if [ "$do_unpin" -eq 1 ]; then
        global_cfg=$(get_global_config_file 2>/dev/null || true)
        if [ -n "$global_cfg" ] && [ -f "$global_cfg" ]; then
            unset_file_key "$global_cfg" "pinned_version" 2>/dev/null || true
            unset_file_key "$global_cfg" "pin_version" 2>/dev/null || true
        fi
        PINNED_VERSION=""
        export PINNED_VERSION
        echo "Gosplan directive lifted: Version pin removed. Tracking $channel releases."
    fi

    # Handle pinning
    if [ "$do_pin" -eq 1 ]; then
        clean_pin="${pin_target#v}"
        global_cfg=$(get_global_config_file 2>/dev/null || true)
        write_file_key "$global_cfg" "pinned_version" "$clean_pin"
        PINNED_VERSION="$clean_pin"
        export PINNED_VERSION
        echo "Gosplan directive enacted: Version pinned to $clean_pin."

        # If pinning to current version, no download required
        if [ "$clean_pin" = "$(get_current_version)" ]; then
            return 0
        fi
    fi

    # Prevent update if version is pinned and neither pin nor unpin was specified
    if [ "$do_pin" -eq 0 ] && [ "$do_unpin" -eq 0 ] && [ -n "${PINNED_VERSION:-}" ]; then
        echo "Error: Version is pinned to $PINNED_VERSION by Gosplan directive." >&2
        echo "To upgrade anyway or release the pin, run 'cccp update --unpin' or 'cccp update --pin <version>'." >&2
        return 1
    fi

    # Determine download URL
    if [ "$do_pin" -eq 1 ] && [ -n "$pin_target" ]; then
        download_url="https://github.com/lumenpink/cccp.sh/releases/download/v${pin_target#v}/cccp.sh"
    elif [ "$channel" = "nightly" ]; then
        download_url="https://github.com/lumenpink/cccp.sh/releases/download/nightly/cccp.sh"
    elif [ "$channel" = "stable" ]; then
        download_url="https://github.com/lumenpink/cccp.sh/releases/latest/download/cccp.sh"
    else
        echo "Error: Invalid update channel '$channel'. Choose 'stable' or 'nightly'." >&2
        return 1
    fi

    # Determine target file to update
    target=""
    if command -v cccp >/dev/null 2>&1; then
        target=$(command -v cccp)
    elif [ -f "${0:-}" ] && [ -w "${0:-}" ]; then
        target="${0:-}"
    elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/cccp.sh" ]; then
        target="$GIT_ROOT/cccp.sh"
    elif [ -f "./cccp.sh" ]; then
        target="./cccp.sh"
    else
        target="${XDG_BIN_HOME:-$HOME/.local/bin}/cccp"
    fi

    echo "Updating cccp from channel '$channel'..."
    echo "Downloading from: $download_url"

    tmp_file="$(mktemp)"
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL "$download_url" -o "$tmp_file" 2>/dev/null; then
            echo "Error: Failed to download update from $download_url" >&2
            rm -f "$tmp_file"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q "$download_url" -O "$tmp_file" 2>/dev/null; then
            echo "Error: Failed to download update from $download_url" >&2
            rm -f "$tmp_file"
            return 1
        fi
    else
        echo "Error: Neither curl nor wget is available." >&2
        rm -f "$tmp_file"
        return 1
    fi

    if [ ! -s "$tmp_file" ]; then
        echo "Error: Downloaded update is empty." >&2
        rm -f "$tmp_file"
        return 1
    fi

    chmod +x "$tmp_file"

    # Backup if target exists
    if [ -f "$target" ]; then
        cp "$target" "$target.bak"
        echo "Backup saved to: $target.bak"
    else
        mkdir -p "$(dirname "$target")"
    fi

    mv "$tmp_file" "$target"
    chmod +x "$target"

    # Refresh update cache timestamp
    cache_file=$(get_update_cache_file)
    now=$(date +%s 2>/dev/null || true)
    if [ -n "$now" ]; then
        mkdir -p "$(dirname "$cache_file")"
        cat > "$cache_file" <<EOF
last_check_timestamp=$now
latest_version=updated
EOF
    fi

    echo "Successfully updated cccp at: $target"
    return 0
}


# =============================================================================
# Status Functions
# =============================================================================
show_status() {
    # Verify git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    git_root=$(git rev-parse --show-toplevel)
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null || echo "detached")

    # Remote tracking info
    upstream=$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || echo "")
    if [ -n "$upstream" ]; then
        ahead=$(git rev-list --count "${upstream}..HEAD" 2>/dev/null || echo "0")
        behind=$(git rev-list --count "HEAD..${upstream}" 2>/dev/null || echo "0")
        if [ "$ahead" -eq 0 ] && [ "$behind" -eq 0 ]; then
            tracking_str="up to date with $upstream"
        elif [ "$ahead" -gt 0 ] && [ "$behind" -eq 0 ]; then
            tracking_str="ahead of $upstream by $ahead commit(s)"
        elif [ "$ahead" -eq 0 ] && [ "$behind" -gt 0 ]; then
            tracking_str="behind $upstream by $behind commit(s)"
        else
            tracking_str="diverged from $upstream (ahead $ahead, behind $behind)"
        fi
    else
        tracking_str="no remote upstream configured"
    fi

    # Working tree counts
    modified_cnt=$(git diff --name-only 2>/dev/null | wc -l | tr -d ' ')
    staged_cnt=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
    untracked_cnt=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l | tr -d ' ')

    if [ "$modified_cnt" -eq 0 ] && [ "$staged_cnt" -eq 0 ] && [ "$untracked_cnt" -eq 0 ]; then
        tree_status="clean (all state directives satisfied)"
    else
        parts=""
        [ "$staged_cnt" -gt 0 ] && parts="${parts}${staged_cnt} staged, "
        [ "$modified_cnt" -gt 0 ] && parts="${parts}${modified_cnt} modified, "
        [ "$untracked_cnt" -gt 0 ] && parts="${parts}${untracked_cnt} untracked, "
        parts=$(echo "$parts" | sed 's/, $//')
        tree_status="dirty ($parts)"
    fi

    # Current version info
    current_ver="none"
    if [ -f "$git_root/VERSION" ]; then
        current_ver=$(cat "$git_root/VERSION" | tr -d '[:space:]')
    else
        last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
        [ -n "$last_tag" ] && current_ver="$last_tag"
    fi

    # Next predictive version
    next_ver="unknown"
    if command -v calculate_target_version >/dev/null 2>&1; then
        next_ver=$(calculate_target_version 2>/dev/null || echo "0.0.1")
    fi

    # Commits ahead of latest tag
    latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$latest_tag" ]; then
        ahead_tag_count=$(git rev-list --count "${latest_tag}..HEAD" 2>/dev/null || echo "0")
    else
        ahead_tag_count=$(git rev-list --count HEAD 2>/dev/null || echo "0")
    fi

    # Hooks health
    hooks_dir="$git_root/.git/hooks"
    hook_status="missing"
    if [ -f "$hooks_dir/commit-msg" ] && [ -f "$hooks_dir/post-commit" ]; then
        if grep -q "cccp-hook-version:" "$hooks_dir/commit-msg" 2>/dev/null; then
            hook_ver=$(sed -n 's/^# cccp-hook-version:[[:space:]]*//p' "$hooks_dir/commit-msg" 2>/dev/null | head -n 1)
            [ -n "$hook_ver" ] && hook_status="installed (v${hook_ver})" || hook_status="installed"
        elif grep -q "CCCP_HOOK_VERSION" "$hooks_dir/commit-msg" 2>/dev/null; then
            hook_ver=$(grep "CCCP_HOOK_VERSION=" "$hooks_dir/commit-msg" 2>/dev/null | cut -d'"' -f2 || echo "")
            [ -n "$hook_ver" ] && hook_status="installed (v${hook_ver})" || hook_status="installed"
        else
            hook_status="custom/non-cccp (run 'cccp hooks' for audit)"
        fi
    elif [ -f "$hooks_dir/commit-msg" ] || [ -f "$hooks_dir/post-commit" ]; then
        hook_status="partially installed (run 'cccp hooks' for audit)"
    fi

    # Post-Push Hook status
    post_push_status="none (optional)"
    if command -v get_post_push_hook_info >/dev/null 2>&1; then
        pp_info=$(get_post_push_hook_info 2>/dev/null || echo "status=none")
        if [ "$(echo "$pp_info" | grep "^status=" | cut -d= -f2-)" = "active" ]; then
            pp_path=$(echo "$pp_info" | grep "^path=" | cut -d= -f2-)
            pp_sync=$(echo "$pp_info" | grep "^sync_actions=" | cut -d= -f2-)
            if [ -n "$pp_sync" ] && [ "$pp_path" = "(built-in sync)" ]; then
                post_push_status="$pp_sync (active)"
            elif [ -n "$pp_sync" ]; then
                post_push_status="$pp_path + $pp_sync (active)"
            else
                post_push_status="$pp_path (active)"
            fi
        fi
    elif [ -n "${POST_PUSH_HOOK:-}" ]; then
        post_push_status="$POST_PUSH_HOOK (active)"
    elif [ -n "${git_root:-}" ] && [ -x "$git_root/.cccp/hooks/post-push" ]; then
        post_push_status="$git_root/.cccp/hooks/post-push (active)"
    elif [ -n "${hooks_dir:-}" ] && [ -x "$hooks_dir/post-push.local" ]; then
        post_push_status="$hooks_dir/post-push.local (active)"
    elif [ -n "${hooks_dir:-}" ] && [ -x "$hooks_dir/post-push" ]; then
        post_push_status="$hooks_dir/post-push (active)"
    fi

    # Configuration files
    local_cfg=$(get_local_config_file 2>/dev/null || echo "")
    global_cfg=$(get_global_config_file 2>/dev/null || echo "")

    local_status="not present"
    [ -n "$local_cfg" ] && [ -f "$local_cfg" ] && local_status="$local_cfg"

    global_status="not present"
    [ -n "$global_cfg" ] && [ -f "$global_cfg" ] && global_status="$global_cfg"

    # Policies
    types_policy=$([ "${STRICT_TYPES:-1}" = "1" ] && echo "strict (curated)" || echo "permissive (any alphanumeric)")
    scopes_policy=$([ "${STRICT_SCOPES:-0}" = "1" ] && echo "strict (curated)" || echo "permissive (any scope)")

    # System binary location
    binary_loc=$(command -v cccp 2>/dev/null || echo "not in PATH")

    # Output State Inspection
    echo "========================================================"
    echo " ★ CCCP State Inspection (Gosplan Quality Control) ★"
    echo "========================================================"
    echo " Repository:       $git_root"
    echo " Active Branch:    $branch ($tracking_str)"
    echo " Working Tree:     $tree_status"
    echo " Current Version:  $current_ver"
    echo " Target Version:   $next_ver ($ahead_tag_count commits ahead of tag)"
    echo " Git Hooks:        $hook_status"
    echo " Post-Push Hook:   $post_push_status"
    echo " System Binary:    $binary_loc"
    echo " Local Config:     $local_status"
    echo " Global Config:    $global_status"
    echo " Types Policy:     $types_policy"
    echo " Scopes Policy:    $scopes_policy"
    echo " Update Channel:   ${UPDATE_CHANNEL:-stable} (every ${UPDATE_INTERVAL_DAYS:-30} days)"
    [ -n "${PINNED_VERSION:-}" ] && echo " Pinned Version:   $PINNED_VERSION (Gosplan Directive Active)"
    echo "========================================================"
    return 0
}


# =============================================================================
# Lint Functions
# =============================================================================
lint_commits() {
    # Verify git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    range="${1:-}"

    # Determine default range if none provided
    if [ -z "$range" ]; then
        if [ -n "${GITHUB_BASE_REF:-}" ]; then
            range="origin/${GITHUB_BASE_REF}..HEAD"
        elif git rev-parse --verify '@{u}' >/dev/null 2>&1; then
            range="@{u}..HEAD"
        elif git rev-parse --verify 'origin/main' >/dev/null 2>&1; then
            range="origin/main..HEAD"
        elif git rev-parse --verify 'HEAD~1' >/dev/null 2>&1; then
            range="HEAD~1..HEAD"
        else
            range="HEAD"
        fi
    fi

    # Verify that the range is valid in git
    if echo "$range" | grep -q '\.\.'; then
        ref1="${range%%..*}"
        ref2="${range##*..}"
        if [ -n "$ref1" ] && ! git rev-parse --verify "$ref1^{commit}" >/dev/null 2>&1; then
            echo "Error: Invalid commit range or revision '$range'" >&2
            return 1
        fi
        if [ -n "$ref2" ] && ! git rev-parse --verify "$ref2^{commit}" >/dev/null 2>&1; then
            echo "Error: Invalid commit range or revision '$range'" >&2
            return 1
        fi
    elif ! git rev-parse --verify "$range^{commit}" >/dev/null 2>&1; then
        echo "Error: Invalid commit range or revision '$range'" >&2
        return 1
    fi

    commit_hashes=$(git rev-list --reverse "$range" 2>/dev/null || true)
    if [ -z "$commit_hashes" ]; then
        echo "========================================================"
        echo " ★ CCCP Party Line Compliance (Commit Linting) ★"
        echo "========================================================"
        echo " Inspected Range:  $range"
        echo " Result:           0 commits found in range to inspect."
        echo " Status:           All clear for the State Plan."
        echo "========================================================"
        return 0
    fi

    echo "========================================================"
    echo " ★ CCCP Party Line Compliance (Commit Linting) ★"
    echo "========================================================"
    echo " Inspected Range:  $range"
    echo "========================================================"

    total_count=0
    passed_count=0
    failed_count=0

    for hash in $commit_hashes; do
        total_count=$((total_count + 1))
        short_hash=$(echo "$hash" | cut -c1-7)
        subject=$(git log -1 --format="%s" "$hash")

        # Capture validation output
        val_output=$(validate_commit_message "$subject" 2>&1 || true)
        val_status=$?

        if echo "$val_output" | grep -q "^Error:"; then
            val_status=1
        fi

        if [ $val_status -eq 0 ]; then
            passed_count=$((passed_count + 1))
            echo "  ✓ [$short_hash] $subject"
        else
            failed_count=$((failed_count + 1))
            echo "  ✗ [$short_hash] $subject"
            err_msg=$(echo "$val_output" | grep "^Error:" | head -n 1 || echo "Error: Invalid conventional commit message")
            echo "    ↳ $err_msg"
        fi
    done

    echo "========================================================"
    echo " Inspection Summary: $total_count inspected, $passed_count compliant, $failed_count ideological deviations."

    if [ "$failed_count" -gt 0 ]; then
        echo " Directive: Order No. 227 - Not One Step Back from Clean Commits!"
        echo " FAILED: Non-compliant commits detected. Correct history before merge."
        echo "========================================================"
        return 1
    else
        echo " PASSED: Full ideological compliance verified. Glory to the collective!"
        echo "========================================================"
        return 0
    fi
}


# =============================================================================
# Interactive Commit Wizard
# =============================================================================
interactive_commit() {
    # Verify git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    # Check if there are staged changes to commit
    staged_count=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
    if [ "$staged_count" -eq 0 ]; then
        echo "Warning: No changes staged for commit." >&2
        echo "Stage your changes first with 'git add <files>' before assembling a commit." >&2
        return 1
    fi

    echo "========================================================"
    echo " ★ CCCP Assembly Line (Interactive Soviet Wizard) ★"
    echo "========================================================"
    echo " Step 1: Select the type of change"
    echo "--------------------------------------------------------"

    # List available types
    types_list="${COMMIT_TYPES:-feat fix perf refactor revert chore build ci docs ops style test merge}"
    type_idx=1
    for t in $types_list; do
        desc=""
        if command -v get_type_description >/dev/null 2>&1; then
            desc=$(get_type_description "$t")
        fi
        printf "  %2d) %-10s - %s\n" "$type_idx" "$t" "$desc"
        type_idx=$((type_idx + 1))
    done

    # Read type selection
    selected_type=""
    while [ -z "$selected_type" ]; do
        printf "\nSelect type [1-%d or name]: " "$((type_idx - 1))"
        read -r choice || return 1
        choice=$(echo "$choice" | tr -d '[:space:]')
        if echo "$choice" | grep -qE '^[0-9]+$'; then
            curr=1
            for t in $types_list; do
                if [ "$curr" -eq "$choice" ]; then
                    selected_type="$t"
                    break
                fi
                curr=$((curr + 1))
            done
        else
            for t in $types_list; do
                if [ "$choice" = "$t" ]; then
                    selected_type="$t"
                    break
                fi
            done
        fi
        if [ -z "$selected_type" ]; then
            echo "Invalid selection. Please choose a number between 1 and $((type_idx - 1)) or enter a type name."
        fi
    done

    # Step 2: Select or enter scope
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 2: Scope of the change (optional)"
    echo "--------------------------------------------------------"
    echo " Common scopes: core, cli, config, ui, api, auth, db, docs, test, build"
    printf "Enter scope (press enter to skip): "
    read -r selected_scope || return 1
    selected_scope=$(echo "$selected_scope" | tr -d '[:space:]')

    # Step 3: Short subject line
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 3: Commit subject (short imperative summary)"
    echo "--------------------------------------------------------"
    selected_subject=""
    while [ -z "$selected_subject" ]; do
        printf "Enter subject: "
        read -r selected_subject || return 1
        selected_subject=$(echo "$selected_subject" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
        if [ -z "$selected_subject" ]; then
            echo "Commit subject cannot be empty."
        fi
    done

    # Step 4: Breaking change
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 4: Breaking changes"
    echo "--------------------------------------------------------"
    printf "Does this change introduce a BREAKING CHANGE? (y/N): "
    read -r is_breaking || return 1
    is_breaking=$(echo "$is_breaking" | tr '[:upper:]' '[:lower:]')

    # Step 5: Longer description body
    echo ""
    echo "--------------------------------------------------------"
    echo " Step 5: Extended commit body (optional)"
    echo "--------------------------------------------------------"
    printf "Enter longer description (press enter to skip): "
    read -r selected_body || return 1
    selected_body=$(echo "$selected_body" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')

    # Assemble header
    header_prefix="$selected_type"
    if [ -n "$selected_scope" ]; then
        header_prefix="${header_prefix}(${selected_scope})"
    fi
    if [ "$is_breaking" = "y" ] || [ "$is_breaking" = "yes" ]; then
        header_prefix="${header_prefix}!"
    fi
    commit_header="${header_prefix}: ${selected_subject}"

    # Assemble full message
    full_message="$commit_header"
    if [ -n "$selected_body" ]; then
        full_message="${full_message}

${selected_body}"
    fi

    # Step 6: Confirmation
    echo ""
    echo "========================================================"
    echo " Proposed State Commit:"
    echo "--------------------------------------------------------"
    echo "$full_message"
    echo "--------------------------------------------------------"
    printf "Proceed with commit to the State archives? [Y/n]: "
    read -r confirm || return 1
    confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')

    if [ "$confirm" = "n" ] || [ "$confirm" = "no" ]; then
        echo "Commit aborted by worker directive."
        return 1
    fi

    # Perform commit
    if ! validate_commit_message "$full_message"; then
        echo "Error: Validation failed on constructed message." >&2
        return 1
    fi

    git commit -m "$full_message"
    echo "========================================================"
    echo " ★ Commit recorded into the State archives successfully! ★"
    echo "========================================================"
    return 0
}


# =============================================================================
# Completion Generator
# =============================================================================
generate_bash_completion() {
    cat << 'EOF'
# CCCP Bash Completion
_cccp_completions() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="status cz commit lint config version tag push tickets sync-tickets agent update-agent sync-agent agents update-agents sync-agents codeberg set-av changelog update install completion hooks soviet help"
    local config_keys="types scopes subscopes strict_types strict_scopes strict_subscopes disable_subscopes disable_multiple_scopes default_base_version no_v update_channel update_interval_days check_updates post_push_hook post_push_sync post_push_sync_tickets post_push_sync_agent post_push_sync_agents tickets_file agent_md_url agents_md_url agent_md_file agents_md_file"

    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return 0
    fi

    local cmd="${COMP_WORDS[1]}"
    case "$cmd" in
        agent|update-agent|sync-agent|agents|update-agents|sync-agents)
            case "$prev" in
                -u|--url|-f|--file)
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-u --url -f --file -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "update check fill set get unset" -- "$cur"))
            fi
            ;;
        codeberg)
            COMPREPLY=($(compgen -W "setup" -- "$cur"))
            if [ "$prev" = "setup" ] || [ "$cur" = "-"* ]; then
                COMPREPLY=($(compgen -W "-u --url -h --help" -- "$cur"))
            fi
            ;;
        set-av)
            return 0
            ;;
        tickets|sync-tickets)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-f --file -o --owner -r --repo -m --max-bytes -t --token -h --help" -- "$cur"))
            fi
            ;;
        config)
            case "$prev" in
                --unset)
                    COMPREPLY=($(compgen -W "$config_keys" -- "$cur"))
                    return 0
                    ;;
                update_channel)
                    COMPREPLY=($(compgen -W "stable nightly" -- "$cur"))
                    return 0
                    ;;
                strict_types|strict_scopes|strict_subscopes|disable_subscopes|disable_multiple_scopes|no_v|check_updates)
                    COMPREPLY=($(compgen -W "0 1" -- "$cur"))
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--global -g --local --list -l --unset -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$config_keys" -- "$cur"))
            fi
            ;;
        commit)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-i --interactive --strict-scopes --allow-any-scope --strict-subscopes --allow-any-subscope --disable-subscopes --enable-subscopes --disable-multiple-scopes --enable-multiple-scopes -h --help" -- "$cur"))
            fi
            ;;
        tag)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-t --title -m --no-v -h --help" -- "$cur"))
            fi
            ;;
        update)
            case "$prev" in
                --channel)
                    COMPREPLY=($(compgen -W "stable nightly" -- "$cur"))
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--channel -h --help" -- "$cur"))
            fi
            ;;
        install)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-g --global --profile -h --help" -- "$cur"))
            fi
            ;;
        completion)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
            ;;
        help)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
    esac
}
complete -F _cccp_completions cccp cccp.sh
EOF
}

generate_zsh_completion() {
    cat << 'EOF'
#compdef cccp cccp.sh

_cccp() {
    local -a commands
    commands=(
        'status:Inspect repository health and Gosplan diagnostics'
        'cz:Interactive terminal wizard for creating commits'
        'commit:Validate formatting and create a commit'
        'lint:Lint commit messages across a git revision range'
        'config:Manage hierarchical configuration'
        'version:Generate predictive SemVer metadata'
        'tag:Create annotated tag, commit VERSION and CHANGELOG'
        'push:Execute git push and trigger optional post-push hook'
        'tickets:Sync Codeberg issues into markdown'
        'sync-tickets:Sync Codeberg issues into markdown'
        'agent:Generate and synchronize AGENTS.md directives'
        'update-agent:Generate and synchronize AGENTS.md directives'
        'sync-agent:Generate and synchronize AGENTS.md directives'
        'agents:Generate and synchronize AGENTS.md directives'
        'update-agents:Generate and synchronize AGENTS.md directives'
        'sync-agents:Generate and synchronize AGENTS.md directives'
        'set-av:Fast shortcut to configure local agent variable'
        'codeberg:Configure Codeberg integration and AGENTS template wizard'
        'changelog:Generate or update CHANGELOG.md'
        'update:Update cccp executable from GitHub releases'
        'install:Install to PATH or configure Git hooks'
        'completion:Generate shell autocompletion script'
        'hooks:Inspect git hooks provenance and diff wrappers'
        'soviet:Display Soviet State anthem and art'
        'help:Display documentation for any command'
    )

    local -a config_keys
    config_keys=(
        'types:Allowed commit types list'
        'scopes:Allowed commit scopes list'
        'subscopes:Allowed commit subscopes list'
        'strict_types:Enforce type matching known types list'
        'strict_scopes:Require scope to match known list'
        'strict_subscopes:Require subscope to match known list'
        'disable_subscopes:Disallow slash-delimited subscopes'
        'disable_multiple_scopes:Disallow comma-separated scopes'
        'default_base_version:Fallback SemVer when no tags exist'
        'no_v:Tag without v prefix'
        'update_channel:Update channel (stable or nightly)'
        'update_interval_days:Days between update checks'
        'check_updates:Enable automated update notifications'
        'post_push_hook:Custom script or command executed following git push'
        'post_push_sync:Combined list of post-push sync actions (tickets, agent)'
        'post_push_sync_tickets:Enable automatic tickets sync on post-push'
        'post_push_sync_agent:Enable automatic AGENTS.md sync on post-push'
        'post_push_sync_agents:Enable automatic AGENTS.md sync on post-push'
        'tickets_file:Destination file for Codeberg issues sync'
        'agent_md_url:URL or path to AGENTS.md skeleton template'
        'agents_md_url:URL or path to AGENTS.md skeleton template'
        'agent_md_file:Destination file for AGENTS.md directives'
        'agents_md_file:Destination file for AGENTS.md directives'
    )

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'cccp command' commands
            ;;
        args)
            case $line[1] in
                config)
                    _arguments \
                        '(-g --global)'{-g,--global}'[Target global config file]' \
                        '--local[Target local repository config file]' \
                        '(-l --list)'{-l,--list}'[List configuration key/value pairs]' \
                        '--unset[Remove a configuration key]:key:->config_keys' \
                        '1:key:->config_keys' \
                        '2:value:'
                    if [[ $state == config_keys ]]; then
                        _describe -t config_keys 'configuration keys' config_keys
                    fi
                    ;;
                commit)
                    _arguments \
                        '(-i --interactive)'{-i,--interactive}'[Launch interactive terminal wizard]' \
                        '--strict-scopes[Enforce known scopes]' \
                        '--allow-any-scope[Allow arbitrary scopes]' \
                        '--strict-subscopes[Enforce known subscopes]' \
                        '--allow-any-subscope[Allow arbitrary subscopes]' \
                        '--disable-subscopes[Disallow subscopes]' \
                        '--enable-subscopes[Enable subscopes]' \
                        '--disable-multiple-scopes[Disallow multiple scopes]' \
                        '--enable-multiple-scopes[Enable multiple scopes]' \
                        '1:commit message:'
                    ;;
                tag)
                    _arguments \
                        '(-t --title)'{-t,--title}'[Release title/description]:title:' \
                        '-m[Custom tag message]:message:' \
                        '--no-v[Tag without v prefix]' \
                        '1:version:'
                    ;;
                tickets|sync-tickets)
                    _arguments \
                        '(-f --file)'{-f,--file}'[Output markdown file]:file:_files' \
                        '(-o --owner)'{-o,--owner}'[Codeberg repository owner]:owner:' \
                        '(-r --repo)'{-r,--repo}'[Codeberg repository name]:repo:' \
                        '(-m --max-bytes)'{-m,--max-bytes}'[Maximum budget in bytes]:bytes:' \
                        '(-t --token)'{-t,--token}'[Codeberg personal access token]:token:'
                    ;;
                agent|update-agent|sync-agent|agents|update-agents|sync-agents)
                    _arguments \
                        '(-u --url)'{-u,--url}'[Template URL or local file path]:url:_files' \
                        '(-f --file)'{-f,--file}'[Output markdown file]:file:_files' \
                        '1:subcommand:(update check fill set get unset)'
                    ;;
                codeberg)
                    _arguments \
                        '(-u --url)'{-u,--url}'[AGENTS template URL]:url:' \
                        '1:subcommand:(setup)'
                    ;;
                set-av)
                    ;;
                update)
                    _arguments \
                        '--channel[Release channel]:channel:(stable nightly)'
                    ;;
                install)
                    _arguments \
                        '(-g --global)'{-g,--global}'[Install globally to PATH]' \
                        '--profile[Update shell profile]'
                    ;;
                completion)
                    _arguments \
                        '1:shell:(bash zsh fish)'
                    ;;
                help)
                    _describe -t commands 'cccp command' commands
                    ;;
            esac
            ;;
    esac
}

_cccp "$@"
EOF
}

generate_fish_completion() {
    cat << 'EOF'
# CCCP Fish Completion

# Disable file completion by default
complete -c cccp -f
complete -c cccp.sh -f

# Main commands
complete -c cccp -n "__fish_use_subcommand" -a status -d "Inspect repository health and Gosplan diagnostics"
complete -c cccp -n "__fish_use_subcommand" -a cz -d "Interactive terminal wizard for creating commits"
complete -c cccp -n "__fish_use_subcommand" -a commit -d "Validate formatting and create a commit"
complete -c cccp -n "__fish_use_subcommand" -a lint -d "Lint commit messages across a git revision range"
complete -c cccp -n "__fish_use_subcommand" -a config -d "Manage hierarchical configuration"
complete -c cccp -n "__fish_use_subcommand" -a version -d "Generate predictive SemVer metadata"
complete -c cccp -n "__fish_use_subcommand" -a tag -d "Create annotated tag, commit VERSION and CHANGELOG"
complete -c cccp -n "__fish_use_subcommand" -a push -d "Execute git push and trigger optional post-push hook"
complete -c cccp -n "__fish_use_subcommand" -a tickets -d "Sync Codeberg issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a sync-tickets -d "Sync Codeberg issues into markdown"
complete -c cccp -n "__fish_use_subcommand" -a agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a update-agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a sync-agent -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a update-agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a sync-agents -d "Generate and synchronize AGENTS.md directives"
complete -c cccp -n "__fish_use_subcommand" -a set-av -d "Fast shortcut to configure local agent variable"
complete -c cccp -n "__fish_use_subcommand" -a codeberg -d "Configure Codeberg integration and AGENTS template wizard"
complete -c cccp -n "__fish_use_subcommand" -a changelog -d "Generate or update CHANGELOG.md"
complete -c cccp -n "__fish_use_subcommand" -a update -d "Update cccp executable from GitHub releases"
complete -c cccp -n "__fish_use_subcommand" -a install -d "Install to PATH or configure Git hooks"
complete -c cccp -n "__fish_use_subcommand" -a completion -d "Generate shell autocompletion script"
complete -c cccp -n "__fish_use_subcommand" -a soviet -d "Display Soviet State anthem and art"
complete -c cccp -n "__fish_use_subcommand" -a help -d "Display documentation for any command"

# Options for commit
complete -c cccp -n "__fish_seen_subcommand_from commit" -s i -l interactive -d "Launch interactive terminal wizard"

# Options for tag
complete -c cccp -n "__fish_seen_subcommand_from tag" -s t -l title -d "Release title"
complete -c cccp -n "__fish_seen_subcommand_from tag" -l no-v -d "Tag without v prefix"

# Options for tickets
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s f -l file -d "Output markdown file"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s o -l owner -d "Codeberg repository owner"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s r -l repo -d "Codeberg repository name"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s m -l max-bytes -d "Maximum budget in bytes"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s t -l token -d "Codeberg personal access token"

# Options for agent
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -s u -l url -d "Template URL or local file path"
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -s f -l file -d "Output markdown file"
complete -c cccp -n "__fish_seen_subcommand_from agent update-agent sync-agent agents update-agents sync-agents" -a "update check fill set get unset"

# Options for codeberg
complete -c cccp -n "__fish_seen_subcommand_from codeberg" -a "setup" -d "Interactive Codeberg setup wizard"
complete -c cccp -n "__fish_seen_subcommand_from codeberg" -s u -l url -d "AGENTS template URL"

# Options for config
complete -c cccp -n "__fish_seen_subcommand_from config" -s g -l global -d "Target global config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -l local -d "Target local repository config file"
complete -c cccp -n "__fish_seen_subcommand_from config" -s l -l list -d "List configuration key/value pairs"
complete -c cccp -n "__fish_seen_subcommand_from config" -l unset -d "Remove a configuration key"

# Options for completion
complete -c cccp -n "__fish_seen_subcommand_from completion" -a "bash zsh fish"
EOF
}

cmd_completion() {
    target_shell="${1:-}"

    if [ -z "$target_shell" ]; then
        if [ -n "${SHELL:-}" ]; then
            target_shell=$(basename "$SHELL")
        else
            target_shell="bash"
        fi
    fi

    case "$target_shell" in
        bash)
            generate_bash_completion
            ;;
        zsh)
            generate_zsh_completion
            ;;
        fish)
            generate_fish_completion
            ;;
        *)
            echo "Error: Unsupported shell '$target_shell'. Supported shells: bash, zsh, fish" >&2
            return 1
            ;;
    esac
    return 0
}


# =============================================================================
# Soviet Easter Egg
# =============================================================================
show_soviet() {
    cat << 'EOF'
                 ★
              /-----\
             |  CCCP |   CONVENTIONAL COMMITS COMPLIANCE PROGRAM
              \-----/    "Workers of the World, Conventionalize! ☭"
             /   |   \
            /    |    \

★ COMRADE! The State Planning Committee (Gosplan) salutes your discipline!
★ ORDER NO. 227: Not one step back from Conventional Commits!
★ Five-Year Plan for 100% compliant Git history is fulfilling its quota!

Directives:
  • feat:     Industrial progress for the Motherland (Minor bump)
  • fix:      Repair sabotage in the machinery (Patch bump)
  • BREAKING: Revolutionary paradigm shift (Major bump)

Glory to the Standardized Commit History!
EOF
}


# =============================================================================
# Main script entry point
# =============================================================================
main() {
    # Recursion depth guard to prevent fork bombs
    depth="${CCCP_RECURSION_DEPTH:-0}"
    if [ "$depth" -ge 3 ]; then
        echo "Error: Maximum recursion depth exceeded in cccp." >&2
        return 1
    fi
    export CCCP_RECURSION_DEPTH=$((depth + 1))

    # Load configuration hierarchy (Defaults < Global < Local < Environment)
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    if [ -n "${1:-}" ]; then
       command="$1"
    else
       command=""
    fi
    current_hook=$(basename "$0")
    
    # Check if running as a hook
    case " ${GIT_HOOKS_LIST:-commit-msg post-commit pre-push reference-transaction} " in
        *" $current_hook "*)
            command="$current_hook"
            ;;
    esac

    # Hook version audit in git repositories
    if [ -n "${GIT_ROOT:-}" ] && command -v check_hook_version >/dev/null 2>&1; then
        case "$command" in
            "commit"|"commit-msg"|"post-commit"|"pre-push"|"reference-transaction")
                check_hook_version || true
                ;;
        esac
    fi

    # Verify required system tools for operational commands
    if command -v check_system_tools >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"commit-msg"|"post-commit"|"pre-push"|"reference-transaction"|"hooks")
                check_system_tools || exit 1
                ;;
        esac
    fi

    # Non-blocking periodic update check on interactive user commands
    if command -v check_auto_update >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"completion"|"hooks"|"tickets"|"sync-tickets"|"agents"|"update-agents"|"set-av")
                check_auto_update || true
                ;;
        esac
    fi
    
    case "$command" in
        "git")
            echo "Git command handler"
            exit 0
            ;;
        "config")
            shift || true
            cmd_config "$@"
            exit 0
            ;;
        "status")
            shift || true
            show_status "$@"
            exit 0
            ;;
        "lint")
            shift || true
            lint_commits "$@"
            exit 0
            ;;
        "cz")
            shift || true
            interactive_commit "$@"
            exit 0
            ;;
        "commit")
            shift || true
            commit "$@"
            exit 0
            ;;
        "install")
            shift || true
            install_cccp "$@"
            exit 0
            ;;
        "completion")
            shift || true
            cmd_completion "$@"
            exit 0
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
        "push")
            shift || true
            cmd_push "$@"
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
            if [ "${1:-}" = "commit-msg" ]; then
                shift || true
            fi
            commit_msg "$@"
            exit 0
            ;;
        "post-commit")
            post_commit
            exit 0
            ;;
        "pre-push")
            if [ "${1:-}" = "pre-push" ]; then
                shift || true
            fi
            cmd_pre_push "$@"
            exit 0
            ;;
        "reference-transaction")
            if [ "${1:-}" = "reference-transaction" ]; then
                shift || true
            fi
            cmd_reference_transaction "$@"
            exit 0
            ;;
        "update")
            shift || true
            update_script "$@"
            exit 0
            ;;
        "check-update"|"check-updates")
            shift || true
            cmd_check_update "$@"
            exit 0
            ;;
        "hooks")
            shift || true
            cmd_hooks "$@"
            exit 0
            ;;
        "tickets"|"sync-tickets")
            shift || true
            cmd_tickets "$@"
            exit 0
            ;;
        "codeberg")
            shift || true
            cmd_codeberg "$@"
            exit 0
            ;;
        "agent"|"agents"|"update-agent"|"update-agents"|"sync-agent"|"sync-agents")
            shift || true
            cmd_agent "$@"
            exit 0
            ;;
        "set-av"|"set-agent-var")
            shift || true
            cmd_set_av "$@"
            exit 0
            ;;
        "soviet"|"sputnik"|"anthem"|"gosplan")
            show_soviet
            exit 0
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|cz|install|config|status|lint|completion|version|tag|push|tickets|agent|set-av|changelog|commit-msg|post-commit|update|check-update|hooks|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
            exit 1
            ;;
    esac
}

if [ "${CCCP_SOURCED:-0}" != "1" ]; then
    main "$@"
fi
