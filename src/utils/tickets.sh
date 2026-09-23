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
    if [ -f "$GIT_ROOT/src/utils/repo.sh" ]; then
        . "$GIT_ROOT/src/utils/repo.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Remote URL & Forge resolution delegated to src/utils/repo.sh
# -----------------------------------------------------------------------------
if ! command -v parse_remote_url >/dev/null 2>&1; then
    _script_dir="$(cd "$(dirname "$0")" 2>/dev/null && pwd || true)"
    if [ -n "$_script_dir" ] && [ -f "$_script_dir/repo.sh" ]; then
        . "$_script_dir/repo.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Helper: Resolve Git provider, owner, repo, and host dynamically
# -----------------------------------------------------------------------------
resolve_tickets_info() {
    resolved_provider="${TICKETS_PROVIDER:-}"
    resolved_owner="${TICKETS_OWNER:-${CODEBERG_OWNER:-${CODEBERG_USER:-${GITHUB_OWNER:-${GITHUB_USER:-}}}}}"
    resolved_repo="${TICKETS_REPO:-${CODEBERG_REPO:-${GITHUB_REPO:-}}}"
    resolved_host=""

    # If provider, owner and repo are already known from environment/flags
    if [ -n "$resolved_provider" ] && [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        case "$resolved_provider" in
            github) resolved_host="github.com" ;;
            codeberg|forgejo) resolved_host="codeberg.org" ;;
            gitlab) resolved_host="framagit.org" ;;
            sourcehut) resolved_host="git.sr.ht" ;;
            *) resolved_host="codeberg.org" ;;
        esac
        echo "$resolved_provider $resolved_owner $resolved_repo $resolved_host"
        return 0
    fi

    # Inspect git remotes if inside a git repository
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        if _dinfo=$(detect_repo_info 2>/dev/null); then
            p_prov=$(printf '%s' "$_dinfo" | cut -d' ' -f1)
            p_owner=$(printf '%s' "$_dinfo" | cut -d' ' -f2)
            p_repo=$(printf '%s' "$_dinfo" | cut -d' ' -f3)
            p_host=$(printf '%s' "$_dinfo" | cut -d' ' -f4)
            resolved_provider="${resolved_provider:-$p_prov}"
            resolved_owner="${resolved_owner:-$p_owner}"
            resolved_repo="${resolved_repo:-$p_repo}"
            resolved_host="$p_host"
            printf '%s %s %s %s\n' "$resolved_provider" "$resolved_owner" "$resolved_repo" "$resolved_host"
            return 0
        fi
    fi

    # Fallback if owner and repo are already known from environment
    if [ -n "$resolved_owner" ] && [ -n "$resolved_repo" ]; then
        if [ -z "$resolved_provider" ]; then
            if [ -n "${GITHUB_OWNER:-}" ] || [ -n "${GITHUB_REPO:-}" ] || [ -n "${GITHUB_TOKEN:-}" ]; then
                resolved_provider="github"
                resolved_host="github.com"
            else
                resolved_provider="forgejo"
                resolved_host="codeberg.org"
            fi
        else
            case "$resolved_provider" in
                github) resolved_host="${resolved_host:-github.com}" ;;
                codeberg|forgejo) resolved_host="${resolved_host:-codeberg.org}" ;;
                gitlab) resolved_host="${resolved_host:-framagit.org}" ;;
                sourcehut) resolved_host="${resolved_host:-git.sr.ht}" ;;
            esac
        fi
        echo "$resolved_provider $resolved_owner $resolved_repo $resolved_host"
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
        if [ "$p_prov" = "forgejo" ] || [ "$p_prov" = "codeberg" ]; then
            echo "$info" | cut -d' ' -f2,3
            return 0
        fi
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Drivers: Isolated Fetch and Normalization Functions
# -----------------------------------------------------------------------------

_tickets_fetch_github() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    if [ "$host" = "github.com" ]; then
        api_url="https://api.github.com/repos/${owner}/${repo}/issues?state=open&per_page=30"
    else
        api_url="https://${host}/api/v3/repos/${owner}/${repo}/issues?state=open&per_page=30"
    fi

    auth_header=""
    [ -n "$token" ] && auth_header="Authorization: Bearer ${token}"

    if [ -n "$auth_header" ]; then
        resp=$(curl -s -f -H "User-Agent: cccp" -H "$auth_header" "$api_url" 2>/dev/null || true)
    else
        resp=$(curl -s -f -H "User-Agent: cccp" "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | select(.pull_request == null) | {
        number: .number,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then [.labels[].name] else [] end),
        html_url: .html_url,
        body: .body
    }'
}

_tickets_fetch_forgejo() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    api_url="https://${host}/api/v1/repos/${owner}/${repo}/issues?state=open&type=issues&limit=30"

    if [ -n "$token" ]; then
        resp=$(curl -s -f -H "Authorization: token ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(curl -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | {
        number: .number,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then [.labels[].name] else [] end),
        html_url: .html_url,
        body: .body
    }'
}

_tickets_fetch_gitlab() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    url_encoded=$(echo "${owner}/${repo}" | sed -e 's|/|%2F|g')
    api_url="https://${host}/api/v4/projects/${url_encoded}/issues?state=opened&per_page=30"

    if [ -n "$token" ]; then
        resp=$(curl -s -f -H "PRIVATE-TOKEN: ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(curl -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1
    printf '%s\n' "$resp" | jq -e 'if type == "array" then true else false end' >/dev/null 2>&1 || return 1

    printf '%s\n' "$resp" | jq -c '.[] | {
        number: .iid,
        title: .title,
        state: .state,
        labels: (if (.labels | length) > 0 then (.labels | map(if type == "string" then . else .name end)) else [] end),
        html_url: .web_url,
        body: .description
    }'
}

_tickets_fetch_sourcehut() {
    host="$1"
    owner="$2"
    repo="$3"
    token="$4"

    owner_clean=$(echo "$owner" | sed 's/^~//')
    api_url="https://todo.sr.ht/api/user/~${owner_clean}/trackers/${repo}/tickets"

    if [ -n "$token" ]; then
        resp=$(curl -s -f -H "Authorization: Bearer ${token}" "$api_url" 2>/dev/null || true)
    else
        resp=$(curl -s -f "$api_url" 2>/dev/null || true)
    fi

    [ -z "$resp" ] && return 1

    printf '%s\n' "$resp" | jq -c '.results[]? // .[]? | {
        number: .id,
        title: .title,
        state: .status,
        labels: [],
        html_url: ("https://todo.sr.ht/~" + "'"$owner_clean"'" + "/" + "'"$repo"'" + "/" + (.id|tostring)),
        body: .description
    }'
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
# Synchronize tickets to markdown
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

    # Resolve Provider, Owner, Repo, and Host
    provider="$cli_provider"
    owner="$cli_owner"
    repo="$cli_repo"
    host=""

    if [ -z "$provider" ] || [ -z "$owner" ] || [ -z "$repo" ]; then
        info=$(resolve_tickets_info || true)
        if [ -n "$info" ]; then
            [ -z "$provider" ] && provider=$(echo "$info" | cut -d' ' -f1)
            [ -z "$owner" ] && owner=$(echo "$info" | cut -d' ' -f2)
            [ -z "$repo" ] && repo=$(echo "$info" | cut -d' ' -f3)
            host=$(echo "$info" | cut -d' ' -f4)
        fi
    fi

    if [ -z "$owner" ] || [ -z "$repo" ]; then
        echo "Warning: Unable to determine repository and owner from git remote or environment." >&2
        return 0
    fi

    provider="${provider:-forgejo}"
    [ "$provider" = "codeberg" ] && provider="forgejo"

    if [ -z "$host" ]; then
        case "$provider" in
            github) host="github.com" ;;
            forgejo) host="codeberg.org" ;;
            gitlab) host="framagit.org" ;;
            sourcehut) host="git.sr.ht" ;;
            *) host="codeberg.org" ;;
        esac
    fi

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

    fetch_status=0
    case "$provider" in
        github)
            echo "Fetching open issues from GitHub (${owner}/${repo})..."
            issues_json=$(_tickets_fetch_github "$host" "$owner" "$repo" "${cli_token:-${GITHUB_TOKEN:-}}") || fetch_status=$?
            ;;
        forgejo)
            if [ "$host" = "codeberg.org" ]; then
                echo "Fetching open issues from Codeberg (${owner}/${repo})..."
            else
                echo "Fetching open issues from Forgejo/Gitea (${owner}/${repo} on ${host})..."
            fi
            issues_json=$(_tickets_fetch_forgejo "$host" "$owner" "$repo" "${cli_token:-${CODEBERG_TOKEN:-}}") || fetch_status=$?
            ;;
        gitlab)
            if [ "$host" = "framagit.org" ]; then
                echo "Fetching open issues from Framagit (${owner}/${repo})..."
            else
                echo "Fetching open issues from GitLab (${owner}/${repo} on ${host})..."
            fi
            issues_json=$(_tickets_fetch_gitlab "$host" "$owner" "$repo" "${cli_token:-${GITLAB_TOKEN:-}}") || fetch_status=$?
            ;;
        sourcehut)
            owner_clean=$(echo "$owner" | sed 's/^~//')
            echo "Fetching open issues from SourceHut (~${owner_clean}/${repo})..."
            issues_json=$(_tickets_fetch_sourcehut "$host" "$owner" "$repo" "${cli_token:-${SOURCEHUT_TOKEN:-}}") || fetch_status=$?
            ;;
        *)
            echo "Warning: Unsupported provider '${provider}'." >&2
            return 0
            ;;
    esac

    if [ "$fetch_status" -ne 0 ]; then
        case "$provider" in
            forgejo)
                api_name="Codeberg API"
                [ "$host" != "codeberg.org" ] && api_name="Forgejo API"
                ;;
            github) api_name="GitHub API" ;;
            gitlab) api_name="GitLab API" ;;
            sourcehut) api_name="SourceHut API" ;;
            *) api_name="${provider} API" ;;
        esac
        echo "Warning: Failed to retrieve or parse issues from ${api_name} (${owner}/${repo})." >&2
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

    printf '%s\n' "$issues_json" > "$issues_tmp"

    jq_ticket_filter='
        "## [#" + (.number|tostring) + "] " + .title + "\n" +
        "- **Status**: `" + .state + "`\n" +
        "- **Labels**: " + (if (.labels | length) > 0 then (.labels | join(", ")) else "none" end) + "\n" +
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
