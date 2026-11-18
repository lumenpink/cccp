//! CCCP Profile Management Engine (GOST Standard)
//! Manages active GOST profiles in repositories, supporting canonical aliases,
//! recursive cascading inheritance (`extends`), CUSTOM.md overrides, and assembly of `.gost/GOST.md`.

use crate::cultural::Theme;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct ProfileConfig {
    pub url: String,
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct UpgradeFrontmatter {
    pub standard: String,
    pub tier: String,
    pub version: String,
    pub replaces: Option<String>,
    pub valid_since: String,
    #[serde(default)]
    pub breaking: bool,
}

#[derive(Debug, Clone)]
pub struct UpgradePlan {
    pub standard: String,
    pub current_version: String,
    pub target_version: String,
    pub chain: Vec<String>,
    pub combined_delta: String,
    pub breaking: bool,
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
    let catalog_url = format!("{}/versions.json", clean_url);
    if catalog_url.starts_with("http://") || catalog_url.starts_with("https://") {
        if let Ok(resp) = ureq::get(&catalog_url).call() {
            if let Ok(body) = resp.into_body().read_to_string() {
                return parse_latest_version_from_catalog(&body);
            }
        }
    } else {
        let p = Path::new(&catalog_url);
        if let Ok(c) = fs::read_to_string(p) {
            return parse_latest_version_from_catalog(&c);
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

/// Updates GOST_VERSION in GOSTVER file only if the legacy file still exists.
pub fn update_gostver_version(root: &Path, version: &str) -> io::Result<()> {
    let gostver_path = root.join("GOSTVER");
    if !gostver_path.exists() {
        return Ok(());
    }

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

    let new_content = lines.join("\n") + "\n";
    fs::write(&gostver_path, new_content)?;
    Ok(())
}

/// Builds canonical URLs for RULES.md and RULES.toml following the standard agnostic schema:
/// `{base_url}/{version}/RULES.md` and `{base_url}/{version}/RULES.toml`
pub fn build_canonical_rules_urls(base_url: &str, version: &str) -> (String, String) {
    let clean = base_url.trim_end_matches('/');
    let md = format!("{}/{}/RULES.md", clean, version);
    let toml = format!("{}/{}/RULES.toml", clean, version);
    (md, toml)
}

/// Fetches a document by URL or local path
pub fn fetch_document_content(url_or_path: &str) -> io::Result<String> {
    let trimmed = url_or_path.trim();

    // 1. Direct local file
    let path = Path::new(trimmed);
    if path.is_file() {
        return fs::read_to_string(path);
    }

    // 2. HTTP GET via ureq
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

/// Fetches the declarative `RULES.toml` content corresponding to a normative layer
pub fn fetch_rules_for_layer(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let path = Path::new(trimmed);
    if path.is_file() {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let local_candidates = [parent.join("RULES.toml"), parent.join("rules.toml")];
        for cand in &local_candidates {
            if cand.is_file() {
                if let Ok(c) = fs::read_to_string(cand) {
                    return Some(c);
                }
            }
        }
    }

    let candidates = get_rules_candidate_urls(trimmed);
    fetch_rules_from_urls(&candidates)
}

/// Returns deterministic candidate URL for retrieving `RULES.toml` for a given normative profile URL
pub fn get_rules_candidate_urls(url: &str) -> Vec<String> {
    let trimmed = url.trim();
    let mut candidates = Vec::new();
    if trimmed.ends_with("/RULES.md") {
        candidates.push(trimmed.replace("/RULES.md", "/RULES.toml"));
    } else if trimmed.ends_with(".md") {
        if let Some((dir, _)) = trimmed.rsplit_once('/') {
            candidates.push(format!("{}/RULES.toml", dir));
        } else {
            candidates.push("RULES.toml".to_string());
        }
    } else if trimmed.ends_with('/') {
        candidates.push(format!("{}RULES.toml", trimmed));
    } else {
        candidates.push(format!("{}/RULES.toml", trimmed));
    }
    candidates
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
        let doc_url = if current_url.ends_with(".md") {
            current_url.clone()
        } else if current_url.ends_with('/') {
            format!("{}RULES.md", current_url)
        } else {
            format!("{}/RULES.md", current_url)
        };

        if let Ok(content) = fetch_document_content(&doc_url) {
            let next_parent = extract_extends_url(&content);
            chain.push((doc_url, content));
            if let Some(parent) = next_parent {
                current_url = resolve_profile_url(&parent);
            } else {
                break;
            }
        } else if let Ok(content) = fetch_document_content(&current_url) {
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

    let is_local_file = Path::new(&cfg.url).is_file();
    let (base_profile_url, is_legacy_profile_md) =
        if !is_local_file && (cfg.url.starts_with("http://") || cfg.url.starts_with("https://")) {
            if let Some(base) = cfg.url.strip_suffix("/profile.md") {
                (base.to_string(), true)
            } else {
                (cfg.url.trim_end_matches('/').to_string(), false)
            }
        } else {
            (cfg.url.trim_end_matches('/').to_string(), false)
        };

    let mut start_url = if (cfg.url.ends_with(".md") && !is_legacy_profile_md) || is_local_file {
        cfg.url.clone()
    } else {
        format!("{}/{}/RULES.md", base_profile_url, cfg.version)
    };

    let mut chain = resolve_cascading_chain(&start_url);
    if chain.is_empty() {
        if let Some(latest) = fetch_latest_version_from_catalog(&base_profile_url) {
            if latest != cfg.version {
                cfg.version = latest;
                start_url = format!("{}/{}/RULES.md", base_profile_url, cfg.version);
                chain = resolve_cascading_chain(&start_url);
            }
        }
    }

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

    // Compila regras declarativas com injecao de tier hierarquico
    let mut combined_rules: toml::map::Map<String, toml::Value> = toml::map::Map::new();
    for (idx, (url, _)) in chain.iter().enumerate() {
        let layer_tier = (idx + 1) as i64;
        if let Some(rules_content) = fetch_rules_for_layer(url) {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&rules_content) {
                if let Some(rules_table) = toml_val.get("rules").and_then(|r| r.as_table()) {
                    for (k, v) in rules_table {
                        let mut rule_val = v.clone();
                        if let Some(tbl) = rule_val.as_table_mut() {
                            if !tbl.contains_key("tier") {
                                tbl.insert("tier".to_string(), toml::Value::Integer(layer_tier));
                            }
                        }
                        combined_rules.insert(k.clone(), rule_val);
                    }
                }
            }
        }
    }

    // Merge custom-rules.toml from root if exists (Tier 99 local)
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
                            let mut rule_val = v.clone();
                            if let Some(tbl) = rule_val.as_table_mut() {
                                if !tbl.contains_key("tier") {
                                    tbl.insert("tier".to_string(), toml::Value::Integer(99));
                                }
                            }
                            combined_rules.insert(k.clone(), rule_val);
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
            let _ = fs::write(&rules_out, &serialized);
            let _ = fs::write(rules_dir.join("rules.toml"), &serialized);
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
    if gost_dir.is_dir() || root.join("GOSTVER").is_file() {
        let _ = fs::create_dir_all(&gost_dir);

        let (_, prof, langs) = crate::audit::read_gostver(root);
        let resolved_profile = if prof == "None" || prof == "Nenhum" || prof.is_empty() {
            crate::newrepo::DEFAULT_PROFILE_URL
        } else {
            &prof
        };
        let resolved_langs = if langs == "None" || langs == "Nenhuma" || langs.is_empty() {
            "shell"
        } else {
            &langs
        };

        crate::newrepo::sync_gost_specs(
            root,
            resolved_profile,
            resolved_langs,
            crate::newrepo::DEFAULT_GOST_VERSION,
            true,
            &crate::cultural::Theme::with_color(false),
        );

        let _ = fs::write(gost_dir.join("GOST.md"), &output);
        let rules_toml = root.join(".rules/RULES.toml");
        if rules_toml.is_file() {
            let _ = fs::copy(&rules_toml, gost_dir.join("rules.toml"));
        }

        if root.join("GOSTVER").is_file() {
            let _ = update_gostver_version(root, crate::newrepo::DEFAULT_GOST_VERSION);
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
        Some(crate::cli::ProfileCommand::Upgrade(ref upgrade_args)) => {
            handle_profile_upgrade(root, upgrade_args, theme)
        }
        Some(crate::cli::ProfileCommand::Setup(ref setup_args)) => {
            setup_gost_environment(root, setup_args.profile.as_deref(), setup_args.force, theme)
        }
    }
}

/// Inicializa ou migra um repositório para o GOST v3.0, garantindo a anatomia completa de .gost/ e purgando GOSTVER legado
pub fn setup_gost_environment(
    root: &Path,
    profile_override: Option<&str>,
    force: bool,
    theme: &Theme,
) -> Result<(), String> {
    let gost_dir = root.join(".gost");
    if force && gost_dir.exists() {
        fs::remove_dir_all(&gost_dir)
            .map_err(|e| format!("Falha ao remover .gost existente: {}", e))?;
    }
    fs::create_dir_all(&gost_dir).map_err(|e| format!("Falha ao criar diretório .gost: {}", e))?;

    // 1. Purgar GOSTVER legado da raiz se existir
    let legacy_gostver = root.join("GOSTVER");
    if legacy_gostver.exists() {
        let _ = fs::remove_file(&legacy_gostver);
        println!(
            "  {} Purgado arquivo legado GOSTVER da raiz",
            theme.green("✓")
        );
    }

    // 2. Definir perfil se fornecido
    if let Some(prof) = profile_override {
        let _ = set_active_profile(root, prof);
    }

    // 3. Montar / sincronizar GOST.md e rules.toml
    let _ = assemble_gost_specs(root);

    // 4. Garantir .gost/metadata.json
    let metadata_file = gost_dir.join("metadata.json");
    if !metadata_file.exists() {
        let repo_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("app");
        let site_type = if root.join("Cargo.toml").exists() {
            "binary"
        } else {
            "doc"
        };
        let initial_meta = serde_json::json!({
            "schema_version": 1,
            "slug": repo_name,
            "title": repo_name,
            "site_type": site_type,
            "tagline": format!("Repositório {}", repo_name),
            "summary": format!("Módulo {} do ecossistema Gosplan", repo_name),
            "abstract": format!("Projeto {} governado conforme a norma técnica GOST v3.0.", repo_name),
            "tags": ["gost", "tulipa"]
        });
        let _ = fs::write(
            &metadata_file,
            serde_json::to_string_pretty(&initial_meta).unwrap(),
        );
        println!("  {} Criado .gost/metadata.json inicial", theme.green("✓"));
    }

    // 5. Garantir .gost/delivery.toml declarativo inicial se ausente
    let delivery_file = gost_dir.join("delivery.toml");
    if !delivery_file.exists() {
        let initial_delivery = if root.join("Cargo.toml").exists() {
            "# Manifesto declarativo de entrega contínua (CD)\n[binaries]\n# Distribuição de binários compilados na Banca R2\n"
        } else if root.join("site").exists() || root.join("index.html").exists() {
            "# Manifesto declarativo de entrega contínua (CD)\n[sites]\n# Publicação de site estático no Cloudflare Pages\n"
        } else {
            "# Manifesto declarativo de entrega contínua (CD)\n[artifacts]\n# Distribuição de artefatos genéricos no Cloudflare R2\n"
        };
        let _ = fs::write(&delivery_file, initial_delivery);
        println!(
            "  {} Criado .gost/delivery.toml declarativo inicial",
            theme.green("✓")
        );
    }

    println!(
        "{} Especificação GOST v3.0 configurada com sucesso em .gost/",
        theme.green("✓")
    );
    Ok(())
}

pub fn handle_profile_upgrade(
    root: &Path,
    args: &crate::cli::ProfileUpgradeArgs,
    theme: &Theme,
) -> Result<(), String> {
    println!(
        "{} Iniciando resolução linear de upgrade de perfil...",
        theme.cyan("★ [GOSPLAN]")
    );

    let plan = resolve_upgrade_chain(root, args.target.as_deref(), args.dry_run, None)
        .map_err(|e| format!("Falha no upgrade: {}", e))?;

    if plan.chain.len() <= 1 && plan.current_version == plan.target_version {
        println!(
            "{} Perfil já está na versão mais recente: {}",
            theme.green("✓"),
            theme.cyan(&plan.target_version)
        );
        return Ok(());
    }

    println!("  Perfil             : {}", theme.cyan(&plan.standard));
    println!(
        "  Versão Instalada   : {}",
        theme.yellow(&plan.current_version)
    );
    println!(
        "  Versão Alvo        : {}",
        theme.green(&plan.target_version)
    );
    println!(
        "  Cadeia de Migração : {}",
        theme.cyan(&plan.chain.join(" ➔ "))
    );
    if plan.breaking {
        println!(
            "  {}",
            theme.red_bold(
                "⚠ Atenção: Esta atualização contém quebras de contrato (breaking changes)!"
            )
        );
    }

    if args.dry_run {
        println!();
        println!(
            "{} Modo dry-run ativado. Nenhuma alteração foi persistida no disco.",
            theme.yellow("•")
        );
    } else {
        println!();
        println!(
            "{} Relatório de migração gerado com sucesso em {}",
            theme.green("✓"),
            theme.cyan(".gost/UPGRADE.md")
        );
        println!(
            "{} Configuração .rules/profile.toml atualizada para a versão {}",
            theme.green("✓"),
            theme.cyan(&plan.target_version)
        );
        println!(
            "{} Regras normativas remontadas para o novo perfil.",
            theme.green("✓")
        );
    }

    Ok(())
}

pub fn parse_upgrade_frontmatter(content: &str) -> io::Result<(UpgradeFrontmatter, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "UPGRADE.md deve iniciar com frontmatter delimitado por ---",
        ));
    }
    let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Frontmatter YAML malformado em UPGRADE.md",
        ));
    }
    let fm_str = parts[1];
    let body = parts[2].trim().to_string();

    let mut standard = None;
    let mut tier = None;
    let mut version = None;
    let mut replaces = None;
    let mut valid_since = None;
    let mut breaking = false;

    for line in fm_str.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("standard:") {
            standard = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(rest) = l.strip_prefix("tier:") {
            tier = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(rest) = l.strip_prefix("version:") {
            version = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(rest) = l.strip_prefix("replaces:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            if v != "null" && !v.is_empty() && v != "~" {
                replaces = Some(v.to_string());
            }
        } else if let Some(rest) = l.strip_prefix("valid_since:") {
            valid_since = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(rest) = l.strip_prefix("breaking:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            breaking = v == "true" || v == "yes" || v == "1";
        }
    }

    let standard = standard.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "Campo 'standard' ausente em UPGRADE.md",
        )
    })?;
    let tier = tier.unwrap_or_else(|| "operational".to_string());
    let version = version.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "Campo 'version' ausente em UPGRADE.md",
        )
    })?;
    let valid_since = valid_since.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            "Campo 'valid_since' ausente em UPGRADE.md",
        )
    })?;

    Ok((
        UpgradeFrontmatter {
            standard,
            tier,
            version,
            replaces,
            valid_since,
            breaking,
        },
        body,
    ))
}

pub fn is_version_below(current: &str, valid_since: &str) -> bool {
    let clean_cur = current.trim().trim_start_matches('v');
    let clean_valid = valid_since.trim().trim_start_matches('v');
    if let (Ok(v_cur), Ok(v_valid)) = (
        semver::Version::parse(clean_cur),
        semver::Version::parse(clean_valid),
    ) {
        v_cur < v_valid
    } else {
        clean_cur < clean_valid
    }
}

pub fn fetch_upgrade_file(base_url: &str, version: &str) -> io::Result<String> {
    let clean_base = base_url.trim_end_matches('/');
    let url = format!("{}/{}/UPGRADE.md", clean_base, version);
    if url.starts_with("http://") || url.starts_with("https://") {
        let resp = ureq::get(&url).call().map_err(|e| {
            Error::new(
                ErrorKind::NotFound,
                format!("Falha ao obter {}: {}", url, e),
            )
        })?;
        resp.into_body().read_to_string().map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Falha ao ler resposta de {}: {}", url, e),
            )
        })
    } else {
        let p = Path::new(&url);
        fs::read_to_string(p).map_err(|e| {
            Error::new(
                ErrorKind::NotFound,
                format!("Arquivo local {} não encontrado: {}", url, e),
            )
        })
    }
}

pub fn resolve_upgrade_chain(
    root: &Path,
    target_version: Option<&str>,
    dry_run: bool,
    mock_target_content: Option<&str>,
) -> io::Result<UpgradePlan> {
    let mut cfg = read_profile_config(root)?;
    let current_version = cfg.version.clone();
    let base_profile_url = if let Some(base) = cfg.url.strip_suffix("/profile.md") {
        base.to_string()
    } else {
        cfg.url.trim_end_matches('/').to_string()
    };

    let target_ver = match target_version {
        Some(t) => t.to_string(),
        None => fetch_latest_version_from_catalog(&base_profile_url).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "Não foi possível determinar versão latest em {}/versions.json",
                    base_profile_url
                ),
            )
        })?,
    };

    if current_version == target_ver {
        return Ok(UpgradePlan {
            standard: cfg.url.clone(),
            current_version: current_version.clone(),
            target_version: target_ver,
            chain: vec![current_version],
            combined_delta: "# Já atualizado na versão mais recente.\n".to_string(),
            breaking: false,
        });
    }

    // Baixa UPGRADE.md do alvo
    let target_content = match mock_target_content {
        Some(c) => c.to_string(),
        None => fetch_upgrade_file(&base_profile_url, &target_ver)?,
    };

    let (target_fm, target_body) = parse_upgrade_frontmatter(&target_content)?;

    // Validação Fail-Fast de valid_since
    if is_version_below(&current_version, &target_fm.valid_since) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "[FAIL-FAST] ERR_UPGRADE_BELOW_VALID_SINCE: Versão instalada ({}) é inferior ao piso valid_since ({}) exigido pela versão alvo {}. Migre previamente para a baseline {}.",
                current_version, target_fm.valid_since, target_ver, target_fm.valid_since
            ),
        ));
    }

    let mut deltas = Vec::new();
    let mut chain = Vec::new();
    let mut any_breaking = target_fm.breaking;

    deltas.push((target_ver.clone(), target_body));
    chain.push(target_ver.clone());

    let mut next_replaces = target_fm.replaces;

    while let Some(rep) = next_replaces {
        if rep == current_version || is_version_below(&rep, &current_version) {
            break;
        }
        let link_content = fetch_upgrade_file(&base_profile_url, &rep)?;
        let (link_fm, link_body) = parse_upgrade_frontmatter(&link_content)?;
        if link_fm.breaking {
            any_breaking = true;
        }
        deltas.push((rep.clone(), link_body));
        chain.push(rep.clone());
        next_replaces = link_fm.replaces;
    }

    // Inverte para ficar em ordem cronológica ascendente
    deltas.reverse();
    chain.reverse();

    let mut full_chain = vec![current_version.clone()];
    for c in &chain {
        if c != &current_version {
            full_chain.push(c.clone());
        }
    }

    let mut combined_delta = format!(
        "# Relatório Consolidado de Migração (GOST DD-TDD)\n\n\
         - **Perfil:** `{}`\n\
         - **Baseline Instalada:** `{}`\n\
         - **Versão Alvo:** `{}`\n\
         - **Cadeia Percorrida:** {}\n\
         - **Quebras de Contrato:** {}\n\n---\n\n",
        target_fm.standard,
        current_version,
        target_ver,
        full_chain.join(" ➔ "),
        if any_breaking {
            "Sim (atenção redobrada requerida)"
        } else {
            "Não"
        }
    );

    for (v, body) in &deltas {
        combined_delta.push_str(&format!(
            "## Delta de Migração para {}\n\n{}\n\n---\n\n",
            v, body
        ));
    }

    let plan = UpgradePlan {
        standard: target_fm.standard,
        current_version,
        target_version: target_ver.clone(),
        chain: full_chain,
        combined_delta: combined_delta.clone(),
        breaking: any_breaking,
    };

    if !dry_run {
        let gost_dir = root.join(".gost");
        fs::create_dir_all(&gost_dir)?;
        fs::write(gost_dir.join("UPGRADE.md"), &combined_delta)?;

        cfg.version = target_ver;
        write_profile_config(root, &cfg)?;

        // Remonta as regras
        let _ = assemble_rules_specs(root);
    }

    Ok(plan)
}
