//! CCCP Catalog Manager & Repository Resolver
//! Loads catalog.json and resolves target repositories by slug, name, or interactive picker.

use crate::forges::{ForgeType, RepoForgeInfo};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CatalogEntry {
    pub name: String,
    #[serde(default)]
    pub forge: String,
    pub slug: String,
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
}

impl fmt::Display for CatalogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.description.is_empty() {
            write!(f, "{} ({}/{})", self.name, self.forge, self.slug)
        } else {
            write!(f, "{} ({}) - {}", self.name, self.slug, self.description)
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
            _ => (ForgeType::Codeberg, "codeberg.org".to_string()),
        };

        RepoForgeInfo {
            provider,
            host,
            owner,
            repo,
        }
    }
}

fn expand_tilde(path: &str) -> PathBuf {
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
            if let Ok(entries) = serde_json::from_str::<Vec<CatalogEntry>>(&content) {
                return entries;
            }
        }
    }
    Vec::new()
}

/// Resolve target repository from user input (e.g. 'cccp', 'owner/repo', or catalog match)
pub fn resolve_target_repo(
    input: &str,
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Option<RepoForgeInfo> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Full remote URL
    if clean.contains("://") || clean.starts_with("git@") {
        return crate::forges::parse_remote_url(clean);
    }

    let catalog = load_catalog(config, start_dir);

    // 2. Exact match on slug (e.g. 'lumenpink/cccp')
    if let Some(entry) = catalog.iter().find(|e| e.slug.eq_ignore_ascii_case(clean)) {
        return Some(entry.to_forge_info());
    }

    // 3. Exact match on name (e.g. 'cccp')
    if let Some(entry) = catalog.iter().find(|e| e.name.eq_ignore_ascii_case(clean)) {
        return Some(entry.to_forge_info());
    }

    // 4. If formatted as owner/repo, infer codeberg as default forge
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some(RepoForgeInfo {
                provider: ForgeType::Codeberg,
                host: "codeberg.org".to_string(),
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            });
        }
    }

    None
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
            _ => ForgeType::Codeberg,
        };
        return Ok(RepoForgeInfo {
            provider,
            host,
            owner: parts[1].to_string(),
            repo: parts[2].trim_end_matches(".git").to_string(),
        });
    }

    // 3. Short slug (e.g. 'owner/repo') -> default to Codeberg
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        return Ok(RepoForgeInfo {
            provider: ForgeType::Codeberg,
            host: "codeberg.org".to_string(),
            owner: parts[0].to_string(),
            repo: parts[1].trim_end_matches(".git").to_string(),
        });
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
    let save_path = find_catalog_path(config, None).unwrap_or_else(|| get_catalog_save_path(config));
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create directory '{}': {}",
                parent.display(),
                e
            )
        })?;
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

/// Add or update a remote repository in the catalog without requiring a prior local clone
pub fn add_remote_repo(
    repo_input: &str,
    name_override: Option<&str>,
    desc_override: Option<&str>,
    config: Option<&crate::config::Config>,
) -> Result<(CatalogEntry, PathBuf, bool), String> {
    let info = parse_repo_identifier(repo_input)?;
    let mut catalog = load_catalog(config, None);

    let forge_name = match info.provider {
        ForgeType::GitHub => "github",
        ForgeType::GitLab => "gitlab",
        ForgeType::Codeberg | ForgeType::Forgejo => {
            if info.host.eq_ignore_ascii_case("codeberg.org") {
                "codeberg"
            } else {
                "forgejo"
            }
        }
    }
    .to_string();

    let slug = format!("{}/{}", info.owner, info.repo);

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
        entry = existing.clone();
        is_updated = true;
    } else {
        entry = CatalogEntry {
            name: entry_name,
            forge: forge_name,
            slug,
            description: entry_desc,
            topics: remote_meta.topics,
            license: remote_meta.license,
            website: remote_meta.website,
            is_upstream: false,
        };
        catalog.push(entry.clone());
        is_updated = false;
    }

    let save_path = save_catalog(&catalog, config)?;
    Ok((entry, save_path, is_updated))
}

