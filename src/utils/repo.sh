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
# Helper: Dispatch HTTP probe via sv / svyaz / curl
# -----------------------------------------------------------------------------
_repo_http() {
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
# Helper: Detect ForgeApp (Software Engine / Protocol Driver) from domain or provider
# Returns normalized engine identifier:
#   forgeapp_forgejo | forgeapp_github | forgeapp_gitlab | forgeapp_sourcehut | unknown
# -----------------------------------------------------------------------------
detect_forgeapp() {
    _fhost="$1"
    [ -z "$_fhost" ] && return 1

    # 1. Check local config override if set
    _cfg_provider="${FORGEAPP:-${TICKETS_PROVIDER:-}}"
    if [ -z "$_cfg_provider" ] && command -v read_file_key >/dev/null 2>&1; then
        _git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
        if [ -n "$_git_root" ] && [ -f "$_git_root/.cccprc" ]; then
            _cfg_provider="$(read_file_key "$_git_root/.cccprc" "forgeapp" 2>/dev/null || true)"
            [ -z "$_cfg_provider" ] && _cfg_provider="$(read_file_key "$_git_root/.cccprc" "tickets_provider" 2>/dev/null || true)"
        fi
    fi
    if [ -n "$_cfg_provider" ]; then
        case "$_cfg_provider" in
            forgeapp_forgejo|forgejo|codeberg|tildegit|gitea) printf 'forgeapp_forgejo\n'; return 0 ;;
            forgeapp_github|github) printf 'forgeapp_github\n'; return 0 ;;
            forgeapp_gitlab|gitlab) printf 'forgeapp_gitlab\n'; return 0 ;;
            forgeapp_sourcehut|sourcehut) printf 'forgeapp_sourcehut\n'; return 0 ;;
            forgeapp_*) printf '%s\n' "$_cfg_provider"; return 0 ;;
        esac
    fi

    # 2. Fast-path: Deterministic domain lookup
    case "$_fhost" in
        *github.com*|forgeapp_github|github)
            printf 'forgeapp_github\n'
            return 0
            ;;
        *codeberg.org*|*gitea.com*|*tildegit.org*|*disroot.org*|forgeapp_forgejo|forgejo|codeberg|tildegit)
            printf 'forgeapp_forgejo\n'
            return 0
            ;;
        *gitlab.com*|*framagit.org*|forgeapp_gitlab|gitlab)
            printf 'forgeapp_gitlab\n'
            return 0
            ;;
        *.sr.ht*|git.sr.ht|todo.sr.ht|forgeapp_sourcehut|sourcehut)
            printf 'forgeapp_sourcehut\n'
            return 0
            ;;
    esac

    # 3. Cache lookup (in .git/cccp_forgeapp_cache)
    _git_dir="$(git rev-parse --git-dir 2>/dev/null || true)"
    _cache_file=""
    if [ -n "$_git_dir" ] && [ -d "$_git_dir" ]; then
        _cache_file="$_git_dir/cccp_forgeapp_cache"
        if [ -f "$_cache_file" ]; then
            _cached=$(grep -E "^${_fhost}:" "$_cache_file" 2>/dev/null | head -n 1 | cut -d':' -f2 || true)
            if [ -n "$_cached" ]; then
                printf '%s\n' "$_cached"
                return 0
            fi
        fi
    fi

    # 4. Active Probing (Strict detection without false assumptions)
    if command -v _cccp_http >/dev/null 2>&1 || command -v sv >/dev/null 2>&1 || command -v svyaz >/dev/null 2>&1 || command -v curl >/dev/null 2>&1; then
        _probe_gitea=$(_repo_http -s -f -m 3 "https://${_fhost}/api/v1/version" 2>/dev/null || true)
        if [ -n "$_probe_gitea" ]; then
            if command -v jq >/dev/null 2>&1; then
                if printf '%s\n' "$_probe_gitea" | jq -e '.version' >/dev/null 2>&1; then
                    [ -n "$_cache_file" ] && printf '%s:forgeapp_forgejo\n' "$_fhost" >> "$_cache_file"
                    printf 'forgeapp_forgejo\n'
                    return 0
                fi
            else
                case "$_probe_gitea" in
                    *\"version\"*)
                        [ -n "$_cache_file" ] && printf '%s:forgeapp_forgejo\n' "$_fhost" >> "$_cache_file"
                        printf 'forgeapp_forgejo\n'
                        return 0
                        ;;
                esac
            fi
        fi

        _probe_gl=$(_repo_http -s -f -I -m 3 "https://${_fhost}/api/v4/version" 2>/dev/null || _repo_http -s -f -I -m 3 "https://${_fhost}/" 2>/dev/null || true)
        case "$_probe_gl" in
            *gitlab*|*GitLab*)
                [ -n "$_cache_file" ] && printf '%s:forgeapp_gitlab\n' "$_fhost" >> "$_cache_file"
                printf 'forgeapp_gitlab\n'
                return 0
                ;;
        esac
    fi

    printf 'unknown\n'
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Detect Forge Provider from hostname (Backward Compatibility)
# Returns normalized provider identifier:
#   github | codeberg | forgejo | tildegit | gitlab | sourcehut | unknown
# -----------------------------------------------------------------------------
detect_forge_provider() {
    _fhost="$1"
    [ -z "$_fhost" ] && return 1

    case "$_fhost" in
        *codeberg.org*|codeberg) printf 'forgejo\n'; return 0 ;;
        *tildegit.org*|tildegit) printf 'tildegit\n'; return 0 ;;
    esac

    _app=$(detect_forgeapp "$_fhost" 2>/dev/null || true)
    case "$_app" in
        forgeapp_forgejo) printf 'forgejo\n'; return 0 ;;
        forgeapp_github) printf 'github\n'; return 0 ;;
        forgeapp_gitlab) printf 'gitlab\n'; return 0 ;;
        forgeapp_sourcehut) printf 'sourcehut\n'; return 0 ;;
        *) return 1 ;;
    esac
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

    if ! _rforgeapp=$(detect_forgeapp "$_rhost" 2>/dev/null); then
        return 1
    fi
    [ "$_rforgeapp" = "unknown" ] && return 1

    printf '%s %s %s %s\n' "$_rhost" "$_rowner" "$_rname" "$_rforgeapp"
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

get_repo_domain() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f1
    return 0
}

get_repo_forge() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f1
    return 0
}

get_repo_forgeapp() {
    _info=$(detect_repo_info "${1:-.}") || return 1
    printf '%s\n' "$_info" | cut -d' ' -f4
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
# Helper: Extract GitHub CLI token from config file or gh CLI
# -----------------------------------------------------------------------------
get_gh_token() {
    _target_host="${1:-github.com}"
    _gh_cfg_dir="${GH_CONFIG_DIR:-${XDG_CONFIG_HOME:-${HOME}/.config}/gh}"
    _cfg="${_gh_cfg_dir}/hosts.yml"

    if [ -f "$_cfg" ]; then
        _found_host=0
        while IFS= read -r _line || [ -n "$_line" ]; do
            case "$_line" in
                *"$_target_host:"*)
                    _found_host=1
                    ;;
                *":"*)
                    case "$_line" in
                        " "*|"\t"*)
                            ;;
                        *)
                            _found_host=0
                            ;;
                    esac
                    ;;
            esac
            if [ "$_found_host" -eq 1 ]; then
                case "$_line" in
                    *"oauth_token:"*)
                        _token=$(printf '%s' "$_line" | sed -e 's/^[[:space:]]*oauth_token:[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                        if [ -n "$_token" ]; then
                            printf '%s\n' "$_token"
                            return 0
                        fi
                        ;;
                esac
            fi
        done < "$_cfg"
    fi

    # Fallback to gh CLI auth token if gh is available
    if command -v gh >/dev/null 2>&1; then
        _tok=$(gh auth token -h "$_target_host" 2>/dev/null || gh auth token 2>/dev/null || true)
        if [ -n "$_tok" ]; then
            printf '%s\n' "$_tok"
            return 0
        fi
    fi

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Unified Forge API Token Resolution (Memory -> .env -> .cccprc -> Global -> Tea -> GH)
# -----------------------------------------------------------------------------
resolve_forge_token() {
    _forge="$1"
    _keys=""

    case "$_forge" in
        *codeberg.org*|codeberg|forgejo|forgeapp_forgejo)
            [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
            [ -n "${CODEBERG_TOKEN:-}" ] && { printf '%s\n' "$CODEBERG_TOKEN"; return 0; }
            _keys="forgejo_token FORGEJO_TOKEN codeberg_token CODEBERG_TOKEN"
            ;;
        *tildegit.org*|tildegit)
            [ -n "${TILDEGIT_TOKEN:-}" ] && { printf '%s\n' "$TILDEGIT_TOKEN"; return 0; }
            [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
            _keys="tildegit_token TILDEGIT_TOKEN forgejo_token FORGEJO_TOKEN"
            ;;
        *github.com*|github|forgeapp_github)
            [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
            [ -n "${GH_TOKEN:-}" ] && { printf '%s\n' "$GH_TOKEN"; return 0; }
            _keys="github_token GITHUB_TOKEN GH_TOKEN"
            ;;
        *gitlab.com*|*framagit.org*|gitlab|forgeapp_gitlab)
            [ -n "${GITLAB_TOKEN:-}" ] && { printf '%s\n' "$GITLAB_TOKEN"; return 0; }
            [ -n "${FRAMAGIT_TOKEN:-}" ] && { printf '%s\n' "$FRAMAGIT_TOKEN"; return 0; }
            _keys="gitlab_token GITLAB_TOKEN framagit_token FRAMAGIT_TOKEN"
            ;;
        *.sr.ht*|git.sr.ht|todo.sr.ht|sourcehut|forgeapp_sourcehut)
            [ -n "${SOURCEHUT_TOKEN:-}" ] && { printf '%s\n' "$SOURCEHUT_TOKEN"; return 0; }
            _keys="sourcehut_token SOURCEHUT_TOKEN"
            ;;
        *)
            _inferred_app=$(detect_forgeapp "$_forge" 2>/dev/null || true)
            case "$_inferred_app" in
                forgeapp_forgejo)
                    [ -n "${FORGEJO_TOKEN:-}" ] && { printf '%s\n' "$FORGEJO_TOKEN"; return 0; }
                    _keys="forgejo_token FORGEJO_TOKEN"
                    ;;
                forgeapp_github)
                    [ -n "${GITHUB_TOKEN:-}" ] && { printf '%s\n' "$GITHUB_TOKEN"; return 0; }
                    _keys="github_token GITHUB_TOKEN"
                    ;;
                forgeapp_gitlab)
                    [ -n "${GITLAB_TOKEN:-}" ] && { printf '%s\n' "$GITLAB_TOKEN"; return 0; }
                    _keys="gitlab_token GITLAB_TOKEN"
                    ;;
                forgeapp_sourcehut)
                    [ -n "${SOURCEHUT_TOKEN:-}" ] && { printf '%s\n' "$SOURCEHUT_TOKEN"; return 0; }
                    _keys="sourcehut_token SOURCEHUT_TOKEN"
                    ;;
                *)
                    return 1
                    ;;
            esac
            ;;
    esac

    # 1. Check local and home .env files
    for _env_f in ".env" "${HOME}/.env"; do
        if [ -f "$_env_f" ] && command -v read_file_key >/dev/null 2>&1; then
            for _k in $_keys; do
                if _tok=$(read_file_key "$_env_f" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                    printf '%s\n' "$_tok"
                    return 0
                fi
            done
        fi
    done

    # 2. Check local config (.cccprc)
    _local_cfg=""
    if command -v get_local_config_file >/dev/null 2>&1; then
        _local_cfg=$(get_local_config_file 2>/dev/null || true)
    fi
    [ -z "$_local_cfg" ] && [ -f ".cccprc" ] && _local_cfg=".cccprc"

    if [ -n "$_local_cfg" ] && [ -f "$_local_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        for _k in $_keys; do
            if _tok=$(read_file_key "$_local_cfg" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                printf '%s\n' "$_tok"
                return 0
            fi
        done
    fi

    # 3. Check global config
    _global_cfg=""
    if command -v get_global_config_file >/dev/null 2>&1; then
        _global_cfg=$(get_global_config_file 2>/dev/null || true)
    else
        _global_cfg="${XDG_CONFIG_HOME:-${HOME}/.config}/cccp/config"
    fi

    if [ -n "$_global_cfg" ] && [ -f "$_global_cfg" ] && command -v read_file_key >/dev/null 2>&1; then
        for _k in $_keys; do
            if _tok=$(read_file_key "$_global_cfg" "$_k" 2>/dev/null) && [ -n "$_tok" ]; then
                printf '%s\n' "$_tok"
                return 0
            fi
        done
    fi

    # 4. Check tea/gh config
    case "$_forge" in
        *codeberg.org*|*tildegit.org*|*gitea.com*|forgejo|codeberg|tildegit|forgeapp_forgejo)
            get_tea_token "$_forge" && return 0
            get_tea_token "codeberg" && return 0
            get_tea_token "forgejo" && return 0
            ;;
        *github.com*|github|forgeapp_github)
            get_gh_token "github.com" && return 0
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

# -----------------------------------------------------------------------------
# Terminal Colors & NO_COLOR Support for Repo Management
# -----------------------------------------------------------------------------
_repo_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        REPO_C_RED='\033[0;31m'
        REPO_C_GREEN='\033[0;32m'
        REPO_C_YELLOW='\033[0;33m'
        REPO_C_CYAN='\033[0;36m'
        REPO_C_BOLD='\033[1m'
        REPO_C_RESET='\033[0m'
    else
        REPO_C_RED=''
        REPO_C_GREEN=''
        REPO_C_YELLOW=''
        REPO_C_CYAN=''
        REPO_C_BOLD=''
        REPO_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# Helper: Discover repositories to audit
# -----------------------------------------------------------------------------
_repo_discover_list() {
    _d_dir="${1:-}"
    _d_catalog="${2:-}"

    # 1. If explicit catalog provided
    if [ -n "$_d_catalog" ]; then
        if [ ! -f "$_d_catalog" ]; then
            echo "Error: Catalog file '$_d_catalog' not found" >&2
            return 1
        fi
        _cat_dir=$(cd "$(dirname "$_d_catalog")" && pwd)
        _cat_names=$(sed -n 's/^[[:space:]]*"name":[[:space:]]*"\([^"]*\)".*/\1/p' "$_d_catalog" 2>/dev/null || true)
        for _cn in $_cat_names; do
            if [ -d "${_cat_dir}/${_cn}/.git" ]; then
                printf '%s\n' "${_cat_dir}/${_cn}"
            elif [ -d "${HOME}/src/${_cn}/.git" ]; then
                printf '%s\n' "${HOME}/src/${_cn}"
            elif [ -d "./${_cn}/.git" ]; then
                printf '%s\n' "./${_cn}"
            fi
        done
        return 0
    fi

    # 2. If explicit directory provided
    if [ -n "$_d_dir" ]; then
        if [ ! -d "$_d_dir" ]; then
            echo "Error: Directory '$_d_dir' not found" >&2
            return 1
        fi
        if [ -d "$_d_dir/.git" ]; then
            printf '%s\n' "$_d_dir"
            return 0
        fi
        for _sub in "$_d_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # 3. Default discovery:
    # Check for catalog.json in ., .., or $HOME/src
    _found_cat=""
    if [ -f "./catalog.json" ]; then
        _found_cat="./catalog.json"
    elif [ -f "../catalog.json" ]; then
        _found_cat="../catalog.json"
    elif [ -f "${HOME}/src/catalog.json" ]; then
        _found_cat="${HOME}/src/catalog.json"
    fi

    if [ -n "$_found_cat" ]; then
        _cat_dir=$(cd "$(dirname "$_found_cat")" && pwd)
        _cat_names=$(sed -n 's/^[[:space:]]*"name":[[:space:]]*"\([^"]*\)".*/\1/p' "$_found_cat" 2>/dev/null || true)
        for _cn in $_cat_names; do
            if [ -d "${_cat_dir}/${_cn}/.git" ]; then
                printf '%s\n' "${_cat_dir}/${_cn}"
            elif [ -d "${HOME}/src/${_cn}/.git" ]; then
                printf '%s\n' "${HOME}/src/${_cn}"
            fi
        done
        for _sub in "$_cat_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # 4. Fallback: check sibling repos in parent dir or ~/src or current dir
    _parent_dir=$(cd .. 2>/dev/null && pwd || true)
    _has_siblings=0
    if [ -n "$_parent_dir" ] && [ "$_parent_dir" != "/" ]; then
        for _sub in "$_parent_dir"/*; do
            if [ -d "$_sub/.git" ]; then
                _has_siblings=1
                printf '%s\n' "$_sub"
            fi
        done
    fi
    [ "$_has_siblings" -eq 1 ] && return 0

    if [ -d "${HOME}/src" ]; then
        for _sub in "${HOME}/src"/*; do
            if [ -d "$_sub/.git" ]; then
                printf '%s\n' "$_sub"
            fi
        done
        return 0
    fi

    # If current directory is a git repo
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git rev-parse --show-toplevel 2>/dev/null
        return 0
    fi

    return 0
}

# -----------------------------------------------------------------------------
# Helper: Audit single git repository
# Returns tab-separated values: name \t branch \t dirty \t unpushed \t issues \t path
# -----------------------------------------------------------------------------
_repo_audit_single() {
    _rpath="$1"
    _do_fetch="${2:-0}"

    [ -d "$_rpath" ] || return 1

    _rname=$(basename "$_rpath")
    _rbranch=$(git -C "$_rpath" symbolic-ref --short HEAD 2>/dev/null || git -C "$_rpath" rev-parse --short HEAD 2>/dev/null || echo "detached")

    # Dirty status: count unstaged, staged, or untracked files
    _rdirty=$(git -C "$_rpath" status --porcelain 2>/dev/null | wc -l | tr -d ' ')

    # Unpushed commits
    _rupstream=$(git -C "$_rpath" rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)
    _runpushed=0
    if [ -n "$_rupstream" ]; then
        _runpushed=$(git -C "$_rpath" rev-list --count "${_rupstream}..HEAD" 2>/dev/null || echo "0")
    else
        _has_remotes=$(git -C "$_rpath" remote 2>/dev/null || true)
        if [ -n "$_has_remotes" ]; then
            if git -C "$_rpath" rev-parse --verify "origin/${_rbranch}" >/dev/null 2>&1; then
                _runpushed=$(git -C "$_rpath" rev-list --count "origin/${_rbranch}..HEAD" 2>/dev/null || echo "0")
            else
                _runpushed=$(git -C "$_rpath" rev-list --count HEAD 2>/dev/null || echo "0")
            fi
        fi
    fi

    # Tickets / issues count
    _rissues=0
    if [ -f "$_rpath/.antigravity/tickets.md" ]; then
        _rissues=$(grep -c '^## \[#[0-9]' "$_rpath/.antigravity/tickets.md" 2>/dev/null || true)
        _rissues=$(printf '%s' "$_rissues" | tr -cd '0-9')
        [ -z "$_rissues" ] && _rissues=0
    fi

    if [ "$_do_fetch" -eq 1 ] && [ "$_rissues" -eq 0 ]; then
        _rinfo=$(detect_repo_info "$_rpath" 2>/dev/null || true)
        if [ -n "$_rinfo" ]; then
            _rf_owner=$(printf '%s' "$_rinfo" | cut -d' ' -f2)
            _rf_repo=$(printf '%s' "$_rinfo" | cut -d' ' -f3)
            if command -v tea >/dev/null 2>&1; then
                _tea_cnt=$(tea issues --repo "${_rf_owner}/${_rf_repo}" --state open --fields index 2>/dev/null | tr -cd '0-9\n' | grep -v '^$' | wc -l | tr -d ' ' || echo "0")
                [ -n "$_tea_cnt" ] && [ "$_tea_cnt" -gt 0 ] && _rissues="$_tea_cnt"
            fi
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$_rname" "$_rbranch" "$_rdirty" "$_runpushed" "$_rissues" "$_rpath"
    return 0
}

# -----------------------------------------------------------------------------
# Main Subcommand: cmd_repo
# Aggregated multi-repository overview, status, and management (Ticket #38)
# -----------------------------------------------------------------------------
cmd_repo() {
    _opt_dirty=0
    _opt_unpushed=0
    _opt_issues=0
    _opt_json=0
    _opt_fetch=0
    _opt_dir=""
    _opt_catalog=""

    while [ $# -gt 0 ]; do
        case "$1" in
            "status"|"list"|"overview")
                shift
                ;;
            "-h"|"--help"|"help")
                if command -v show_help >/dev/null 2>&1; then
                    show_help "repo"
                else
                    echo "Usage: cccp repo [status|list] [--dirty] [--unpushed] [--issues] [--json] [-d <dir>] [-c <catalog>] [-f|--fetch]"
                fi
                return 0
                ;;
            "--dirty")
                _opt_dirty=1
                shift
                ;;
            "--unpushed")
                _opt_unpushed=1
                shift
                ;;
            "--issues"|"--tickets")
                _opt_issues=1
                shift
                ;;
            "--json")
                _opt_json=1
                shift
                ;;
            "-f"|"--fetch")
                _opt_fetch=1
                shift
                ;;
            "-d"|"--dir")
                shift
                [ $# -gt 0 ] || { echo "Error: --dir requires directory path" >&2; return 1; }
                _opt_dir="$1"
                shift
                ;;
            "-c"|"--catalog")
                shift
                [ $# -gt 0 ] || { echo "Error: --catalog requires file path" >&2; return 1; }
                _opt_catalog="$1"
                shift
                ;;
            *)
                if [ -d "$1" ]; then
                    _opt_dir="$1"
                    shift
                else
                    echo "Error: Unknown option or subcommand: $1" >&2
                    echo "Run 'cccp repo --help' for usage." >&2
                    return 1
                fi
                ;;
        esac
    done

    _discovered=$(_repo_discover_list "$_opt_dir" "$_opt_catalog" 2>/dev/null | sort -u || true)

    if [ -z "$_discovered" ]; then
        if [ "$_opt_json" -eq 1 ]; then
            printf '[\n]\n'
        else
            echo "Nenhum repositório Git encontrado no escopo especificado."
        fi
        return 0
    fi

    _repo_colors

    _total_scanned=0
    _total_matched=0
    _total_dirty=0
    _total_unpushed=0
    _total_issues=0

    _matching_rows=""
    _first_json=1

    if [ "$_opt_json" -eq 1 ]; then
        printf '[\n'
    fi

    # Audit each discovered repository
    while IFS= read -r _rpath || [ -n "$_rpath" ]; do
        [ -n "$_rpath" ] || continue
        _row=$(_repo_audit_single "$_rpath" "$_opt_fetch" 2>/dev/null || true)
        [ -n "$_row" ] || continue

        _name=$(printf '%s' "$_row" | cut -f1)
        _branch=$(printf '%s' "$_row" | cut -f2)
        _dirty=$(printf '%s' "$_row" | cut -f3)
        _unpushed=$(printf '%s' "$_row" | cut -f4)
        _issues=$(printf '%s' "$_row" | cut -f5)
        _path=$(printf '%s' "$_row" | cut -f6)

        _total_scanned=$((_total_scanned + 1))
        [ "$_dirty" -gt 0 ] && _total_dirty=$((_total_dirty + 1))
        [ "$_unpushed" -gt 0 ] && _total_unpushed=$((_total_unpushed + 1))
        [ "$_issues" -gt 0 ] && _total_issues=$((_total_issues + 1))

        # Filter check
        if [ "$_opt_dirty" -eq 1 ] || [ "$_opt_unpushed" -eq 1 ] || [ "$_opt_issues" -eq 1 ]; then
            _matches=0
            [ "$_opt_dirty" -eq 1 ] && [ "$_dirty" -gt 0 ] && _matches=1
            [ "$_opt_unpushed" -eq 1 ] && [ "$_unpushed" -gt 0 ] && _matches=1
            [ "$_opt_issues" -eq 1 ] && [ "$_issues" -gt 0 ] && _matches=1
            [ "$_matches" -eq 1 ] || continue
        fi

        _total_matched=$((_total_matched + 1))

        # Determine textual status & Gosplan label
        if [ "$_dirty" -gt 0 ] && [ "$_unpushed" -gt 0 ]; then
            _status_txt="DIRTY/UNPUSHED"
            _status_label="MODIFICADO / PENDENTE"
            _status_color="$REPO_C_RED"
        elif [ "$_dirty" -gt 0 ]; then
            _status_txt="DIRTY"
            _status_label="MODIFICADO"
            _status_color="$REPO_C_YELLOW"
        elif [ "$_unpushed" -gt 0 ]; then
            _status_txt="UNPUSHED"
            _status_label="PENDENTE DE ENVIO"
            _status_color="$REPO_C_CYAN"
        elif [ "$_issues" -gt 0 ]; then
            _status_txt="ISSUES"
            _status_label="COM TAREFAS"
            _status_color="$REPO_C_CYAN"
        else
            _status_txt="CLEAN"
            _status_label="EM CONFORMIDADE"
            _status_color="$REPO_C_GREEN"
        fi

        if [ "$_opt_json" -eq 1 ]; then
            [ "$_first_json" -eq 1 ] && _first_json=0 || printf ',\n'
            cat <<EOF
  {
    "name": "$(escape_json_string "$_name")",
    "path": "$(escape_json_string "$_path")",
    "branch": "$(escape_json_string "$_branch")",
    "dirty": $_dirty,
    "unpushed": $_unpushed,
    "issues": $_issues,
    "status": "$(escape_json_string "$_status_txt")"
  }
EOF
        else
            # Truncate strings for neat Gosplan tabular formatting
            _name_trunc=$(printf '%.22s' "$_name")
            _branch_trunc=$(printf '%.20s' "$_branch")
            
            # Format row
            _matching_rows="${_matching_rows}$(printf '%-22s %-20s %6s %9s %8s  %b[%s]%b\n' \
                "$_name_trunc" "$_branch_trunc" "$_dirty" "$_unpushed" "$_issues" \
                "$_status_color" "$_status_label" "$REPO_C_RESET")
"
        fi
    done <<EOF
$_discovered
EOF

    if [ "$_opt_json" -eq 1 ]; then
        printf '\n]\n'
        return 0
    fi

    # Display Gosplan Header
    printf '%b╔══════════════════════════════════════════════════════════════════════════════════════╗%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"
    printf '%b║             ГОСПЛАН — REGISTRO GERAL E STATUS DE COOPERATIVAS (REPOS)                ║%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"
    printf '%b╚══════════════════════════════════════════════════════════════════════════════════════╝%b\n' "$REPO_C_BOLD" "$REPO_C_RESET"

    _filter_desc=""
    [ "$_opt_dirty" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }modificados (dirty)"
    [ "$_opt_unpushed" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }pendentes de envio (unpushed)"
    [ "$_opt_issues" -eq 1 ] && _filter_desc="${_filter_desc:+${_filter_desc}, }com tickets em aberto (issues)"

    if [ -n "$_filter_desc" ]; then
        printf 'Filtro ativo: apenas %s\n' "$_filter_desc"
    fi
    echo ""

    if [ -z "$_matching_rows" ]; then
        echo "Nenhum repositório correspondente aos filtros foi encontrado."
        echo ""
    else
        printf '%b%-22s %-20s %6s %9s %8s  %s%b\n' "$REPO_C_BOLD" "REPOSITÓRIO" "RAMAL" "DIRTY" "UNPUSHED" "TICKETS" "ESTADO" "$REPO_C_RESET"
        printf '%s\n' '----------------------------------------------------------------------------------------'
        printf '%b' "$_matching_rows"
        printf '%s\n' '----------------------------------------------------------------------------------------'
    fi

    # Display Gosplan Summary
    echo "Resumo do Plano Geral:"
    echo "  • Repositórios auditados: ${_total_scanned} (correspondentes: ${_total_matched})"
    echo "  • Com alterações não salvas (dirty): ${_total_dirty}"
    echo "  • Com commits pendentes de envio (unpushed): ${_total_unpushed}"
    echo "  • Com tarefas/tickets em aberto: ${_total_issues}"
    echo ""
    return 0
}

