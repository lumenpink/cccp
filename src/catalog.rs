//! CCCP Catalog Manager & Repository Resolver
//! Loads catalog.json and resolves target repositories by slug, name, or interactive picker.

use crate::forges::{ForgeType, RepoForgeInfo};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

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

        CatalogEntry {
            name: raw.name,
            forge: raw.forge,
            slug: raw.slug,
            aliases,
            description: raw.description,
            topics: raw.topics,
            license: raw.license,
            website: raw.website,
            is_upstream: raw.is_upstream,
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

    // 4. Match on alias (e.g. 'app')
    let alias_matches: Vec<&CatalogEntry> = catalog
        .iter()
        .filter(|e| e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean)))
        .collect();

    if alias_matches.len() == 1 {
        return Some(alias_matches[0].to_forge_info());
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

    // 5. Local filesystem discovery in ~/src/<name>, start_dir/<name>, or current directory
    if let Ok(home) = std::env::var("HOME") {
        let src_path = PathBuf::from(&home).join("src").join(clean);
        if let Some(remote_url) = crate::git::get_remote_url(&src_path) {
            if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                return Some(info);
            }
        }
    }
    if let Some(base) = start_dir {
        let candidate = base.join(clean);
        if let Some(remote_url) = crate::git::get_remote_url(&candidate) {
            if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                return Some(info);
            }
        }
    }
    if let Ok(cur) = std::env::current_dir() {
        if let Some(dir_name) = cur.file_name().and_then(|n| n.to_str()) {
            if dir_name.eq_ignore_ascii_case(clean) || clean == "." {
                if let Some(remote_url) = crate::git::get_remote_url(&cur) {
                    if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                        return Some(info);
                    }
                }
            }
        }
        let candidate = cur.join(clean);
        if let Some(remote_url) = crate::git::get_remote_url(&candidate) {
            if let Some(info) = crate::forges::parse_remote_url(&remote_url) {
                return Some(info);
            }
        }
    }

    // 5. If formatted as owner/repo, infer codeberg as default forge
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some(RepoForgeInfo::new(
                ForgeType::Codeberg,
                "codeberg.org".to_string(),
                parts[0].to_string(),
                parts[1].to_string(),
            ));
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

/// Add or update a remote repository in the catalog without requiring a prior local clone
pub fn add_remote_repo(
    repo_input: &str,
    name_override: Option<&str>,
    desc_override: Option<&str>,
    config: Option<&crate::config::Config>,
) -> Result<(CatalogEntry, PathBuf, bool), String> {
    let info = parse_repo_identifier(repo_input)?;
    let mut catalog = load_catalog(config, None);

    let forge_name = info.forge_name().to_string();

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
            aliases: Vec::new(),
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

    // 2. ~/src/<entry.name>
    if let Ok(home) = std::env::var("HOME") {
        let p1 = PathBuf::from(&home).join("src").join(&entry.name);
        if p1.join(".git").exists() {
            return Some(p1);
        }

        // 3. ~/src/<repo> (from slug owner/repo)
        let repo_part = entry.slug.split('/').nth(1).unwrap_or(&entry.slug);
        let p2 = PathBuf::from(&home).join("src").join(repo_part);
        if p2.join(".git").exists() {
            return Some(p2);
        }

        // 4. ~/src/<alias>
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
}

/// Gather live status (clean/dirty, unpushed commits, open tickets) for catalog entries
pub fn summarize_catalog(entries: &[CatalogEntry]) -> Vec<CatalogRepoSummary> {
    entries
        .iter()
        .map(|entry| {
            let local_path = find_local_repo_path(entry);
            let (ticket_count, is_clean, unpushed, status_label) =
                if let Some(ref path) = local_path {
                    let tickets = count_open_tickets(path);
                    if let Some(st) = crate::git::inspect_local_status(path) {
                        let label = match (st.is_clean, st.ahead) {
                            (true, 0) => "clean".to_string(),
                            (true, n) => format!("clean [↑{}]", n),
                            (false, 0) => "dirty".to_string(),
                            (false, n) => format!("dirty [↑{}]", n),
                        };
                        (tickets, Some(st.is_clean), st.ahead, label)
                    } else {
                        (tickets, None, 0, "unknown".to_string())
                    }
                } else {
                    (0, None, 0, "remote".to_string())
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
            }
        })
        .collect()
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
    let h_tickets = "TICKETS";
    let h_status = "STATUS";
    let h_path = "LOCAL PATH";

    let mut w_repo = h_repo.chars().count();
    let mut w_forge = h_forge.chars().count();
    let mut w_tickets = h_tickets.chars().count();
    let mut w_status = h_status.chars().count();
    let mut w_path = h_path.chars().count();

    for s in summaries {
        let repo_display = if s.aliases.is_empty() {
            s.name.clone()
        } else {
            format!("{} [{}]", s.name, s.aliases.join(", "))
        };
        w_repo = w_repo.max(repo_display.chars().count());
        w_forge = w_forge.max(s.forge.chars().count());
        w_tickets = w_tickets.max(s.ticket_count.to_string().chars().count());
        w_status = w_status.max(s.status_label.chars().count());
        w_path = w_path.max(format_path(&s.local_path).chars().count());
    }

    let mut out = String::new();

    // Top border
    out.push_str(&format!(
        "┌─{repo}─┬─{forge}─┬─{tickets}─┬─{status}─┬─{path}─┐\n",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
        tickets = "─".repeat(w_tickets),
        status = "─".repeat(w_status),
        path = "─".repeat(w_path)
    ));

    // Header row
    let pad_r = " ".repeat(w_repo.saturating_sub(h_repo.chars().count()));
    let pad_f = " ".repeat(w_forge.saturating_sub(h_forge.chars().count()));
    let pad_t = " ".repeat(w_tickets.saturating_sub(h_tickets.chars().count()));
    let pad_s = " ".repeat(w_status.saturating_sub(h_status.chars().count()));
    let pad_p = " ".repeat(w_path.saturating_sub(h_path.chars().count()));

    out.push_str(&format!(
        "│ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │\n",
        theme.bold(h_repo),
        pad_r,
        theme.bold(h_forge),
        pad_f,
        pad_t,
        theme.bold(h_tickets),
        theme.bold(h_status),
        pad_s,
        theme.bold(h_path),
        pad_p,
    ));

    // Separator
    out.push_str(&format!(
        "├─{repo}─┼─{forge}─┼─{tickets}─┼─{status}─┼─{path}─┤\n",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
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

        let pad_repo = " ".repeat(w_repo.saturating_sub(repo_display.chars().count()));
        let pad_forge = " ".repeat(w_forge.saturating_sub(s.forge.chars().count()));
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

        let styled_repo = theme.bold(&repo_display);
        let styled_forge = theme.cyan(&s.forge);
        let styled_path = if path_str == "-" {
            theme.dim(&path_str)
        } else {
            path_str
        };

        out.push_str(&format!(
            "│ {}{} │ {}{} │ {}{} │ {}{} │ {}{} │\n",
            styled_repo,
            pad_repo,
            styled_forge,
            pad_forge,
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
        "└─{repo}─┴─{forge}─┴─{tickets}─┴─{status}─┴─{path}─┘",
        repo = "─".repeat(w_repo),
        forge = "─".repeat(w_forge),
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

    // 3. Sort
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
        _ => {
            // Default "name"
            summaries.sort_by_key(|a| a.name.to_ascii_lowercase());
        }
    }

    // 4. Reverse
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
    out.push_str(&format!("{}\n", theme.gold_bold("★ Gosplan Pre-Push Batch Report ★")));

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


