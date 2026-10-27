//! CCCP Profile Management Engine (GOST Standard)
//! Manages active GOST profiles in repositories, supporting canonical aliases,
//! recursive cascading inheritance (`extends`), CUSTOM.md overrides, and assembly of `.gost/GOST.md`.

use crate::cultural::Theme;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

/// Canonical profile catalog (alias, canonical URL, description)
pub const KNOWN_PROFILES: &[(&str, &str, &str)] = &[
    (
        "lumen",
        "https://gostlumen.2lp.in/profile.md",
        "Perfil Pessoal Lumen (extends Kollontai, short/long versions, 2-line footer, Dark Mode)",
    ),
    (
        "default",
        "https://gostlumen.2lp.in/profile.md",
        "Alias padrão para o perfil pessoal Lumen",
    ),
    (
        "kollontai",
        "https://kollontai.2lp.in/profile.md",
        "Perfil Base de Ecossistema Kollontai (Codeberg, cccp, *.2lp.in, Rust/PHP/Shell)",
    ),
];

/// Resolves a profile alias or raw URL into a canonical URL
pub fn resolve_profile_url(name_or_url: &str) -> String {
    let trimmed = name_or_url.trim();
    for (alias, url, _) in KNOWN_PROFILES {
        if trimmed.eq_ignore_ascii_case(alias) {
            return (*url).to_string();
        }
    }
    trimmed.to_string()
}

/// Extracts `extends: <url>` from markdown frontmatter or YAML block
pub fn extract_extends_url(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("extends:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Reads the currently configured GOST_PROFILE_URL from GOSTVER in the given directory
pub fn get_active_profile(root: &Path) -> io::Result<String> {
    let gostver_path = root.join("GOSTVER");
    if !gostver_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Arquivo GOSTVER não encontrado em {}", root.display()),
        ));
    }

    let content = fs::read_to_string(&gostver_path)?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("GOST_PROFILE_URL=") {
            let unquoted = rest.trim().trim_matches('"').trim_matches('\'');
            return Ok(unquoted.to_string());
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        "Chave GOST_PROFILE_URL não encontrada no GOSTVER",
    ))
}

/// Sets or updates the active GOST profile URL in GOSTVER
pub fn set_active_profile(root: &Path, name_or_url: &str) -> io::Result<String> {
    let canonical_url = resolve_profile_url(name_or_url);
    let gostver_path = root.join("GOSTVER");

    let new_content = if gostver_path.exists() {
        let content = fs::read_to_string(&gostver_path)?;
        let mut replaced = false;
        let mut lines = Vec::new();

        for line in content.lines() {
            if line.trim().starts_with("GOST_PROFILE_URL=") {
                lines.push(format!("GOST_PROFILE_URL=\"{}\"", canonical_url));
                replaced = true;
            } else {
                lines.push(line.to_string());
            }
        }

        if !replaced {
            lines.push(format!("GOST_PROFILE_URL=\"{}\"", canonical_url));
        }

        lines.join("\n") + "\n"
    } else {
        format!(
            "# GOSTVER — Declaração de Conformidade com o Padrão GOST\nGOST_VERSION=\"1.4.0\"\nGOST_PROFILE_URL=\"{}\"\nGOST_LANGUAGES=\"shell\"\n",
            canonical_url
        )
    };

    fs::write(&gostver_path, new_content)?;
    Ok(canonical_url)
}

/// Fetches a document by URL or local path
pub fn fetch_document_content(url_or_path: &str) -> io::Result<String> {
    let trimmed = url_or_path.trim();

    // 1. Direct local file
    let path = Path::new(trimmed);
    if path.is_file() {
        return fs::read_to_string(path);
    }

    // 2. Known local checkouts in ~/src
    if trimmed.contains("gostlumen.2lp.in") || trimmed.contains("gost-profile-lumen") {
        let local = PathBuf::from("/home/lumen/src/gost-profile-lumen/profile.md");
        if local.is_file() {
            if let Ok(c) = fs::read_to_string(&local) {
                return Ok(c);
            }
        }
    }
    if trimmed.contains("kollontai.2lp.in") || trimmed.contains("gost-profile-kollontai") {
        let local = PathBuf::from("/home/lumen/src/gost-profile-kollontai/profile.md");
        if local.is_file() {
            if let Ok(c) = fs::read_to_string(&local) {
                return Ok(c);
            }
        }
    }
    if trimmed.contains("gost.2lp.in") || trimmed.ends_with("SPEC.md") {
        let local = PathBuf::from("/home/lumen/src/gost/SPEC.md");
        if local.is_file() {
            if let Ok(c) = fs::read_to_string(&local) {
                return Ok(c);
            }
        }
    }

    // 3. HTTP GET via ureq
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if let Ok(resp) = ureq::get(trimmed).call() {
            if let Ok(body) = resp.into_body().read_to_string() {
                let lower = body.to_ascii_lowercase();
                if !lower.contains("<html") && !lower.contains("<!doctype") {
                    return Ok(body);
                }
            }
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        format!("Não foi possível obter conteúdo de '{}'", url_or_path),
    ))
}

/// Recursively resolves the cascading profile chain starting from the root profile URL down to base GOST
pub fn resolve_cascading_chain(start_url: &str) -> Vec<(String, String)> {
    let mut chain = Vec::new();
    let mut current_url = resolve_profile_url(start_url);
    let mut visited = std::collections::HashSet::new();

    while !current_url.is_empty() && visited.insert(current_url.clone()) {
        if let Ok(content) = fetch_document_content(&current_url) {
            let next_parent = extract_extends_url(&content);
            chain.push((current_url.clone(), content));
            if let Some(parent) = next_parent {
                current_url = resolve_profile_url(&parent);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Always ensure base GOST spec is included if not present
    let has_base = chain
        .iter()
        .any(|(u, _)| u.contains("gost.2lp.in") || u.ends_with("SPEC.md"));
    if !has_base {
        if let Ok(base_spec) = fetch_document_content("https://gost.2lp.in/SPEC.md") {
            chain.push(("https://gost.2lp.in/SPEC.md".to_string(), base_spec));
        } else if let Ok(base_spec) =
            fetch_document_content("https://codeberg.org/lumenpink/gost/raw/branch/main/SPEC.md")
        {
            chain.push((
                "https://codeberg.org/lumenpink/gost/raw/branch/main/SPEC.md".to_string(),
                base_spec,
            ));
        } else if let Ok(base_spec) = fetch_document_content("/home/lumen/src/gost/SPEC.md") {
            chain.push(("GOST Central SPEC.md".to_string(), base_spec));
        }
    }

    // The chain was built top-down (leaf -> parent -> base).
    // For compilation, we want base -> parent -> leaf so subsequent layers override earlier ones.
    chain.reverse();
    chain
}

/// Assembles all cascading profile layers and CUSTOM.md into `.gost/GOST.md`
pub fn assemble_gost_specs(root: &Path) -> io::Result<String> {
    let gostver_profile = get_active_profile(root).unwrap_or_else(|_| "lumen".to_string());
    let chain = resolve_cascading_chain(&gostver_profile);

    let mut output = String::new();
    output.push_str("# ☭ ГОСТ / GOSPLAN — Especificação Canônica Consolidada\n\n");
    output.push_str("> **Aviso de Integridade Gosplan:**\n");
    output.push_str(
        "> Este documento foi montado automaticamente pelo CCCP (`cccp profile assemble`).\n",
    );
    output.push_str("> A precedência é estrita de baixo para cima: cada camada subsequente sobrepõe as definições anteriores.\n\n");

    output.push_str("### 🌳 Árvore de Herança e Precedência Resolvida:\n");
    for (idx, (url, _)) in chain.iter().enumerate() {
        output.push_str(&format!("{}. Camada {}: `{}`\n", idx + 1, idx + 1, url));
    }

    let custom_file = root.join("CUSTOM.md");
    if custom_file.is_file() {
        output.push_str(&format!(
            "{}. Camada Local Soberana: `CUSTOM.md`\n",
            chain.len() + 1
        ));
    }
    output.push_str("\n---\n\n");

    // Adiciona cada camada da cadeia
    for (idx, (url, content)) in chain.iter().enumerate() {
        output.push_str(
            "<!-- ===================================================================== -->\n",
        );
        output.push_str(&format!("<!-- CAMADA {}: {} -->\n", idx + 1, url));
        output.push_str(
            "<!-- ===================================================================== -->\n\n",
        );
        output.push_str(content.trim());
        output.push_str("\n\n---\n\n");
    }

    // Anexa CUSTOM.md se existir
    if custom_file.is_file() {
        if let Ok(custom_content) = fs::read_to_string(&custom_file) {
            output.push_str(
                "<!-- ===================================================================== -->\n",
            );
            output.push_str(
                "<!-- CAMADA LOCAL SOBERANA: CUSTOM.md (Precedência Máxima do Repositório) -->\n",
            );
            output.push_str(
                "<!-- ===================================================================== -->\n\n",
            );
            output.push_str(custom_content.trim());
            output.push_str("\n\n");
        }
    }

    let gost_dir = root.join(".gost");
    let _ = fs::create_dir_all(&gost_dir);
    let target_path = gost_dir.join("GOST.md");
    fs::write(&target_path, &output)?;

    Ok(output)
}

/// Executes the CLI profile command
pub fn handle_profile_command(
    root: &Path,
    cmd: Option<crate::cli::ProfileCommand>,
    theme: &Theme,
) -> Result<(), String> {
    match cmd {
        Some(crate::cli::ProfileCommand::Get) | None => match get_active_profile(root) {
            Ok(url) => {
                let matching_alias = KNOWN_PROFILES
                    .iter()
                    .find(|(_, u, _)| *u == url)
                    .map(|(a, _, _)| *a);

                println!(
                    "{} Perfil GOST ativo no repositório:",
                    theme.cyan("★ [GOSPLAN]")
                );
                println!("  URL   : {}", theme.green(&url));
                if let Some(alias) = matching_alias {
                    println!("  Alias : {}", theme.cyan(alias));
                }
                Ok(())
            }
            Err(e) => Err(format!("Falha ao ler perfil ativo: {}", e)),
        },
        Some(crate::cli::ProfileCommand::List) => {
            let active = get_active_profile(root).ok();
            println!(
                "{} Perfis canônicos GOST catalogados:",
                theme.cyan("★ [GOSPLAN]")
            );
            println!();
            for (alias, url, desc) in KNOWN_PROFILES {
                if *alias == "default" {
                    continue;
                }
                let is_current = active.as_deref() == Some(*url);
                let mark = if is_current {
                    theme.green("● [ativo]")
                } else {
                    theme.dim("○")
                };
                println!("  {} {} ({})", mark, theme.bold(alias), theme.dim(url));
                println!("    ↳ {}", theme.dim(desc));
            }
            Ok(())
        }
        Some(crate::cli::ProfileCommand::Set(args)) => {
            match set_active_profile(root, &args.profile) {
                Ok(resolved_url) => {
                    println!(
                        "{} Perfil GOST atualizado com sucesso!",
                        theme.green("✓ [GOSPLAN]")
                    );
                    println!("  Novo Perfil : {}", theme.green(&resolved_url));
                    println!(
                        "  Alvo        : {}",
                        theme.dim(&root.join("GOSTVER").display().to_string())
                    );
                    // Automaticamente monta o perfil após definir
                    let _ = assemble_gost_specs(root);
                    Ok(())
                }
                Err(e) => Err(format!("Falha ao atualizar GOSTVER: {}", e)),
            }
        }
        Some(crate::cli::ProfileCommand::Assemble(ref assemble_args)) => {
            if let Some(ref target) = assemble_args.repo {
                let entries = crate::catalog::load_catalog(None, None);
                crate::catalog::assemble_catalog_repo(
                    &entries,
                    target,
                    assemble_args.dry_run,
                    assemble_args.auto_commit,
                    None,
                    theme,
                )
            } else if assemble_args.all {
                let entries = crate::catalog::load_catalog(None, None);
                let report =
                    crate::catalog::batch_assemble_gost(&entries, assemble_args.dry_run, theme);
                println!(
                    "{}",
                    crate::catalog::format_batch_assemble_report(&report, theme)
                );
                if !report.failed.is_empty() {
                    return Err(format!(
                        "Falha na montagem de {} repositório(s)",
                        report.failed.len()
                    ));
                }
                Ok(())
            } else {
                crate::catalog::assemble_repo(
                    root,
                    assemble_args.dry_run,
                    assemble_args.auto_commit,
                    theme,
                )
            }
        }
    }
}
