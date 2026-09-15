#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$GIT_ROOT" ]; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Set up paths relative to git root
GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"

# Source dependencies if available
if [ -f "$GIT_ROOT/src/config/config.sh" ]; then
    . "$GIT_ROOT/src/config/config.sh"
fi
if [ -f "$GIT_ROOT/src/utils/version.sh" ]; then
    . "$GIT_ROOT/src/utils/version.sh"
fi
if [ -f "$GIT_ROOT/src/utils/changelog.sh" ]; then
    . "$GIT_ROOT/src/utils/changelog.sh"
fi

# -----------------------------------------------------------------------------
# Create release tag, commit updated VERSION/CHANGELOG, and tag the release commit
# -----------------------------------------------------------------------------
create_tag() {
    GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$GIT_ROOT" ]; then
        echo "Error: Not a git repository" >&2
        return 1
    fi

    if command -v check_prerequisites >/dev/null 2>&1; then
        check_prerequisites
    fi

    # Ensure working tree has no uncommitted changes to tracked files
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        echo "Error: Working directory has uncommitted changes. Please commit or stash them before creating a tag." >&2
        return 1
    fi

    target_ver=""
    no_prefix=${NO_V:-0}
    message=""

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
    echo "Tag '$tag_name' created successfully."
    echo "Updated VERSION: $clean_ver"
    echo "Updated CHANGELOG.md"
    echo ""
    echo "To push the release and tag to remote, run:"
    echo "  git push origin $current_branch"
    echo "  git push origin $tag_name"
}
