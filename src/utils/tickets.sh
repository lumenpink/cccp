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
fi

# -----------------------------------------------------------------------------
# Helper: Extract Git provider, owner and repository from remote URL
# -----------------------------------------------------------------------------
parse_remote_url() {
    url="$1"
    clean=$(echo "$url" | sed -e 's/\.git$//' -e 's|/*$||')
    case "$clean" in
        *github.com[:/]*)
            repo_path=$(echo "$clean" | sed -e 's|^.*github\.com[:/]||' -e 's|^/*||')
            owner=$(echo "$repo_path" | cut -d'/' -f1)
            repo=$(echo "$repo_path" | cut -d'/' -f2)
            if [ -n "$owner" ] && [ -n "$repo" ]; then
                echo "github $owner $repo"
                return 0
            fi
            ;;
        *codeberg.org[:/]*)
            repo_path=$(echo "$clean" | sed -e 's|^.*codeberg\.org[:/]||' -e 's|^/*||')
            owner=$(echo "$repo_path" | cut -d'/' -f1)
            repo=$(echo "$repo_path" | cut -d'/' -f2)
            if [ -n "$owner" ] && [ -n "$repo" ]; then
                echo "codeberg $owner $repo"
                return 0
            fi
            ;;
    esac
    return 1
}

parse_codeberg_url() {
    if parsed=$(parse_remote_url "$1" 2>/dev/null); then
        prov=$(echo "$parsed" | cut -d' ' -f1)
        if [ "$prov" = "codeberg" ]; then
            echo "$parsed" | cut -d' ' -f2,3
            return 0
        fi
    fi
    return 1
}

parse_github_url() {
    if parsed=$(parse_remote_url "$1" 2>/dev/null); then
        prov=$(echo "$parsed" | cut -d' ' -f1)
        if [ "$prov" = "github" ]; then
            echo "$parsed" | cut -d' ' -f2,3
            return 0
        fi
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Resolve Git provider, owner and repo dynamically
# Priority: Flags/Env -> Remote push URL -> Remote fetch URL -> Other remotes
# -----------------------------------------------------------------------------
resolve_tickets_info() {
    resolved_provider="${TICKETS_PROVIDER:-}"
    resolved_owner="${TICKETS_OWNER:-${CODEBERG_OWNER:-${CODEBERG_USER:-${GITHUB_OWNER:-${GITHUB_USER:-}}}}}"
    resolved_repo="${TICKETS_REPO:-${CODEBERG_REPO:-${GITHUB_REPO:-}}}"

    # Inspect git remotes if inside a git repository
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        # 1. Check 'origin' push URL, then fetch URL
        origin_push_url=$(git config remote.origin.pushurl 2>/dev/null || git remote get-url --push origin 2>/dev/null || true)
        origin_fetch_url=$(git config remote.origin.url 2>/dev/null || git remote get-url origin 2>/dev/null || true)

        for u in "$origin_push_url" "$origin_fetch_url"; do
            [ -z "$u" ] && continue
            if parsed=$(parse_remote_url "$u"); then
                p_prov=$(echo "$parsed" | cut -d' ' -f1)
                p_owner=$(echo "$parsed" | cut -d' ' -f2)
                p_repo=$(echo "$parsed" | cut -d' ' -f3)
                resolved_provider="${resolved_provider:-$p_prov}"
                resolved_owner="${resolved_owner:-$p_owner}"
                resolved_repo="${resolved_repo:-$p_repo}"
                echo "$resolved_provider $resolved_owner $resolved_repo"
                return 0
            fi
        done

        # 2. Check any other configured git remotes
        all_remotes=$(git remote 2>/dev/null || true)
        for rmt in $all_remotes; do
            [ "$rmt" = "origin" ] && continue
            rmt_push_url=$(git config "remote.${rmt}.pushurl" 2>/dev/null || git remote get-url --push "$rmt" 2>/dev/null || true)
            rmt_fetch_url=$(git config "remote.${rmt}.url" 2>/dev/null || git remote get-url "$rmt" 2>/dev/null || true)
            for u in "$rmt_push_url" "$rmt_fetch_url"; do
                [ -z "$u" ] && continue
                if parsed=$(parse_remote_url "$u"); then
                    p_prov=$(echo "$parsed" | cut -d' ' -f1)
                    p_owner=$(echo "$parsed" | cut -d' ' -f2)
                    p_repo=$(echo "$parsed" | cut -d' ' -f3)
                    resolved_provider="${resolved_provider:-$p_prov}"
                    resolved_owner="${resolved_owner:-$p_owner}"
                    resolved_repo="${resolved_repo:-$p_repo}"
                    echo "$resolved_provider $resolved_owner $resolved_repo"
                    return 0
                fi
            done
        done
    fi

    # Fallback if owner and repo are already known from environment
    if [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        if [ -z "$resolved_provider" ]; then
            if [ -n "${GITHUB_OWNER:-}" ] || [ -n "${GITHUB_REPO:-}" ] || [ -n "${GITHUB_TOKEN:-}" ]; then
                resolved_provider="github"
            else
                resolved_provider="codeberg"
            fi
        fi
        echo "$resolved_provider $resolved_owner $resolved_repo"
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
        if [ "$p_prov" = "codeberg" ]; then
            echo "$info" | cut -d' ' -f2,3
            return 0
        fi
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Gosplan Rate Limiter (Max 55 requests / hour)
# -----------------------------------------------------------------------------
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

# -----------------------------------------------------------------------------
# Helper: Ensure target tickets file is present in .gitignore
# -----------------------------------------------------------------------------
ensure_tickets_gitignored() {
    target_file="$1"
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

    [ -z "$git_root" ] && return 0
    [ ! -d "$git_root/.git" ] && return 0

    gitignore_file="$git_root/.gitignore"

    # If git already ignores this path, nothing to do
    if git check-ignore -q "$target_file" 2>/dev/null; then
        return 0
    fi

    # Determine relative path from git root
    rel_path=$(echo "$target_file" | sed "s|^$git_root/||")

    case "$rel_path" in
        .antigravity/*)
            if [ ! -f "$gitignore_file" ] || ! grep -q "^\.antigravity/" "$gitignore_file" 2>/dev/null; then
                echo ".antigravity/" >> "$gitignore_file"
            fi
            ;;
        *)
            escaped=$(echo "$rel_path" | sed 's/[.[\*^$]/\\&/g')
            if [ ! -f "$gitignore_file" ] || ! grep -q "^$escaped$" "$gitignore_file" 2>/dev/null; then
                echo "$rel_path" >> "$gitignore_file"
            fi
            ;;
    esac
    return 0
}

# -----------------------------------------------------------------------------
# Synchronize tickets to markdown (GitHub / Codeberg)
# -----------------------------------------------------------------------------
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
    for tool in curl jq; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Warning: Required tool '$tool' is not installed or not in PATH. Skipping tickets sync." >&2
            return 0
        fi
    done

    # Resolve Provider, Owner and Repo
    provider="$cli_provider"
    owner="$cli_owner"
    repo="$cli_repo"

    if [ -z "$provider" ] || [ -z "$owner" ] || [ -z "$repo" ]; then
        info=$(resolve_tickets_info || true)
        if [ -n "$info" ]; then
            [ -z "$provider" ] && provider=$(echo "$info" | cut -d' ' -f1)
            [ -z "$owner" ] && owner=$(echo "$info" | cut -d' ' -f2)
            [ -z "$repo" ] && repo=$(echo "$info" | cut -d' ' -f3)
        fi
    fi

    if [ -z "$owner" ] || [ -z "$repo" ]; then
        echo "Warning: Unable to determine repository and owner from git remote or environment." >&2
        return 0
    fi

    provider="${provider:-github}"

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

    if [ "$provider" = "github" ]; then
        api_url="https://api.github.com/repos/${owner}/${repo}/issues?state=open&per_page=30"
        echo "Fetching open issues from GitHub (${owner}/${repo})..."
        token="${cli_token:-${GITHUB_TOKEN:-}}"
        if [ -n "$token" ]; then
            response=$(curl -s -f -H "User-Agent: cccp" -H "Authorization: Bearer ${token}" "$api_url" 2>/dev/null || true)
        else
            response=$(curl -s -f -H "User-Agent: cccp" "$api_url" 2>/dev/null || true)
        fi
        api_name="GitHub API"
    else
        api_url="https://codeberg.org/api/v1/repos/${owner}/${repo}/issues?state=open&type=issues&limit=30"
        echo "Fetching open issues from Codeberg (${owner}/${repo})..."
        token="${cli_token:-${CODEBERG_TOKEN:-}}"
        if [ -n "$token" ]; then
            response=$(curl -s -f -H "Authorization: token ${token}" "$api_url" 2>/dev/null || true)
        else
            response=$(curl -s -f "$api_url" 2>/dev/null || true)
        fi
        api_name="Codeberg API"
    fi

    if [ -z "$response" ] || ! printf '%s\n' "$response" | jq -e . >/dev/null 2>&1; then
        echo "Warning: Failed to retrieve or parse issues from ${api_name} (${owner}/${repo})." >&2
        return 0
    fi

    if ! printf '%s\n' "$response" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1; then
        echo "Warning: Unexpected response format from ${api_name}." >&2
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

    printf '%s\n' "$response" | jq -c '.[] | select(.pull_request == null)' > "$issues_tmp"

    jq_ticket_filter='
        "## [#" + (.number|tostring) + "] " + .title + "\n" +
        "- **Status**: `" + .state + "`\n" +
        "- **Labels**: " + (if (.labels | length) > 0 then ([.labels[].name] | join(", ")) else "none" end) + "\n" +
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
