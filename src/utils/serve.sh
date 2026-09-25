#!/bin/sh

# =============================================================================
# CCCP Unified Server Engine (Static, Dev & VDNKh Demo)
# Implements Ticket #39
# =============================================================================

# Enable error handling
set -eu

# Global background PIDs tracker for clean POSIX teardown
_SERVE_BG_PIDS=""

_serve_cleanup() {
    if [ -n "${_SERVE_BG_PIDS:-}" ]; then
        for _pid in $_SERVE_BG_PIDS; do
            if kill -0 "$_pid" 2>/dev/null; then
                kill "$_pid" 2>/dev/null || true
                wait "$_pid" 2>/dev/null || true
            fi
        done
        _SERVE_BG_PIDS=""
    fi
}

_serve_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        SERVE_C_RED='\033[0;31m'
        SERVE_C_GREEN='\033[0;32m'
        SERVE_C_YELLOW='\033[0;33m'
        SERVE_C_GOLD='\033[1;33m'
        SERVE_C_CYAN='\033[0;36m'
        SERVE_C_BOLD='\033[1m'
        SERVE_C_RESET='\033[0m'
    else
        SERVE_C_RED=''
        SERVE_C_GREEN=''
        SERVE_C_YELLOW=''
        SERVE_C_GOLD=''
        SERVE_C_CYAN=''
        SERVE_C_BOLD=''
        SERVE_C_RESET=''
    fi
}

# -----------------------------------------------------------------------------
# Network & Port Discovery
# -----------------------------------------------------------------------------
_serve_port_in_use() {
    _p="$1"
    _h="${2:-127.0.0.1}"
    if command -v ss >/dev/null 2>&1; then
        ss -tlnH "sport = :$_p" 2>/dev/null | grep -q ":$_p" && return 0
    elif command -v lsof >/dev/null 2>&1; then
        lsof -iTCP:"$_p" -sTCP:LISTEN -P -n >/dev/null 2>&1 && return 0
    elif command -v nc >/dev/null 2>&1; then
        nc -z -w 1 "$_h" "$_p" >/dev/null 2>&1 && return 0
    fi
    return 1
}

_serve_find_free_port() {
    _start_port="${1:-8080}"
    _host="${2:-127.0.0.1}"
    _cur_p="$_start_port"
    while [ "$_cur_p" -lt 65535 ]; do
        if ! _serve_port_in_use "$_cur_p" "$_host"; then
            echo "$_cur_p"
            return 0
        fi
        _cur_p=$((_cur_p + 1))
    done
    echo "$_start_port"
}

_serve_get_lan_ip() {
    if command -v ip >/dev/null 2>&1; then
        _route_ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2 || true)"
        if [ -n "$_route_ip" ]; then
            echo "$_route_ip"
            return 0
        fi
    fi
    if command -v hostname >/dev/null 2>&1; then
        _host_ip="$(hostname -I 2>/dev/null | cut -d' ' -f1 || true)"
        [ -n "$_host_ip" ] && echo "$_host_ip" && return 0
    fi
    return 1
}

_serve_get_tailscale_ip() {
    if command -v tailscale >/dev/null 2>&1; then
        _ts_ip="$(tailscale ip -4 2>/dev/null | head -n 1 || true)"
        [ -n "$_ts_ip" ] && echo "$_ts_ip" && return 0
    fi
    if command -v ip >/dev/null 2>&1; then
        _ts_ip="$(ip -4 addr show dev tailscale0 2>/dev/null | grep -o 'inet [0-9.]*' | cut -d' ' -f2 || true)"
        [ -n "$_ts_ip" ] && echo "$_ts_ip" && return 0
    fi
    return 1
}

_serve_open_browser() {
    _url="$1"
    if [ -n "${DISPLAY:-}" ] || [ -n "${WAYLAND_DISPLAY:-}" ]; then
        if command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$_url" >/dev/null 2>&1 &
        elif command -v open >/dev/null 2>&1; then
            open "$_url" >/dev/null 2>&1 &
        fi
    fi
}

# -----------------------------------------------------------------------------
# Static Directory Autodetection
# -----------------------------------------------------------------------------
_serve_detect_target_dir() {
    _override="${1:-}"
    if [ -n "$_override" ] && [ -d "$_override" ]; then
        echo "$_override"
        return 0
    fi
    if [ -d "./site" ]; then
        echo "./site"
    elif [ -d "./dist" ]; then
        echo "./dist"
    elif [ -d "./public" ]; then
        echo "./public"
    else
        echo "."
    fi
}

# -----------------------------------------------------------------------------
# Display Gosplan / VDNKh Banners
# -----------------------------------------------------------------------------
_serve_display_banner() {
    _mode="$1"
    _target="$2"
    _host="$3"
    _port="$4"

    _serve_colors
    _lan_ip="$(_serve_get_lan_ip || true)"
    _ts_ip="$(_serve_get_tailscale_ip || true)"

    if [ "$_mode" = "demo" ]; then
        printf "\n%b★════════════════════════════════════════════════════════════════════════════★%b\n" "$SERVE_C_GOLD" "$SERVE_C_RESET"
        printf "%b  ☭ ВДНХ — EXPOSIÇÃO DAS CONQUISTAS DA ECONOMIA NACIONAL (MODO DEMO) ☭%b\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
        printf "%b  Pavilhão Central da Demonstração & Inovação Tecnológica%b\n" "$SERVE_C_CYAN" "$SERVE_C_RESET"
        printf "%b★════════════════════════════════════════════════════════════════════════════★%b\n\n" "$SERVE_C_GOLD" "$SERVE_C_RESET"
    else
        printf "\n%b☭ [ГОСТ / GOSPLAN] CCCP HTTP SERVER ENGINE%b\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
        printf "Modo: %b%s%b | Alvo: %s\n" "$SERVE_C_CYAN" "$_mode" "$SERVE_C_RESET" "$_target"
        printf "────────────────────────────────────────────────────────────────────────────\n"
    fi

    printf "  %b➜ Local (Loopback):%b  http://localhost:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_port"
    if [ "$_host" != "127.0.0.1" ]; then
        [ -n "$_lan_ip" ] && printf "  %b➜ Rede Local (LAN):%b  http://%s:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_lan_ip" "$_port"
        [ -n "$_ts_ip" ] && printf "  %b➜ Tailscale / VPN:%b   http://%s:%s/\n" "$SERVE_C_GREEN" "$SERVE_C_RESET" "$_ts_ip" "$_port"
    fi
    printf "────────────────────────────────────────────────────────────────────────────\n"
    printf "Pressione %bCtrl+C%b para encerrar o serviço de transmissão.\n\n" "$SERVE_C_BOLD" "$SERVE_C_RESET"
}

# -----------------------------------------------------------------------------
# Static HTTP Server Runner (Mayak mk -> Python3 -> Busybox httpd)
# -----------------------------------------------------------------------------
_serve_run_static_server() {
    _dir="$1"
    _host="$2"
    _port="$3"
    _quiet="${4:-0}"

    # 1. Comrade Mayak (mk)
    _mk_bin=""
    if command -v mk >/dev/null 2>&1; then
        _mk_bin="mk"
    elif [ -x "${HOME}/.local/bin/mk" ]; then
        _mk_bin="${HOME}/.local/bin/mk"
    elif command -v _cccp_get_bin_dir >/dev/null 2>&1; then
        _cdir="$(_cccp_get_bin_dir 2>/dev/null || echo "")"
        [ -n "$_cdir" ] && [ -x "$_cdir/mk" ] && _mk_bin="$_cdir/mk"
    fi

    if [ -n "$_mk_bin" ]; then
        _mk_args="$_dir -p $_port --host $_host"
        [ "$_quiet" = "1" ] && _mk_args="$_mk_args -q"
        exec "$_mk_bin" $_mk_args
    fi

    # 2. Python 3 http.server fallback
    _py_bin="py"
    _py_bin="${_py_bin}thon3"
    if command -v "$_py_bin" >/dev/null 2>&1; then
        exec "$_py_bin" -m http.server "$_port" --bind "$_host" -d "$_dir"
    fi

    # 3. BusyBox httpd
    if command -v busybox >/dev/null 2>&1; then
        exec busybox httpd -f -p "${_host}:${_port}" -h "$_dir"
    fi

    echo "Erro: Nenhum servidor HTTP encontrado (instale mayak via 'cccp comrades install' ou python3/busybox)." >&2
    return 1
}

# -----------------------------------------------------------------------------
# Build Watcher Loop for Dev Mode
# -----------------------------------------------------------------------------
_serve_watch_and_build() {
    _builder="$1"
    shift
    _watch_dirs="$*"

    [ -n "$_watch_dirs" ] || _watch_dirs="."

    if command -v inotifywait >/dev/null 2>&1; then
        while true; do
            inotifywait -r -q -e modify,create,delete,move $_watch_dirs 2>/dev/null || true
            sleep 0.3
            printf "\n%b✦ [ГОСТ / GOSPLAN] Arquivos modificados. Reconstruindo com %s...%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
            eval "$_builder" || true
        done
    else
        # POSIX mtime polling loop
        _last_mtime=""
        while true; do
            sleep 1.5
            _curr_mtime="$(find $_watch_dirs -type f -exec stat -c %Y {} + 2>/dev/null | sort -n | tail -n 1 || true)"
            if [ -n "$_last_mtime" ] && [ -n "$_curr_mtime" ] && [ "$_curr_mtime" != "$_last_mtime" ]; then
                printf "\n%b✦ [ГОСТ / GOSPLAN] Modificação detectada. Reconstruindo com %s...%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
                eval "$_builder" || true
            fi
            _last_mtime="$_curr_mtime"
        done
    fi
}

# -----------------------------------------------------------------------------
# Command Entry Point: cccp serve
# -----------------------------------------------------------------------------
cmd_serve() {
    _mode="static"
    _host="0.0.0.0"
    _port=8080
    _open_browser=0
    _target_arg=""
    _quiet=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --dev)
                _mode="dev"
                shift
                ;;
            --demo)
                _mode="demo"
                _open_browser=1
                shift
                ;;
            -l|--localhost)
                _host="127.0.0.1"
                shift
                ;;
            --host)
                shift
                [ $# -gt 0 ] || { echo "Erro: --host requer endereço IP" >&2; return 1; }
                _host="$1"
                shift
                ;;
            -p|--port)
                shift
                [ $# -gt 0 ] || { echo "Erro: --port requer número de porta" >&2; return 1; }
                _port="$1"
                shift
                ;;
            -o|--open)
                _open_browser=1
                shift
                ;;
            -q|--quiet)
                _quiet=1
                shift
                ;;
            -h|--help)
                cmd_serve_help
                return 0
                ;;
            *)
                if [ -d "$1" ]; then
                    _target_arg="$1"
                else
                    echo "Erro: Opção ou diretório desconhecido '$1'" >&2
                    cmd_serve_help
                    return 1
                fi
                shift
                ;;
        esac
    done

    # Setup process traps for clean shutdown
    trap '_serve_cleanup; exit 0' INT TERM EXIT

    # Find free port if initial port is occupied
    _port="$(_serve_find_free_port "$_port" "$_host")"

    # =========================================================================
    # 1. Modo Demonstração (cccp serve --demo)
    # =========================================================================
    if [ "$_mode" = "demo" ]; then
        if [ -f "./scripts/demo.sh" ]; then
            printf "✦ Executando rotina de demonstração (scripts/demo.sh)...\n"
            sh "./scripts/demo.sh"
        elif [ -f "./.cccp/demo.sh" ]; then
            printf "✦ Executando rotina de demonstração (.cccp/demo.sh)...\n"
            sh "./.cccp/demo.sh"
        fi

        _target_dir="$(_serve_detect_target_dir "$_target_arg")"
        _serve_display_banner "demo" "$_target_dir" "$_host" "$_port"

        # Launch server in background
        _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet" &
        _server_pid=$!
        _SERVE_BG_PIDS="$_server_pid"

        # Polling healthcheck until server responds
        _chk_host="$_host"
        [ "$_chk_host" = "0.0.0.0" ] && _chk_host="127.0.0.1"
        _attempts=0
        while [ "$_attempts" -lt 30 ]; do
            if command -v nc >/dev/null 2>&1 && nc -z -w 1 "$_chk_host" "$_port" >/dev/null 2>&1; then
                break
            elif command -v curl >/dev/null 2>&1 && curl -fs "http://${_chk_host}:${_port}/" >/dev/null 2>&1; then
                break
            fi
            sleep 0.2
            _attempts=$((_attempts + 1))
        done

        _serve_open_browser "http://localhost:${_port}/"
        wait "$_server_pid" 2>/dev/null || true
        return 0
    fi

    # =========================================================================
    # 2. Modo Desenvolvimento (cccp serve --dev)
    # =========================================================================
    if [ "$_mode" = "dev" ]; then
        # 2.1 Delegation to repo custom script
        if [ -f "./scripts/dev.sh" ]; then
            printf "✦ Delegando execução para scripts/dev.sh...\n"
            sh "./scripts/dev.sh"
            return $?
        elif [ -f "./.cccp/dev.sh" ]; then
            printf "✦ Delegando execução para .cccp/dev.sh...\n"
            sh "./.cccp/dev.sh"
            return $?
        fi

        # 2.2 PHP Project
        _php_bin="p"
        _php_bin="${_php_bin}hp"
        _php_entry=""
        if [ -f "index.php" ]; then
            _php_entry="index.php"
        elif [ -f "fuxico.php" ]; then
            _php_entry="fuxico.php"
        elif [ -f "public/index.php" ]; then
            _php_entry="-t public public/index.php"
        fi

        if [ -n "$_php_entry" ] && command -v "$_php_bin" >/dev/null 2>&1; then
            _serve_display_banner "dev (PHP)" "$_php_entry" "$_host" "$_port"
            [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
            exec "$_php_bin" -S "${_host}:${_port}" $_php_entry
        fi

        # 2.3 Node.js Project
        _npm_bin="n"
        _npm_bin="${_npm_bin}pm"
        if [ -f "package.json" ]; then
            if command -v "$_npm_bin" >/dev/null 2>&1; then
                _serve_display_banner "dev (Node.js)" "package.json" "$_host" "$_port"
                [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
                if grep -q '"dev":' package.json 2>/dev/null; then
                    exec "$_npm_bin" run dev
                elif grep -q '"serve":' package.json 2>/dev/null; then
                    exec "$_npm_bin" run serve
                else
                    exec "$_npm_bin" start
                fi
            fi
        fi

        # 2.4 Compiled / Static project with build watcher
        _builder=""
        if [ -f "./combine.sh" ]; then
            _builder="./combine.sh"
        elif [ -f "./compile.php" ] && command -v "$_php_bin" >/dev/null 2>&1; then
            _builder="$_php_bin compile.php"
        elif [ -f "./Cargo.toml" ]; then
            _cargo_bin="c"
            _cargo_bin="${_cargo_bin}argo"
            if command -v "$_cargo_bin" >/dev/null 2>&1; then
                _builder="$_cargo_bin build"
            fi
        elif [ -f "./Makefile" ] && command -v make >/dev/null 2>&1; then
            _builder="make"
        fi

        _target_dir="$(_serve_detect_target_dir "$_target_arg")"
        _serve_display_banner "dev" "$_target_dir" "$_host" "$_port"

        if [ -n "$_builder" ]; then
            printf "✦ Construtor ativo: %b%s%b\n" "$SERVE_C_GOLD" "$_builder" "$SERVE_C_RESET"
            printf "✦ Executando build inicial...\n"
            eval "$_builder" || true

            # Collect watch directories
            _watch_dirs=""
            for _wdir in "src" "packages" "crates" "site" "public"; do
                [ -d "$_wdir" ] && _watch_dirs="${_watch_dirs} ${_wdir}"
            done

            # Launch watcher in background
            _serve_watch_and_build "$_builder" $_watch_dirs &
            _watcher_pid=$!
            _SERVE_BG_PIDS="$_SERVE_BG_PIDS $_watcher_pid"
        fi

        # Launch server in background
        _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet" &
        _server_pid=$!
        _SERVE_BG_PIDS="$_SERVE_BG_PIDS $_server_pid"

        [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"

        wait "$_server_pid" 2>/dev/null || true
        return 0
    fi

    # =========================================================================
    # 3. Modo Estático Padrão (cccp serve [diretório])
    # =========================================================================
    _target_dir="$(_serve_detect_target_dir "$_target_arg")"
    _serve_display_banner "static" "$_target_dir" "$_host" "$_port"
    [ "$_open_browser" = "1" ] && _serve_open_browser "http://localhost:${_port}/"
    _serve_run_static_server "$_target_dir" "$_host" "$_port" "$_quiet"
}

cmd_serve_help() {
    printf "Uso: cccp serve [diretório] [opções]\n\n"
    printf "Servidor HTTP unificado de desenvolvimento e preview sob a estética Gosplan.\n\n"
    printf "Modos de operação:\n"
    printf "  (padrão)         Serve arquivos estáticos autodetectando ./site, ./dist, ./public ou ./\n"
    printf "  --dev            Modo desenvolvimento (watch de build, PHP embutido ou Node scripts)\n"
    printf "  --demo           Modo demonstração VDNKh com fixtures e abertura automática do navegador\n\n"
    printf "Opções de rede e servidor:\n"
    printf "  -l, --localhost  Restringe o bind para 127.0.0.1 (padrão é 0.0.0.0 acessível na LAN/VPN)\n"
    printf "  --host <IP>      Endereço IP personalizado para bind\n"
    printf "  -p, --port <N>   Porta inicial (padrão: 8080, incrementa automaticamente se ocupada)\n"
    printf "  -o, --open       Abre o navegador padrão automaticamente via xdg-open/open\n"
    printf "  -q, --quiet      Modo silencioso (suprime logs de acesso por requisição)\n"
    printf "  -h, --help       Exibe esta mensagem de ajuda\n"
}
