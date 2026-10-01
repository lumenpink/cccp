//! CCCP Dependencies Manager (cmd_deps)
//! Centralized catalog, constraint verification, and automated acquisition for
//! external binaries, CI runners (act), and Gosplan ecosystem comrades.

use crate::cultural::Theme;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: &'static str,
    pub category: &'static str,
    pub requirement: &'static str,
    pub min_version: &'static str,
    pub source: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
}

pub fn get_catalog() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "git",
            category: "core",
            requirement: "Obrigatório",
            min_version: "2.25.0",
            source: "sistema",
            description: "Sistema de controle de versão essencial para o CCCP",
            aliases: &[],
        },
        Dependency {
            name: "docker",
            category: "ci",
            requirement: "Recomendado",
            min_version: "20.10.0",
            source: "sistema",
            description: "Engine de contêineres para execução do CI gate local (act)",
            aliases: &[],
        },
        Dependency {
            name: "act",
            category: "ci",
            requirement: "Opcional",
            min_version: "0.2.50",
            source: "gh:nektos/act",
            description: "Runner local para workflows de CI (GitHub/Forgejo Actions)",
            aliases: &[],
        },
        Dependency {
            name: "sv",
            category: "comrade",
            requirement: "Opcional",
            min_version: "0.1.0",
            source: "codeberg:lumenpink/svyaz",
            description: "Camarada micro HTTP/HTTPS em Rust com TLS embutido",
            aliases: &["svyaz"],
        },
        Dependency {
            name: "kv",
            category: "comrade",
            requirement: "Opcional",
            min_version: "0.1.0",
            source: "codeberg:lumenpink/kvant",
            description: "Camarada micro editor de texto de terminal em Rust",
            aliases: &["kvant"],
        },
        Dependency {
            name: "mk",
            category: "comrade",
            requirement: "Opcional",
            min_version: "0.1.0",
            source: "codeberg:lumenpink/mayak",
            description: "Camarada micro servidor estático com Mayak Command Bridge",
            aliases: &["mayak"],
        },
        Dependency {
            name: "curl",
            category: "http",
            requirement: "Recomendado",
            min_version: "7.60.0",
            source: "sistema",
            description: "Cliente HTTP para sincronização de tickets, atualizações e API",
            aliases: &[],
        },
        Dependency {
            name: "wget",
            category: "http",
            requirement: "Opcional",
            min_version: "1.20.0",
            source: "sistema",
            description: "Cliente HTTP/FTP alternativo para downloads e atualizações",
            aliases: &[],
        },
        Dependency {
            name: "jq",
            category: "util",
            requirement: "Opcional",
            min_version: "1.6",
            source: "sistema",
            description: "Acelerador opcional de extração e parsing de payloads JSON",
            aliases: &[],
        },
        Dependency {
            name: "tea",
            category: "forge",
            requirement: "Opcional",
            min_version: "0.9.0",
            source: "sistema",
            description: "CLI oficial do Gitea / Codeberg para gestão de repositórios",
            aliases: &[],
        },
        Dependency {
            name: "gh",
            category: "forge",
            requirement: "Opcional",
            min_version: "2.0.0",
            source: "sistema",
            description: "CLI oficial do GitHub para integração e despacho de tickets",
            aliases: &[],
        },
    ]
}

/// Compares if `actual` SemVer version is greater than or equal to `required`.
/// Development builds (e.g. starting with "dev", "development", "trunk", "master", "main")
/// are treated as conforming.
pub fn version_ge(actual: &str, required: &str) -> bool {
    let clean_act = actual.strip_prefix('v').or_else(|| actual.strip_prefix('V')).unwrap_or(actual).trim();
    let clean_req = required.strip_prefix('v').or_else(|| required.strip_prefix('V')).unwrap_or(required).trim();

    if clean_act.starts_with("dev")
        || clean_act.starts_with("development")
        || clean_act.starts_with("trunk")
        || clean_act.starts_with("master")
        || clean_act.starts_with("main")
    {
        return true;
    }

    let parse_triplet = |s: &str| -> (u64, u64, u64) {
        let parts: Vec<&str> = s.split('.').collect();
        let extract_num = |val: &str| -> u64 {
            let num_str: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
            num_str.parse().unwrap_or(0)
        };

        let maj = parts.first().map(|p| extract_num(p)).unwrap_or(0);
        let min = parts.get(1).map(|p| extract_num(p)).unwrap_or(0);
        let patch = parts.get(2).map(|p| extract_num(p)).unwrap_or(0);
        (maj, min, patch)
    };

    let act_triplet = parse_triplet(clean_act);
    let req_triplet = parse_triplet(clean_req);

    act_triplet >= req_triplet
}

/// Locates a binary in PATH by checking its primary name and any aliases
pub fn find_binary_path(name: &str, aliases: &[&str]) -> Option<PathBuf> {
    let check_cmd = |cmd: &str| -> Option<PathBuf> {
        let out = Command::new("which").arg(cmd).output().ok()?;
        if out.status.success() {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
        None
    };

    if let Some(p) = check_cmd(name) {
        return Some(p);
    }

    for alias in aliases {
        if let Some(p) = check_cmd(alias) {
            return Some(p);
        }
    }

    None
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Probes and extracts the installed version of a binary
pub fn probe_version(name: &str, bin_path: &Path) -> Option<String> {
    let out = Command::new(bin_path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stripped = strip_ansi(&stdout);
    let first_line = stripped.lines().next().unwrap_or("").trim();

    let clean_ver = match name {
        "git" => {
            // "git version 2.43.0"
            first_line.split("version").nth(1)?.split_whitespace().next()?.to_string()
        }
        "docker" => {
            // "Docker version 28.0.1, build ..."
            let after_ver = first_line.split("version").nth(1)?;
            after_ver.split(',').next()?.split_whitespace().next()?.to_string()
        }
        "act" => {
            // "act version 0.2.75"
            first_line.split("version").nth(1)?.split_whitespace().next()?.to_string()
        }
        "curl" => {
            // "curl 8.5.0 (x86_64-pc-linux-gnu)..."
            first_line.strip_prefix("curl")?.split_whitespace().next()?.to_string()
        }
        "wget" => {
            // "GNU Wget 1.21.4 built on linux-gnu."
            first_line.split("Wget").nth(1)?.split_whitespace().next()?.to_string()
        }
        "jq" => {
            // "jq-1.7.1"
            first_line.strip_prefix("jq-").unwrap_or(first_line).split_whitespace().next()?.to_string()
        }
        "tea" => {
            // "tea version 0.9.2" or "Version: 0.9.2"
            if let Some(after) = first_line.split("version").nth(1) {
                after.split_whitespace().next()?.to_string()
            } else if let Some(after) = first_line.split("Version:").nth(1) {
                after.split_whitespace().next()?.to_string()
            } else {
                first_line.to_string()
            }
        }
        "gh" => {
            // "gh version 2.45.0 (2024-03-04)"
            first_line.split("version").nth(1)?.split_whitespace().next()?.to_string()
        }
        "kv" | "sv" | "mk" => {
            // extract any sequence of digits and dots
            let mut found = String::new();
            for word in first_line.split_whitespace() {
                let stripped = word.strip_prefix('v').unwrap_or(word);
                if stripped.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && stripped.contains('.') {
                    found = stripped.to_string();
                    break;
                }
            }
            if !found.is_empty() {
                found
            } else {
                "0.1.0".to_string()
            }
        }
        _ => {
            first_line.split_whitespace().last()?.to_string()
        }
    };

    let result = clean_ver.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-').to_string();
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Detects the host system package manager for automated install instructions
pub fn get_system_pkg_mgr() -> &'static str {
    if Path::new("/etc/arch-release").exists() || Path::new("/etc/artix-release").exists() {
        "pacman -S"
    } else if Path::new("/etc/debian_version").exists() {
        "apt-get install -y"
    } else if Path::new("/etc/fedora-release").exists() || Path::new("/etc/redhat-release").exists() {
        "dnf install -y"
    } else if Path::new("/etc/alpine-release").exists() {
        "apk add"
    } else if std::env::consts::OS == "macos" {
        "brew install"
    } else {
        "apt-get install -y"
    }
}

/// Subcommand: cccp deps list
pub fn cmd_deps_list(theme: &Theme) {
    println!("{}", theme.bold("========================================================================================"));
    println!("{}", theme.bold(" ★ [ГОСПЛАН] Catálogo de Dependências e Utilitários do Ecossistema CCCP ★"));
    println!("{}", theme.bold("========================================================================================"));
    println!(
        "{}",
        theme.bold(&format!(
            "{:<8} {:<9} {:<12} {:<10} {:<22} {}",
            "BINÁRIO", "CATEGORIA", "EXIGÊNCIA", "MÍNIMO", "ORIGEM", "PROPÓSITO"
        ))
    );
    println!(
        "{:<8} {:<9} {:<12} {:<10} {:<22} {}",
        "-------", "---------", "---------", "------", "------", "---------"
    );

    for dep in get_catalog() {
        println!(
            "{:<8} {:<9} {:<12} {:<10} {:<22} {}",
            dep.name,
            dep.category,
            dep.requirement,
            format!(">= {}", dep.min_version),
            dep.source,
            dep.description
        );
    }

    println!("{}", theme.bold("========================================================================================"));
}

/// Subcommand: cccp deps check [dep1 dep2...]
pub fn cmd_deps_check(filter_deps: &[String], theme: &Theme) -> Result<bool, String> {
    let catalog = get_catalog();

    let mut has_critical_failure = false;
    let mut total_checked = 0;
    let mut total_ok = 0;
    let mut total_missing = 0;
    let mut total_outdated = 0;

    println!("{}", theme.bold("========================================================================================"));
    println!("{}", theme.bold(" ★ [ГОСПЛАН] Auditoria de Conformidade de Dependências (Deps Check) ★"));
    println!("{}", theme.bold("========================================================================================"));
    println!(
        "{}",
        theme.bold(&format!(
            "{:<10} {:<12} {:<14} {:<14} {:<16} {}",
            "BINÁRIO", "EXIGÊNCIA", "INSTALADA", "REQUISITO", "ESTADO", "LOCALIZAÇÃO / DETALHES"
        ))
    );
    println!(
        "{:<10} {:<12} {:<14} {:<14} {:<16} {}",
        "-------", "---------", "---------", "---------", "------", "----------------------"
    );

    for dep in catalog {
        if !filter_deps.is_empty() && !filter_deps.iter().any(|f| f == dep.name || dep.aliases.contains(&f.as_str())) {
            continue;
        }

        total_checked += 1;
        let bin_path = find_binary_path(dep.name, dep.aliases);

        if bin_path.is_none() {
            total_missing += 1;
            if dep.requirement == "Obrigatório" {
                has_critical_failure = true;
            }

            println!(
                "{:<10} {:<12} {:<14} {:<14} {} {}",
                dep.name,
                dep.requirement,
                "-",
                format!(">= {}", dep.min_version),
                theme.red(&format!("{:<16}", "[AUSENTE]")),
                format!("Não encontrado no PATH (Origem: {})", dep.source)
            );
            continue;
        }

        let path = bin_path.unwrap();
        let cur_ver = probe_version(dep.name, &path).unwrap_or_else(|| "detectado".to_string());

        let mut extra_info = String::new();
        if dep.name == "docker" {
            let docker_running = Command::new("docker")
                .arg("info")
                .output()
                .map(|o| o.status.success())
                .unwrap_or_else(|_| Path::new("/var/run/docker.sock").exists());

            if docker_running {
                extra_info = "daemon operacional".to_string();
            } else {
                extra_info = "daemon parado ou sem permissão".to_string();
            }
        }

        let ver_ok = if cur_ver != "detectado" {
            version_ge(&cur_ver, dep.min_version)
        } else {
            true
        };

        if dep.name == "docker" && extra_info == "daemon parado ou sem permissão" {
            total_outdated += 1;
            println!(
                "{:<10} {:<12} {:<14} {:<14} {} {} ({})",
                dep.name,
                dep.requirement,
                cur_ver,
                format!(">= {}", dep.min_version),
                theme.yellow(&format!("{:<16}", "[DAEMON OFF]")),
                path.display(),
                extra_info
            );
        } else if ver_ok {
            total_ok += 1;
            let mut details = path.display().to_string();
            if !extra_info.is_empty() {
                details = format!("{} ({})", details, extra_info);
            }
            println!(
                "{:<10} {:<12} {:<14} {:<14} {} {}",
                dep.name,
                dep.requirement,
                cur_ver,
                format!(">= {}", dep.min_version),
                theme.green(&format!("{:<16}", "[OK]")),
                details
            );
        } else {
            total_outdated += 1;
            println!(
                "{:<10} {:<12} {:<14} {:<14} {} {} (abaixo do requisito recomendado)",
                dep.name,
                dep.requirement,
                cur_ver,
                format!(">= {}", dep.min_version),
                theme.yellow(&format!("{:<16}", "[DESATUALIZADO]")),
                path.display()
            );
        }
    }

    println!("{}", theme.bold("========================================================================================"));
    println!(
        "Total verificado: {} | Conformes: {} | Ausentes: {} | Alertas: {}",
        total_checked,
        theme.green(&total_ok.to_string()),
        theme.red(&total_missing.to_string()),
        theme.yellow(&total_outdated.to_string())
    );

    if has_critical_failure {
        eprintln!(
            "{}",
            theme.red("[FALHA] Dependências obrigatórias ausentes. O CCCP não poderá operar normalmente.")
        );
        Ok(false)
    } else {
        println!(
            "{}",
            theme.green("[SUCESSO] Auditoria de dependências satisfeita.")
        );
        Ok(true)
    }
}

/// Subcommand: cccp deps install <dep... | --all>
pub fn cmd_deps_install(targets: &[String], install_all: bool, theme: &Theme) -> Result<(), String> {
    if targets.is_empty() && !install_all {
        println!("Uso: cccp deps install <nome-da-dependência ... | --all>");
        println!("Exemplo: cccp deps install act");
        println!("         cccp deps install sv kv mk");
        println!("         cccp deps install --all");
        return Err("Nenhuma dependência especificada para instalação.".to_string());
    }

    let pm = get_system_pkg_mgr();

    if install_all {
        println!(
            "{}",
            theme.bold("★ Instalando ferramentas standalone e camaradas do ecossistema... ★\n")
        );
        println!(
            "Para dependências gerenciadas pelo sistema operacional:\n  {}",
            theme.cyan(&format!("sudo {} git docker curl jq", pm))
        );
        return Ok(());
    }

    for target in targets {
        match target.as_str() {
            "act" => {
                println!(
                    "{}",
                    theme.cyan("★ Para instalar o act runner via GitHub Releases: ★")
                );
                println!("  Consulte: https://github.com/nektos/act/releases");
                println!("  Ou execute via package manager se disponível: brew install act / pacman -S act");
            }
            "kv" | "kvant" | "sv" | "svyaz" | "mk" | "mayak" => {
                println!(
                    "{}",
                    theme.bold(&format!("Camarada '{}': consulte o módulo de camaradas (cccp comrades).", target))
                );
            }
            "git" | "docker" | "curl" | "wget" | "jq" | "tea" | "gh" => {
                println!("{}", theme.bold(&format!("Dependência de sistema: {}", target)));
                println!("  Para instalar no seu sistema operacional, execute:");
                println!("    {}\n", theme.cyan(&format!("sudo {} {}", pm, target)));
            }
            _ => {
                eprintln!(
                    "Erro: Dependência desconhecida '{}'. Consulte a lista com: cccp deps list",
                    target
                );
            }
        }
    }

    Ok(())
}
