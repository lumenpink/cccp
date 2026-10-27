//! CCCP Catalog Manager & Repository Resolver
//! Loads catalog.json and resolves target repositories by slug, name, or interactive picker.

use crate::forges::{ForgeType, RepoForgeInfo};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CiPipelineStatus {
    pub name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(from = "RawCatalogEntry")]
pub struct CatalogEntry {
    pub name: String,
    #[serde(default)]
    pub forge: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub is_upstream: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_tickets: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ci_status: Vec<CiPipelineStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_host: Option<String>,
}

#[derive(Deserialize)]
struct RawCatalogEntry {
    pub name: String,
    #[serde(default)]
    pub forge: String,
    pub slug: String,
    #[serde(default)]
    pub alias: Option<StringOrVec>,
    #[serde(default)]
    pub aliases: Option<StringOrVec>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub is_upstream: bool,
    #[serde(default)]
    pub open_tickets: Option<usize>,
    #[serde(default)]
    pub tickets: Option<usize>,
    #[serde(default)]
    pub ci_status: Vec<CiPipelineStatus>,
    #[serde(default)]
    pub canonical_url: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub server_host: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

impl From<RawCatalogEntry> for CatalogEntry {
    fn from(raw: RawCatalogEntry) -> Self {
        let mut aliases = Vec::new();
        let add_str_or_vec = |dest: &mut Vec<String>, sov: StringOrVec| match sov {
            StringOrVec::Single(s) => {
                let trimmed = s.trim().to_string();
                if !trimmed.is_empty() && !dest.contains(&trimmed) {
                    dest.push(trimmed);
                }
            }
            StringOrVec::Multiple(vec) => {
                for s in vec {
                    let trimmed = s.trim().to_string();
                    if !trimmed.is_empty() && !dest.contains(&trimmed) {
                        dest.push(trimmed);
                    }
                }
            }
        };

        if let Some(a) = raw.alias {
            add_str_or_vec(&mut aliases, a);
        }
        if let Some(a) = raw.aliases {
            add_str_or_vec(&mut aliases, a);
        }

        let canonical = raw.canonical_url.or(raw.url);
        let host = raw.server_host.or(raw.host);

        let mut entry = CatalogEntry {
            name: raw.name,
            forge: raw.forge,
            slug: raw.slug,
            aliases,
            description: raw.description,
            topics: raw.topics,
            license: raw.license,
            website: raw.website,
            is_upstream: raw.is_upstream,
            open_tickets: raw.open_tickets.or(raw.tickets),
            ci_status: raw.ci_status,
            canonical_url: canonical,
            server_host: host,
        };
        enrich_catalog_entry(&mut entry, None);
        entry
    }
}

/// Extracts the domain prefix according to Soviet Gosplan rules:
/// - If first part is "git" and there are more parts (e.g. git.subversao.su), uses the second part ("subversao").
/// - Otherwise uses the first part (e.g. gambi.2lp.in -> "gambi", codeberg.org -> "codeberg").
pub fn extract_domain_prefix(host: &str) -> Option<String> {
    let clean = host.split(':').next().unwrap_or(host).trim();
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }
    if parts[0].eq_ignore_ascii_case("git") && parts.len() > 1 {
        Some(parts[1].to_ascii_lowercase())
    } else {
        Some(parts[0].to_ascii_lowercase())
    }
}

/// Enriches a CatalogEntry with canonical URL, server host, domain-based aliases,
/// and custom aliases from Config.
pub fn enrich_catalog_entry(entry: &mut CatalogEntry, config: Option<&crate::config::Config>) {
    // 1. Resolve canonical_url
    if entry.canonical_url.is_none() && !entry.slug.is_empty() {
        let url = if entry.forge.starts_with("http://") || entry.forge.starts_with("https://") {
            format!(
                "{}/{}",
                entry.forge.trim_end_matches('/'),
                entry.slug.trim_start_matches('/')
            )
        } else if !entry.forge.is_empty() {
            format!(
                "https://{}/{}",
                entry.forge.trim_end_matches('/'),
                entry.slug.trim_start_matches('/')
            )
        } else {
            String::new()
        };
        if !url.is_empty() {
            entry.canonical_url = Some(url);
        }
    }

    // 2. Resolve server_host
    if entry.server_host.is_none() {
        let source = entry.canonical_url.as_deref().unwrap_or(&entry.forge);
        if !source.is_empty() {
            let host_part = if let Some(stripped) = source.strip_prefix("https://") {
                stripped
            } else if let Some(stripped) = source.strip_prefix("http://") {
                stripped
            } else if let Some(stripped) = source.strip_prefix("git@") {
                stripped.split(':').next().unwrap_or(stripped)
            } else {
                source
            };
            let host = host_part.split('/').next().unwrap_or(host_part);
            let clean_host = host.split(':').next().unwrap_or(host).trim();
            if !clean_host.is_empty() {
                entry.server_host = Some(clean_host.to_string());
            }
        }
    }

    // 3. Automatic domain-based aliases
    if let Some(ref host) = entry.server_host {
        if let Some(domain) = extract_domain_prefix(host) {
            let repo_name = entry.slug.split('/').nth(1).unwrap_or(&entry.slug);
            if !repo_name.is_empty() {
                let alias1 = format!("{}:{}", domain, repo_name);
                let alias2 = format!("{}/{}", domain, repo_name);
                if !entry.aliases.contains(&alias1) {
                    entry.aliases.push(alias1);
                }
                if !entry.aliases.contains(&alias2) {
                    entry.aliases.push(alias2);
                }
            }
        }
    }

    // 4. Custom aliases from config
    if let Some(cfg) = config {
        let mut to_add = Vec::new();
        if let Some(custom) = cfg.repo_aliases.get(&entry.slug) {
            to_add.extend(custom.clone());
        }
        if let Some(custom) = cfg.repo_aliases.get(&entry.name) {
            to_add.extend(custom.clone());
        }
        let repo_part = entry.slug.split('/').nth(1).unwrap_or(&entry.slug);
        if let Some(custom) = cfg.repo_aliases.get(repo_part) {
            to_add.extend(custom.clone());
        }
        for a in to_add {
            if !entry.aliases.contains(&a) {
                entry.aliases.push(a);
            }
        }
    }
}

impl fmt::Display for CatalogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alias_str = if self.aliases.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.aliases.join(", "))
        };
        if self.description.is_empty() {
            write!(
                f,
                "{}{} ({}/{})",
                self.name, alias_str, self.forge, self.slug
            )
        } else {
            write!(
                f,
                "{}{} ({}) - {}",
                self.name, alias_str, self.slug, self.description
            )
        }
    }
}

impl CatalogEntry {
    pub fn to_forge_info(&self) -> RepoForgeInfo {
        let parts: Vec<&str> = self.slug.split('/').collect();
        let (owner, repo) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("".to_string(), self.slug.clone())
        };

        let (provider, host) = match self.forge.to_ascii_lowercase().as_str() {
            "github" => (ForgeType::GitHub, "github.com".to_string()),
            "gitlab" => (ForgeType::GitLab, "gitlab.com".to_string()),
            "tildegit" | "tildegit.org" => (ForgeType::Tildegit, "tildegit.org".to_string()),
            _ => (ForgeType::Codeberg, "codeberg.org".to_string()),
        };

        RepoForgeInfo::new(provider, host, owner, repo)
    }
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    } else if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(path)
}

/// Find catalog.json according to configuration and standard hierarchical locations
pub fn find_catalog_path(
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Option<PathBuf> {
    // 1. Explicit path in configuration (e.g. .cccprc or environment)
    if let Some(cfg) = config {
        if let Some(ref path_str) = cfg.catalog_file {
            return Some(expand_tilde(path_str));
        }
    }

    // 2. Direct environment variables: CCCP_CATALOG_FILE or CATALOG_FILE
    if let Ok(env_path) =
        std::env::var("CCCP_CATALOG_FILE").or_else(|_| std::env::var("CATALOG_FILE"))
    {
        let trimmed = env_path.trim();
        if !trimmed.is_empty() {
            return Some(expand_tilde(trimmed));
        }
    }

    // 3. User configuration directory: $XDG_CONFIG_HOME/cccp/catalog.json or ~/.config/cccp/catalog.json
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("cccp").join("catalog.json");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(&home)
            .join(".config")
            .join("cccp")
            .join("catalog.json");
        if p.is_file() {
            return Some(p);
        }
    }

    // 4. Current directory or specified starting directory
    if let Some(d) = start_dir {
        let p = d.join("catalog.json");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(cur) = std::env::current_dir() {
        let p = cur.join("catalog.json");
        if p.is_file() {
            return Some(p);
        }
        if let Some(parent) = cur.parent() {
            let p = parent.join("catalog.json");
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // 5. Fallback for legacy ~/src/catalog.json if it exists
    if let Ok(home) = std::env::var("HOME") {
        let src_cat = PathBuf::from(&home).join("src").join("catalog.json");
        if src_cat.is_file() {
            return Some(src_cat);
        }
    }

    None
}

/// Determine where new or updated catalog.json should be saved
pub fn get_catalog_save_path(config: Option<&crate::config::Config>) -> PathBuf {
    if let Some(cfg) = config {
        if let Some(ref path_str) = cfg.catalog_file {
            return expand_tilde(path_str);
        }
    }
    if let Ok(env_path) =
        std::env::var("CCCP_CATALOG_FILE").or_else(|_| std::env::var("CATALOG_FILE"))
    {
        return expand_tilde(env_path.trim());
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let dir = PathBuf::from(xdg).join("cccp");
        let _ = fs::create_dir_all(&dir);
        return dir.join("catalog.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        let dir = PathBuf::from(&home).join(".config").join("cccp");
        let _ = fs::create_dir_all(&dir);
        return dir.join("catalog.json");
    }
    PathBuf::from("catalog.json")
}

/// Load catalog entries from disk using configured location
pub fn load_catalog(
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Vec<CatalogEntry> {
    if let Some(path) = find_catalog_path(config, start_dir) {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(mut entries) = serde_json::from_str::<Vec<CatalogEntry>>(&content) {
                for entry in &mut entries {
                    enrich_catalog_entry(entry, config);
                }
                return entries;
            }
        }
    }
    Vec::new()
}

/// Resolves target repository forge information and optionally its local workspace path.
pub fn resolve_target_repo_workspace(
    input: &str,
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Option<(RepoForgeInfo, Option<PathBuf>)> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }

    let catalog = load_catalog(config, start_dir);

    // 1. Full remote URL
    if clean.contains("://") || clean.starts_with("git@") {
        if let Some(info) = crate::forges::parse_remote_url(clean) {
            let full_slug = format!("{}/{}", info.owner, info.repo);
            if let Some(entry) = catalog.iter().find(|e| e.slug.eq_ignore_ascii_case(&full_slug)) {
                let local = find_local_repo_path_with_config(entry, config);
                return Some((info, local));
            }
            return Some((info, None));
        }
    }

    // 2. Exact match on slug (e.g. 'lumenpink/cccp')
    if let Some(entry) = catalog.iter().find(|e| e.slug.eq_ignore_ascii_case(clean)) {
        let local = find_local_repo_path_with_config(entry, config);
        return Some((entry.to_forge_info(), local));
    }

    // 3. Exact match on name (e.g. 'cccp')
    if let Some(entry) = catalog.iter().find(|e| e.name.eq_ignore_ascii_case(clean)) {
        let local = find_local_repo_path_with_config(entry, config);
        return Some((entry.to_forge_info(), local));
    }

    // 4. Match on alias (e.g. 'app')
    let alias_matches: Vec<&CatalogEntry> = catalog
        .iter()
        .filter(|e| e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean)))
        .collect();

    if alias_matches.len() == 1 {
        let entry = alias_matches[0];
        let local = find_local_repo_path_with_config(entry, config);
        return Some((entry.to_forge_info(), local));
    } else if alias_matches.len() > 1 {
        let matched_names: Vec<String> = alias_matches
            .iter()
            .map(|e| format!("'{}' ({})", e.name, e.slug))
            .collect();
        eprintln!(
            "Warning: Ambiguous repository alias '{}' matches multiple catalog entries: [{}]. Disambiguate using full slug or name.",
            clean,
            matched_names.join(", ")
        );
        return None;
    }

    // 5. Local filesystem discovery in start_dir/<name>, ~/src/<name>, or current directory
    if let Some(base) = start_dir {
        let candidate = base.join(clean);
        if candidate.join(".git").exists() {
            if let Some(remote_url) = crate::git::get_remote_url(&candidate) {
                if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                    return Some((info, Some(candidate)));
                }
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let src_path = PathBuf::from(&home).join("src").join(clean);
        if src_path.join(".git").exists() {
            if let Some(remote_url) = crate::git::get_remote_url(&src_path) {
                if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                    return Some((info, Some(src_path)));
                }
            }
        }
    }
    if let Ok(cur) = std::env::current_dir() {
        if let Some(dir_name) = cur.file_name().and_then(|n| n.to_str()) {
            if dir_name.eq_ignore_ascii_case(clean) || clean == "." {
                if let Some(remote_url) = crate::git::get_remote_url(&cur) {
                    if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                        return Some((info, Some(cur)));
                    }
                }
            }
        }
        let candidate = cur.join(clean);
        if candidate.join(".git").exists() {
            if let Some(remote_url) = crate::git::get_remote_url(&candidate) {
                if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                    return Some((info, Some(candidate)));
                }
            }
        }
    }

    // 6. If formatted as owner/repo, infer codeberg as default forge
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((
                RepoForgeInfo::new(
                    ForgeType::Codeberg,
                    "codeberg.org".to_string(),
                    parts[0].to_string(),
                    parts[1].to_string(),
                ),
                None,
            ));
        }
    }

    None
}

/// Resolve target repository from user input (e.g. 'cccp', 'owner/repo', or catalog match)
pub fn resolve_target_repo(
    input: &str,
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Option<RepoForgeInfo> {
    resolve_target_repo_workspace(input, config, start_dir).map(|(info, _)| info)
}

/// Parse user repository identifier (URL or slug) into RepoForgeInfo
pub fn parse_repo_identifier(input: &str) -> Result<RepoForgeInfo, String> {
    let clean = input.trim();
    if clean.is_empty() {
        return Err("Repository identifier cannot be empty".to_string());
    }

    // 1. Full remote URL (https://... or git@...)
    if clean.contains("://") || clean.starts_with("git@") {
        return crate::forges::parse_remote_url(clean)
            .ok_or_else(|| format!("Invalid remote repository URL: '{}'", clean));
    }

    // 2. Slug with host prefix (e.g. 'codeberg.org/owner/repo', 'github.com/owner/repo')
    let parts: Vec<&str> = clean.split('/').collect();
    if parts.len() == 3 && (parts[0].contains('.') || parts[0].contains(':')) {
        let host = parts[0].to_string();
        let provider = match host.to_ascii_lowercase().as_str() {
            "github.com" => ForgeType::GitHub,
            "gitlab.com" => ForgeType::GitLab,
            "tildegit.org" => ForgeType::Tildegit,
            _ => ForgeType::Codeberg,
        };
        return Ok(RepoForgeInfo::new(
            provider,
            host,
            parts[1].to_string(),
            parts[2].trim_end_matches(".git").to_string(),
        ));
    }

    // 3. Short slug (e.g. 'owner/repo') -> default to Codeberg
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        return Ok(RepoForgeInfo::new(
            ForgeType::Codeberg,
            "codeberg.org".to_string(),
            parts[0].to_string(),
            parts[1].trim_end_matches(".git").to_string(),
        ));
    }

    // 4. Local repository in ~/src or current directory
    if let Ok(home) = std::env::var("HOME") {
        let src_path = PathBuf::from(&home).join("src").join(clean);
        if let Some(remote_url) = crate::git::get_remote_url(&src_path) {
            if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                return Ok(info);
            }
        }
    }
    if let Ok(cur) = std::env::current_dir() {
        let candidate = cur.join(clean);
        if let Some(remote_url) = crate::git::get_remote_url(&candidate) {
            if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                return Ok(info);
            }
        }
    }

    Err(format!(
        "Invalid repository format: '{}'. Expected 'owner/repo', 'host/owner/repo' or full git URL.",
        clean
    ))
}

/// Save catalog entries to disk at the designated configuration path
pub fn save_catalog(
    entries: &[CatalogEntry],
    config: Option<&crate::config::Config>,
) -> Result<PathBuf, String> {
    let save_path =
        find_catalog_path(config, None).unwrap_or_else(|| get_catalog_save_path(config));
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Failed to serialize catalog: {}", e))?;

    fs::write(&save_path, json).map_err(|e| {
        format!(
            "Failed to write catalog file '{}': {}",
            save_path.display(),
            e
        )
    })?;

    Ok(save_path)
}

/// Validate whether an alias collides with existing repository names, slugs, or other aliases.
pub fn validate_alias_collision(
    target_slug: &str,
    alias: &str,
    catalog: &[CatalogEntry],
) -> Result<(), String> {
    let clean = alias.trim();
    if clean.is_empty() {
        return Err("Apelido não pode ser vazio.".to_string());
    }

    for entry in catalog {
        if entry.slug.eq_ignore_ascii_case(target_slug) {
            if entry.name.eq_ignore_ascii_case(clean) {
                return Err(format!(
                    "Conflito: o apelido '{}' coincide com o nome do próprio repositório '{}'.",
                    clean, entry.name
                ));
            }
            continue;
        }

        // Collision with canonical name of another repo
        if entry.name.eq_ignore_ascii_case(clean) {
            return Err(format!(
                "Conflito de identificador: o apelido '{}' colide com o nome canônico do repositório '{}' ({}).",
                clean, entry.name, entry.slug
            ));
        }

        // Collision with slug of another repo
        if entry.slug.eq_ignore_ascii_case(clean) {
            return Err(format!(
                "Conflito de identificador: o apelido '{}' colide com o slug do repositório '{}'.",
                clean, entry.slug
            ));
        }

        // Collision with alias of another repo
        if entry.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean)) {
            return Err(format!(
                "Conflito de identificador: o apelido '{}' já está em uso pelo repositório '{}' ({}).",
                clean, entry.name, entry.slug
            ));
        }
    }

    Ok(())
}

/// Add one or more aliases to an existing catalog repository, validating collision safety.
pub fn add_repo_aliases(
    repo_identifier: &str,
    new_aliases: &[String],
    config: Option<&crate::config::Config>,
) -> Result<CatalogEntry, String> {
    let clean_id = repo_identifier.trim();
    let mut catalog = load_catalog(config, None);

    let idx = catalog
        .iter()
        .position(|e| {
            e.slug.eq_ignore_ascii_case(clean_id)
                || e.name.eq_ignore_ascii_case(clean_id)
                || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean_id))
        })
        .ok_or_else(|| format!("Repositório '{}' não encontrado no catálogo.", clean_id))?;

    let target_slug = catalog[idx].slug.clone();

    // Validate all new aliases
    for alias in new_aliases {
        validate_alias_collision(&target_slug, alias, &catalog)?;
    }

    // Add and deduplicate
    for alias in new_aliases {
        let clean_a = alias.trim();
        if !catalog[idx]
            .aliases
            .iter()
            .any(|a| a.eq_ignore_ascii_case(clean_a))
        {
            catalog[idx].aliases.push(clean_a.to_string());
        }
    }

    let updated_entry = catalog[idx].clone();
    save_catalog(&catalog, config)?;
    Ok(updated_entry)
}

/// Remove one or more aliases from an existing catalog repository.
pub fn remove_repo_aliases(
    repo_identifier: &str,
    aliases_to_remove: &[String],
    config: Option<&crate::config::Config>,
) -> Result<CatalogEntry, String> {
    let clean_id = repo_identifier.trim();
    let mut catalog = load_catalog(config, None);

    let idx = catalog
        .iter()
        .position(|e| {
            e.slug.eq_ignore_ascii_case(clean_id)
                || e.name.eq_ignore_ascii_case(clean_id)
                || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean_id))
        })
        .ok_or_else(|| format!("Repositório '{}' não encontrado no catálogo.", clean_id))?;

    for alias in aliases_to_remove {
        let clean_a = alias.trim();
        catalog[idx]
            .aliases
            .retain(|a| !a.eq_ignore_ascii_case(clean_a));
    }

    let updated_entry = catalog[idx].clone();
    save_catalog(&catalog, config)?;
    Ok(updated_entry)
}

/// Add or update a remote repository in the catalog without requiring a prior local clone
pub fn add_remote_repo(
    repo_input: &str,
    name_override: Option<&str>,
    desc_override: Option<&str>,
    aliases: &[String],
    config: Option<&crate::config::Config>,
) -> Result<(CatalogEntry, PathBuf, bool), String> {
    let info = parse_repo_identifier(repo_input)?;
    let mut catalog = load_catalog(config, None);

    let forge_name = info.forge_name().to_string();
    let slug = format!("{}/{}", info.owner, info.repo);

    // Validate aliases for collisions
    for alias in aliases {
        validate_alias_collision(&slug, alias, &catalog)?;
    }

    // Fetch remote metadata (graceful fallback if unavailable)
    let forge_client = crate::forges::ForgeClient::new();
    let remote_meta = forge_client.fetch_repo_metadata(&info).unwrap_or_default();

    let entry_name = name_override
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| info.repo.clone());

    let entry_desc = desc_override
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| remote_meta.description.clone());

    let is_updated;
    let entry;

    if let Some(existing) = catalog.iter_mut().find(|e| {
        e.slug.eq_ignore_ascii_case(&slug)
            && (e.forge.is_empty() || e.forge.eq_ignore_ascii_case(&forge_name))
    }) {
        existing.name = entry_name;
        existing.forge = forge_name;
        if !entry_desc.is_empty() || existing.description.is_empty() {
            existing.description = entry_desc;
        }
        if !remote_meta.topics.is_empty() {
            existing.topics = remote_meta.topics;
        }
        if !remote_meta.license.is_empty() {
            existing.license = remote_meta.license;
        }
        if !remote_meta.website.is_empty() {
            existing.website = remote_meta.website;
        }
        for alias in aliases {
            let clean_a = alias.trim();
            if !existing
                .aliases
                .iter()
                .any(|a| a.eq_ignore_ascii_case(clean_a))
            {
                existing.aliases.push(clean_a.to_string());
            }
        }
        entry = existing.clone();
        is_updated = true;
    } else {
        let mut entry_aliases = Vec::new();
        for alias in aliases {
            let clean_a = alias.trim();
            if !entry_aliases
                .iter()
                .any(|a: &String| a.eq_ignore_ascii_case(clean_a))
            {
                entry_aliases.push(clean_a.to_string());
            }
        }
        entry = CatalogEntry {
            name: entry_name,
            forge: forge_name,
            slug,
            aliases: entry_aliases,
            description: entry_desc,
            topics: remote_meta.topics,
            license: remote_meta.license,
            website: remote_meta.website,
            is_upstream: false,
            open_tickets: None,
            ci_status: Vec::new(),
            ..Default::default()
        };
        catalog.push(entry.clone());
        is_updated = false;
    }

    let save_path = save_catalog(&catalog, config)?;
    Ok((entry, save_path, is_updated))
}

/// Remove a repository from the catalog matching by slug, name, or alias (case-insensitive)
pub fn remove_repo_from_catalog(
    identifier: &str,
    config: Option<&crate::config::Config>,
) -> Result<(CatalogEntry, PathBuf), String> {
    let (entry, save_path, _) = remove_repo_from_catalog_ext(identifier, false, config)?;
    Ok((entry, save_path))
}

/// Remove a repository from the catalog, optionally deleting its local directory if clean
pub fn remove_repo_from_catalog_ext(
    identifier: &str,
    delete_dir: bool,
    config: Option<&crate::config::Config>,
) -> Result<(CatalogEntry, PathBuf, Option<PathBuf>), String> {
    let clean = identifier.trim();
    if clean.is_empty() {
        return Err("Repository identifier cannot be empty".to_string());
    }

    let mut catalog = load_catalog(config, None);
    let initial_len = catalog.len();

    let pos = catalog.iter().position(|e| {
        e.slug.eq_ignore_ascii_case(clean)
            || e.name.eq_ignore_ascii_case(clean)
            || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean))
    });

    let index = match pos {
        Some(idx) => idx,
        None => {
            return Err(format!(
                "Repository '{}' not found in catalog (checked name, slug, and aliases across {} repositories)",
                clean, initial_len
            ))
        }
    };

    let target_entry = &catalog[index];
    let local_path = find_local_repo_path_with_config(target_entry, config);

    let mut purged_path = None;
    if delete_dir {
        if let Some(ref path) = local_path {
            if path.exists() {
                // Strict cleanliness verification
                if let Some(status) = crate::git::inspect_local_status(path) {
                    if !status.is_clean || status.ahead > 0 {
                        return Err(format!(
                            "Cannot delete repository directory '{}': repository is not clean (modified/untracked files: {}, unpushed commits: {}). Entire operation aborted.",
                            path.display(),
                            status.modified_files,
                            status.ahead
                        ));
                    }
                } else {
                    return Err(format!(
                        "Cannot delete repository directory '{}': failed to inspect git status. Entire operation aborted.",
                        path.display()
                    ));
                }

                // If clean, remove directory
                fs::remove_dir_all(path).map_err(|e| {
                    format!(
                        "Failed to remove repository directory '{}': {}. Entire operation aborted.",
                        path.display(),
                        e
                    )
                })?;
                purged_path = Some(path.clone());
            }
        }
    }

    let removed = catalog.remove(index);
    let save_path = save_catalog(&catalog, config)?;
    Ok((removed, save_path, purged_path))
}

#[derive(Debug, Clone, Default)]
pub struct BatchAddResult {
    pub total_scanned: usize,
    pub added: Vec<CatalogEntry>,
    pub skipped_existing: Vec<String>,
    pub skipped_not_git_or_remote: Vec<String>,
    pub catalog_path: PathBuf,
}

/// Scan a source directory (e.g. ~/src) and register all git repositories into the catalog
pub fn scan_and_add_from_src(
    src_dir: &Path,
    config: Option<&crate::config::Config>,
) -> Result<BatchAddResult, String> {
    if !src_dir.is_dir() {
        return Err(format!(
            "Source directory does not exist or is not a directory: '{}'",
            src_dir.display()
        ));
    }

    let mut catalog = load_catalog(config, None);
    let forge_client = crate::forges::ForgeClient::new();
    let mut result = BatchAddResult::default();

    let read_dir = fs::read_dir(src_dir).map_err(|e| {
        format!(
            "Failed to read source directory '{}': {}",
            src_dir.display(),
            e
        )
    })?;

    let mut dir_entries: Vec<PathBuf> = read_dir
        .filter_map(|e| e.ok().map(|de| de.path()))
        .filter(|p| p.is_dir())
        .collect();

    // Sort alphabetically for deterministic ordering
    dir_entries.sort();

    for dir in dir_entries {
        result.total_scanned += 1;
        let dir_name = match dir.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Check if directory is a git repository and has a remote URL
        let remote_url = match crate::git::get_remote_url(&dir) {
            Some(u) => u,
            None => {
                result.skipped_not_git_or_remote.push(dir_name);
                continue;
            }
        };

        let info = match crate::forges::parse_remote_url(&remote_url) {
            Some(i) => i,
            None => {
                result.skipped_not_git_or_remote.push(dir_name);
                continue;
            }
        };

        let forge_name = info.forge_name().to_string();

        let slug = format!("{}/{}", info.owner, info.repo);

        // Check if already in catalog (by slug + forge or slug)
        if catalog.iter().any(|e| {
            e.slug.eq_ignore_ascii_case(&slug)
                && (e.forge.is_empty() || e.forge.eq_ignore_ascii_case(&forge_name))
        }) {
            result.skipped_existing.push(dir_name);
            continue;
        }

        // Fetch remote metadata (graceful fallback if unavailable)
        let remote_meta = forge_client.fetch_repo_metadata(&info).unwrap_or_default();

        let entry = CatalogEntry {
            name: dir_name,
            forge: forge_name,
            slug,
            aliases: Vec::new(),
            description: remote_meta.description,
            topics: remote_meta.topics,
            license: remote_meta.license,
            website: remote_meta.website,
            is_upstream: false,
            open_tickets: None,
            ci_status: Vec::new(),
            ..Default::default()
        };

        catalog.push(entry.clone());
        result.added.push(entry);
    }

    if !result.added.is_empty() {
        result.catalog_path = save_catalog(&catalog, config)?;
    } else {
        result.catalog_path =
            find_catalog_path(config, None).unwrap_or_else(|| get_catalog_save_path(config));
    }

    Ok(result)
}

/// Locate local clone path for a catalog entry if it exists on disk
pub fn find_local_repo_path(entry: &CatalogEntry) -> Option<PathBuf> {
    find_local_repo_path_with_config(entry, None)
}

/// Locate local clone path for a catalog entry, optionally checking paths relative to the catalog file
pub fn find_local_repo_path_with_config(
    entry: &CatalogEntry,
    config: Option<&crate::config::Config>,
) -> Option<PathBuf> {
    // 1. Current directory if matches slug, name, or any alias
    if let Ok(cur) = std::env::current_dir() {
        if cur.join(".git").exists() {
            if let Some(dir_name) = cur.file_name().and_then(|n| n.to_str()) {
                if dir_name.eq_ignore_ascii_case(&entry.name)
                    || entry
                        .aliases
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(dir_name))
                {
                    return Some(cur);
                }
            }
        }
    }

    // 2. Directory co-located with catalog.json if known
    if let Some(cat_path) = find_catalog_path(config, None) {
        if let Some(parent) = cat_path.parent() {
            let p1 = parent.join(&entry.name);
            if p1.join(".git").exists() || p1.join(".antigravity").exists() {
                return Some(p1);
            }
            let repo_part = entry.slug.split('/').nth(1).unwrap_or(&entry.slug);
            let p2 = parent.join(repo_part);
            if p2.join(".git").exists() || p2.join(".antigravity").exists() {
                return Some(p2);
            }
            for alias in &entry.aliases {
                let pa = parent.join(alias);
                if pa.join(".git").exists() || pa.join(".antigravity").exists() {
                    return Some(pa);
                }
            }
        }
    }

    // 3. ~/src/<entry.name>
    if let Ok(home) = std::env::var("HOME") {
        let p1 = PathBuf::from(&home).join("src").join(&entry.name);
        if p1.join(".git").exists() {
            return Some(p1);
        }

        // 4. ~/src/<repo> (from slug owner/repo)
        let repo_part = entry.slug.split('/').nth(1).unwrap_or(&entry.slug);
        let p2 = PathBuf::from(&home).join("src").join(repo_part);
        if p2.join(".git").exists() {
            return Some(p2);
        }

        // 5. ~/src/<alias>
        for alias in &entry.aliases {
            let pa = PathBuf::from(&home).join("src").join(alias);
            if pa.join(".git").exists() {
                return Some(pa);
            }
        }
    }

    None
}

/// Count open tickets in a local repository (.antigravity/tickets.md or tickets.md)
pub fn count_open_tickets(repo_dir: &Path) -> usize {
    let tickets_path = repo_dir.join(".antigravity").join("tickets.md");
    let fallback_path = repo_dir.join("tickets.md");

    let target = if tickets_path.is_file() {
        Some(tickets_path)
    } else if fallback_path.is_file() {
        Some(fallback_path)
    } else {
        None
    };

    if let Some(p) = target {
        if let Ok(content) = fs::read_to_string(p) {
            return content
                .lines()
                .filter(|line| line.starts_with("## [#"))
                .count();
        }
    }

    0
}

/// Extracts declared GOST version (e.g. "1.4.0") from the GOSTVER file in a repository directory
pub fn extract_gost_version(repo_dir: &Path) -> Option<String> {
    let gostver_path = repo_dir.join("GOSTVER");
    if !gostver_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(gostver_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.starts_with("GOST_VERSION") {
            continue;
        }
        if let Some((_, val)) = trimmed.split_once('=') {
            let clean = val.trim().trim_matches(|c| c == '"' || c == '\'');
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatalogRepoSummary {
    pub name: String,
    pub forge: String,
    pub slug: String,
    pub aliases: Vec<String>,
    pub ticket_count: usize,
    pub is_clean: Option<bool>, // None if remote
    pub unpushed: usize,
    pub status_label: String, // "clean", "clean [↑4]", "dirty", "dirty [↑2]", "remote"
    pub local_path: Option<PathBuf>,
    pub gost_version: Option<String>,
    pub ci_status: Vec<CiPipelineStatus>,
}

/// Gather live status (clean/dirty, unpushed commits, open tickets) for catalog entries
pub fn summarize_catalog(entries: &[CatalogEntry]) -> Vec<CatalogRepoSummary> {
    entries
        .iter()
        .map(|entry| {
            let local_path = find_local_repo_path(entry);
            let (ticket_count, is_clean, unpushed, status_label) =
                if let Some(ref path) = local_path {
                    let local_tickets = count_open_tickets(path);
                    let tickets = if local_tickets > 0 {
                        local_tickets
                    } else {
                        entry.open_tickets.unwrap_or(0)
                    };
                    if let Some(st) = crate::git::inspect_local_status(path) {
                        let label = match (st.is_clean, st.ahead, st.modified_files) {
                            (true, 0, _) => "clean".to_string(),
                            (true, n, _) => format!("clean [↑{}]", n),
                            (false, 0, m) => {
                                if m > 0 {
                                    format!("dirty [{}*]", m)
                                } else {
                                    "dirty".to_string()
                                }
                            }
                            (false, n, m) => {
                                if m > 0 {
                                    format!("dirty [{}* ↑{}]", m, n)
                                } else {
                                    format!("dirty [↑{}]", n)
                                }
                            }
                        };
                        (tickets, Some(st.is_clean), st.ahead, label)
                    } else {
                        (tickets, None, 0, "unknown".to_string())
                    }
                } else {
                    (
                        entry.open_tickets.unwrap_or(0),
                        None,
                        0,
                        "remote".to_string(),
                    )
                };

            let gost_version = local_path.as_deref().and_then(extract_gost_version);

            let ci_status = if !entry.ci_status.is_empty() {
                entry.ci_status.clone()
            } else if let Some(ref path) = local_path {
                let ci_file = path.join(".antigravity").join("ci_status.json");
                if ci_file.is_file() {
                    std::fs::read_to_string(&ci_file)
                        .ok()
                        .and_then(|c| serde_json::from_str::<Vec<CiPipelineStatus>>(&c).ok())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            CatalogRepoSummary {
                name: entry.name.clone(),
                forge: entry.forge.clone(),
                slug: entry.slug.clone(),
                aliases: entry.aliases.clone(),
                ticket_count,
                is_clean,
                unpushed,
                status_label,
                local_path,
                gost_version,
                ci_status,
            }
        })
        .collect()
}

/// Helper to summarize CI pipeline execution state into a short label and styling key
pub fn format_ci_summary(ci: &[CiPipelineStatus]) -> (String, &'static str) {
    if ci.is_empty() {
        return ("-".to_string(), "dim");
    }
    let total = ci.len();
    let failed = ci
        .iter()
        .filter(|p| {
            let st = p.status.to_ascii_lowercase();
            st == "failure" || st == "fail" || st == "error"
        })
        .count();
    let running = ci
        .iter()
        .filter(|p| {
            let st = p.status.to_ascii_lowercase();
            st == "pending" || st == "running" || st == "waiting" || st == "progress"
        })
        .count();

    if failed > 0 {
        let text = if total > 1 {
            format!("✗ fail ({}/{})", failed, total)
        } else {
            "✗ fail".to_string()
        };
        (text, "red")
    } else if running > 0 {
        let text = if total > 1 {
            format!("⏳ pending ({}/{})", running, total)
        } else {
            "⏳ pending".to_string()
        };
        (text, "yellow")
    } else {
        let text = if total > 1 {
            format!("✓ pass ({})", total)
        } else {
            "✓ pass".to_string()
        };
        (text, "green")
    }
}

fn ci_rank(status: &[CiPipelineStatus]) -> usize {
    if status.is_empty() {
        return 3;
    }
    if status.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "failure" || st == "fail" || st == "error"
    }) {
        return 0;
    }
    if status.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "pending" || st == "running" || st == "waiting" || st == "progress"
    }) {
        return 1;
    }
    2
}

/// Format expanded CI pipeline execution details for display under catalog list
pub fn format_catalog_ci_details(
    summaries: &[CatalogRepoSummary],
    theme: &crate::cultural::Theme,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n{} CI Pipeline Details:\n",
        theme.gold_bold("★")
    ));
    let mut any_ci = false;
    for s in summaries {
        if s.ci_status.is_empty() {
            continue;
        }
        any_ci = true;
        out.push_str(&format!(
            "\n  {} ({}) [{}]:\n",
            theme.bold(&s.name),
            theme.cyan(&s.slug),
            theme.dim(&s.forge)
        ));
        for p in &s.ci_status {
            let st = p.status.to_ascii_lowercase();
            let (icon, styled_status) = if st == "success" || st == "pass" {
                ("✓", theme.green(&p.status))
            } else if st == "failure" || st == "fail" || st == "error" {
                ("✗", theme.red_bold(&p.status))
            } else if st == "pending" || st == "running" || st == "waiting" {
                ("⏳", theme.yellow(&p.status))
            } else {
                ("•", theme.dim(&p.status))
            };

            let mut line = format!("    {} {} - {}", icon, styled_status, theme.bold(&p.name));
            if let Some(ref desc) = p.description {
                line.push_str(&format!(" ({})", desc));
            }
            if let Some(ref updated) = p.updated_at {
                line.push_str(&format!(" [{}]", theme.dim(updated)));
            }
            if let Some(ref url) = p.target_url {
                line.push_str(&format!(" -> {}", theme.dim(url)));
            }
            out.push_str(&format!("{}\n", line));
        }
    }
    if !any_ci {
        out.push_str(&format!(
            "  {}\n",
            theme.dim("No CI pipelines recorded for displayed repositories.")
        ));
    }
    out
}

/// Pretty-format catalog repository summaries into a Gosplan table
pub fn format_catalog_table(
    summaries: &[CatalogRepoSummary],
    theme: &crate::cultural::Theme,
) -> String {
    if summaries.is_empty() {
        return "No repositories registered in catalog.".to_string();
    }

    let home = std::env::var("HOME").ok();
    let format_path = |p: &Option<PathBuf>| -> String {
        match p {
            Some(path) => {
                let s = path.to_string_lossy();
                if let Some(ref h) = home {
                    if let Some(stripped) = s.strip_prefix(h) {
                        return format!("~{}", stripped);
                    }
                }
                s.to_string()
            }
            None => "-".to_string(),
        }
    };

    let h_repo = "REPOSITORY";
    let h_forge = "FORGE";
    let h_gost = "GOST";
    let h_ci = "CI";
    let h_tickets = "TICKETS";
    let h_status = "STATUS";
    let h_path = "LOCAL PATH";

    let mut w_repo = h_repo.chars().count();
    let mut w_forge = h_forge.chars().count();
    let mut w_gost = h_gost.chars().count();
    let mut w_ci = h_ci.chars().count();
    let mut w_tickets = h_tickets.chars().count();
    let mut w_status = h_status.chars().count();
    let mut w_path = h_path.chars().count();

    for s in summaries {
        let repo_display = if s.aliases.is_empty() {
            s.name.clone()
        } else {
            format!("{} [{}]", s.name, s.aliases.join(", "))
        };
        let gost_str = s.gost_version.as_deref().unwrap_or("-");
        let (ci_text, _) = format_ci_summary(&s.ci_status);
        w_repo = w_repo.max(repo_display.chars().count());
        w_forge = w_forge.max(s.forge.chars().count());
        w_gost = w_gost.max(gost_str.chars().count());
        w_ci = w_ci.max(ci_text.chars().count());
        w_tickets = w_tickets.max(s.ticket_count.to_string().chars().count());
        w_status = w_status.max(s.status_label.chars().count());
        w_path = w_path.max(format_path(&s.local_path).chars().count());
    }

    let mut out = String::new();

    // Top border
    out.push_str(&format!(
        "┌─{repo}─┬─{forge}─┬─{gost}─┬─{ci}─┬─{tickets}─┬─{status}─┬─{path}─┐\n",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
        gost = "─".repeat(w_gost),
        ci = "─".repeat(w_ci),
        tickets = "─".repeat(w_tickets),
        status = "─".repeat(w_status),
        path = "─".repeat(w_path)
    ));

    // Header row
    let pad_r = " ".repeat(w_repo.saturating_sub(h_repo.chars().count()));
    let pad_f = " ".repeat(w_forge.saturating_sub(h_forge.chars().count()));
    let pad_g = " ".repeat(w_gost.saturating_sub(h_gost.chars().count()));
    let pad_c = " ".repeat(w_ci.saturating_sub(h_ci.chars().count()));
    let pad_t = " ".repeat(w_tickets.saturating_sub(h_tickets.chars().count()));
    let pad_s = " ".repeat(w_status.saturating_sub(h_status.chars().count()));
    let pad_p = " ".repeat(w_path.saturating_sub(h_path.chars().count()));

    out.push_str(&format!(
        "│ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │\n",
        theme.bold(h_repo),
        pad_r,
        theme.bold(h_forge),
        pad_f,
        theme.bold(h_gost),
        pad_g,
        theme.bold(h_ci),
        pad_c,
        pad_t,
        theme.bold(h_tickets),
        theme.bold(h_status),
        pad_s,
        theme.bold(h_path),
        pad_p,
    ));

    // Separator
    out.push_str(&format!(
        "├─{repo}─┼─{forge}─┼─{gost}─┼─{ci}─┼─{tickets}─┼─{status}─┼─{path}─┤\n",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
        gost = "─".repeat(w_gost),
        ci = "─".repeat(w_ci),
        tickets = "─".repeat(w_tickets),
        status = "─".repeat(w_status),
        path = "─".repeat(w_path)
    ));

    // Rows
    for s in summaries {
        let repo_display = if s.aliases.is_empty() {
            s.name.clone()
        } else {
            format!("{} [{}]", s.name, s.aliases.join(", "))
        };
        let path_str = format_path(&s.local_path);
        let tickets_str = s.ticket_count.to_string();
        let gost_display = s.gost_version.as_deref().unwrap_or("-");
        let (ci_text, ci_style) = format_ci_summary(&s.ci_status);

        let pad_repo = " ".repeat(w_repo.saturating_sub(repo_display.chars().count()));
        let pad_forge = " ".repeat(w_forge.saturating_sub(s.forge.chars().count()));
        let pad_gost = " ".repeat(w_gost.saturating_sub(gost_display.chars().count()));
        let pad_ci = " ".repeat(w_ci.saturating_sub(ci_text.chars().count()));
        let pad_tickets = " ".repeat(w_tickets.saturating_sub(tickets_str.chars().count()));
        let pad_status = " ".repeat(w_status.saturating_sub(s.status_label.chars().count()));
        let pad_path = " ".repeat(w_path.saturating_sub(path_str.chars().count()));

        let styled_status = if s.status_label.starts_with("clean") {
            theme.green(&s.status_label)
        } else if s.status_label.starts_with("dirty") {
            theme.red_bold(&s.status_label)
        } else {
            theme.dim(&s.status_label)
        };

        let styled_tickets = if s.ticket_count > 0 {
            theme.gold_bold(&tickets_str)
        } else {
            theme.dim(&tickets_str)
        };

        let styled_ci = match ci_style {
            "green" => theme.green(&ci_text),
            "red" => theme.red_bold(&ci_text),
            "yellow" => theme.yellow(&ci_text),
            _ => theme.dim(&ci_text),
        };

        let styled_repo = theme.bold(&repo_display);
        let styled_forge = theme.cyan(&s.forge);
        let styled_gost = if s.gost_version.is_some() {
            theme.cyan(gost_display)
        } else {
            theme.dim(gost_display)
        };
        let styled_path = if path_str == "-" {
            theme.dim(&path_str)
        } else {
            path_str
        };

        out.push_str(&format!(
            "│ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │\n",
            styled_repo,
            pad_repo,
            styled_forge,
            pad_forge,
            styled_gost,
            pad_gost,
            styled_ci,
            pad_ci,
            pad_tickets,
            styled_tickets,
            styled_status,
            pad_status,
            styled_path,
            pad_path,
        ));
    }

    // Bottom border
    out.push_str(&format!(
        "└─{repo}─┴─{forge}─┴─{gost}─┴─{ci}─┴─{tickets}─┴─{status}─┴─{path}─┘",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
        gost = "─".repeat(w_gost),
        ci = "─".repeat(w_ci),
        tickets = "─".repeat(w_tickets),
        status = "─".repeat(w_status),
        path = "─".repeat(w_path)
    ));

    out
}

/// Filter and sort catalog repository summaries according to user criteria
pub fn filter_and_sort_summaries(
    summaries: &mut Vec<CatalogRepoSummary>,
    filter: &crate::cli::CatalogFilterArgs,
) {
    // 0. Filter by direct flags: --dirty and --unpushed
    if filter.dirty {
        summaries.retain(|s| s.is_clean == Some(false));
    }
    if filter.unpushed {
        summaries.retain(|s| s.is_clean == Some(true) && s.unpushed > 0);
    }

    // 1. Filter by status
    if let Some(ref st_filter) = filter.status {
        let clean_st = st_filter.trim().to_ascii_lowercase();
        summaries.retain(|s| match clean_st.as_str() {
            "clean" => s.status_label.starts_with("clean"),
            "dirty" => s.status_label.starts_with("dirty"),
            "unpushed" => s.unpushed > 0,
            "remote" => s.status_label == "remote" || s.is_clean.is_none(),
            other => s.status_label.to_ascii_lowercase().contains(other),
        });
    }

    // 2. Filter by minimum tickets
    if let Some(min_t) = filter.min_tickets {
        summaries.retain(|s| s.ticket_count >= min_t);
    }

    // 3. Filter by CI status
    if let Some(ref ci_filter) = filter.ci {
        let target = ci_filter.trim().to_ascii_lowercase();
        summaries.retain(|s| {
            let has_fail = s.ci_status.iter().any(|p| {
                let st = p.status.to_ascii_lowercase();
                st == "failure" || st == "fail" || st == "error"
            });
            let has_running = s.ci_status.iter().any(|p| {
                let st = p.status.to_ascii_lowercase();
                st == "pending" || st == "running" || st == "waiting" || st == "progress"
            });
            let is_all_pass = !s.ci_status.is_empty()
                && !has_fail
                && !has_running
                && s.ci_status.iter().all(|p| {
                    let st = p.status.to_ascii_lowercase();
                    st == "success" || st == "pass"
                });

            match target.as_str() {
                "pass" | "success" => is_all_pass,
                "fail" | "failure" => has_fail,
                "pending" | "running" => has_running && !has_fail,
                "none" => s.ci_status.is_empty(),
                other => s.ci_status.iter().any(|p| {
                    p.status.to_ascii_lowercase().contains(other)
                        || p.name.to_ascii_lowercase().contains(other)
                }),
            }
        });
    }

    // 4. Sort
    let sort_field = filter.sort.trim().to_ascii_lowercase();
    match sort_field.as_str() {
        "tickets" => {
            summaries.sort_by(|a, b| {
                b.ticket_count
                    .cmp(&a.ticket_count)
                    .then_with(|| a.name.cmp(&b.name))
            });
        }
        "status" => {
            summaries.sort_by(|a, b| {
                a.status_label
                    .cmp(&b.status_label)
                    .then_with(|| a.name.cmp(&b.name))
            });
        }
        "forge" => {
            summaries.sort_by(|a, b| a.forge.cmp(&b.forge).then_with(|| a.name.cmp(&b.name)));
        }
        "ci" => {
            summaries.sort_by(|a, b| {
                ci_rank(&a.ci_status)
                    .cmp(&ci_rank(&b.ci_status))
                    .then_with(|| a.name.cmp(&b.name))
            });
        }
        "gost" | "gostver" => {
            summaries.sort_by(|a, b| match (&a.gost_version, &b.gost_version) {
                (Some(va), Some(vb)) => va.cmp(vb).then_with(|| a.name.cmp(&b.name)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            });
        }
        _ => {
            // Default "name"
            summaries.sort_by_key(|a| a.name.to_ascii_lowercase());
        }
    }

    // 5. Reverse
    if filter.reverse {
        summaries.reverse();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatchPushReport {
    pub pushed: Vec<String>,
    pub dry_run: Vec<String>,
    pub skipped_dirty: Vec<String>,
    pub skipped_up_to_date: Vec<String>,
    pub skipped_remote: Vec<String>,
    pub failed: Vec<(String, String)>,
}

/// Executes safe batch push for clean repositories with pending unpushed commits
pub fn batch_push_clean_repos(
    summaries: &[CatalogRepoSummary],
    dry_run: bool,
    _theme: &crate::cultural::Theme,
) -> BatchPushReport {
    let mut report = BatchPushReport::default();

    for summary in summaries {
        let Some(ref path) = summary.local_path else {
            report.skipped_remote.push(summary.name.clone());
            continue;
        };

        if summary.is_clean.is_none() {
            report.skipped_remote.push(summary.name.clone());
            continue;
        }

        if summary.is_clean != Some(true) {
            report.skipped_dirty.push(summary.name.clone());
            continue;
        }

        if summary.unpushed == 0 {
            report.skipped_up_to_date.push(summary.name.clone());
            continue;
        }

        if dry_run {
            report.dry_run.push(summary.name.clone());
        } else {
            let output = std::process::Command::new("git")
                .arg("push")
                .env("CCCP_MAESTRO_ACTIVE", "1")
                .current_dir(path)
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    report.pushed.push(summary.name.clone());
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    let msg = if err.is_empty() {
                        "git push exited with failure".to_string()
                    } else {
                        err
                    };
                    report.failed.push((summary.name.clone(), msg));
                }
                Err(e) => {
                    report.failed.push((summary.name.clone(), e.to_string()));
                }
            }
        }
    }

    report
}

/// Gathers summaries and executes batch push for catalog entries
pub fn batch_push_catalog(
    entries: &[CatalogEntry],
    dry_run: bool,
    theme: &crate::cultural::Theme,
) -> BatchPushReport {
    let summaries = summarize_catalog(entries);
    batch_push_clean_repos(&summaries, dry_run, theme)
}

/// Pretty-formats the batch push report into a constructivist Gosplan summary
pub fn format_batch_push_report(
    report: &BatchPushReport,
    theme: &crate::cultural::Theme,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}\n",
        theme.gold_bold("★ Gosplan Pre-Push Batch Report ★")
    ));

    if !report.dry_run.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.cyan("✦ [DRY-RUN]"),
            theme.bold("Repositories eligible for push:"),
            report.dry_run.len()
        ));
        for name in &report.dry_run {
            out.push_str(&format!("  {} {}\n", theme.cyan("→"), name));
        }
    }

    if !report.pushed.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.green("✔"),
            theme.bold("Pushed successfully:"),
            report.pushed.len()
        ));
        for name in &report.pushed {
            out.push_str(&format!("  {} {}\n", theme.green("✓"), name));
        }
    }

    if !report.failed.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.red_bold("✖"),
            theme.bold("Push failed:"),
            report.failed.len()
        ));
        for (name, err) in &report.failed {
            out.push_str(&format!("  {} {}: {}\n", theme.red("✗"), name, err));
        }
    }

    if !report.skipped_dirty.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.yellow("⚠"),
            theme.bold("Skipped dirty working trees (uncommitted changes):"),
            report.skipped_dirty.len()
        ));
        for name in &report.skipped_dirty {
            out.push_str(&format!("  {} {}\n", theme.yellow("!"), name));
        }
    }

    if !report.skipped_up_to_date.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.dim("•"),
            theme.dim("Up-to-date (no unpushed commits):"),
            report.skipped_up_to_date.len()
        ));
        for name in &report.skipped_up_to_date {
            out.push_str(&format!("  {} {}\n", theme.dim("•"), theme.dim(name)));
        }
    }

    if !report.skipped_remote.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.dim("•"),
            theme.dim("Remote only / unmanaged locally:"),
            report.skipped_remote.len()
        ));
        for name in &report.skipped_remote {
            out.push_str(&format!("  {} {}\n", theme.dim("•"), theme.dim(name)));
        }
    }

    let total_active = report.dry_run.len() + report.pushed.len() + report.failed.len();
    if total_active == 0 && report.skipped_dirty.is_empty() {
        out.push_str(&format!(
            "\n{} All local repositories are up-to-date with remotes.\n",
            theme.green("✓")
        ));
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatchAssembleReport {
    pub assembled: Vec<String>,
    pub dry_run: Vec<String>,
    pub skipped_no_gostver: Vec<String>,
    pub skipped_remote: Vec<String>,
    pub failed: Vec<(String, String)>,
}

/// Executes batch GOST specification assembly for resolved repository paths
pub fn batch_assemble_gost_repos(
    repos: &[(&str, Option<&Path>)],
    dry_run: bool,
    _theme: &crate::cultural::Theme,
) -> BatchAssembleReport {
    let mut report = BatchAssembleReport::default();

    for (name, path_opt) in repos {
        let Some(path) = path_opt else {
            report.skipped_remote.push(name.to_string());
            continue;
        };

        if !path.exists() {
            report.skipped_remote.push(name.to_string());
            continue;
        }

        if !path.join("GOSTVER").is_file() {
            report.skipped_no_gostver.push(name.to_string());
            continue;
        }

        // Guard against dirty working tree in batch assemble
        if let Ok(st) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
        {
            if !st.stdout.is_empty() {
                report.failed.push((
                    name.to_string(),
                    "Working tree possui modificações pendentes".to_string(),
                ));
                continue;
            }
        }

        if dry_run {
            report.dry_run.push(name.to_string());
        } else {
            match crate::profile::assemble_gost_specs(path) {
                Ok(_) => {
                    report.assembled.push(name.to_string());
                }
                Err(e) => {
                    report.failed.push((name.to_string(), e.to_string()));
                }
            }
        }
    }

    report
}

/// Assembles GOST specification (.gost/GOST.md) for a single repository path,
/// enforcing clean working tree and supporting auto-commit.
pub fn assemble_repo(
    path: &Path,
    dry_run: bool,
    auto_commit: bool,
    _theme: &crate::cultural::Theme,
) -> Result<(), String> {
    if !path.exists() {
        return Err(format!(
            "Caminho do repositório não existe: {}",
            path.display()
        ));
    }

    // 1. Check clean working tree
    let status_out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Falha ao inspecionar status do git: {}", e))?;

    if !status_out.stdout.is_empty() {
        return Err("Repositório possui modificações pendentes na working tree. Faça commit ou stash antes do assemble.".to_string());
    }

    if dry_run {
        return Ok(());
    }

    // 2. Assemble specification
    crate::profile::assemble_gost_specs(path)
        .map_err(|e| format!("Falha no assemble da especificação GOST: {}", e))?;

    // 3. Auto-commit if requested
    if auto_commit {
        let diff_out = std::process::Command::new("git")
            .args(["status", "--porcelain", ".gost/GOST.md"])
            .current_dir(path)
            .output()
            .map_err(|e| format!("Falha ao verificar alterações em .gost/GOST.md: {}", e))?;

        if !diff_out.stdout.is_empty() {
            let add_status = std::process::Command::new("git")
                .args(["add", ".gost/GOST.md"])
                .current_dir(path)
                .status()
                .map_err(|e| format!("Falha ao adicionar .gost/GOST.md ao git stage: {}", e))?;

            if !add_status.success() {
                return Err("Falha ao adicionar .gost/GOST.md ao git stage".to_string());
            }

            let mut commit_cmd = std::process::Command::new("git");
            commit_cmd
                .args([
                    "commit",
                    "-m",
                    "docs(gost): atualizar especificacao gost compilada",
                ])
                .env("CCCP_MAESTRO_ACTIVE", "1")
                .current_dir(path);

            let commit_status = commit_cmd
                .status()
                .map_err(|e| format!("Falha ao criar commit de assemble GOST: {}", e))?;

            if !commit_status.success() {
                return Err("Falha ao executar commit de assemble GOST".to_string());
            }
        }
    }

    Ok(())
}

/// Assembles GOST specification for a repository located in the State Catalog by name, slug or alias.
pub fn assemble_catalog_repo(
    entries: &[CatalogEntry],
    target: &str,
    dry_run: bool,
    auto_commit: bool,
    config: Option<&crate::config::Config>,
    theme: &crate::cultural::Theme,
) -> Result<(), String> {
    let lower_target = target.to_lowercase();
    let entry = entries.iter().find(|e| {
        let repo_part = e.slug.split('/').nth(1).unwrap_or(&e.slug);
        e.slug.to_lowercase() == lower_target
            || repo_part.to_lowercase() == lower_target
            || e.name.to_lowercase() == lower_target
            || e.aliases.iter().any(|a| a.to_lowercase() == lower_target)
    });

    let Some(entry) = entry else {
        return Err(format!(
            "Repositório '{}' não encontrado no catálogo",
            target
        ));
    };

    if entry.is_upstream {
        return Err(format!(
            "Repositório '{}' está configurado como upstream e é isolado contra mutações locais.",
            entry.slug
        ));
    }

    let Some(local_path) = find_local_repo_path_with_config(entry, config) else {
        return Err(format!(
            "Repositório '{}' não possui workspace local clonado",
            target
        ));
    };

    assemble_repo(&local_path, dry_run, auto_commit, theme)
}

/// Executes batch GOST specification assembly for catalog repositories
pub fn batch_assemble_gost(
    entries: &[CatalogEntry],
    dry_run: bool,
    theme: &crate::cultural::Theme,
) -> BatchAssembleReport {
    let resolved: Vec<(String, Option<PathBuf>)> = entries
        .iter()
        .map(|e| {
            if e.is_upstream {
                (e.name.clone(), None)
            } else {
                (e.name.clone(), find_local_repo_path(e))
            }
        })
        .collect();
    let borrowed: Vec<(&str, Option<&Path>)> = resolved
        .iter()
        .map(|(n, p)| (n.as_str(), p.as_deref()))
        .collect();
    batch_assemble_gost_repos(&borrowed, dry_run, theme)
}

/// Formats the batch assemble report into a constructivist summary
pub fn format_batch_assemble_report(
    report: &BatchAssembleReport,
    theme: &crate::cultural::Theme,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}\n",
        theme.gold_bold("★ Gosplan Batch GOST Assembly Report ★")
    ));

    if !report.dry_run.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.cyan("✦ [DRY-RUN]"),
            theme.bold("Repositories eligible for GOST assembly:"),
            report.dry_run.len()
        ));
        for name in &report.dry_run {
            out.push_str(&format!("  {} {}\n", theme.cyan("→"), name));
        }
    }

    if !report.assembled.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.green("✔"),
            theme.bold("Assembled .gost/GOST.md successfully:"),
            report.assembled.len()
        ));
        for name in &report.assembled {
            out.push_str(&format!("  {} {}\n", theme.green("✓"), name));
        }
    }

    if !report.failed.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.red_bold("✖"),
            theme.bold("Failed to assemble GOST:"),
            report.failed.len()
        ));
        for (name, err) in &report.failed {
            out.push_str(&format!("  {} {}: {}\n", theme.red("✗"), name, err));
        }
    }

    if !report.skipped_no_gostver.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.dim("•"),
            theme.dim("Skipped (no GOSTVER defined):"),
            report.skipped_no_gostver.len()
        ));
        for name in &report.skipped_no_gostver {
            out.push_str(&format!("  {} {}\n", theme.dim("•"), theme.dim(name)));
        }
    }

    if !report.skipped_remote.is_empty() {
        out.push_str(&format!(
            "\n{} {} ({})\n",
            theme.dim("•"),
            theme.dim("Remote only / unmanaged locally:"),
            report.skipped_remote.len()
        ));
        for name in &report.skipped_remote {
            out.push_str(&format!("  {} {}\n", theme.dim("•"), theme.dim(name)));
        }
    }

    let total_active = report.dry_run.len() + report.assembled.len() + report.failed.len();
    if total_active == 0 && report.skipped_no_gostver.is_empty() && report.skipped_remote.is_empty()
    {
        out.push_str(&format!(
            "\n{} No catalog repositories found to assemble.\n",
            theme.green("✓")
        ));
    }

    out
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTicketsReport {
    pub updated: Vec<CatalogEntry>,
    pub failed: Vec<(String, String)>,
}

/// Update open ticket count for specified repositories or all repositories in catalog
pub fn update_catalog_tickets(
    target_repos: &[String],
    all: bool,
    config: Option<&crate::config::Config>,
) -> Result<UpdateTicketsReport, String> {
    let mut entries = load_catalog(config, None);
    if entries.is_empty() {
        return Err("Catalog is empty".to_string());
    }

    let client = crate::forges::ForgeClient::new();
    let mut report = UpdateTicketsReport::default();
    let mut modified = false;

    let mut target_indices: Vec<usize> = Vec::new();
    if all {
        target_indices = (0..entries.len()).collect();
    } else if !target_repos.is_empty() {
        for query in target_repos {
            let q_clean = query.trim().to_ascii_lowercase();
            let found = entries.iter().position(|e| {
                e.slug.to_ascii_lowercase() == q_clean
                    || e.name.to_ascii_lowercase() == q_clean
                    || e.aliases.iter().any(|a| a.to_ascii_lowercase() == q_clean)
            });
            if let Some(idx) = found {
                if !target_indices.contains(&idx) {
                    target_indices.push(idx);
                }
            } else {
                report.failed.push((
                    query.clone(),
                    format!("Repository '{}' not found in catalog", query),
                ));
            }
        }
    } else {
        // Tenta identificar se o repositório atual está no catálogo
        if let Ok(cur) = std::env::current_dir() {
            let found = entries.iter().position(|e| {
                if let Some(local) = find_local_repo_path_with_config(e, config) {
                    if let (Ok(c_can), Ok(l_can)) = (cur.canonicalize(), local.canonicalize()) {
                        return c_can == l_can;
                    }
                }
                false
            });
            if let Some(idx) = found {
                target_indices.push(idx);
            } else {
                return Err(
                    "No repository specified and current directory is not in catalog. Specify repositories or use --all."
                        .to_string(),
                );
            }
        } else {
            return Err("No repository specified. Specify repositories or use --all.".to_string());
        }
    }

    for idx in target_indices {
        let entry = &mut entries[idx];
        let forge_info = entry.to_forge_info();
        let mut ticket_count = None;

        // 1. Tenta buscar da forge remota se configurada
        if !forge_info.owner.is_empty() && !forge_info.repo.is_empty() {
            if let Ok(count) = client.fetch_open_issues_count(&forge_info) {
                ticket_count = Some(count);
            }
        }

        // 2. Se a forge remota falhou ou não retornou, tenta o clone local
        if ticket_count.is_none() {
            if let Some(path) = find_local_repo_path_with_config(entry, config) {
                let local_count = count_open_tickets(&path);
                if local_count > 0 || entry.open_tickets.is_none() {
                    ticket_count = Some(local_count);
                }
            }
        }

        // 3. Sincroniza status das esteiras de CI
        if !forge_info.owner.is_empty() && !forge_info.repo.is_empty() {
            if let Ok(ci) = client.fetch_ci_status(&forge_info, Some("main")) {
                if !ci.is_empty() {
                    entry.ci_status = ci;
                    modified = true;
                }
            }
        }

        if let Some(count) = ticket_count {
            entry.open_tickets = Some(count);
            report.updated.push(entry.clone());
            modified = true;
        } else if entry.open_tickets.is_some() {
            report.updated.push(entry.clone());
        } else {
            report.failed.push((
                entry.slug.clone(),
                "Could not retrieve tickets from remote forge or local clone".to_string(),
            ));
        }
    }

    if modified {
        save_catalog(&entries, config)?;
    }

    Ok(report)
}
