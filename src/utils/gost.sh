#!/bin/sh

# =============================================================================
# CCCP GOST Compliance, Audit & Synchronization Engine
# Implements Ticket #47 & Ticket #42 according to GOST standard specification
# =============================================================================

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
    if [ -f "$GIT_ROOT/src/utils/comrades.sh" ]; then
        . "$GIT_ROOT/src/utils/comrades.sh"
    fi
fi

# -----------------------------------------------------------------------------
# Terminal Colors & NO_COLOR Support
# -----------------------------------------------------------------------------
_gost_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        GOST_C_RED='\033[0;31m'
        GOST_C_GREEN='\033[0;32m'
        GOST_C_YELLOW='\033[0;33m'
        GOST_C_CYAN='\033[0;36m'
        GOST_C_BOLD='\033[1m'
        GOST_C_RESET='\033[0m'
    else
        GOST_C_RED=''
        GOST_C_GREEN=''
        GOST_C_YELLOW=''
        GOST_C_CYAN=''
        GOST_C_BOLD=''
        GOST_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# HTTP Dispatcher for GOST Resources
# -----------------------------------------------------------------------------
_gost_http() {
    if command -v _cccp_http >/dev/null 2>&1; then
        _cccp_http "$@"
        return $?
    fi
    if [ -n "${CCCP_HTTP_CLIENT:-}" ]; then
        "$CCCP_HTTP_CLIENT" "$@"
        return $?
    fi
    if command -v sv >/dev/null 2>&1; then
        sv "$@"
        return $?
    fi
    if command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
        return $?
    fi
    if command -v curl >/dev/null 2>&1; then
        curl "$@"
        return $?
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -qO- "$@"
        return $?
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Read GOSTVER file in project root (Pure POSIX without arbitrary eval)
# -----------------------------------------------------------------------------
_gost_read_gostver() {
    _target_dir="${1:-.}"
    _gostver_file="${_target_dir}/GOSTVER"

    GOST_VERSION=""
    GOST_PROFILE_URL=""
    GOST_LANGUAGES=""

    if [ ! -f "$_gostver_file" ]; then
        return 1
    fi

    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            \#*|"") continue ;;
            GOST_VERSION=*)
                _val="${_line#GOST_VERSION=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_VERSION="$_val"
                ;;
            GOST_PROFILE_URL=*)
                _val="${_line#GOST_PROFILE_URL=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_PROFILE_URL="$_val"
                ;;
            GOST_LANGUAGES=*)
                _val="${_line#GOST_LANGUAGES=}"
                _val="${_val#\"}"
                _val="${_val%\"}"
                _val="${_val#\'}"
                _val="${_val%\'}"
                GOST_LANGUAGES="$_val"
                ;;
        esac
    done < "$_gostver_file"

    [ -n "$GOST_VERSION" ] || return 1
    return 0
}

# -----------------------------------------------------------------------------
# Fetch Resource from GOST upstream or local development fallback
# -----------------------------------------------------------------------------
_gost_fetch_resource() {
    _target_res="$1"     # e.g. "SPEC.md", "profiles/lumen.md", "specs/rules.json", "specs/languages/shell.md"
    _target_dest="$2"    # destination file path
    _ver="${3:-${GOST_VERSION:-main}}"

    _dest_dir="$(dirname "$_target_dest")"
    mkdir -p "$_dest_dir"

    # 1. Local fast-path / offline mode if GOST_LOCAL_PATH or sibling repository exists
    _local_gost=""
    if [ -n "${GOST_LOCAL_PATH:-}" ] && [ -d "$GOST_LOCAL_PATH" ]; then
        _local_gost="$GOST_LOCAL_PATH"
    elif [ -d "/home/lumen/src/gost" ]; then
        _local_gost="/home/lumen/src/gost"
    elif [ -d "../gost" ]; then
        _local_gost="../gost"
    fi

    if [ -n "$_local_gost" ] && [ -f "${_local_gost}/${_target_res}" ]; then
        cp "${_local_gost}/${_target_res}" "$_target_dest"
        return 0
    fi

    # 2. Remote download via _gost_http
    case "$_target_res" in
        http://*|https://*)
            _gost_http -fsSL "$_target_res" -o "$_target_dest" 2>/dev/null
            return $?
            ;;
    esac

    # Try tag with 'v' prefix
    _tag_url="https://codeberg.org/lumenpink/gost/raw/tag/v${_ver}/${_target_res}"
    if _gost_http -fsSL "$_tag_url" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    # Try tag without 'v' prefix
    _tag_url_nov="https://codeberg.org/lumenpink/gost/raw/tag/${_ver}/${_target_res}"
    if _gost_http -fsSL "$_tag_url_nov" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    # Fallback to main branch
    _main_url="https://codeberg.org/lumenpink/gost/raw/branch/main/${_target_res}"
    if _gost_http -fsSL "$_main_url" -o "$_target_dest" 2>/dev/null; then
        return 0
    fi

    return 1
}

# -----------------------------------------------------------------------------
# SemVer 2.0.0 Validator (POSIX Pure)
# -----------------------------------------------------------------------------
_gost_is_semver() {
    _ver="$1"
    printf "%s\n" "$_ver" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
}

# -----------------------------------------------------------------------------
# cccp gost sync: Synchronize GOST standard and language specs to .gost/
# -----------------------------------------------------------------------------
cmd_gost_sync() {
    _gost_colors
    _root_dir="${1:-.}"

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bErro: Arquivo GOSTVER não encontrado em '%s'.%b\n" "$GOST_C_RED" "$_root_dir" "$GOST_C_RESET" >&2
        printf "Crie o arquivo GOSTVER na raiz do projeto ou execute 'cccp gost init'.\n" >&2
        return 1
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Sincronizando norma GOST v%s...%b\n" "$GOST_C_CYAN" "$GOST_VERSION" "$GOST_C_RESET"

    mkdir -p "${_root_dir}/.gost/languages"

    # 1. SPEC.md -> .gost/GOST.md
    printf "  Baixando especificação canônica (GOST.md)... "
    if _gost_fetch_resource "SPEC.md" "${_root_dir}/.gost/GOST.md" "$GOST_VERSION"; then
        printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    else
        printf "%bFALHA%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    # 2. GOST_PROFILE_URL -> .gost/PROFILE.md
    if [ -n "$GOST_PROFILE_URL" ]; then
        printf "  Baixando perfil (%s -> PROFILE.md)... " "$GOST_PROFILE_URL"
        if _gost_fetch_resource "$GOST_PROFILE_URL" "${_root_dir}/.gost/PROFILE.md" "$GOST_VERSION"; then
            printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
        else
            printf "%bAVISO (não foi possível obter o perfil)%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
        fi
    fi

    # 3. specs/rules.json -> .gost/rules.json
    printf "  Sincronizando catálogo de regras (rules.json)... "
    if _gost_fetch_resource "specs/rules.json" "${_root_dir}/.gost/rules.json" "$GOST_VERSION"; then
        printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    else
        printf "%bAVISO (regras remotas indisponíveis)%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
    fi

    # 4. Only specs for languages listed in GOST_LANGUAGES
    if [ -n "$GOST_LANGUAGES" ]; then
        _old_ifs="$IFS"
        IFS=','
        for _lang in $GOST_LANGUAGES; do
            _l="$(echo "$_lang" | tr -d ' ')"
            [ -n "$_l" ] || continue
            printf "  Baixando especificação de linguagem (%s.md)... " "$_l"
            if _gost_fetch_resource "specs/languages/${_l}.md" "${_root_dir}/.gost/languages/${_l}.md" "$GOST_VERSION"; then
                printf "%bOK%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
            else
                printf "%bAVISO (especificação '%s' não encontrada)%b\n" "$GOST_C_YELLOW" "$_l" "$GOST_C_RESET"
            fi
        done
        IFS="$_old_ifs"
    fi

    printf "%b✔ Sincronização GOST v%s concluída com sucesso em .gost/%b\n" "$GOST_C_GREEN" "$GOST_VERSION" "$GOST_C_RESET"
    return 0
}

# -----------------------------------------------------------------------------
# cccp gost check / cccp check: Audit project against GOST rules
# -----------------------------------------------------------------------------
cmd_gost_check() {
    _gost_colors
    _root_dir="${1:-.}"
    _strict_mode="${2:-0}"

    _passed=0
    _warnings=0
    _failures=0

    _gost_ver_display="Não declarado"
    _gost_prof_display="Nenhum"
    _gost_lang_display="Nenhuma"

    if _gost_read_gostver "$_root_dir"; then
        _gost_ver_display="v$GOST_VERSION"
        [ -n "$GOST_PROFILE_URL" ] && _gost_prof_display="$GOST_PROFILE_URL"
        [ -n "$GOST_LANGUAGES" ] && _gost_lang_display="$GOST_LANGUAGES"
    fi

    printf "\n%b☭ [ГОСТ / GOSPLAN] COMISSÃO DE CONTROLE DE QUALIDADE GOST%b\n" "$GOST_C_BOLD" "$GOST_C_RESET"
    printf "Norma: %s | Perfil: %s | Linguagens: %s\n" "$_gost_ver_display" "$_gost_prof_display" "$_gost_lang_display"
    printf "────────────────────────────────────────────────────────────────────────────\n"

    # 1. GOST-16-GOSTVER-COMPLIANCE (MUST / SHOULD)
    if [ -f "${_root_dir}/GOSTVER" ] && [ -n "$GOST_VERSION" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-16-GOSTVER" "Declaração de Versão via GOSTVER ($GOST_VERSION)"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-16-GOSTVER" "Arquivo GOSTVER ausente ou sem GOST_VERSION declarada"
        _failures=$((_failures + 1))
    fi

    # 2. GOST-01-LICENSE (MUST)
    if [ -f "${_root_dir}/LICENSE" ] && [ -s "${_root_dir}/LICENSE" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-01-LICENSE" "Arquivo LICENSE declarado e não-vazio"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-01-LICENSE" "Arquivo LICENSE ausente ou vazio"
        _failures=$((_failures + 1))
    fi

    # 3. GOST-02-ANATOMY-ROOT (MUST)
    _missing_root=""
    for _req_file in "README.md" "VERSION" "CHANGELOG.md" ".gitignore" ".editorconfig"; do
        if [ ! -f "${_root_dir}/${_req_file}" ]; then
            _missing_root="${_missing_root} ${_req_file}"
        fi
    done

    if [ -z "$_missing_root" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-02-ANATOMY-ROOT" "Arquivos básicos da raiz presentes"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-02-ANATOMY-ROOT" "Arquivos ausentes na raiz:${_missing_root}"
        _failures=$((_failures + 1))
    fi

    # 4. GOST-03-VERSION-SEMVER (MUST)
    if [ -f "${_root_dir}/VERSION" ]; then
        _v_content="$(head -n 1 "${_root_dir}/VERSION" 2>/dev/null | tr -d '\r\n ' || echo "")"
        if _gost_is_semver "$_v_content"; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Arquivo VERSION SemVer 2.0.0 ($_v_content)"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Versão '$_v_content' não é SemVer 2.0.0 estrito"
            _failures=$((_failures + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-03-VERSION-SEMVER" "Arquivo VERSION inexistente"
        _failures=$((_failures + 1))
    fi

    # 5. GOST-04-TICKETS-COMMITS (MUST)
    if [ -d "${_root_dir}/.git" ] && command -v git >/dev/null 2>&1; then
        _commits_count=0
        _commits_valid=0
        _commit_log="$(cd "$_root_dir" && git log -n 5 --format="%s" 2>/dev/null || true)"
        if [ -n "$_commit_log" ]; then
            while IFS= read -r _subj || [ -n "$_subj" ]; do
                [ -n "$_subj" ] || continue
                _commits_count=$((_commits_count + 1))
                if printf "%s\n" "$_subj" | grep -Eq '\((ref|closes) #[0-9]+\)|(ref|closes) #[0-9]+|\(#[0-9]+\)'; then
                    _commits_valid=$((_commits_valid + 1))
                fi
            done <<EOF
$_commit_log
EOF
            if [ "$_commits_count" -gt 0 ] && [ "$_commits_valid" -eq "$_commits_count" ]; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Rastreabilidade estrita de tickets (${_commits_valid}/${_commits_count} commits recentes)"
                _passed=$((_passed + 1))
            elif [ "$_commits_count" -gt 0 ]; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Commits recentes sem referência de ticket (${_commits_valid}/${_commits_count} conformes)"
                _failures=$((_failures + 1))
            else
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Repositório novo (sem commits para auditar)"
                _passed=$((_passed + 1))
            fi
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Repositório novo (sem histórico de commits)"
            _passed=$((_passed + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-04-TICKETS-COMMITS" "Não é um repositório Git"
        _warnings=$((_warnings + 1))
    fi

    # 6. GOST-05-BRANCH-PER-TICKET (MUST)
    if [ -d "${_root_dir}/.git" ] && command -v git >/dev/null 2>&1; then
        _cur_branch="$(cd "$_root_dir" && git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")"
        if [ "$_cur_branch" = "main" ] || [ "$_cur_branch" = "master" ]; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Branch canônica/release ($_cur_branch)"
            _passed=$((_passed + 1))
        elif printf "%s\n" "$_cur_branch" | grep -Eq '^(feat|fix|docs|refactor|test|chore)/[0-9]+-[a-z0-9-]+$'; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Branch dedicada por ticket ($_cur_branch)"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_RED" "[FAIL]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Nome da branch '$_cur_branch' fora do padrão GOST"
            _failures=$((_failures + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-05-BRANCH-TICKET" "Git indisponível para auditar branches"
        _warnings=$((_warnings + 1))
    fi

    # 7. GOST-09-SITE-HEADERS (SHOULD / MUST if site exists)
    if [ -d "${_root_dir}/site" ]; then
        if [ -f "${_root_dir}/site/_headers" ]; then
            if grep -q "X-Content-Type-Options: nosniff" "${_root_dir}/site/_headers" && grep -Eq "X-Frame-Options: (DENY|SAMEORIGIN)" "${_root_dir}/site/_headers"; then
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Cabeçalhos HTTP de segurança presentes em site/_headers"
                _passed=$((_passed + 1))
            else
                printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "site/_headers não contém nosniff ou DENY/SAMEORIGIN"
                _warnings=$((_warnings + 1))
            fi
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Diretório site/ presente sem site/_headers"
            _warnings=$((_warnings + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-09-SITE-HEADERS" "Vitrine web não declarada (N/A)"
        _passed=$((_passed + 1))
    fi

    # 8. GOST-10-CI-SEPARATION (SHOULD)
    if [ -d "${_root_dir}/.forgejo/workflows" ] || [ -d "${_root_dir}/.github/workflows" ]; then
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-10-CI-SEPARATION" "Workflows de automação CI presentes"
        _passed=$((_passed + 1))
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-10-CI-SEPARATION" "Nenhum workflow de CI (.forgejo/.github) detectado"
        _warnings=$((_warnings + 1))
    fi

    # 9. GOST-17-LANGUAGE-SPECS (SHOULD)
    if [ -n "$GOST_LANGUAGES" ]; then
        _missing_specs=""
        _old_ifs="$IFS"
        IFS=','
        for _l in $GOST_LANGUAGES; do
            _lang="$(echo "$_l" | tr -d ' ')"
            [ -n "$_lang" ] || continue
            if [ ! -f "${_root_dir}/.gost/languages/${_lang}.md" ]; then
                _missing_specs="${_missing_specs} ${_lang}"
            fi
        done
        IFS="$_old_ifs"

        if [ -z "$_missing_specs" ]; then
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Especificações modulares sincronizadas em .gost/languages/"
            _passed=$((_passed + 1))
        else
            printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_YELLOW" "[WARN]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Especificações ausentes:${_missing_specs} (execute 'cccp gost sync')"
            _warnings=$((_warnings + 1))
        fi
    else
        printf "  %b%-6s%b %-25s : %s\n" "$GOST_C_GREEN" "[PASS]" "$GOST_C_RESET" "GOST-17-LANGUAGE-SPECS" "Nenhuma especificação de linguagem declarada"
        _passed=$((_passed + 1))
    fi

    printf "────────────────────────────────────────────────────────────────────────────\n"
    _total=$((_passed + _warnings + _failures))
    _pct=0
    if [ "$_total" -gt 0 ]; then
        _pct=$(( (_passed * 100) / _total ))
    fi

    printf "Total de verificações: %d | Conformes: %b%d%b | Avisos: %b%d%b | Falhas: %b%d%b\n" \
        "$_total" \
        "$GOST_C_GREEN" "$_passed" "$GOST_C_RESET" \
        "$GOST_C_YELLOW" "$_warnings" "$GOST_C_RESET" \
        "$GOST_C_RED" "$_failures" "$GOST_C_RESET"
    printf "Índice de Conformidade Gosplan: %b%d%%%b\n\n" "$GOST_C_BOLD" "$_pct" "$GOST_C_RESET"

    if [ "$_failures" -gt 0 ]; then
        printf "%b✘ Inconformidades críticas detectadas. Corrija os itens marcados com [FAIL].%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    if [ "$_strict_mode" = "1" ] && [ "$_warnings" -gt 0 ]; then
        printf "%b✘ Modo estrito ativado: avisos tratados como falhas bloqueantes.%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        return 1
    fi

    printf "%b✔ Projeto aprovado pela Comissão de Normas Técnicas GOST.%b\n" "$GOST_C_GREEN" "$GOST_C_RESET"
    return 0
}

# -----------------------------------------------------------------------------
# cccp gost upgrade <versão>: Show migration guide, bump GOSTVER and re-sync
# -----------------------------------------------------------------------------
cmd_gost_upgrade() {
    _gost_colors
    _target_version="${1:-}"
    _root_dir="${2:-.}"

    if [ -z "$_target_version" ]; then
        printf "%bErro: Versão alvo não informada.%b\n" "$GOST_C_RED" "$GOST_C_RESET" >&2
        printf "Uso: cccp gost upgrade <versão>\n" >&2
        printf "Exemplo: cccp gost upgrade 1.1.0\n" >&2
        return 1
    fi

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bErro: Arquivo GOSTVER não encontrado em '%s'.%b\n" "$GOST_C_RED" "$_root_dir" "$GOST_C_RESET" >&2
        return 1
    fi

    _curr_ver="$GOST_VERSION"
    if [ "$_curr_ver" = "$_target_version" ]; then
        printf "%bO projeto já está na norma GOST v%s.%b\n" "$GOST_C_YELLOW" "$_curr_ver" "$GOST_C_RESET"
        return 0
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Migração de Norma: v%s → v%s%b\n" "$GOST_C_CYAN" "$_curr_ver" "$_target_version" "$GOST_C_RESET"

    # Search for migration guide: upgrades/from-X-to-Y.md
    _curr_mm="$(echo "$_curr_ver" | cut -d. -f1,2 | tr -d 'v')"
    _target_mm="$(echo "$_target_version" | cut -d. -f1,2 | tr -d 'v')"
    _guide_res="upgrades/from-${_curr_mm}-to-${_target_mm}.md"

    _tmp_guide="/tmp/gost_guide_$$.md"
    if _gost_fetch_resource "$_guide_res" "$_tmp_guide" "$_target_version"; then
        printf "\n%b=== GUIA DE MIGRAÇÃO (%s) ===%b\n" "$GOST_C_BOLD" "$_guide_res" "$GOST_C_RESET"
        cat "$_tmp_guide"
        printf "%b=====================================%b\n\n" "$GOST_C_BOLD" "$GOST_C_RESET"
        rm -f "$_tmp_guide"
    else
        printf "  (Nenhum guia de migração específico encontrado para %s → %s)\n\n" "$_curr_mm" "$_target_mm"
    fi

    # Update GOSTVER file
    _gostver_file="${_root_dir}/GOSTVER"
    _tmp_gostver="/tmp/GOSTVER.$$"
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in
            GOST_VERSION=*)
                printf 'GOST_VERSION="%s"\n' "$_target_version" >> "$_tmp_gostver"
                ;;
            *)
                printf '%s\n' "$_line" >> "$_tmp_gostver"
                ;;
        esac
    done < "$_gostver_file"

    mv "$_tmp_gostver" "$_gostver_file"
    printf "%b✔ GOSTVER atualizado para GOST_VERSION=\"%s\"%b\n" "$GOST_C_GREEN" "$_target_version" "$GOST_C_RESET"

    # Synchronize updated version
    cmd_gost_sync "$_root_dir"
    return $?
}

# -----------------------------------------------------------------------------
# cccp gost init: Initialize a default GOSTVER file
# -----------------------------------------------------------------------------
cmd_gost_init() {
    _gost_colors
    _root_dir="${1:-.}"
    _gostver_file="${_root_dir}/GOSTVER"

    if [ -f "$_gostver_file" ]; then
        printf "%bArquivo GOSTVER já existe em '%s'.%b\n" "$GOST_C_YELLOW" "$_root_dir" "$GOST_C_RESET"
        return 0
    fi

    cat > "$_gostver_file" << 'EOF'
# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.0.0-draft"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="shell"
EOF

    printf "%b✔ Arquivo GOSTVER criado com sucesso em '%s'.%b\n" "$GOST_C_GREEN" "$_gostver_file" "$GOST_C_RESET"
    cmd_gost_sync "$_root_dir"
}

# -----------------------------------------------------------------------------
# cccp gost info: Display current GOST status
# -----------------------------------------------------------------------------
cmd_gost_info() {
    _gost_colors
    _root_dir="${1:-.}"

    if ! _gost_read_gostver "$_root_dir"; then
        printf "%bProjeto não possui declaração GOSTVER.%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
        return 1
    fi

    printf "%b✦ [ГОСТ / GOSPLAN] Status da Norma GOST:%b\n" "$GOST_C_BOLD" "$GOST_C_RESET"
    printf "  Versão Declarada:    %b%s%b\n" "$GOST_C_GREEN" "$GOST_VERSION" "$GOST_C_RESET"
    printf "  Perfil de Referência: %s\n" "${GOST_PROFILE_URL:-Nenhum}"
    printf "  Linguagens Ativas:   %s\n" "${GOST_LANGUAGES:-Nenhuma}"

    if [ -d "${_root_dir}/.gost" ]; then
        printf "  Cache Local (.gost): Sincronizado\n"
        [ -f "${_root_dir}/.gost/GOST.md" ] && printf "    - GOST.md: presente\n"
        [ -f "${_root_dir}/.gost/PROFILE.md" ] && printf "    - PROFILE.md: presente\n"
        [ -f "${_root_dir}/.gost/rules.json" ] && printf "    - rules.json: presente\n"
    else
        printf "  Cache Local (.gost): %bNão sincronizado (execute 'cccp gost sync')%b\n" "$GOST_C_YELLOW" "$GOST_C_RESET"
    fi
}

# -----------------------------------------------------------------------------
# cccp gost help
# -----------------------------------------------------------------------------
cmd_gost_help() {
    printf "Uso: cccp gost <subcomando> [opções]\n\n"
    printf "Gerenciador de Conformidade e Auditoria do Padrão GOST (Государственный стандарт).\n\n"
    printf "Subcomandos:\n"
    printf "  sync             Sincroniza GOST.md, PROFILE.md e especificações em .gost/\n"
    printf "  check, audit     Audita a conformidade das regras GOST no projeto\n"
    printf "  upgrade <versão> Exibe guia de migração, atualiza GOSTVER e sincroniza cache\n"
    printf "  init             Inicializa arquivo GOSTVER na raiz do projeto\n"
    printf "  info, status     Exibe informações do GOSTVER e do cache local\n"
    printf "  help             Exibe esta mensagem de ajuda\n\n"
    printf "Opções globais:\n"
    printf "  -C, --dir <dir>  Executa o comando no diretório ou repositório informado\n"
}

# -----------------------------------------------------------------------------
# cccp check (Top-level command satisfying Ticket #42)
# -----------------------------------------------------------------------------
cmd_check() {
    _strict=0
    _dir="."

    while [ $# -gt 0 ]; do
        case "$1" in
            --strict)
                _strict=1
                shift
                ;;
            -C|--dir)
                shift
                [ $# -gt 0 ] || { echo "Erro: $1 requer caminho do diretório" >&2; return 1; }
                _dir="$1"
                shift
                ;;
            -h|--help)
                cmd_check_help
                return 0
                ;;
            *)
                if [ -d "$1" ]; then
                    _dir="$1"
                else
                    echo "Opção ou diretório desconhecido: $1" >&2
                    return 1
                fi
                shift
                ;;
        esac
    done

    cmd_gost_check "$_dir" "$_strict"
}

cmd_check_help() {
    printf "Uso: cccp check [opções] [diretório]\n\n"
    printf "Executa a auditoria padrão de conformidade com a especificação GOST.\n\n"
    printf "Opções:\n"
    printf "  --strict        Trata avisos (SHOULD) como erros bloqueantes\n"
    printf "  -C, --dir <dir> Diretório alvo da verificação\n"
    printf "  -h, --help      Exibe esta mensagem de ajuda\n"
}

# -----------------------------------------------------------------------------
# Dispatcher for cccp gost subcommands
# -----------------------------------------------------------------------------
cmd_gost() {
    _subcmd="${1:-}"
    case "$_subcmd" in
        "sync")
            shift || true
            cmd_gost_sync "$@"
            ;;
        "check"|"audit")
            shift || true
            cmd_gost_check "$@"
            ;;
        "upgrade")
            shift || true
            cmd_gost_upgrade "$@"
            ;;
        "init")
            shift || true
            cmd_gost_init "$@"
            ;;
        "info"|"status")
            shift || true
            cmd_gost_info "$@"
            ;;
        "help"|"-h"|"--help"|"")
            cmd_gost_help
            ;;
        *)
            _gost_colors
            printf "%bErro: Subcomando desconhecido 'gost %s'.%b\n\n" "$GOST_C_RED" "$_subcmd" "$GOST_C_RESET" >&2
            cmd_gost_help
            return 1
            ;;
    esac
}
