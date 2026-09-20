#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    if [ -f "$GIT_ROOT/src/config/config.sh" ]; then
        . "$GIT_ROOT/src/config/config.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/config_manager.sh" ]; then
        . "$GIT_ROOT/src/utils/config_manager.sh"
    fi
fi

# Default canonical template URL
DEFAULT_AGENTS_MD_URL="https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md"

# -----------------------------------------------------------------------------
# Helper: Fetch template content from URL or local path
# -----------------------------------------------------------------------------
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
                if [ -n "$content" ]; then
                    printf '%s\n' "$content"
                    return 0
                fi
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

# -----------------------------------------------------------------------------
# Helper: Extract unique variable names from template text
# Supports both {{VAR_NAME}} and ${VAR_NAME}
# -----------------------------------------------------------------------------
extract_template_vars() {
    template_text="$1"
    printf '%s\n' "$template_text" | grep -oE '(\{\{[A-Za-z0-9_]+\}|\$\{[A-Za-z0-9_]+\})' 2>/dev/null | \
        sed -E -e 's/^(\{\{|\$\{)//' -e 's/(\}\}|\})$//' | \
        sort -u || true
}

# -----------------------------------------------------------------------------
# Helper: Resolve variable value from environment or local .cccprc
# -----------------------------------------------------------------------------
resolve_agent_var() {
    var_name="$1"
    norm_name=$(echo "$var_name" | tr '[:upper:]' '[:lower:]' | tr '-' '_')
    up_name=$(echo "$var_name" | tr '[:lower:]' '[:upper:]' | tr '-' '_')

    # 1. Environment variables have highest precedence
    # Check AGENT_VAR_NAME or direct VAR_NAME
    env_val=$(eval "printf '%s' \"\${AGENT_VAR_${up_name}:-\${AGENT_VAR_${norm_name}:-}}\"")
    if [ -n "$env_val" ]; then
        echo "$env_val"
        return 0
    fi

    # 2. Local repository configuration (.cccprc)
    local_file=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        local_file=$(get_local_config_file 2>/dev/null || true)
    fi

    if [ -n "$local_file" ] && [ -f "$local_file" ] && command -v read_file_key >/dev/null 2>&1; then
        val=$(read_file_key "$local_file" "agent_var_${norm_name}" 2>/dev/null || true)
        [ -z "$val" ] && val=$(read_file_key "$local_file" "agent_var_${up_name}" 2>/dev/null || true)
        [ -z "$val" ] && val=$(read_file_key "$local_file" "${norm_name}" 2>/dev/null || true)
        if [ -n "$val" ]; then
            echo "$val"
            return 0
        fi
    fi

    # 3. Global configuration
    global_file=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        global_file=$(get_global_config_file 2>/dev/null || true)
    fi

    if [ -n "$global_file" ] && [ -f "$global_file" ] && command -v read_file_key >/dev/null 2>&1; then
        val=$(read_file_key "$global_file" "agent_var_${norm_name}" 2>/dev/null || true)
        [ -z "$val" ] && val=$(read_file_key "$global_file" "agent_var_${up_name}" 2>/dev/null || true)
        if [ -n "$val" ]; then
            echo "$val"
            return 0
        fi
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Main agents command dispatcher
# -----------------------------------------------------------------------------
cmd_agents() {
    # Ensure config hierarchy is loaded
    if command -v load_hierarchical_config >/dev/null 2>&1; then
        load_hierarchical_config
    fi

    action="update"
    target_url=""
    target_file=""
    var_key=""
    var_val=""

    # Check for subcommands or options
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                if command -v show_help >/dev/null 2>&1; then
                    show_help "agents"
                else
                    echo "Usage: cccp agents [update|check|set|get|unset] [options]"
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
            update)
                action="update"
                shift
                ;;
            -u|--url)
                shift
                [ $# -gt 0 ] || { echo "Error: Missing argument for --url" >&2; return 1; }
                target_url="$1"
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

    # Handle variable setting (ergonomic set-av / agents set)
    if [ "$action" = "set" ]; then
        norm_key=$(echo "$var_key" | sed 's/^agent_var_//' | tr '[:upper:]' '[:lower:]' | tr '-' '_')
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -z "$local_cfg" ]; then
            echo "Error: Not inside a git repository. Cannot save to local .cccprc." >&2
            return 1
        fi
        set_file_key "$local_cfg" "agent_var_${norm_key}" "$var_val"
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

    # Resolve template URL
    if [ -z "$target_url" ]; then
        target_url="${AGENTS_MD_URL:-}"
    fi
    if [ -z "$target_url" ]; then
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -f "$local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_url=$(read_file_key "$local_cfg" "agents_md_url" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_url" ]; then
        global_cfg=$(get_global_config_file 2>/dev/null || true)
        if [ -n "$global_cfg" ] && [ -f "$global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_url=$(read_file_key "$global_cfg" "agents_md_url" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_url" ]; then
        target_url="$DEFAULT_AGENTS_MD_URL"
    fi

    # Resolve output destination file
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [ -z "$target_file" ]; then
        target_file="${AGENTS_MD_FILE:-}"
    fi
    if [ -z "$target_file" ]; then
        local_cfg=$(get_local_config_file 2>/dev/null || true)
        if [ -n "$local_cfg" ] && [ -f "$local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
            target_file=$(read_file_key "$local_cfg" "agents_md_file" 2>/dev/null || true)
        fi
    fi
    if [ -z "$target_file" ]; then
        if [ -n "$git_root" ]; then
            target_file="$git_root/AGENTS.md"
        else
            target_file="AGENTS.md"
        fi
    fi

    # Fetch template
    echo "Fetching AGENTS template from: $target_url"
    if ! template_content=$(fetch_agents_template "$target_url"); then
        echo "Error: Unable to fetch AGENTS template from '$target_url'." >&2
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
            if val=$(resolve_agent_var "$v"); then
                echo "  [✓] $v = $val"
            else
                echo "  [✗] $v = (missing - configure via: cccp agents set $norm_v \"<val>\")"
                missing_count=$((missing_count + 1))
            fi
        done
        echo ""
        if [ "$missing_count" -gt 0 ]; then
            echo "Status: $missing_count variable(s) missing. Cannot generate $target_file."
            return 1
        else
            echo "Status: All variables configured! Ready to run 'cccp agents'."
            return 0
        fi
    fi

    # Generate / update mode
    missing_vars=""
    for v in $vars; do
        if ! resolve_agent_var "$v" >/dev/null 2>&1; then
            norm_v=$(echo "$v" | tr '[:upper:]' '[:lower:]')
            missing_vars="${missing_vars}\n  - $v (configure with: cccp agents set $norm_v \"<value>\")"
        fi
    done

    if [ -n "$missing_vars" ]; then
        printf "Error: Missing required variables for %s:%b\n" "$target_file" "$missing_vars" >&2
        echo "Aborting generation of $target_file." >&2
        return 1
    fi

    # Perform interpolation
    result="$template_content"
    interpolated_count=0
    for v in $vars; do
        val=$(resolve_agent_var "$v")
        # Replace {{VAR}}
        pattern_mustache="{{${v}}}"
        # Replace ${VAR}
        pattern_shell="\${${v}}"

        # Safe multi-line string replacement using awk/sed or temp python/sh
        # We can use awk or sed with escaped delimiters
        esc_val=$(printf '%s\n' "$val" | sed -e 's/[\/&]/\\&/g')
        result=$(printf '%s\n' "$result" | sed -e "s/$pattern_mustache/$esc_val/g" -e "s/$pattern_shell/$esc_val/g")
        interpolated_count=$((interpolated_count + 1))
    done

    # Write destination file atomically
    tmp_out="${target_file}.tmp.$$"
    mkdir -p "$(dirname "$target_file")"
    printf '%s\n' "$result" > "$tmp_out"
    mv "$tmp_out" "$target_file"

    echo "Done! Generated $target_file with $interpolated_count variable(s) substituted."
    return 0
}

# -----------------------------------------------------------------------------
# Ergonomic shortcut for set-av
# -----------------------------------------------------------------------------
cmd_set_av() {
    cmd_agents set "$@"
}
