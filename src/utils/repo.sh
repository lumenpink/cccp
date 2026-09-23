#!/bin/sh

# Enable error handling
set -eu

# =============================================================================
# CCCP Repository & Remote Forge Resolver
# Unified pure-POSIX utilities for inspecting Git remotes, extracting
# host/owner/repo metadata, and identifying remote forge providers.
# Zero external runtime dependencies; strict zero-awk compliance.
# =============================================================================

# -----------------------------------------------------------------------------
# Helper: Extract Git host, owner and repository name from any Git remote URL
# Supports HTTPS, SSH (scp-like), git://, and custom ports.
# Returns: "<host> <owner> <repo>"
# -----------------------------------------------------------------------------
parse_remote_url() {
    _raw_url="$1"
    [ -z "$_raw_url" ] && return 1

    _clean=$(printf '%s\n' "$_raw_url" | sed -e 's/\.git$//' -e 's|/*$||')

    _host=""
    _path=""
    case "$_clean" in
        *://*)
            _no_proto=$(printf '%s\n' "$_clean" | sed -e 's|^[a-zA-Z][a-zA-Z0-9+-.]*://||')
            _host_part=$(printf '%s\n' "$_no_proto" | cut -d'/' -f1)
            _host=$(printf '%s\n' "$_host_part" | sed -e 's|^.*@||' -e 's|:[0-9]*$||')
            _path=$(printf '%s\n' "$_no_proto" | sed -e 's|^[^/]*/||' -e 's|^/*||')
            ;;
        *:*)
            _host_part=$(printf '%s\n' "$_clean" | sed -e 's|:.*$||')
            _host=$(printf '%s\n' "$_host_part" | sed -e 's|^.*@||')
            _path=$(printf '%s\n' "$_clean" | sed -e 's|^[^:]*:||' -e 's|^/*||')
            ;;
        *)
            return 1
            ;;
    esac

    [ -z "$_path" ] && return 1

    _owner=$(printf '%s\n' "$_path" | sed -e 's|/[^/]*$||')
    _repo=$(printf '%s\n' "$_path" | sed -e 's|^.*/||')

    [ -n "$_host" ] && [ -n "$_owner" ] && [ -n "$_repo" ] || return 1
    printf '%s %s %s\n' "$_host" "$_owner" "$_repo"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Detect Forge Provider from hostname
# Returns normalized provider identifier:
#   github | codeberg | forgejo | tildegit | gitlab | sourcehut | unknown
# -----------------------------------------------------------------------------
detect_forge_provider() {
    _fhost="$1"
    [ -z "$_fhost" ] && return 1

    # 1. Check local config override if set
    _cfg_provider="${TICKETS_PROVIDER:-}"
    if [ -z "$_cfg_provider" ] && command -v read_file_key >/dev/null 2>&1; then
        _git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
        if [ -n "$_git_root" ] && [ -f "$_git_root/.cccprc" ]; then
            _cfg_provider="$(read_file_key "$_git_root/.cccprc" "tickets_provider" 2>/dev/null || true)"
        fi
    fi
    if [ -n "$_cfg_provider" ]; then
        printf '%s\n' "$_cfg_provider"
        return 0
    fi

    # 2. Fast-path: Deterministic domain lookup
    case "$_fhost" in
        *github.com*)
            printf 'github\n'
            return 0
            ;;
        *codeberg.org*|*gitea.com*)
            printf 'forgejo\n'
            return 0
            ;;
        *tildegit.org*)
            printf 'tildegit\n'
            return 0
            ;;
        *gitlab.com*|*framagit.org*)
            printf 'gitlab\n'
            return 0
            ;;
        *.sr.ht*|git.sr.ht|todo.sr.ht)
            printf 'sourcehut\n'
            return 0
            ;;
    esac

    # 3. Cache lookup (in .git/cccp_provider_cache)
    _git_dir="$(git rev-parse --git-dir 2>/dev/null || true)"
    _cache_file=""
    if [ -n "$_git_dir" ] && [ -d "$_git_dir" ]; then
        _cache_file="$_git_dir/cccp_provider_cache"
        if [ -f "$_cache_file" ]; then
            _cached=$(grep -E "^${_fhost}:" "$_cache_file" 2>/dev/null | head -n 1 | cut -d':' -f2 || true)
            if [ -n "$_cached" ]; then
                printf '%s\n' "$_cached"
                return 0
            fi
        fi
    fi

    # 4. Active Probing (Strict detection without false assumptions)
    if command -v curl >/dev/null 2>&1; then
        _probe_gitea=$(curl -s -f -m 3 "https://${_fhost}/api/v1/version" 2>/dev/null || true)
        if [ -n "$_probe_gitea" ]; then
            if command -v jq >/dev/null 2>&1; then
                if printf '%s\n' "$_probe_gitea" | jq -e '.version' >/dev/null 2>&1; then
                    [ -n "$_cache_file" ] && printf '%s:forgejo\n' "$_fhost" >> "$_cache_file"
                    printf 'forgejo\n'
                    return 0
                fi
            else
                case "$_probe_gitea" in
                    *\"version\"*)
                        [ -n "$_cache_file" ] && printf '%s:forgejo\n' "$_fhost" >> "$_cache_file"
                        printf 'forgejo\n'
                        return 0
                        ;;
                esac
            fi
        fi

        _probe_gl=$(curl -s -f -I -m 3 "https://${_fhost}/api/v4/version" 2>/dev/null || curl -s -f -I -m 3 "https://${_fhost}/" 2>/dev/null || true)
        case "$_probe_gl" in
            *gitlab*|*GitLab*)
                [ -n "$_cache_file" ] && printf '%s:gitlab\n' "$_fhost" >> "$_cache_file"
                printf 'gitlab\n'
                return 0
                ;;
        esac
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Specific Forge URL Parsers (Backward Compatibility)
# -----------------------------------------------------------------------------
parse_codeberg_url() { _parse_forge_remote "$1" "codeberg"; }
parse_github_url()   { _parse_forge_remote "$1" "github"; }
parse_gitlab_url()   { _parse_forge_remote "$1" "gitlab"; }

_parse_forge_remote() {
    _raw="$1"
    _target_type="$2"
    if _parsed=$(parse_remote_url "$_raw" 2>/dev/null); then
        _host=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
        _owner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
        _repo=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)
        _match=0
        case "$_target_type" in
            codeberg)
                case "$_host" in *codeberg.org*) _match=1 ;; esac ;;
            github)
                case "$_host" in *github.com*) _match=1 ;; esac ;;
            gitlab)
                case "$_host" in *gitlab.com*|*framagit.org*) _match=1 ;; esac ;;
        esac
        if [ "$_match" -eq 1 ]; then
            printf '%s %s\n' "$_owner" "$_repo"
            return 0
        fi
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Core: Detect Repository Information for a local Git repository
# Usage:
#   detect_repo_info [repo_path]
# Returns:
#   "<forge> <owner> <repo> <host>"
# -----------------------------------------------------------------------------
detect_repo_info() {
    _target_path="${1:-.}"

    if [ ! -d "$_target_path" ]; then
        return 1
    fi

    # Check origin push URL first, then fetch URL
    _origin_url=$(git -C "$_target_path" config remote.origin.pushurl 2>/dev/null \
        || git -C "$_target_path" config remote.origin.url 2>/dev/null \
        || git -C "$_target_path" remote get-url origin 2>/dev/null || true)

    if [ -z "$_origin_url" ]; then
        # Try any configured remote
        _first_remote=$(git -C "$_target_path" remote 2>/dev/null | head -n 1 || true)
        if [ -n "$_first_remote" ]; then
            _origin_url=$(git -C "$_target_path" config "remote.${_first_remote}.url" 2>/dev/null \
                || git -C "$_target_path" remote get-url "$_first_remote" 2>/dev/null || true)
        fi
    fi

    [ -z "$_origin_url" ] && return 1

    if ! _parsed=$(parse_remote_url "$_origin_url" 2>/dev/null); then
        return 1
    fi

    _rhost=$(printf '%s\n' "$_parsed" | cut -d' ' -f1)
    _rowner=$(printf '%s\n' "$_parsed" | cut -d' ' -f2)
    _rname=$(printf '%s\n' "$_parsed" | cut -d' ' -f3)

    if ! _rforge=$(detect_forge_provider "$_rhost" 2>/dev/null); then
        return 1
    fi
    [ "$_rforge" = "unknown" ] && return 1

    printf '%s %s %s %s\n' "$_rforge" "$_rowner" "$_rname" "$_rhost"
    return 0
}

# -----------------------------------------------------------------------------
# Convenience Accessors
# -----------------------------------------------------------------------------
get_git_root() {
    git rev-parse --show-toplevel 2>/dev/null || true
}

get_repo_slug() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    _o=$(printf '%s\n' "$_info" | cut -d' ' -f2)
    _r=$(printf '%s\n' "$_info" | cut -d' ' -f3)
    printf '%s/%s\n' "$_o" "$_r"
    return 0
}

get_repo_owner() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f2
    return 0
}

get_repo_name() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f3
    return 0
}

get_repo_forge() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f1
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Ensure target file is present in .gitignore
# -----------------------------------------------------------------------------
ensure_gitignored() {
    _target_file="$1"
    _git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

    [ -z "$_git_root" ] && return 0
    [ ! -d "$_git_root/.git" ] && return 0

    _gitignore_file="$_git_root/.gitignore"

    # If git already ignores this path, nothing to do
    if git check-ignore -q "$_target_file" 2>/dev/null; then
        return 0
    fi

    # Determine relative path from git root
    _rel_path=$(printf '%s\n' "$_target_file" | sed "s|^$_git_root/||")

    case "$_rel_path" in
        .antigravity/*)
            if [ ! -f "$_gitignore_file" ] || ! grep -q "^\.antigravity/" "$_gitignore_file" 2>/dev/null; then
                printf '.antigravity/\n' >> "$_gitignore_file"
            fi
            ;;
        *)
            _escaped=$(printf '%s\n' "$_rel_path" | sed 's/[.[\*^$]/\\&/g')
            if [ ! -f "$_gitignore_file" ] || ! grep -q "^$_escaped$" "$_gitignore_file" 2>/dev/null; then
                printf '%s\n' "$_rel_path" >> "$_gitignore_file"
            fi
            ;;
    esac
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Extract Tea CLI token for Gitea/Forgejo/Codeberg
# -----------------------------------------------------------------------------
get_tea_token() {
    _target="$1"
    _cfg="${HOME}/.config/tea/config.yml"
    [ -f "$_cfg" ] || return 1

    _found_target=0
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            *"- name:"*"$_target"*|*"name:"*"$_target"*)
                _found_target=1
                ;;
            *"- name:"*)
                _found_target=0
                ;;
            *"token:"*)
                if [ "$_found_target" -eq 1 ]; then
                    _token=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*token:[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                    printf '%s\n' "$_token"
                    return 0
                fi
                ;;
        esac
    done < "$_cfg"
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Unified Forge API Token Resolution (Memory -> .env -> .cccprc -> Global -> Tea)
# -----------------------------------------------------------------------------
resolve_forge_token() {
    _forge="$1"
    _env_var=""
    _cfg_k=""

    case "$_forge" in
        codeberg|forgejo)
            [ -n "${CODEBERG_TOKEN:-}" ] && { printf '%s\n' "$CODEBERG_TOKEN"; return 0; }
            _env_var="CODEBERG_TOKEN"
            _cfg_k="codeberg_token"
            ;;
        tildegit)
            [ -n "${TILDEGIT_TOKEN:-}" ] && { printf '%s\n' "$TILDEGIT_TOKEN"; return 0; }
            _env_var="TILDEGIT_TOKEN"
            _cfg_k="tildegit_token"
            ;;
        github)
            [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
            [ -n "${GH_TOKEN:-}" ] && { printf '%s\n' "$GH_TOKEN"; return 0; }
            _env_var="GITHUB_TOKEN"
            _cfg_k="github_token"
            ;;
        gitlab)
            [ -n "${GITLAB_TOKEN:-}" ] && { printf '%s\n' "$GITLAB_TOKEN"; return 0; }
            _env_var="GITLAB_TOKEN"
            _cfg_k="gitlab_token"
            ;;
        sourcehut)
            [ -n "${SOURCEHUT_TOKEN:-}" ] && { printf '%s\n' "$SOURCEHUT_TOKEN"; return 0; }
            _env_var="SOURCEHUT_TOKEN"
            _cfg_k="sourcehut_token"
            ;;
        *)
            return 1
            ;;
    esac

    # 1. Check local and home .env files
    for _env_f in ".env" "${HOME}/.env"; do
        if [ -f "$_env_f" ]; then
            if command -v read_file_key >/dev/null 2>&1; then
                if _tok=$(read_file_key "$_env_f" "$_cfg_k" 2>/dev/null) && [ -n "$_tok" ]; then
                    printf '%s\n' "$_tok"
                    return 0
                fi
                if [ -n "$_env_var" ]; then
                    if _tok=$(read_file_key "$_env_f" "$_env_var" 2>/dev/null) && [ -n "$_tok" ]; then
                        printf '%s\n' "$_tok"
                        return 0
                    fi
                fi
            fi
        fi
    done

    # 2. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        if _tok=$(read_file_key "$_local_cfg" "$_cfg_k" 2>/dev/null) && [ -n "$_tok" ]; then
            printf '%s\n' "$_tok"
            return 0
        fi
    fi

    # 3. Check global config
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    fi
    [ -z "$_global_cfg" ] && [ -f "${HOME}/.cccprc" ] && _global_cfg="${HOME}/.cccprc"

    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        if _tok=$(read_file_key "$_global_cfg" "$_cfg_k" 2>/dev/null) && [ -n "$_tok" ]; then
            printf '%s\n' "$_tok"
            return 0
        fi
    fi

    # 4. Check tea config
    case "$_forge" in
        codeberg|forgejo|tildegit)
            get_tea_token "$_forge" && return 0
            ;;
    esac

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Escape JSON String (Pure POSIX)
# -----------------------------------------------------------------------------
escape_json_string() {
    _raw="$1"
    printf '%s' "$_raw" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' -e 's/\r//g'
}
