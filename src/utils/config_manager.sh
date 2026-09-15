#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Configuration Paths
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Low-level key/value file operations
# -----------------------------------------------------------------------------
normalize_config_key() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | tr '-' '_'
}

get_default_config_value() {
    case "$1" in
        strict_scopes) echo "0" ;;
        strict_subscopes) echo "0" ;;
        disable_subscopes) echo "0" ;;
        disable_multiple_scopes) echo "0" ;;
        default_base_version) echo "0.0.1" ;;
        no_v) echo "0" ;;
        update_channel) echo "stable" ;;
        update_interval_days) echo "30" ;;
        check_updates) echo "1" ;;
        allow_any_scope) echo "1" ;;
        allow_any_subscope) echo "1" ;;
        *) return 1 ;;
    esac
}

read_file_key() {
    file="$1"
    raw_key="$2"
    key=$(normalize_config_key "$raw_key")

    if [ ! -f "$file" ]; then
        return 1
    fi

    # Read line ignoring comments and empty lines
    awk -v target="$key" '
    BEGIN { found = 0 }
    /^[[:space:]]*[#;]/ { next }
    /^[[:space:]]*$/ { next }
    {
        split($0, parts, "=")
        k = parts[1]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
        k = tolower(k)
        gsub(/-/, "_", k)
        if (k == target) {
            # Recombine value in case value contains "="
            val = substr($0, index($0, "=") + 1)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", val)
            # Remove surrounding quotes
            sub(/^"/, "", val); sub(/"$/, "", val)
            sub(/^\x27/, "", val); sub(/\x27$/, "", val)
            print val
            found = 1
            exit
        }
    }
    END { if (!found) exit 1 }
    ' "$file"
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

    # Check if key already exists
    if grep -qE "^[[:space:]]*${key}[[:space:]]*=" "$file" 2>/dev/null; then
        # Replace existing key
        tmp_file="${file}.tmp.$$"
        awk -v target="$key" -v val="$value" '
        BEGIN { replaced = 0 }
        /^[[:space:]]*[#;]/ { print; next }
        {
            split($0, parts, "=")
            k = parts[1]
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
            k = tolower(k)
            gsub(/-/, "_", k)
            if (k == target && !replaced) {
                print target " = " val
                replaced = 1
                next
            }
            print
        }
        ' "$file" > "$tmp_file"
        mv "$tmp_file" "$file"
    else
        # Append new key
        echo "$key = $value" >> "$file"
    fi
}

unset_file_key() {
    file="$1"
    raw_key="$2"
    key=$(normalize_config_key "$raw_key")

    if [ ! -f "$file" ]; then
        return 0
    fi

    tmp_file="${file}.tmp.$$"
    awk -v target="$key" '
    {
        split($0, parts, "=")
        k = parts[1]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
        k = tolower(k)
        gsub(/-/, "_", k)
        if (k == target) {
            next
        }
        print
    }
    ' "$file" > "$tmp_file"
    mv "$tmp_file" "$file"
}

list_file_keys() {
    file="$1"
    prefix="$2"

    if [ ! -f "$file" ]; then
        return 0
    fi

    awk -v pfx="$prefix" '
    /^[[:space:]]*[#;]/ { next }
    /^[[:space:]]*$/ { next }
    /=/ {
        split($0, parts, "=")
        k = parts[1]
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
        k = tolower(k)
        gsub(/-/, "_", k)
        val = substr($0, index($0, "=") + 1)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", val)
        sub(/^"/, "", val); sub(/"$/, "", val)
        sub(/^\x27/, "", val); sub(/\x27$/, "", val)
        print "[" pfx "] " k " = " val
    }
    ' "$file"
}

# -----------------------------------------------------------------------------
# Hierarchical config loading: Defaults < Global < Local < Environment
# -----------------------------------------------------------------------------
load_hierarchical_config() {
    # Capture any pre-existing environment variables
    env_strict_scopes="${STRICT_SCOPES:-}"
    env_strict_subscopes="${STRICT_SUBSCOPES:-}"
    env_disable_subscopes="${DISABLE_SUBSCOPES:-}"
    env_disable_multi="${DISABLE_MULTIPLE_SCOPES:-}"
    env_base_ver="${DEFAULT_BASE_VERSION:-}"
    env_no_v="${NO_V:-}"
    env_channel="${UPDATE_CHANNEL:-}"
    env_interval="${UPDATE_INTERVAL_DAYS:-}"
    env_check="${CHECK_UPDATES:-}"
    env_allow_any_scope="${ALLOW_ANY_SCOPE:-}"
    env_allow_any_subscope="${ALLOW_ANY_SUBSCOPE:-}"

    # Default internal values
    STRICT_SCOPES="0"
    STRICT_SUBSCOPES="0"
    DISABLE_SUBSCOPES="0"
    DISABLE_MULTIPLE_SCOPES="0"
    DEFAULT_BASE_VERSION="0.0.1"
    NO_V="0"
    UPDATE_CHANNEL="stable"
    UPDATE_INTERVAL_DAYS="30"
    CHECK_UPDATES="1"

    global_file=$(get_global_config_file)
    local_file=$(get_local_config_file)

    # 1. Global config file overrides
    if [ -f "$global_file" ]; then
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
    fi

    # 2. Local repository config (.cccprc) overrides global
    if [ -n "$local_file" ] && [ -f "$local_file" ]; then
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

        l_channel=$(read_file_key "$local_file" "update_channel" 2>/dev/null || true)
        [ -n "$l_channel" ] && UPDATE_CHANNEL="$l_channel"

        l_interval=$(read_file_key "$local_file" "update_interval_days" 2>/dev/null || true)
        [ -n "$l_interval" ] && UPDATE_INTERVAL_DAYS="$l_interval"

        l_check=$(read_file_key "$local_file" "check_updates" 2>/dev/null || true)
        [ -n "$l_check" ] && CHECK_UPDATES="$l_check"
    fi

    # 3. Environment variables take highest precedence
    [ -n "$env_strict_scopes" ] && STRICT_SCOPES="$env_strict_scopes"
    [ -n "$env_strict_subscopes" ] && STRICT_SUBSCOPES="$env_strict_subscopes"
    [ -n "$env_disable_subscopes" ] && DISABLE_SUBSCOPES="$env_disable_subscopes"
    [ -n "$env_disable_multi" ] && DISABLE_MULTIPLE_SCOPES="$env_disable_multi"
    [ -n "$env_base_ver" ] && DEFAULT_BASE_VERSION="$env_base_ver"
    [ -n "$env_no_v" ] && NO_V="$env_no_v"
    [ -n "$env_channel" ] && UPDATE_CHANNEL="$env_channel"
    [ -n "$env_interval" ] && UPDATE_INTERVAL_DAYS="$env_interval"
    [ -n "$env_check" ] && CHECK_UPDATES="$env_check"

    # Legacy environment overrides
    if [ "$env_allow_any_scope" = "0" ]; then
        STRICT_SCOPES=1
    elif [ "$env_allow_any_scope" = "1" ]; then
        STRICT_SCOPES=0
    fi

    if [ "$env_allow_any_subscope" = "0" ]; then
        STRICT_SUBSCOPES=1
    elif [ "$env_allow_any_subscope" = "1" ]; then
        STRICT_SUBSCOPES=0
    fi

    ALLOW_ANY_SCOPE=$([ "$STRICT_SCOPES" = "1" ] && echo "0" || echo "1")
    ALLOW_ANY_SUBSCOPE=$([ "$STRICT_SUBSCOPES" = "1" ] && echo "0" || echo "1")
}

# -----------------------------------------------------------------------------
# CLI command: cccp config
# -----------------------------------------------------------------------------
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
        # Cascaded get: local first, then global, then default
        val=""
        if [ -n "$local_file" ] && [ -f "$local_file" ]; then
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
