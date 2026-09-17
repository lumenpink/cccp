#!/bin/sh

# Enable error handling
set -eu

# Source dependencies if available
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ] && [ -f "$GIT_ROOT/src/config/config.sh" ]; then
    . "$GIT_ROOT/src/config/config.sh"
fi

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

# -----------------------------------------------------------------------------
# Hook version audit: verify that repo hooks match current cccp version
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Auto-check for updates every N days (non-blocking)
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Check update status command: cccp check-update
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Function to update the script binary
# -----------------------------------------------------------------------------
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
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -f "$local_cfg" ]; then
            unset_file_key "$local_cfg" "pinned_version" 2>/dev/null || true
            unset_file_key "$local_cfg" "pin_version" 2>/dev/null || true
        fi
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
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -n "${GIT_ROOT:-}" ] && [ -d "$GIT_ROOT/.git" ]; then
            write_file_key "$local_cfg" "pinned_version" "$clean_pin"
        else
            global_cfg=$(get_global_config_file 2>/dev/null || true)
            write_file_key "$global_cfg" "pinned_version" "$clean_pin"
        fi
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