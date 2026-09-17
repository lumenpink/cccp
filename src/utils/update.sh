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
    # Check if update checks are enabled
    if [ "${CHECK_UPDATES:-1}" = "0" ]; then
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
# Function to update the script binary
# -----------------------------------------------------------------------------
update_script() {
    channel="${UPDATE_CHANNEL:-stable}"

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
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "update"
                else
                    echo "Usage: cccp update [--channel stable|nightly]"
                fi
                return 0
                ;;
            *)
                echo "Error: Unexpected argument '$1'" >&2
                return 1
                ;;
        esac
    done

    case "$channel" in
        stable)
            download_url="https://github.com/lumenpink/cccp.sh/releases/latest/download/cccp.sh"
            ;;
        nightly)
            download_url="https://github.com/lumenpink/cccp.sh/releases/download/nightly/cccp.sh"
            ;;
        *)
            echo "Error: Invalid update channel '$channel'. Choose 'stable' or 'nightly'." >&2
            return 1
            ;;
    esac

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