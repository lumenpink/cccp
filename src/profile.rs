//! CCCP Profile Management Engine (GOST Standard)
//! Manages active GOST profiles in repositories, supporting canonical aliases,
//! recursive cascading inheritance (`extends`), CUSTOM.md overrides, and assembly of `.gost/GOST.md`.

use crate::cultural::Theme;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct ProfileConfig {
    pub url: String,
    pub version: String,
}

pub fn parse_latest_version_from_catalog(catalog_json: &str) -> Option<String> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(catalog_json) {
        if let Some(latest) = v.get("latest").and_then(|l| l.as_str()) {
            return Some(latest.to_string());
        }
    }
    None
}

pub fn fetch_latest_version_from_catalog(base_url: &str) -> Option<String> {
    let clean_url = base_url.trim_end_matches('/');
    // Check local checkouts first
    if clean_url.contains("lilya.2lp.in") || clean_url.contains("gost-profile-lilya") {
        let p = Path::new("/home/lumen/src/gost-profile-lilya/site/versions.json");
        if let Ok(c) = fs::read_to_string(p) {
            if let Some(v) = parse_latest_version_from_catalog(&c) {
                return Some(v);
            }
        }
    }
    if clean_url.contains("kollontai.2lp.in") || clean_url.contains("gost-profile-kollontai") {
        let p = Path::new("/home/lumen/src/gost-profile-kollontai/site/versions.json");
        if let Ok(c) = fs::read_to_string(p) {
            if let Some(v) = parse_latest_version_from_catalog(&c) {
                return Some(v);
            }
        }
    }

    let catalog_url = format!("{}/versions.json", clean_url);
    if let Ok(resp) = ureq::get(&catalog_url).call() {
        if let Ok(body) = resp.into_body().read_to_string() {
            return parse_latest_version_from_catalog(&body);
        }
    }
    None
}

pub fn read_profile_config(root: &Path) -> io::Result<ProfileConfig> {
    let profile_toml = root.join(".rules/profile.toml");
    if profile_toml.is_file() {
        let content = fs::read_to_string(&profile_toml)?;
        if let Ok(cfg) = toml::from_str::<ProfileConfig>(&content) {
            return Ok(cfg);
        }
    }

    // Fallback: check GOSTVER
    if let Ok(url) = get_active_profile(root) {
        let (ver, _, _) = crate::audit::read_gostver(root);
        return Ok(ProfileConfig {
            url,
            version: if ver.is_empty() || ver == "Nenhuma" {
                "2.1.0".to_string()
            } else {
                ver
            },
        });
    }

    Err(Error::new(
        ErrorKind::NotFound,
        format!(
            "Nenhum perfil configurado em .rules/profile.toml ou GOSTVER em {}",
            root.display()
        ),
    ))
}

pub fn write_profile_config(root: &Path, cfg: &ProfileConfig) -> io::Result<()> {
    let rules_dir = root.join(".rules");
    fs::create_dir_all(&rules_dir)?;
    let serialized =
        toml::to_string_pretty(cfg).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    fs::write(rules_dir.join("profile.toml"), serialized)?;
    Ok(())
}

/// Canonical profile catalog (alias, canonical URL, description)
pub const KNOWN_PROFILES: &[(&str, &str, &str)] = &[
    (
        "lilya",
        "https://lilya.2lp.in/profile.md",
        "Perfil Pessoal Lilya (extends Kollontai, short/long versions, 2-line footer, Dark Mode)",
    ),
    (
        "lumen",
        "https://lilya.2lp.in/profile.md",
        "Alias de compatibilidade para o perfil pessoal Lilya",
    ),
    (
        "default",
        "https://lilya.2lp.in/profile.md",
        "Alias padrão para o perfil pessoal Lilya",
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
    if trimmed == "profiles/lumen.md" || trimmed == "profiles/lilya.md" {
        return "https://lilya.2lp.in/profile.md".to_string();
    }
    for (alias, url, _) in KNOWN_PROFILES {
        if trimmed.eq_ignore_ascii_case(alias) {
            return (*url).to_string();
        }
    }
    trimmed.to_string()
}

/// Extracts `extends: <url>` or `inherits_from.base_url` from markdown frontmatter or YAML block
pub fn extract_extends_url(content: &str) -> Option<String> {
    if let Some(base) = extract_inherits_from_base_url(content) {
        return Some(base);
    }
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

/// Extracts `base_url` inside `inherits_from:` block from YAML frontmatter
pub fn extract_inherits_from_base_url(content: &str) -> Option<String> {
    let mut in_inherits = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("inherits_from:") {
            in_inherits = true;
            continue;
        }
        if in_inherits {
            if let Some(rest) = trimmed.strip_prefix("base_url:") {
                let val = rest.trim().trim_matches('"').trim_matches('\'').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            } else if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                in_inherits = false;
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
            "# GOSTVER — Declaração de Conformidade com o Padrão GOST\nGOST_VERSION=\"{}\"\nGOST_PROFILE_URL=\"{}\"\nGOST_LANGUAGES=\"shell\"\n",
            crate::newrepo::DEFAULT_GOST_VERSION,
            canonical_url
        )
    };

    fs::write(&gostver_path, new_content)?;
    Ok(canonical_url)
}

/// Updates or sets GOST_VERSION in GOSTVER file
pub fn update_gostver_version(root: &Path, version: &str) -> io::Result<()> {
    let gostver_path = root.join("GOSTVER");

    let new_content = if gostver_path.exists() {
        let content = fs::read_to_string(&gostver_path)?;
        let mut replaced = false;
        let mut lines = Vec::new();

        for line in content.lines() {
            if line.trim().starts_with("GOST_VERSION=") {
                lines.push(format!("GOST_VERSION=\"{}\"", version));
                replaced = true;
            } else {
                lines.push(line.to_string());
            }
        }

        if !replaced {
            lines.insert(0, format!("GOST_VERSION=\"{}\"", version));
        }

        lines.join("\n") + "\n"
    } else {
        format!(
            "# GOSTVER — Declaração de Conformidade com o Padrão GOST\nGOST_VERSION=\"{}\"\nGOST_PROFILE_URL=\"{}\"\nGOST_LANGUAGES=\"shell\"\n",
            version,
            crate::newrepo::DEFAULT_PROFILE_URL
        )
    };

    fs::write(&gostver_path, new_content)?;
    Ok(())
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
    if trimmed.contains("lilya.2lp.in")
        || trimmed.contains("gost-profile-lilya")
        || trimmed.contains("gostlumen.2lp.in")
        || trimmed.contains("gost-profile-lumen")
    {
        let local = PathBuf::from("/home/lumen/src/gost-profile-lilya/profile.md");
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
    if trimmed.contains("gosplan.2lp.in") || trimmed.contains("gosplan") {
        let local = PathBuf::from("/home/lumen/src/gosplan/SPEC.md");
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

/// Fetches the declarative `rules.toml` content corresponding to a normative layer
pub fn fetch_rules_for_layer(url: &str) -> Option<String> {
    if url.contains("lilya") {
        let p = Path::new("/home/lumen/src/gost-profile-lilya/rules.toml");
        if p.is_file() {
            if let Ok(c) = fs::read_to_string(p) {
                return Some(c);
            }
        }
    }
    if url.contains("kollontai") {
        let p = Path::new("/home/lumen/src/gost-profile-kollontai/rules.toml");
        if p.is_file() {
            if let Ok(c) = fs::read_to_string(p) {
                return Some(c);
            }
        }
    }
    if url.contains("gosplan") {
        let p = Path::new("/home/lumen/src/gosplan/rules.toml");
        if p.is_file() {
            if let Ok(c) = fs::read_to_string(p) {
                return Some(c);
            }
        }
    }
    if url.contains("gost") {
        let p1 = Path::new("/home/lumen/src/gost/specs/rules.toml");
        if p1.is_file() {
            if let Ok(c) = fs::read_to_string(p1) {
                return Some(c);
            }
        }
        let p2 = Path::new("/home/lumen/src/gost/rules.toml");
        if p2.is_file() {
            if let Ok(c) = fs::read_to_string(p2) {
                return Some(c);
            }
        }
    }

    let candidates = get_rules_candidate_urls(url);
    fetch_rules_from_urls(&candidates)
}

/// Returns ordered candidate URLs for retrieving `rules.toml` for a given normative profile URL
pub fn get_rules_candidate_urls(url: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let rules_url = if url.ends_with('/') {
        format!("{}rules.toml", url)
    } else if let Some((dir, _)) = url.rsplit_once('/') {
        format!("{}/rules.toml", dir)
    } else {
        format!("{}/rules.toml", url)
    };
    candidates.push(rules_url);

    if url.contains("lilya") {
        candidates.push(
            "https://gambi.2lp.in/lumen/gost-profile-lilya/raw/branch/main/rules.toml".to_string(),
        );
        candidates.push("https://lilya.2lp.in/rules.toml".to_string());
    } else if url.contains("kollontai") {
        candidates.push(
            "https://gambi.2lp.in/lumen/gost-profile-kollontai/raw/branch/main/rules.toml"
                .to_string(),
        );
        candidates.push("https://kollontai.2lp.in/rules.toml".to_string());
    } else if url.contains("gosplan") {
        candidates
            .push("https://gambi.2lp.in/lumen/gosplan/raw/branch/main/rules.toml".to_string());
        candidates.push("https://gosplan.2lp.in/rules.toml".to_string());
    } else if url.contains("gost") {
        candidates.push("https://gambi.2lp.in/lumen/gost/raw/branch/main/rules.toml".to_string());
        candidates.push("https://gost.2lp.in/rules.toml".to_string());
    }

    let mut deduped = Vec::new();
    for c in candidates {
        if !deduped.contains(&c) {
            deduped.push(c);
        }
    }
    deduped
}

/// Tries to fetch rules content from an ordered list of candidate URLs
pub fn fetch_rules_from_urls(urls: &[String]) -> Option<String> {
    for c_url in urls {
        if c_url.starts_with("http://") || c_url.starts_with("https://") {
            if let Ok(resp) = ureq::get(c_url).call() {
                if let Ok(body) = resp.into_body().read_to_string() {
                    let lower = body.to_ascii_lowercase();
                    if !lower.contains("<html")
                        && !lower.contains("<!doctype")
                        && !body.trim().is_empty()
                    {
                        return Some(body);
                    }
                }
            }
        }
    }
    None
}

/// Sanitizes prohibited imperialist terms from compiled normative specs.
/// Enforces compliance with GOST anti-imperialism directives by mapping
/// prohibited vocabulary to canonical autonomy terminology.
pub fn sanitize_prohibited_terms(text: &str) -> String {
    text.replace("Soberania", "Autonomia")
        .replace("soberania", "autonomia")
        .replace("Soberano", "Canônico")
        .replace("soberano", "canônico")
        .replace("Soberana", "Canônica")
        .replace("soberana", "canônica")
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
            fetch_document_content("https://gambi.2lp.in/lumen/gost/raw/branch/main/SPEC.md")
        {
            chain.push((
                "https://gambi.2lp.in/lumen/gost/raw/branch/main/SPEC.md".to_string(),
                base_spec,
            ));
        } else if let Ok(base_spec) =
            fetch_document_content("https://git.2lp.in/lumen/gost/raw/branch/main/SPEC.md")
        {
            chain.push((
                "https://git.2lp.in/lumen/gost/raw/branch/main/SPEC.md".to_string(),
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

/// Assembles all cascading profile layers and CUSTOM.md into `.rules/RULES.md` and `.rules/RULES.toml`.
pub fn assemble_rules_specs(root: &Path) -> io::Result<String> {
    let mut cfg = read_profile_config(root).unwrap_or_else(|_| ProfileConfig {
        url: "https://lilya.2lp.in".to_string(),
        version: "2.1.0".to_string(),
    });

    if cfg.version.is_empty() || cfg.version == "latest" {
        if let Some(latest) = fetch_latest_version_from_catalog(&cfg.url) {
            cfg.version = latest;
            let _ = write_profile_config(root, &cfg);
        }
    }

    let chain = resolve_cascading_chain(&cfg.url);

    let mut output = String::new();
    output.push_str("# ☭ Normas e Diretrizes Compiladas do Repositório\n\n");
    output.push_str("> **Aviso de Integridade:**\n");
    output.push_str(
        "> Este documento foi montado automaticamente pelo CCCP (`cccp sync` / `assemble`).\n",
    );
    output.push_str("> A precedência é estrita de baixo para cima: cada camada subsequente sobrepõe as definições anteriores.\n\n");

    output.push_str("### 🌳 Árvore de Herança e Precedência Resolvida:\n");
    for (idx, (url, _)) in chain.iter().enumerate() {
        output.push_str(&format!("{}. Camada {}: `{}`\n", idx + 1, idx + 1, url));
    }

    let custom_file = root.join("CUSTOM.md");
    if custom_file.is_file() {
        output.push_str(&format!(
            "{}. Camada Local Autônoma: `CUSTOM.md`\n",
            chain.len() + 1
        ));
    }
    output.push_str("\n---\n\n");

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

    if custom_file.is_file() {
        if let Ok(custom_content) = fs::read_to_string(&custom_file) {
            output.push_str(
                "<!-- ===================================================================== -->\n",
            );
            output.push_str(
                "<!-- CAMADA LOCAL AUTÔNOMA: CUSTOM.md (Precedência Máxima do Repositório) -->\n",
            );
            output.push_str(
                "<!-- ===================================================================== -->\n\n",
            );
            output.push_str(custom_content.trim());
            output.push_str("\n\n");
        }
    }

    let rules_dir = root.join(".rules");
    fs::create_dir_all(&rules_dir)?;
    let target_rules_md = rules_dir.join("RULES.md");
    let sanitized_output = sanitize_prohibited_terms(&output);
    fs::write(&target_rules_md, &sanitized_output)?;

    // Compila regras declarativas
    let mut combined_rules: toml::map::Map<String, toml::Value> = toml::map::Map::new();
    for (url, _) in &chain {
        if let Some(rules_content) = fetch_rules_for_layer(url) {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&rules_content) {
                if let Some(rules_table) = toml_val.get("rules").and_then(|r| r.as_table()) {
                    for (k, v) in rules_table {
                        combined_rules.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    }

    // Merge custom-rules.toml from root if exists
    let custom_rules_candidates = [
        root.join("custom-rules.toml"),
        root.join("custom_rules.toml"),
        root.join("rules.toml"),
    ];
    for cr in &custom_rules_candidates {
        if cr.is_file() {
            if let Ok(cr_content) = fs::read_to_string(cr) {
                if let Ok(toml_val) = toml::from_str::<toml::Value>(&cr_content) {
                    if let Some(rules_table) = toml_val.get("rules").and_then(|r| r.as_table()) {
                        for (k, v) in rules_table {
                            combined_rules.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            break;
        }
    }

    if !combined_rules.is_empty() {
        let mut root_val = toml::map::Map::new();
        root_val.insert("rules".to_string(), toml::Value::Table(combined_rules));
        if let Ok(serialized) = toml::to_string_pretty(&toml::Value::Table(root_val)) {
            let rules_out = rules_dir.join("RULES.toml");
            let _ = fs::write(&rules_out, serialized);
        }
    }

    // Purga arquivos obsoletos em .rules/
    let _ = fs::remove_file(rules_dir.join("rules.json"));
    let _ = fs::remove_file(rules_dir.join("PROFILE.md"));

    Ok(sanitized_output)
}

/// Assembles all cascading profile layers and CUSTOM.md into `.gost/GOST.md`,
/// synchronizes GOST resources in `.gost/`, and updates GOSTVER to the canonical version.
pub fn assemble_gost_specs(root: &Path) -> io::Result<String> {
    let output = assemble_rules_specs(root)?;
    let gost_dir = root.join(".gost");
    if gost_dir.is_dir() {
        let _ = fs::write(gost_dir.join("GOST.md"), &output);
        let rules_toml = root.join(".rules/RULES.toml");
        if rules_toml.is_file() {
            let _ = fs::copy(&rules_toml, gost_dir.join("rules.toml"));
        }
    }
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
