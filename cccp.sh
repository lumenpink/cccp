#!/bin/sh

# =============================================================================
# Conventional Commits Compliance Program
# This script provides tools for managing git commits following conventional commit
# standards, version management, and changelog generation.
# =============================================================================

# Enable error handling
set -eu

# Script Version (Single Source of Truth, synchronized from VERSION)
CCCP_VERSION="2.4.0-dev.35+20260927.af638bc"
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
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Error: Not a git repository" >&2
        return 1
    fi
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

    # If combine.sh is present, recompile cccp.sh to embed the release version
    if [ -f "$GIT_ROOT/combine.sh" ]; then
        ( cd "$GIT_ROOT" && ./combine.sh >/dev/null 2>&1 || true )
    fi

    # Commit the release files (VERSION and CHANGELOG.md)
    if [ -n "$title" ]; then
        commit_msg="chore(release): $tag_name - $title"
    else
        commit_msg="chore(release): $tag_name"
    fi

    export HOOK_ACTIVE=1
    git add "$GIT_ROOT/VERSION" "$GIT_ROOT/CHANGELOG.md"
    [ -f "$GIT_ROOT/cccp.sh" ] && git add "$GIT_ROOT/cccp.sh"
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
# Repository & Forge Resolver Functions
# =============================================================================
parse_remote_url() {
    _raw_url="$1"
    [ -z "$_raw_url" ] && return 1

    _clean=$(printf '%s\n' "$_raw_url" | sed -e 's/\.git$//' -e 's|/*$||')

    _host=""
    _path=""
    case "$_clean" in
        *://*)
            _no_proto=$(printf '%s\n' "$_clean" | sed -e 's|^[a-zA-Z][a-zA-Z0-9+-.]*://||')
            _host_part=$(printf '%s\n' "$_no_proto" | cut -d'/' -f1)
            _host=$(printf '%s\n' "$_host_part" | sed -e 's|^.*@||' -e 's|:[0-9]*$||')
            _path=$(printf '%s\n' "$_no_proto" | sed -e 's|^[^/]*/||' -e 's|^/*||')
            ;;
        *:*)
            _host_part=$(printf '%s\n' "$_clean" | sed -e 's|:.*$||')
            _host=$(printf '%s\n' "$_host_part" | sed -e 's|^.*@||')
            _path=$(printf '%s\n' "$_clean" | sed -e 's|^[^:]*:||' -e 's|^/*||')
            ;;
        *)
            return 1
            ;;
    esac

    [ -z "$_path" ] && return 1

    _owner=$(printf '%s\n' "$_path" | sed -e 's|/[^/]*$||')
    _repo=$(printf '%s\n' "$_path" | sed -e 's|^.*/||')

    [ -n "$_host" ] && [ -n "$_owner" ] && [ -n "$_repo" ] || return 1
    printf '%s %s %s\n' "$_host" "$_owner" "$_repo"
    return 0
}

_repo_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

detect_forgeapp() {
    _fhost="$1"
    [ -z "$_fhost" ] && return 1

    # 1. Check local config override if set
    _cfg_provider="${FORGEAPP:-${TICKETS_PROVIDER:-}}"
    if [ -z "$_cfg_provider" ] && command -v read_file_key >/dev/null 2>&1; then
        _git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
        if [ -n "$_git_root" ] && [ -f "$_git_root/.cccprc" ]; then
            _cfg_provider="$(read_file_key "$_git_root/.cccprc" "forgeapp" 2>/dev/null || true)"
            [ -z "$_cfg_provider" ] && _cfg_provider="$(read_file_key "$_git_root/.cccprc" "tickets_provider" 2>/dev/null || true)"
        fi
    fi
    if [ -n "$_cfg_provider" ]; then
        case "$_cfg_provider" in
            forgeapp_forgejo|forgejo|codeberg|tildegit|gitea) printf 'forgeapp_forgejo\n'; return 0 ;;
            forgeapp_github|github) printf 'forgeapp_github\n'; return 0 ;;
            forgeapp_gitlab|gitlab) printf 'forgeapp_gitlab\n'; return 0 ;;
            forgeapp_sourcehut|sourcehut) printf 'forgeapp_sourcehut\n'; return 0 ;;
            forgeapp_*) printf '%s\n' "$_cfg_provider"; return 0 ;;
        esac
    fi

    # 2. Fast-path: Deterministic domain lookup
    case "$_fhost" in
        *github.com*|forgeapp_github|github)
            printf 'forgeapp_github\n'
            return 0
            ;;
        *codeberg.org*|*gitea.com*|*tildegit.org*|*disroot.org*|forgeapp_forgejo|forgejo|codeberg|tildegit)
            printf 'forgeapp_forgejo\n'
            return 0
            ;;
        *gitlab.com*|*framagit.org*|forgeapp_gitlab|gitlab)
            printf 'forgeapp_gitlab\n'
            return 0
            ;;
        *.sr.ht*|git.sr.ht|todo.sr.ht|forgeapp_sourcehut|sourcehut)
            printf 'forgeapp_sourcehut\n'
            return 0
            ;;
    esac

    # 3. Cache lookup (in .git/cccp_forgeapp_cache)
    _git_dir="$(git rev-parse --git-dir 2>/dev/null || true)"
    _cache_file=""
    if [ -n "$_git_dir" ] && [ -d "$_git_dir" ]; then
        _cache_file="$_git_dir/cccp_forgeapp_cache"
        if [ -f "$_cache_file" ]; then
            _cached=$(grep -E "^${_fhost}:" "$_cache_file" 2>/dev/null | head -n 1 | cut -d':' -f2 || true)
            if [ -n "$_cached" ]; then
                printf '%s\n' "$_cached"
                return 0
            fi
        fi
    fi

    # 4. Active Probing (Strict detection without false assumptions)
    if command -v _cccp_http >/dev/null 2>&1 || command -v sv >/dev/null 2>&1 || command -v svyaz >/dev/null 2>&1 || command -v curl >/dev/null 2>&1; then
        _probe_gitea=$(_repo_http -s -f -m 3 "https://${_fhost}/api/v1/version" 2>/dev/null || true)
        if [ -n "$_probe_gitea" ]; then
            if command -v jq >/dev/null 2>&1; then
                if printf '%s\n' "$_probe_gitea" | jq -e '.version' >/dev/null 2>&1; then
                    [ -n "$_cache_file" ] && printf '%s:forgeapp_forgejo\n' "$_fhost" >> "$_cache_file"
                    printf 'forgeapp_forgejo\n'
                    return 0
                fi
            else
                case "$_probe_gitea" in
                    *\"version\"*)
                        [ -n "$_cache_file" ] && printf '%s:forgeapp_forgejo\n' "$_fhost" >> "$_cache_file"
                        printf 'forgeapp_forgejo\n'
                        return 0
                        ;;
                esac
            fi
        fi

        _probe_gl=$(_repo_http -s -f -I -m 3 "https://${_fhost}/api/v4/version" 2>/dev/null || _repo_http -s -f -I -m 3 "https://${_fhost}/" 2>/dev/null || true)
        case "$_probe_gl" in
            *gitlab*|*GitLab*)
                [ -n "$_cache_file" ] && printf '%s:forgeapp_gitlab\n' "$_fhost" >> "$_cache_file"
                printf 'forgeapp_gitlab\n'
                return 0
                ;;
        esac
    fi

    printf 'unknown\n'
    return 1
}

detect_forge_provider() {
    _fhost="$1"
    [ -z "$_fhost" ] && return 1

    case "$_fhost" in
        *codeberg.org*|codeberg) printf 'forgejo\n'; return 0 ;;
        *tildegit.org*|tildegit) printf 'tildegit\n'; return 0 ;;
    esac

    _app=$(detect_forgeapp "$_fhost" 2>/dev/null || true)
    case "$_app" in
        forgeapp_forgejo) printf 'forgejo\n'; return 0 ;;
        forgeapp_github) printf 'github\n'; return 0 ;;
        forgeapp_gitlab) printf 'gitlab\n'; return 0 ;;
        forgeapp_sourcehut) printf 'sourcehut\n'; return 0 ;;
        *) return 1 ;;
    esac
}

parse_codeberg_url() { _parse_forge_remote "$1" "codeberg"; }
parse_github_url()   { _parse_forge_remote "$1" "github"; }
parse_gitlab_url()   { _parse_forge_remote "$1" "gitlab"; }

_parse_forge_remote() {
    _raw="$1"
    _target_type="$2"
    if _parsed=$(parse_remote_url "$_raw" 2>/dev/null); then
        _host=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
        _owner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
        _repo=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)
        _match=0
        case "$_target_type" in
            codeberg)
                case "$_host" in *codeberg.org*) _match=1 ;; esac ;;
            github)
                case "$_host" in *github.com*) _match=1 ;; esac ;;
            gitlab)
                case "$_host" in *gitlab.com*|*framagit.org*) _match=1 ;; esac ;;
        esac
        if [ "$_match" -eq 1 ]; then
            printf '%s %s\n' "$_owner" "$_repo"
            return 0
        fi
    fi
    return 1
}

detect_repo_info() {
    _target_path="${1:-.}"

    if [ ! -d "$_target_path" ]; then
        return 1
    fi

    # Check origin push URL first, then fetch URL
    _origin_url=$(git -C "$_target_path" config remote.origin.pushurl 2>/dev/null \
        || git -C "$_target_path" config remote.origin.url 2>/dev/null \
        || git -C "$_target_path" remote get-url origin 2>/dev/null || true)

    if [ -z "$_origin_url" ]; then
        # Try any configured remote
        _first_remote=$(git -C "$_target_path" remote 2>/dev/null | head -n 1 || true)
        if [ -n "$_first_remote" ]; then
            _origin_url=$(git -C "$_target_path" config "remote.${_first_remote}.url" 2>/dev/null \
                || git -C "$_target_path" remote get-url "$_first_remote" 2>/dev/null || true)
        fi
    fi

    [ -z "$_origin_url" ] && return 1

    if ! _parsed=$(parse_remote_url "$_origin_url" 2>/dev/null); then
        return 1
    fi

    _rhost=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
    _rowner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
    _rname=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)

    if ! _rforgeapp=$(detect_forgeapp "$_rhost" 2>/dev/null); then
        return 1
    fi
    [ "$_rforgeapp" = "unknown" ] && return 1

    printf '%s %s %s %s\n' "$_rhost" "$_rowner" "$_rname" "$_rforgeapp"
    return 0
}

get_git_root() {
    git rev-parse --show-toplevel 2>/dev/null || true
}

get_repo_slug() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    _o=$(printf '%s\n' "$_info" | cut -d' ' -f2)
    _r=$(printf '%s\n' "$_info" | cut -d' ' -f3)
    printf '%s/%s\n' "$_o" "$_r"
    return 0
}

get_repo_owner() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f2
    return 0
}

get_repo_name() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f3
    return 0
}

get_repo_domain() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f1
    return 0
}

get_repo_forge() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f1
    return 0
}

get_repo_forgeapp() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f4
    return 0
}

ensure_gitignored() {
    _target_file="$1"
    _git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

    [ -z "$_git_root" ] && return 0
    [ ! -d "$_git_root/.git" ] && return 0

    _gitignore_file="$_git_root/.gitignore"

    # If git already ignores this path, nothing to do
    if git check-ignore -q "$_target_file" 2>/dev/null; then
        return 0
    fi

    # Determine relative path from git root
    _rel_path=$(printf '%s\n' "$_target_file" | sed "s|^$_git_root/||")

    case "$_rel_path" in
        .antigravity/*)
            if [ ! -f "$_gitignore_file" ] || ! grep -q "^\.antigravity/" "$_gitignore_file" 2>/dev/null; then
                printf '.antigravity/\n' >> "$_gitignore_file"
            fi
            ;;
        *)
            _escaped=$(printf '%s\n' "$_rel_path" | sed 's/[.[\*^$]/\\&/g')
            if [ ! -f "$_gitignore_file" ] || ! grep -q "^$_escaped$" "$_gitignore_file" 2>/dev/null; then
                printf '%s\n' "$_rel_path" >> "$_gitignore_file"
            fi
            ;;
    esac
    return 0
}

get_tea_token() {
    _target="$1"
    _cfg="${HOME}/.config/tea/config.yml"
    [ -f "$_cfg" ] || return 1

    _found_target=0
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            *"- name:"*"$_target"*|*"name:"*"$_target"*)
                _found_target=1
                ;;
            *"- name:"*)
                _found_target=0
                ;;
            *"token:"*)
                if [ "$_found_target" -eq 1 ]; then
                    _token=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*token:[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                    printf '%s\n' "$_token"
                    return 0
                fi
                ;;
        esac
    done < "$_cfg"
    return 1
}

get_gh_token() {
    _target_host="${1:-github.com}"
    _gh_cfg_dir="${GH_CONFIG_DIR:-${XDG_CONFIG_HOME:-${HOME}/.config}/gh}"
    _cfg="${_gh_cfg_dir}/hosts.yml"

    if [ -f "$_cfg" ]; then
        _found_host=0
        while IFS= read -r _line || [ -n "$_line" ]; do
            case "$_line" in
                *"$_target_host:"*)
                    _found_host=1
                    ;;
                *":"*)
                    case "$_line" in
                        " "*|"\t"*)
                            ;;
                        *)
                            _found_host=0
                            ;;
                    esac
                    ;;
            esac
            if [ "$_found_host" -eq 1 ]; then
                case "$_line" in
                    *"oauth_token:"*)
                        _token=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*oauth_token:[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                        if [ -n "$_token" ]; then
                            printf '%s\n' "$_token"
                            return 0
                        fi
                        ;;
                esac
            fi
        done < "$_cfg"
    fi

    # Fallback to gh CLI auth token if gh is available
    if command -v gh >/dev/null 2>&1; then
        _tok=$(gh auth token -h "$_target_host" 2>/dev/null || gh auth token 2>/dev/null || true)
        if [ -n "$_tok" ]; then
            printf '%s\n' "$_tok"
            return 0
        fi
    fi

    return 1
}

resolve_forge_token() {
    _forge="$1"
    _keys=""

    case "$_forge" in
        *codeberg.org*|codeberg|forgejo|forgeapp_forgejo)
            [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
            [ -n "${CODEBERG_TOKEN:-}" ] && { printf '%s\n' "$CODEBERG_TOKEN"; return 0; }
            _keys="forgejo_token FORGEJO_TOKEN codeberg_token CODEBERG_TOKEN"
            ;;
        *tildegit.org*|tildegit)
            [ -n "${TILDEGIT_TOKEN:-}" ] && { printf '%s\n' "$TILDEGIT_TOKEN"; return 0; }
            [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
            _keys="tildegit_token TILDEGIT_TOKEN forgejo_token FORGEJO_TOKEN"
            ;;
        *github.com*|github|forgeapp_github)
            [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
            [ -n "${GH_TOKEN:-}" ] && { printf '%s\n' "$GH_TOKEN"; return 0; }
            _keys="github_token GITHUB_TOKEN GH_TOKEN"
            ;;
        *gitlab.com*|*framagit.org*|gitlab|forgeapp_gitlab)
            [ -n "${GITLAB_TOKEN:-}" ] && { printf '%s\n' "$GITLAB_TOKEN"; return 0; }
            [ -n "${FRAMAGIT_TOKEN:-}" ] && { printf '%s\n' "$FRAMAGIT_TOKEN"; return 0; }
            _keys="gitlab_token GITLAB_TOKEN framagit_token FRAMAGIT_TOKEN"
            ;;
        *.sr.ht*|git.sr.ht|todo.sr.ht|sourcehut|forgeapp_sourcehut)
            [ -n "${SOURCEHUT_TOKEN:-}" ] && { printf '%s\n' "$SOURCEHUT_TOKEN"; return 0; }
            _keys="sourcehut_token SOURCEHUT_TOKEN"
            ;;
        *)
            _inferred_app=$(detect_forgeapp "$_forge" 2>/dev/null || true)
            case "$_inferred_app" in
                forgeapp_forgejo)
                    [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
                    _keys="forgejo_token FORGEJO_TOKEN"
                    ;;
                forgeapp_github)
                    [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
                    _keys="github_token GITHUB_TOKEN"
                    ;;
                forgeapp_gitlab)
                    [ -n "${GITLAB_TOKEN:-}" ] && { printf '%s\n' "$GITLAB_TOKEN"; return 0; }
                    _keys="gitlab_token GITLAB_TOKEN"
                    ;;
                forgeapp_sourcehut)
                    [ -n "${SOURCEHUT_TOKEN:-}" ] && { printf '%s\n' "$SOURCEHUT_TOKEN"; return 0; }
                    _keys="sourcehut_token SOURCEHUT_TOKEN"
                    ;;
                *)
                    return 1
                    ;;
            esac
            ;;
    esac

    # 1. Check local and home .env files
    for _env_f in ".env" "${HOME}/.env"; do
        if [ -f "$_env_f" ] && command -v read_file_key >/dev/null 2>&1; then
            for _k in $_keys; do
                if _tok=$(read_file_key "$_env_f" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                    printf '%s\n' "$_tok"
                    return 0
                fi
            done
        fi
    done

    # 2. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        for _k in $_keys; do
            if _tok=$(read_file_key "$_local_cfg" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                printf '%s\n' "$_tok"
                return 0
            fi
        done
    fi

    # 3. Check global config
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    else
        _global_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
    fi

    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        for _k in $_keys; do
            if _tok=$(read_file_key "$_global_cfg" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                printf '%s\n' "$_tok"
                return 0
            fi
        done
    fi

    # 4. Check tea/gh config
    case "$_forge" in
        *codeberg.org*|*tildegit.org*|*gitea.com*|forgejo|codeberg|tildegit|forgeapp_forgejo)
            get_tea_token "$_forge" && return 0
            get_tea_token "codeberg" && return 0
            get_tea_token "forgejo" && return 0
            ;;
        *github.com*|github|forgeapp_github)
            get_gh_token "github.com" && return 0
            ;;
    esac

    return 1
}

escape_json_string() {
    _raw="$1"
    printf '%s' "$_raw" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g'
}

_repo_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        REPO_C_RED='\033[0;31m'
        REPO_C_GREEN='\033[0;32m'
        REPO_C_YELLOW='\033[0;33m'
        REPO_C_CYAN='\033[0;36m'
        REPO_C_BOLD='\033[1m'
        REPO_C_RESET='\033[0m'
    else
        REPO_C_RED=''
        REPO_C_GREEN=''
        REPO_C_YELLOW=''
        REPO_C_CYAN=''
        REPO_C_BOLD=''
        REPO_C_RESET=''
    fi
}

_repo_discover_list() {
    _d_dir="${1:-}"
    _d_catalog="${2:-}"

    # 1. If explicit catalog provided
    if [ -n "$_d_catalog" ]; then
        if [ ! -f "$_d_catalog" ]; then
            echo "Error: Catalog file '$_d_catalog' not found" >&2
            return 1
        fi
        _cat_dir=$(cd "$(dirname "$_d_catalog")" && pwd)
        _cat_names=$(sed -n 's/^[[:space:]]*"name":[[:space:]]*"\([^"]*\)".*/\1/p' "$_d_catalog" 2>/dev/null || true)
        for _cn in $_cat_names; do
            if [ -d "${_cat_dir}/${_cn}/.git" ]; then
                printf '%s\n' "${_cat_dir}/${_cn}"
            elif [ -d "${HOME}/src/${_cn}/.git" ]; then
                printf '%s\n' "${HOME}/src/${_cn}"
            elif [ -d "./${_cn}/.git" ]; then
                printf '%s\n' "./${_cn}"
            fi
        done
        return 0
    fi

    # 2. If explicit directory provided
    if [ -n "$_d_dir" ]; then
        if [ ! -d "$_d_dir" ]; then
            echo "Error: Directory '$_d_dir' not found" >&2
            return 1
        fi
        if [ -d "$_d_dir/.git" ]; then
            printf '%s\n' "$_d_dir"
            return 0
        fi
        for _sub in "$_d_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # 3. Default discovery:
    # Check for catalog.json in ., .., or $HOME/src
    _found_cat=""
    if [ -f "./catalog.json" ]; then
        _found_cat="./catalog.json"
    elif [ -f "../catalog.json" ]; then
        _found_cat="../catalog.json"
    elif [ -f "${HOME}/src/catalog.json" ]; then
        _found_cat="${HOME}/src/catalog.json"
    fi

    if [ -n "$_found_cat" ]; then
        _cat_dir=$(cd "$(dirname "$_found_cat")" && pwd)
        _cat_names=$(sed -n 's/^[[:space:]]*"name":[[:space:]]*"\([^"]*\)".*/\1/p' "$_found_cat" 2>/dev/null || true)
        for _cn in $_cat_names; do
            if [ -d "${_cat_dir}/${_cn}/.git" ]; then
                printf '%s\n' "${_cat_dir}/${_cn}"
            elif [ -d "${HOME}/src/${_cn}/.git" ]; then
                printf '%s\n' "${HOME}/src/${_cn}"
            fi
        done
        for _sub in "$_cat_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # 4. Fallback: check sibling repos in parent dir or ~/src or current dir
    _parent_dir=$(cd .. 2>/dev/null && pwd || true)
    _has_siblings=0
    if [ -n "$_parent_dir" ] && [ "$_parent_dir" != "/" ]; then
        for _sub in "$_parent_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                _has_siblings=1
                printf '%s\n' "$_sub"
            fi
        done
    fi
    [ "$_has_siblings" -eq 1 ] && return 0

    if [ -d "${HOME}/src" ]; then
        for _sub in "${HOME}/src"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # If current directory is a git repo
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git rev-parse --show-toplevel 2>/dev/null
        return 0
    fi

    return 0
}

_repo_audit_single() {
    _rpath="$1"
    _do_fetch="${2:-0}"

    [ -d "$_rpath" ] || return 1

    _rname=$(basename "$_rpath")
    _rbranch=$(git -C "$_rpath" symbolic-ref --short HEAD 2>/dev/null || git -C "$_rpath" rev-parse --short HEAD 2>/dev/null || echo "detached")

    # Dirty status: count unstaged, staged, or untracked files
    _rdirty=$(git -C "$_rpath" status --porcelain 2>/dev/null | wc -l | tr -d ' ')

    # Unpushed commits
    _rupstream=$(git -C "$_rpath" rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)
    _runpushed=0
    if [ -n "$_rupstream" ]; then
        _runpushed=$(git -C "$_rpath" rev-list --count "${_rupstream}..HEAD" 2>/dev/null || echo "0")
    else
        _has_remotes=$(git -C "$_rpath" remote 2>/dev/null || true)
        if [ -n "$_has_remotes" ]; then
            if git -C "$_rpath" rev-parse --verify "origin/${_rbranch}" >/dev/null 2>&1; then
                _runpushed=$(git -C "$_rpath" rev-list --count "origin/${_rbranch}..HEAD" 2>/dev/null || echo "0")
            else
                _runpushed=$(git -C "$_rpath" rev-list --count HEAD 2>/dev/null || echo "0")
            fi
        fi
    fi

    # Tickets / issues count
    _rissues=0
    if [ -f "$_rpath/.antigravity/tickets.md" ]; then
        _rissues=$(grep -c '^## \[#[0-9]' "$_rpath/.antigravity/tickets.md" 2>/dev/null || true)
        _rissues=$(printf '%s' "$_rissues" | tr -cd '0-9')
        [ -z "$_rissues" ] && _rissues=0
    fi

    if [ "$_do_fetch" -eq 1 ] && [ "$_rissues" -eq 0 ]; then
        _rinfo=$(detect_repo_info "$_rpath" 2>/dev/null || true)
        if [ -n "$_rinfo" ]; then
            _rf_owner=$(printf '%s' "$_rinfo" | cut -d' ' -f2)
            _rf_repo=$(printf '%s' "$_rinfo" | cut -d' ' -f3)
            if command -v tea >/dev/null 2>&1; then
                _tea_cnt=$(tea issues --repo "${_rf_owner}/${_rf_repo}" --state open --fields index 2>/dev/null | tr -cd '0-9\n' | grep -v '^$' | wc -l | tr -d ' ' || echo "0")
                [ -n "$_tea_cnt" ] && [ "$_tea_cnt" -gt 0 ] && _rissues="$_tea_cnt"
            fi
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$_rname" "$_rbranch" "$_rdirty" "$_runpushed" "$_rissues" "$_rpath"
    return 0
}

cmd_repo() {
    _opt_dirty=0
    _opt_unpushed=0
    _opt_issues=0
    _opt_json=0
    _opt_fetch=0
    _opt_dir=""
    _opt_catalog=""

    while [ $# -gt 0 ]; do
        case "$1" in
            "new"|"create"|"init")
                shift
                if command -v cmd_newrepo >/dev/null 2>&1; then
                    cmd_newrepo "$@"
                    return $?
                else
                    echo "Error: cmd_newrepo is not available." >&2
                    return 1
                fi
                ;;
            "status"|"list"|"overview")
                shift
                ;;
            "-h"|"--help"|"help")
                if command -v show_help >/dev/null 2>&1; then
                    show_help "repo"
                else
                    echo "Usage: cccp repo [status|list] [--dirty] [--unpushed] [--issues] [--json] [-d <dir>] [-c <catalog>] [-f|--fetch]"
                fi
                return 0
                ;;
            "--dirty")
                _opt_dirty=1
                shift
                ;;
            "--unpushed")
                _opt_unpushed=1
                shift
                ;;
            "--issues"|"--tickets")
                _opt_issues=1
                shift
                ;;
            "--json")
                _opt_json=1
                shift
                ;;
            "-f"|"--fetch")
                _opt_fetch=1
                shift
                ;;
            "-d"|"--dir")
                shift
                [ $# -gt 0 ] || { echo "Error: --dir requires directory path" >&2; return 1; }
                _opt_dir="$1"
                shift
                ;;
            "-c"|"--catalog")
                shift
                [ $# -gt 0 ] || { echo "Error: --catalog requires file path" >&2; return 1; }
                _opt_catalog="$1"
                shift
                ;;
            *)
                if [ -d "$1" ]; then
                    _opt_dir="$1"
                    shift
                else
                    echo "Error: Unknown option or subcommand: $1" >&2
                    echo "Run 'cccp repo --help' for usage." >&2
                    return 1
                fi
                ;;
        esac
    done

    _discovered=$(_repo_discover_list "$_opt_dir" "$_opt_catalog" 2>/dev/null | sort -u || true)

    if [ -z "$_discovered" ]; then
        if [ "$_opt_json" -eq 1 ]; then
            printf '[\n]\n'
        else
            echo "Nenhum repositório Git encontrado no escopo especificado."
        fi
        return 0
    fi

    _repo_colors

    _total_scanned=0
    _total_matched=0
    _total_dirty=0
    _total_unpushed=0
    _total_issues=0

    _matching_rows=""
    _first_json=1

    if [ "$_opt_json" -eq 1 ]; then
        printf '[\n'
    fi

    # Audit each discovered repository
    while IFS= read -r _rpath || [ -n "$_rpath" ]; do
        [ -n "$_rpath" ] || continue
        _row=$(_repo_audit_single "$_rpath" "$_opt_fetch" 2>/dev/null || true)
        [ -n "$_row" ] || continue

        _name=$(printf '%s' "$_row" | cut -f1)
        _branch=$(printf '%s' "$_row" | cut -f2)
        _dirty=$(printf '%s' "$_row" | cut -f3)
        _unpushed=$(printf '%s' "$_row" | cut -f4)
        _issues=$(printf '%s' "$_row" | cut -f5)
        _path=$(printf '%s' "$_row" | cut -f6)

        _total_scanned=$((_total_scanned + 1))
        [ "$_dirty" -gt 0 ] && _total_dirty=$((_total_dirty + 1))
        [ "$_unpushed" -gt 0 ] && _total_unpushed=$((_total_unpushed + 1))
        [ "$_issues" -gt 0 ] && _total_issues=$((_total_issues + 1))

        # Filter check
        if [ "$_opt_dirty" -eq 1 ] || [ "$_opt_unpushed" -eq 1 ] || [ "$_opt_issues" -eq 1 ]; then
            _matches=0
            [ "$_opt_dirty" -eq 1 ] && [ "$_dirty" -gt 0 ] && _matches=1
            [ "$_opt_unpushed" -eq 1 ] && [ "$_unpushed" -gt 0 ] && _matches=1
            [ "$_opt_issues" -eq 1 ] && [ "$_issues" -gt 0 ] && _matches=1
            [ "$_matches" -eq 1 ] || continue
        fi

        _total_matched=$((_total_matched + 1))

        # Determine textual status & Gosplan label
        if [ "$_dirty" -gt 0 ] && [ "$_unpushed" -gt 0 ]; then
            _status_txt="DIRTY/UNPUSHED"
            _status_label="MODIFICADO / PENDENTE"
            _status_color="$REPO_C_RED"
        elif [ "$_dirty" -gt 0 ]; then
            _status_txt="DIRTY"
            _status_label="MODIFICADO"
            _status_color="$REPO_C_YELLOW"
        elif [ "$_unpushed" -gt 0 ]; then
            _status_txt="UNPUSHED"
            _status_label="PENDENTE DE ENVIO"
            _status_color="$REPO_C_CYAN"
        elif [ "$_issues" -gt 0 ]; then
            _status_txt="ISSUES"
            _status_label="COM TAREFAS"
            _status_color="$REPO_C_CYAN"
        else
            _status_txt="CLEAN"
            _status_label="EM CONFORMIDADE"
            _status_color="$REPO_C_GREEN"
        fi

        if [ "$_opt_json" -eq 1 ]; then
            [ "$_first_json" -eq 1 ] && _first_json=0 || printf ',\n'
            cat <<EOF
  {
    "name": "$(escape_json_string "$_name")",
    "path": "$(escape_json_string "$_path")",
    "branch": "$(escape_json_string "$_branch")",
    "dirty": $_dirty,
    "unpushed": $_unpushed,
    "issues": $_issues,
    "status": "$(escape_json_string "$_status_txt")"
  }
EOF
        else
            # Truncate strings for neat Gosplan tabular formatting
            _name_trunc=$(printf '%.22s' "$_name")
            _branch_trunc=$(printf '%.20s' "$_branch")
            
            # Format row
            _matching_rows="${_matching_rows}$(printf '%-22s %-20s %6s %9s %8s  %b[%s]%b\n' \
                "$_name_trunc" "$_branch_trunc" "$_dirty" "$_unpushed" "$_issues" \
                "$_status_color" "$_status_label" "$REPO_C_RESET")
"
        fi
    done <<EOF
$_discovered
EOF

    if [ "$_opt_json" -eq 1 ]; then
        printf '\n]\n'
        return 0
    fi

    # Display Gosplan Header
    printf '%b╔══════════════════════════════════════════════════════════════════════════════════════╗%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"
    printf '%b║             ГОСПЛАН — REGISTRO GERAL E STATUS DE COOPERATIVAS (REPOS)                ║%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"
    printf '%b╚══════════════════════════════════════════════════════════════════════════════════════╝%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"

    _filter_desc=""
    [ "$_opt_dirty" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }modificados (dirty)"
    [ "$_opt_unpushed" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }pendentes de envio (unpushed)"
    [ "$_opt_issues" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }com tickets em aberto (issues)"

    if [ -n "$_filter_desc" ]; then
        printf 'Filtro ativo: apenas %s\n' "$_filter_desc"
    fi
    echo ""

    if [ -z "$_matching_rows" ]; then
        echo "Nenhum repositório correspondente aos filtros foi encontrado."
        echo ""
    else
        printf '%b%-22s %-20s %6s %9s %8s  %s%b\n' "$REPO_C_BOLD" "REPOSITÓRIO" "RAMAL" "DIRTY" "UNPUSHED" "TICKETS" "ESTADO" "$REPO_C_RESET"
        printf '%s\n' '----------------------------------------------------------------------------------------'
        printf '%b' "$_matching_rows"
        printf '%s\n' '----------------------------------------------------------------------------------------'
    fi

    # Display Gosplan Summary
    echo "Resumo do Plano Geral:"
    echo "  • Repositórios auditados: ${_total_scanned} (correspondentes: ${_total_matched})"
    echo "  • Com alterações não salvas (dirty): ${_total_dirty}"
    echo "  • Com commits pendentes de envio (unpushed): ${_total_unpushed}"
    echo "  • Com tarefas/tickets em aberto: ${_total_issues}"
    echo ""
    return 0
}


# =============================================================================
# Comrades & Unified HTTP Dispatcher Functions
# =============================================================================
_cccp_get_bin_dir() {
    if [ -n "${COMRADES_BIN_DIR:-}" ]; then
        _bdir="$COMRADES_BIN_DIR"
    elif [ -n "${XDG_BIN_HOME:-}" ]; then
        _bdir="$XDG_BIN_HOME"
    elif [ -d "${HOME}/.local/bin" ] || mkdir -p "${HOME}/.local/bin" 2>/dev/null; then
        _bdir="${HOME}/.local/bin"
    else
        _bdir="${XDG_DATA_HOME:-$HOME/.local/share}/cccp/bin"
    fi
    mkdir -p "$_bdir" 2>/dev/null || true

    # Ensure target directory is in PATH for active shell execution
    case ":${PATH:-}:" in
        *":${_bdir}:"*) ;;
        *) export PATH="${_bdir}:${PATH:-}" ;;
    esac

    printf '%s\n' "$_bdir"
}

_cccp_detect_arch() {
    _raw_os=$(uname -s 2>/dev/null | tr '[:upper:]' '[:lower:]' || echo "unknown")
    _raw_arch=$(uname -m 2>/dev/null | tr '[:upper:]' '[:lower:]' || echo "unknown")

    _os="unknown"
    case "$_raw_os" in
        linux*) _os="linux" ;;
        darwin*) _os="macos" ;;
        freebsd*|openbsd*|netbsd*) _os="bsd" ;;
    esac

    _arch="unknown"
    case "$_raw_arch" in
        x86_64|amd64) _arch="x86_64" ;;
        aarch64|arm64) _arch="aarch64" ;;
        armv7*|armv6*|armhf|armv5*|arm|arm32) _arch="armv7" ;;
        i386|i486|i586|i686|x86) _arch="i686" ;;
        riscv64*) _arch="riscv64" ;;
        riscv32*) _arch="riscv32" ;;
    esac

    # If architecture is still unknown, check bitness and CPU hints
    if [ "$_arch" = "unknown" ]; then
        case "$_raw_arch" in
            *arm*) _arch="armv7" ;;
            *)
                if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                    _arch="i686"
                else
                    _arch="x86_64"
                fi
                ;;
        esac
    fi

    printf '%s-%s\n' "$_os" "$_arch"
}

_cccp_download_raw() {
    _url="$1"
    _dest="$2"

    [ -z "$_url" ] || [ -z "$_dest" ] && return 1

    # Tier 1: Modern toolchain
    if command -v sv >/dev/null 2>&1; then
        sv -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v curl >/dev/null 2>&1; then
        curl -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$_url" -O "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v busybox >/dev/null 2>&1 && busybox wget --help >/dev/null 2>&1; then
        busybox wget -q "$_url" -O "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    fi

    # Tier 2: 1960/1970 retro fallbacks for constrained routers (OpenWrt without SSL)
    case "$_url" in
        ftp://*)
            _raw_ftp="${_url#ftp://}"
            _ftp_host="${_raw_ftp%%/*}"
            _ftp_path="/${_raw_ftp#*/}"

            if command -v ftpget >/dev/null 2>&1; then
                ftpget "$_ftp_host" "$_dest" "$_ftp_path" 2>/dev/null && [ -s "$_dest" ] && return 0
            elif command -v busybox >/dev/null 2>&1 && busybox ftpget --help >/dev/null 2>&1; then
                busybox ftpget "$_ftp_host" "$_dest" "$_ftp_path" 2>/dev/null && [ -s "$_dest" ] && return 0
            elif command -v ftp >/dev/null 2>&1; then
                printf "open %s\nuser anonymous cccp@gosplan.su\nbinary\nget %s %s\nquit\n" "$_ftp_host" "$_ftp_path" "$_dest" | ftp -n 2>/dev/null
                [ -s "$_dest" ] && return 0
            fi
            ;;

        http://*)
            _raw_http="${_url#http://}"
            _http_host_port="${_raw_http%%/*}"
            _http_path="/${_raw_http#*/}"
            case "$_http_host_port" in
                *:*)
                    _http_host="${_http_host_port%%:*}"
                    _http_port="${_http_host_port##*:}"
                    ;;
                *)
                    _http_host="$_http_host_port"
                    _http_port="80"
                    ;;
            esac

            # Netcat (nc / busybox nc) HTTP/1.0
            if command -v nc >/dev/null 2>&1; then
                if printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" | \
                    nc "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            elif command -v busybox >/dev/null 2>&1 && busybox nc --help >/dev/null 2>&1; then
                if printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" | \
                    busybox nc "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            fi

            # Telnet (1960s style TCP handshake)
            if command -v telnet >/dev/null 2>&1; then
                if (printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host"; sleep 2) | \
                    telnet "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            elif command -v busybox >/dev/null 2>&1 && busybox telnet --help >/dev/null 2>&1; then
                if (printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host"; sleep 2) | \
                    busybox telnet "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            fi

            # Raw TCP socket via shell builtin /dev/tcp (if supported)
            if (printf '' > "/dev/tcp/${_http_host}/${_http_port}") 2>/dev/null; then
                (
                    exec 3<>/dev/tcp/"${_http_host}"/"${_http_port}"
                    printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" >&3
                    sed '1,/^[\r]*$/d' <&3 > "$_dest"
                    exec 3>&-
                ) 2>/dev/null
                [ -s "$_dest" ] && return 0
                rm -f "$_dest"
            fi
            ;;

        https://*)
            # Low-level HTTPS bootstrap via OpenSSL s_client
            if command -v openssl >/dev/null 2>&1; then
                _host=$(printf '%s' "$_url" | sed -e 's|^https://||' -e 's|/.*$||')
                _path="/$(printf '%s' "$_url" | sed -e 's|^https://[^/]*/*||')"
                if printf "GET %s HTTP/1.1\r\nHost: %s\r\nUser-Agent: CCCP-Bootstrap/1.0\r\nConnection: close\r\n\r\n" "$_path" "$_host" | \
                    openssl s_client -quiet -connect "${_host}:443" -servername "$_host" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    if [ -s "$_dest" ]; then
                        return 0
                    fi
                fi
                rm -f "$_dest"
            fi
            ;;
    esac

    return 1
}

_cccp_ensure_comrade() {
    _name="${1:-}"
    _silent="${2:-0}"

    [ -z "$_name" ] && return 1

    # Check if target binary is already in PATH
    _target_bin="$_name"
    case "$_name" in
        kv|kvant) _target_bin="kv" ;;
        sv|svyaz) _target_bin="sv" ;;
        mk|mayak) _target_bin="mk" ;;
    esac

    if command -v "$_target_bin" >/dev/null 2>&1; then
        return 0
    fi

    _bdir=$(_cccp_get_bin_dir)
    if [ -x "${_bdir}/${_target_bin}" ]; then
        case ":${PATH:-}:" in
            *":${_bdir}:"*) ;;
            *) export PATH="${_bdir}:${PATH:-}" ;;
        esac
        return 0
    fi

    # Map comrade to repo and archive name
    _repo=""
    _prefix=""
    case "$_name" in
        kv|kvant)
            _repo="kvant"
            _prefix="kv"
            ;;
        sv|svyaz)
            _repo="svyaz"
            _prefix="sv"
            ;;
        mk|mayak)
            _repo="mayak"
            _prefix="mk"
            ;;
        *)
            [ "$_silent" -eq 0 ] && echo "Erro: Camarada desconhecido '$_name'." >&2
            return 1
            ;;
    esac

    _arch=$(_cccp_detect_arch)
    _asset="${_prefix}-${_arch}.tar.gz"

    # Default fallback if unknown architecture
    case "$_arch" in
        unknown*|*-unknown)
            case "${_raw_arch:-}" in
                *arm*) _asset="${_prefix}-linux-armv7.tar.gz" ;;
                *)
                    if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                        _asset="${_prefix}-linux-i686.tar.gz"
                    else
                        _asset="${_prefix}-linux-x86_64.tar.gz"
                    fi
                    ;;
            esac
            ;;
    esac

    # Allow custom mirror via CCCP_COMRADES_MIRROR or comrades_mirror config
    _mirror="${CCCP_COMRADES_MIRROR:-${COMRADES_MIRROR:-}}"
    if [ -z "$_mirror" ] && command -v cmd_config >/dev/null 2>&1; then
        _mirror=$(cmd_config comrades_mirror 2>/dev/null || true)
    fi

    _primary_url="https://codeberg.org/lumenpink/${_repo}/releases/download/nightly/${_asset}"

    _urls=""
    if [ -n "$_mirror" ]; then
        _urls="${_mirror%/}/${_asset} ${_primary_url}"
    else
        _urls="${_primary_url}"
    fi

    if [ "$_silent" -eq 0 ]; then
        echo "★ [ГОСПЛАН] Convocando camarada '${_name}' (${_asset})... ★"
    fi

    _tmp_tar=$(mktemp 2>/dev/null || printf '/tmp/cccp_comrade_%s_%s.tar.gz' "$_name" "$$")
    _download_ok=0
    for _try_url in $_urls; do
        if _cccp_download_raw "$_try_url" "$_tmp_tar"; then
            _download_ok=1
            break
        fi
    done

    if [ "$_download_ok" -eq 0 ]; then
        rm -f "$_tmp_tar"
        if [ "$_silent" -eq 0 ]; then
            echo "Aviso: Falha ao baixar camarada '${_name}' das fontes disponíveis." >&2
            echo "Dica: Em sistemas minimalistas sem SSL (OpenWrt/BusyBox), configure um mirror HTTP ou FTP:" >&2
            echo "      export CCCP_COMRADES_MIRROR=\"http://seu-servidor/cccp\"" >&2
        fi
        return 1
    fi

    if ! tar -xzf "$_tmp_tar" -C "$_bdir" 2>/dev/null; then
        rm -f "$_tmp_tar"
        if [ "$_silent" -eq 0 ]; then
            echo "Aviso: Falha ao descompactar ${_asset} em ${_bdir}." >&2
        fi
        return 1
    fi

    rm -f "$_tmp_tar"
    chmod +x "${_bdir}"/* 2>/dev/null || true

    if [ "$_silent" -eq 0 ]; then
        echo "★ [ГОСПЛАН] Camarada '${_name}' recrutado e instalado com sucesso em ${_bdir} ★"
    fi

    return 0
}

_cccp_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi

    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v sv >/dev/null 2>&1; then
        sv "$@"
        return $?
    fi

    if command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
        return $?
    fi

    # Check local bin dir
    _bdir=$(_cccp_get_bin_dir)
    if [ -x "${_bdir}/sv" ]; then
        "${_bdir}/sv" "$@"
        return $?
    fi

    # Attempt silent bootstrap of sv
    if _cccp_ensure_comrade "sv" 1 >/dev/null 2>&1; then
        if command -v sv >/dev/null 2>&1; then
            sv "$@"
            return $?
        fi
        if [ -x "${_bdir}/sv" ]; then
            "${_bdir}/sv" "$@"
            return $?
        fi
    fi

    # Fallback to curl
    if command -v curl >/dev/null 2>&1; then
        curl "$@"
        return $?
    fi

    echo "Error: Nenhum cliente HTTP disponível ('sv', 'svyaz' ou 'curl' necessários)." >&2
    return 127
}

cmd_comrades() {
    _sub="${1:-status}"

    case "$_sub" in
        status)
            echo "★ [ГОСПЛАН] Registro Central dos Camaradas do Ecossistema ★"
            echo "==========================================================="
            _bdir=$(_cccp_get_bin_dir)
            echo "Diretório de Binários: ${_bdir}"
            echo ""
            printf "%-10s %-12s %-16s %s\n" "CAMARADA" "PAPEL" "ESTADO" "LOCALIZAÇÃO"
            printf "%-10s %-12s %-16s %s\n" "--------" "-----" "------" "-----------"

            for _c in "kv" "sv" "mk"; do
                _role=""
                case "$_c" in
                    kv) _role="Editor (kvant)" ;;
                    sv) _role="HTTP (svyaz)" ;;
                    mk) _role="Static (mayak)" ;;
                esac

                _p=$(command -v "$_c" 2>/dev/null || true)
                if [ -n "$_p" ]; then
                    _ver=$("$_c" --version 2>/dev/null | head -n 1 || echo "presente")
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "PRONTO ($_ver)" "$_p"
                elif [ -x "${_bdir}/${_c}" ]; then
                    _ver=$("${_bdir}/${_c}" --version 2>/dev/null | head -n 1 || echo "presente")
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "PRONTO ($_ver)" "${_bdir}/${_c}"
                else
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "NÃO INSTALADO" "(disponível via 'cccp comrades install')"
                fi
            done
            echo ""
            ;;

        install|recrutar)
            _target="${2:-all}"
            echo "★ [ГОСПЛАН] Recrutando camaradas para o posto de trabalho... ★"
            case "$_target" in
                kv|kvant)
                    _cccp_ensure_comrade "kv" 0
                    ;;
                sv|svyaz)
                    _cccp_ensure_comrade "sv" 0
                    ;;
                mk|mayak)
                    _cccp_ensure_comrade "mk" 0
                    ;;
                all|*)
                    _cccp_ensure_comrade "kv" 0
                    _cccp_ensure_comrade "sv" 0
                    _cccp_ensure_comrade "mk" 0
                    ;;
            esac
            ;;

        update|atualizar)
            echo "★ [ГОСПЛАН] Atualizando camaradas para as últimas compilações nightly... ★"
            _bdir=$(_cccp_get_bin_dir)
            _arch=$(_cccp_detect_arch)
            case "$_arch" in
                unknown*|*-unknown)
                    case "${_raw_arch:-}" in
                        *arm*) _arch="linux-armv7" ;;
                        *)
                            if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                                _arch="linux-i686"
                            else
                                _arch="linux-x86_64"
                            fi
                            ;;
                    esac
                    ;;
            esac

            _mirror="${CCCP_COMRADES_MIRROR:-${COMRADES_MIRROR:-}}"
            if [ -z "$_mirror" ] && command -v cmd_config >/dev/null 2>&1; then
                _mirror=$(cmd_config comrades_mirror 2>/dev/null || true)
            fi

            for _pair in "kv:kvant" "sv:svyaz" "mk:mayak"; do
                _bin=$(printf '%s' "$_pair" | cut -d: -f1)
                _repo=$(printf '%s' "$_pair" | cut -d: -f2)
                _asset="${_bin}-${_arch}.tar.gz"
                _primary_url="https://codeberg.org/lumenpink/${_repo}/releases/download/nightly/${_asset}"

                _urls=""
                if [ -n "$_mirror" ]; then
                    _urls="${_mirror%/}/${_asset} ${_primary_url}"
                else
                    _urls="${_primary_url}"
                fi

                echo "  -> Baixando ${_bin} (${_asset})..."
                _tmp_tar=$(mktemp 2>/dev/null || printf '/tmp/cccp_up_%s.tar.gz' "$_bin")
                _up_ok=0
                for _try_url in $_urls; do
                    if _cccp_download_raw "$_try_url" "$_tmp_tar"; then
                        _up_ok=1
                        break
                    fi
                done

                if [ "$_up_ok" -eq 1 ]; then
                    tar -xzf "$_tmp_tar" -C "$_bdir" 2>/dev/null || true
                    rm -f "$_tmp_tar"
                    chmod +x "${_bdir}/${_bin}"* 2>/dev/null || true
                    echo "  [OK] ${_bin} atualizado em ${_bdir}."
                else
                    rm -f "$_tmp_tar"
                    echo "  [ERRO] Falha ao atualizar ${_bin}." >&2
                fi
            done
            echo "★ Atualização concluída com sucesso! ★"
            ;;

        help|--help|-h)
            echo "★ CCCP - Gestão de Camaradas (Kvant e Svyaz) ★"
            echo ""
            echo "Uso:"
            echo "  cccp comrades [status]          Exibe o status dos binários kv e sv"
            echo "  cccp comrades install [all|kv|sv] Baixa e instala os comrades no sistema"
            echo "  cccp comrades update            Atualiza kv e sv para o último nightly"
            echo ""
            ;;

        *)
            echo "Uso: cccp comrades [status|install|update|help]" >&2
            return 1
            ;;
    esac
    return 0
}


# =============================================================================
# Tickets Functions
# =============================================================================
_tickets_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

resolve_tickets_info() {
    resolved_provider="${TICKETS_PROVIDER:-}"
    resolved_owner="${TICKETS_OWNER:-${CODEBERG_OWNER:-${CODEBERG_USER:-${GITHUB_OWNER:-${GITHUB_USER:-}}}}}"
    resolved_repo="${TICKETS_REPO:-${CODEBERG_REPO:-${GITHUB_REPO:-}}}"
    resolved_host=""

    # If provider, owner and repo are already known from environment/flags
    if [ -n "$resolved_provider" ] && [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        case "$resolved_provider" in
            github) resolved_host="github.com" ;;
            forgejo|codeberg) resolved_host="${resolved_host:-}" ;;
            gitlab) resolved_host="framagit.org" ;;
            sourcehut) resolved_host="git.sr.ht" ;;
            *) resolved_host="" ;;
        esac
        if [ -z "$resolved_host" ] && [ "$resolved_provider" = "forgejo" ]; then
            printf 'Warning: forgejo provider requires --host or a git remote to detect the instance URL.\n' >&2
        fi
        echo "$resolved_provider $resolved_owner $resolved_repo $resolved_host"
        return 0
    fi

    # Inspect git remotes if inside a git repository
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        if _dinfo=$(detect_repo_info 2>/dev/null); then
            _domain=$(printf '%s' "$_dinfo" | cut -d' ' -f1)
            p_owner=$(printf '%s' "$_dinfo" | cut -d' ' -f2)
            p_repo=$(printf '%s' "$_dinfo" | cut -d' ' -f3)
            _forgeapp=$(printf '%s' "$_dinfo" | cut -d' ' -f4)
            p_prov=""
            case "$_forgeapp" in
                forgeapp_github) p_prov="github" ;;
                forgeapp_forgejo) p_prov="forgejo" ;;
                forgeapp_gitlab) p_prov="gitlab" ;;
                forgeapp_sourcehut) p_prov="sourcehut" ;;
                *) p_prov="$_forgeapp" ;;
            esac
            resolved_provider="${resolved_provider:-$p_prov}"
            resolved_owner="${resolved_owner:-$p_owner}"
            resolved_repo="${resolved_repo:-$p_repo}"
            resolved_host="${resolved_host:-$_domain}"
            printf '%s %s %s %s\n' "$resolved_provider" "$resolved_owner" "$resolved_repo" "$resolved_host"
            return 0
        fi
    fi

    # Fallback if owner and repo are already known from environment
    if [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        if [ -z "$resolved_provider" ]; then
            if [ -n "${GITHUB_OWNER:-}" ] || [ -n "${GITHUB_REPO:-}" ] || [ -n "${GITHUB_TOKEN:-}" ]; then
                resolved_provider="github"
                resolved_host="github.com"
            elif [ -n "${CODEBERG_OWNER:-}" ] || [ -n "${CODEBERG_REPO:-}" ] || [ -n "${CODEBERG_TOKEN:-}" ]; then
                resolved_provider="forgejo"
                resolved_host="codeberg.org"
            elif [ -n "${FORGEJO_OWNER:-}" ] || [ -n "${FORGEJO_REPO:-}" ] || [ -n "${FORGEJO_TOKEN:-}" ] || [ -n "${FORGEJO_HOST:-}" ]; then
                resolved_provider="forgejo"
                resolved_host="${FORGEJO_HOST:-}"
            else
                resolved_provider="forgejo"
                resolved_host="${resolved_host:-}"
            fi
        else
            case "$resolved_provider" in
                github) resolved_host="${resolved_host:-github.com}" ;;
                codeberg) resolved_host="${resolved_host:-codeberg.org}" ;;
                forgejo) resolved_host="${resolved_host:-${FORGEJO_HOST:-}}" ;;
                gitlab) resolved_host="${resolved_host:-framagit.org}" ;;
                sourcehut) resolved_host="${resolved_host:-git.sr.ht}" ;;
            esac
        fi
        echo "$resolved_provider $resolved_owner $resolved_repo $resolved_host"
        return 0
    fi

    return 1
}

resolve_codeberg_info() {
    if [ -n "${CODEBERG_OWNER:-}" ] && [ -n "${CODEBERG_REPO:-}" ]; then
        echo "$CODEBERG_OWNER $CODEBERG_REPO"
        return 0
    fi
    if info=$(resolve_tickets_info 2>/dev/null); then
        p_prov=$(echo "$info" | cut -d' ' -f1)
        if [ "$p_prov" = "forgejo" ] || [ "$p_prov" = "codeberg" ]; then
            echo "$info" | cut -d' ' -f2,3
            return 0
        fi
    fi
    return 1
}

_tickets_fetch_github() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    if [ "$host" = "github.com" ]; then
        api_url="https://api.github.com/repos/${owner}/${repo}/issues?state=open&per_page=30"
    else
        api_url="https://${host}/api/v3/repos/${owner}/${repo}/issues?state=open&per_page=30"
    fi

    auth_header=""
    [ -n "$token" ] && auth_header="Authorization: Bearer ${token}"

    if [ -n "$auth_header" ]; then
        resp=$(_tickets_http -s -f -H "User-Agent: cccp" -H "$auth_header" "$api_url" 2>/dev/null || true)
    else
        resp=$(_tickets_http -s -f -H "User-Agent: cccp" "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | select(.pull_request == null) | {
        number: .number,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then [.labels[].name] else [] end),
        html_url: .html_url,
        body: .body
    }'
}

_tickets_fetch_forgejo() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    api_url="https://${host}/api/v1/repos/${owner}/${repo}/issues?state=open&type=issues&limit=30"

    if [ -n "$token" ]; then
        resp=$(_tickets_http -s -f -H "Authorization: token ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(_tickets_http -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | {
        number: .number,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then [.labels[].name] else [] end),
        html_url: .html_url,
        body: .body
    }'
}

_tickets_fetch_gitlab() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    url_encoded=$(echo "${owner}/${repo}" | sed -e 's|/|%2F|g')
    api_url="https://${host}/api/v4/projects/${url_encoded}/issues?state=opened&per_page=30"

    if [ -n "$token" ]; then
        resp=$(_tickets_http -s -f -H "PRIVATE-TOKEN: ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(_tickets_http -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | {
        number: .iid,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then (.labels | map(if type == "string" then . else .name end)) else [] end),
        html_url: .web_url,
        body: .description
    }'
}

_tickets_fetch_sourcehut() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    owner_clean=$(echo "$owner" | sed 's/^~//')
    api_url="https://todo.sr.ht/api/user/~${owner_clean}/trackers/${repo}/tickets"

    if [ -n "$token" ]; then
        resp=$(_tickets_http -s -f -H "Authorization: Bearer ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(_tickets_http -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1

    printf '%s\n' "$resp" | jq -c '.results[]? // .[]? | {
        number: .id,
        title: .title,
        state: .status,
        labels: [],
        html_url: ("https://todo.sr.ht/~" + "'"$owner_clean"'" + "/" + "'"$repo"'" + "/" + (.id|tostring)),
        body: .description
    }'
}

check_tickets_rate_limit() {
    rate_limit_file="${TICKETS_RATE_LIMIT_FILE:-}"
    if [ -z "$rate_limit_file" ]; then
        git_dir="$(git rev-parse --git-dir 2>/dev/null || true)"
        if [ -n "$git_dir" ] && [ -d "$git_dir" ]; then
            rate_limit_file="$git_dir/cccp_tickets_rate_limit"
        else
            rate_limit_file="${TMPDIR:-/tmp}/cccp_tickets_rate_limit_${USER:-default}"
        fi
    fi

    max_calls="${TICKETS_HOURLY_LIMIT:-55}"
    now=$(date +%s)
    cutoff=$((now - 3600))
    tmp_rl="${rate_limit_file}.tmp.$$"
    count=0

    if [ -f "$rate_limit_file" ]; then
        while IFS= read -r ts || [ -n "$ts" ]; do
            case "$ts" in
                ''|*[!0-9]*) continue ;;
            esac
            if [ "$ts" -gt "$cutoff" ] 2>/dev/null; then
                echo "$ts" >> "$tmp_rl"
                count=$((count + 1))
            fi
        done < "$rate_limit_file"
    fi

    if [ "$count" -ge "$max_calls" ]; then
        rm -f "$tmp_rl"
        echo "Предупреждение (Gosplan Quota): Hourly limit reached (${count}/${max_calls} requests in the last hour). Skipping tickets sync gracefully." >&2
        return 1
    fi

    echo "$now" >> "$tmp_rl"
    mv "$tmp_rl" "$rate_limit_file"
    return 0
}

ensure_tickets_gitignored() {
    if command -v ensure_gitignored >/dev/null 2>&1; then
        ensure_gitignored "$@"
        return $?
    fi
    return 0
}

cmd_tickets() {
    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    cli_provider=""
    cli_owner=""
    cli_repo=""
    cli_file=""
    cli_max_bytes=""
    cli_token="${TICKETS_TOKEN:-}"

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "tickets"
                else
                    echo "Usage: cccp tickets [--file <path>] [--owner <owner>] [--repo <repo>] [--provider <provider>] [--max-bytes <num>]"
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
            -p|--provider)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --provider" >&2; return 1; }
                cli_provider="$1"
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
    if ! command -v sv >/dev/null 2>&1 && ! command -v svyaz >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
        if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
            _cccp_ensure_comrade "sv" 1 >/dev/null 2>&1 || true
        fi
    fi
    if ! command -v sv >/dev/null 2>&1 && ! command -v svyaz >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
        echo "Warning: Required tool 'sv' or 'curl' is not installed or not in PATH. Skipping tickets sync." >&2
        return 0
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "Warning: Required tool 'jq' is not installed or not in PATH. Skipping tickets sync." >&2
        return 0
    fi

    # Resolve Provider, Owner, Repo, and Host
    provider="$cli_provider"
    owner="$cli_owner"
    repo="$cli_repo"
    host=""

    if [ -z "$provider" ] || [ -z "$owner" ] || [ -z "$repo" ]; then
        info=$(resolve_tickets_info || true)
        if [ -n "$info" ]; then
            [ -z "$provider" ] && provider=$(echo "$info" | cut -d' ' -f1)
            [ -z "$owner" ] && owner=$(echo "$info" | cut -d' ' -f2)
            [ -z "$repo" ] && repo=$(echo "$info" | cut -d' ' -f3)
            host=$(echo "$info" | cut -d' ' -f4)
        fi
    fi

    if [ -z "$owner" ] || [ -z "$repo" ]; then
        echo "Warning: Unable to determine repository and owner from git remote or environment." >&2
        return 0
    fi

    provider="${provider:-forgejo}"
    [ "$provider" = "codeberg" ] && provider="forgejo"
    case "$provider" in
        forgeapp_github|*github.com*) provider="github" ;;
        forgeapp_forgejo|*codeberg.org*|*tildegit.org*|*gitea*) provider="forgejo" ;;
        forgeapp_gitlab|*gitlab.com*|*framagit.org*) provider="gitlab" ;;
        forgeapp_sourcehut|*sr.ht*) provider="sourcehut" ;;
    esac

    if [ -z "$host" ]; then
        case "$provider" in
            github) host="github.com" ;;
            forgejo)
                if [ -n "${FORGEJO_HOST:-}" ]; then
                    host="$FORGEJO_HOST"
                elif [ -n "${CODEBERG_OWNER:-}" ] || [ -n "${CODEBERG_REPO:-}" ] || [ -n "${CODEBERG_TOKEN:-}" ]; then
                    host="codeberg.org"
                else
                    echo "Warning: Forgejo instance host not specified. Please provide --host or configure git remote." >&2
                fi
                ;;
            gitlab) host="framagit.org" ;;
            sourcehut) host="git.sr.ht" ;;
            *)
                echo "Warning: Unsupported provider '${provider}'." >&2
                return 0
                ;;
        esac
    fi

    # Check rate limit (55 requests/hour)
    if ! check_tickets_rate_limit; then
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

    fetch_status=0
    case "$provider" in
        github)
            echo "Fetching open issues from GitHub (${owner}/${repo})..."
            _cur_token="${cli_token:-}"
            [ -z "$_cur_token" ] && command -v resolve_forge_token >/dev/null 2>&1 && _cur_token=$(resolve_forge_token "github" 2>/dev/null || true)
            issues_json=$(_tickets_fetch_github "$host" "$owner" "$repo" "${_cur_token:-${GITHUB_TOKEN:-}}") || fetch_status=$?
            ;;
        forgejo)
            if [ "$host" = "codeberg.org" ]; then
                echo "Fetching open issues from Codeberg (${owner}/${repo})..."
            else
                echo "Fetching open issues from Forgejo/Gitea (${owner}/${repo} on ${host})..."
            fi
            _cur_token="${cli_token:-}"
            if [ -z "$_cur_token" ] && command -v resolve_forge_token >/dev/null 2>&1; then
                _cur_token=$(resolve_forge_token "forgejo" 2>/dev/null || true)
                if [ -z "$_cur_token" ] && [ "$host" = "codeberg.org" ]; then
                    _cur_token=$(resolve_forge_token "codeberg" 2>/dev/null || true)
                fi
            fi
            issues_json=$(_tickets_fetch_forgejo "$host" "$owner" "$repo" "${_cur_token:-${FORGEJO_TOKEN:-${CODEBERG_TOKEN:-}}}") || fetch_status=$?
            ;;
        gitlab)
            if [ "$host" = "framagit.org" ]; then
                echo "Fetching open issues from Framagit (${owner}/${repo})..."
            else
                echo "Fetching open issues from GitLab (${owner}/${repo} on ${host})..."
            fi
            _cur_token="${cli_token:-}"
            [ -z "$_cur_token" ] && command -v resolve_forge_token >/dev/null 2>&1 && _cur_token=$(resolve_forge_token "gitlab" 2>/dev/null || true)
            issues_json=$(_tickets_fetch_gitlab "$host" "$owner" "$repo" "${_cur_token:-${GITLAB_TOKEN:-}}") || fetch_status=$?
            ;;
        sourcehut)
            owner_clean=$(echo "$owner" | sed 's/^~//')
            echo "Fetching open issues from SourceHut (~${owner_clean}/${repo})..."
            _cur_token="${cli_token:-}"
            [ -z "$_cur_token" ] && command -v resolve_forge_token >/dev/null 2>&1 && _cur_token=$(resolve_forge_token "sourcehut" 2>/dev/null || true)
            issues_json=$(_tickets_fetch_sourcehut "$host" "$owner" "$repo" "${_cur_token:-${SOURCEHUT_TOKEN:-}}") || fetch_status=$?
            ;;
        *)
            echo "Warning: Unsupported provider '${provider}'." >&2
            return 0
            ;;
    esac

    if [ "$fetch_status" -ne 0 ]; then
        case "$provider" in
            forgejo)
                api_name="Codeberg API"
                [ "$host" != "codeberg.org" ] && api_name="Forgejo API"
                ;;
            github) api_name="GitHub API" ;;
            gitlab) api_name="GitLab API" ;;
            sourcehut) api_name="SourceHut API" ;;
            *) api_name="${provider} API" ;;
        esac
        echo "Warning: Failed to retrieve or parse issues from ${api_name} (${owner}/${repo})." >&2
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

    printf '%s\n' "$issues_json" > "$issues_tmp"

    jq_ticket_filter='
        "## [#" + (.number|tostring) + "] " + .title + "\n" +
        "- **Status**: `" + .state + "`\n" +
        "- **Labels**: " + (if (.labels | length) > 0 then (.labels | join(", ")) else "none" end) + "\n" +
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
_agents_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

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
            if command -v _cccp_http >/dev/null 2>&1 || command -v sv >/dev/null 2>&1 || command -v svyaz >/dev/null 2>&1 || command -v curl >/dev/null 2>&1; then
                content=$(_agents_http -s -f "$url" 2>/dev/null || true)
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
                echo "Warning: Neither 'sv' nor 'curl' is available; cannot fetch remote template." >&2
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
    if command -v ensure_gitignored >/dev/null 2>&1; then
        ensure_gitignored "$@"
        return $?
    fi
    return 0
}

_posix_replace_all() {
    _haystack="$1"
    _needle="$2"
    _repl="$3"
    _res=""

    [ -z "$_needle" ] && { printf '%s' "$_haystack"; return 0; }

    while :; do
        case "$_haystack" in
            *"$_needle"*)
                _before="${_haystack%%"$_needle"*}"
                _after="${_haystack#*"$_needle"}"
                _res="${_res}${_before}${_repl}"
                _haystack="$_after"
                ;;
            *)
                _res="${_res}${_haystack}"
                break
                ;;
        esac
    done
    printf '%s' "$_res"
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
        clean_val=""
        _first=1
        while IFS= read -r _line || [ -n "$_line" ]; do
            if [ "$_first" -eq 1 ]; then
                clean_val="$_line"
                _first=0
            else
                clean_val="${clean_val}\\n${_line}"
            fi
        done <<EOF
$var_val
EOF

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
            m_tag="{{${v}}}"
            s_tag="\${${v}}"
            result=$(_posix_replace_all "$result" "$m_tag" "$val")
            result=$(_posix_replace_all "$result" "$s_tag" "$val")
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
# Catalog Functions
# =============================================================================
_catalog_get_tea_token() {
    if command -v get_tea_token >/dev/null 2>&1; then
        get_tea_token "$@"
        return $?
    fi
    return 1
}

_catalog_resolve_token() {
    if command -v resolve_forge_token >/dev/null 2>&1; then
        resolve_forge_token "$@"
        return $?
    fi
    return 1
}

_catalog_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

_catalog_detect_forge_and_slug() {
    _url="$1"
    if [ -z "$_url" ]; then
        printf 'local unknown false\n'
        return 0
    fi

    if ! _parsed=$(parse_remote_url "$_url" 2>/dev/null); then
        printf 'local unknown false\n'
        return 0
    fi

    _chost=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
    _cowner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
    _crepo=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)
    _cslug="${_cowner}/${_crepo}"
    printf '%s %s false\n' "$_chost" "$_cslug"
}

_catalog_extract_description() {
    _dir="$1"
    _desc=""

    # 1. Check existing .git/description (if not default placeholder)
    if [ -f "$_dir/.git/description" ]; then
        _raw=$(head -n 1 "$_dir/.git/description" | tr -d '\r\n')
        case "$_raw" in
            *"Unnamed repository; edit this file 'description'"*|"")
                ;;
            *)
                printf '%s\n' "$_raw"
                return 0
                ;;
        esac
    fi

    # 2. Check Cargo.toml
    if [ -f "$_dir/Cargo.toml" ]; then
        _val=$(grep '^[[:space:]]*description[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*description[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 3. Check package.json
    if [ -f "$_dir/package.json" ]; then
        _val=$(grep '^[[:space:]]*"description"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*"description"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 4. Check composer.json
    if [ -f "$_dir/composer.json" ]; then
        _val=$(grep '^[[:space:]]*"description"[[:space:]]*:' "$_dir/composer.json" 2>/dev/null | head -n 1 || true)
        if [ -n "$_val" ]; then
            _desc=$(printf '%s' "$_val" | sed -e 's/^[[:space:]]*"description"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
            if [ -n "$_desc" ]; then
                printf '%s\n' "$_desc"
                return 0
            fi
        fi
    fi

    # 5. Check README.md
    if [ -f "$_dir/README.md" ]; then
        while IFS= read -r _rline || [ -n "$_rline" ]; do
            case "$_rline" in
                [#\<]*|"") continue ;;
                *)
                    _desc=$(printf '%s' "$_rline" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                    if [ -n "$_desc" ]; then
                        printf '%s\n' "$_desc"
                        return 0
                    fi
                    ;;
            esac
        done < "$_dir/README.md"
    fi

    printf ''
}

_catalog_extract_license() {
    _dir="$1"

    # 1. Cargo.toml
    if [ -f "$_dir/Cargo.toml" ]; then
        _lic=$(grep '^[[:space:]]*license[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*license[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 2. package.json
    if [ -f "$_dir/package.json" ]; then
        _lic=$(grep '^[[:space:]]*"license"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"license"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 3. composer.json
    if [ -f "$_dir/composer.json" ]; then
        _lic=$(grep '^[[:space:]]*"license"[[:space:]]*:' "$_dir/composer.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"license"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_lic" ] && { printf '%s\n' "$_lic"; return 0; }
    fi

    # 4. LICENSE file inspection
    for _lic_file in "$_dir/LICENSE" "$_dir/LICENSE.md" "$_dir/LICENSE.txt" "$_dir/COPYING"; do
        if [ -f "$_lic_file" ]; then
            if grep -qi "Apache License" "$_lic_file"; then
                printf 'Apache-2.0\n'; return 0
            elif grep -qi "MIT License" "$_lic_file" || grep -q "MIT" "$_lic_file"; then
                printf 'MIT\n'; return 0
            elif grep -qi "Affero General Public License" "$_lic_file" || grep -q "AGPL" "$_lic_file"; then
                printf 'AGPL-3.0\n'; return 0
            elif grep -qi "General Public License" "$_lic_file" || grep -q "GPL" "$_lic_file"; then
                printf 'GPL-3.0\n'; return 0
            elif grep -qi "Creative Commons Zero" "$_lic_file" || grep -q "CC0" "$_lic_file"; then
                printf 'CC0-1.0\n'; return 0
            elif grep -qi "Unlicense" "$_lic_file"; then
                printf 'Unlicense\n'; return 0
            elif grep -qi "Creative Commons Attribution-ShareAlike" "$_lic_file"; then
                printf 'CC-BY-SA-4.0\n'; return 0
            fi
        fi
    done

    printf 'MIT\n'
}

_catalog_extract_website() {
    _dir="$1"
    # Check package.json homepage
    if [ -f "$_dir/package.json" ]; then
        _hp=$(grep '^[[:space:]]*"homepage"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*"homepage"[[:space:]]*:[[:space:]]*"//' -e 's/",*[[:space:]]*$//' || true)
        [ -n "$_hp" ] && { printf '%s\n' "$_hp"; return 0; }
    fi
    # Check Cargo.toml homepage
    if [ -f "$_dir/Cargo.toml" ]; then
        _hp=$(grep '^[[:space:]]*homepage[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | head -n 1 | sed -e 's/^[[:space:]]*homepage[[:space:]]*=[[:space:]]*"//' -e 's/"[[:space:]]*$//' || true)
        [ -n "$_hp" ] && { printf '%s\n' "$_hp"; return 0; }
    fi
    printf ''
}

_catalog_extract_topics() {
    _dir="$1"
    # package.json keywords
    if [ -f "$_dir/package.json" ]; then
        _kw=$(grep -A 10 '^[[:space:]]*"keywords"[[:space:]]*:' "$_dir/package.json" 2>/dev/null | grep '"' | grep -v 'keywords' | sed -e 's/^[[:space:]]*"//' -e 's/",*[[:space:]]*$//' | tr '\n' ' ' || true)
        [ -n "$_kw" ] && { printf '%s\n' "$_kw"; return 0; }
    fi
    # Cargo.toml keywords
    if [ -f "$_dir/Cargo.toml" ]; then
        _kw=$(grep '^[[:space:]]*keywords[[:space:]]*=' "$_dir/Cargo.toml" 2>/dev/null | sed -e 's/^[[:space:]]*keywords[[:space:]]*=[[:space:]]*\[//' -e 's/\][[:space:]]*$//' -e 's/"//g' -e 's/,/ /g' || true)
        [ -n "$_kw" ] && { printf '%s\n' "$_kw"; return 0; }
    fi
    printf ''
}

_catalog_read_existing_json() {
    _json_file="$1"
    [ -f "$_json_file" ] || return 1

    _in_obj=0; _in_topics=0
    _cur_name=""; _cur_forge=""; _cur_slug=""; _cur_desc=""
    _cur_website=""; _cur_upstream="false"; _cur_lic=""; _cur_topics=""

    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            *"{"*)
                _in_obj=1; _in_topics=0
                _cur_name=""; _cur_forge=""; _cur_slug=""; _cur_desc=""
                _cur_website=""; _cur_upstream="false"; _cur_lic=""; _cur_topics=""
                ;;
            *"}"*)
                if [ "$_in_obj" -eq 1 ] && [ -n "$_cur_name" ]; then
                    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
                        "$_cur_name" "$_cur_forge" "$_cur_slug" "$_cur_lic" \
                        "$_cur_upstream" "$_cur_website" "$_cur_topics" "$_cur_desc"
                fi
                _in_obj=0; _in_topics=0
                ;;
            *\"name\":*)
                _cur_name=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"name":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"forge\":*)
                _cur_forge=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"forge":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"slug\":*)
                _cur_slug=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"slug":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"description\":*)
                _cur_desc=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"description":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"license\":*)
                _cur_lic=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"license":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"website\":*)
                _cur_website=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"website":[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                ;;
            *\"is_upstream\":*)
                case "$_line" in
                    *true*) _cur_upstream="true" ;;
                    *) _cur_upstream="false" ;;
                esac
                ;;
            *\"topics\":*)
                _in_topics=1
                ;;
            *\]*)
                _in_topics=0
                ;;
            *)
                if [ "$_in_topics" -eq 1 ]; then
                    _t=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*"//' -e 's/",*[[:space:]]*$//')
                    if [ -n "$_t" ]; then
                        _cur_topics="${_cur_topics}${_cur_topics:+, }$_t"
                    fi
                fi
                ;;
        esac
    done < "$_json_file"
}

_catalog_sync_local() {
    _src_dir="$1"
    if [ ! -d "$_src_dir" ]; then
        echo "Error: Directory '$_src_dir' does not exist." >&2
        return 1
    fi

    echo "-> [ГОСПЛАН] Sincronizando .git/description em cada repositório de $_src_dir..."
    _cat_json="${_src_dir}/catalog.json"
    _cat_readme="${_src_dir}/README.md"
    _existing_records=""
    if [ -f "$_cat_json" ]; then
        _existing_records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
    fi

    _tmp_json="${_cat_json}.tmp.$$"
    _tmp_md="${_cat_readme}.tmp.$$"

    printf '[\n' > "$_tmp_json"
    printf '# Catálogo de Projetos (`%s`)\n\n' "$_src_dir" > "$_tmp_md"
    printf 'Índice estruturado de todos os repositórios locais e suas publicações remotas.\n\n' >> "$_tmp_md"
    printf '| Projeto | Forge | Licença | Descrição | Tags |\n' >> "$_tmp_md"
    printf '| :--- | :---: | :---: | :--- | :--- |\n' >> "$_tmp_md"

    _first_repo=1
    for _repo_entry in "$_src_dir"/*; do
        [ -d "$_repo_entry" ] || continue
        [ -d "$_repo_entry/.git" ] || [ -f "$_repo_entry/.git" ] || continue

        _rname=$(basename "$_repo_entry")
        _rforge="unknown"
        _rslug="$_rname"
        _rupstream="false"
        _rdesc=""
        _rlicense=""
        _rwebsite=""
        _rtopics=""

        # Check existing metadata from previous catalog.json
        if [ -n "$_existing_records" ]; then
            _rec=$(printf '%s\n' "$_existing_records" | grep "^${_rname}[[:space:]]" | head -n 1 || true)
            if [ -n "$_rec" ]; then
                _rforge=$(printf '%s' "$_rec" | cut -f2)
                _rslug=$(printf '%s' "$_rec" | cut -f3)
                _rlicense=$(printf '%s' "$_rec" | cut -f4)
                _rupstream=$(printf '%s' "$_rec" | cut -f5)
                _rwebsite=$(printf '%s' "$_rec" | cut -f6)
                _rtopics=$(printf '%s' "$_rec" | cut -f7)
                _rdesc=$(printf '%s' "$_rec" | cut -f8-)
            fi
        fi

        # Detect forge and slug from git remote if unconfigured or generic
        _origin_url=$(git -C "$_repo_entry" config remote.origin.url 2>/dev/null || true)
        if [ -n "$_origin_url" ]; then
            _detected=$(_catalog_detect_forge_and_slug "$_origin_url")
            _det_forge=$(printf '%s' "$_detected" | cut -d' ' -f1)
            _det_slug=$(printf '%s' "$_detected" | cut -d' ' -f2)
            if [ "$_rforge" = "unknown" ] || [ -z "$_rforge" ]; then
                _rforge="$_det_forge"
            fi
            if [ "$_rslug" = "$_rname" ] || [ -z "$_rslug" ]; then
                _rslug="$_det_slug"
            fi
        fi

        # Extract description from git/files if empty
        if [ -z "$_rdesc" ]; then
            _rdesc=$(_catalog_extract_description "$_repo_entry")
        fi
        [ -z "$_rdesc" ] && _rdesc="Repositório Git ${_rname}"

        # Write description to .git/description if writable
        if [ -d "$_repo_entry/.git" ]; then
            printf '%s\n' "$_rdesc" > "$_repo_entry/.git/description" 2>/dev/null || true
            echo "  [OK] ${_rname}/.git/description"
        else
            echo "  [OK] ${_rname} (atualizado)"
        fi

        # Extract license if empty
        if [ -z "$_rlicense" ]; then
            _rlicense=$(_catalog_extract_license "$_repo_entry")
        fi

        # Extract website if empty
        if [ -z "$_rwebsite" ]; then
            _rwebsite=$(_catalog_extract_website "$_repo_entry")
        fi

        # Extract topics if empty
        if [ -z "$_rtopics" ]; then
            _rtopics=$(_catalog_extract_topics "$_repo_entry")
        fi

        # Format JSON topics array
        _topics_json=""
        if [ -n "$_rtopics" ]; then
            _tfirst=1
            # Replace comma separators with spaces
            _clean_topics=$(printf '%s' "$_rtopics" | tr ',' ' ')
            for _top in $_clean_topics; do
                [ -n "$_top" ] || continue
                [ "$_tfirst" -eq 1 ] && _tfirst=0 || _topics_json="${_topics_json}, "
                _topics_json="${_topics_json}\"${_top}\""
            done
        fi

        # Escape quotes in description for JSON
        _desc_json=$(printf '%s' "$_rdesc" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')

        # Append to catalog.json
        if [ "$_first_repo" -eq 1 ]; then
            _first_repo=0
        else
            printf ',\n' >> "$_tmp_json"
        fi

        cat >> "$_tmp_json" <<EOF
  {
    "name": "${_rname}",
    "forge": "${_rforge}",
    "slug": "${_rslug}",
    "description": "${_desc_json}",
    "topics": [${_topics_json}],
    "license": "${_rlicense}",
    "website": "${_rwebsite}",
    "is_upstream": ${_rupstream}
  }
EOF

        # Append to README.md
        _forge_icon="$_rforge"
        case "$_rforge" in
            *github.com*|github)   _forge_icon="🐙 GitHub" ;;
            *codeberg.org*|codeberg) _forge_icon="🏔️ Codeberg" ;;
            *tildegit.org*|tildegit) _forge_icon="🟣 TildeGit" ;;
            *gitlab.com*|*framagit.org*|gitlab)   _forge_icon="🦊 GitLab" ;;
            *sr.ht*|sourcehut) _forge_icon="🛖 SourceHut" ;;
        esac

        _bt=$(printf '\140')
        _tags_md=""
        _tag_count=0
        for _top in $(printf '%s' "$_rtopics" | tr ',' ' '); do
            [ -n "$_top" ] || continue
            _tags_md="${_tags_md} ${_bt}${_top}${_bt}"
            _tag_count=$((_tag_count + 1))
            [ "$_tag_count" -ge 4 ] && break
        done

        printf '| **[%s](./%s)** | %s | %s%s%s | %s |%s |\n' \
            "$_rname" "$_rname" "$_forge_icon" "$_bt" "$_rlicense" "$_bt" "$_rdesc" "$_tags_md" >> "$_tmp_md"
    done

    printf '\n]\n' >> "$_tmp_json"

    mv "$_tmp_json" "$_cat_json"
    mv "$_tmp_md" "$_cat_readme"

    echo "  [OK] Gravado $_cat_json"
    echo "  [OK] Gravado $_cat_readme"
    return 0
}

_catalog_sync_remote() {
    _src_dir="$1"
    _cat_json="${_src_dir}/catalog.json"
    if [ ! -f "$_cat_json" ]; then
        echo "Error: Catálogo '$_cat_json' não encontrado. Execute 'cccp catalog sync-local' primeiro." >&2
        return 1
    fi

    echo ""
    echo "-> [ГОСПЛАН] Sincronizando metadados com os servidores remotos (Codeberg, TildeGit, GitHub)..."

    _records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
    if [ -z "$_records" ]; then
        echo "Aviso: Nenhum repositório encontrado em $_cat_json."
        return 0
    fi

    echo "$_records" | while IFS= read -r _entry || [ -n "$_entry" ]; do
        [ -n "$_entry" ] || continue
        _name=$(printf '%s' "$_entry" | cut -f1)
        _forge=$(printf '%s' "$_entry" | cut -f2)
        _slug=$(printf '%s' "$_entry" | cut -f3)
        _license=$(printf '%s' "$_entry" | cut -f4)
        _is_upstream=$(printf '%s' "$_entry" | cut -f5)
        _website=$(printf '%s' "$_entry" | cut -f6)
        _topics=$(printf '%s' "$_entry" | cut -f7)
        _desc=$(printf '%s' "$_entry" | cut -f8-)

        if [ "$_is_upstream" = "true" ]; then
            echo "  [INFO] ${_name} (${_forge}): Repositório upstream de terceiros (${_slug}). Pulando metadados remotos."
            continue
        fi

        _forgeapp=""
        if command -v detect_forgeapp >/dev/null 2>&1; then
            _forgeapp=$(detect_forgeapp "$_forge" 2>/dev/null || true)
        fi

        case "$_forgeapp" in
            forgeapp_forgejo)
                _domain="$_forge"
                case "$_domain" in
                    codeberg) _domain="codeberg.org" ;;
                    tildegit) _domain="tildegit.org" ;;
                    forgejo) _domain="${FORGEJO_HOST:-codeberg.org}" ;;
                esac
                _base_url="https://${_domain}"

                _token=$(_catalog_resolve_token "$_domain" 2>/dev/null || true)
                [ -z "$_token" ] && _token=$(_catalog_resolve_token "$_forge" 2>/dev/null || true)
                if [ -z "$_token" ]; then
                    echo "  [AVISO] ${_name} (${_forge}): Token de autenticação não configurado para ${_forge}. Pulando."
                    continue
                fi

                # 1. Update repo description & website
                _esc_desc=$(escape_json_string "$_desc")
                _esc_web=$(escape_json_string "$_website")
                _payload="{\"description\":\"${_esc_desc}\""
                if [ -n "$_esc_web" ]; then
                    _payload="${_payload},\"website\":\"${_esc_web}\""
                fi
                _payload="${_payload}}"

                _res=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PATCH \
                    -H "Authorization: token ${_token}" \
                    -H "Content-Type: application/json" \
                    -H "Accept: application/json" \
                    -H "User-Agent: CCCP-Catalog/1.0" \
                    -d "$_payload" \
                    "${_base_url}/api/v1/repos/${_slug}" 2>/dev/null || echo "000")

                # 2. Update topics
                _topics_arr=""
                _tfirst=1
                for _t in $(printf '%s' "$_topics" | tr ',' ' '); do
                    [ -n "$_t" ] || continue
                    [ "$_tfirst" -eq 1 ] && _tfirst=0 || _topics_arr="${_topics_arr},"
                    _topics_arr="${_topics_arr}\"${_t}\""
                done
                _top_payload="{\"topics\":[${_topics_arr}]}"

                _res_top=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PUT \
                    -H "Authorization: token ${_token}" \
                    -H "Content-Type: application/json" \
                    -H "Accept: application/json" \
                    -H "User-Agent: CCCP-Catalog/1.0" \
                    -d "$_top_payload" \
                    "${_base_url}/api/v1/repos/${_slug}/topics" 2>/dev/null || echo "000")

                case "$_res" in
                    200|201|204)
                        echo "  [OK] ${_forge}: ${_slug} (descrição e tópicos sincronizados)"
                        ;;
                    *)
                        echo "  [ERRO] ${_forge} ${_slug}: HTTP ${_res}"
                        ;;
                esac
                ;;

            forgeapp_github|github)
                if command -v gh >/dev/null 2>&1; then
                    _gh_args="repo edit ${_slug} --description \"${_desc}\""
                    if [ -n "$_website" ]; then
                        _gh_args="${_gh_args} --homepage \"${_website}\""
                    fi
                    for _t in $(printf '%s' "$_topics" | tr ',' ' '); do
                        [ -n "$_t" ] || continue
                        _gh_args="${_gh_args} --add-topic \"${_t}\""
                    done

                    if eval "gh ${_gh_args} >/dev/null 2>&1"; then
                        echo "  [OK] GitHub: ${_slug} (descrição e tópicos via gh)"
                    else
                        echo "  [ERRO] GitHub: ${_slug} (falha ao executar gh repo edit)"
                    fi
                else
                    _token=$(_catalog_resolve_token "github.com" 2>/dev/null || true)
                    [ -z "$_token" ] && _token=$(_catalog_resolve_token "$_forge" 2>/dev/null || true)
                    if [ -z "$_token" ]; then
                        echo "  [AVISO] ${_name} (github): nem 'gh' nem GITHUB_TOKEN configurados. Pulando."
                        continue
                    fi

                    _esc_desc=$(escape_json_string "$_desc")
                    _esc_web=$(escape_json_string "$_website")
                    _payload="{\"description\":\"${_esc_desc}\""
                    [ -n "$_esc_web" ] && _payload="${_payload},\"homepage\":\"${_esc_web}\""
                    _payload="${_payload}}"

                    _res=$(_catalog_http -s -o /dev/null -w "%{http_code}" -X PATCH \
                        -H "Authorization: token ${_token}" \
                        -H "Accept: application/vnd.github+json" \
                        -H "User-Agent: CCCP-Catalog/1.0" \
                        -d "$_payload" \
                        "https://api.github.com/repos/${_slug}" 2>/dev/null || echo "000")

                    case "$_res" in
                        200|201|204)
                            echo "  [OK] GitHub: ${_slug} (descrição atualizada via REST)"
                            ;;
                        *)
                            echo "  [ERRO] GitHub ${_slug}: HTTP ${_res}"
                            ;;
                    esac
                fi
                ;;

            *)
                echo "  [INFO] ${_name} (${_forge}): sincronização remota não suportada para ${_forge}."
                ;;
        esac
    done
    return 0
}

_catalog_list() {
    _src_dir="$1"
    _as_json="${2:-0}"
    _cat_json="${_src_dir}/catalog.json"

    if [ "$_as_json" -eq 1 ]; then
        if [ -f "$_cat_json" ]; then
            cat "$_cat_json"
            return 0
        fi
    fi

    if [ -f "$_cat_json" ]; then
        _records=$(_catalog_read_existing_json "$_cat_json" 2>/dev/null || true)
        if [ -z "$_records" ]; then
            echo "Catálogo vazio em $_cat_json"
            return 0
        fi

        printf '%-20s %-10s %-30s %-12s %s\n' "PROJETO" "FORGE" "SLUG" "LICENÇA" "DESCRIÇÃO"
        printf '%-20s %-10s %-30s %-12s %s\n' "-------" "-----" "----" "-------" "---------"
        echo "$_records" | while IFS= read -r _line || [ -n "$_line" ]; do
            [ -n "$_line" ] || continue
            _p_name=$(printf '%s' "$_line" | cut -f1)
            _p_forge=$(printf '%s' "$_line" | cut -f2)
            _p_slug=$(printf '%s' "$_line" | cut -f3)
            _p_lic=$(printf '%s' "$_line" | cut -f4)
            _p_desc=$(printf '%s' "$_line" | cut -f8-)
            printf '%-20s %-10s %-30s %-12s %s\n' "$_p_name" "$_p_forge" "$_p_slug" "$_p_lic" "$_p_desc"
        done
        return 0
    fi

    # Fallback: scan on the fly
    echo "Nenhum catalog.json encontrado. Varrendo $_src_dir..."
    _catalog_sync_local "$_src_dir"
}

_catalog_resolve_target_dir() {
    _cli_dir="${1:-}"
    _resolved=""

    # 1. Command-line flag
    if [ -n "$_cli_dir" ]; then
        _resolved="$_cli_dir"
    # 2. Environment variable
    elif [ -n "${CATALOG_DIR:-}" ]; then
        _resolved="$CATALOG_DIR"
    elif [ -n "${CCCP_CATALOG_DIR:-}" ]; then
        _resolved="$CCCP_CATALOG_DIR"
    # 3. Global CCCP configuration
    else
        _cfg_dir=""
        _local_cfg=""
        if command -v get_local_config_file >/dev/null 2>&1; then
            _local_cfg=$(get_local_config_file 2>/dev/null || true)
        fi
        if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _cfg_dir=$(read_file_key "$_local_cfg" "catalog_dir" 2>/dev/null || true)
        fi
        if [ -z "$_cfg_dir" ]; then
            _global_cfg=""
            if command -v get_global_config_file >/dev/null 2>&1; then
                _global_cfg=$(get_global_config_file 2>/dev/null || true)
            else
                _global_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
            fi
            if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
                _cfg_dir=$(read_file_key "$_global_cfg" "catalog_dir" 2>/dev/null || true)
            fi
        fi

        if [ -n "$_cfg_dir" ]; then
            _resolved="$_cfg_dir"
        else
            # 4. Fallback: ~/src
            _resolved="~/src"
        fi
    fi

    # Expand leading tilde (~) if present
    case "$_resolved" in
        "~") _resolved="$HOME" ;;
        "~/"*) _resolved="${HOME}/${_resolved#\~/}" ;;
    esac

    printf '%s\n' "$_resolved"
}

cmd_catalog() {
    _action="list"
    _cli_dir=""
    _as_json=0

    while [ $# -gt 0 ]; do
        case "$1" in
            list)
                _action="list"
                shift
                ;;
            local|sync-local)
                _action="sync-local"
                shift
                ;;
            remote|sync-remote)
                _action="sync-remote"
                shift
                ;;
            all)
                _action="all"
                shift
                ;;
            --json)
                _as_json=1
                shift
                ;;
            --dir|-d)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing directory argument for $1" >&2; return 1; }
                _cli_dir="$1"
                shift
                ;;
            -h|--help|help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "catalog"
                else
                    echo "Usage: cccp catalog [list|sync-local|sync-remote|all] [--dir <path>] [--json]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unknown catalog subcommand or option '$1'" >&2
                return 1
                ;;
        esac
    done

    _target_dir=$(_catalog_resolve_target_dir "$_cli_dir")

    case "$_action" in
        list)
            _catalog_list "$_target_dir" "$_as_json"
            ;;
        sync-local)
            _catalog_sync_local "$_target_dir"
            ;;
        sync-remote)
            _catalog_sync_remote "$_target_dir"
            ;;
        all)
            _catalog_sync_local "$_target_dir"
            _catalog_sync_remote "$_target_dir"
            ;;
    esac
}


# =============================================================================
# Diva Functions
# =============================================================================
_diva_resolve_api_key() {
    # 1. Environment / In-memory variable
    if [ -n "${GEMINI_API_KEY:-}" ]; then
        printf '%s\n' "$GEMINI_API_KEY"
        return 0
    fi

    # 2. Check ~/.env (or local .env)
    for _env_file in "${HOME}/.env" ".env"; do
        if [ -f "$_env_file" ] && command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_env_file" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    done

    # 3. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_local_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    # 4. Check global config (~/.config/cccp/config)
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    else
        _global_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
    fi

    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ]; then
        if command -v read_file_key >/dev/null 2>&1; then
            if _k=$(read_file_key "$_global_cfg" "gemini_api_key" 2>/dev/null) && [ -n "$_k" ]; then
                printf '%s\n' "$_k"
                return 0
            fi
        fi
    fi

    return 1
}

_diva_resolve_forge() {
    if command -v detect_repo_info >/dev/null 2>&1; then
        detect_repo_info
    else
        return 1
    fi
}

_diva_resolve_forge_token() {
    _forge="$1"
    if command -v resolve_forge_token >/dev/null 2>&1; then
        resolve_forge_token "$_forge" && return 0
    fi
    if command -v _catalog_resolve_token >/dev/null 2>&1; then
        _catalog_resolve_token "$_forge" && return 0
    fi
    return 1
}

_diva_http() {
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    case "$(type sv 2>/dev/null)" in
        *function*)
            sv "$@"
            return $?
            ;;
    esac
    case "$(type curl 2>/dev/null)" in
        *function*)
            curl "$@"
            return $?
            ;;
    esac

    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
    elif command -v sv >/dev/null 2>&1; then
        sv "$@"
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
    elif command -v curl >/dev/null 2>&1; then
        curl "$@"
    else
        return 127
    fi
}

_diva_resolve_editor() {
    [ -n "${DIVA_EDITOR:-}" ] && { printf '%s\n' "$DIVA_EDITOR"; return 0; }

    # 1. Check local .cccprc
    if command -v get_local_config_file >/dev/null 2>&1; then
        _lcfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$_lcfg" ] && [ -f "$_lcfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _e=$(read_file_key "$_lcfg" "diva_editor" 2>/dev/null || true)
            [ -n "$_e" ] && { printf '%s\n' "$_e"; return 0; }
        fi
    fi

    # 2. Check global .cccprc
    if command -v get_global_config_file >/dev/null 2>&1; then
        _gcfg=$(get_global_config_file 2>/dev/null || true)
        if [ -n "$_gcfg" ] && [ -f "$_gcfg" ] && command -v read_file_key >/dev/null 2>&1; then
            _e=$(read_file_key "$_gcfg" "diva_editor" 2>/dev/null || true)
            [ -n "$_e" ] && { printf '%s\n' "$_e"; return 0; }
        fi
    fi

    # 3. Check VISUAL / EDITOR
    if [ -n "${VISUAL:-}" ]; then
        printf '%s\n' "$VISUAL"
        return 0
    fi
    if [ -n "${EDITOR:-}" ]; then
        printf '%s\n' "$EDITOR"
        return 0
    fi

    # 4. Probe dedicated micro-editor with full keyboard navigation: kv / kvant
    if command -v kv >/dev/null 2>&1; then
        printf 'kv\n'
        return 0
    fi
    if command -v kvant >/dev/null 2>&1; then
        printf 'kvant\n'
        return 0
    fi
    if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
        if _cccp_ensure_comrade "kv" 0 >&2; then
            if command -v kv >/dev/null 2>&1; then
                printf 'kv\n'
                return 0
            fi
        fi
    fi

    # 5. Fallback to standard POSIX editor
    if command -v vi >/dev/null 2>&1; then
        printf 'vi\n'
        return 0
    fi

    return 1
}

_diva_resolve_custom_repo() {
    _input_repo="$1"
    [ -n "$_input_repo" ] || return 1

    # 1. If already in owner/repo format, extract and return
    case "$_input_repo" in
        */*)
            _r_owner=$(printf '%s' "$_input_repo" | cut -d'/' -f1)
            _r_repo=$(printf '%s' "$_input_repo" | cut -d'/' -f2)
            _r_domain=""
            _r_forgeapp=""
            _found_locally=0

            # Check if matching entry exists in catalog to infer domain
            _cat_files=""
            _d_root=""
            if command -v get_git_root >/dev/null 2>&1; then
                _d_root=$(get_git_root 2>/dev/null || true)
            fi
            [ -z "$_d_root" ] && _d_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
            [ -n "$_d_root" ] && _cat_files="${_d_root}/catalog.json $(dirname "$_d_root")/catalog.json"
            _cat_files="${_cat_files} catalog.json ${HOME}/src/catalog.json ${XDG_DATA_HOME:-${HOME}/.local/share}/cccp/catalog.json ${HOME}/.config/cccp/catalog.json"

            for _cf in $_cat_files; do
                if [ -f "$_cf" ] && command -v _catalog_read_existing_json >/dev/null 2>&1; then
                    while IFS="$(printf '\t')" read -r _cn _cf_forge _c_slug _c_lic _c_up _c_web _c_top _c_desc || [ -n "$_cn" ]; do
                        if [ "$_c_slug" = "$_input_repo" ]; then
                            _r_domain="$_cf_forge"
                            _found_locally=1
                            break 2
                        fi
                    done <<EOF
$(_catalog_read_existing_json "$_cf" 2>/dev/null || true)
EOF
                fi
            done

            case "$_r_domain" in
                codeberg) _r_domain="codeberg.org" ;;
                tildegit) _r_domain="tildegit.org" ;;
                github) _r_domain="github.com" ;;
                gitlab) _r_domain="gitlab.com" ;;
            esac

            # Check sibling directory
            if [ -n "$_d_root" ]; then
                _parent=$(dirname "$_d_root")
                if [ -d "${_parent}/${_r_repo}/.git" ]; then
                    if _sib_info=$(detect_repo_info "${_parent}/${_r_repo}" 2>/dev/null); then
                        _s_dom=$(printf '%s' "$_sib_info" | cut -d' ' -f1)
                        _s_own=$(printf '%s' "$_sib_info" | cut -d' ' -f2)
                        if [ "$_s_own" = "$_r_owner" ]; then
                            [ -z "$_r_domain" ] && _r_domain="$_s_dom"
                            _found_locally=1
                        fi
                    fi
                fi
            fi

            # Check catalog base directory (e.g. ~/src)
            _cat_base=""
            if command -v _catalog_resolve_target_dir >/dev/null 2>&1; then
                _cat_base=$(_catalog_resolve_target_dir "" 2>/dev/null || echo "")
            fi
            [ -z "$_cat_base" ] && _cat_base="${HOME}/src"
            if [ -n "$_cat_base" ] && [ -d "${_cat_base}/${_r_repo}/.git" ]; then
                if _base_info=$(detect_repo_info "${_cat_base}/${_r_repo}" 2>/dev/null); then
                    _b_dom=$(printf '%s' "$_base_info" | cut -d' ' -f1)
                    _b_own=$(printf '%s' "$_base_info" | cut -d' ' -f2)
                    if [ "$_b_own" = "$_r_owner" ]; then
                        [ -z "$_r_domain" ] && _r_domain="$_b_dom"
                        _found_locally=1
                    fi
                fi
            fi

            # Check active repo
            if _cur_info=$(detect_repo_info 2>/dev/null); then
                _cur_dom=$(printf '%s' "$_cur_info" | cut -d' ' -f1)
                _cur_own=$(printf '%s' "$_cur_info" | cut -d' ' -f2)
                _cur_rep=$(printf '%s' "$_cur_info" | cut -d' ' -f3)
                if [ "$_cur_own" = "$_r_owner" ] && [ "$_cur_rep" = "$_r_repo" ]; then
                    [ -z "$_r_domain" ] && _r_domain="$_cur_dom"
                    _found_locally=1
                fi
            fi

            if [ "$_found_locally" -eq 0 ]; then
                printf 'Error: Repositório "%s" não encontrado localmente.\n' "$_input_repo" >&2
                return 1
            fi

            [ -z "$_r_domain" ] && _r_domain="codeberg.org"
            _r_forgeapp=$(detect_forgeapp "$_r_domain" 2>/dev/null || echo "forgeapp_forgejo")

            printf '%s %s %s %s\n' "$_r_domain" "$_r_owner" "$_r_repo" "$_r_forgeapp"
            return 0
            ;;
    esac

    # 2. Partial name resolution
    _candidates=""
    _candidate_keys=""

    # 2a. Search catalog.json
    _cat_files=""
    _d_root=""
    if command -v get_git_root >/dev/null 2>&1; then
        _d_root=$(get_git_root 2>/dev/null || true)
    fi
    [ -z "$_d_root" ] && _d_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    [ -n "$_d_root" ] && _cat_files="${_d_root}/catalog.json $(dirname "$_d_root")/catalog.json"
    _cat_files="${_cat_files} catalog.json ${HOME}/src/catalog.json ${XDG_DATA_HOME:-${HOME}/.local/share}/cccp/catalog.json ${HOME}/.config/cccp/catalog.json"

    for _cf in $_cat_files; do
        if [ -f "$_cf" ] && command -v _catalog_read_existing_json >/dev/null 2>&1; then
            while IFS="$(printf '\t')" read -r _cn _cf_forge _c_slug _c_lic _c_up _c_web _c_top _c_desc || [ -n "$_cn" ]; do
                [ -n "$_c_slug" ] || continue
                _c_r=$(printf '%s' "$_c_slug" | cut -d'/' -f2)
                if [ "$_cn" = "$_input_repo" ] || [ "$_c_r" = "$_input_repo" ]; then
                    _c_domain="$_cf_forge"
                    case "$_c_domain" in
                        github) _c_domain="github.com" ;;
                        codeberg) _c_domain="codeberg.org" ;;
                        tildegit) _c_domain="tildegit.org" ;;
                        gitlab) _c_domain="gitlab.com" ;;
                    esac
                    _c_forgeapp=$(detect_forgeapp "$_c_domain" 2>/dev/null || echo "forgeapp_forgejo")
                    _cand="${_c_slug}:${_c_domain}:${_c_forgeapp}"
                    case " ${_candidate_keys} " in
                        *" ${_c_slug}@${_c_domain} "*) ;;
                        *)
                            _candidate_keys="${_candidate_keys}${_candidate_keys:+ }${_c_slug}@${_c_domain}"
                            _candidates="${_candidates}${_candidates:+ }${_cand}"
                            ;;
                    esac
                fi
            done <<EOF
$(_catalog_read_existing_json "$_cf" 2>/dev/null || true)
EOF
        fi
    done

    # 2b. Search sibling directories
    if [ -n "$_d_root" ]; then
        _parent=$(dirname "$_d_root")
        if [ -d "${_parent}/${_input_repo}/.git" ]; then
            if _sib_info=$(detect_repo_info "${_parent}/${_input_repo}" 2>/dev/null); then
                _s_domain=$(printf '%s' "$_sib_info" | cut -d' ' -f1)
                _s_owner=$(printf '%s' "$_sib_info" | cut -d' ' -f2)
                _s_repo=$(printf '%s' "$_sib_info" | cut -d' ' -f3)
                _s_forgeapp=$(printf '%s' "$_sib_info" | cut -d' ' -f4)
                if [ -n "$_s_owner" ] && [ -n "$_s_repo" ]; then
                    _s_slug="${_s_owner}/${_s_repo}"
                    _cand="${_s_slug}:${_s_domain}:${_s_forgeapp}"
                    case " ${_candidate_keys} " in
                        *" ${_s_slug}@${_s_domain} "*) ;;
                        *)
                            _candidate_keys="${_candidate_keys}${_candidate_keys:+ }${_s_slug}@${_s_domain}"
                            _candidates="${_candidates}${_candidates:+ }${_cand}"
                            ;;
                    esac
                fi
            fi
        fi
    fi

    # 2c. Search catalog base directory (e.g. ~/src)
    _cat_base=""
    if command -v _catalog_resolve_target_dir >/dev/null 2>&1; then
        _cat_base=$(_catalog_resolve_target_dir "" 2>/dev/null || echo "")
    fi
    [ -z "$_cat_base" ] && _cat_base="${HOME}/src"
    if [ -n "$_cat_base" ] && [ -d "${_cat_base}/${_input_repo}/.git" ]; then
        if _base_info=$(detect_repo_info "${_cat_base}/${_input_repo}" 2>/dev/null); then
            _b_domain=$(printf '%s' "$_base_info" | cut -d' ' -f1)
            _b_owner=$(printf '%s' "$_base_info" | cut -d' ' -f2)
            _b_repo=$(printf '%s' "$_base_info" | cut -d' ' -f3)
            _b_forgeapp=$(printf '%s' "$_base_info" | cut -d' ' -f4)
            if [ -n "$_b_owner" ] && [ -n "$_b_repo" ]; then
                _b_slug="${_b_owner}/${_b_repo}"
                _cand="${_b_slug}:${_b_domain}:${_b_forgeapp}"
                case " ${_candidate_keys} " in
                    *" ${_b_slug}@${_b_domain} "*) ;;
                    *)
                        _candidate_keys="${_candidate_keys}${_candidate_keys:+ }${_b_slug}@${_b_domain}"
                        _candidates="${_candidates}${_candidates:+ }${_cand}"
                        ;;
                esac
            fi
        fi
    fi

    # 2d. Check current repo if it matches
    _cur_owner=""
    _cur_domain=""
    _cur_forgeapp=""
    if _cur_info=$(detect_repo_info 2>/dev/null); then
        _cur_domain=$(printf '%s' "$_cur_info" | cut -d' ' -f1)
        _cur_owner=$(printf '%s' "$_cur_info" | cut -d' ' -f2)
        _cur_repo=$(printf '%s' "$_cur_info" | cut -d' ' -f3)
        _cur_forgeapp=$(printf '%s' "$_cur_info" | cut -d' ' -f4)
        if [ "$_cur_repo" = "$_input_repo" ] && [ -n "$_cur_owner" ]; then
            _cur_slug="${_cur_owner}/${_cur_repo}"
            _cand="${_cur_slug}:${_cur_domain}:${_cur_forgeapp}"
            case " ${_candidate_keys} " in
                *" ${_cur_slug}@${_cur_domain} "*) ;;
                *)
                    _candidate_keys="${_candidate_keys}${_candidate_keys:+ }${_cur_slug}@${_cur_domain}"
                    _candidates="${_candidates}${_candidates:+ }${_cand}"
                    ;;
            esac
        fi
    fi

    # Count unique candidates
    _count=0
    for _c in $_candidates; do
        _count=$((_count + 1))
    done

    if [ "$_count" -eq 1 ]; then
        _c_slug=$(printf '%s' "$_candidates" | cut -d':' -f1)
        _c_domain=$(printf '%s' "$_candidates" | cut -d':' -f2)
        _c_forgeapp=$(printf '%s' "$_candidates" | cut -d':' -f3)
        _r_owner=$(printf '%s' "$_c_slug" | cut -d'/' -f1)
        _r_repo=$(printf '%s' "$_c_slug" | cut -d'/' -f2)
        [ -z "$_c_forgeapp" ] && _c_forgeapp=$(detect_forgeapp "$_c_domain" 2>/dev/null || echo "forgeapp_forgejo")
        printf '%s %s %s %s\n' "$_c_domain" "$_r_owner" "$_r_repo" "$_c_forgeapp"
        return 0
    elif [ "$_count" -gt 1 ]; then
        printf 'Error: O nome de repositório "%s" é ambíguo. Encontrados múltiplos correspondentes:\n' "$_input_repo" >&2
        for _c in $_candidates; do
            _c_slug=$(printf '%s' "$_c" | cut -d':' -f1)
            _c_domain=$(printf '%s' "$_c" | cut -d':' -f2)
            printf '  - %s (%s)\n' "$_c_slug" "${_c_domain:-desconhecida}" >&2
        done
        printf 'Por favor, informe o nome completo no formato "proprietario/repositorio".\n' >&2
        return 1
    fi

    printf 'Error: Repositório "%s" não encontrado localmente.\n' "$_input_repo" >&2
    return 1
}

_diva_escape_json() {
    _input="$1"
    _out=""
    _first=1
    while IFS= read -r _l || [ -n "$_l" ]; do
        if command -v escape_json_string >/dev/null 2>&1; then
            _esc_l=$(escape_json_string "$_l")
        else
            _esc_l=$(printf '%s' "$_l" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g')
        fi
        if [ "$_first" -eq 1 ]; then
            _out="$_esc_l"
            _first=0
        else
            _out="${_out}\\n${_esc_l}"
        fi
    done <<EOF
$_input
EOF
    printf '%s' "$_out"
}

_diva_save_draft() {
    _d_forge="${1:-unknown}"
    _d_owner="${2:-}"
    _d_repo="${3:-}"
    _d_title="${4:-}"
    _d_body="${5:-}"
    _d_labels="${6:-}"

    _d_git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$_d_git_root" ]; then
        printf '[ERRO] Impossível determinar git root para salvar draft.\n' >&2
        return 1
    fi

    _d_dir="${_d_git_root}/.antigravity/diva_drafts"
    mkdir -p "$_d_dir"
    if command -v ensure_gitignored >/dev/null 2>&1; then
        ensure_gitignored ".antigravity/"
    fi

    _d_ts=$(date +%Y%m%d_%H%M%S 2>/dev/null || printf 'unknown')
    _d_slug=$(printf '%s_%s_%s' "$_d_forge" "$_d_owner" "$_d_repo" | tr '/' '_')
    _d_file="${_d_dir}/${_d_ts}_${_d_slug}.json"

    _d_esc_title=$(_diva_escape_json "$_d_title")
    _d_esc_body=$(_diva_escape_json "$_d_body")
    _d_esc_labels=$(_diva_escape_json "$_d_labels")

    printf '{"forge":"%s","owner":"%s","repo":"%s","title":"%s","body":"%s","labels":"%s","saved_at":"%s"}\n' \
        "$_d_forge" "$_d_owner" "$_d_repo" \
        "$_d_esc_title" "$_d_esc_body" "$_d_esc_labels" \
        "$_d_ts" > "$_d_file"

    printf '  ⚠️  Draft salvo em: %s\n' "$_d_file" >&2
}

_diva_call_gemini() {
    _api_key="$1"
    _model="$2"
    _user_text="$3"
    _repo_context="${4:-}"
    _temperature="${5:-0.2}"
    _refinement="${6:-}"
    _prev_tickets="${7:-}"

    case "$_temperature" in
        [0-9]*.[0-9]*|[0-9]*) ;;
        *) _temperature="0.2" ;;
    esac

    _esc_text=$(_diva_escape_json "$_user_text")
    _esc_ctx=$(_diva_escape_json "$_repo_context")

    _sys_instructions="You are a senior software engineering triage assistant (O Divã do Gosplan). \
Your task is to analyze user reports, bug descriptions, and feature requests, and decompose them into one or more discrete, atomic issue tickets following Conventional Commits format for titles (e.g. feat(scope): title, fix(scope): title, refactor(scope): title). \
Split multi-topic or unrelated problems into separate tickets. \
Return ONLY a valid JSON object matching this schema: \
{\\\"tickets\\\": [{\\\"title\\\": \\\"Conventional Commit title\\\", \\\"rationale\\\": \\\"Why this is a distinct issue\\\", \\\"labels\\\": [\\\"bug\\\"], \\\"body\\\": \\\"## Description\\\\n\\\\n## Steps to Reproduce\\\\n\\\\n## Expected vs Actual\\\\n\\\"}]} \
Write the ticket body and descriptions in the language used by the user report."

    if [ -n "$_refinement" ] || [ -n "$_prev_tickets" ]; then
        _esc_ref=$(_diva_escape_json "$_refinement")
        _esc_prev=$(_diva_escape_json "$_prev_tickets")
        _prompt="Repository context: ${_esc_ctx}\n\nInitial user input:\n${_esc_text}\n\nPrevious proposed tickets JSON:\n${_esc_prev}\n\nUser refinement and parameter adjustment request:\n${_esc_ref}\n\nPlease update and refine the ticket decomposition according to the user's feedback. Return ONLY valid JSON matching the schema."
    else
        _prompt="Repository context: ${_esc_ctx}\n\nUser input to triage:\n${_esc_text}"
    fi

    _req_body=$(cat <<EOF
{
  "system_instruction": {
    "parts": [{ "text": "${_sys_instructions}" }]
  },
  "contents": [
    {
      "parts": [{ "text": "${_prompt}" }]
    }
  ],
  "generationConfig": {
    "responseMimeType": "application/json",
    "temperature": ${_temperature}
  }
}
EOF
)

    _url="https://generativelanguage.googleapis.com/v1beta/models/${_model}:generateContent?key=${_api_key}"

    _res=$(_diva_http -s -f -X POST \
        -H "Content-Type: application/json" \
        -d "$_req_body" \
        "$_url" 2>/dev/null || true)

    if [ -z "$_res" ]; then
        return 1
    fi

    printf '%s\n' "$_res"
    return 0
}

_diva_extract_tickets() {
    _raw_gemini_resp="$1"

    # Extract JSON text part
    _json_text=""
    if command -v jq >/dev/null 2>&1; then
        _json_text=$(printf '%s' "$_raw_gemini_resp" | jq -r '.candidates[0].content.parts[0].text // empty' 2>/dev/null || true)
    else
        # Pure POSIX fallback
        _json_text=$(printf '%s\n' "$_raw_gemini_resp" | sed -n '/[[:space:]]*"text":[[:space:]]*"/,$p' | sed -e '1s/^[[:space:]]*"text":[[:space:]]*"//' -e 's/"[[:space:]]*$//' -e 's/\\n/\n/g' -e 's/\\"/"/g' || true)
    fi

    if [ -z "$_json_text" ]; then
        return 1
    fi

    printf '%s\n' "$_json_text"
    return 0
}

_diva_extract_html_url() {
    _raw="$1"
    if command -v jq >/dev/null 2>&1; then
        _u=$(printf '%s' "$_raw" | jq -r '.html_url // empty' 2>/dev/null || true)
        if [ -n "$_u" ]; then
            printf '%s\n' "$_u"
            return 0
        fi
    fi
    # Pure POSIX fallback
    printf '%s\n' "$_raw" | sed -n 's/.*"html_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1
}

_diva_box_field() {
    _b_prefix="$1"
    _b_text="$2"
    _b_cont="              "
    _b_first=1

    _trimmed=$(printf '%s' "$_b_text" | tr -d ' \t\n')
    if [ -z "$_trimmed" ]; then
        _spaces=$(printf "%*s" 61 "")
        printf "│ %s%s │\n" "$_b_prefix" "$_spaces"
        return 0
    fi

    printf '%s\n' "$_b_text" | fold -s -w 61 | while IFS= read -r _b_chunk || [ -n "$_b_chunk" ]; do
        _b_chunk=$(printf '%s' "$_b_chunk" | sed 's/[[:space:]]*$//')
        [ -z "$_b_chunk" ] && continue

        if [ "$_b_first" -eq 1 ]; then
            _pfx="$_b_prefix"
            _b_first=0
        else
            _pfx="$_b_cont"
        fi

        _clen=$(printf '%s' "$_b_chunk" | wc -m | tr -d ' ')
        _pad=$((61 - _clen))
        [ "$_pad" -lt 0 ] && _pad=0
        _spaces=""
        [ "$_pad" -gt 0 ] && _spaces=$(printf "%*s" "$_pad" "")
        printf "│ %s%s%s │\n" "$_pfx" "$_b_chunk" "$_spaces"
    done
}

_diva_box_body() {
    _bb_body="$1"
    echo "│ 📝 Prévia do Corpo:                                                         │"
    [ -z "$_bb_body" ] && return 0

    printf '%s\n' "$_bb_body" | while IFS= read -r _bb_line || [ -n "$_bb_line" ]; do
        _bb_trimmed=$(printf '%s' "$_bb_line" | tr -d ' \t\n')
        if [ -z "$_bb_trimmed" ]; then
            _bb_spaces=$(printf "%*s" 75 "")
            printf "│ %s │\n" "$_bb_spaces"
        else
            printf '%s\n' "$_bb_line" | fold -s -w 72 | while IFS= read -r _bb_chunk || [ -n "$_bb_chunk" ]; do
                _bb_chunk=$(printf '%s' "$_bb_chunk" | sed 's/[[:space:]]*$//')
                [ -z "$_bb_chunk" ] && continue
                _clen=$(printf '%s' "$_bb_chunk" | wc -m | tr -d ' ')
                _pad=$((72 - _clen))
                [ "$_pad" -lt 0 ] && _pad=0
                _bb_spaces=""
                [ "$_pad" -gt 0 ] && _bb_spaces=$(printf "%*s" "$_pad" "")
                printf "│    %s%s │\n" "$_bb_chunk" "$_bb_spaces"
            done
        fi
    done
}

_diva_print_issue_and_branch() {
    _i_url="$1"
    _i_title="$2"

    [ -z "$_i_url" ] && return 0

    _i_num=$(printf '%s\n' "$_i_url" | sed -n 's|.*/issues/\([0-9]*\).*|\1|p')
    [ -z "$_i_num" ] && _i_num=$(printf '%s\n' "$_i_url" | sed -n 's|.*/\([0-9]*\)$|\1|p')

    if [ -n "$_i_num" ]; then
        echo "  🎯 Issue: #${_i_num}"

        _b_type="feat"
        case "$_i_title" in
            fix*|bug*|hotfix*) _b_type="fix" ;;
            docs*) _b_type="docs" ;;
            refactor*) _b_type="refactor" ;;
            test*) _b_type="test" ;;
            chore*) _b_type="chore" ;;
            ci*) _b_type="ci" ;;
            perf*) _b_type="perf" ;;
            build*) _b_type="build" ;;
        esac

        _b_subj=$(printf '%s\n' "$_i_title" | sed -e 's/^[^:]*:[[:space:]]*//' | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9' '-' | sed -e 's/^-//' -e 's/-$//')
        _b_slug=$(printf '%.35s' "$_b_subj" | sed 's/-$//')
        [ -z "$_b_slug" ] && _b_slug="ticket"

        echo "  🌿 Branch sugerido: git checkout -b ${_b_type}/${_i_num}-${_b_slug}"
    fi
}

_diva_present_and_dispatch() {
    _tickets_json="$1"
    _forge="$2"
    _owner="$3"
    _repo="$4"
    _dry_run="${5:-0}"
    _host_or_app="${6:-}"
    _api_key="${7:-}"
    _model="${8:-gemini-3.6-flash}"
    _user_text="${9:-}"
    _temp="${10:-0.2}"

    _domain=""
    _forgeapp=""
    case "$_host_or_app" in
        forgeapp_*)
            _forgeapp="$_host_or_app"
            _domain="$_forge"
            ;;
        *)
            if [ -n "$_host_or_app" ]; then
                case "$_forge" in
                    forgeapp_*|forgejo|codeberg|tildegit|gitea|github|gitlab|sourcehut)
                        _domain="$_host_or_app"
                        _forgeapp="$_forge"
                        ;;
                    *)
                        _domain="$_host_or_app"
                        _forgeapp="$_forge"
                        ;;
                esac
            else
                _domain="$_forge"
                _forgeapp=""
            fi
            ;;
    esac

    case "$_domain" in
        codeberg) _domain="codeberg.org" ;;
        tildegit) _domain="tildegit.org" ;;
        github) _domain="github.com" ;;
        gitlab) _domain="gitlab.com" ;;
        sourcehut) _domain="git.sr.ht" ;;
    esac

    case "$_forgeapp" in
        forgeapp_*) ;;
        forgejo|codeberg|tildegit|gitea) _forgeapp="forgeapp_forgejo" ;;
        github) _forgeapp="forgeapp_github" ;;
        gitlab) _forgeapp="forgeapp_gitlab" ;;
        sourcehut) _forgeapp="forgeapp_sourcehut" ;;
        *)
            _forgeapp=""
            if command -v detect_forgeapp >/dev/null 2>&1; then
                _forgeapp=$(detect_forgeapp "$_domain" 2>/dev/null || true)
                case "$_forgeapp" in
                    forgeapp_*) ;;
                    *) _forgeapp="" ;;
                esac
            fi
            [ -z "$_forgeapp" ] && _forgeapp="unknown"
            ;;
    esac

    _count=0
    if command -v jq >/dev/null 2>&1; then
        _count=$(printf '%s' "$_tickets_json" | jq '.tickets | length' 2>/dev/null || echo 0)
    else
        _count=$(printf '%s' "$_tickets_json" | grep -c '"title":' || echo 0)
    fi

    if [ "$_count" -le 0 ]; then
        echo "Aviso: Nenhum ticket foi identificado pelo modelo."
        return 0
    fi

    echo ""
    echo "★ [ГОСПЛАН] O Divã identificou $_count ticket(s) para abertura ★"
    echo ""

    _created_count=0

    _idx=0
    _should_restart=0
    while [ "$_idx" -lt "$_count" ]; do
        _title=""
        _rationale=""
        _labels=""
        _body=""

        if command -v jq >/dev/null 2>&1; then
            _title=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].title // empty")
            _rationale=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].rationale // empty")
            _labels=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].labels // [] | join(\", \")")
            _body=$(printf '%s' "$_tickets_json" | jq -r ".tickets[$_idx].body // empty")
        else
            # Pure POSIX fallback extraction for ticket fields
            _title=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"title"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | sed -n "$((_idx + 1))p")
            [ -z "$_title" ] && _title="Ticket $((_idx + 1))"
            _rationale=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"rationale"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | sed -n "$((_idx + 1))p")
            _labels=""
            _body=$(printf '%s\n' "$_tickets_json" | sed -n 's/.*"body"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | sed -n "$((_idx + 1))p" | sed -e 's/\\n/\n/g' -e 's/\\"/"/g')
        fi

        _num=$((_idx + 1))

        if [ "$_dry_run" -eq 1 ]; then
            echo "┌─────────────────────────────────────────────────────────────────────────────┐"
            _hdr=$(printf "[ДИВАН] Ticket %d/%d" "$_num" "$_count")
            _hlen=$(printf '%s' "$_hdr" | wc -m | tr -d ' ')
            _hpad=$((75 - _hlen))
            [ "$_hpad" -lt 0 ] && _hpad=0
            _hspaces=""
            [ "$_hpad" -gt 0 ] && _hspaces=$(printf "%*s" "$_hpad" "")
            printf "│ %s%s │\n" "$_hdr" "$_hspaces"
            echo "├─────────────────────────────────────────────────────────────────────────────┤"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                _diva_box_field "🎯 Destino  : " "${_owner}/${_repo} (${_forge})"
            fi
            _diva_box_field "📌 Título   : " "$_title"
            if [ -n "$_rationale" ]; then
                _diva_box_field "💡 Racional : " "$_rationale"
            fi
            if [ -n "$_labels" ]; then
                _diva_box_field "🏷️ Labels   : " "$_labels"
            fi
            _diva_box_body "$_body"
            echo "└─────────────────────────────────────────────────────────────────────────────┘"
            echo "  [DRY-RUN] Simulação ativa. Ticket não publicado."
            _idx=$((_idx + 1))
            continue
        fi

        while true; do
            echo "┌─────────────────────────────────────────────────────────────────────────────┐"
            _hdr=$(printf "[ДИВАН] Ticket %d/%d" "$_num" "$_count")
            _hlen=$(printf '%s' "$_hdr" | wc -m | tr -d ' ')
            _hpad=$((75 - _hlen))
            [ "$_hpad" -lt 0 ] && _hpad=0
            _hspaces=""
            [ "$_hpad" -gt 0 ] && _hspaces=$(printf "%*s" "$_hpad" "")
            printf "│ %s%s │\n" "$_hdr" "$_hspaces"
            echo "├─────────────────────────────────────────────────────────────────────────────┤"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                _diva_box_field "🎯 Destino  : " "${_owner}/${_repo} (${_forge})"
            fi
            _diva_box_field "📌 Título   : " "$_title"
            if [ -n "$_rationale" ]; then
                _diva_box_field "💡 Racional : " "$_rationale"
            fi
            if [ -n "$_labels" ]; then
                _diva_box_field "🏷️ Labels   : " "$_labels"
            fi
            _diva_box_body "$_body"
            echo "└─────────────────────────────────────────────────────────────────────────────┘"

            # Interactive confirmation
            _choice="y"
            if [ -t 0 ]; then
                if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                    printf "  [y] Criar em %s/%s (%s)  [n] Pular  [t] Editar título  [b] Editar corpo  [r] Rediscutir com IA  [f] Ver corpo completo → " "$_owner" "$_repo" "$_forge"
                else
                    printf "  [y] Criar  [n] Pular  [t] Editar título  [b] Editar corpo  [r] Rediscutir com IA  [f] Ver corpo completo → "
                fi
                read -r _user_choice || _user_choice="n"
                _choice=$(printf '%s' "$_user_choice" | tr '[:upper:]' '[:lower:]')
            else
                if read -r _user_choice; then
                    _choice=$(printf '%s' "$_user_choice" | tr '[:upper:]' '[:lower:]')
                else
                    _choice="y"
                fi
            fi

            case "$_choice" in
                y|yes|"")
                    echo "  -> Publicando ticket na forja ${_domain} (${_owner}/${_repo})..."
                    _issue_url=""

                    case "$_forgeapp" in
                        forgeapp_forgejo)
                            _base_url="https://${_domain}"

                            _token=$(_diva_resolve_forge_token "$_domain" 2>/dev/null || true)
                            [ -z "$_token" ] && _token=$(_diva_resolve_forge_token "$_forgeapp" 2>/dev/null || true)
                            if [ -z "$_token" ]; then
                                printf '  [ERRO] Token para %s não encontrado. Ticket não publicado.\n' "$_domain" >&2
                                _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                break
                            fi

                            _esc_t=$(_diva_escape_json "$_title")
                            _esc_b=$(_diva_escape_json "$_body")
                            _issue_payload="{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}"

                            _post_res=$(_diva_http -s -X POST \
                                -H "Authorization: token ${_token}" \
                                -H "Content-Type: application/json" \
                                -H "Accept: application/json" \
                                -d "$_issue_payload" \
                                "${_base_url}/api/v1/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)

                            _issue_url=$(_diva_extract_html_url "$_post_res")
                            if [ -n "$_issue_url" ]; then
                                echo "  ✅ Criado: ${_issue_url}"
                                _diva_print_issue_and_branch "$_issue_url" "$_title"
                                _created_count=$((_created_count + 1))
                            else
                                printf '  [ERRO] Falha ao publicar em %s. Resposta: %s\n' "$_domain" "$_post_res" >&2
                                _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            fi
                            ;;

                        forgeapp_github)
                            if command -v gh >/dev/null 2>&1; then
                                _gh_url=$(gh issue create --repo "${_owner}/${_repo}" --title "$_title" --body "$_body" 2>/dev/null || true)
                                if [ -n "$_gh_url" ]; then
                                    echo "  ✅ Criado: ${_gh_url}"
                                    _diva_print_issue_and_branch "$_gh_url" "$_title"
                                    _created_count=$((_created_count + 1))
                                else
                                    printf '  [ERRO] gh cli falhou ao criar issue no GitHub.\n' >&2
                                    _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                fi
                            else
                                _token=$(_diva_resolve_forge_token "github.com" 2>/dev/null || true)
                                [ -z "$_token" ] && _token=$(_diva_resolve_forge_token "$_forgeapp" 2>/dev/null || true)
                                if [ -n "$_token" ]; then
                                    _esc_t=$(_diva_escape_json "$_title")
                                    _esc_b=$(_diva_escape_json "$_body")
                                    _post_res=$(_diva_http -s -X POST \
                                        -H "Authorization: token ${_token}" \
                                        -H "Content-Type: application/json" \
                                        -H "Accept: application/vnd.github+json" \
                                        -d "{\"title\":\"${_esc_t}\",\"body\":\"${_esc_b}\"}" \
                                        "https://api.github.com/repos/${_owner}/${_repo}/issues" 2>/dev/null || true)
                                    _issue_url=$(_diva_extract_html_url "$_post_res")
                                    if [ -n "$_issue_url" ]; then
                                        echo "  ✅ Criado: ${_issue_url}"
                                        _diva_print_issue_and_branch "$_issue_url" "$_title"
                                        _created_count=$((_created_count + 1))
                                    else
                                        printf '  [ERRO] Falha ao publicar no GitHub. Resposta: %s\n' "$_post_res" >&2
                                        _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                    fi
                                else
                                    printf '  [ERRO] Nem "gh" CLI nem GITHUB_TOKEN disponíveis para publicação no GitHub.\n' >&2
                                    _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                                fi
                            fi
                            ;;

                        *)
                            printf '  [ERRO] Forja "%s" (motor "%s") não suportada para publicação automática.\n' "$_domain" "$_forgeapp" >&2
                            _diva_save_draft "$_domain" "$_owner" "$_repo" "$_title" "$_body" "$_labels"
                            ;;
                    esac
                    break
                    ;;

                f|full)
                    echo ""
                    echo "--- CORPO COMPLETO ---"
                    printf '%s\n' "$_body"
                    echo "-----------------------"
                    ;;

                t|title|e|edit)
                    printf "  Novo título [%s]: " "$_title"
                    if [ -t 0 ] && [ -e /dev/tty ]; then
                        read -r _new_title < /dev/tty || _new_title=""
                    else
                        read -r _new_title || _new_title=""
                    fi
                    if [ -n "$_new_title" ]; then
                        _title="$_new_title"
                        echo "  ✓ Título atualizado."
                    fi
                    ;;

                b|body)
                    _editor=$(_diva_resolve_editor 2>/dev/null || true)

                    _tmp_body=$(mktemp 2>/dev/null || printf '/tmp/diva_body_%s_%s.md' "$$" "$_idx")
                    printf '%s\n' "$_body" > "$_tmp_body"

                    if [ -n "$_editor" ] && { [ -t 0 ] || [ "$_editor" != "vi" ]; }; then
                        echo "  -> Abrindo editor ($_editor) para editar o corpo do ticket..."
                        if [ -t 0 ] && [ -e /dev/tty ]; then
                            $_editor "$_tmp_body" < /dev/tty > /dev/tty 2>&1 || true
                        else
                            $_editor "$_tmp_body" || true
                        fi
                        if [ -f "$_tmp_body" ]; then
                            _body=$(cat "$_tmp_body")
                            echo "  ✓ Corpo atualizado."
                        fi
                    else
                        printf "  Digite o novo corpo (ou deixe vazio para manter atual): "
                        if [ -t 0 ] && [ -e /dev/tty ]; then
                            read -r _inline_body < /dev/tty || _inline_body=""
                        else
                            read -r _inline_body || _inline_body=""
                        fi
                        if [ -n "$_inline_body" ]; then
                            _body="$_inline_body"
                            echo "  ✓ Corpo atualizado."
                        fi
                    fi
                    rm -f "$_tmp_body" 2>/dev/null || true
                    ;;

                r|refine|rediscutir)
                    if [ -z "$_api_key" ]; then
                        _api_key=$(_diva_resolve_api_key 2>/dev/null || true)
                    fi
                    if [ -z "$_api_key" ]; then
                        printf '  [ERRO] GEMINI_API_KEY não configurada. Não é possível rediscutir com a IA.\n' >&2
                        continue
                    fi

                    printf "  💬 Instruções para a Gemini (ou ':temp <val>', ':model <val>'): "
                    if [ -t 0 ] && [ -e /dev/tty ]; then
                        read -r _refinement_raw < /dev/tty || _refinement_raw=""
                    else
                        read -r _refinement_raw || _refinement_raw=""
                    fi

                    if [ -z "$_refinement_raw" ]; then
                        echo "  Aviso: Nenhuma instrução informada. Mantendo proposta atual."
                        continue
                    fi

                    # Parse inline directives like :temp <val> or :model <val>
                    _clean_instructions=""
                    _next_arg=""
                    for _tok in $_refinement_raw; do
                        if [ "$_next_arg" = "temp" ]; then
                            _temp="$_tok"
                            _next_arg=""
                        elif [ "$_next_arg" = "model" ]; then
                            _model="$_tok"
                            _next_arg=""
                        elif [ "$_tok" = ":temp" ]; then
                            _next_arg="temp"
                        elif [ "$_tok" = ":model" ]; then
                            _next_arg="model"
                        elif [ "${_tok#*:temp=}" != "$_tok" ]; then
                            _temp="${_tok#:temp=}"
                        elif [ "${_tok#*:model=}" != "$_tok" ]; then
                            _model="${_tok#:model=}"
                        else
                            if [ -z "$_clean_instructions" ]; then
                                _clean_instructions="$_tok"
                            else
                                _clean_instructions="${_clean_instructions} ${_tok}"
                            fi
                        fi
                    done

                    case "$_temp" in
                        [0-9]*.[0-9]*|[0-9]*) ;;
                        *) _temp="0.2" ;;
                    esac

                    echo "  -> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) com refinamento..."
                    _repo_ctx=""
                    if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                        _repo_ctx="${_owner}/${_repo} (${_forge})"
                    fi

                    _raw_resp=$(_diva_call_gemini "$_api_key" "$_model" "$_user_text" "$_repo_ctx" "$_temp" "$_clean_instructions" "$_tickets_json" 2>/dev/null || true)
                    _new_tickets=$(_diva_extract_tickets "$_raw_resp" 2>/dev/null || true)

                    _new_count=0
                    if [ -n "$_new_tickets" ]; then
                        if command -v jq >/dev/null 2>&1; then
                            _new_count=$(printf '%s' "$_new_tickets" | jq '.tickets | length' 2>/dev/null || echo 0)
                        else
                            _new_count=$(printf '%s' "$_new_tickets" | grep -c '"title":' || echo 0)
                        fi
                    fi

                    if [ "$_new_count" -gt 0 ]; then
                        echo "  ✓ Proposta refinada com sucesso pela Gemini ($_new_count ticket(s))!"
                        _tickets_json="$_new_tickets"
                        _count="$_new_count"
                        _idx=0
                        _should_restart=1
                        break
                    else
                        printf '  [ERRO] Falha ao rediscutir com a Gemini ou resposta inválida. Mantendo proposta atual.\n' >&2
                    fi
                    ;;

                n|no|*)
                    echo "  ⏭️  Pulado."
                    break
                    ;;
            esac
        done

        if [ "$_should_restart" -eq 1 ]; then
            _should_restart=0
            continue
        fi

        _idx=$((_idx + 1))
    done

    # Re-sync local tickets if tickets were created
    if [ "$_created_count" -gt 0 ]; then
        if command -v cmd_tickets >/dev/null 2>&1; then
            echo ""
            echo "-> Sincronizando .antigravity/tickets.md..."
            cmd_tickets >/dev/null 2>&1 || true
        fi
    fi

    echo ""
    echo "Sessão concluída. $_created_count ticket(s) publicado(s)."
    return 0
}

cmd_diva() {
    _model="${GEMINI_MODEL:-gemini-3.6-flash}"
    _temp="${GEMINI_TEMPERATURE:-0.2}"
    _dry_run=0
    _user_text=""
    _custom_repo=""

    while [ $# -gt 0 ]; do
        case "$1" in
            config)
                shift
                if [ $# -gt 0 ]; then
                    _key="$1"
                    _local_cfg=$(get_local_config_file 2>/dev/null || true)
                    if [ -z "$_local_cfg" ]; then
                        if command -v get_global_config_file >/dev/null 2>&1; then
                            _local_cfg=$(get_global_config_file 2>/dev/null || true)
                        else
                            _local_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
                        fi
                        mkdir -p "$(dirname "$_local_cfg")" 2>/dev/null || true
                    fi
                    write_file_key "$_local_cfg" "gemini_api_key" "$_key"
                    echo "Chave gemini_api_key salva com sucesso em $_local_cfg"
                    return 0
                else
                    echo "Uso: cccp diva config <GEMINI_API_KEY>"
                    return 1
                fi
                ;;
            --web|-w)
                shift
                if command -v cmd_cockpit >/dev/null 2>&1; then
                    cmd_cockpit "diva" "$@"
                    return $?
                elif command -v cmd_web >/dev/null 2>&1; then
                    cmd_web "diva" "$@"
                    return $?
                else
                    echo "Error: cccp cockpit module is not loaded." >&2
                    return 1
                fi
                ;;
            --dry-run)
                _dry_run=1
                shift
                ;;
            --model|-m)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _model="$1"
                shift
                ;;
            --temp|-t|--temperature)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _temp="$1"
                shift
                ;;
            --repo|-r)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for $1" >&2; return 1; }
                _custom_repo="$1"
                shift
                ;;
            -h|--help|help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "diva"
                else
                    echo "Uso: cccp diva [options] [texto_do_problema]"
                    echo "  --dry-run          Executa triagem sem publicar na forja"
                    echo "  --model <model>    Modelo Gemini (padrão: gemini-3.6-flash)"
                    echo "  --temp <val>       Temperatura de amostragem (padrão: 0.2)"
                    echo "  --repo <slug>      Repositório alvo (padrão: detectado do git remote)"
                    echo "  config <api-key>   Salva GEMINI_API_KEY no .cccprc"
                fi
                return 0
                ;;
            -*)
                echo "Error: Unknown option '$1'" >&2
                return 1
                ;;
            *)
                if [ -z "$_user_text" ]; then
                    _user_text="$1"
                else
                    _user_text="${_user_text} $1"
                fi
                shift
                ;;
        esac
    done

    # 1. Resolve Gemini API Key
    _api_key=$(_diva_resolve_api_key 2>/dev/null || true)
    if [ -z "$_api_key" ]; then
        echo "Error: GEMINI_API_KEY não configurada." >&2
        echo "Configure através de:" >&2
        echo "  export GEMINI_API_KEY=\"sua_chave\"" >&2
        echo "  ou: cccp diva config <sua_chave>" >&2
        return 1
    fi

    # 2. Resolve Forge Info
    _domain="unknown"
    _owner=""
    _repo=""
    _forgeapp="unknown"
    if [ -n "$_custom_repo" ]; then
        if _cinfo=$(_diva_resolve_custom_repo "$_custom_repo"); then
            _domain=$(printf '%s' "$_cinfo" | cut -d' ' -f1)
            _owner=$(printf '%s' "$_cinfo" | cut -d' ' -f2)
            _repo=$(printf '%s' "$_cinfo" | cut -d' ' -f3)
            _forgeapp=$(printf '%s' "$_cinfo" | cut -d' ' -f4)
        else
            return 1
        fi
    else
        if _finfo=$(_diva_resolve_forge 2>/dev/null); then
            _domain=$(printf '%s' "$_finfo" | cut -d' ' -f1)
            _owner=$(printf '%s' "$_finfo" | cut -d' ' -f2)
            _repo=$(printf '%s' "$_finfo" | cut -d' ' -f3)
            _forgeapp=$(printf '%s' "$_finfo" | cut -d' ' -f4)
        fi
    fi
    _forge="$_domain"
    _host="$_domain"

    # 3. Read input from stdin if not provided as argument
    if [ -z "$_user_text" ]; then
        if [ ! -t 0 ]; then
            _user_text=$(cat)
        else
            echo "★ [ГОСПЛАН] O Divã — Triagem Interativa de Tickets ★"
            if [ -n "$_owner" ] && [ -n "$_repo" ]; then
                echo "🎯 Repositório Alvo: ${_owner}/${_repo} (${_domain})"
            fi
            echo "Descreva seu problema, bug ou funcionalidade (digite seu relato e pressione Enter):"
            printf "> "
            read -r _user_text || _user_text=""
        fi
    fi

    if [ -z "$_user_text" ]; then
        echo "Nenhum relato informado. Encerrando sessão."
        return 0
    fi

    case "$_temp" in
        [0-9]*.[0-9]*|[0-9]*) ;;
        *) _temp="0.2" ;;
    esac

    if [ -n "$_owner" ] && [ -n "$_repo" ]; then
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) para ${_owner}/${_repo} (${_domain})..."
    else
        echo "-> [ГОСПЛАН] Consultando Gemini ($_model, temp=$_temp) para decomposição de tickets..."
    fi

    _repo_ctx="${_owner}/${_repo} (${_domain})"
    _raw_resp=$(_diva_call_gemini "$_api_key" "$_model" "$_user_text" "$_repo_ctx" "$_temp" 2>/dev/null || true)

    if [ -z "$_raw_resp" ]; then
        echo "Error: Falha na comunicação com a API do Google Gemini." >&2
        return 1
    fi

    _tickets_json=$(_diva_extract_tickets "$_raw_resp" 2>/dev/null || true)
    if [ -z "$_tickets_json" ]; then
        echo "Error: Resposta do modelo não pôde ser interpretada." >&2
        return 1
    fi

    _diva_present_and_dispatch "$_tickets_json" "$_domain" "$_owner" "$_repo" "$_dry_run" "$_forgeapp" "$_api_key" "$_model" "$_user_text" "$_temp"
}

cmd_divan() {
    cmd_diva "$@"
}

cmd_ticket() {
    cmd_diva "$@"
}

cmd_issue() {
    cmd_diva "$@"
}


# =============================================================================
# Matraca Mobile Web REPL & Filesystem Queue Functions
# =============================================================================
_matraca_get_dir() {
    if [ -n "${MATRACA_DATA_DIR:-}" ]; then
        _mdir="$MATRACA_DATA_DIR"
    elif [ -n "${XDG_DATA_HOME:-}" ]; then
        _mdir="${XDG_DATA_HOME}/cccp/matraca"
    else
        _mdir="${HOME}/.local/share/cccp/matraca"
    fi

    # Ensure queue directory structure exists
    mkdir -p "${_mdir}/clients" \
             "${_mdir}/queue/pending" \
             "${_mdir}/queue/processing" \
             "${_mdir}/queue/done" \
             "${_mdir}/queue/error" 2>/dev/null || true

    printf '%s\n' "$_mdir"
}

_matraca_rand() {
    _len="${1:-6}"
    if [ -r /dev/urandom ]; then
        od -vN 16 -An -tx1 /dev/urandom 2>/dev/null | tr -cd 'a-z0-9' | cut -c 1-"$_len"
    else
        printf '%s%s' "$(date +%s)" "$$" | tr -cd 'a-z0-9' | cut -c 1-"$_len"
    fi
}

_matraca_escape_json() {
    _raw="$1"
    _out=""
    _first=1
    while IFS= read -r _line || [ -n "$_line" ]; do
        _esc_line=$(printf '%s' "$_line" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g')
        if [ "$_first" -eq 1 ]; then
            _out="$_esc_line"
            _first=0
        else
            _out="${_out}\\n${_esc_line}"
        fi
    done <<EOF
$_raw
EOF
    printf '%s' "$_out"
}

_matraca_extract_field() {
    _json_file="$1"
    _field_name="$2"
    if [ ! -f "$_json_file" ]; then
        return 1
    fi
    if command -v jq >/dev/null 2>&1; then
        jq -r ".${_field_name} // empty" "$_json_file" 2>/dev/null && return 0
    fi
    sed -n "s/.*\"${_field_name}\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" "$_json_file" 2>/dev/null | head -n 1
}

_matraca_trigger_worker() {
    if [ "${MATRACA_NO_AUTO_WORKER:-0}" = "1" ]; then
        return 0
    fi
    _bin=$(command -v cccp 2>/dev/null || true)
    if [ -z "$_bin" ] && [ -n "${GIT_ROOT:-}" ] && [ -x "${GIT_ROOT}/bin/cccp" ]; then
        _bin="${GIT_ROOT}/bin/cccp"
    fi
    if [ -z "$_bin" ]; then
        _bin="$0"
    fi

    # Trigger worker run in detached background process
    if [ -x "$_bin" ]; then
        "$_bin" matraca worker --once >/dev/null 2>&1 &
    fi
}

cmd_matraca_send() {
    _client_id=""
    _workspace=""
    _prompt=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --client-id)
                shift
                [ $# -gt 0 ] || { echo "Error: --client-id requires an ID argument" >&2; return 1; }
                _client_id="$1"
                ;;
            --workspace)
                shift
                [ $# -gt 0 ] || { echo "Error: --workspace requires a directory argument" >&2; return 1; }
                _workspace="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca send [--client-id <id>] [--workspace <dir>] <prompt...>"
                return 0
                ;;
            *)
                if [ -z "$_prompt" ]; then
                    _prompt="$1"
                else
                    _prompt="${_prompt} $1"
                fi
                ;;
        esac
        shift
    done

    # If prompt is still empty, try reading from stdin if available
    if [ -z "$_prompt" ] && [ ! -t 0 ]; then
        _prompt=$(cat)
    fi

    if [ -z "$_prompt" ]; then
        printf '{"ok": false, "error": "Prompt cannot be empty"}\n' >&2
        return 1
    fi

    _mdir=$(_matraca_get_dir)
    _now=$(date +%s 2>/dev/null || echo 0)

    # 1. Ensure client_id
    if [ -z "$_client_id" ]; then
        _client_id="cli_${_now}_$(_matraca_rand 8)"
    fi

    # 2. Idempotency Check: if client_id already submitted, return existing job
    _client_file="${_mdir}/clients/${_client_id}"
    if [ -f "$_client_file" ]; then
        _existing_job_id=$(head -n 1 "$_client_file" | tr -d ' \t\r\n')
        if [ -n "$_existing_job_id" ]; then
            for _sub in done processing pending error; do
                _candidate="${_mdir}/queue/${_sub}/${_existing_job_id}.json"
                if [ -f "$_candidate" ]; then
                    cat "$_candidate"
                    printf '\n'
                    return 0
                fi
            done
        fi
    fi

    # 3. Resolve Workspace
    if [ -z "$_workspace" ]; then
        _workspace="${PWD:-$(pwd)}"
    fi

    # 4. Generate Monotonic Unique Job ID (timestamp + random suffix + pid)
    _job_id="${_now}_$(_matraca_rand 6)_$$"

    _esc_prompt=$(_matraca_escape_json "$_prompt")
    _esc_workspace=$(_matraca_escape_json "$_workspace")
    _esc_client_id=$(_matraca_escape_json "$_client_id")

    # 5. Write Job File Atomically to Pending Queue
    _tmp_job="${_mdir}/queue/tmp_${_job_id}.json"
    _target_job="${_mdir}/queue/pending/${_job_id}.json"

    cat > "$_tmp_job" << EOF
{
  "id": "${_job_id}",
  "client_id": "${_esc_client_id}",
  "status": "pending",
  "workspace": "${_esc_workspace}",
  "prompt": "${_esc_prompt}",
  "response": "",
  "exit_code": 0,
  "created_at": ${_now},
  "updated_at": ${_now}
}
EOF

    mv "$_tmp_job" "$_target_job"

    # 6. Record client_id mapping for idempotency
    printf '%s\n' "$_job_id" > "$_client_file"

    # 7. Print immediate acknowledgment in <10ms
    printf '{"ok": true, "id": "%s", "client_id": "%s", "status": "pending", "created_at": %s}\n' \
        "$_job_id" "$_client_id" "$_now"

    # 8. Trigger background worker
    _matraca_trigger_worker

    return 0
}

cmd_matraca_poll() {
    if [ $# -lt 1 ]; then
        echo "Usage: cccp matraca poll <job_id|client_id>" >&2
        return 1
    fi
    _query_id="$1"
    _mdir=$(_matraca_get_dir)

    _job_id="$_query_id"
    # Check if input is a known client_id
    if [ -f "${_mdir}/clients/${_query_id}" ]; then
        _job_id=$(head -n 1 "${_mdir}/clients/${_query_id}" | tr -d ' \t\r\n')
    fi

    # Search in order of resolution probability: done, processing, pending, error
    for _sub in done processing pending error; do
        _f="${_mdir}/queue/${_sub}/${_job_id}.json"
        if [ -f "$_f" ]; then
            cat "$_f"
            printf '\n'
            return 0
        fi
    done

    printf '{"ok": false, "status": "not_found", "error": "Job '%s' not found"}\n' "$_query_id" >&2
    return 1
}

cmd_matraca_worker() {
    _daemon=0
    _target_job_id=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --daemon)
                _daemon=1
                ;;
            --once)
                _daemon=0
                ;;
            --job-id)
                shift
                [ $# -gt 0 ] || { echo "Error: --job-id requires an ID argument" >&2; return 1; }
                _target_job_id="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca worker [--daemon] [--once] [--job-id <id>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)

    _process_single_file() {
        _pending_file="$1"
        [ -f "$_pending_file" ] || return 0

        _fname=$(basename "$_pending_file")
        _processing_file="${_mdir}/queue/processing/${_fname}"

        # Atomic Claim via rename()
        if ! mv "$_pending_file" "$_processing_file" 2>/dev/null; then
            # Another worker claimed it first
            return 0
        fi

        _job_id=$(_matraca_extract_field "$_processing_file" "id")
        _client_id=$(_matraca_extract_field "$_processing_file" "client_id")
        _workspace=$(_matraca_extract_field "$_processing_file" "workspace")

        if [ -z "$_workspace" ] || [ ! -d "$_workspace" ]; then
            _workspace="${PWD:-$(pwd)}"
        fi

        # Extract prompt
        _prompt=""
        if command -v jq >/dev/null 2>&1; then
            _prompt=$(jq -r '.prompt // empty' "$_processing_file" 2>/dev/null || true)
        fi
        if [ -z "$_prompt" ]; then
            _prompt=$(_matraca_extract_field "$_processing_file" "prompt")
        fi

        # Execute Agent Runner
        _agent_output=""
        _agent_exit=0
        _start_time=$(date +%s 2>/dev/null || echo 0)

        if [ -n "${MATRACA_RUNNER:-}" ]; then
            # Custom / Mock Runner for hermetic automated tests
            _agent_output=$(eval "$MATRACA_RUNNER" 2>&1) || _agent_exit=$?
        else
            # Official Antigravity CLI invocation in workspace
            _agy_bin=$(command -v agy 2>/dev/null || true)
            if [ -n "$_agy_bin" ]; then
                _agent_output=$( (cd "$_workspace" && "$_agy_bin" --continue --print "$_prompt") 2>&1 ) || _agent_exit=$?
            else
                _agent_output="Erro: Binário 'agy' (Antigravity CLI) não encontrado no PATH."
                _agent_exit=127
            fi
        fi

        _finish_time=$(date +%s 2>/dev/null || echo 0)
        _esc_out=$(_matraca_escape_json "$_agent_output")
        _esc_p=$(_matraca_escape_json "$_prompt")
        _esc_ws=$(_matraca_escape_json "$_workspace")
        _esc_cid=$(_matraca_escape_json "$_client_id")

        if [ "$_agent_exit" -eq 0 ]; then
            _final_status="done"
            _dest_dir="${_mdir}/queue/done"
        else
            _final_status="error"
            _dest_dir="${_mdir}/queue/error"
        fi

        _tmp_dest="${_mdir}/queue/tmp_final_${_fname}"
        cat > "$_tmp_dest" << EOF
{
  "id": "${_job_id}",
  "client_id": "${_esc_cid}",
  "status": "${_final_status}",
  "workspace": "${_esc_ws}",
  "prompt": "${_esc_p}",
  "response": "${_esc_out}",
  "exit_code": ${_agent_exit},
  "created_at": ${_start_time},
  "updated_at": ${_finish_time}
}
EOF

        mv "$_tmp_dest" "${_dest_dir}/${_fname}"
        rm -f "$_processing_file" 2>/dev/null || true
    }

    if [ -n "$_target_job_id" ]; then
        _single_target="${_mdir}/queue/pending/${_target_job_id}.json"
        _process_single_file "$_single_target"
        return 0
    fi

    if [ "$_daemon" -eq 1 ]; then
        echo "[Matraca Worker] Daemon iniciado. Monitorando fila em ${_mdir}/queue/pending..."
        while true; do
            _found=0
            for _f in "${_mdir}"/queue/pending/*.json; do
                if [ -f "$_f" ]; then
                    _found=1
                    _process_single_file "$_f"
                fi
            done
            if [ "$_found" -eq 0 ]; then
                sleep 1
            fi
        done
    else
        # Process once across all current pending files
        for _f in "${_mdir}"/queue/pending/*.json; do
            if [ -f "$_f" ]; then
                _process_single_file "$_f"
            fi
        done
    fi

    return 0
}

cmd_matraca_history() {
    _limit=50
    while [ $# -gt 0 ]; do
        case "$1" in
            --limit|-n)
                shift
                [ $# -gt 0 ] || { echo "Error: --limit requires a number" >&2; return 1; }
                _limit="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca history [--limit <N>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)

    # Collect files from done and error (and pending/processing)
    _count=0
    printf '{"ok": true, "messages": ['
    _first=1

    # In pure POSIX, gather and sort descending
    for _sub in done error processing pending; do
        if [ -d "${_mdir}/queue/${_sub}" ]; then
            for _f in "${_mdir}/queue/${_sub}"/*.json; do
                if [ -f "$_f" ]; then
                    _count=$((_count + 1))
                    if [ "$_count" -gt "$_limit" ]; then
                        break
                    fi
                    if [ "$_first" -eq 1 ]; then
                        _first=0
                    else
                        printf ','
                    fi
                    cat "$_f"
                fi
            done
        fi
        if [ "$_count" -ge "$_limit" ]; then
            break
        fi
    done

    printf ']}\n'
    return 0
}

cmd_matraca_status() {
    _json=0
    case "${1:-}" in
        --json) _json=1 ;;
        -h|--help)
            echo "Usage: cccp matraca status [--json]"
            return 0
            ;;
    esac

    _mdir=$(_matraca_get_dir)

    _count_dir() {
        _d="$1"
        _c=0
        if [ -d "$_d" ]; then
            for _f in "${_d}"/*.json; do
                [ -f "$_f" ] && _c=$((_c + 1))
            done
        fi
        printf '%s' "$_c"
    }

    _pending_count=$(_count_dir "${_mdir}/queue/pending")
    _processing_count=$(_count_dir "${_mdir}/queue/processing")
    _done_count=$(_count_dir "${_mdir}/queue/done")
    _error_count=$(_count_dir "${_mdir}/queue/error")

    _clients_count=0
    if [ -d "${_mdir}/clients" ]; then
        for _cf in "${_mdir}/clients"/*; do
            [ -f "$_cf" ] && _clients_count=$((_clients_count + 1))
        done
    fi

    if [ "$_json" -eq 1 ]; then
        printf '{"ok": true, "pending": %s, "processing": %s, "done": %s, "error": %s, "clients": %s, "data_dir": "%s"}\n' \
            "$_pending_count" "$_processing_count" "$_done_count" "$_error_count" "$_clients_count" "$_mdir"
    else
        echo "🦜 CCCP Matraca — Queue & Spool Status"
        echo "======================================"
        echo "  Diretório : ${_mdir}"
        echo "  Na fila   : ${_pending_count}"
        echo "  Executando: ${_processing_count}"
        echo "  Concluídos: ${_done_count}"
        echo "  Erros     : ${_error_count}"
        echo "  Clientes  : ${_clients_count}"
    fi

    return 0
}

cmd_matraca_clean() {
    _days=7
    while [ $# -gt 0 ]; do
        case "$1" in
            --days|-d)
                shift
                [ $# -gt 0 ] || { echo "Error: --days requires a number" >&2; return 1; }
                _days="$1"
                ;;
            -h|--help)
                echo "Usage: cccp matraca clean [--days <N>]"
                return 0
                ;;
        esac
        shift
    done

    _mdir=$(_matraca_get_dir)
    # Find files older than N days in done and error queues
    find "${_mdir}/queue/done" "${_mdir}/queue/error" -type f -mtime +"$_days" -delete 2>/dev/null || true
    echo "Fila do Matraca limpa com sucesso (arquivos > ${_days} dias removidos)."
    return 0
}

cmd_matraca() {
    if [ $# -eq 0 ]; then
        cmd_matraca --help
        echo ""
        echo "💡 Para acessar a interface gráfica completa no navegador ou celular, execute:"
        echo "   cccp cockpit"
        return 0
    fi

    _sub="$1"
    shift

    case "$_sub" in
        send)
            cmd_matraca_send "$@"
            ;;
        poll)
            cmd_matraca_poll "$@"
            ;;
        worker)
            cmd_matraca_worker "$@"
            ;;
        history)
            cmd_matraca_history "$@"
            ;;
        status)
            cmd_matraca_status "$@"
            ;;
        clean)
            cmd_matraca_clean "$@"
            ;;
        web|serve)
            echo "💡 A interface web foi unificada no comando 'cccp cockpit'." >&2
            if command -v cmd_cockpit >/dev/null 2>&1; then
                cmd_cockpit matraca "$@"
            elif command -v cmd_web >/dev/null 2>&1; then
                cmd_web matraca "$@"
            fi
            ;;
        -h|--help|help)
            echo "cccp matraca - Mobile & Offline-Tolerant Web REPL for Antigravity (agy)"
            echo "====================================================================="
            echo ""
            echo "Usage:"
            echo "  cccp matraca send [--client-id <id>] [--workspace <dir>] <prompt...>"
            echo "  cccp matraca poll <job_id|client_id>"
            echo "  cccp matraca worker [--daemon] [--once] [--job-id <id>]"
            echo "  cccp matraca history [--limit <N>]"
            echo "  cccp matraca status [--json]"
            echo "  cccp matraca clean [--days <N>]"
            echo ""
            echo "Note:"
            echo "  Para abrir o Web REPL no navegador/celular, utilize: cccp cockpit"
            return 0
            ;;
        *)
            # If unknown subcommand, check if it was intended as a prompt for send
            cmd_matraca_send "$_sub" "$@"
            ;;
    esac
}


# =============================================================================
# Web Server & Bridge Functions
# =============================================================================
_web_generate_token() {
    if [ -r /dev/urandom ]; then
        od -vN 16 -An -tx1 /dev/urandom 2>/dev/null | tr -d ' \t\r\n'
    else
        printf '%s%s%s\n' "$(date +%s)" "$$" "$RANDOM" | tr -cd '0-9a-zA-Z'
    fi
}

_web_export_assets() {
    _out_dir="$1"
    _token="${2:-}"
    _listen_port="${3:-8080}"
    _enable_matraca="${4:-1}"
    _enable_diva="${5:-1}"
    _enable_status="${6:-1}"
    _enable_tickets="${7:-1}"
    _initial_tab="${8:-matraca}"

    _nav_html=""
    _main_html=""

    if [ "$_enable_matraca" -eq 1 ]; then
        _act=""
        [ "$_initial_tab" = "matraca" ] && _act=" active"
        _nav_html="${_nav_html}    <button id=\"nav-matraca\" class=\"${_act}\" onclick=\"switchTab('matraca')\">🦜 Matraca (Web REPL)</button>
"
        _main_html="${_main_html}    <!-- TAB 0: MATRACA (WEB REPL) -->
    <div id=\"tab-matraca\" class=\"tab-content${_act}\">
      <div class=\"card\" style=\"display:flex; flex-direction:column; height:calc(100vh - 220px); min-height:480px; padding:1.25rem;\">
        <div style=\"display:flex; justify-content:space-between; align-items:center; margin-bottom:0.75rem; flex-wrap:wrap; gap:0.5rem; border-bottom:1px solid var(--border); padding-bottom:0.75rem;\">
          <div style=\"display:flex; align-items:center; gap:0.6rem;\">
            <h2>🦜 Matraca — Web REPL Móvel</h2>
            <span id=\"matraca-conn-status\" style=\"font-size:0.75rem; background:rgba(35,134,54,0.15); color:var(--green); border:1px solid var(--green); padding:0.15rem 0.5rem; border-radius:12px; font-weight:600;\">● Online</span>
          </div>
          <div style=\"display:flex; gap:0.5rem; align-items:center;\">
            <button class=\"btn btn-secondary\" style=\"padding:0.25rem 0.6rem; font-size:0.8rem;\" onclick=\"loadMatracaHistory()\">Atualizar</button>
            <button class=\"btn btn-secondary\" style=\"padding:0.25rem 0.6rem; font-size:0.8rem;\" onclick=\"clearMatracaChat()\">Limpar Tela</button>
          </div>
        </div>

        <div id=\"matraca-chat\" style=\"flex:1; overflow-y:auto; padding:0.5rem; display:flex; flex-direction:column; gap:0.75rem; margin-bottom:0.75rem; scroll-behavior:smooth;\">
          <div style=\"text-align:center; color:var(--text-muted); font-size:0.85rem; padding:1.5rem 0;\">
            Conectado ao despachante do CCCP. Digite uma mensagem abaixo para interagir com o agente.
          </div>
        </div>

        <div style=\"display:flex; gap:0.5rem; align-items:flex-end;\">
          <textarea id=\"matraca-input\" rows=\"2\" placeholder=\"Digite uma instrução para o agy (Shift+Enter para quebra de linha, Enter para enviar)...\" style=\"margin-bottom:0; resize:none;\" onkeydown=\"handleMatracaKey(event)\"></textarea>
          <button id=\"btn-matraca-send\" class=\"btn\" style=\"height:54px; min-width:80px; display:flex; align-items:center; justify-content:center;\" onclick=\"sendMatraca()\">
            <span>➤ Enviar</span>
          </button>
        </div>
      </div>
    </div>
"
    fi

    if [ "$_enable_diva" -eq 1 ]; then
        _act=""
        [ "$_initial_tab" = "diva" ] && _act=" active"
        _nav_html="${_nav_html}    <button id=\"nav-diva\" class=\"${_act}\" onclick=\"switchTab('diva')\">🛋️ O Divã (Triage IA)</button>
"
        _main_html="${_main_html}    <!-- TAB 1: O DIVÃ -->
    <div id=\"tab-diva\" class=\"tab-content${_act}\">
      <div class=\"card\">
        <div style=\"display:flex; justify-content:space-between; align-items:center; margin-bottom:0.5rem; flex-wrap:wrap; gap:0.5rem;\">
          <h2>🛋️ O Divã — Triagem Semântica de Tickets</h2>
          <span id=\"diva-repo-hint\" style=\"font-family:var(--font-mono); font-size:0.85rem; color:var(--gold); font-weight:bold;\"></span>
        </div>
        <p class=\"desc\">Descreva livremente o problema, bug ou necessidade. A IA do Gosplan estruturará tickets atômicos seguindo Conventional Commits com sugestão de branch GOST.</p>
        
        <textarea id=\"diva-input\" rows=\"4\" placeholder=\"Ex: O botão de login congela quando o token JWT expira no meio de uma requisição assíncrona, exigindo reload da página...\"></textarea>
        
        <div style=\"display:flex; gap:0.5rem;\">
          <button id=\"btn-diva-run\" class=\"btn\" onclick=\"runDiva()\">
            <span>✦ Consultar Divã (IA)</span>
          </button>
          <button class=\"btn btn-secondary\" onclick=\"document.getElementById('diva-input').value = ''\">Limpar</button>
        </div>
      </div>

      <div id=\"diva-results\"></div>
    </div>
"
    fi

    if [ "$_enable_status" -eq 1 ]; then
        _act=""
        [ "$_initial_tab" = "status" ] && _act=" active"
        _nav_html="${_nav_html}    <button id=\"nav-status\" class=\"${_act}\" onclick=\"switchTab('status')\">📊 Inspeção do Repositório</button>
"
        _main_html="${_main_html}    <!-- TAB 2: STATUS -->
    <div id=\"tab-status\" class=\"tab-content${_act}\">
      <div class=\"card\">
        <div style=\"display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;\">
          <h2>📊 Inspeção Gosplan do Repositório</h2>
          <button class=\"btn btn-secondary\" onclick=\"loadStatus()\">Atualizar Status</button>
        </div>
        <pre id=\"status-output\" class=\"terminal\">Carregando status do repositório...</pre>
      </div>
    </div>
"
    fi

    if [ "$_enable_tickets" -eq 1 ]; then
        _act=""
        [ "$_initial_tab" = "tickets" ] && _act=" active"
        _nav_html="${_nav_html}    <button id=\"nav-tickets\" class=\"${_act}\" onclick=\"switchTab('tickets')\">📋 Tickets Ativos</button>
"
        _main_html="${_main_html}    <!-- TAB 3: TICKETS -->
    <div id=\"tab-tickets\" class=\"tab-content${_act}\">
      <div class=\"card\">
        <div style=\"display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;\">
          <h2>📋 Tickets Ativos do Projeto</h2>
          <button class=\"btn btn-secondary\" onclick=\"loadTickets()\">Sincronizar com Forge</button>
        </div>
        <pre id=\"tickets-output\" class=\"terminal\">Carregando tickets...</pre>
      </div>
    </div>
"
    fi

    mkdir -p "$_out_dir"
    chmod 700 "$_out_dir" 2>/dev/null || true

    cat > "$_out_dir/index.html" << EOF
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CCCP Web Cockpit — Радиостанция «Маяк»</title>
  <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>☭</text></svg>">
  <style>
    :root {
      --bg: #0d1117;
      --surface: #161b22;
      --surface-hover: #1f242c;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --text-bright: #f0f6fc;
      --red: #da3633;
      --red-hover: #b62324;
      --gold: #f1e05a;
      --green: #238636;
      --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      line-height: 1.5;
      padding-bottom: 4rem;
    }
    header {
      background: var(--surface);
      border-bottom: 1px solid var(--border);
      padding: 1rem 1.5rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      font-weight: 700;
      font-size: 1.25rem;
      color: var(--text-bright);
    }
    .brand span.badge {
      background: var(--red);
      color: #fff;
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      letter-spacing: 0.5px;
    }
    .session-info {
      font-family: var(--font-mono);
      font-size: 0.8rem;
      color: var(--text-muted);
      background: var(--bg);
      padding: 0.35rem 0.75rem;
      border: 1px solid var(--border);
      border-radius: 6px;
    }
    nav {
      background: #11151c;
      border-bottom: 1px solid var(--border);
      display: flex;
      padding: 0 1.5rem;
      gap: 0.5rem;
    }
    nav button {
      background: none;
      border: none;
      color: var(--text-muted);
      font-size: 0.95rem;
      font-weight: 600;
      padding: 0.85rem 1.25rem;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      transition: all 0.2s;
    }
    nav button:hover {
      color: var(--text-bright);
    }
    nav button.active {
      color: var(--gold);
      border-bottom-color: var(--gold);
    }
    main {
      max-width: 1000px;
      margin: 2rem auto;
      padding: 0 1.5rem;
    }
    .tab-content { display: none; }
    .tab-content.active { display: block; }
    .card {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.5rem;
      margin-bottom: 1.5rem;
    }
    h2 {
      font-size: 1.3rem;
      color: var(--text-bright);
      margin-bottom: 0.5rem;
    }
    p.desc {
      color: var(--text-muted);
      font-size: 0.9rem;
      margin-bottom: 1.25rem;
    }
    textarea, input[type="text"], input[type="password"], select {
      width: 100%;
      background: var(--bg);
      border: 1px solid var(--border);
      color: var(--text-bright);
      border-radius: 6px;
      padding: 0.75rem;
      font-size: 0.95rem;
      font-family: inherit;
      margin-bottom: 1rem;
    }
    textarea:focus, input[type="text"]:focus, input[type="password"]:focus, select:focus {
      outline: none;
      border-color: var(--gold);
    }
    .btn {
      background: var(--red);
      color: #fff;
      border: none;
      padding: 0.65rem 1.25rem;
      border-radius: 6px;
      font-weight: 600;
      font-size: 0.95rem;
      cursor: pointer;
      transition: background 0.2s;
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
    }
    .btn:hover { background: var(--red-hover); }
    .btn-secondary {
      background: var(--surface-hover);
      color: var(--text);
      border: 1px solid var(--border);
    }
    .btn-secondary:hover {
      background: #282e38;
      color: var(--text-bright);
    }
    .spinner {
      display: inline-block;
      width: 1rem;
      height: 1rem;
      border: 2px solid rgba(255,255,255,0.3);
      border-radius: 50%;
      border-top-color: #fff;
      animation: spin 0.8s linear infinite;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    pre.terminal {
      background: #05080c;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 1rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
      color: #3fb950;
      overflow-x: auto;
      white-space: pre-wrap;
      word-break: break-all;
    }
    .ticket-card {
      background: var(--bg);
      border: 1px solid var(--border);
      border-left: 4px solid var(--gold);
      border-radius: 6px;
      padding: 1rem;
      margin-bottom: 1rem;
    }
    .ticket-header {
      display: flex;
      justify-content: space-between;
      margin-bottom: 0.5rem;
    }
    .badge-type {
      font-family: var(--font-mono);
      font-size: 0.75rem;
      font-weight: bold;
      background: #238636;
      color: #fff;
      padding: 0.15rem 0.5rem;
      border-radius: 4px;
    }
    .branch-hint {
      background: #1c2128;
      border: 1px dashed var(--border);
      border-radius: 6px;
      padding: 0.6rem 0.8rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
      color: var(--gold);
      margin-top: 0.75rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .copy-btn {
      background: none;
      border: 1px solid var(--border);
      color: var(--text-muted);
      border-radius: 4px;
      padding: 0.2rem 0.5rem;
      cursor: pointer;
      font-size: 0.75rem;
    }
    .copy-btn:hover { color: var(--text-bright); border-color: var(--text-bright); }
  </style>
</head>
<body>

  <!-- MODAL DE AUTENTICAÇÃO -->
  <div id="auth-modal" style="display:none; position:fixed; inset:0; background:rgba(13,17,23,0.85); backdrop-filter:blur(4px); z-index:9999; align-items:center; justify-content:center;">
    <div class="card" style="width:100%; max-width:440px; margin:1rem; box-shadow:0 8px 24px rgba(0,0,0,0.6); border-color:var(--border);">
      <div style="text-align:center; margin-bottom:1.5rem;">
        <span style="font-size:2.5rem;">🔐</span>
        <h2 style="margin-top:0.5rem; font-size:1.3rem;">Autenticação Requerida</h2>
        <p class="desc" style="margin-bottom:0;">Informe a chave de API ou token do Gosplan para desbloquear o Cockpit.</p>
      </div>
      <form onsubmit="handleAuthSubmit(event)">
        <input type="password" id="auth-token-input" placeholder="Cole seu token aqui..." autocomplete="off" required style="font-family:var(--font-mono); letter-spacing:1px; margin-bottom:0.75rem;" />
        <div id="auth-error" style="display:none; color:var(--red); font-size:0.85rem; margin-bottom:0.75rem;"></div>
        <button type="submit" id="btn-auth-submit" class="btn" style="width:100%; justify-content:center;">
          <span>✦ Autenticar</span>
        </button>
      </form>
    </div>
  </div>

  <header>
    <div class="brand">
      <span>☭ CCCP Web Cockpit</span>
      <span class="badge">Маяк Bridge</span>
    </div>
    <div style="display:flex; align-items:center; gap:0.75rem; flex-wrap:wrap;">
      <div style="display:flex; align-items:center; gap:0.4rem;">
        <label for="repo-select" style="font-size:0.85rem; color:var(--text-muted); font-weight:600;">📁 Repositório:</label>
        <select id="repo-select" onchange="switchRepository(this.value)" style="margin-bottom:0; width:auto; min-width:180px; padding:0.35rem 0.6rem; font-size:0.85rem; height:32px;">
          <option value="">Carregando...</option>
        </select>
      </div>
      <div class="session-info">
        Port: <strong>${_listen_port}</strong>
      </div>
      <button id="btn-auth-lock" class="btn btn-secondary" style="padding:0.35rem 0.75rem; font-size:0.8rem; display:none;" onclick="logout()">
        🔒 Bloquear
      </button>
    </div>
  </header>

  <nav>
${_nav_html}  </nav>

  <main>
${_main_html}  </main>

  <script>
    let sessionToken = sessionStorage.getItem('cccp_token') || new URLSearchParams(window.location.search).get('token') || '';
    let activeRepo = sessionStorage.getItem('cccp_active_repo') || '';

    function showAuthModal(errMsg = '') {
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (errMsg) {
        errEl.textContent = errMsg;
        errEl.style.display = 'block';
      } else {
        errEl.style.display = 'none';
      }
      modal.style.display = 'flex';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'none';
      setTimeout(() => {
        const inp = document.getElementById('auth-token-input');
        if (inp) inp.focus();
      }, 50);
    }

    function hideAuthModal() {
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (modal) modal.style.display = 'none';
      if (errEl) errEl.style.display = 'none';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'inline-flex';
    }

    function logout() {
      sessionStorage.removeItem('cccp_token');
      sessionToken = '';
      const url = new URL(window.location);
      if (url.searchParams.has('token')) {
        url.searchParams.delete('token');
        window.history.replaceState({}, '', url.pathname + url.search);
      }
      showAuthModal('Sessão bloqueada.');
    }

    async function handleAuthSubmit(e) {
      e.preventDefault();
      const input = document.getElementById('auth-token-input');
      const val = input.value.trim();
      if (!val) return;

      const btn = document.getElementById('btn-auth-submit');
      btn.disabled = true;
      btn.innerHTML = '<span class="spinner"></span> Validando...';

      try {
        const res = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + val,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });

        if (res.status === 401) {
          showAuthModal('Chave ou token inválido. Tente novamente.');
          input.value = '';
          input.focus();
          return;
        }

        sessionToken = val;
        sessionStorage.setItem('cccp_token', sessionToken);
        hideAuthModal();
        await loadRepositories();
        const activeTab = document.querySelector('nav button.active');
        const tabName = activeTab ? activeTab.id.replace('nav-', '') : 'diva';
        switchTab(tabName);
      } catch (err) {
        showAuthModal('Erro ao conectar ao Mayak: ' + err.message);
      } finally {
        btn.disabled = false;
        btn.innerHTML = '<span>✦ Autenticar</span>';
      }
    }

    async function apiExec(args, stdin = null) {
      if (!sessionToken) {
        showAuthModal('Informe a chave de API para continuar.');
        throw new Error('Não autenticado');
      }

      let finalArgs = args;
      if (activeRepo && args.length > 0 && args[0] !== '-C' && args[0] !== 'catalog') {
        finalArgs = ['-C', activeRepo, ...args];
      }

      const res = await fetch('/api/exec', {
        method: 'POST',
        headers: {
          'Authorization': 'Bearer ' + sessionToken,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ args: finalArgs, stdin })
      });

      if (res.status === 401) {
        logout();
        showAuthModal('Sessão expirada ou chave inválida.');
        throw new Error('401 Unauthorized');
      }

      if (!res.ok) {
        throw new Error('HTTP ' + res.status + ': ' + (await res.text()));
      }
      return await res.json();
    }

    async function loadRepositories() {
      const select = document.getElementById('repo-select');
      if (!select) return;

      let initialRepo = '';
      try {
        const statusResp = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });
        if (statusResp.ok) {
          const data = await statusResp.json();
          const match = (data.stdout || '').match(/Repository:\s+([^\r\n]+)/);
          if (match && match[1]) {
            initialRepo = match[1].trim().split('/').pop();
          }
        }
      } catch (e) {}

      let repos = [];
      try {
        const catRes = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['catalog', 'list', '--json'] })
        });
        if (catRes.ok) {
          const data = await catRes.json();
          repos = JSON.parse(data.stdout || '[]');
        }
      } catch (e) {}

      select.innerHTML = '';

      if (!activeRepo) {
        activeRepo = initialRepo;
        if (activeRepo) {
          sessionStorage.setItem('cccp_active_repo', activeRepo);
        }
      }

      const seen = new Set();
      let hasActiveInList = false;

      if (repos && repos.length > 0) {
        repos.forEach(r => {
          if (r.name && !seen.has(r.name)) {
            seen.add(r.name);
            const opt = document.createElement('option');
            opt.value = r.name;
            opt.textContent = r.name + ' (' + (r.forge || 'local') + ')';
            if (r.name === activeRepo) {
              opt.selected = true;
              hasActiveInList = true;
            }
            select.appendChild(opt);
          }
        });
      }

      if (activeRepo && !hasActiveInList) {
        const opt = document.createElement('option');
        opt.value = activeRepo;
        opt.textContent = activeRepo + ' (ativo)';
        opt.selected = true;
        select.insertBefore(opt, select.firstChild);
      } else if (!activeRepo && select.options.length > 0) {
        activeRepo = select.options[0].value;
        sessionStorage.setItem('cccp_active_repo', activeRepo);
      }

      updateDivaContext();
    }

    function switchRepository(repoName) {
      activeRepo = repoName;
      sessionStorage.setItem('cccp_active_repo', activeRepo);
      updateDivaContext();

      const activeTab = document.querySelector('nav button.active');
      const tabName = activeTab ? activeTab.id.replace('nav-', '') : 'diva';

      if (tabName === 'status') {
        loadStatus();
      } else if (tabName === 'tickets') {
        loadTickets();
      }
    }

    function updateDivaContext() {
      const hint = document.getElementById('diva-repo-hint');
      if (hint) {
        hint.textContent = activeRepo ? '[Repo: ' + activeRepo + ']' : '';
      }
    }

    function switchTab(name) {
      document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('nav button').forEach(el => el.classList.remove('active'));
      const targetTab = document.getElementById('tab-' + name);
      const targetNav = document.getElementById('nav-' + name);
      if (targetTab) targetTab.classList.add('active');
      if (targetNav) targetNav.classList.add('active');

      if (name === 'status') loadStatus();
      if (name === 'tickets') loadTickets();
      if (name === 'matraca') loadMatracaHistory();
    }

    const activePolls = new Set();

    function formatTime(epoch) {
      if (!epoch) return '';
      const d = new Date(epoch * 1000);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }

    function renderMatracaMessage(msg, isUser, status = 'done') {
      const chat = document.getElementById('matraca-chat');
      const bubble = document.createElement('div');
      bubble.style.maxWidth = '85%';
      bubble.style.padding = '0.75rem 1rem';
      bubble.style.borderRadius = isUser ? '12px 12px 2px 12px' : '12px 12px 12px 2px';
      bubble.style.background = isUser ? '#1f6feb' : '#21262d';
      bubble.style.color = isUser ? '#ffffff' : 'var(--text-bright)';
      bubble.style.border = isUser ? 'none' : '1px solid var(--border)';
      bubble.style.alignSelf = isUser ? 'flex-end' : 'flex-start';
      bubble.style.wordBreak = 'break-word';
      bubble.style.fontSize = '0.92rem';
      bubble.style.boxShadow = '0 1px 3px rgba(0,0,0,0.2)';

      let badgeHtml = '';
      if (status === 'pending') {
        badgeHtml = '<span style="color:var(--gold); font-size:0.75rem; margin-left:6px;">⏳ Na fila...</span>';
      } else if (status === 'processing') {
        badgeHtml = '<span style="color:#58a6ff; font-size:0.75rem; margin-left:6px;">⚙ Processando...</span>';
      } else if (status === 'error') {
        badgeHtml = '<span style="color:var(--red); font-size:0.75rem; margin-left:6px;">✖ Erro</span>';
      }

      const timeStr = msg.created_at ? formatTime(msg.created_at) : '';
      const content = (msg.response || msg.prompt || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

      bubble.innerHTML = '<div style="white-space:pre-wrap; font-family:' + (isUser ? 'inherit' : 'var(--font-mono)') + ';">' + content + '</div>' +
        '<div style="display:flex; justify-content:flex-end; align-items:center; margin-top:4px; font-size:0.7rem; opacity:0.7;">' +
        timeStr + badgeHtml +
        '</div>';

      if (msg.id) bubble.id = 'matraca-msg-' + msg.id;
      chat.appendChild(bubble);
      chat.scrollTop = chat.scrollHeight;
      return bubble;
    }

    async function loadMatracaHistory() {
      const chat = document.getElementById('matraca-chat');
      try {
        const resp = await apiExec(['matraca', 'history', '--limit', '30']);
        if (resp.ok && resp.stdout) {
          const data = JSON.parse(resp.stdout);
          if (data && data.messages && data.messages.length > 0) {
            chat.innerHTML = '';
            const sorted = data.messages.sort((a,b) => (a.created_at || 0) - (b.created_at || 0));
            for (const m of sorted) {
              if (m.prompt) {
                renderMatracaMessage({ prompt: m.prompt, created_at: m.created_at }, true);
              }
              if (m.response || m.status) {
                renderMatracaMessage({ response: m.response, created_at: m.updated_at || m.created_at, id: m.id }, false, m.status);
              }
            }
          }
        }
      } catch (err) {
        console.error('Erro ao carregar histórico do Matraca:', err);
      }
    }

    function clearMatracaChat() {
      document.getElementById('matraca-chat').innerHTML = '<div style="text-align:center; color:var(--text-muted); font-size:0.85rem; padding:1.5rem 0;">Tela limpa. Fila continua salva no sistema de arquivos.</div>';
    }

    function handleMatracaKey(e) {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMatraca();
      }
    }

    async function sendMatraca() {
      const input = document.getElementById('matraca-input');
      const text = input.value.trim();
      if (!text) return;

      const clientId = 'cli_' + Date.now() + '_' + Math.random().toString(36).substring(2, 8);
      input.value = '';

      renderMatracaMessage({ prompt: text, created_at: Math.floor(Date.now() / 1000) }, true);
      const botMsg = renderMatracaMessage({ response: 'Aguardando agente...', created_at: Math.floor(Date.now() / 1000) }, false, 'pending');

      const btn = document.getElementById('btn-matraca-send');
      btn.disabled = true;

      try {
        const resp = await apiExec(['matraca', 'send', '--client-id', clientId, text]);
        btn.disabled = false;

        let jobId = '';
        if (resp.stdout) {
          try {
            const parsed = JSON.parse(resp.stdout);
            jobId = parsed.id;
          } catch(e) {}
        }

        if (!jobId) {
          botMsg.querySelector('div').textContent = 'Erro ao enfileirar: ' + (resp.stderr || 'Resposta inválida');
          return;
        }

        pollMatraca(jobId, botMsg);
      } catch (err) {
        btn.disabled = false;
        botMsg.querySelector('div').textContent = 'Erro de rede: ' + err.message;
      }
    }

    function pollMatraca(jobId, bubbleEl) {
      if (activePolls.has(jobId)) return;
      activePolls.add(jobId);

      const statusPill = document.getElementById('matraca-conn-status');

      const check = async () => {
        try {
          const resp = await apiExec(['matraca', 'poll', jobId]);
          if (statusPill) {
            statusPill.textContent = '● Online';
            statusPill.style.color = 'var(--green)';
          }

          if (resp.stdout) {
            const data = JSON.parse(resp.stdout);
            if (data.status === 'done' || data.status === 'error') {
              activePolls.delete(jobId);
              const content = (data.response || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
              bubbleEl.querySelector('div').innerHTML = content;
              const metaDiv = bubbleEl.querySelectorAll('div')[1];
              if (metaDiv) {
                const badge = data.status === 'done' ? '<span style="color:var(--green); font-size:0.75rem; margin-left:6px;">✔ Concluído</span>' : '<span style="color:var(--red); font-size:0.75rem; margin-left:6px;">✖ Erro</span>';
                metaDiv.innerHTML = formatTime(data.updated_at || data.created_at) + badge;
              }
              const chat = document.getElementById('matraca-chat');
              chat.scrollTop = chat.scrollHeight;
              return;
            } else if (data.status === 'processing') {
              const metaDiv = bubbleEl.querySelectorAll('div')[1];
              if (metaDiv) {
                metaDiv.innerHTML = formatTime(data.created_at) + '<span style="color:#58a6ff; font-size:0.75rem; margin-left:6px;">⚙ Executando...</span>';
              }
            }
          }
        } catch (err) {
          if (statusPill) {
            statusPill.textContent = '▲ Reconectando...';
            statusPill.style.color = 'var(--gold)';
          }
        }

        setTimeout(check, 2000);
      };

      setTimeout(check, 1000);
    }

    async function runDiva() {
      const text = document.getElementById('diva-input').value.trim();
      if (!text) return alert('Por favor, informe a descrição do problema.');

      const btn = document.getElementById('btn-diva-run');
      const container = document.getElementById('diva-results');
      btn.disabled = true;
      btn.innerHTML = '<span class="spinner"></span> Analisando com o Divã...';
      container.innerHTML = '<p style="color:var(--gold);">✦ Consultando o Gosplan & IA...</p>';

      try {
        const resp = await apiExec(['diva', '--dry-run', text]);
        if (!resp.ok && resp.exit_code !== 0) {
          container.innerHTML = '<pre class="terminal" style="color:var(--red);">' + (resp.stderr || resp.stdout) + '</pre>';
          return;
        }

        container.innerHTML = '<div class="card"><h2>Resultado da Triagem</h2><pre class="terminal">' + (resp.stdout || 'Nenhum ticket retornado.') + '</pre></div>';
      } catch (err) {
        container.innerHTML = '<pre class="terminal" style="color:var(--red);">Erro: ' + err.message + '</pre>';
      } finally {
        btn.disabled = false;
        btn.innerHTML = '<span>✦ Consultar Divã (IA)</span>';
      }
    }

    async function loadStatus() {
      const el = document.getElementById('status-output');
      el.textContent = 'Carregando status do repositório...';
      try {
        const resp = await apiExec(['status']);
        el.textContent = resp.stdout || resp.stderr || 'Nenhum retorno recebido.';
      } catch (err) {
        el.textContent = 'Erro ao consultar status: ' + err.message;
      }
    }

    async function loadTickets() {
      const el = document.getElementById('tickets-output');
      el.textContent = 'Consultando tickets sincronizados...';
      try {
        const resp = await apiExec(['tickets']);
        el.textContent = resp.stdout || resp.stderr || 'Nenhum ticket encontrado.';
      } catch (err) {
        el.textContent = 'Erro ao consultar tickets: ' + err.message;
      }
    }

    async function initCockpit() {
      if (!sessionToken) {
        showAuthModal();
        return;
      }
      try {
        const res = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });
        if (res.status === 401) {
          logout();
          return;
        }
        sessionStorage.setItem('cccp_token', sessionToken);
        hideAuthModal();
        await loadRepositories();
        const urlTab = new URLSearchParams(window.location.search).get('tab');
        if (urlTab && document.getElementById('tab-' + urlTab)) {
          switchTab(urlTab);
        } else {
          switchTab('${_initial_tab}');
        }
      } catch (err) {
        hideAuthModal();
      }
    }

    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', initCockpit);
    } else {
      initCockpit();
    }
  </script>
</body>
</html>
EOF
}

cmd_cockpit() {
    _port=8080
    _host="127.0.0.1"
    _open_browser=0
    _quiet=0
    _initial_tab="matraca"
    _cli_token=""
    _enable_matraca=1
    _enable_diva=1
    _enable_status=1
    _enable_tickets=1

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                echo "cccp cockpit - Gosplan Interactive Web Cockpit & Mayak Command Bridge"
                echo "====================================================================="
                echo ""
                echo "Usage:"
                echo "  cccp cockpit [options] [tab]"
                echo "  cccp web [options] [tab]"
                echo ""
                echo "Options:"
                echo "  -p, --port <port>       Port to listen on (default: 8080, auto-increments if busy)"
                echo "  --host <ip>             Host address to bind to (default: 127.0.0.1)"
                echo "  -o, --open              Open default web browser automatically (opt-in)"
                echo "  -t, --token <token>     API token for authentication (or via CCCP_TOKEN/CCCP_API_KEY)"
                echo "  --key, --api-key <k>    Alias for --token"
                echo "  --no-open               Do not open web browser (default behavior)"
                echo "  --no-matraca            Disable Matraca (Web REPL) tab"
                echo "  --no-diva               Disable O Divã (triage bot) tab"
                echo "  --no-status             Disable Repository Status tab"
                echo "  --no-tickets            Disable Active Tickets tab"
                echo "  -q, --quiet             Quiet mode (suppress verbose logs)"
                echo "  -h, --help              Show this help message"
                echo ""
                echo "Tabs:"
                echo "  matraca                 Open directly on Matraca mobile Web REPL (default)"
                echo "  diva                    Open directly on Diva ticket triage cockpit"
                echo "  status                  Open directly on repository inspection"
                echo "  tickets                 Open directly on active tickets list"
                return 0
                ;;
            -p|--port)
                shift
                [ $# -gt 0 ] || { echo "Error: --port requires a port argument" >&2; return 1; }
                _port="$1"
                ;;
            --host)
                shift
                [ $# -gt 0 ] || { echo "Error: --host requires an IP address argument" >&2; return 1; }
                _host="$1"
                ;;
            -o|--open)
                _open_browser=1
                ;;
            --no-open)
                _open_browser=0
                ;;
            --no-matraca)
                _enable_matraca=0
                ;;
            --no-diva)
                _enable_diva=0
                ;;
            --no-status)
                _enable_status=0
                ;;
            --no-tickets)
                _enable_tickets=0
                ;;
            -t|--token|--key|--api-key)
                shift
                [ $# -gt 0 ] || { echo "Error: $1 requires a token argument" >&2; return 1; }
                _cli_token="$1"
                ;;
            -q|--quiet)
                _quiet=1
                ;;
            diva|status|tickets|matraca)
                _initial_tab="$1"
                ;;
            *)
                echo "Error: Unknown option or argument '$1'. Use --help for usage." >&2
                return 1
                ;;
        esac
        shift
    done

    # Verify at least one tab is enabled
    if [ "$_enable_matraca" -eq 0 ] && [ "$_enable_diva" -eq 0 ] && [ "$_enable_status" -eq 0 ] && [ "$_enable_tickets" -eq 0 ]; then
        echo "Error: At least one tab must be enabled in cockpit." >&2
        return 1
    fi

    # Fallback initial tab if chosen tab is disabled
    case "$_initial_tab" in
        matraca)
            if [ "$_enable_matraca" -eq 0 ]; then
                if [ "$_enable_diva" -eq 1 ]; then _initial_tab="diva"
                elif [ "$_enable_status" -eq 1 ]; then _initial_tab="status"
                elif [ "$_enable_tickets" -eq 1 ]; then _initial_tab="tickets"
                fi
            fi
            ;;
        diva)
            if [ "$_enable_diva" -eq 0 ]; then
                if [ "$_enable_matraca" -eq 1 ]; then _initial_tab="matraca"
                elif [ "$_enable_status" -eq 1 ]; then _initial_tab="status"
                elif [ "$_enable_tickets" -eq 1 ]; then _initial_tab="tickets"
                fi
            fi
            ;;
        status)
            if [ "$_enable_status" -eq 0 ]; then
                if [ "$_enable_matraca" -eq 1 ]; then _initial_tab="matraca"
                elif [ "$_enable_diva" -eq 1 ]; then _initial_tab="diva"
                elif [ "$_enable_tickets" -eq 1 ]; then _initial_tab="tickets"
                fi
            fi
            ;;
        tickets)
            if [ "$_enable_tickets" -eq 0 ]; then
                if [ "$_enable_matraca" -eq 1 ]; then _initial_tab="matraca"
                elif [ "$_enable_diva" -eq 1 ]; then _initial_tab="diva"
                elif [ "$_enable_status" -eq 1 ]; then _initial_tab="status"
                fi
            fi
            ;;
    esac

    # 1. Resolve Mayak comrade binary
    _mk_bin=$(command -v mk 2>/dev/null || command -v mayak 2>/dev/null || true)
    if [ -z "$_mk_bin" ]; then
        if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
            _cccp_ensure_comrade "mk" "mayak" || {
                echo "Error: Failed to obtain Comrade Mayak (mk). Install it or place in PATH." >&2
                return 1
            }
            _mk_bin=$(command -v mk 2>/dev/null || command -v mayak 2>/dev/null || true)
        fi
    fi

    if [ -z "$_mk_bin" ]; then
        echo "Error: Comrade Mayak (mk) is required to run the Web Cockpit." >&2
        echo "Install it via 'cccp comrades' or compile from https://codeberg.org/lumenpink/mayak." >&2
        return 1
    fi

    # 2. Resolve CCCP executable for bridge
    _cccp_bin=$(command -v cccp 2>/dev/null || true)
    if [ -z "$_cccp_bin" ]; then
        _cccp_bin="$0"
    fi

    # 3. Resolve session token
    _token="${_cli_token:-${CCCP_TOKEN:-${CCCP_API_KEY:-}}}"
    if [ -z "$_token" ]; then
        _token=$(_web_generate_token)
    fi

    # 4. Prepare temporary assets
    _uid=$(id -u 2>/dev/null || echo 1000)
    _web_dir="${TMPDIR:-/tmp}/cccp-web-${_uid}-$$"
    _web_export_assets "$_web_dir" "$_token" "$_port" "$_enable_matraca" "$_enable_diva" "$_enable_status" "$_enable_tickets" "$_initial_tab"

    trap 'rm -rf "$_web_dir" 2>/dev/null || true' EXIT INT TERM

    # 5. Build Mayak arguments
    set -- "$_web_dir" -p "$_port" --host "$_host" --bridge "$_cccp_bin" --auth "$_token" --cors
    if [ "$_open_browser" -eq 1 ]; then
        set -- "$@" --open
    fi
    if [ "$_quiet" -eq 1 ]; then
        set -- "$@" --quiet
    fi

    if [ "$_quiet" -eq 0 ]; then
        echo "★ Iniciando CCCP Web Cockpit via Comrade Mayak (mk) ★"
        echo "  🔑 Chave / Token: $_token"
        echo "  🌐 URL:           http://${_host}:${_port}/?tab=${_initial_tab}"
        if [ "$_open_browser" -eq 1 ]; then
            echo "  🚀 Navegador:     abertura automática acionada (--open)"
        fi
    fi

    "$_mk_bin" "$@"
}

cmd_web() {
    cmd_cockpit "$@"
}


# =============================================================================
# Hooks Functions
# =============================================================================
install_global_binary() {
    # 1. Determine target destination based on EUID / uid
    if [ -n "${CCCP_INSTALL_DIR:-}" ]; then
        target_dir="$CCCP_INSTALL_DIR"
    elif [ -n "${XDG_BIN_HOME:-}" ]; then
        target_dir="$XDG_BIN_HOME"
    else
        user_id=$(id -u 2>/dev/null || echo "1000")
        if [ "$user_id" -eq 0 ]; then
            target_dir="/usr/local/bin"
        elif [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
            target_dir="$HOME/.local/bin"
        else
            target_dir="${XDG_DATA_HOME:-$HOME/.local/share}/cccp/bin"
        fi
    fi

    mkdir -p "$target_dir"
    target_bin="$target_dir/cccp"

    # 2. Determine source file
    source_bin=""
    if [ -n "${1:-}" ] && [ -f "$1" ]; then
        source_bin="$1"
    elif [ -f "./cccp.sh" ]; then
        source_bin="./cccp.sh"
    elif [ -n "${GIT_ROOT:-}" ] && [ -f "$GIT_ROOT/cccp.sh" ]; then
        source_bin="$GIT_ROOT/cccp.sh"
    elif [ -f "${0:-}" ] && [ -s "${0:-}" ]; then
        source_bin="${0:-}"
    fi

    # If source is already the target binary, it's already installed
    if [ -n "$source_bin" ] && [ -f "$source_bin" ]; then
        _src_real=$(cd "$(dirname "$source_bin")" 2>/dev/null && pwd)/$(basename "$source_bin")
        _tgt_real=$(cd "$target_dir" 2>/dev/null && pwd)/cccp
        if [ "$_src_real" = "$_tgt_real" ]; then
            chmod +x "$target_bin"
            echo "cccp is already up to date at: $target_bin"
            return 0
        fi
    fi

    if [ -z "$source_bin" ] || [ ! -f "$source_bin" ]; then
        tmp_download="$(mktemp)"
        update_url="${UPDATE_URL:-https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh}"
        if command -v _cccp_http >/dev/null 2>&1; then
            _cccp_http -fsSL "$update_url" -o "$tmp_download" 2>/dev/null || true
        elif command -v sv >/dev/null 2>&1; then
            sv -fsSL "$update_url" -o "$tmp_download" 2>/dev/null || true
        elif command -v svyaz >/dev/null 2>&1; then
            svyaz -fsSL "$update_url" -o "$tmp_download" 2>/dev/null || true
        elif command -v curl >/dev/null 2>&1; then
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
            rm -f "$hook_path.old."*
            mv -f "$hook_path" "$backup_name"
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

    # Execute Act local CI gate if available
    if command -v act_ci_gate >/dev/null 2>&1; then
        act_ci_gate "${1:-origin}" "${2:-}" || return 1
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

is_post_push_sync_enabled() {
    _target="$1"
    case "$_target" in
        ticket|tickets)
            case "${POST_PUSH_SYNC_TICKETS:-0}" in
                1|true|TRUE|yes|YES|on|ON) return 0 ;;
            esac
            _matches="ticket tickets sync-ticket sync-tickets"
            ;;
        agent|agents)
            case "${POST_PUSH_SYNC_AGENT:-${POST_PUSH_SYNC_AGENTS:-0}}" in
                1|true|TRUE|yes|YES|on|ON) return 0 ;;
            esac
            _matches="agent agents sync-agent sync-agents"
            ;;
        *)
            return 1
            ;;
    esac

    for action in $(printf '%s' "${POST_PUSH_SYNC:-}" | tr ',' ' '); do
        case " $_matches " in
            *" $action "*) return 0 ;;
        esac
    done
    return 1
}

is_post_push_sync_tickets_enabled() {
    is_post_push_sync_enabled tickets
}

is_post_push_sync_agent_enabled() {
    is_post_push_sync_enabled agent
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
        *fetch*|*pull*|*clone*|*fast-import*|*filter-repo*)
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
            echo "cccp.sh tickets - Synchronize Issues into Markdown (GitHub / Codeberg)"
            echo "======================================================================="
            echo ""
            echo "Fetches open issues from GitHub or Codeberg API dynamically and formats them into a clean"
            echo "markdown document with a strict byte ceiling (10KB default), strictly avoiding"
            echo "truncating tickets mid-text. Protected by Gosplan rate limiter (55 requests/hour)."
            echo ""
            echo "Usage:"
            echo "  $0 tickets [options]"
            echo "  $0 sync-tickets [options]"
            echo ""
            echo "Options:"
            echo "  -f, --file <path>        Output markdown file (default: .antigravity/tickets.md)"
            echo "  -o, --owner <owner>      Repository owner/organization"
            echo "  -r, --repo <repo>        Repository name"
            echo "  -p, --provider <name>    Provider: github or codeberg (auto-detected by default)"
            echo "  -m, --max-bytes <num>    Maximum budget in bytes (default: 10240)"
            echo "  -t, --token <token>      Personal access token (optional, or via TICKETS_TOKEN/GITHUB_TOKEN/CODEBERG_TOKEN)"
            echo ""
            echo "Dynamic Repository Discovery:"
            echo "  1. CLI options (--owner, --repo) and environment variables (TICKETS_OWNER, TICKETS_REPO)"
            echo "  2. Git remote URL inspection (origin or other remotes) supporting SSH and HTTPS"
            echo "  3. Graceful fallback: If remote is not supported or network fails, logs to stderr and exits 0"
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
        "catalog")
            echo "cccp.sh catalog - Gosplan Workspace Inventory & Forge Synchronizer"
            echo "=================================================================="
            echo ""
            echo "Inventories git repositories across workspaces, extracts metadata,"
            echo "updates .git/description, and synchronizes with remote forges."
            echo ""
            echo "Usage:"
            echo "  $0 catalog [subcommand] [options]"
            echo ""
            echo "Subcommands:"
            echo "  list                      List cataloged repositories with metadata (default)"
            echo "  sync-local, local         Scan workspace, update .git/description, generate catalog.json & README.md"
            echo "  sync-remote, remote       Synchronize metadata with remote forges (Codeberg, TildeGit, GitHub)"
            echo "  all                       Execute sync-local and sync-remote sequentially"
            echo ""
            echo "Options:"
            echo "  --dir, -d <path>          Target workspace root directory (precedence: Flag > Env > Global Config > ~/src)"
            echo "  --json                    Output catalog directly in JSON format (with list)"
            echo "  -h, --help                Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 catalog"
            echo "  $0 catalog list --dir /home/user/src"
            echo "  $0 catalog sync-local"
            echo "  $0 catalog sync-remote"
            ;;
        "diva"|"divan"|"ticket"|"issue")
            echo "cccp.sh diva/ticket - Gemini-Powered Interactive Ticket Triage Bot (O Divã)"
            echo "=========================================================================="
            echo ""
            echo "Decomposes freeform problem reports, bug descriptions, and feature ideas"
            echo "into discrete Conventional Commits issues via Google Gemini REST API,"
            echo "creating tickets on forge with automated GOST git branch suggestions."
            echo ""
            echo "Usage:"
            echo "  $0 ticket [options] [problem-description]"
            echo "  $0 diva [options] [problem-description]"
            echo "  echo 'description' | $0 ticket"
            echo ""
            echo "Options:"
            echo "  --web, -w                 Open interactive triage cockpit in web browser"
            echo "  --dry-run                 Simulate triage and preview tickets without forge publication"
            echo "  --model, -m <model>       Gemini model name (default: gemini-3.6-flash)"
            echo "  --temp, -t <val>          Sampling temperature (default: 0.2)"
            echo "  --repo, -r <owner/repo>   Target repository slug (default: detected from git origin)"
            echo "  config <api-key>          Persist GEMINI_API_KEY in .cccprc"
            echo "  -h, --help                Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 ticket"
            echo "  $0 ticket 'Token expiration leads to UI lock and crash on refresh'"
            echo "  $0 diva --web"
            echo "  $0 diva 'Token expiration leads to UI lock and crash on refresh'"
            echo "  $0 diva --dry-run 'Various bug reports from support inbox...'"
            echo "  $0 diva config AIzaSy..."
            ;;
        "cockpit"|"web")
            echo "cccp.sh cockpit - Gosplan Interactive Web Cockpit & Mayak Command Bridge"
            echo "====================================================================="
            echo ""
            echo "Launches an interactive, zero-overhead web interface powered by Mayak (mk)"
            echo "acting as a local authenticated Command Bridge for Matraca REPL, Diva triage, and status."
            echo ""
            echo "Usage:"
            echo "  $0 cockpit [options] [tab]"
            echo "  $0 web [options] [tab]"
            echo ""
            echo "Options:"
            echo "  -p, --port <port>       Port to listen on (default: 8080, auto-increments if busy)"
            echo "  --host <ip>             Host address to bind to (default: 127.0.0.1)"
            echo "  -o, --open              Open default web browser automatically (opt-in)"
            echo "  -t, --token <token>     API token for authentication (or via CCCP_TOKEN/CCCP_API_KEY)"
            echo "  --key, --api-key <k>    Alias for --token"
            echo "  --no-open               Do not open web browser (default behavior)"
            echo "  --no-matraca            Disable Matraca mobile REPL tab"
            echo "  --no-diva               Disable Diva ticket triage tab"
            echo "  --no-status             Disable repository status tab"
            echo "  --no-tickets            Disable active tickets tab"
            echo "  -q, --quiet             Quiet mode (suppress verbose logs)"
            echo "  -h, --help              Show this help message"
            echo ""
            echo "Tabs:"
            echo "  matraca                 Open directly on Matraca mobile REPL (default)"
            echo "  diva                    Open directly on Diva ticket triage cockpit"
            echo "  status                  Open directly on repository inspection"
            echo "  tickets                 Open directly on active tickets list"
            echo ""
            echo "Examples:"
            echo "  $0 cockpit"
            echo "  $0 cockpit --open"
            echo "  $0 cockpit --no-matraca"
            echo "  $0 cockpit diva -p 3000"
            echo "  $0 web --open"
            ;;
        "matraca")
            echo "cccp.sh matraca - Mobile & Offline-Tolerant Web REPL for Antigravity (agy)"
            echo "=========================================================================="
            echo ""
            echo "Provides a resilient, filesystem-based FIFO queue and mobile Web REPL"
            echo "for Antigravity CLI (agy) that functions reliably even over unstable mobile networks."
            echo ""
            echo "Usage:"
            echo "  $0 matraca [send|poll|worker|history|status|clean] [options]"
            echo ""
            echo "Subcommands:"
            echo "  send [--client-id <id>] [--workspace <dir>] <prompt...>"
            echo "                         Enqueue prompt with idempotency and immediate ack (<10ms)"
            echo "  poll <job_id|client_id> Query execution status and retrieve agent response (<1ms)"
            echo "  worker [--daemon|--once] Process pending jobs from queue via Antigravity CLI (agy)"
            echo "  history [--limit <N>]   List recent completed and active interactions"
            echo "  status [--json]        Inspect queue metrics (pending, processing, done, error)"
            echo "  clean [--days <N>]     Purge jobs older than N days (default: 7)"
            echo ""
            echo "Examples:"
            echo "  $0 matraca send 'List all git branches and uncommitted files'"
            echo "  $0 matraca poll 1727438100_abc123_4567"
            echo "  $0 matraca worker --daemon"
            echo "  $0 cockpit --open"
            ;;
        "comrades"|"camaradas"|"comrade")
            echo "cccp.sh comrades - Gosplan Micro Tools Manager (Kvant & Svyaz)"
            echo "=============================================================="
            echo ""
            echo "Manages, auto-recruits, and updates the dedicated minimalist micro-stack:"
            echo "  - kv (Kvant): Pure Rust micro terminal text editor with full keyboard navigation"
            echo "  - sv (Svyaz): Pure Rust micro HTTP/HTTPS client with built-in Rustls"
            echo ""
            echo "Usage:"
            echo "  $0 comrades [subcommand]"
            echo ""
            echo "Subcommands:"
            echo "  status               Inspect presence, location, and version of kv and sv"
            echo "  install [all|kv|sv]  Download and install comrade binaries to ~/.local/bin"
            echo "  update               Update installed comrades to the latest nightly release"
            echo ""
            echo "Examples:"
            echo "  $0 comrades status"
            echo "  $0 comrades install"
            echo "  $0 comrades update"
            ;;
        "check")
            echo "cccp.sh check - GOST Standard Compliance Check"
            echo "============================================="
            echo ""
            echo "Executes the canonical project compliance check against the GOST standard,"
            echo "verifying license, SemVer, root anatomy, commits traceability, and branches."
            echo ""
            echo "Usage:"
            echo "  $0 check [options] [directory]"
            echo ""
            echo "Options:"
            echo "  --strict        Treat advisory checks (SHOULD) as blocking failures"
            echo "  -C, --dir <dir> Target directory or repository to inspect"
            echo "  -h, --help      Show this help message"
            ;;
        "gost")
            echo "cccp.sh gost - GOST Compliance & Synchronization Engine"
            echo "======================================================="
            echo ""
            echo "Manages GOST standard adherence, synchronization of specs and rules,"
            echo "and version upgrades for consumer projects."
            echo ""
            echo "Usage:"
            echo "  $0 gost <subcommand> [options]"
            echo ""
            echo "Subcommands:"
            echo "  sync             Download standard, profile and language specs into .gost/"
            echo "  check, audit     Audit repository adherence against GOST rules"
            echo "  upgrade <ver>    Display migration notes, bump GOSTVER and re-sync"
            echo "  init             Create a default GOSTVER file in project root"
            echo "  info, status     Display current GOST declaration and local cache status"
            echo ""
            echo "Examples:"
            echo "  $0 gost sync"
            echo "  $0 gost check"
            echo "  $0 gost upgrade 1.1.0"
            ;;
        "serve")
            echo "cccp.sh serve - Unified HTTP Development & Preview Server"
            echo "======================================================="
            echo ""
            echo "Serves local repositories in static, dev, or VDNKh demo modes,"
            echo "prioritizing comrade Mayak (mk) with transparent fallbacks."
            echo ""
            echo "Usage:"
            echo "  $0 serve [directory] [options]"
            echo ""
            echo "Modes:"
            echo "  (default)        Static HTTP server serving ./site, ./dist, ./public, or ./"
            echo "  --dev            Development mode with builder watcher, embedded PHP, or Node scripts"
            echo "  --demo           VDNKh demo mode with fixtures/seed, healthcheck, and browser launch"
            echo ""
            echo "Options:"
            echo "  -l, --localhost  Restrict bind to 127.0.0.1 (default is 0.0.0.0 for LAN/Tailscale)"
            echo "  --host <ip>      Custom IP address to bind"
            echo "  -p, --port <N>   Starting port (default: 8080, auto-increments if occupied)"
            echo "  -o, --open       Open default browser automatically"
            echo "  -q, --quiet      Suppress per-request access logs"
            echo "  -h, --help       Show this help message"
            ;;
        "repo"|"repos")
            echo "cccp.sh repo - Gosplan Multi-Repository Management & Overview"
            echo "==========================================================="
            echo ""
            echo "Provides a unified status overview and audit across multiple repositories"
            echo "in workspaces (such as ~/src or catalog.json), identifying dirty working trees,"
            echo "unpushed commits, and open forge tickets in a single dashboard."
            echo ""
            echo "Usage:"
            echo "  $0 repo [status|list] [options]"
            echo ""
            echo "Options:"
            echo "  --dirty             List only repositories with uncommitted/unstaged changes"
            echo "  --unpushed          List only repositories with commits pending push"
            echo "  --issues, --tickets List only repositories with open tickets/issues"
            echo "  --json              Output aggregated status as structured JSON"
            echo "  -d, --dir <path>    Scan target directory for repositories (default: auto)"
            echo "  -c, --catalog <cat> Use specific catalog.json file"
            echo "  -f, --fetch         Query remote forge for live issue counts"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 repo"
            echo "  $0 repo --dirty"
            echo "  $0 repo --unpushed"
            echo "  $0 repo --issues"
            echo "  $0 repo --json"
            echo "  $0 repo -d ~/src"
            ;;
        "newrepo"|"new-repo")
            echo "cccp.sh newrepo - Scaffolding Automatizado de Repositórios no Modelo GOST"
            echo "===================================================================="
            echo ""
            echo "Cria novos repositórios ou diretórios pré-configurados em estrita conformidade"
            echo "com a norma técnica GOST v1.1.0 e convenções semânticas do CCCP."
            echo ""
            echo "Usage:"
            echo "  $0 newrepo <nome-do-repositorio> [opções]"
            echo "  $0 repo new <nome-do-repositorio> [opções]"
            echo ""
            echo "Arguments:"
            echo "  <nome-do-repositorio>   Nome do projeto e diretório a ser criado"
            echo ""
            echo "Options:"
            echo "  --root <dir>            Diretório raiz alvo (padrão: ./<nome-do-repositorio>)"
            echo "  --desc <texto>          Descrição resumida do projeto para README e portal"
            echo "  --lang <langs>          Linguagens ativas separadas por vírgula (padrão: shell)"
            echo "  --license <spdx>        Licença SPDX (padrão: CC0-1.0; opções: MIT, GPL-3.0,"
            echo "                          CC-BY-SA-4.0, Apache-2.0)"
            echo "  --profile <url>         URL do perfil de implementação (padrão: profiles/lumen.md)"
            echo "  --no-git                Não inicializar repositório Git local"
            echo "  --no-hooks              Não instalar os Git hooks do cccp"
            echo "  --no-ci                 Não gerar workflow base de CI (.forgejo/workflows/ci.yaml)"
            echo "  --no-sync               Não sincronizar especificações GOST (.gost/)"
            echo "  -q, --quiet             Modo silencioso, omite mensagens informativas"
            echo "  -h, --help              Exibe esta mensagem de ajuda e sai"
            echo ""
            echo "Examples:"
            echo "  $0 newrepo meu-servico"
            echo "  $0 newrepo meu-cli --lang rust,shell --license MIT"
            echo "  $0 newrepo portal-docs --license CC-BY-SA-4.0 --desc 'Portal de documentação'"
            ;;
        "act")
            echo "cccp.sh act - Local CI Runner & Gate (act / Docker)"
            echo "================================================="
            echo ""
            echo "Manages local CI gate execution and inspects structured build logs."
            echo "Automatically triggered during 'git push' by the pre-push hook."
            echo ""
            echo "Usage:"
            echo "  $0 act [subcommand] [options]"
            echo ""
            echo "Subcommands:"
            echo "  log, logs        View or tail the most recent act CI gate log"
            echo "  run [job]        Execute local CI runner (default: test job)"
            echo "  status           Inspect act and Docker runner health"
            echo "  help             Show this help message"
            echo ""
            echo "Options:"
            echo "  -f, --follow     Follow log stream in real time (for 'cccp act log')"
            echo "  -j, --job <name> Target workflow job name (default: test)"
            echo "  -h, --help       Show this help message"
            echo ""
            echo "Environment & Bypasses:"
            echo "  git push --no-verify  Conscious bypass of local CI pre-push gate"
            echo "  CCCP_ACT_SKIP=1       Environment flag to bypass CI runner"
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
            echo "  catalog [options]  Inventory workspaces, update .git/description and sync forges"
            echo "  ticket, issue      Interactive issue creation via Diva with GOST branch suggestion"
            echo "  diva, divan        Gemini-powered issue triage bot with multi-forge dispatch"
            echo "  matraca            Mobile & offline-tolerant Web REPL for Antigravity CLI (agy)"
            echo "  cockpit, web       Interactive Gosplan Web Cockpit via Mayak Command Bridge"
            echo "  comrades [status]  Manage micro ecosystem tools (kvant editor and svyaz HTTP client)"
            echo "  check [options]    Audit project compliance against GOST standard"
            echo "  gost <subcommand>  Manage GOST synchronization, audit and upgrades"
            echo "  serve [options]    Unified HTTP server (static, dev with watcher, VDNKh demo)"
            echo "  repo [options]     Multi-repository aggregated overview (dirty, unpushed, issues)"
            echo "  newrepo <name>     Scaffolding of new repositories in GOST standard"
            echo "  act [subcommand]   Local CI runner, gate and structured log viewer"
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

get_global_data_dir() {
    echo "${XDG_DATA_HOME:-$HOME/.local/share}/cccp"
}

get_global_cache_dir() {
    echo "${XDG_CACHE_HOME:-$HOME/.cache}/cccp"
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
        catalog_dir) echo "~/src" ;;
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
                k=$(echo "$k" | sed -e 's/^[[:space:]]*export[[:space:]][[:space:]]*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
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
    if command -v get_global_cache_dir >/dev/null 2>&1; then
        echo "$(get_global_cache_dir)/update_cache"
    else
        echo "${XDG_CACHE_HOME:-$HOME/.cache}/cccp/update_cache"
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

    # Require HTTP client (sv, svyaz, curl or wget)
    if ! command -v _cccp_http >/dev/null 2>&1 && ! command -v sv >/dev/null 2>&1 && ! command -v svyaz >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
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
            ver_url="https://cccp.2lp.in/nightly/version"
        else
            ver_url="https://cccp.2lp.in/version"
        fi

        response=""
        if command -v _cccp_http >/dev/null 2>&1; then
            response=$(_cccp_http -s "$ver_url" 2>/dev/null || true)
        elif command -v sv >/dev/null 2>&1; then
            response=$(sv -s "$ver_url" 2>/dev/null || true)
        elif command -v svyaz >/dev/null 2>&1; then
            response=$(svyaz -s "$ver_url" 2>/dev/null || true)
        elif command -v curl >/dev/null 2>&1; then
            response=$(curl -s --max-time 2 "$ver_url" 2>/dev/null || true)
        elif command -v wget >/dev/null 2>&1; then
            response=$(wget -q -T 2 -O- "$ver_url" 2>/dev/null || true)
        fi

        if [ -n "$response" ]; then
            if echo "$response" | grep -q '"tag_name":'; then
                remote_ver=$(echo "$response" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1 | sed 's/^v//')
            else
                remote_ver=$(echo "$response" | head -n 1 | tr -d ' \r\n' | sed 's/^v//')
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
        ver_url="https://cccp.2lp.in/nightly/version"
    else
        ver_url="https://cccp.2lp.in/version"
    fi

    response=""
    if command -v _cccp_http >/dev/null 2>&1; then
        response=$(_cccp_http -s "$ver_url" 2>/dev/null || true)
    elif command -v sv >/dev/null 2>&1; then
        response=$(sv -s "$ver_url" 2>/dev/null || true)
    elif command -v svyaz >/dev/null 2>&1; then
        response=$(svyaz -s "$ver_url" 2>/dev/null || true)
    elif command -v curl >/dev/null 2>&1; then
        response=$(curl -s --max-time 3 "$ver_url" 2>/dev/null || true)
    elif command -v wget >/dev/null 2>&1; then
        response=$(wget -q -T 3 -O- "$ver_url" 2>/dev/null || true)
    fi

    if [ -n "$response" ]; then
        if echo "$response" | grep -q '"tag_name":'; then
            remote_ver=$(echo "$response" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1 | sed 's/^v//')
        else
            remote_ver=$(echo "$response" | head -n 1 | tr -d ' \r\n' | sed 's/^v//')
        fi
    fi

    if [ -z "$remote_ver" ]; then
        echo "Remote Version    : unavailable (unable to reach distribution endpoint)"
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
            --comrades)
                if command -v cmd_comrades >/dev/null 2>&1; then
                    cmd_comrades "update"
                    return $?
                fi
                echo "Error: Comrades module not available." >&2
                return 1
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
    fallback_url=""
    if [ "$do_pin" -eq 1 ] && [ -n "$pin_target" ]; then
        clean_pin="${pin_target#v}"
        download_url="https://cccp.2lp.in/releases/v${clean_pin}/cccp.sh"
        fallback_url="https://codeberg.org/lumenpink/cccp/raw/tag/v${clean_pin}/cccp.sh"
    elif [ "$channel" = "nightly" ]; then
        download_url="https://cccp.2lp.in/nightly/cccp.sh"
        fallback_url="https://codeberg.org/lumenpink/cccp/raw/branch/main/cccp.sh"
    elif [ "$channel" = "stable" ]; then
        download_url="https://cccp.2lp.in/cccp.sh"
        fallback_url="https://codeberg.org/lumenpink/cccp/raw/branch/main/cccp.sh"
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
    downloaded=0
    if command -v _cccp_http >/dev/null 2>&1; then
        if _cccp_http -fsSL "$download_url" -o "$tmp_file" 2>/dev/null; then
            downloaded=1
        elif [ -n "$fallback_url" ] && _cccp_http -fsSL "$fallback_url" -o "$tmp_file" 2>/dev/null; then
            echo "Notice: Downloaded from fallback mirror."
            downloaded=1
        fi
    elif command -v sv >/dev/null 2>&1; then
        if sv -fsSL "$download_url" -o "$tmp_file" 2>/dev/null; then
            downloaded=1
        elif [ -n "$fallback_url" ] && sv -fsSL "$fallback_url" -o "$tmp_file" 2>/dev/null; then
            echo "Notice: Downloaded from fallback mirror."
            downloaded=1
        fi
    elif command -v svyaz >/dev/null 2>&1; then
        if svyaz -fsSL "$download_url" -o "$tmp_file" 2>/dev/null; then
            downloaded=1
        elif [ -n "$fallback_url" ] && svyaz -fsSL "$fallback_url" -o "$tmp_file" 2>/dev/null; then
            echo "Notice: Downloaded from fallback mirror."
            downloaded=1
        fi
    elif command -v curl >/dev/null 2>&1; then
        if curl -fsSL "$download_url" -o "$tmp_file" 2>/dev/null; then
            downloaded=1
        elif [ -n "$fallback_url" ] && curl -fsSL "$fallback_url" -o "$tmp_file" 2>/dev/null; then
            echo "Notice: Downloaded from fallback mirror."
            downloaded=1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q "$download_url" -O "$tmp_file" 2>/dev/null; then
            downloaded=1
        elif [ -n "$fallback_url" ] && wget -q "$fallback_url" -O "$tmp_file" 2>/dev/null; then
            echo "Notice: Downloaded from fallback mirror."
            downloaded=1
        fi
    else
        echo "Error: Neither svyaz, curl nor wget is available." >&2
        rm -f "$tmp_file"
        return 1
    fi

    if [ "$downloaded" -eq 0 ] || [ ! -s "$tmp_file" ]; then
        echo "Error: Failed to download update from $download_url" >&2
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

    if command -v cmd_comrades >/dev/null 2>&1; then
        _bdir=$(_cccp_get_bin_dir 2>/dev/null || true)
        if [ -n "$_bdir" ] && { [ -x "${_bdir}/kv" ] || [ -x "${_bdir}/sv" ]; }; then
            echo ""
            cmd_comrades "update" || true
        fi
    fi

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
# GOST Compliance & Audit Functions
# =============================================================================
_gost_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        GOST_C_RED='\033[0;31m'
        GOST_C_GREEN='\033[0;32m'
        GOST_C_YELLOW='\033[0;33m'
        GOST_C_CYAN='\033[0;36m'
        GOST_C_BOLD='\033[1m'
        GOST_C_RESET='\033[0m'
    else
        GOST_C_RED=''
        GOST_C_GREEN=''
        GOST_C_YELLOW=''
        GOST_C_CYAN=''
        GOST_C_BOLD=''
        GOST_C_RESET=''
    fi
}

_gost_http() {
    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
        return $?
    fi
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    if command -v sv >/dev/null 2>&1; then
        sv "$@"
        return $?
    fi
    if command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
        return $?
    fi
    if command -v curl >/dev/null 2>&1; then
        curl "$@"
        return $?
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -qO- "$@"
        return $?
    fi
    return 1
}

_gost_read_gostver() {
    _target_dir="${1:-.}"
    _gostver_file="${_target_dir}/GOSTVER"

    GOST_VERSION=""
    GOST_PROFILE_URL=""
    GOST_LANGUAGES=""

    if [ ! -f "$_gostver_file" ]; then
        return 1
    fi

    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            \#*|"") continue ;;
            GOST_VERSION=*)
                _val="${_line#GOST_VERSION=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_VERSION="$_val"
                ;;
            GOST_PROFILE_URL=*)
                _val="${_line#GOST_PROFILE_URL=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_PROFILE_URL="$_val"
                ;;
            GOST_LANGUAGES=*)
                _val="${_line#GOST_LANGUAGES=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_LANGUAGES="$_val"
                ;;
        esac
    done < "$_gostver_file"

    [ -n "$GOST_VERSION" ] || return 1
    return 0
}

_gost_fetch_resource() {
    _target_res="$1"     # e.g. "SPEC.md", "profiles/lumen.md", "specs/rules.json", "specs/languages/shell.md"
    _target_dest="$2"    # destination file path
    _ver="${3:-${GOST_VERSION:-main}}"

    _dest_dir="$(dirname "$_target_dest")"
    mkdir -p "$_dest_dir"

    # 1. Local fast-path / offline mode if GOST_LOCAL_PATH or sibling repository exists
    _local_gost=""
    if [ -n "${GOST_LOCAL_PATH:-}" ] && [ -d "$GOST_LOCAL_PATH" ]; then
        _local_gost="$GOST_LOCAL_PATH"
    elif [ -d "/home/lumen/src/gost" ]; then
        _local_gost="/home/lumen/src/gost"
    elif [ -d "../gost" ]; then
        _local_gost="../gost"
    fi

    if [ -n "$_local_gost" ] && [ -f "${_local_gost}/${_target_res}" ]; then
        cp "${_local_gost}/${_target_res}" "$_target_dest"
        return 0
    fi

    # 2. Remote download via _gost_http
    case "$_target_res" in
        http://*|https://*)
            _gost_http -fsSL "$_target_res" -o "$_target_dest" 2>/dev/null
            return $?
            ;;
    esac

    # Try tag with 'v' prefix
    _tag_url="https://codeberg.org/lumenpink/gost/raw/tag/v${_ver}/${_target_res}"
    if _gost_http -fsSL "$_tag_url" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    # Try tag without 'v' prefix
    _tag_url_nov="https://codeberg.org/lumenpink/gost/raw/tag/${_ver}/${_target_res}"
    if _gost_http -fsSL "$_tag_url_nov" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    # Fallback to main branch
    _main_url="https://codeberg.org/lumenpink/gost/raw/branch/main/${_target_res}"
    if _gost_http -fsSL "$_main_url" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    return 1
}

_gost_is_semver() {
    _ver="$1"
    printf "%s\n" "$_ver" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
}

cmd_gost_sync() {
    _gost_colors
    _root_dir="${1:-.}"

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bErro: Arquivo GOSTVER não encontrado em '%s'.%b\n" "$GOST_C_RED" "$_root_dir" "$GOST_C_RESET" >&2
        printf "Crie o arquivo GOSTVER na raiz do projeto ou execute 'cccp gost init'.\n" >&2
        return 1
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Sincronizando norma GOST v%s...%b\n" "$GOST_C_CYAN" "$GOST_VERSION" "$GOST_C_RESET"

    mkdir -p "${_root_dir}/.gost/languages"

    # 1. SPEC.md -> .gost/GOST.md
    printf "  Baixando especificação canônica (GOST.md)... "
    if _gost_fetch_resource "SPEC.md" "${_root_dir}/.gost/GOST.md" "$GOST_VERSION"; then
        printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    else
        printf "%bFALHA%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    # 2. GOST_PROFILE_URL -> .gost/PROFILE.md
    if [ -n "$GOST_PROFILE_URL" ]; then
        printf "  Baixando perfil (%s -> PROFILE.md)... " "$GOST_PROFILE_URL"
        if _gost_fetch_resource "$GOST_PROFILE_URL" "${_root_dir}/.gost/PROFILE.md" "$GOST_VERSION"; then
            printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
        else
            printf "%bAVISO (não foi possível obter o perfil)%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
        fi
    fi

    # 3. specs/rules.json -> .gost/rules.json
    printf "  Sincronizando catálogo de regras (rules.json)... "
    if _gost_fetch_resource "specs/rules.json" "${_root_dir}/.gost/rules.json" "$GOST_VERSION"; then
        printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    else
        printf "%bAVISO (regras remotas indisponíveis)%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
    fi

    # 4. Only specs for languages listed in GOST_LANGUAGES
    if [ -n "$GOST_LANGUAGES" ]; then
        _old_ifs="$IFS"
        IFS=','
        for _lang in $GOST_LANGUAGES; do
            _l="$(echo "$_lang" | tr -d ' ')"
            [ -n "$_l" ] || continue
            printf "  Baixando especificação de linguagem (%s.md)... " "$_l"
            if _gost_fetch_resource "specs/languages/${_l}.md" "${_root_dir}/.gost/languages/${_l}.md" "$GOST_VERSION"; then
                printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
            else
                printf "%bAVISO (especificação '%s' não encontrada)%b\n" "$GOST_C_YELLOW" "$_l" "$GOST_C_RESET"
            fi
        done
        IFS="$_old_ifs"
    fi

    printf "%b✔ Sincronização GOST v%s concluída com sucesso em .gost/%b\n" "$GOST_C_GREEN" "$GOST_VERSION" "$GOST_C_RESET"
    return 0
}

cmd_gost_check() {
    _gost_colors
    _root_dir="${1:-.}"
    _strict_mode="${2:-0}"

    _passed=0
    _warnings=0
    _failures=0

    _gost_ver_display="Não declarado"
    _gost_prof_display="Nenhum"
    _gost_lang_display="Nenhuma"

    if _gost_read_gostver "$_root_dir"; then
        _gost_ver_display="v$GOST_VERSION"
        [ -n "$GOST_PROFILE_URL" ] && _gost_prof_display="$GOST_PROFILE_URL"
        [ -n "$GOST_LANGUAGES" ] && _gost_lang_display="$GOST_LANGUAGES"
    fi

    printf "\n%b☭ [ГОСТ / GOSPLAN] COMISSÃO DE CONTROLE DE QUALIDADE GOST%b\n" "$GOST_C_BOLD" "$GOST_C_RESET"
    printf "Norma: %s | Perfil: %s | Linguagens: %s\n" "$_gost_ver_display" "$_gost_prof_display" "$_gost_lang_display"
    printf "────────────────────────────────────────────────────────────────────────────\n"

    # 1. GOST-16-GOSTVER-COMPLIANCE (MUST / SHOULD)
    if [ -f "${_root_dir}/GOSTVER" ] && [ -n "$GOST_VERSION" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-16-GOSTVER" "Declaração de Versão via GOSTVER ($GOST_VERSION)"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-16-GOSTVER" "Arquivo GOSTVER ausente ou sem GOST_VERSION declarada"
        _failures=$((_failures + 1))
    fi

    # 2. GOST-01-LICENSE (MUST)
    if [ -f "${_root_dir}/LICENSE" ] && [ -s "${_root_dir}/LICENSE" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-01-LICENSE" "Arquivo LICENSE declarado e não-vazio"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-01-LICENSE" "Arquivo LICENSE ausente ou vazio"
        _failures=$((_failures + 1))
    fi

    # 3. GOST-02-ANATOMY-ROOT (MUST)
    _missing_root=""
    for _req_file in "README.md" "VERSION" "CHANGELOG.md" ".gitignore" ".editorconfig"; do
        if [ ! -f "${_root_dir}/${_req_file}" ]; then
            _missing_root="${_missing_root} ${_req_file}"
        fi
    done

    if [ -z "$_missing_root" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-02-ANATOMY-ROOT" "Arquivos básicos da raiz presentes"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-02-ANATOMY-ROOT" "Arquivos ausentes na raiz:${_missing_root}"
        _failures=$((_failures + 1))
    fi

    # 4. GOST-03-VERSION-SEMVER (MUST)
    if [ -f "${_root_dir}/VERSION" ]; then
        _v_content="$(head -n 1 "${_root_dir}/VERSION" 2>/dev/null | tr -d '\r\n ' || echo "")"
        if _gost_is_semver "$_v_content"; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Arquivo VERSION SemVer 2.0.0 ($_v_content)"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Versão '$_v_content' não é SemVer 2.0.0 estrito"
            _failures=$((_failures + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Arquivo VERSION inexistente"
        _failures=$((_failures + 1))
    fi

    # 5. GOST-04-TICKETS-COMMITS (MUST)
    if [ -d "${_root_dir}/.git" ] && command -v git >/dev/null 2>&1; then
        _commits_count=0
        _commits_valid=0
        _commit_log="$(cd "$_root_dir" && git log -n 5 --format="%s" 2>/dev/null || true)"
        if [ -n "$_commit_log" ]; then
            while IFS= read -r _subj || [ -n "$_subj" ]; do
                [ -n "$_subj" ] || continue
                _commits_count=$((_commits_count + 1))
                if printf "%s\n" "$_subj" | grep -Eq '\((ref|closes) #[0-9]+\)|(ref|closes) #[0-9]+|\(#[0-9]+\)'; then
                    _commits_valid=$((_commits_valid + 1))
                fi
            done <<EOF
$_commit_log
EOF
            if [ "$_commits_count" -gt 0 ] && [ "$_commits_valid" -eq "$_commits_count" ]; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Rastreabilidade estrita de tickets (${_commits_valid}/${_commits_count} commits recentes)"
                _passed=$((_passed + 1))
            elif [ "$_commits_count" -gt 0 ]; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Commits recentes sem referência de ticket (${_commits_valid}/${_commits_count} conformes)"
                _failures=$((_failures + 1))
            else
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Repositório novo (sem commits para auditar)"
                _passed=$((_passed + 1))
            fi
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Repositório novo (sem histórico de commits)"
            _passed=$((_passed + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Não é um repositório Git"
        _warnings=$((_warnings + 1))
    fi

    # 6. GOST-05-BRANCH-PER-TICKET (MUST)
    if [ -d "${_root_dir}/.git" ] && command -v git >/dev/null 2>&1; then
        _cur_branch="$(cd "$_root_dir" && (git branch --show-current 2>/dev/null || git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --abbrev-ref HEAD 2>/dev/null || echo ""))"
        if [ "$_cur_branch" = "main" ] || [ "$_cur_branch" = "master" ]; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Branch canônica/release ($_cur_branch)"
            _passed=$((_passed + 1))
        elif printf "%s\n" "$_cur_branch" | grep -Eq '^(feat|fix|docs|refactor|test|chore)/[0-9]+-[a-z0-9-]+$'; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Branch dedicada por ticket ($_cur_branch)"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Nome da branch '$_cur_branch' fora do padrão GOST"
            _failures=$((_failures + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Git indisponível para auditar branches"
        _warnings=$((_warnings + 1))
    fi

    # 7. GOST-09-SITE-HEADERS (SHOULD / MUST if site exists)
    if [ -d "${_root_dir}/site" ]; then
        if [ -f "${_root_dir}/site/_headers" ]; then
            if grep -q "X-Content-Type-Options: nosniff" "${_root_dir}/site/_headers" && grep -Eq "X-Frame-Options: (DENY|SAMEORIGIN)" "${_root_dir}/site/_headers"; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Cabeçalhos HTTP de segurança presentes em site/_headers"
                _passed=$((_passed + 1))
            else
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "site/_headers não contém nosniff ou DENY/SAMEORIGIN"
                _warnings=$((_warnings + 1))
            fi
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Diretório site/ presente sem site/_headers"
            _warnings=$((_warnings + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Vitrine web não declarada (N/A)"
        _passed=$((_passed + 1))
    fi

    # 8. GOST-10-CI-SEPARATION (SHOULD)
    if [ -d "${_root_dir}/.forgejo/workflows" ] || [ -d "${_root_dir}/.github/workflows" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-10-CI-SEPARATION" "Workflows de automação CI presentes"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-10-CI-SEPARATION" "Nenhum workflow de CI (.forgejo/.github) detectado"
        _warnings=$((_warnings + 1))
    fi

    # 9. GOST-17-LANGUAGE-SPECS (SHOULD)
    if [ -n "$GOST_LANGUAGES" ]; then
        _missing_specs=""
        _old_ifs="$IFS"
        IFS=','
        for _l in $GOST_LANGUAGES; do
            _lang="$(echo "$_l" | tr -d ' ')"
            [ -n "$_lang" ] || continue
            if [ ! -f "${_root_dir}/.gost/languages/${_lang}.md" ]; then
                _missing_specs="${_missing_specs} ${_lang}"
            fi
        done
        IFS="$_old_ifs"

        if [ -z "$_missing_specs" ]; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Especificações modulares sincronizadas em .gost/languages/"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Especificações ausentes:${_missing_specs} (execute 'cccp gost sync')"
            _warnings=$((_warnings + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Nenhuma especificação de linguagem declarada"
        _passed=$((_passed + 1))
    fi

    printf "────────────────────────────────────────────────────────────────────────────\n"
    _total=$((_passed + _warnings + _failures))
    _pct=0
    if [ "$_total" -gt 0 ]; then
        _pct=$(( (_passed * 100) / _total ))
    fi

    printf "Total de verificações: %d | Conformes: %b%d%b | Avisos: %b%d%b | Falhas: %b%d%b\n" \
        "$_total" \
        "$GOST_C_GREEN" "$_passed" "$GOST_C_RESET" \
        "$GOST_C_YELLOW" "$_warnings" "$GOST_C_RESET" \
        "$GOST_C_RED" "$_failures" "$GOST_C_RESET"
    printf "Índice de Conformidade Gosplan: %b%d%%%b\n\n" "$GOST_C_BOLD" "$_pct" "$GOST_C_RESET"

    if [ "$_failures" -gt 0 ]; then
        printf "%b✘ Inconformidades críticas detectadas. Corrija os itens marcados com [FAIL].%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    if [ "$_strict_mode" = "1" ] && [ "$_warnings" -gt 0 ]; then
        printf "%b✘ Modo estrito ativado: avisos tratados como falhas bloqueantes.%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    printf "%b✔ Projeto aprovado pela Comissão de Normas Técnicas GOST.%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    return 0
}

cmd_gost_upgrade() {
    _gost_colors
    _target_version="${1:-}"
    _root_dir="${2:-.}"

    if [ -z "$_target_version" ]; then
        printf "%bErro: Versão alvo não informada.%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        printf "Uso: cccp gost upgrade <versão>\n" >&2
        printf "Exemplo: cccp gost upgrade 1.1.0\n" >&2
        return 1
    fi

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bErro: Arquivo GOSTVER não encontrado em '%s'.%b\n" "$GOST_C_RED" "$_root_dir" "$GOST_C_RESET" >&2
        return 1
    fi

    _curr_ver="$GOST_VERSION"
    if [ "$_curr_ver" = "$_target_version" ]; then
        printf "%bO projeto já está na norma GOST v%s.%b\n" "$GOST_C_YELLOW" "$_curr_ver" "$GOST_C_RESET"
        return 0
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Migração de Norma: v%s → v%s%b\n" "$GOST_C_CYAN" "$_curr_ver" "$_target_version" "$GOST_C_RESET"

    # Search for migration guide: upgrades/from-X-to-Y.md
    _curr_mm="$(echo "$_curr_ver" | cut -d. -f1,2 | tr -d 'v')"
    _target_mm="$(echo "$_target_version" | cut -d. -f1,2 | tr -d 'v')"
    _guide_res="upgrades/from-${_curr_mm}-to-${_target_mm}.md"

    _tmp_guide="/tmp/gost_guide_$$.md"
    if _gost_fetch_resource "$_guide_res" "$_tmp_guide" "$_target_version"; then
        printf "\n%b=== GUIA DE MIGRAÇÃO (%s) ===%b\n" "$GOST_C_BOLD" "$_guide_res" "$GOST_C_RESET"
        cat "$_tmp_guide"
        printf "%b=====================================%b\n\n" "$GOST_C_BOLD" "$GOST_C_RESET"
        rm -f "$_tmp_guide"
    else
        printf "  (Nenhum guia de migração específico encontrado para %s → %s)\n\n" "$_curr_mm" "$_target_mm"
    fi

    # Update GOSTVER file
    _gostver_file="${_root_dir}/GOSTVER"
    _tmp_gostver="/tmp/GOSTVER.$$"
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            GOST_VERSION=*)
                printf 'GOST_VERSION="%s"\n' "$_target_version" >> "$_tmp_gostver"
                ;;
            *)
                printf '%s\n' "$_line" >> "$_tmp_gostver"
                ;;
        esac
    done < "$_gostver_file"

    mv "$_tmp_gostver" "$_gostver_file"
    printf "%b✔ GOSTVER atualizado para GOST_VERSION=\"%s\"%b\n" "$GOST_C_GREEN" "$_target_version" "$GOST_C_RESET"

    # Synchronize updated version
    cmd_gost_sync "$_root_dir"
    return $?
}

cmd_gost_init() {
    _gost_colors
    _root_dir="${1:-.}"
    _gostver_file="${_root_dir}/GOSTVER"

    if [ -f "$_gostver_file" ]; then
        printf "%bArquivo GOSTVER já existe em '%s'.%b\n" "$GOST_C_YELLOW" "$_root_dir" "$GOST_C_RESET"
        return 0
    fi

    cat > "$_gostver_file" << 'EOF'
# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.0.0-draft"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell"
EOF

    printf "%b✔ Arquivo GOSTVER criado com sucesso em '%s'.%b\n" "$GOST_C_GREEN" "$_gostver_file" "$GOST_C_RESET"
    cmd_gost_sync "$_root_dir"
}

cmd_gost_info() {
    _gost_colors
    _root_dir="${1:-.}"

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bProjeto não possui declaração GOSTVER.%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
        return 1
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Status da Norma GOST:%b\n" "$GOST_C_BOLD" "$GOST_C_RESET"
    printf "  Versão Declarada:    %b%s%b\n" "$GOST_C_GREEN" "$GOST_VERSION" "$GOST_C_RESET"
    printf "  Perfil de Referência: %s\n" "${GOST_PROFILE_URL:-Nenhum}"
    printf "  Linguagens Ativas:   %s\n" "${GOST_LANGUAGES:-Nenhuma}"

    if [ -d "${_root_dir}/.gost" ]; then
        printf "  Cache Local (.gost): Sincronizado\n"
        [ -f "${_root_dir}/.gost/GOST.md" ] && printf "    - GOST.md: presente\n"
        [ -f "${_root_dir}/.gost/PROFILE.md" ] && printf "    - PROFILE.md: presente\n"
        [ -f "${_root_dir}/.gost/rules.json" ] && printf "    - rules.json: presente\n"
    else
        printf "  Cache Local (.gost): %bNão sincronizado (execute 'cccp gost sync')%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
    fi
}

cmd_gost_help() {
    printf "Uso: cccp gost <subcomando> [opções]\n\n"
    printf "Gerenciador de Conformidade e Auditoria do Padrão GOST (Государственный стандарт).\n\n"
    printf "Subcomandos:\n"
    printf "  sync             Sincroniza GOST.md, PROFILE.md e especificações em .gost/\n"
    printf "  check, audit     Audita a conformidade das regras GOST no projeto\n"
    printf "  upgrade <versão> Exibe guia de migração, atualiza GOSTVER e sincroniza cache\n"
    printf "  init             Inicializa arquivo GOSTVER na raiz do projeto\n"
    printf "  info, status     Exibe informações do GOSTVER e do cache local\n"
    printf "  help             Exibe esta mensagem de ajuda\n\n"
    printf "Opções globais:\n"
    printf "  -C, --dir <dir>  Executa o comando no diretório ou repositório informado\n"
}

cmd_check() {
    _strict=0
    _dir="."

    while [ $# -gt 0 ]; do
        case "$1" in
            --strict)
                _strict=1
                shift
                ;;
            -C|--dir)
                shift
                [ $# -gt 0 ] || { echo "Erro: $1 requer caminho do diretório" >&2; return 1; }
                _dir="$1"
                shift
                ;;
            -h|--help)
                cmd_check_help
                return 0
                ;;
            *)
                if [ -d "$1" ]; then
                    _dir="$1"
                else
                    echo "Opção ou diretório desconhecido: $1" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    cmd_gost_check "$_dir" "$_strict"
}

cmd_check_help() {
    printf "Uso: cccp check [opções] [diretório]\n\n"
    printf "Executa a auditoria padrão de conformidade com a especificação GOST.\n\n"
    printf "Opções:\n"
    printf "  --strict        Trata avisos (SHOULD) como erros bloqueantes\n"
    printf "  -C, --dir <dir> Diretório alvo da verificação\n"
    printf "  -h, --help      Exibe esta mensagem de ajuda\n"
}

cmd_gost() {
    _subcmd="${1:-}"
    case "$_subcmd" in
        "sync")
            shift || true
            cmd_gost_sync "$@"
            ;;
        "check"|"audit")
            shift || true
            cmd_gost_check "$@"
            ;;
        "upgrade")
            shift || true
            cmd_gost_upgrade "$@"
            ;;
        "init")
            shift || true
            cmd_gost_init "$@"
            ;;
        "info"|"status")
            shift || true
            cmd_gost_info "$@"
            ;;
        "help"|"-h"|"--help"|"")
            cmd_gost_help
            ;;
        *)
            _gost_colors
            printf "%bErro: Subcomando desconhecido 'gost %s'.%b\n\n" "$GOST_C_RED" "$_subcmd" "$GOST_C_RESET" >&2
            cmd_gost_help
            return 1
            ;;
    esac
}


# =============================================================================
# Repository Scaffolding (GOST Standard) Functions
# =============================================================================
_newrepo_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        NR_C_RED='\033[0;31m'
        NR_C_GREEN='\033[0;32m'
        NR_C_YELLOW='\033[0;33m'
        NR_C_CYAN='\033[0;36m'
        NR_C_BOLD='\033[1m'
        NR_C_RESET='\033[0m'
    else
        NR_C_RED=''
        NR_C_GREEN=''
        NR_C_YELLOW=''
        NR_C_CYAN=''
        NR_C_BOLD=''
        NR_C_RESET=''
    fi
}

newrepo_usage() {
    cat << 'EOF'
cccp newrepo - Scaffolding Automatizado de Repositórios no Modelo GOST
====================================================================

Cria novos repositórios ou diretórios pré-configurados em estrita conformidade
com a norma técnica GOST v1.1.0 e convenções semânticas do CCCP.

Uso:
  cccp newrepo <nome-do-repositorio> [opções]
  cccp repo new <nome-do-repositorio> [opções]

Parâmetros:
  <nome-do-repositorio>   Nome do projeto e diretório a ser criado

Opções:
  --root <dir>            Diretório raiz alvo (padrão: ./<nome-do-repositorio>)
  --desc <texto>          Descrição resumida do projeto para README e portal
  --lang <langs>          Linguagens ativas separadas por vírgula (padrão: shell)
  --license <spdx>        Licença SPDX (padrão: CC0-1.0; opções: MIT, GPL-3.0,
                          CC-BY-SA-4.0, Apache-2.0)
  --profile <url>         URL do perfil de implementação (padrão: profiles/lumen.md)
  --no-git                Não inicializar repositório Git local
  --no-hooks              Não instalar os Git hooks do cccp
  --no-ci                 Não gerar workflow base de CI (.forgejo/workflows/ci.yaml)
  --no-sync               Não sincronizar especificações GOST (.gost/)
  -q, --quiet             Modo silencioso, omite mensagens informativas
  -h, --help              Exibe esta mensagem de ajuda e sai

Exemplos:
  cccp newrepo meu-servico
  cccp newrepo meu-cli --lang rust,shell --license MIT
  cccp newrepo portal-docs --license CC-BY-SA-4.0 --desc "Portal público de documentação"
  cccp newrepo app-web --no-git
EOF
}

_newrepo_write_file_if_missing() {
    _target="$1"
    _content="$2"
    _label="$3"
    _quiet="${4:-0}"

    if [ -f "$_target" ]; then
        if [ "$_quiet" -eq 0 ]; then
            printf "  %b[EXISTE]%b %s (preservado)\n" "$NR_C_YELLOW" "$NR_C_RESET" "$_label"
        fi
    else
        _dir=$(dirname "$_target")
        [ -d "$_dir" ] || mkdir -p "$_dir"
        printf "%s\n" "$_content" > "$_target"
        if [ "$_quiet" -eq 0 ]; then
            printf "  %b[CRIADO]%b %s\n" "$NR_C_GREEN" "$NR_C_RESET" "$_label"
        fi
    fi
}

_newrepo_license_text() {
    _spdx="$1"
    _name="${2:-Projeto}"
    _year=$(date +%Y 2>/dev/null || echo "2026")

    case "$_spdx" in
        CC0-1.0|CC0)
            cat << 'EOF'
CC0 1.0 Universal (CC0 1.0) Public Domain Dedication

The person who associated a work with this deed has dedicated the work to the
public domain by waiving all of his or her rights to the work worldwide under
copyright law, including all related and neighboring rights, to the extent
allowed by law.

You can copy, modify, distribute and perform the work, even for commercial
purposes, all without asking permission.
EOF
            ;;
        MIT)
            cat << EOF
MIT License

Copyright (c) $_year $_name

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
            ;;
        CC-BY-SA-4.0|CC-BY-SA)
            cat << EOF
Creative Commons Attribution-ShareAlike 4.0 International (CC-BY-SA 4.0)

You are free to:
- Share — copy and redistribute the material in any medium or format
- Adapt — remix, transform, and build upon the material for any purpose, even commercially.

Under the following terms:
- Attribution — You must give appropriate credit, provide a link to the license,
  and indicate if changes were made.
- ShareAlike — If you remix, transform, or build upon the material, you must
  distribute your contributions under the same license as the original.
EOF
            ;;
        GPL-3.0|GPLv3)
            cat << EOF
GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007

Copyright (C) $_year $_name

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
EOF
            ;;
        Apache-2.0|Apache)
            cat << EOF
Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

Copyright $_year $_name

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
EOF
            ;;
        *)
            cat << EOF
Software License: $_spdx
Copyright (c) $_year $_name
EOF
            ;;
    esac
}

cmd_newrepo() {
    _newrepo_colors

    _nr_name=""
    _nr_root=""
    _nr_desc=""
    _nr_lang="shell"
    _nr_license="CC0-1.0"
    _nr_profile="profiles/lumen.md"
    _nr_git=1
    _nr_hooks=1
    _nr_ci=1
    _nr_sync=1
    _nr_quiet=0

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                newrepo_usage
                return 0
                ;;
            -q|--quiet)
                _nr_quiet=1
                shift
                ;;
            --root)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --root requer um diretório.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_root="$1"
                shift
                ;;
            --name)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --name requer um nome.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_name="$1"
                shift
                ;;
            --desc)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --desc requer um texto descritivo.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_desc="$1"
                shift
                ;;
            --lang)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --lang requer uma lista de linguagens.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_lang="$1"
                shift
                ;;
            --license)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --license requer um identificador SPDX.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_license="$1"
                shift
                ;;
            --profile)
                shift
                [ $# -gt 0 ] || { printf "%bErro: --profile requer uma URL de perfil.%b\n" "$NR_C_RED" "$NR_C_RESET" >&2; return 1; }
                _nr_profile="$1"
                shift
                ;;
            --no-git)
                _nr_git=0
                shift
                ;;
            --git)
                _nr_git=1
                shift
                ;;
            --no-hooks)
                _nr_hooks=0
                shift
                ;;
            --hooks)
                _nr_hooks=1
                shift
                ;;
            --no-ci)
                _nr_ci=0
                shift
                ;;
            --ci)
                _nr_ci=1
                shift
                ;;
            --no-sync)
                _nr_sync=0
                shift
                ;;
            --sync)
                _nr_sync=1
                shift
                ;;
            -*)
                printf "%bErro: Opção desconhecida '%s'. Use --help para ver as opções disponíveis.%b\n" "$NR_C_RED" "$1" "$NR_C_RESET" >&2
                return 1
                ;;
            *)
                if [ -z "$_nr_name" ]; then
                    _nr_name="$1"
                else
                    printf "%bErro: Parâmetro posicional inesperado '%s'.%b\n" "$NR_C_RED" "$1" "$NR_C_RESET" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    # Validation: project name or target directory must be specified
    if [ -z "$_nr_name" ] && [ -z "$_nr_root" ]; then
        printf "%bErro: Informe o nome do repositório a ser criado (ex: cccp newrepo meu-app).%b\n" "$NR_C_RED" "$NR_C_RESET" >&2
        return 1
    fi

    # Determine root directory
    if [ -z "$_nr_root" ]; then
        _nr_root="./${_nr_name}"
    fi

    # Determine project name if derived from root
    if [ -z "$_nr_name" ]; then
        _nr_name=$(basename "$_nr_root")
    fi

    if [ -z "$_nr_desc" ]; then
        _nr_desc="Repositório estruturado em estrita conformidade com o padrão GOST v1.1.0"
    fi

    # Ensure target directory exists
    [ -d "$_nr_root" ] || mkdir -p "$_nr_root"

    if [ "$_nr_quiet" -eq 0 ]; then
        printf "%b☭ [ГОСТ / GOSPLAN] Scaffolding de Repositório: %s%b\n" "$NR_C_BOLD" "$_nr_name" "$NR_C_RESET"
        printf "Diretório Alvo: %s | Linguagens: %s | Licença: %s\n" "$_nr_root" "$_nr_lang" "$_nr_license"
        printf "────────────────────────────────────────────────────────────────────────────\n"
    fi

    # 1. Git Repository Initialization
    if [ "$_nr_git" -eq 1 ] && [ ! -d "${_nr_root}/.git" ]; then
        (cd "$_nr_root" && git init -b main >/dev/null 2>&1 || git init >/dev/null 2>&1)
        if [ "$_nr_quiet" -eq 0 ]; then
            printf "  %b[GIT]%b Repositório Git inicializado na branch main\n" "$NR_C_GREEN" "$NR_C_RESET"
        fi
    fi

    # 2. GOSTVER (GOST-16)
    _gostver_content=$(cat << EOF
# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.1.0"
GOST_PROFILE_URL="$_nr_profile"
GOST_LANGUAGES="$_nr_lang"
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/GOSTVER" "$_gostver_content" "GOSTVER (v1.1.0)" "$_nr_quiet"

    # 3. .editorconfig (GOST-02)
    _editorconfig_content=$(cat << 'EOF'
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 2

[*.md]
trim_trailing_whitespace = false
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/.editorconfig" "$_editorconfig_content" ".editorconfig" "$_nr_quiet"

    # 4. .gitignore (GOST-02)
    _gitignore_content=$(cat << 'EOF'
# Dependências e compilação
/target/
/vendor/
/node_modules/
/dist/
/build/

# Caches e arquivos temporários
.cache/
*.tmp
*.log
.DS_Store

# Metadados de agentes locais
.antigravity/
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/.gitignore" "$_gitignore_content" ".gitignore" "$_nr_quiet"

    # 5. VERSION (GOST-03)
    _newrepo_write_file_if_missing "${_nr_root}/VERSION" "0.1.0-dev.1" "VERSION (0.1.0-dev.1)" "$_nr_quiet"

    # 6. CHANGELOG.md (GOST-02)
    _changelog_content=$(cat << 'EOF'
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Inicialização do repositório em conformidade com o padrão GOST.
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/CHANGELOG.md" "$_changelog_content" "CHANGELOG.md" "$_nr_quiet"

    # 7. LICENSE (GOST-01)
    _license_body=$(_newrepo_license_text "$_nr_license" "$_nr_name")
    _newrepo_write_file_if_missing "${_nr_root}/LICENSE" "$_license_body" "LICENSE ($_nr_license)" "$_nr_quiet"

    # 8. README.md (GOST-02)
    _readme_content=$(cat << EOF
# $_nr_name

$_nr_desc

## 📋 Conformidade GOST

Este projeto adota a norma **[GOST](https://codeberg.org/lumenpink/gost)** (Gosplan Standard for Technical Software).
- **Versão GOST:** \`1.1.0\`
- **Linguagens declaradas:** \`$_nr_lang\`
- **Licença:** \`$_nr_license\`

## 🚀 Como Iniciar

\`\`\`bash
# Clonar o repositório
git clone <url-do-repositorio>
cd $_nr_name

# Verificar conformidade
cccp check
\`\`\`

## 📄 Licença

Distribuído sob a licença [$_nr_license](./LICENSE).
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/README.md" "$_readme_content" "README.md" "$_nr_quiet"

    # 9. site/_headers e site/index.html (GOST-09)
    _headers_content=$(cat << 'EOF'
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: strict-origin-when-cross-origin
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/site/_headers" "$_headers_content" "site/_headers" "$_nr_quiet"

    _site_index_content=$(cat << EOF
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>$_nr_name — Documentação</title>
  <style>
    body { font-family: monospace, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; line-height: 1.6; }
    h1 { border-bottom: 2px solid #333; padding-bottom: 8px; }
    .badge { display: inline-block; padding: 2px 8px; background: #eee; border: 1px solid #ccc; font-size: 0.9em; }
  </style>
</head>
<body>
  <h1>$_nr_name</h1>
  <p>$_nr_desc</p>
  <p><span class="badge">GOST v1.1.0</span> <span class="badge">$_nr_license</span></p>
</body>
</html>
EOF
)
    _newrepo_write_file_if_missing "${_nr_root}/site/index.html" "$_site_index_content" "site/index.html" "$_nr_quiet"

    # 10. Workflows de CI (GOST-10)
    if [ "$_nr_ci" -eq 1 ]; then
        _ci_workflow_content=$(cat << 'EOF'
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  check:
    runs-on: native
    steps:
      - uses: actions/checkout@v4
      - name: GOST Compliance Audit
        run: |
          if command -v cccp >/dev/null 2>&1; then
            cccp check --strict
          fi
EOF
)
        _newrepo_write_file_if_missing "${_nr_root}/.forgejo/workflows/ci.yaml" "$_ci_workflow_content" ".forgejo/workflows/ci.yaml (CI)" "$_nr_quiet"
    fi

    # 11. Sincronização GOST (.gost/) (GOST-17)
    if [ "$_nr_sync" -eq 1 ]; then
        if command -v cmd_gost_sync >/dev/null 2>&1; then
            cmd_gost_sync "$_nr_root" >/dev/null 2>&1 || true
            if [ "$_nr_quiet" -eq 0 ] && [ -d "${_nr_root}/.gost" ]; then
                printf "  %b[GOST]%b Especificações GOST sincronizadas em .gost/\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        fi
    fi

    # 12. Instalação de hooks semânticos CCCP
    if [ "$_nr_hooks" -eq 1 ] && [ -d "${_nr_root}/.git" ]; then
        if command -v cccp >/dev/null 2>&1; then
            (cd "$_nr_root" && cccp install >/dev/null 2>&1 || true)
            if [ "$_nr_quiet" -eq 0 ]; then
                printf "  %b[HOOKS]%b Git hooks instalados via 'cccp install'\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        elif command -v install_cccp >/dev/null 2>&1; then
            (cd "$_nr_root" && install_cccp >/dev/null 2>&1 || true)
            if [ "$_nr_quiet" -eq 0 ]; then
                printf "  %b[HOOKS]%b Git hooks instalados via 'install_cccp'\n" "$NR_C_GREEN" "$NR_C_RESET"
            fi
        fi
    fi

    if [ "$_nr_quiet" -eq 0 ]; then
        printf "────────────────────────────────────────────────────────────────────────────\n"
        printf "%b✔ Repositório '%s' criado com sucesso com 100%% de conformidade GOST!%b\n\n" "$NR_C_GREEN" "$_nr_name" "$NR_C_RESET"
        printf "Para iniciar o desenvolvimento:\n"
        printf "  cd %s\n" "$_nr_root"
        printf "  cccp check\n\n"
    fi

    return 0
}


# =============================================================================
# Unified Server Functions (Static, Dev & Demo)
# =============================================================================
_serve_cleanup() {
    if [ -n "${_SERVE_BG_PIDS:-}" ]; then
        for _pid in $_SERVE_BG_PIDS; do
            if kill -0 "$_pid" 2>/dev/null; then
                kill "$_pid" 2>/dev/null || true
                wait "$_pid" 2>/dev/null || true
            fi
        done
        _SERVE_BG_PIDS=""
    fi
}

_serve_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        SERVE_C_RED='\033[0;31m'
        SERVE_C_GREEN='\033[0;32m'
        SERVE_C_YELLOW='\033[0;33m'
        SERVE_C_GOLD='\033[1;33m'
        SERVE_C_CYAN='\033[0;36m'
        SERVE_C_BOLD='\033[1m'
        SERVE_C_RESET='\033[0m'
    else
        SERVE_C_RED=''
        SERVE_C_GREEN=''
        SERVE_C_YELLOW=''
        SERVE_C_GOLD=''
        SERVE_C_CYAN=''
        SERVE_C_BOLD=''
        SERVE_C_RESET=''
    fi
}

_serve_port_in_use() {
    _p="$1"
    _h="${2:-127.0.0.1}"
    if command -v ss >/dev/null 2>&1; then
        ss -tlnH "sport = :$_p" 2>/dev/null | grep -q ":$_p" && return 0
    elif command -v lsof >/dev/null 2>&1; then
        lsof -iTCP:"$_p" -sTCP:LISTEN -P -n >/dev/null 2>&1 && return 0
    elif command -v nc >/dev/null 2>&1; then
        nc -z -w 1 "$_h" "$_p" >/dev/null 2>&1 && return 0
    fi
    return 1
}

_serve_find_free_port() {
    _start_port="${1:-8080}"
    _host="${2:-127.0.0.1}"
    _cur_p="$_start_port"
    while [ "$_cur_p" -lt 65535 ]; do
        if ! _serve_port_in_use "$_cur_p" "$_host"; then
            echo "$_cur_p"
            return 0
        fi
        _cur_p=$((_cur_p + 1))
    done
    echo "$_start_port"
}

_serve_get_lan_ip() {
    if command -v ip >/dev/null 2>&1; then
        _route_ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2 || true)"
        if [ -n "$_route_ip" ]; then
            echo "$_route_ip"
            return 0
        fi
    fi
    if command -v hostname >/dev/null 2>&1; then
        _host_ip="$(hostname -I 2>/dev/null | cut -d' ' -f1 || true)"
        [ -n "$_host_ip" ] && echo "$_host_ip" && return 0
    fi
    return 1
}

_serve_get_tailscale_ip() {
    if command -v tailscale >/dev/null 2>&1; then
        _ts_ip="$(tailscale ip -4 2>/dev/null | head -n 1 || true)"
        [ -n "$_ts_ip" ] && echo "$_ts_ip" && return 0
    fi
    if command -v ip >/dev/null 2>&1; then
        _ts_ip="$(ip -4 addr show dev tailscale0 2>/dev/null | grep -o 'inet [0-9.]*' | cut -d' ' -f2 || true)"
        [ -n "$_ts_ip" ] && echo "$_ts_ip" && return 0
    fi
    return 1
}

_serve_open_browser() {
    _url="$1"
    if [ -n "${DISPLAY:-}" ] || [ -n "${WAYLAND_DISPLAY:-}" ]; then
        if command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$_url" >/dev/null 2>&1 &
        elif command -v open >/dev/null 2>&1; then
            open "$_url" >/dev/null 2>&1 &
        fi
    fi
}

_serve_detect_target_dir() {
    _override="${1:-}"
    if [ -n "$_override" ] && [ -d "$_override" ]; then
        echo "$_override"
        return 0
    fi
    if [ -d "./site" ]; then
        echo "./site"
    elif [ -d "./dist" ]; then
        echo "./dist"
    elif [ -d "./public" ]; then
        echo "./public"
    else
        echo "."
    fi
}

_serve_display_banner() {
    _mode="$1"
    _target="$2"
    _host="$3"
    _port="$4"

    _serve_colors
    _lan_ip="$(_serve_get_lan_ip || true)"
    _ts_ip="$(_serve_get_tailscale_ip || true)"

    if [ "$_mode" = "demo" ]; then
        printf "\n%b★════════════════════════════════════════════════════════════════════════════★%b\n" "$SERVE_C_GOLD" "$SERVE_C_RESET"
        printf "%b  ☭ ВДНХ — EXPOSIÇÃO DAS CONQUISTAS DA ECONOMIA NACIONAL (MODO DEMO) ☭%b\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
        printf "%b  Pavilhão Central da Demonstração & Inovação Tecnológica%b\n" "$SERVE_C_CYAN" "$SERVE_C_RESET"
        printf "%b★════════════════════════════════════════════════════════════════════════════★%b\n\n" "$SERVE_C_GOLD" "$SERVE_C_RESET"
    else
        printf "\n%b☭ [ГОСТ / GOSPLAN] CCCP HTTP SERVER ENGINE%b\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
        printf "Modo: %b%s%b | Alvo: %s\n" "$SERVE_C_CYAN" "$_mode" "$SERVE_C_RESET" "$_target"
        printf "────────────────────────────────────────────────────────────────────────────\n"
    fi

    printf "  %b➜ Local (Loopback):%b  http://localhost:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_port"
    if [ "$_host" != "127.0.0.1" ]; then
        [ -n "$_lan_ip" ] && printf "  %b➜ Rede Local (LAN):%b  http://%s:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_lan_ip" "$_port"
        [ -n "$_ts_ip" ] && printf "  %b➜ Tailscale / VPN:%b   http://%s:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_ts_ip" "$_port"
    fi
    printf "────────────────────────────────────────────────────────────────────────────\n"
    printf "Pressione %bCtrl+C%b para encerrar o serviço de transmissão.\n\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
}

_serve_run_static_server() {
    _dir="$1"
    _host="$2"
    _port="$3"
    _quiet="${4:-0}"

    # 1. Comrade Mayak (mk)
    _mk_bin=""
    if command -v mk >/dev/null 2>&1; then
        _mk_bin="mk"
    elif [ -x "${HOME}/.local/bin/mk" ]; then
        _mk_bin="${HOME}/.local/bin/mk"
    elif command -v _cccp_get_bin_dir >/dev/null 2>&1; then
        _cdir="$(_cccp_get_bin_dir 2>/dev/null || echo "")"
        [ -n "$_cdir" ] && [ -x "$_cdir/mk" ] && _mk_bin="$_cdir/mk"
    fi

    if [ -n "$_mk_bin" ]; then
        _mk_args="$_dir -p $_port --host $_host"
        [ "$_quiet" = "1" ] && _mk_args="$_mk_args -q"
        exec "$_mk_bin" $_mk_args
    fi

    # 2. Python 3 http.server fallback
    _py_bin="py"
    _py_bin="${_py_bin}thon3"
    if command -v "$_py_bin" >/dev/null 2>&1; then
        exec "$_py_bin" -m http.server "$_port" --bind "$_host" -d "$_dir"
    fi

    # 3. BusyBox httpd
    if command -v busybox >/dev/null 2>&1; then
        exec busybox httpd -f -p "${_host}:${_port}" -h "$_dir"
    fi

    echo "Erro: Nenhum servidor HTTP encontrado (instale mayak via 'cccp comrades install' ou python3/busybox)." >&2
    return 1
}

_serve_watch_and_build() {
    _builder="$1"
    shift
    _watch_dirs="$*"

    [ -n "$_watch_dirs" ] || _watch_dirs="."

    if command -v inotifywait >/dev/null 2>&1; then
        while true; do
            inotifywait -r -q -e modify,create,delete,move $_watch_dirs 2>/dev/null || true
            sleep 0.3
            printf "\n%b✦ [ГОСТ / GOSPLAN] Arquivos modificados. Reconstruindo com %s...%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
            eval "$_builder" || true
        done
    else
        # POSIX mtime polling loop
        _last_mtime=""
        while true; do
            sleep 1.5
            _curr_mtime="$(find $_watch_dirs -type f -exec stat -c %Y {} + 2>/dev/null | sort -n | tail -n 1 || true)"
            if [ -n "$_last_mtime" ] && [ -n "$_curr_mtime" ] && [ "$_curr_mtime" != "$_last_mtime" ]; then
                printf "\n%b✦ [ГОСТ / GOSPLAN] Modificação detectada. Reconstruindo com %s...%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
                eval "$_builder" || true
            fi
            _last_mtime="$_curr_mtime"
        done
    fi
}

cmd_serve() {
    _mode="static"
    _host="0.0.0.0"
    _port=8080
    _open_browser=0
    _target_arg=""
    _quiet=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --dev)
                _mode="dev"
                shift
                ;;
            --demo)
                _mode="demo"
                _open_browser=1
                shift
                ;;
            -l|--localhost)
                _host="127.0.0.1"
                shift
                ;;
            --host)
                shift
                [ $# -gt 0 ] || { echo "Erro: --host requer endereço IP" >&2; return 1; }
                _host="$1"
                shift
                ;;
            -p|--port)
                shift
                [ $# -gt 0 ] || { echo "Erro: --port requer número de porta" >&2; return 1; }
                _port="$1"
                shift
                ;;
            -o|--open)
                _open_browser=1
                shift
                ;;
            -q|--quiet)
                _quiet=1
                shift
                ;;
            -h|--help)
                cmd_serve_help
                return 0
                ;;
            *)
                if [ -d "$1" ]; then
                    _target_arg="$1"
                else
                    echo "Erro: Opção ou diretório desconhecido '$1'" >&2
                    cmd_serve_help
                    return 1
                fi
                shift
                ;;
        esac
    done

    # Setup process traps for clean shutdown
    trap '_serve_cleanup; exit 0' INT TERM EXIT

    # Find free port if initial port is occupied
    _port="$(_serve_find_free_port "$_port" "$_host")"

    # =========================================================================
    # 1. Modo Demonstração (cccp serve --demo)
    # =========================================================================
    if [ "$_mode" = "demo" ]; then
        if [ -f "./scripts/demo.sh" ]; then
            printf "✦ Executando rotina de demonstração (scripts/demo.sh)...\n"
            sh "./scripts/demo.sh"
        elif [ -f "./.cccp/demo.sh" ]; then
            printf "✦ Executando rotina de demonstração (.cccp/demo.sh)...\n"
            sh "./.cccp/demo.sh"
        fi

        _target_dir="$(_serve_detect_target_dir "$_target_arg")"
        _serve_display_banner "demo" "$_target_dir" "$_host" "$_port"

        # Launch server in background
        _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet" &
        _server_pid=$!
        _SERVE_BG_PIDS="$_server_pid"

        # Polling healthcheck until server responds
        _chk_host="$_host"
        [ "$_chk_host" = "0.0.0.0" ] && _chk_host="127.0.0.1"
        _attempts=0
        while [ "$_attempts" -lt 30 ]; do
            if command -v nc >/dev/null 2>&1 && nc -z -w 1 "$_chk_host" "$_port" >/dev/null 2>&1; then
                break
            elif command -v curl >/dev/null 2>&1 && curl -fs "http://${_chk_host}:${_port}/" >/dev/null 2>&1; then
                break
            fi
            sleep 0.2
            _attempts=$((_attempts + 1))
        done

        _serve_open_browser "http://localhost:${_port}/"
        wait "$_server_pid" 2>/dev/null || true
        return 0
    fi

    # =========================================================================
    # 2. Modo Desenvolvimento (cccp serve --dev)
    # =========================================================================
    if [ "$_mode" = "dev" ]; then
        # 2.1 Delegation to repo custom script
        if [ -f "./scripts/dev.sh" ]; then
            printf "✦ Delegando execução para scripts/dev.sh...\n"
            sh "./scripts/dev.sh"
            return $?
        elif [ -f "./.cccp/dev.sh" ]; then
            printf "✦ Delegando execução para .cccp/dev.sh...\n"
            sh "./.cccp/dev.sh"
            return $?
        fi

        # 2.2 PHP Project
        _php_bin="p"
        _php_bin="${_php_bin}hp"
        _php_entry=""
        if [ -f "index.php" ]; then
            _php_entry="index.php"
        elif [ -f "fuxico.php" ]; then
            _php_entry="fuxico.php"
        elif [ -f "public/index.php" ]; then
            _php_entry="-t public public/index.php"
        fi

        if [ -n "$_php_entry" ] && command -v "$_php_bin" >/dev/null 2>&1; then
            _serve_display_banner "dev (PHP)" "$_php_entry" "$_host" "$_port"
            [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
            exec "$_php_bin" -S "${_host}:${_port}" $_php_entry
        fi

        # 2.3 Node.js Project
        _npm_bin="n"
        _npm_bin="${_npm_bin}pm"
        if [ -f "package.json" ]; then
            if command -v "$_npm_bin" >/dev/null 2>&1; then
                _serve_display_banner "dev (Node.js)" "package.json" "$_host" "$_port"
                [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
                if grep -q '"dev":' package.json 2>/dev/null; then
                    exec "$_npm_bin" run dev
                elif grep -q '"serve":' package.json 2>/dev/null; then
                    exec "$_npm_bin" run serve
                else
                    exec "$_npm_bin" start
                fi
            fi
        fi

        # 2.4 Compiled / Static project with build watcher
        _builder=""
        if [ -f "./combine.sh" ]; then
            _builder="./combine.sh"
        elif [ -f "./compile.php" ] && command -v "$_php_bin" >/dev/null 2>&1; then
            _builder="$_php_bin compile.php"
        elif [ -f "./Cargo.toml" ]; then
            _cargo_bin="c"
            _cargo_bin="${_cargo_bin}argo"
            if command -v "$_cargo_bin" >/dev/null 2>&1; then
                _builder="$_cargo_bin build"
            fi
        elif [ -f "./Makefile" ] && command -v make >/dev/null 2>&1; then
            _builder="make"
        fi

        _target_dir="$(_serve_detect_target_dir "$_target_arg")"
        _serve_display_banner "dev" "$_target_dir" "$_host" "$_port"

        if [ -n "$_builder" ]; then
            printf "✦ Construtor ativo: %b%s%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
            printf "✦ Executando build inicial...\n"
            eval "$_builder" || true

            # Collect watch directories
            _watch_dirs=""
            for _wdir in "src" "packages" "crates" "site" "public"; do
                [ -d "$_wdir" ] && _watch_dirs="${_watch_dirs} ${_wdir}"
            done

            # Launch watcher in background
            _serve_watch_and_build "$_builder" $_watch_dirs &
            _watcher_pid=$!
            _SERVE_BG_PIDS="$_SERVE_BG_PIDS $_watcher_pid"
        fi

        # Launch server in background
        _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet" &
        _server_pid=$!
        _SERVE_BG_PIDS="$_SERVE_BG_PIDS $_server_pid"

        [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"

        wait "$_server_pid" 2>/dev/null || true
        return 0
    fi

    # =========================================================================
    # 3. Modo Estático Padrão (cccp serve [diretório])
    # =========================================================================
    _target_dir="$(_serve_detect_target_dir "$_target_arg")"
    _serve_display_banner "static" "$_target_dir" "$_host" "$_port"
    [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
    _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet"
}

cmd_serve_help() {
    printf "Uso: cccp serve [diretório] [opções]\n\n"
    printf "Servidor HTTP unificado de desenvolvimento e preview sob a estética Gosplan.\n\n"
    printf "Modos de operação:\n"
    printf "  (padrão)         Serve arquivos estáticos autodetectando ./site, ./dist, ./public ou ./\n"
    printf "  --dev            Modo desenvolvimento (watch de build, PHP embutido ou Node scripts)\n"
    printf "  --demo           Modo demonstração VDNKh com fixtures e abertura automática do navegador\n\n"
    printf "Opções de rede e servidor:\n"
    printf "  -l, --localhost  Restringe o bind para 127.0.0.1 (padrão é 0.0.0.0 acessível na LAN/VPN)\n"
    printf "  --host <IP>      Endereço IP personalizado para bind\n"
    printf "  -p, --port <N>   Porta inicial (padrão: 8080, incrementa automaticamente se ocupada)\n"
    printf "  -o, --open       Abre o navegador padrão automaticamente via xdg-open/open\n"
    printf "  -q, --quiet      Modo silencioso (suprime logs de acesso por requisição)\n"
    printf "  -h, --help       Exibe esta mensagem de ajuda\n"
}


# =============================================================================
# Act Local CI Runner & Pre-Push Gate
# =============================================================================
_act_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        ACT_C_RED='\033[0;31m'
        ACT_C_GREEN='\033[0;32m'
        ACT_C_YELLOW='\033[0;33m'
        ACT_C_CYAN='\033[0;36m'
        ACT_C_BOLD='\033[1m'
        ACT_C_RESET='\033[0m'
    else
        ACT_C_RED=''
        ACT_C_GREEN=''
        ACT_C_YELLOW=''
        ACT_C_CYAN=''
        ACT_C_BOLD=''
        ACT_C_RESET=''
    fi
}

_act_format_time() {
    _total_sec="${1:-0}"
    case "$_total_sec" in
        ''|*[!0-9]*) _total_sec=0 ;;
    esac
    _min=$((_total_sec / 60))
    _sec=$((_total_sec % 60))
    printf '%dm %02ds\n' "$_min" "$_sec"
}

_act_check_prerequisites() {
    _act_bin="${CCCP_ACT_BIN:-act}"
    if ! command -v "$_act_bin" >/dev/null 2>&1; then
        return 1
    fi

    # Check Docker daemon availability
    if command -v docker >/dev/null 2>&1; then
        if docker info >/dev/null 2>&1 || docker ps >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 2
}

_act_find_workflow() {
    _git_root="${1:-}"
    if [ -z "$_git_root" ]; then
        _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    fi
    [ -z "$_git_root" ] && return 1

    # Check Forgejo workflows first, then GitHub
    for _candidate in \
        "$_git_root/.forgejo/workflows/ci.yml" \
        "$_git_root/.forgejo/workflows/ci.yaml" \
        "$_git_root/.forgejo/workflows/test.yml" \
        "$_git_root/.forgejo/workflows/test.yaml" \
        "$_git_root/.github/workflows/ci.yml" \
        "$_git_root/.github/workflows/ci.yaml" \
        "$_git_root/.github/workflows/test.yml" \
        "$_git_root/.github/workflows/test.yaml"; do
        if [ -f "$_candidate" ]; then
            printf '%s\n' "$_candidate"
            return 0
        fi
    done

    # Check any .yml or .yaml in workflow directories
    for _wf_dir in "$_git_root/.forgejo/workflows" "$_git_root/.github/workflows"; do
        if [ -d "$_wf_dir" ]; then
            for _f in "$_wf_dir"/*.yml "$_wf_dir"/*.yaml; do
                if [ -f "$_f" ]; then
                    printf '%s\n' "$_f"
                    return 0
                fi
            done
        fi
    done

    return 1
}

_act_get_log_dir() {
    _git_dir=$(git rev-parse --git-dir 2>/dev/null || true)
    if [ -z "$_git_dir" ]; then
        _git_dir=".git"
    fi
    _target_dir="$_git_dir/act-logs"
    if [ ! -d "$_target_dir" ]; then
        mkdir -p "$_target_dir" 2>/dev/null || true
    fi
    printf '%s\n' "$_target_dir"
}

act_ci_gate() {
    _remote="${1:-origin}"
    _remote_url="${2:-}"

    _act_colors

    # 1. Conscious bypass check via environment
    if [ -n "${CCCP_ACT_SKIP:-}" ] || [ -n "${ACT_SKIP:-}" ]; then
        printf '%b[pre-push] CI gate bypassed via environment directive.%b\n' "$ACT_C_YELLOW" "$ACT_C_RESET"
        return 0
    fi

    # 2. Check if repository has CI workflow
    _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
    if [ -z "$_git_root" ] || ! _act_find_workflow "$_git_root" >/dev/null 2>&1; then
        # No CI workflow found; allow push without running runner
        return 0
    fi

    # 3. Prerequisites check: act & docker (Graceful degradation)
    if ! _act_check_prerequisites; then
        printf '%b[pre-push] Aviso: '\''act'\'' ou Docker indisponível. Pulando CI gate local (degradação graciosa).%b\n' "$ACT_C_YELLOW" "$ACT_C_RESET"
        return 0
    fi

    # 4. Prepare structured log file path (.git/act-logs/<YYYY-MM-DD>_<HH-MM>_<branch>_<short-sha>.log)
    _log_dir=$(_act_get_log_dir)
    _branch=$(git symbolic-ref --short HEAD 2>/dev/null || echo "head" | tr '/' '-')
    _sha=$(git rev-parse --short HEAD 2>/dev/null || echo "0000000")
    _ts=$(date '+%Y-%m-%d_%H-%M')
    _logfile="${_log_dir}/${_ts}_${_branch}_${_sha}.log"

    _job_name="${CCCP_ACT_JOB:-test}"
    _act_bin="${CCCP_ACT_BIN:-act}"

    _t_start=$(date +%s 2>/dev/null || echo 0)

    # 5. Execute act with live feedback
    if [ -t 1 ] && [ "${CI:-}" != "true" ] && [ "${SHELLSPEC:-}" != "1" ]; then
        printf '%b[pre-push] Running CI gate locally...%b\n' "$ACT_C_CYAN" "$ACT_C_RESET"
        printf '  Job: %s\n' "$_job_name"
        printf '  Log: %s\n' "$_logfile"

        # Run in background to display interactive spinner and timer
        "$_act_bin" -j "$_job_name" --no-pull > "$_logfile" 2>&1 &
        _act_pid=$!

        _spin_chars='|/-\'
        _spin_idx=0
        while kill -0 "$_act_pid" 2>/dev/null; do
            _t_curr=$(date +%s 2>/dev/null || echo 0)
            _t_diff=$((_t_curr - _t_start))
            [ "$_t_diff" -lt 0 ] && _t_diff=0
            _time_disp=$(_act_format_time "$_t_diff")

            _spin_char=$(printf '%s' "$_spin_chars" | cut -c$(( (_spin_idx % 4) + 1 )))
            _spin_idx=$((_spin_idx + 1))

            _step_name=$(tail -n 10 "$_logfile" 2>/dev/null | grep -E '(⭐ Run|\[.*\])' | tail -n 1 | sed -e 's/^[[:space:]]*//' -e 's/\x1b\[[0-9;]*m//g' | cut -c1-35 || true)
            [ -z "$_step_name" ] && _step_name="Executing workflow..."

            printf '\r  %b%s%b Job: %s › %-35s [%s]   ' "$ACT_C_CYAN" "$_spin_char" "$ACT_C_RESET" "$_job_name" "$_step_name" "$_time_disp"
            sleep 0.2 2>/dev/null || sleep 1
        done

        wait "$_act_pid"
        _exit_code=$?
        printf '\r%80s\r' "" # clear spinner line
    else
        # Non-interactive or test environment
        "$_act_bin" -j "$_job_name" --no-pull > "$_logfile" 2>&1
        _exit_code=$?
    fi

    _t_end=$(date +%s 2>/dev/null || echo 0)
    _elapsed_sec=$((_t_end - _t_start))
    [ "$_elapsed_sec" -lt 0 ] && _elapsed_sec=0
    _elapsed_fmt=$(_act_format_time "$_elapsed_sec")

    # 6. Evaluation & Summary
    if [ "$_exit_code" -eq 0 ]; then
        printf '%b✅ CI gate passed (%s) — pushing to %s/%s%b\n' "$ACT_C_GREEN" "$_elapsed_fmt" "$_remote" "$_branch" "$ACT_C_RESET"
        return 0
    else
        _failed_step=$(grep -iE '(##\[error\]|error:|fatal:|failure:|failed:)' "$_logfile" 2>/dev/null | tail -n 1 | sed -e 's/^[[:space:]]*//' -e 's/\x1b\[[0-9;]*m//g' | cut -c1-70 || true)
        [ -z "$_failed_step" ] && _failed_step="job '${_job_name}' failed with code ${_exit_code}"

        printf '\n%b❌ CI gate failed (%s) — push aborted%b\n' "$ACT_C_RED" "$_elapsed_fmt" "$ACT_C_RESET"
        printf '   → Step failed: %s\n' "$_failed_step"
        printf '   → Log: %s\n' "$_logfile"
        printf '   → Bypass (conscious): %bgit push --no-verify%b\n\n' "$ACT_C_BOLD" "$ACT_C_RESET"
        return 1
    fi
}

cmd_act() {
    _act_colors

    _subcmd="${1:-}"
    case "$_subcmd" in
        "log"|"logs")
            shift || true
            _follow=0
            while [ $# -gt 0 ]; do
                case "$1" in
                    "-f"|"--follow")
                        _follow=1
                        shift
                        ;;
                    "-h"|"--help")
                        echo "Usage: cccp act log [-f|--follow]"
                        return 0
                        ;;
                    *)
                        shift
                        ;;
                esac
            done

            _log_dir=$(_act_get_log_dir)
            _latest_log=$(ls -t "$_log_dir"/*.log 2>/dev/null | head -n 1 || true)

            if [ -z "$_latest_log" ] || [ ! -f "$_latest_log" ]; then
                echo "Error: No act CI logs found in $_log_dir" >&2
                return 1
            fi

            printf '%b=== Most Recent CI Log: %s ===%b\n' "$ACT_C_BOLD" "$_latest_log" "$ACT_C_RESET"
            if [ "$_follow" -eq 1 ]; then
                exec tail -f "$_latest_log"
            else
                cat "$_latest_log"
            fi
            return 0
            ;;

        "run")
            shift || true
            _run_job="${CCCP_ACT_JOB:-test}"
            while [ $# -gt 0 ]; do
                case "$1" in
                    "-j"|"--job")
                        shift
                        [ $# -gt 0 ] || { echo "Error: --job requires a job name" >&2; return 1; }
                        _run_job="$1"
                        shift
                        ;;
                    *)
                        _run_job="$1"
                        shift
                        ;;
                esac
            done
            CCCP_ACT_JOB="$_run_job" act_ci_gate "local" "manual"
            return $?
            ;;

        "status")
            echo "========================================================"
            echo " ★ CCCP Act Local CI Runner Status ★"
            echo "========================================================"
            _act_bin="${CCCP_ACT_BIN:-act}"
            if command -v "$_act_bin" >/dev/null 2>&1; then
                _act_loc=$(command -v "$_act_bin")
                echo " Act Runner Binary: $_act_loc (available)"
            else
                echo " Act Runner Binary: not installed (optional, runs in Docker)"
            fi

            if command -v docker >/dev/null 2>&1 && (docker info >/dev/null 2>&1 || docker ps >/dev/null 2>&1); then
                echo " Docker Daemon:     running (operational)"
            else
                echo " Docker Daemon:     not reachable / stopped"
            fi

            _git_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
            if [ -n "$_git_root" ]; then
                _wf=$(_act_find_workflow "$_git_root" 2>/dev/null || true)
                if [ -n "$_wf" ]; then
                    echo " Active Workflow:   $_wf"
                else
                    echo " Active Workflow:   none found"
                fi
            fi

            _log_dir=$(_act_get_log_dir)
            _log_count=$(ls "$_log_dir"/*.log 2>/dev/null | wc -l | tr -d ' ' || echo 0)
            echo " Recorded CI Logs:  $_log_count logs in $_log_dir"
            echo "========================================================"
            return 0
            ;;

        "-h"|"--help"|"help"|"")
            if command -v show_help >/dev/null 2>&1; then
                show_help "act"
            else
                echo "Usage: cccp act [log|run|status|help] [options]"
            fi
            return 0
            ;;

        *)
            echo "Error: Unknown act subcommand: $_subcmd" >&2
            echo "Run 'cccp act help' for usage." >&2
            return 1
            ;;
    esac
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

    local commands="status cz commit lint config version tag push tickets sync-tickets ticket issue agent update-agent sync-agent agents update-agents sync-agents codeberg set-av catalog sync-catalog diva divan web comrades camaradas comrade check gost serve repo repos act changelog update install completion hooks soviet help"
    local config_keys="types scopes subscopes strict_types strict_scopes strict_subscopes disable_subscopes disable_multiple_scopes default_base_version no_v update_channel update_interval_days check_updates post_push_hook post_push_sync post_push_sync_tickets post_push_sync_agent post_push_sync_agents tickets_file agent_md_url agents_md_url agent_md_file agents_md_file"

    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return 0
    fi

    local cmd="${COMP_WORDS[1]}"
    case "$cmd" in
        diva|divan)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--web -w --dry-run --model -m --temp -t --repo -r -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "config" -- "$cur"))
            fi
            ;;
        web)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-p --port --host -o --open -t --token --key --api-key --no-open -q --quiet -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "diva status tickets" -- "$cur"))
            fi
            ;;
        check)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--strict -C --dir -h --help" -- "$cur"))
            fi
            ;;
        gost)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-C --dir -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "sync check audit upgrade init info status help" -- "$cur"))
            fi
            ;;
        serve)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dev --demo -l --localhost --host -p --port -o --open -q --quiet -h --help" -- "$cur"))
            fi
            ;;
        repo|repos)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dirty --unpushed --issues --tickets --json -d --dir -c --catalog -f --fetch -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "status list overview help" -- "$cur"))
            fi
            ;;
        act)
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "-f --follow -j --job -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "log logs run status help" -- "$cur"))
            fi
            ;;
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
                COMPREPLY=($(compgen -W "-f --file -o --owner -r --repo -p --provider -m --max-bytes -t --token -h --help" -- "$cur"))
            fi
            ;;
        ticket|issue|diva|divan)
            case "$prev" in
                -m|--model|-t|--temp|-r|--repo)
                    return 0
                    ;;
            esac
            if [ "${cur#*-}" != "$cur" ]; then
                COMPREPLY=($(compgen -W "--dry-run --model -m --temp -t --repo -r -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "config" -- "$cur"))
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
        'ticket:Interactive issue creation via Diva with GOST branch suggestion'
        'issue:Interactive issue creation via Diva with GOST branch suggestion'
        'diva:Gemini-powered issue triage bot with multi-forge dispatch'
        'divan:Gemini-powered issue triage bot with multi-forge dispatch'
        'catalog:Inventory workspaces, update description and sync forges'
        'comrades:Manage micro ecosystem tools (kvant, svyaz, mayak)'
        'agent:Generate and synchronize AGENTS.md directives'
        'update-agent:Generate and synchronize AGENTS.md directives'
        'sync-agent:Generate and synchronize AGENTS.md directives'
        'agents:Generate and synchronize AGENTS.md directives'
        'update-agents:Generate and synchronize AGENTS.md directives'
        'sync-agents:Generate and synchronize AGENTS.md directives'
        'set-av:Fast shortcut to configure local agent variable'
        'codeberg:Configure Codeberg integration and AGENTS template wizard'
        'catalog:Inventory workspaces, update .git/description and sync forges'
        'diva:Gemini-powered issue triage bot with multi-forge dispatch'
        'divan:Gemini-powered issue triage bot with multi-forge dispatch'
        'web:Interactive Gosplan Web Cockpit via Mayak Command Bridge'
        'comrades:Manage micro ecosystem tools (kvant and svyaz)'
        'check:Audit project compliance against GOST standard'
        'gost:Manage GOST synchronization, audit and upgrades'
        'serve:Unified HTTP server (static, dev with watcher, VDNKh demo)'
        'repo:Gosplan multi-repository aggregated management and overview'
        'act:Local CI runner, gate and structured log viewer'
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
                ticket|issue|diva|divan)
                    _arguments \
                        '--dry-run[Simulate triage and preview tickets without forge publication]' \
                        '(-m --model)'{-m,--model}'[Gemini model name]:model:' \
                        '(-t --temp)'{-t,--temp}'[Sampling temperature]:temperature:' \
                        '(-r --repo)'{-r,--repo}'[Target repository slug]:repo:' \
                        '1:problem description or config:'
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
                repo|repos)
                    _arguments \
                        '--dirty[List only repositories with uncommitted/unstaged changes]' \
                        '--unpushed[List only repositories with commits pending push]' \
                        '--issues[List only repositories with open tickets/issues]' \
                        '--tickets[List only repositories with open tickets/issues]' \
                        '--json[Output aggregated status as structured JSON]' \
                        '(-d --dir)'{-d,--dir}'[Target directory to scan for repositories]:directory:_directories' \
                        '(-c --catalog)'{-c,--catalog}'[Catalog file to use]:catalog file:_files' \
                        '(-f --fetch)'{-f,--fetch}'[Query remote forge for live issue counts]' \
                        '(-h --help)'{-h,--help}'[Display help message]' \
                        '1:subcommand:(status list overview help)'
                    ;;
                act)
                    _arguments \
                        '(-f --follow)'{-f,--follow}'[Follow log stream in real time]' \
                        '(-j --job)'{-j,--job}'[Target workflow job name]:job name:' \
                        '(-h --help)'{-h,--help}'[Display help message]' \
                        '1:subcommand:(log logs run status help)'
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
complete -c cccp -n "__fish_use_subcommand" -a ticket -d "Interactive issue creation via Diva with GOST branch suggestion"
complete -c cccp -n "__fish_use_subcommand" -a issue -d "Interactive issue creation via Diva with GOST branch suggestion"
complete -c cccp -n "__fish_use_subcommand" -a diva -d "Gemini-powered issue triage bot with multi-forge dispatch"
complete -c cccp -n "__fish_use_subcommand" -a divan -d "Gemini-powered issue triage bot with multi-forge dispatch"
complete -c cccp -n "__fish_use_subcommand" -a catalog -d "Inventory workspaces, update description and sync forges"
complete -c cccp -n "__fish_use_subcommand" -a comrades -d "Manage micro ecosystem tools (kvant, svyaz, mayak)"
complete -c cccp -n "__fish_use_subcommand" -a check -d "Audit project compliance against GOST standard"
complete -c cccp -n "__fish_use_subcommand" -a gost -d "Manage GOST synchronization, audit and upgrades"
complete -c cccp -n "__fish_use_subcommand" -a serve -d "Unified HTTP server (static, dev with watcher, VDNKh demo)"
complete -c cccp -n "__fish_use_subcommand" -a repo -d "Gosplan multi-repository management and overview"
complete -c cccp -n "__fish_use_subcommand" -a act -d "Local CI runner, gate and structured log viewer"
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
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s o -l owner -d "Repository owner"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s r -l repo -d "Repository name"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s p -l provider -d "Provider (github or codeberg)"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s m -l max-bytes -d "Maximum budget in bytes"
complete -c cccp -n "__fish_seen_subcommand_from tickets sync-tickets" -s t -l token -d "Personal access token"

# Options for ticket and diva
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -l dry-run -d "Simulate triage without forge publication"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s m -l model -d "Gemini model name"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s t -l temp -d "Sampling temperature"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -s r -l repo -d "Target repository slug"
complete -c cccp -n "__fish_seen_subcommand_from ticket issue diva divan" -a "config" -d "Configure Gemini API key"

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

# Options for diva
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s w -l web -d "Open interactive triage cockpit in web browser"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -l dry-run -d "Simulate triage without forge publication"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s m -l model -d "Gemini model name"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s t -l temp -d "Sampling temperature"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -s r -l repo -d "Target repository slug"
complete -c cccp -n "__fish_seen_subcommand_from diva divan" -a "config" -d "Persist GEMINI_API_KEY in .cccprc"

# Options for web
complete -c cccp -n "__fish_seen_subcommand_from web" -s p -l port -d "Port to listen on"
complete -c cccp -n "__fish_seen_subcommand_from web" -l host -d "Host address to bind to"
complete -c cccp -n "__fish_seen_subcommand_from web" -s o -l open -d "Open browser automatically"
complete -c cccp -n "__fish_seen_subcommand_from web" -s t -l token -d "API authentication token"
complete -c cccp -n "__fish_seen_subcommand_from web" -l no-open -d "Do not open browser"
complete -c cccp -n "__fish_seen_subcommand_from web" -s q -l quiet -d "Quiet mode"
complete -c cccp -n "__fish_seen_subcommand_from web" -a "diva status tickets"

# Options for repo
complete -c cccp -n "__fish_seen_subcommand_from repo" -l dirty -d "List only repositories with uncommitted/unstaged changes"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l unpushed -d "List only repositories with commits pending push"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l issues -d "List only repositories with open tickets"
complete -c cccp -n "__fish_seen_subcommand_from repo" -l json -d "Output aggregated status as JSON"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s d -l dir -d "Target directory to scan for repositories"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s c -l catalog -d "Catalog file to use"
complete -c cccp -n "__fish_seen_subcommand_from repo" -s f -l fetch -d "Query remote forge for live issue counts"
complete -c cccp -n "__fish_seen_subcommand_from repo" -a "status list overview help"

# Options for act
complete -c cccp -n "__fish_seen_subcommand_from act" -s f -l follow -d "Follow log stream in real time"
complete -c cccp -n "__fish_seen_subcommand_from act" -s j -l job -d "Target workflow job name"
complete -c cccp -n "__fish_seen_subcommand_from act" -a "log logs run status help"

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

    # Global directory override option (-C <path|repo>)
    while [ $# -gt 0 ]; do
        case "$1" in
            -C|--dir)
                shift
                [ $# -gt 0 ] || { echo "Error: $1 requires directory path or repository name" >&2; return 1; }
                _target_repo="$1"
                _resolved_dir=""
                if [ -d "$_target_repo" ]; then
                    _resolved_dir="$_target_repo"
                elif [ -d "${HOME}/src/${_target_repo}" ]; then
                    _resolved_dir="${HOME}/src/${_target_repo}"
                fi

                if [ -z "$_resolved_dir" ] && command -v _catalog_resolve_target_dir >/dev/null 2>&1; then
                    _base_cat=$(_catalog_resolve_target_dir "" 2>/dev/null || echo "")
                    if [ -n "$_base_cat" ] && [ -d "${_base_cat}/${_target_repo}" ]; then
                        _resolved_dir="${_base_cat}/${_target_repo}"
                    fi
                fi

                if [ -n "$_resolved_dir" ]; then
                    cd "$_resolved_dir" || { echo "Error: Failed to change directory to '$_resolved_dir'" >&2; return 1; }
                else
                    echo "Error: Repository directory '$_target_repo' not found" >&2
                    return 1
                fi
                shift
                ;;
            *)
                break
                ;;
        esac
    done

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
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"commit-msg"|"post-commit"|"pre-push"|"reference-transaction"|"hooks"|"catalog")
                check_system_tools || exit 1
                ;;
        esac
    fi

    # Non-blocking periodic update check on interactive user commands
    if command -v check_auto_update >/dev/null 2>&1; then
        case "$command" in
            "commit"|"cz"|"version"|"tag"|"push"|"changelog"|"config"|"status"|"lint"|"completion"|"hooks"|"tickets"|"sync-tickets"|"agents"|"update-agents"|"set-av"|"catalog"|"sync-catalog"|"diva"|"divan"|"ticket"|"issue"|"matraca"|"cockpit"|"web"|"gost"|"check"|"serve"|"newrepo"|"new-repo")
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
        "catalog"|"sync-catalog")
            shift || true
            cmd_catalog "$@"
            exit 0
            ;;
        "ticket"|"issue"|"diva"|"divan")
            shift || true
            cmd_ticket "$@"
            exit 0
            ;;
        "matraca")
            shift || true
            cmd_matraca "$@"
            exit $?
            ;;
        "cockpit"|"web")
            shift || true
            cmd_cockpit "$@"
            exit $?
            ;;
        "comrades"|"camaradas"|"comrade")
            shift || true
            cmd_comrades "$@"
            exit 0
            ;;
        "soviet"|"sputnik"|"anthem"|"gosplan")
            show_soviet
            exit 0
            ;;
        "check")
            shift || true
            cmd_check "$@"
            exit $?
            ;;
        "gost")
            shift || true
            cmd_gost "$@"
            exit $?
            ;;
        "serve")
            shift || true
            cmd_serve "$@"
            exit $?
            ;;
        "repo"|"repos")
            shift || true
            cmd_repo "$@"
            exit $?
            ;;
        "newrepo"|"new-repo")
            shift || true
            cmd_newrepo "$@"
            exit $?
            ;;
        "act")
            shift || true
            cmd_act "$@"
            exit $?
            ;;
        "help"|"-h"|"--help")
            show_help "${2:-}"
            exit 0
            ;;
        *)
            echo "Usage: $0 [git|commit|cz|install|config|status|lint|completion|version|tag|push|tickets|ticket|issue|agent|set-av|catalog|diva|matraca|cockpit|web|comrades|changelog|commit-msg|post-commit|update|check-update|hooks|gost|check|serve|repo|newrepo|act|help]"
            echo "Run '$0 help' or '$0 help <command>' for more information."
            exit 1
            ;;
    esac
}

if [ "${CCCP_SOURCED:-0}" != "1" ]; then
    main "$@"
fi
