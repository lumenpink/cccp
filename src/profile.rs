//! CCCP Profile Management Engine (GOST Standard)
//! Manages active GOST profiles in repositories, supporting canonical aliases and custom URLs.

use crate::cultural::Theme;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

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
                    Ok(())
                }
                Err(e) => Err(format!("Falha ao atualizar GOSTVER: {}", e)),
            }
        }
    }
}
