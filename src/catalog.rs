//! CCCP Catalog Manager & Repository Resolver
//! Loads catalog.json and resolves target repositories by slug, name, or interactive picker.

use crate::forges::{ForgeType, RepoForgeInfo};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogEntry {
    pub name: String,
    #[serde(default)]
    pub forge: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
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

/// Find catalog.json in standard hierarchical locations
pub fn find_catalog_path(start_dir: Option<&Path>) -> Option<PathBuf> {
    // 1. Current or specified dir
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

    // 2. $HOME/src/catalog.json
    if let Ok(home) = std::env::var("HOME") {
        let src_cat = PathBuf::from(&home).join("src").join("catalog.json");
        if src_cat.is_file() {
            return Some(src_cat);
        }
        let config_cat = PathBuf::from(&home)
            .join(".config")
            .join("cccp")
            .join("catalog.json");
        if config_cat.is_file() {
            return Some(config_cat);
        }
    }

    None
}

/// Load catalog entries from disk
pub fn load_catalog(start_dir: Option<&Path>) -> Vec<CatalogEntry> {
    if let Some(path) = find_catalog_path(start_dir) {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str::<Vec<CatalogEntry>>(&content) {
                return entries;
            }
        }
    }
    Vec::new()
}

/// Resolve target repository from user input (e.g. 'cccp', 'owner/repo', or catalog match)
pub fn resolve_target_repo(input: &str, start_dir: Option<&Path>) -> Option<RepoForgeInfo> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Full remote URL
    if clean.contains("://") || clean.starts_with("git@") {
        return crate::forges::parse_remote_url(clean);
    }

    let catalog = load_catalog(start_dir);

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

    // 5. Check local directory in $HOME/src/<clean>
    if let Ok(home) = std::env::var("HOME") {
        let local_dir = PathBuf::from(&home).join("src").join(clean);
        if let Ok(repo) = crate::git::GitRepo::open(Some(&local_dir)) {
            if let Ok(info) = crate::forges::detect_forge_from_repo(&repo) {
                return Some(info);
            }
        }
    }

    None
}
