#!/bin/sh

# Enable error handling
set -eu

# -----------------------------------------------------------------------------
# Helper: Enable a sync target in post_push_sync inside .cccprc
# Adds the target if not already present; creates the key if missing.
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Subcommand: setup
# Interactive wizard that detects Codeberg remote and configures .cccprc
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Main codeberg command dispatcher
# -----------------------------------------------------------------------------
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
