//! CCCP Web Cockpit & Mayak Command Bridge
//! Embedded Web UI, session authentication, and API execution bridge.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration options for `cccp web` / `cccp cockpit`
#[derive(Debug, Clone)]
pub struct CockpitOptions {
    pub port: u16,
    pub host: String,
    pub token: Option<String>,
    pub open: bool,
    pub quiet: bool,
    pub initial_tab: String,
    pub enable_matraca: bool,
    pub enable_diva: bool,
    pub enable_status: bool,
    pub enable_tickets: bool,
}

impl Default for CockpitOptions {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            token: None,
            open: false,
            quiet: false,
            initial_tab: "matraca".to_string(),
            enable_matraca: true,
            enable_diva: true,
            enable_status: true,
            enable_tickets: true,
        }
    }
}

/// Generates a random session token of 32 hexadecimal characters
pub fn generate_token() -> String {
    if let Ok(mut f) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut buf = [0u8; 16];
        if f.read_exact(&mut buf).is_ok() {
            return buf.iter().map(|b| format!("{:02x}", b)).collect();
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{:016x}{:016x}", nanos, pid as u128)
}

/// Locates executable in system PATH
pub fn find_in_path(binary: &str) -> Option<PathBuf> {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(binary);
            if candidate.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = candidate.metadata() {
                        if meta.permissions().mode() & 0o111 != 0 {
                            return Some(candidate);
                        }
                    }
                }
                #[cfg(not(unix))]
                return Some(candidate);
            }
        }
    }
    None
}

/// Exports single-page Cockpit HTML and assets to output directory
pub fn export_assets(out_dir: &Path, opts: &CockpitOptions) -> Result<PathBuf, String> {
    if !opts.enable_matraca && !opts.enable_diva && !opts.enable_status && !opts.enable_tickets {
        return Err("Error: At least one tab must be enabled in cockpit.".to_string());
    }

    fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "Failed to create asset directory '{}': {}",
            out_dir.display(),
            e
        )
    })?;

    let mut nav_html = String::new();
    let mut main_html = String::new();

    if opts.enable_matraca {
        let act = if opts.initial_tab == "matraca" {
            " active"
        } else {
            ""
        };
        nav_html.push_str(&format!(
            "    <button id=\"nav-matraca\" class=\"{}\" onclick=\"switchTab('matraca')\">🦜 Matraca (Web REPL)</button>\n",
            act
        ));
        main_html.push_str(&format!(
            r#"    <!-- TAB 0: MATRACA (WEB REPL) -->
    <div id="tab-matraca" class="tab-content{}">
      <div class="card" style="display:flex; flex-direction:column; height:calc(100vh - 220px); min-height:480px; padding:1.25rem;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.75rem; flex-wrap:wrap; gap:0.5rem; border-bottom:1px solid var(--border); padding-bottom:0.75rem;">
          <div style="display:flex; align-items:center; gap:0.6rem;">
            <h2>🦜 Matraca — Web REPL Móvel</h2>
            <span id="matraca-conn-status" style="font-size:0.75rem; background:rgba(35,134,54,0.15); color:var(--green); border:1px solid var(--green); padding:0.15rem 0.5rem; border-radius:12px; font-weight:600;">● Online</span>
          </div>
          <div style="display:flex; gap:0.5rem; align-items:center;">
            <button class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.8rem;" onclick="loadMatracaHistory()">Atualizar</button>
            <button class="btn btn-secondary" style="padding:0.25rem 0.6rem; font-size:0.8rem;" onclick="clearMatracaChat()">Limpar Tela</button>
          </div>
        </div>

        <div id="matraca-chat" style="flex:1; overflow-y:auto; padding:0.5rem; display:flex; flex-direction:column; gap:0.75rem; margin-bottom:0.75rem; scroll-behavior:smooth;">
          <div style="text-align:center; color:var(--text-muted); font-size:0.85rem; padding:1.5rem 0;">
            Conectado ao despachante do CCCP. Digite uma mensagem abaixo para interagir com o agente.
          </div>
        </div>

        <div style="display:flex; gap:0.5rem; align-items:flex-end;">
          <textarea id="matraca-input" rows="2" placeholder="Digite uma instrução para o agy (Shift+Enter para quebra de linha, Enter para enviar)..." style="margin-bottom:0; resize:none;" onkeydown="handleMatracaKey(event)"></textarea>
          <button id="btn-matraca-send" class="btn" style="height:54px; min-width:80px; display:flex; align-items:center; justify-content:center;" onclick="sendMatraca()">
            <span>➤ Enviar</span>
          </button>
        </div>
      </div>
    </div>
"#,
            act
        ));
    }

    if opts.enable_diva {
        let act = if opts.initial_tab == "diva" {
            " active"
        } else {
            ""
        };
        nav_html.push_str(&format!(
            "    <button id=\"nav-diva\" class=\"{}\" onclick=\"switchTab('diva')\">🛋️ O Divã (Triage IA)</button>\n",
            act
        ));
        main_html.push_str(&format!(
            r#"    <!-- TAB 1: O DIVÃ -->
    <div id="tab-diva" class="tab-content{}">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.5rem; flex-wrap:wrap; gap:0.5rem;">
          <h2>🛋️ O Divã — Triagem Semântica de Tickets</h2>
          <span id="diva-repo-hint" style="font-family:var(--font-mono); font-size:0.85rem; color:var(--gold); font-weight:bold;"></span>
        </div>
        <p class="desc">Descreva livremente o problema, bug ou necessidade. A IA do Gosplan estruturará tickets atômicos seguindo Conventional Commits com o checklist canônico das 9 fases DD-TDD do GOST.</p>
        
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
"#,
            act
        ));
    }

    if opts.enable_status {
        let act = if opts.initial_tab == "status" {
            " active"
        } else {
            ""
        };
        nav_html.push_str(&format!(
            "    <button id=\"nav-status\" class=\"{}\" onclick=\"switchTab('status')\">📊 Inspeção do Repositório</button>\n",
            act
        ));
        main_html.push_str(&format!(
            r#"    <!-- TAB 2: STATUS -->
    <div id="tab-status" class="tab-content{}">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
          <h2>📊 Inspeção Gosplan do Repositório</h2>
          <button class="btn btn-secondary" onclick="loadStatus()">Atualizar Status</button>
        </div>
        <pre id="status-output" class="terminal">Carregando status do repositório...</pre>
      </div>
    </div>
"#,
            act
        ));
    }

    if opts.enable_tickets {
        let act = if opts.initial_tab == "tickets" {
            " active"
        } else {
            ""
        };
        nav_html.push_str(&format!(
            "    <button id=\"nav-tickets\" class=\"{}\" onclick=\"switchTab('tickets')\">📋 Tickets Ativos</button>\n",
            act
        ));
        main_html.push_str(&format!(
            r#"    <!-- TAB 3: TICKETS -->
    <div id="tab-tickets" class="tab-content{}">
      <div class="card">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
          <h2>📋 Tickets Ativos do Projeto</h2>
          <button class="btn btn-secondary" onclick="loadTickets()">Sincronizar com Forge</button>
        </div>
        <pre id="tickets-output" class="terminal">Carregando tickets...</pre>
      </div>
    </div>
"#,
            act
        ));
    }

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CCCP Web Cockpit — Радиостанция «Маяк»</title>
  <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>☭</text></svg>">
  <style>
    :root {{
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
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      line-height: 1.5;
      padding-bottom: 4rem;
    }}
    header {{
      background: var(--surface);
      border-bottom: 1px solid var(--border);
      padding: 1rem 1.5rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
    }}
    .brand {{
      display: flex;
      align-items: center;
      gap: 0.75rem;
      font-weight: 700;
      font-size: 1.25rem;
      color: var(--text-bright);
    }}
    .brand span.badge {{
      background: var(--red);
      color: #fff;
      font-size: 0.75rem;
      padding: 0.2rem 0.5rem;
      border-radius: 4px;
      letter-spacing: 0.5px;
    }}
    .session-info {{
      font-family: var(--font-mono);
      font-size: 0.8rem;
      color: var(--text-muted);
      background: var(--bg);
      padding: 0.35rem 0.75rem;
      border: 1px solid var(--border);
      border-radius: 6px;
    }}
    nav {{
      background: #11151c;
      border-bottom: 1px solid var(--border);
      display: flex;
      padding: 0 1.5rem;
      gap: 0.5rem;
    }}
    nav button {{
      background: none;
      border: none;
      color: var(--text-muted);
      font-size: 0.95rem;
      font-weight: 600;
      padding: 0.85rem 1.25rem;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      transition: all 0.2s;
    }}
    nav button:hover {{ color: var(--text-bright); }}
    nav button.active {{ color: var(--gold); border-bottom-color: var(--gold); }}
    main {{ max-width: 1000px; margin: 2rem auto; padding: 0 1.5rem; }}
    .tab-content {{ display: none; }}
    .tab-content.active {{ display: block; }}
    .card {{
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.5rem;
      margin-bottom: 1.5rem;
    }}
    h2 {{ font-size: 1.3rem; color: var(--text-bright); margin-bottom: 0.5rem; }}
    p.desc {{ color: var(--text-muted); font-size: 0.9rem; margin-bottom: 1.25rem; }}
    textarea, input[type="text"], input[type="password"], select {{
      width: 100%;
      background: var(--bg);
      border: 1px solid var(--border);
      color: var(--text-bright);
      border-radius: 6px;
      padding: 0.75rem;
      font-size: 0.95rem;
      font-family: inherit;
      margin-bottom: 1rem;
    }}
    textarea:focus, input[type="text"]:focus, input[type="password"]:focus, select:focus {{
      outline: none;
      border-color: var(--gold);
    }}
    .btn {{
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
    }}
    .btn:hover {{ background: var(--red-hover); }}
    .btn-secondary {{ background: var(--surface-hover); color: var(--text); border: 1px solid var(--border); }}
    .btn-secondary:hover {{ background: #282e38; color: var(--text-bright); }}
    pre.terminal {{
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
    }}
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
        Port: <strong>{port}</strong>
      </div>
      <button id="btn-auth-lock" class="btn btn-secondary" style="padding:0.35rem 0.75rem; font-size:0.8rem; display:none;" onclick="logout()">
        🔒 Bloquear
      </button>
    </div>
  </header>

  <nav>
{nav_html}  </nav>

  <main>
{main_html}  </main>

  <script>
    let sessionToken = sessionStorage.getItem('cccp_token') || new URLSearchParams(window.location.search).get('token') || '';
    let activeRepo = sessionStorage.getItem('cccp_active_repo') || '';

    function showAuthModal(errMsg = '') {{
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (errMsg) {{
        errEl.textContent = errMsg;
        errEl.style.display = 'block';
      }} else {{
        errEl.style.display = 'none';
      }}
      modal.style.display = 'flex';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'none';
      setTimeout(() => {{
        const inp = document.getElementById('auth-token-input');
        if (inp) inp.focus();
      }}, 50);
    }}

    function hideAuthModal() {{
      const modal = document.getElementById('auth-modal');
      const errEl = document.getElementById('auth-error');
      if (modal) modal.style.display = 'none';
      if (errEl) errEl.style.display = 'none';
      const lockBtn = document.getElementById('btn-auth-lock');
      if (lockBtn) lockBtn.style.display = 'inline-flex';
    }}

    function logout() {{
      sessionStorage.removeItem('cccp_token');
      sessionToken = '';
      showAuthModal('Sessão bloqueada.');
    }}

    function switchTab(name) {{
      document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('nav button').forEach(el => el.classList.remove('active'));
      const targetTab = document.getElementById('tab-' + name);
      const targetNav = document.getElementById('nav-' + name);
      if (targetTab) targetTab.classList.add('active');
      if (targetNav) targetNav.classList.add('active');
    }}
  </script>
</body>
</html>
"#,
        port = opts.port,
        nav_html = nav_html,
        main_html = main_html
    );

    let index_path = out_dir.join("index.html");
    fs::write(&index_path, index_html).map_err(|e| {
        format!(
            "Failed to write index.html to '{}': {}",
            index_path.display(),
            e
        )
    })?;

    Ok(index_path)
}

/// Executes `cccp web` / `cccp cockpit`
pub fn cmd_cockpit(mut opts: CockpitOptions) -> Result<(), String> {
    if !opts.enable_matraca && !opts.enable_diva && !opts.enable_status && !opts.enable_tickets {
        return Err("Error: At least one tab must be enabled in cockpit.".to_string());
    }

    // Resolve token: explicit -> CCCP_TOKEN -> CCCP_API_KEY -> generated
    let token = opts
        .token
        .clone()
        .or_else(|| std::env::var("CCCP_TOKEN").ok())
        .or_else(|| std::env::var("CCCP_API_KEY").ok())
        .unwrap_or_else(generate_token);
    opts.token = Some(token.clone());

    // Resolve initial tab fallback if selected tab is disabled
    match opts.initial_tab.as_str() {
        "matraca" if !opts.enable_matraca => {
            if opts.enable_diva {
                opts.initial_tab = "diva".to_string();
            } else if opts.enable_status {
                opts.initial_tab = "status".to_string();
            } else if opts.enable_tickets {
                opts.initial_tab = "tickets".to_string();
            }
        }
        "diva" if !opts.enable_diva => {
            if opts.enable_matraca {
                opts.initial_tab = "matraca".to_string();
            } else if opts.enable_status {
                opts.initial_tab = "status".to_string();
            } else if opts.enable_tickets {
                opts.initial_tab = "tickets".to_string();
            }
        }
        "status" if !opts.enable_status => {
            if opts.enable_matraca {
                opts.initial_tab = "matraca".to_string();
            } else if opts.enable_diva {
                opts.initial_tab = "diva".to_string();
            } else if opts.enable_tickets {
                opts.initial_tab = "tickets".to_string();
            }
        }
        "tickets" if !opts.enable_tickets => {
            if opts.enable_matraca {
                opts.initial_tab = "matraca".to_string();
            } else if opts.enable_diva {
                opts.initial_tab = "diva".to_string();
            } else if opts.enable_status {
                opts.initial_tab = "status".to_string();
            }
        }
        _ => {}
    }

    // Check for Comrade Mayak (mk) in PATH or ~/.local/bin
    let mk_bin = find_in_path("mk")
        .or_else(|| find_in_path("mayak"))
        .or_else(|| {
            if let Ok(home) = std::env::var("HOME") {
                let candidate = PathBuf::from(home).join(".local/bin/mk");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
            None
        });

    let current_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "cccp".to_string());

    let temp_dir = std::env::temp_dir().join(format!("cccp-web-{}", std::process::id()));
    export_assets(&temp_dir, &opts)?;

    if let Some(mk_path) = mk_bin {
        let mut cmd = Command::new(mk_path);
        cmd.arg(&temp_dir)
            .args(["-p", &opts.port.to_string()])
            .args(["--host", &opts.host])
            .args(["--bridge", &current_exe])
            .args(["--auth", &token])
            .arg("--cors");

        if opts.open {
            cmd.arg("--open");
        }
        if opts.quiet {
            cmd.arg("--quiet");
        }

        if !opts.quiet {
            println!("★ Iniciando CCCP Web Cockpit via Comrade Mayak (mk) ★");
            println!("  🔑 Chave / Token: {}", token);
            println!(
                "  🌐 URL:           http://{}:{}/?tab={}",
                opts.host, opts.port, opts.initial_tab
            );
            if opts.open {
                println!("  🚀 Navegador:     abertura automática acionada (--open)");
            }
        }

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to run Mayak: {}", e))?;
        let _ = fs::remove_dir_all(&temp_dir);

        if status.success() {
            Ok(())
        } else {
            Err(format!("Mayak process exited with {:?}", status.code()))
        }
    } else {
        // Fallback: use embedded static server
        if !opts.quiet {
            println!("★ Iniciando CCCP Web Cockpit (servidor embutido) ★");
            println!("  🔑 Chave / Token: {}", token);
            println!(
                "  🌐 URL:           http://{}:{}/?tab={}",
                opts.host, opts.port, opts.initial_tab
            );
        }

        let res =
            crate::serve::run_static_server(&temp_dir, &opts.host, opts.port, opts.quiet, None);
        let _ = fs::remove_dir_all(&temp_dir);
        res
    }
}
