#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Display repository status and Gosplan inspection
# -----------------------------------------------------------------------------
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
        if grep -q "CCCP_HOOK_VERSION" "$hooks_dir/commit-msg" 2>/dev/null; then
            hook_ver=$(grep "CCCP_HOOK_VERSION=" "$hooks_dir/commit-msg" 2>/dev/null | cut -d'"' -f2 || echo "")
            if [ -n "$hook_ver" ]; then
                hook_status="installed (v${hook_ver})"
            else
                hook_status="installed"
            fi
        else
            hook_status="custom/non-cccp"
        fi
    elif [ -f "$hooks_dir/commit-msg" ] || [ -f "$hooks_dir/post-commit" ]; then
        hook_status="partially installed"
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
