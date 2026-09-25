#!/bin/sh

# =============================================================================
# Web Interface & Mayak Command Bridge
# =============================================================================

# Generate a cryptographic session token
_web_generate_token() {
    if [ -r /dev/urandom ]; then
        od -vN 16 -An -tx1 /dev/urandom 2>/dev/null | tr -d ' \t\r\n'
    else
        printf '%s%s%s\n' "$(date +%s)" "$$" "$RANDOM" | tr -cd '0-9a-zA-Z'
    fi
}

# Export embedded single-page Gosplan Web UI assets
_web_export_assets() {
    _out_dir="$1"
    if [ $# -ge 3 ]; then
        _listen_port="$3"
    elif [ $# -ge 2 ]; then
        _listen_port="$2"
    else
        _listen_port="8080"
    fi

    mkdir -p "$_out_dir"
    chmod 700 "$_out_dir" 2>/dev/null || true

    cat > "$_out_dir/index.html" << EOF
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CCCP Web Cockpit — Радиостанция «Маяк»</title>
  <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>☭</text></svg>">
  <style>
    :root {
      --bg: #0d1117;
      --surface: #161b22;
      --surface-hover: #1f242c;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --text-bright: #f0f6fc;
      --red: #da3633;
      --red-hover: #b62324;
      --gold: #f1e05a;
      --green: #238636;
      --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      line-height: 1.5;
      padding-bottom: 4rem;
    }
    header {
      background: var(--surface);
      border-bottom: 1px solid var(--border);
      padding: 1rem 1.5rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      font-weight: 700;
      font-size: 1.25rem;
      color: var(--text-bright);
    }
    .brand span.badge {
      background: var(--red);
      color: #fff;
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      letter-spacing: 0.5px;
    }
    .session-info {
      font-family: var(--font-mono);
      font-size: 0.8rem;
      color: var(--text-muted);
      background: var(--bg);
      padding: 0.35rem 0.75rem;
      border: 1px solid var(--border);
      border-radius: 6px;
    }
    nav {
      background: #11151c;
      border-bottom: 1px solid var(--border);
      display: flex;
      padding: 0 1.5rem;
      gap: 0.5rem;
    }
    nav button {
      background: none;
      border: none;
      color: var(--text-muted);
      font-size: 0.95rem;
      font-weight: 600;
      padding: 0.85rem 1.25rem;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      transition: all 0.2s;
    }
    nav button:hover {
      color: var(--text-bright);
    }
    nav button.active {
      color: var(--gold);
      border-bottom-color: var(--gold);
    }
    main {
      max-width: 1000px;
      margin: 2rem auto;
      padding: 0 1.5rem;
    }
    .tab-content { display: none; }
    .tab-content.active { display: block; }
    .card {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.5rem;
      margin-bottom: 1.5rem;
    }
    h2 {
      font-size: 1.3rem;
      color: var(--text-bright);
      margin-bottom: 0.5rem;
    }
    p.desc {
      color: var(--text-muted);
      font-size: 0.9rem;
      margin-bottom: 1.25rem;
    }
    textarea, input[type="text"], input[type="password"], select {
      width: 100%;
      background: var(--bg);
      border: 1px solid var(--border);
      color: var(--text-bright);
      border-radius: 6px;
      padding: 0.75rem;
      font-size: 0.95rem;
      font-family: inherit;
      margin-bottom: 1rem;
    }
    textarea:focus, input[type="text"]:focus, input[type="password"]:focus, select:focus {
      outline: none;
      border-color: var(--gold);
    }
    .btn {
      background: var(--red);
      color: #fff;
      border: none;
      padding: 0.65rem 1.25rem;
      border-radius: 6px;
      font-weight: 600;
      font-size: 0.95rem;
      cursor: pointer;
      transition: background 0.2s;
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
    }
    .btn:hover { background: var(--red-hover); }
    .btn-secondary {
      background: var(--surface-hover);
      color: var(--text);
      border: 1px solid var(--border);
    }
    .btn-secondary:hover {
      background: #282e38;
      color: var(--text-bright);
    }
    .spinner {
      display: inline-block;
      width: 1rem;
      height: 1rem;
      border: 2px solid rgba(255,255,255,0.3);
      border-radius: 50%;
      border-top-color: #fff;
      animation: spin 0.8s linear infinite;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    pre.terminal {
      background: #05080c;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 1rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
      color: #3fb950;
      overflow-x: auto;
      white-space: pre-wrap;
      word-break: break-all;
    }
    .ticket-card {
      background: var(--bg);
      border: 1px solid var(--border);
      border-left: 4px solid var(--gold);
      border-radius: 6px;
      padding: 1rem;
      margin-bottom: 1rem;
    }
    .ticket-header {
      display: flex;
      justify-content: space-between;
      margin-bottom: 0.5rem;
    }
    .badge-type {
      font-family: var(--font-mono);
      font-size: 0.75rem;
      font-weight: bold;
      background: #238636;
      color: #fff;
      padding: 0.15rem 0.5rem;
      border-radius: 4px;
    }
    .branch-hint {
      background: #1c2128;
      border: 1px dashed var(--border);
      border-radius: 6px;
      padding: 0.6rem 0.8rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
      color: var(--gold);
      margin-top: 0.75rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .copy-btn {
      background: none;
      border: 1px solid var(--border);
      color: var(--text-muted);
      border-radius: 4px;
      padding: 0.2rem 0.5rem;
      cursor: pointer;
      font-size: 0.75rem;
    }
    .copy-btn:hover { color: var(--text-bright); border-color: var(--text-bright); }
  </style>
</head>
<body>

  <!-- MODAL DE AUTENTICAÇÃO -->
  <div id="auth-modal" style="display:none; position:fixed; inset:0; background:rgba(13,17,23,0.85); backdrop-filter:blur(4px); z-index:9999; align-items:center; justify-content:center;">
    <div class="card" style="width:100%; max-width:440px; margin:1rem; box-shadow:0 8px 24px rgba(0,0,0,0.6); border-color:var(--border);">
      <div style="text-align:center; margin-bottom:1.5rem;">
        <span style="font-size:2.5rem;">🔐</span>
        <h2 style="margin-top:0.5rem; font-size:1.3rem;">Autenticação Requerida</h2>
        <p class="desc" style="margin-bottom:0;">Informe a chave de API ou token do Gosplan para desbloquear o Cockpit.</p>
      </div>
      <form onsubmit="handleAuthSubmit(event)">
        <input type="password" id="auth-token-input" placeholder="Cole seu token aqui..." autocomplete="off" required style="font-family:var(--font-mono); letter-spacing:1px; margin-bottom:0.75rem;" />
        <div id="auth-error" style="display:none; color:var(--red); font-size:0.85rem; margin-bottom:0.75rem;"></div>
        <button type="submit" id="btn-auth-submit" class="btn" style="width:100%; justify-content:center;">
          <span>✦ Autenticar</span>
        </button>
      </form>
    </div>
  </div>

  <header>
    <div class="brand">
      <span>☭ CCCP Web Cockpit</span>
      <span class="badge">Маяк Bridge</span>
    </div>
    <div style="display:flex; align-items:center; gap:0.75rem; flex-wrap:wrap;">
      <div style="display:flex; align-items:center; gap:0.4rem;">
        <label for="repo-select" style="font-size:0.85rem; color:var(--text-muted); font-weight:600;">📁 Repositório:</label>
        <select id="repo-select" onchange="switchRepository(this.value)" style="margin-bottom:0; width:auto; min-width:180px; padding:0.35rem 0.6rem; font-size:0.85rem; height:32px;">
          <option value="">Carregando...</option>
        </select>
      </div>
      <div class="session-info">
        Port: <strong>${_listen_port}</strong>
      </div>
      <button id="btn-auth-lock" class="btn btn-secondary" style="padding:0.35rem 0.75rem; font-size:0.8rem; display:none;" onclick="logout()">
        🔒 Bloquear
      </button>
    </div>
  </header>

  <nav>
    <button id="nav-diva" class="active" onclick="switchTab('diva')">🛋️ O Divã (Triage IA)</button>
    <button id="nav-status" onclick="switchTab('status')">📊 Inspeção do Repositório</button>
    <button id="nav-tickets" onclick="switchTab('tickets')">📋 Tickets Ativos</button>
  </nav>

  <main>
    <!-- TAB 1: O DIVÃ -->
    <div id="tab-diva" class="tab-content active">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.5rem; flex-wrap:wrap; gap:0.5rem;">
          <h2>🛋️ O Divã — Triagem Semântica de Tickets</h2>
          <span id="diva-repo-hint" style="font-family:var(--font-mono); font-size:0.85rem; color:var(--gold); font-weight:bold;"></span>
        </div>
        <p class="desc">Descreva livremente o problema, bug ou necessidade. A IA do Gosplan estruturará tickets atômicos seguindo Conventional Commits com sugestão de branch GOST.</p>
        
        <textarea id="diva-input" rows="4" placeholder="Ex: O botão de login congela quando o token JWT expira no meio de uma requisição assíncrona, exigindo reload da página..."></textarea>
        
        <div style="display:flex; gap:0.5rem;">
          <button id="btn-diva-run" class="btn" onclick="runDiva()">
            <span>✦ Consultar Divã (IA)</span>
          </button>
          <button class="btn btn-secondary" onclick="document.getElementById('diva-input').value = ''">Limpar</button>
        </div>
      </div>

      <div id="diva-results"></div>
    </div>

    <!-- TAB 2: STATUS -->
    <div id="tab-status" class="tab-content">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
          <h2>📊 Inspeção Gosplan do Repositório</h2>
          <button class="btn btn-secondary" onclick="loadStatus()">Atualizar Status</button>
        </div>
        <pre id="status-output" class="terminal">Carregando status do repositório...</pre>
      </div>
    </div>

    <!-- TAB 3: TICKETS -->
    <div id="tab-tickets" class="tab-content">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
          <h2>📋 Tickets Ativos do Projeto</h2>
          <button class="btn btn-secondary" onclick="loadTickets()">Sincronizar com Forge</button>
        </div>
        <pre id="tickets-output" class="terminal">Carregando tickets...</pre>
      </div>
    </div>
  </main>

  <script>
    let sessionToken = sessionStorage.getItem('cccp_token') || new URLSearchParams(window.location.search).get('token') || '';
    let activeRepo = sessionStorage.getItem('cccp_active_repo') || '';

    function showAuthModal(errMsg = '') {
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (errMsg) {
        errEl.textContent = errMsg;
        errEl.style.display = 'block';
      } else {
        errEl.style.display = 'none';
      }
      modal.style.display = 'flex';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'none';
      setTimeout(() => {
        const inp = document.getElementById('auth-token-input');
        if (inp) inp.focus();
      }, 50);
    }

    function hideAuthModal() {
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (modal) modal.style.display = 'none';
      if (errEl) errEl.style.display = 'none';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'inline-flex';
    }

    function logout() {
      sessionStorage.removeItem('cccp_token');
      sessionToken = '';
      const url = new URL(window.location);
      if (url.searchParams.has('token')) {
        url.searchParams.delete('token');
        window.history.replaceState({}, '', url.pathname + url.search);
      }
      showAuthModal('Sessão bloqueada.');
    }

    async function handleAuthSubmit(e) {
      e.preventDefault();
      const input = document.getElementById('auth-token-input');
      const val = input.value.trim();
      if (!val) return;

      const btn = document.getElementById('btn-auth-submit');
      btn.disabled = true;
      btn.innerHTML = '<span class="spinner"></span> Validando...';

      try {
        const res = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + val,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });

        if (res.status === 401) {
          showAuthModal('Chave ou token inválido. Tente novamente.');
          input.value = '';
          input.focus();
          return;
        }

        sessionToken = val;
        sessionStorage.setItem('cccp_token', sessionToken);
        hideAuthModal();
        await loadRepositories();
        const activeTab = document.querySelector('nav button.active');
        const tabName = activeTab ? activeTab.id.replace('nav-', '') : 'diva';
        switchTab(tabName);
      } catch (err) {
        showAuthModal('Erro ao conectar ao Mayak: ' + err.message);
      } finally {
        btn.disabled = false;
        btn.innerHTML = '<span>✦ Autenticar</span>';
      }
    }

    async function apiExec(args, stdin = null) {
      if (!sessionToken) {
        showAuthModal('Informe a chave de API para continuar.');
        throw new Error('Não autenticado');
      }

      let finalArgs = args;
      if (activeRepo && args.length > 0 && args[0] !== '-C' && args[0] !== 'catalog') {
        finalArgs = ['-C', activeRepo, ...args];
      }

      const res = await fetch('/api/exec', {
        method: 'POST',
        headers: {
          'Authorization': 'Bearer ' + sessionToken,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ args: finalArgs, stdin })
      });

      if (res.status === 401) {
        logout();
        showAuthModal('Sessão expirada ou chave inválida.');
        throw new Error('401 Unauthorized');
      }

      if (!res.ok) {
        throw new Error('HTTP ' + res.status + ': ' + (await res.text()));
      }
      return await res.json();
    }

    async function loadRepositories() {
      const select = document.getElementById('repo-select');
      if (!select) return;

      let initialRepo = '';
      try {
        const statusResp = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });
        if (statusResp.ok) {
          const data = await statusResp.json();
          const match = (data.stdout || '').match(/Repository:\s+([^\r\n]+)/);
          if (match && match[1]) {
            initialRepo = match[1].trim().split('/').pop();
          }
        }
      } catch (e) {}

      let repos = [];
      try {
        const catRes = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['catalog', 'list', '--json'] })
        });
        if (catRes.ok) {
          const data = await catRes.json();
          repos = JSON.parse(data.stdout || '[]');
        }
      } catch (e) {}

      select.innerHTML = '';

      if (!activeRepo) {
        activeRepo = initialRepo;
        if (activeRepo) {
          sessionStorage.setItem('cccp_active_repo', activeRepo);
        }
      }

      const seen = new Set();
      let hasActiveInList = false;

      if (repos && repos.length > 0) {
        repos.forEach(r => {
          if (r.name && !seen.has(r.name)) {
            seen.add(r.name);
            const opt = document.createElement('option');
            opt.value = r.name;
            opt.textContent = r.name + ' (' + (r.forge || 'local') + ')';
            if (r.name === activeRepo) {
              opt.selected = true;
              hasActiveInList = true;
            }
            select.appendChild(opt);
          }
        });
      }

      if (activeRepo && !hasActiveInList) {
        const opt = document.createElement('option');
        opt.value = activeRepo;
        opt.textContent = activeRepo + ' (ativo)';
        opt.selected = true;
        select.insertBefore(opt, select.firstChild);
      } else if (!activeRepo && select.options.length > 0) {
        activeRepo = select.options[0].value;
        sessionStorage.setItem('cccp_active_repo', activeRepo);
      }

      updateDivaContext();
    }

    function switchRepository(repoName) {
      activeRepo = repoName;
      sessionStorage.setItem('cccp_active_repo', activeRepo);
      updateDivaContext();

      const activeTab = document.querySelector('nav button.active');
      const tabName = activeTab ? activeTab.id.replace('nav-', '') : 'diva';

      if (tabName === 'status') {
        loadStatus();
      } else if (tabName === 'tickets') {
        loadTickets();
      }
    }

    function updateDivaContext() {
      const hint = document.getElementById('diva-repo-hint');
      if (hint) {
        hint.textContent = activeRepo ? '[Repo: ' + activeRepo + ']' : '';
      }
    }

    function switchTab(name) {
      document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('nav button').forEach(el => el.classList.remove('active'));
      const targetTab = document.getElementById('tab-' + name);
      const targetNav = document.getElementById('nav-' + name);
      if (targetTab) targetTab.classList.add('active');
      if (targetNav) targetNav.classList.add('active');

      if (name === 'status') loadStatus();
      if (name === 'tickets') loadTickets();
    }

    async function runDiva() {
      const text = document.getElementById('diva-input').value.trim();
      if (!text) return alert('Por favor, informe a descrição do problema.');

      const btn = document.getElementById('btn-diva-run');
      const container = document.getElementById('diva-results');
      btn.disabled = true;
      btn.innerHTML = '<span class="spinner"></span> Analisando com o Divã...';
      container.innerHTML = '<p style="color:var(--gold);">✦ Consultando o Gosplan & IA...</p>';

      try {
        const resp = await apiExec(['diva', '--dry-run', text]);
        if (!resp.ok && resp.exit_code !== 0) {
          container.innerHTML = '<pre class="terminal" style="color:var(--red);">' + (resp.stderr || resp.stdout) + '</pre>';
          return;
        }

        container.innerHTML = '<div class="card"><h2>Resultado da Triagem</h2><pre class="terminal">' + (resp.stdout || 'Nenhum ticket retornado.') + '</pre></div>';
      } catch (err) {
        container.innerHTML = '<pre class="terminal" style="color:var(--red);">Erro: ' + err.message + '</pre>';
      } finally {
        btn.disabled = false;
        btn.innerHTML = '<span>✦ Consultar Divã (IA)</span>';
      }
    }

    async function loadStatus() {
      const el = document.getElementById('status-output');
      el.textContent = 'Carregando status do repositório...';
      try {
        const resp = await apiExec(['status']);
        el.textContent = resp.stdout || resp.stderr || 'Nenhum retorno recebido.';
      } catch (err) {
        el.textContent = 'Erro ao consultar status: ' + err.message;
      }
    }

    async function loadTickets() {
      const el = document.getElementById('tickets-output');
      el.textContent = 'Consultando tickets sincronizados...';
      try {
        const resp = await apiExec(['tickets']);
        el.textContent = resp.stdout || resp.stderr || 'Nenhum ticket encontrado.';
      } catch (err) {
        el.textContent = 'Erro ao consultar tickets: ' + err.message;
      }
    }

    async function initCockpit() {
      if (!sessionToken) {
        showAuthModal();
        return;
      }
      try {
        const res = await fetch('/api/exec', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + sessionToken,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ args: ['status'] })
        });
        if (res.status === 401) {
          logout();
          return;
        }
        sessionStorage.setItem('cccp_token', sessionToken);
        hideAuthModal();
        await loadRepositories();
        const urlTab = new URLSearchParams(window.location.search).get('tab');
        if (urlTab && document.getElementById('tab-' + urlTab)) {
          switchTab(urlTab);
        }
      } catch (err) {
        hideAuthModal();
      }
    }

    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', initCockpit);
    } else {
      initCockpit();
    }
  </script>
</body>
</html>
EOF
}

# Main command dispatcher for cccp web
cmd_web() {
    _port=8080
    _host="127.0.0.1"
    _open_browser=0
    _quiet=0
    _initial_tab="diva"
    _cli_token=""

    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                echo "cccp web - Gosplan Interactive Web Cockpit & Mayak Command Bridge"
                echo "================================================================="
                echo ""
                echo "Usage:"
                echo "  cccp web [options] [tab]"
                echo "  cccp diva --web [options]"
                echo ""
                echo "Options:"
                echo "  -p, --port <port>       Port to listen on (default: 8080, auto-increments if busy)"
                echo "  --host <ip>             Host address to bind to (default: 127.0.0.1)"
                echo "  -o, --open              Open default web browser automatically (opt-in)"
                echo "  -t, --token <token>     API token for authentication (or via CCCP_TOKEN/CCCP_API_KEY)"
                echo "  --key, --api-key <k>    Alias for --token"
                echo "  --no-open               Do not open web browser (default behavior)"
                echo "  -q, --quiet             Quiet mode (suppress verbose logs)"
                echo "  -h, --help              Show this help message"
                echo ""
                echo "Tabs:"
                echo "  diva                    Open directly on Diva ticket triage cockpit (default)"
                echo "  status                  Open directly on repository inspection"
                echo "  tickets                 Open directly on active tickets list"
                return 0
                ;;
            -p|--port)
                shift
                [ $# -gt 0 ] || { echo "Error: --port requires a port argument" >&2; return 1; }
                _port="$1"
                ;;
            --host)
                shift
                [ $# -gt 0 ] || { echo "Error: --host requires an IP address argument" >&2; return 1; }
                _host="$1"
                ;;
            -o|--open)
                _open_browser=1
                ;;
            --no-open)
                _open_browser=0
                ;;
            -t|--token|--key|--api-key)
                shift
                [ $# -gt 0 ] || { echo "Error: $1 requires a token argument" >&2; return 1; }
                _cli_token="$1"
                ;;
            -q|--quiet)
                _quiet=1
                ;;
            diva|status|tickets)
                _initial_tab="$1"
                ;;
            *)
                echo "Error: Unknown option or argument '$1'. Use --help for usage." >&2
                return 1
                ;;
        esac
        shift
    done

    # 1. Resolve Mayak comrade binary
    _mk_bin=$(command -v mk 2>/dev/null || command -v mayak 2>/dev/null || true)
    if [ -z "$_mk_bin" ]; then
        if command -v _cccp_ensure_comrade >/dev/null 2>&1; then
            _cccp_ensure_comrade "mk" "mayak" || {
                echo "Error: Failed to obtain Comrade Mayak (mk). Install it or place in PATH." >&2
                return 1
            }
            _mk_bin=$(command -v mk 2>/dev/null || command -v mayak 2>/dev/null || true)
        fi
    fi

    if [ -z "$_mk_bin" ]; then
        echo "Error: Comrade Mayak (mk) is required to run the Web Cockpit." >&2
        echo "Install it via 'cccp comrades' or compile from https://codeberg.org/lumenpink/mayak." >&2
        return 1
    fi

    # 2. Resolve CCCP executable for bridge
    _cccp_bin=$(command -v cccp 2>/dev/null || true)
    if [ -z "$_cccp_bin" ]; then
        _cccp_bin="$0"
    fi

    # 3. Resolve session token
    _token="${_cli_token:-${CCCP_TOKEN:-${CCCP_API_KEY:-}}}"
    if [ -z "$_token" ]; then
        _token=$(_web_generate_token)
    fi

    # 4. Prepare temporary assets
    _uid=$(id -u 2>/dev/null || echo 1000)
    _web_dir="${TMPDIR:-/tmp}/cccp-web-${_uid}-$$"
    _web_export_assets "$_web_dir" "$_token" "$_port"

    trap 'rm -rf "$_web_dir" 2>/dev/null || true' EXIT INT TERM

    # 5. Build Mayak arguments
    set -- "$_web_dir" -p "$_port" --host "$_host" --bridge "$_cccp_bin" --auth "$_token" --cors
    if [ "$_open_browser" -eq 1 ]; then
        set -- "$@" --open
    fi
    if [ "$_quiet" -eq 1 ]; then
        set -- "$@" --quiet
    fi

    if [ "$_quiet" -eq 0 ]; then
        echo "★ Iniciando CCCP Web Cockpit via Comrade Mayak (mk) ★"
        echo "  🔑 Chave / Token: $_token"
        echo "  🌐 URL:           http://${_host}:${_port}/?tab=${_initial_tab}"
        if [ "$_open_browser" -eq 1 ]; then
            echo "  🚀 Navegador:     abertura automática acionada (--open)"
        fi
    fi

    "$_mk_bin" "$@"
}
