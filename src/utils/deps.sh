#!/bin/sh
# =============================================================================
# CCCP Dependencies Manager (cmd_deps)
# Centralized catalog, constraint verification, and automated acquisition for
# external binaries, CI runners (act), and Gosplan ecosystem comrades.
# Strict zero-awk, pure POSIX compliant.
# =============================================================================
set -eu

# -----------------------------------------------------------------------------
# Terminal Colors & Styling
# -----------------------------------------------------------------------------
_deps_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        DEPS_C_RED='\033[0;31m'
        DEPS_C_GREEN='\033[0;32m'
        DEPS_C_YELLOW='\033[0;33m'
        DEPS_C_CYAN='\033[0;36m'
        DEPS_C_BOLD='\033[1m'
        DEPS_C_RESET='\033[0m'
    else
        DEPS_C_RED=''
        DEPS_C_GREEN=''
        DEPS_C_YELLOW=''
        DEPS_C_CYAN=''
        DEPS_C_BOLD=''
        DEPS_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# Pure POSIX SemVer Comparator: _deps_version_ge <actual> <required>
# Returns 0 if actual >= required, 1 otherwise.
# -----------------------------------------------------------------------------
_deps_version_ge() {
    _d_v_act="${1#v}"
    _d_v_act="${_d_v_act#V}"
    _d_v_req="${2#v}"
    _d_v_req="${_d_v_req#V}"

    case "$_d_v_act" in
        dev*|development*|trunk*|master*|main*) return 0 ;;
    esac

    # Extract major, minor, patch for actual version
    _d_act_maj="${_d_v_act%%.*}"
    _d_rem="${_d_v_act#*.}"
    if [ "$_d_rem" != "$_d_v_act" ]; then
        _d_act_min="${_d_rem%%.*}"
        _d_rem="${_d_rem#*.}"
        _d_act_patch="${_d_rem%%[!0-9]*}"
    else
        _d_act_min=0
        _d_act_patch=0
    fi
    _d_act_maj=$(printf '%s' "$_d_act_maj" | tr -cd '0-9')
    _d_act_min=$(printf '%s' "$_d_act_min" | tr -cd '0-9')
    _d_act_patch=$(printf '%s' "${_d_act_patch:-0}" | tr -cd '0-9')
    [ -z "$_d_act_maj" ] && _d_act_maj=0
    [ -z "$_d_act_min" ] && _d_act_min=0
    [ -z "$_d_act_patch" ] && _d_act_patch=0

    # Extract major, minor, patch for required version
    _d_req_maj="${_d_v_req%%.*}"
    _d_rem="${_d_v_req#*.}"
    if [ "$_d_rem" != "$_d_v_req" ]; then
        _d_req_min="${_d_rem%%.*}"
        _d_rem="${_d_rem#*.}"
        _d_req_patch="${_d_rem%%[!0-9]*}"
    else
        _d_req_min=0
        _d_req_patch=0
    fi
    _d_req_maj=$(printf '%s' "$_d_req_maj" | tr -cd '0-9')
    _d_req_min=$(printf '%s' "$_d_req_min" | tr -cd '0-9')
    _d_req_patch=$(printf '%s' "${_d_req_patch:-0}" | tr -cd '0-9')
    [ -z "$_d_req_maj" ] && _d_req_maj=0
    [ -z "$_d_req_min" ] && _d_req_min=0
    [ -z "$_d_req_patch" ] && _d_req_patch=0

    if [ "$_d_act_maj" -gt "$_d_req_maj" ]; then
        return 0
    elif [ "$_d_act_maj" -lt "$_d_req_maj" ]; then
        return 1
    fi

    if [ "$_d_act_min" -gt "$_d_req_min" ]; then
        return 0
    elif [ "$_d_act_min" -lt "$_d_req_min" ]; then
        return 1
    fi

    if [ "$_d_act_patch" -ge "$_d_req_patch" ]; then
        return 0
    fi
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Probe Active Binary Version in pure POSIX
# -----------------------------------------------------------------------------
_deps_get_version() {
    _d_name="$1"
    _raw_ver=""

    case "$_d_name" in
        git)
            _raw_ver=$(git --version 2>/dev/null | head -n 1 | sed -e 's/.*version[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        docker)
            _raw_ver=$(docker --version 2>/dev/null | sed -e 's/.*version[[:space:]]*//' -e 's/,.*//' -e 's/[[:space:]].*//' || true)
            ;;
        act)
            _raw_ver=$(act --version 2>/dev/null | sed -e 's/.*version[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        curl)
            _raw_ver=$(curl --version 2>/dev/null | head -n 1 | sed -e 's/curl[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        wget)
            _raw_ver=$(wget --version 2>/dev/null | head -n 1 | sed -e 's/GNU Wget[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        jq)
            _raw_ver=$(jq --version 2>/dev/null | sed -e 's/jq-//' -e 's/[[:space:]].*//' || true)
            ;;
        tea)
            _raw_ver=$(tea --version 2>/dev/null | head -n 1 | sed -e 's/.*[Vv]ersion:[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        gh)
            _raw_ver=$(gh --version 2>/dev/null | head -n 1 | sed -e 's/.*version[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            ;;
        kv|kvant)
            if command -v kv >/dev/null 2>&1; then
                _raw_ver=$(kv --version 2>/dev/null | head -n 1 | sed -e 's/.*kvant[[:space:]]*//' -e 's/.*kv[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            elif command -v kvant >/dev/null 2>&1; then
                _raw_ver=$(kvant --version 2>/dev/null | head -n 1 | sed -e 's/.*kvant[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            fi
            ;;
        sv|svyaz)
            if command -v sv >/dev/null 2>&1; then
                _raw_ver=$(sv --version 2>/dev/null | head -n 1 | sed -e 's/.*svyaz[[:space:]]*//' -e 's/.*sv[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            elif command -v svyaz >/dev/null 2>&1; then
                _raw_ver=$(svyaz --version 2>/dev/null | head -n 1 | sed -e 's/.*svyaz[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            fi
            ;;
        mk|mayak)
            if command -v mk >/dev/null 2>&1; then
                _raw_ver=$(mk --version 2>/dev/null | head -n 1 | sed -e 's/.*mayak[[:space:]]*//' -e 's/.*mk[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            elif command -v mayak >/dev/null 2>&1; then
                _raw_ver=$(mayak --version 2>/dev/null | head -n 1 | sed -e 's/.*mayak[[:space:]]*//' -e 's/[[:space:]].*//' || true)
            fi
            ;;
    esac

    _raw_ver=$(printf '%s' "$_raw_ver" | tr -d '\r\n')
    printf '%s\n' "$_raw_ver"
}

# -----------------------------------------------------------------------------
# Helper: Detect Host Package Manager
# -----------------------------------------------------------------------------
_deps_get_system_pkg_mgr() {
    if [ -f /etc/arch-release ] || [ -f /etc/artix-release ]; then
        printf 'pacman -S'
    elif [ -f /etc/debian_version ]; then
        printf 'apt-get install -y'
    elif [ -f /etc/fedora-release ] || [ -f /etc/redhat-release ]; then
        printf 'dnf install -y'
    elif [ -f /etc/alpine-release ]; then
        printf 'apk add'
    elif [ "$(uname -s 2>/dev/null || true)" = "Darwin" ]; then
        printf 'brew install'
    elif [ -f /etc/os-release ]; then
        _id=$(grep -E '^ID=' /etc/os-release 2>/dev/null | head -n 1 | sed -e 's/^ID=//' -e 's/"//g' || true)
        case "$_id" in
            arch|manjaro|endeavouros) printf 'pacman -S' ;;
            debian|ubuntu|pop|linuxmint) printf 'apt-get install -y' ;;
            fedora|rhel|centos) printf 'dnf install -y' ;;
            alpine) printf 'apk add' ;;
            *) printf 'unknown' ;;
        esac
    else
        printf 'unknown'
    fi
}

# -----------------------------------------------------------------------------
# Dependency Catalog: _deps_get_catalog
# Output fields separated by '|':
# Name | Category | Requirement | Constraint | Source | Description
# -----------------------------------------------------------------------------
_deps_get_catalog() {
    cat << 'EOF'
git|core|Obrigatório|2.25.0|sistema|Sistema de controle de versão essencial para o CCCP
docker|ci|Recomendado|20.10.0|sistema|Engine de contêineres para execução do CI gate local (act)
act|ci|Opcional|0.2.50|gh:nektos/act|Runner local para workflows de CI (GitHub/Forgejo Actions)
sv|comrade|Opcional|0.1.0|codeberg:lumenpink/svyaz|Camarada micro HTTP/HTTPS em Rust com TLS embutido
kv|comrade|Opcional|0.1.0|codeberg:lumenpink/kvant|Camarada micro editor de texto de terminal em Rust
mk|comrade|Opcional|0.1.0|codeberg:lumenpink/mayak|Camarada micro servidor estático com Mayak Command Bridge
curl|http|Recomendado|7.60.0|sistema|Cliente HTTP para sincronização de tickets, atualizações e API
wget|http|Opcional|1.20.0|sistema|Cliente HTTP/FTP alternativo para downloads e atualizações
jq|util|Opcional|1.6|sistema|Acelerador opcional de extração e parsing de payloads JSON
tea|forge|Opcional|0.9.0|sistema|CLI oficial do Gitea / Codeberg para gestão de repositórios
gh|forge|Opcional|2.0.0|sistema|CLI oficial do GitHub para integração e despacho de tickets
EOF
}

# -----------------------------------------------------------------------------
# Subcommand: cccp deps list
# -----------------------------------------------------------------------------
cmd_deps_list() {
    _deps_colors

    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b ★ [ГОСПЛАН] Catálogo de Dependências e Utilitários do Ecossistema CCCP ★%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b%-8s %-9s %-12s %-10s %-22s %s%b\n' "$DEPS_C_BOLD" "BINÁRIO" "CATEGORIA" "EXIGÊNCIA" "MÍNIMO" "ORIGEM" "PROPÓSITO" "$DEPS_C_RESET"
    printf '%-8s %-9s %-12s %-10s %-22s %s\n' "-------" "---------" "---------" "------" "------" "---------"

    _deps_get_catalog | while IFS='|' read -r _name _cat _req _min _src _desc; do
        [ -z "$_name" ] && continue
        printf '%-8s %-9s %-12s %-10s %-22s %s\n' "$_name" "$_cat" "$_req" ">= $_min" "$_src" "$_desc"
    done
    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    return 0
}

# -----------------------------------------------------------------------------
# Subcommand: cccp deps check [dep1 dep2...]
# -----------------------------------------------------------------------------
cmd_deps_check() {
    _deps_colors

    _filter_deps="$*"
    _has_critical_failure=0
    _total_checked=0
    _total_ok=0
    _total_missing=0
    _total_outdated=0

    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b ★ [ГОСПЛАН] Auditoria de Conformidade de Dependências (Deps Check) ★%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf '%b%-10s %-12s %-14s %-14s %-16s %s%b\n' "$DEPS_C_BOLD" "BINÁRIO" "EXIGÊNCIA" "INSTALADA" "REQUISITO" "ESTADO" "LOCALIZAÇÃO / DETALHES" "$DEPS_C_RESET"
    printf '%-10s %-12s %-14s %-14s %-16s %s\n' "-------" "---------" "---------" "---------" "------" "----------------------"

    _catalog=$(_deps_get_catalog)
    _old_ifs="$IFS"
    IFS='
'
    for _line in $_catalog; do
        IFS="$_old_ifs"
        [ -z "$_line" ] && continue
        _name=$(printf '%s' "$_line" | cut -d'|' -f1)
        _cat=$(printf '%s' "$_line" | cut -d'|' -f2)
        _req=$(printf '%s' "$_line" | cut -d'|' -f3)
        _min=$(printf '%s' "$_line" | cut -d'|' -f4)
        _src=$(printf '%s' "$_line" | cut -d'|' -f5)
        _desc=$(printf '%s' "$_line" | cut -d'|' -f6)

        # Filter check if user specified names
        if [ -n "$_filter_deps" ]; then
            _matched=0
            for _f in $_filter_deps; do
                if [ "$_f" = "$_name" ]; then
                    _matched=1
                    break
                fi
            done
            [ "$_matched" -eq 0 ] && continue
        fi

        _total_checked=$((_total_checked + 1))
        _bin_path=$(command -v "$_name" 2>/dev/null || true)

        # Fallback path for comrades with alternative aliases
        if [ -z "$_bin_path" ]; then
            case "$_name" in
                kv) _bin_path=$(command -v kvant 2>/dev/null || true) ;;
                sv) _bin_path=$(command -v svyaz 2>/dev/null || true) ;;
                mk) _bin_path=$(command -v mayak 2>/dev/null || true) ;;
            esac
        fi

        if [ -z "$_bin_path" ]; then
            # Missing binary
            _total_missing=$((_total_missing + 1))
            if [ "$_req" = "Obrigatório" ]; then
                _has_critical_failure=1
            fi
            printf '%-10s %-12s %-14s %-14s %b%-16s%b %s\n' \
                "$_name" "$_req" "-" ">= $_min" \
                "$DEPS_C_RED" "[AUSENTE]" "$DEPS_C_RESET" \
                "Não encontrado no PATH (Origem: $_src)"
            continue
        fi

        # Binary is present; determine version
        _cur_ver=$(_deps_get_version "$_name")
        [ -z "$_cur_ver" ] && _cur_ver="detectado"

        # Special check for docker daemon
        _extra_info=""
        if [ "$_name" = "docker" ]; then
            if docker info >/dev/null 2>&1 || docker ps >/dev/null 2>&1; then
                _extra_info="daemon operacional"
            else
                _extra_info="daemon parado ou sem permissão"
            fi
        fi

        # Compare version constraint
        _ver_ok=1
        if [ "$_cur_ver" != "detectado" ]; then
            if ! _deps_version_ge "$_cur_ver" "$_min"; then
                _ver_ok=0
            fi
        fi

        if [ "$_name" = "docker" ] && [ "$_extra_info" = "daemon parado ou sem permissão" ]; then
            _total_outdated=$((_total_outdated + 1))
            printf '%-10s %-12s %-14s %-14s %b%-16s%b %s (%s)\n' \
                "$_name" "$_req" "$_cur_ver" ">= $_min" \
                "$DEPS_C_YELLOW" "[DAEMON OFF]" "$DEPS_C_RESET" \
                "$_bin_path" "$_extra_info"
        elif [ "$_ver_ok" -eq 1 ]; then
            _total_ok=$((_total_ok + 1))
            _details="$_bin_path"
            [ -n "$_extra_info" ] && _details="$_details ($_extra_info)"
            printf '%-10s %-12s %-14s %-14s %b%-16s%b %s\n' \
                "$_name" "$_req" "$_cur_ver" ">= $_min" \
                "$DEPS_C_GREEN" "[OK]" "$DEPS_C_RESET" \
                "$_details"
        else
            _total_outdated=$((_total_outdated + 1))
            printf '%-10s %-12s %-14s %-14s %b%-16s%b %s (abaixo do requisito recomendado)\n' \
                "$_name" "$_req" "$_cur_ver" ">= $_min" \
                "$DEPS_C_YELLOW" "[DESATUALIZADO]" "$DEPS_C_RESET" \
                "$_bin_path"
        fi
    done
    IFS="$_old_ifs"

    printf '%b========================================================================================%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
    printf 'Total verificado: %d | Conformes: %b%d%b | Ausentes: %b%d%b | Alertas: %b%d%b\n' \
        "$_total_checked" \
        "$DEPS_C_GREEN" "$_total_ok" "$DEPS_C_RESET" \
        "$DEPS_C_RED" "$_total_missing" "$DEPS_C_RESET" \
        "$DEPS_C_YELLOW" "$_total_outdated" "$DEPS_C_RESET"

    if [ "$_has_critical_failure" -eq 1 ]; then
        printf '%b[FALHA] Dependências obrigatórias ausentes. O CCCP não poderá operar normalmente.%b\n' "$DEPS_C_RED" "$DEPS_C_RESET"
        return 1
    fi

    printf '%b[SUCESSO] Auditoria de dependências satisfeita.%b\n' "$DEPS_C_GREEN" "$DEPS_C_RESET"
    return 0
}

# -----------------------------------------------------------------------------
# Helper: Install act runner binary from GitHub Releases
# -----------------------------------------------------------------------------
_deps_install_act() {
    _silent="${1:-0}"
    _deps_colors

    _bdir=$(_cccp_get_bin_dir 2>/dev/null || true)
    if [ -z "$_bdir" ]; then
        _bdir="${HOME}/.local/bin"
        mkdir -p "$_bdir" 2>/dev/null || true
    fi

    _arch_raw=$(_cccp_detect_arch 2>/dev/null || echo "linux-x86_64")
    _act_asset=""

    case "$_arch_raw" in
        linux-x86_64) _act_asset="act_Linux_x86_64.tar.gz" ;;
        linux-aarch64) _act_asset="act_Linux_arm64.tar.gz" ;;
        linux-armv7) _act_asset="act_Linux_armv7.tar.gz" ;;
        macos-x86_64) _act_asset="act_Darwin_x86_64.tar.gz" ;;
        macos-aarch64) _act_asset="act_Darwin_arm64.tar.gz" ;;
        *)
            echo "Erro: Arquitetura host '$_arch_raw' não suportada diretamente para download automático do act." >&2
            echo "Consulte: https://github.com/nektos/act/releases" >&2
            return 1
            ;;
    esac

    _url="https://github.com/nektos/act/releases/latest/download/${_act_asset}"

    if [ "$_silent" -eq 0 ]; then
        printf '%b★ Baixando runner act do GitHub (%s)... ★%b\n' "$DEPS_C_CYAN" "$_act_asset" "$DEPS_C_RESET"
    fi

    _tmp_tar=$(mktemp 2>/dev/null || printf '/tmp/cccp_act_%s.tar.gz' "$$")
    if ! _cccp_download_raw "$_url" "$_tmp_tar"; then
        rm -f "$_tmp_tar"
        echo "Erro: Falha ao baixar release do act em: $_url" >&2
        return 1
    fi

    # Unpack binary directly to bin directory
    if ! tar -xzf "$_tmp_tar" -C "$_bdir" act 2>/dev/null; then
        # Try extracting without member specification in case of structure variation
        if ! tar -xzf "$_tmp_tar" -C "$_bdir" 2>/dev/null; then
            rm -f "$_tmp_tar"
            echo "Erro: Falha ao descompactar $_act_asset em $_bdir" >&2
            return 1
        fi
    fi

    rm -f "$_tmp_tar"
    chmod +x "${_bdir}/act" 2>/dev/null || true

    if command -v "${_bdir}/act" >/dev/null 2>&1 || [ -x "${_bdir}/act" ]; then
        _inst_ver=$("${_bdir}/act" --version 2>/dev/null | sed -e 's/.*version[[:space:]]*//' -e 's/[[:space:]].*//' || true)
        if [ "$_silent" -eq 0 ]; then
            printf '%b✅ act instalado com sucesso em %s/act (versão: %s)%b\n' "$DEPS_C_GREEN" "$_bdir" "$_inst_ver" "$DEPS_C_RESET"
        fi
        return 0
    fi

    echo "Erro: Binário act não encontrado em $_bdir após instalação." >&2
    return 1
}

# -----------------------------------------------------------------------------
# Helper: Install Single Dependency
# -----------------------------------------------------------------------------
_deps_install_single() {
    _name="$1"
    _deps_colors

    case "$_name" in
        act)
            _deps_install_act 0
            return $?
            ;;
        kv|kvant|sv|svyaz|mk|mayak)
            if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
                _cccp_ensure_comrade "$_name" 0
                return $?
            else
                echo "Erro: Módulo de camaradas indisponível para instalar '$_name'." >&2
                return 1
            fi
            ;;
        git|docker|curl|wget|jq|tea|gh)
            _pm=$(_deps_get_system_pkg_mgr)
            printf '%bDependência de sistema: %s%b\n' "$DEPS_C_BOLD" "$_name" "$DEPS_C_RESET"
            if [ "$_pm" != "unknown" ]; then
                printf '  Para instalar no seu sistema operacional, execute:\n'
                printf '    %bsudo %s %s%b\n\n' "$DEPS_C_CYAN" "$_pm" "$_name" "$DEPS_C_RESET"
            else
                printf '  Instale o pacote "%s" através do gerenciador de pacotes da sua distribuição.\n\n' "$_name"
            fi
            return 0
            ;;
        *)
            echo "Erro: Dependência desconhecida '$_name'." >&2
            echo "Consulte a lista disponível com: cccp deps list" >&2
            return 1
            ;;
    esac
}

# -----------------------------------------------------------------------------
# Subcommand: cccp deps install <dep1 dep2... | --all | all>
# -----------------------------------------------------------------------------
cmd_deps_install() {
    _deps_colors

    if [ $# -eq 0 ]; then
        echo "Uso: cccp deps install <nome-da-dependência ... | --all | all>"
        echo "Exemplo: cccp deps install act"
        echo "         cccp deps install sv kv mk"
        echo "         cccp deps install --all"
        return 1
    fi

    _target_all=0
    _targets=""

    for _arg in "$@"; do
        case "$_arg" in
            --all|all)
                _target_all=1
                ;;
            -h|--help)
                echo "Uso: cccp deps install <nome ... | --all>"
                return 0
                ;;
            *)
                _targets="${_targets} ${_arg}"
                ;;
        esac
    done

    if [ "$_target_all" -eq 1 ]; then
        # Install all user-space installable tools (act + comrades)
        printf '%b★ Instalando ferramentas standalone e camaradas do ecossistema... ★%b\n\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
        _deps_install_single "act" || true
        _deps_install_single "sv" || true
        _deps_install_single "kv" || true
        _deps_install_single "mk" || true

        # For system packages, provide helpful guidance
        _pm=$(_deps_get_system_pkg_mgr)
        printf '\n%bPara dependências gerenciadas pelo sistema operacional:%b\n' "$DEPS_C_BOLD" "$DEPS_C_RESET"
        if [ "$_pm" != "unknown" ]; then
            printf '  %bsudo %s git docker curl jq%b\n' "$DEPS_C_CYAN" "$_pm" "$DEPS_C_RESET"
        fi
        return 0
    fi

    _err=0
    for _dep in $_targets; do
        if ! _deps_install_single "$_dep"; then
            _err=1
        fi
    done
    return "$_err"
}

# -----------------------------------------------------------------------------
# Main Subcommand Router: cccp deps [subcommand] [options]
# -----------------------------------------------------------------------------
cmd_deps() {
    _subcmd="${1:-check}"

    case "$_subcmd" in
        list|ls)
            shift || true
            cmd_deps_list "$@"
            return $?
            ;;
        check|status)
            shift || true
            cmd_deps_check "$@"
            return $?
            ;;
        install|get|add)
            shift || true
            cmd_deps_install "$@"
            return $?
            ;;
        -h|--help|help)
            if command -v show_help >/dev/null 2>&1; then
                show_help "deps"
            else
                echo "Uso: cccp deps [list | check | install <nome>]"
            fi
            return 0
            ;;
        *)
            # If user passed dependency names directly to cccp deps, treat as check
            cmd_deps_check "$@"
            return $?
            ;;
    esac
}
