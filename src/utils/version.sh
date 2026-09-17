#!/bin/sh

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"

# Set up paths relative to git root
GIT_HOOKS_DIR=$([ -n "$GIT_ROOT" ] && echo "$GIT_ROOT/.git/hooks" || echo "")

# Source the configuration if available
[ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/config/config.sh" ] && . "$GIT_ROOT/src/config/config.sh"

# -----------------------------------------------------------------------------
# Calculate predicted target version
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Generate predictive version information
# -----------------------------------------------------------------------------
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