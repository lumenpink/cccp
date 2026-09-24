#!/bin/sh

# Enable error handling
set -eu

# Find the git root directory
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$GIT_ROOT" ]; then
    GIT_HOOKS_DIR="$GIT_ROOT/.git/hooks"
    if [ -f "$GIT_ROOT/src/utils/repo.sh" ]; then
        . "$GIT_ROOT/src/utils/repo.sh"
    fi
    if [ -f "$GIT_ROOT/src/config/config.sh" ]; then
        . "$GIT_ROOT/src/config/config.sh"
    fi
    if [ -f "$GIT_ROOT/src/utils/config_manager.sh" ]; then
        . "$GIT_ROOT/src/utils/config_manager.sh"
    fi
fi

# Default canonical template URL
DEFAULT_AGENT_MD_URL="https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md"
DEFAULT_AGENTS_MD_URL="$DEFAULT_AGENT_MD_URL"

# -----------------------------------------------------------------------------
# Helper: Dispatch HTTP request via sv / svyaz / curl
# -----------------------------------------------------------------------------
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
# Helper: Ensure target agent file is present in .gitignore
# -----------------------------------------------------------------------------
ensure_agent_gitignored() {
    if command -v ensure_gitignored >/dev/null 2>&1; then
        ensure_gitignored "$@"
        return $?
    fi
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Pure POSIX substring replacement without awk or sed escaping issues
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Helper: Resolve variable value from environment or local .cccprc
# Normalizes variable names with/without agent_ and agents_ prefixes
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Main agent command dispatcher (canonical singular)
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Backwards-compatible plural dispatcher
# -----------------------------------------------------------------------------
cmd_agents() {
    cmd_agent "$@"
}

# -----------------------------------------------------------------------------
# Ergonomic shortcut for set-av
# -----------------------------------------------------------------------------
cmd_set_av() {
    cmd_agent set "$@"
}
