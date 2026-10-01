//! CCCP Unified Server Engine (Static, Dev & VDNKh Demo)
//! Pure-Rust HTTP server for local preview, documentation, and development.

use crate::cultural::Theme;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Configuration options for `cccp serve`
#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub target: Option<String>,
    pub dev: bool,
    pub demo: bool,
    pub host: String,
    pub port: u16,
    pub open: bool,
    pub quiet: bool,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            target: None,
            dev: false,
            demo: false,
            host: "0.0.0.0".to_string(),
            port: 8080,
            open: false,
            quiet: false,
        }
    }
}

/// Detects target directory following priority: explicit override > ./site > ./dist > ./public > ./
pub fn detect_target_dir(override_dir: Option<&str>, base_dir: &Path) -> PathBuf {
    if let Some(d) = override_dir {
        if !d.is_empty() {
            let p = base_dir.join(d);
            if p.is_dir() {
                return p;
            }
            let direct = PathBuf::from(d);
            if direct.is_dir() {
                return direct;
            }
        }
    }

    for candidate in ["site", "dist", "public"] {
        let p = base_dir.join(candidate);
        if p.is_dir() {
            return p;
        }
    }

    base_dir.to_path_buf()
}

/// Checks whether a given TCP port is currently in use on the host
pub fn is_port_in_use(port: u16, host: &str) -> bool {
    TcpListener::bind((host, port)).is_err()
}

/// Discovers a free TCP port starting from `start_port`
pub fn find_free_port(start_port: u16, host: &str) -> u16 {
    let mut cur = start_port;
    while cur < 65535 {
        if !is_port_in_use(cur, host) {
            return cur;
        }
        cur = cur.saturating_add(1);
    }
    start_port
}

/// Retrieves local network (LAN) IP address
pub fn get_lan_ip() -> Option<String> {
    if let Ok(out) = Command::new("ip").args(["route", "get", "1.1.1.1"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(idx) = text.find("src ") {
                let rest = &text[idx + 4..];
                if let Some(ip) = rest.split_whitespace().next() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    if let Ok(out) = Command::new("hostname").arg("-I").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(ip) = text.split_whitespace().next() {
                return Some(ip.to_string());
            }
        }
    }

    None
}

/// Retrieves Tailscale VPN IP address if active
pub fn get_tailscale_ip() -> Option<String> {
    if let Ok(out) = Command::new("tailscale").args(["ip", "-4"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(ip) = text.lines().next() {
                let trimmed = ip.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    if let Ok(out) = Command::new("ip")
        .args(["-4", "addr", "show", "dev", "tailscale0"])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(idx) = text.find("inet ") {
                let rest = &text[idx + 5..];
                if let Some(ip_with_mask) = rest.split_whitespace().next() {
                    let ip = ip_with_mask.split('/').next().unwrap_or(ip_with_mask);
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Attempts to open URL in standard desktop browser
pub fn open_browser(url: &str) {
    let has_display = std::env::var("DISPLAY").map(|v| !v.is_empty()).unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

    if !has_display {
        return;
    }

    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();

    #[cfg(not(target_os = "macos"))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

/// Guess MIME type based on file extension
pub fn guess_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "txt" | "md" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Decodes percent-encoded URL string (e.g. `%20` -> ` `)
fn url_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(val);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}

/// Displays Soviet Gosplan or VDNKh Demo banner
pub fn display_banner(mode: &str, target: &Path, host: &str, port: u16, theme: &Theme) {
    let lan_ip = get_lan_ip();
    let ts_ip = get_tailscale_ip();

    if mode == "demo" {
        println!(
            "\n{}",
            theme.gold_bold("★════════════════════════════════════════════════════════════════════════════★")
        );
        println!(
            "{}",
            theme.bold("  ☭ ВДНХ — EXPOSIÇÃO DAS CONQUISTAS DA ECONOMIA NACIONAL (MODO DEMO) ☭")
        );
        println!(
            "{}",
            theme.cyan("  Pavilhão Central da Demonstração & Inovação Tecnológica")
        );
        println!(
            "{}\n",
            theme.gold_bold("★════════════════════════════════════════════════════════════════════════════★")
        );
    } else {
        println!(
            "\n{}",
            theme.bold("☭ [ГОСТ / GOSPLAN] CCCP HTTP SERVER ENGINE")
        );
        println!(
            "Modo: {} | Alvo: {}",
            theme.cyan(mode),
            target.display()
        );
        println!("────────────────────────────────────────────────────────────────────────────");
    }

    println!(
        "  {}  http://localhost:{}/",
        theme.green("➜ Local (Loopback):"),
        port
    );
    if host != "127.0.0.1" {
        if let Some(ref ip) = lan_ip {
            println!(
                "  {}  http://{}:{}/",
                theme.green("➜ Rede Local (LAN):"),
                ip,
                port
            );
        }
        if let Some(ref ip) = ts_ip {
            println!(
                "  {}  http://{}:{}/",
                theme.green("➜ Tailscale / VPN:"),
                ip,
                port
            );
        }
    }
    println!("────────────────────────────────────────────────────────────────────────────");
    println!(
        "Pressione {} para encerrar o serviço de transmissão.\n",
        theme.bold("Ctrl+C")
    );
}

/// Handles a single incoming HTTP connection
fn handle_connection(mut stream: TcpStream, root_dir: &Path, quiet: bool) {
    let mut buffer = [0u8; 4096];
    let n = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request_str = String::from_utf8_lossy(&buffer[..n]);
    let first_line = match request_str.lines().next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    let method = parts[0];
    let raw_path = parts[1];

    if method != "GET" && method != "HEAD" {
        let _ = stream.write_all(
            b"HTTP/1.1 405 Method Not Allowed\r\nAllow: GET, HEAD\r\nContent-Length: 0\r\n\r\n",
        );
        return;
    }

    // Strip query parameters
    let path_no_query = raw_path.split('?').next().unwrap_or(raw_path);
    let decoded_path = url_decode(path_no_query);

    // Prevent directory traversal
    if decoded_path.contains("..") {
        let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    let clean_path = decoded_path.trim_start_matches('/');
    let candidate = root_dir.join(clean_path);

    let target_file = if candidate.is_dir() {
        let index = candidate.join("index.html");
        if index.is_file() {
            Some(index)
        } else {
            None
        }
    } else if candidate.is_file() {
        Some(candidate)
    } else {
        None
    };

    match target_file {
        Some(file_path) => {
            if let Ok(metadata) = fs::metadata(&file_path) {
                let file_len = metadata.len();
                let mime = guess_mime_type(&file_path);

                if !quiet {
                    println!("  [200] {} {} ({} bytes)", method, raw_path, file_len);
                }

                let header = format!(
                    "HTTP/1.1 200 OK\r\n\
Content-Type: {}\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
X-Content-Type-Options: nosniff\r\n\
\r\n",
                    mime, file_len
                );

                if stream.write_all(header.as_bytes()).is_ok() && method == "GET" {
                    if let Ok(mut f) = fs::File::open(&file_path) {
                        let mut copy_buf = [0u8; 8192];
                        while let Ok(read_bytes) = f.read(&mut copy_buf) {
                            if read_bytes == 0 || stream.write_all(&copy_buf[..read_bytes]).is_err() {
                                break;
                            }
                        }
                    }
                }
            } else {
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            }
        }
        None => {
            if !quiet {
                println!("  [404] {} {}", method, raw_path);
            }
            let body = "404 Not Found\n";
            let resp = format!(
                "HTTP/1.1 404 Not Found\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
\r\n\
{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    }
}

/// Runs the multi-threaded static HTTP server
pub fn run_static_server(
    root_dir: &Path,
    host: &str,
    port: u16,
    quiet: bool,
    shutdown_flag: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    let listener = TcpListener::bind((host, port))
        .map_err(|e| format!("Failed to bind server to {}:{}: {}", host, port, e))?;

    // Set non-blocking to allow checking shutdown flag gracefully
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to configure listener: {}", e))?;

    let root_path = root_dir.to_path_buf();

    loop {
        if let Some(ref flag) = shutdown_flag {
            if flag.load(Ordering::Relaxed) {
                break;
            }
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let r = root_path.clone();
                thread::spawn(move || {
                    handle_connection(stream, &r, quiet);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                if !quiet {
                    eprintln!("Connection accept error: {}", e);
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    Ok(())
}

/// Executes `cccp serve`
pub fn cmd_serve(opts: ServeOptions, theme: &Theme) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target_dir = detect_target_dir(opts.target.as_deref(), &current_dir);

    let free_port = find_free_port(opts.port, &opts.host);

    // =========================================================================
    // 1. Modo Demonstração (cccp serve --demo)
    // =========================================================================
    if opts.demo {
        let demo_scripts = [
            current_dir.join("scripts/demo.sh"),
            current_dir.join(".cccp/demo.sh"),
        ];

        for script in &demo_scripts {
            if script.is_file() {
                println!(
                    "✦ Executando rotina de demonstração ({})...",
                    script.display()
                );
                let _ = Command::new("sh").arg(script).status();
                break;
            }
        }

        display_banner("demo", &target_dir, &opts.host, free_port, theme);

        let url = format!("http://localhost:{}/", free_port);
        open_browser(&url);

        return run_static_server(&target_dir, &opts.host, free_port, opts.quiet, None);
    }

    // =========================================================================
    // 2. Modo Desenvolvimento (cccp serve --dev)
    // =========================================================================
    if opts.dev {
        // 2.1 Delegation to repo custom script
        let dev_scripts = [
            current_dir.join("scripts/dev.sh"),
            current_dir.join(".cccp/dev.sh"),
        ];

        for script in &dev_scripts {
            if script.is_file() {
                println!("✦ Delegando execução para {}...", script.display());
                let status = Command::new("sh")
                    .arg(script)
                    .status()
                    .map_err(|e| format!("Failed to execute '{}': {}", script.display(), e))?;
                if status.success() {
                    return Ok(());
                } else {
                    return Err(format!(
                        "Script '{}' exited with code {:?}",
                        script.display(),
                        status.code()
                    ));
                }
            }
        }

        // 2.2 PHP Project
        let php_index = current_dir.join("index.php");
        let php_public = current_dir.join("public/index.php");
        if php_index.is_file() || php_public.is_file() {
            if let Ok(check) = Command::new("php").arg("-v").output() {
                if check.status.success() {
                    display_banner("dev (PHP)", &target_dir, &opts.host, free_port, theme);
                    if opts.open {
                        open_browser(&format!("http://localhost:{}/", free_port));
                    }
                    let mut cmd = Command::new("php");
                    cmd.args(["-S", &format!("{}:{}", opts.host, free_port)]);
                    if php_public.is_file() {
                        cmd.args(["-t", "public"]);
                    }
                    let _ = cmd.status();
                    return Ok(());
                }
            }
        }

        // 2.3 Node.js Project
        let package_json = current_dir.join("package.json");
        if package_json.is_file() {
            if let Ok(check) = Command::new("npm").arg("--version").output() {
                if check.status.success() {
                    display_banner("dev (Node.js)", &package_json, &opts.host, free_port, theme);
                    if opts.open {
                        open_browser(&format!("http://localhost:{}/", free_port));
                    }
                    let content = fs::read_to_string(&package_json).unwrap_or_default();
                    let npm_subcmd = if content.contains("\"dev\":") {
                        "run"
                    } else if content.contains("\"serve\":") {
                        "run"
                    } else {
                        "start"
                    };

                    let mut cmd = Command::new("npm");
                    if npm_subcmd == "run" {
                        let target_script = if content.contains("\"dev\":") {
                            "dev"
                        } else {
                            "serve"
                        };
                        cmd.args(["run", target_script]);
                    } else {
                        cmd.arg("start");
                    }
                    let _ = cmd.status();
                    return Ok(());
                }
            }
        }

        // 2.4 Compiled / Static project with optional initial build
        if current_dir.join("combine.sh").is_file() {
            println!("✦ Executando build inicial com combine.sh...");
            let _ = Command::new("./combine.sh").status();
        } else if current_dir.join("Cargo.toml").is_file() {
            println!("✦ Executando build inicial com cargo build...");
            let _ = Command::new("cargo").arg("build").status();
        } else if current_dir.join("Makefile").is_file() {
            println!("✦ Executando build inicial com make...");
            let _ = Command::new("make").status();
        }

        display_banner("dev", &target_dir, &opts.host, free_port, theme);
        if opts.open {
            open_browser(&format!("http://localhost:{}/", free_port));
        }

        return run_static_server(&target_dir, &opts.host, free_port, opts.quiet, None);
    }

    // =========================================================================
    // 3. Modo Estático Padrão (cccp serve [diretório])
    // =========================================================================
    display_banner("static", &target_dir, &opts.host, free_port, theme);
    if opts.open {
        open_browser(&format!("http://localhost:{}/", free_port));
    }
    run_static_server(&target_dir, &opts.host, free_port, opts.quiet, None)
}
