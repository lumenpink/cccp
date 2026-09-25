#!/bin/sh
# =============================================================================
# CCCP Comrades Manager & Unified HTTP Dispatcher
# Automatic recruitment, management, and network dispatch for Kvant (kv) & Svyaz (sv)
# Strict zero-awk, pure POSIX compliant.
# =============================================================================
set -eu

# -----------------------------------------------------------------------------
# Helper: Determine and ensure binary target directory (OpenDesktop / XDG)
# Priority: COMRADES_BIN_DIR -> XDG_BIN_HOME -> ~/.local/bin -> $XDG_DATA_HOME/cccp/bin
# -----------------------------------------------------------------------------
_cccp_get_bin_dir() {
    if [ -n "${COMRADES_BIN_DIR:-}" ]; then
        _bdir="$COMRADES_BIN_DIR"
    elif [ -n "${XDG_BIN_HOME:-}" ]; then
        _bdir="$XDG_BIN_HOME"
    elif [ -d "${HOME}/.local/bin" ] || mkdir -p "${HOME}/.local/bin" 2>/dev/null; then
        _bdir="${HOME}/.local/bin"
    else
        _bdir="${XDG_DATA_HOME:-$HOME/.local/share}/cccp/bin"
    fi
    mkdir -p "$_bdir" 2>/dev/null || true

    # Ensure target directory is in PATH for active shell execution
    case ":${PATH:-}:" in
        *":${_bdir}:"*) ;;
        *) export PATH="${_bdir}:${PATH:-}" ;;
    esac

    printf '%s\n' "$_bdir"
}

# -----------------------------------------------------------------------------
# Helper: Detect OS and CPU Architecture in pure POSIX
# Returns: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, etc.
# -----------------------------------------------------------------------------
_cccp_detect_arch() {
    _raw_os=$(uname -s 2>/dev/null | tr '[:upper:]' '[:lower:]' || echo "unknown")
    _raw_arch=$(uname -m 2>/dev/null | tr '[:upper:]' '[:lower:]' || echo "unknown")

    _os="unknown"
    case "$_raw_os" in
        linux*) _os="linux" ;;
        darwin*) _os="macos" ;;
        freebsd*|openbsd*|netbsd*) _os="bsd" ;;
    esac

    _arch="unknown"
    case "$_raw_arch" in
        x86_64|amd64) _arch="x86_64" ;;
        aarch64|arm64) _arch="aarch64" ;;
        armv7*|armv6*|armhf|armv5*|arm|arm32) _arch="armv7" ;;
        i386|i486|i586|i686|x86) _arch="i686" ;;
        riscv64*) _arch="riscv64" ;;
        riscv32*) _arch="riscv32" ;;
    esac

    # If architecture is still unknown, check bitness and CPU hints
    if [ "$_arch" = "unknown" ]; then
        case "$_raw_arch" in
            *arm*) _arch="armv7" ;;
            *)
                if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                    _arch="i686"
                else
                    _arch="x86_64"
                fi
                ;;
        esac
    fi

    printf '%s-%s\n' "$_os" "$_arch"
}

# -----------------------------------------------------------------------------
# Helper: Internal HTTP/FTP Downloader
# Hierarchy: sv -> svyaz -> curl -> wget -> busybox -> nc/telnet/ftp (1960 style)
# -----------------------------------------------------------------------------
_cccp_download_raw() {
    _url="$1"
    _dest="$2"

    [ -z "$_url" ] || [ -z "$_dest" ] && return 1

    # Tier 1: Modern toolchain
    if command -v sv >/dev/null 2>&1; then
        sv -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v svyaz >/dev/null 2>&1; then
        svyaz -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v curl >/dev/null 2>&1; then
        curl -fsSL "$_url" -o "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$_url" -O "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    elif command -v busybox >/dev/null 2>&1 && busybox wget --help >/dev/null 2>&1; then
        busybox wget -q "$_url" -O "$_dest" 2>/dev/null && [ -s "$_dest" ] && return 0
    fi

    # Tier 2: 1960/1970 retro fallbacks for constrained routers (OpenWrt without SSL)
    case "$_url" in
        ftp://*)
            _raw_ftp="${_url#ftp://}"
            _ftp_host="${_raw_ftp%%/*}"
            _ftp_path="/${_raw_ftp#*/}"

            if command -v ftpget >/dev/null 2>&1; then
                ftpget "$_ftp_host" "$_dest" "$_ftp_path" 2>/dev/null && [ -s "$_dest" ] && return 0
            elif command -v busybox >/dev/null 2>&1 && busybox ftpget --help >/dev/null 2>&1; then
                busybox ftpget "$_ftp_host" "$_dest" "$_ftp_path" 2>/dev/null && [ -s "$_dest" ] && return 0
            elif command -v ftp >/dev/null 2>&1; then
                printf "open %s\nuser anonymous cccp@gosplan.su\nbinary\nget %s %s\nquit\n" "$_ftp_host" "$_ftp_path" "$_dest" | ftp -n 2>/dev/null
                [ -s "$_dest" ] && return 0
            fi
            ;;

        http://*)
            _raw_http="${_url#http://}"
            _http_host_port="${_raw_http%%/*}"
            _http_path="/${_raw_http#*/}"
            case "$_http_host_port" in
                *:*)
                    _http_host="${_http_host_port%%:*}"
                    _http_port="${_http_host_port##*:}"
                    ;;
                *)
                    _http_host="$_http_host_port"
                    _http_port="80"
                    ;;
            esac

            # Netcat (nc / busybox nc) HTTP/1.0
            if command -v nc >/dev/null 2>&1; then
                if printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" | \
                    nc "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            elif command -v busybox >/dev/null 2>&1 && busybox nc --help >/dev/null 2>&1; then
                if printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" | \
                    busybox nc "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            fi

            # Telnet (1960s style TCP handshake)
            if command -v telnet >/dev/null 2>&1; then
                if (printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host"; sleep 2) | \
                    telnet "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            elif command -v busybox >/dev/null 2>&1 && busybox telnet --help >/dev/null 2>&1; then
                if (printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host"; sleep 2) | \
                    busybox telnet "$_http_host" "$_http_port" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    [ -s "$_dest" ] && return 0
                fi
                rm -f "$_dest"
            fi

            # Raw TCP socket via shell builtin /dev/tcp (if supported)
            if (printf '' > "/dev/tcp/${_http_host}/${_http_port}") 2>/dev/null; then
                (
                    exec 3<>/dev/tcp/"${_http_host}"/"${_http_port}"
                    printf "GET %s HTTP/1.0\r\nHost: %s\r\nUser-Agent: CCCP-1960\r\nConnection: close\r\n\r\n" "$_http_path" "$_http_host" >&3
                    sed '1,/^[\r]*$/d' <&3 > "$_dest"
                    exec 3>&-
                ) 2>/dev/null
                [ -s "$_dest" ] && return 0
                rm -f "$_dest"
            fi
            ;;

        https://*)
            # Low-level HTTPS bootstrap via OpenSSL s_client
            if command -v openssl >/dev/null 2>&1; then
                _host=$(printf '%s' "$_url" | sed -e 's|^https://||' -e 's|/.*$||')
                _path="/$(printf '%s' "$_url" | sed -e 's|^https://[^/]*/*||')"
                if printf "GET %s HTTP/1.1\r\nHost: %s\r\nUser-Agent: CCCP-Bootstrap/1.0\r\nConnection: close\r\n\r\n" "$_path" "$_host" | \
                    openssl s_client -quiet -connect "${_host}:443" -servername "$_host" 2>/dev/null | \
                    sed '1,/^[\r]*$/d' > "$_dest"; then
                    if [ -s "$_dest" ]; then
                        return 0
                    fi
                fi
                rm -f "$_dest"
            fi
            ;;
    esac

    return 1
}

# -----------------------------------------------------------------------------
# Helper: Ensure Comrade is Installed (Auto-Recruit from Nightly Releases)
# Arguments: <comrade_name: kv|kvant|sv|svyaz> [silent: 0|1]
# -----------------------------------------------------------------------------
_cccp_ensure_comrade() {
    _name="${1:-}"
    _silent="${2:-0}"

    [ -z "$_name" ] && return 1

    # Check if target binary is already in PATH
    _target_bin="$_name"
    case "$_name" in
        kv|kvant) _target_bin="kv" ;;
        sv|svyaz) _target_bin="sv" ;;
        mk|mayak) _target_bin="mk" ;;
    esac

    if command -v "$_target_bin" >/dev/null 2>&1; then
        return 0
    fi

    _bdir=$(_cccp_get_bin_dir)
    if [ -x "${_bdir}/${_target_bin}" ]; then
        case ":${PATH:-}:" in
            *":${_bdir}:"*) ;;
            *) export PATH="${_bdir}:${PATH:-}" ;;
        esac
        return 0
    fi

    # Map comrade to repo and archive name
    _repo=""
    _prefix=""
    case "$_name" in
        kv|kvant)
            _repo="kvant"
            _prefix="kv"
            ;;
        sv|svyaz)
            _repo="svyaz"
            _prefix="sv"
            ;;
        mk|mayak)
            _repo="mayak"
            _prefix="mk"
            ;;
        *)
            [ "$_silent" -eq 0 ] && echo "Erro: Camarada desconhecido '$_name'." >&2
            return 1
            ;;
    esac

    _arch=$(_cccp_detect_arch)
    _asset="${_prefix}-${_arch}.tar.gz"

    # Default fallback if unknown architecture
    case "$_arch" in
        unknown*|*-unknown)
            case "${_raw_arch:-}" in
                *arm*) _asset="${_prefix}-linux-armv7.tar.gz" ;;
                *)
                    if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                        _asset="${_prefix}-linux-i686.tar.gz"
                    else
                        _asset="${_prefix}-linux-x86_64.tar.gz"
                    fi
                    ;;
            esac
            ;;
    esac

    # Allow custom mirror via CCCP_COMRADES_MIRROR or comrades_mirror config
    _mirror="${CCCP_COMRADES_MIRROR:-${COMRADES_MIRROR:-}}"
    if [ -z "$_mirror" ] && command -v cmd_config >/dev/null 2>&1; then
        _mirror=$(cmd_config comrades_mirror 2>/dev/null || true)
    fi

    _primary_url="https://codeberg.org/lumenpink/${_repo}/releases/download/nightly/${_asset}"

    _urls=""
    if [ -n "$_mirror" ]; then
        _urls="${_mirror%/}/${_asset} ${_primary_url}"
    else
        _urls="${_primary_url}"
    fi

    if [ "$_silent" -eq 0 ]; then
        echo "★ [ГОСПЛАН] Convocando camarada '${_name}' (${_asset})... ★"
    fi

    _tmp_tar=$(mktemp 2>/dev/null || printf '/tmp/cccp_comrade_%s_%s.tar.gz' "$_name" "$$")
    _download_ok=0
    for _try_url in $_urls; do
        if _cccp_download_raw "$_try_url" "$_tmp_tar"; then
            _download_ok=1
            break
        fi
    done

    if [ "$_download_ok" -eq 0 ]; then
        rm -f "$_tmp_tar"
        if [ "$_silent" -eq 0 ]; then
            echo "Aviso: Falha ao baixar camarada '${_name}' das fontes disponíveis." >&2
            echo "Dica: Em sistemas minimalistas sem SSL (OpenWrt/BusyBox), configure um mirror HTTP ou FTP:" >&2
            echo "      export CCCP_COMRADES_MIRROR=\"http://seu-servidor/cccp\"" >&2
        fi
        return 1
    fi

    if ! tar -xzf "$_tmp_tar" -C "$_bdir" 2>/dev/null; then
        rm -f "$_tmp_tar"
        if [ "$_silent" -eq 0 ]; then
            echo "Aviso: Falha ao descompactar ${_asset} em ${_bdir}." >&2
        fi
        return 1
    fi

    rm -f "$_tmp_tar"
    chmod +x "${_bdir}"/* 2>/dev/null || true

    if [ "$_silent" -eq 0 ]; then
        echo "★ [ГОСПЛАН] Camarada '${_name}' recrutado e instalado com sucesso em ${_bdir} ★"
    fi

    return 0
}

# -----------------------------------------------------------------------------
# Unified HTTP Dispatcher for CCCP (sv -> svyaz -> curl -> wget)
# Automatically recruits sv if missing and possible.
# -----------------------------------------------------------------------------
_cccp_http() {
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

    if command -v sv >/dev/null 2>&1; then
        sv "$@"
        return $?
    fi

    if command -v svyaz >/dev/null 2>&1; then
        svyaz "$@"
        return $?
    fi

    # Check local bin dir
    _bdir=$(_cccp_get_bin_dir)
    if [ -x "${_bdir}/sv" ]; then
        "${_bdir}/sv" "$@"
        return $?
    fi

    # Attempt silent bootstrap of sv
    if _cccp_ensure_comrade "sv" 1 >/dev/null 2>&1; then
        if command -v sv >/dev/null 2>&1; then
            sv "$@"
            return $?
        fi
        if [ -x "${_bdir}/sv" ]; then
            "${_bdir}/sv" "$@"
            return $?
        fi
    fi

    # Fallback to curl
    if command -v curl >/dev/null 2>&1; then
        curl "$@"
        return $?
    fi

    echo "Error: Nenhum cliente HTTP disponível ('sv', 'svyaz' ou 'curl' necessários)." >&2
    return 127
}

# -----------------------------------------------------------------------------
# Subcommand: cccp comrades [status|install|update|help]
# -----------------------------------------------------------------------------
cmd_comrades() {
    _sub="${1:-status}"

    case "$_sub" in
        status)
            echo "★ [ГОСПЛАН] Registro Central dos Camaradas do Ecossistema ★"
            echo "==========================================================="
            _bdir=$(_cccp_get_bin_dir)
            echo "Diretório de Binários: ${_bdir}"
            echo ""
            printf "%-10s %-12s %-16s %s\n" "CAMARADA" "PAPEL" "ESTADO" "LOCALIZAÇÃO"
            printf "%-10s %-12s %-16s %s\n" "--------" "-----" "------" "-----------"

            for _c in "kv" "sv" "mk"; do
                _role=""
                case "$_c" in
                    kv) _role="Editor (kvant)" ;;
                    sv) _role="HTTP (svyaz)" ;;
                    mk) _role="Static (mayak)" ;;
                esac

                _p=$(command -v "$_c" 2>/dev/null || true)
                if [ -n "$_p" ]; then
                    _ver=$("$_c" --version 2>/dev/null | head -n 1 || echo "presente")
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "PRONTO ($_ver)" "$_p"
                elif [ -x "${_bdir}/${_c}" ]; then
                    _ver=$("${_bdir}/${_c}" --version 2>/dev/null | head -n 1 || echo "presente")
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "PRONTO ($_ver)" "${_bdir}/${_c}"
                else
                    printf "%-10s %-12s %-16s %s\n" "$_c" "$_role" "NÃO INSTALADO" "(disponível via 'cccp comrades install')"
                fi
            done
            echo ""
            ;;

        install|recrutar)
            _target="${2:-all}"
            echo "★ [ГОСПЛАН] Recrutando camaradas para o posto de trabalho... ★"
            case "$_target" in
                kv|kvant)
                    _cccp_ensure_comrade "kv" 0
                    ;;
                sv|svyaz)
                    _cccp_ensure_comrade "sv" 0
                    ;;
                mk|mayak)
                    _cccp_ensure_comrade "mk" 0
                    ;;
                all|*)
                    _cccp_ensure_comrade "kv" 0
                    _cccp_ensure_comrade "sv" 0
                    _cccp_ensure_comrade "mk" 0
                    ;;
            esac
            ;;

        update|atualizar)
            echo "★ [ГОСПЛАН] Atualizando camaradas para as últimas compilações nightly... ★"
            _bdir=$(_cccp_get_bin_dir)
            _arch=$(_cccp_detect_arch)
            case "$_arch" in
                unknown*|*-unknown)
                    case "${_raw_arch:-}" in
                        *arm*) _arch="linux-armv7" ;;
                        *)
                            if [ "$(getconf LONG_BIT 2>/dev/null || echo 64)" = "32" ]; then
                                _arch="linux-i686"
                            else
                                _arch="linux-x86_64"
                            fi
                            ;;
                    esac
                    ;;
            esac

            _mirror="${CCCP_COMRADES_MIRROR:-${COMRADES_MIRROR:-}}"
            if [ -z "$_mirror" ] && command -v cmd_config >/dev/null 2>&1; then
                _mirror=$(cmd_config comrades_mirror 2>/dev/null || true)
            fi

            for _pair in "kv:kvant" "sv:svyaz" "mk:mayak"; do
                _bin=$(printf '%s' "$_pair" | cut -d: -f1)
                _repo=$(printf '%s' "$_pair" | cut -d: -f2)
                _asset="${_bin}-${_arch}.tar.gz"
                _primary_url="https://codeberg.org/lumenpink/${_repo}/releases/download/nightly/${_asset}"

                _urls=""
                if [ -n "$_mirror" ]; then
                    _urls="${_mirror%/}/${_asset} ${_primary_url}"
                else
                    _urls="${_primary_url}"
                fi

                echo "  -> Baixando ${_bin} (${_asset})..."
                _tmp_tar=$(mktemp 2>/dev/null || printf '/tmp/cccp_up_%s.tar.gz' "$_bin")
                _up_ok=0
                for _try_url in $_urls; do
                    if _cccp_download_raw "$_try_url" "$_tmp_tar"; then
                        _up_ok=1
                        break
                    fi
                done

                if [ "$_up_ok" -eq 1 ]; then
                    tar -xzf "$_tmp_tar" -C "$_bdir" 2>/dev/null || true
                    rm -f "$_tmp_tar"
                    chmod +x "${_bdir}/${_bin}"* 2>/dev/null || true
                    echo "  [OK] ${_bin} atualizado em ${_bdir}."
                else
                    rm -f "$_tmp_tar"
                    echo "  [ERRO] Falha ao atualizar ${_bin}." >&2
                fi
            done
            echo "★ Atualização concluída com sucesso! ★"
            ;;

        help|--help|-h)
            echo "★ CCCP - Gestão de Camaradas (Kvant e Svyaz) ★"
            echo ""
            echo "Uso:"
            echo "  cccp comrades [status]          Exibe o status dos binários kv e sv"
            echo "  cccp comrades install [all|kv|sv] Baixa e instala os comrades no sistema"
            echo "  cccp comrades update            Atualiza kv e sv para o último nightly"
            echo ""
            ;;

        *)
            echo "Uso: cccp comrades [status|install|update|help]" >&2
            return 1
            ;;
    esac
    return 0
}
