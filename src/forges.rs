//! Git Forge API client and remote URL detector (Codeberg, Forgejo, GitHub, GitLab).
//! Pure Rust standalone HTTPS via `ureq` + `rustls` + `webpki-roots`.

use crate::cultural::Theme;
use crate::git::GitRepo;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeSoftware {
    Forgejo,
    GitHub,
    GitLab,
}

impl std::fmt::Display for ForgeSoftware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forgejo => write!(f, "Forgejo"),
            Self::GitHub => write!(f, "GitHub"),
            Self::GitLab => write!(f, "GitLab"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeType {
    Codeberg,
    Tildegit,
    Forgejo,
    GitHub,
    GitLab,
}

impl ForgeType {
    pub fn software(&self) -> ForgeSoftware {
        match self {
            Self::Codeberg | Self::Tildegit | Self::Forgejo => ForgeSoftware::Forgejo,
            Self::GitHub => ForgeSoftware::GitHub,
            Self::GitLab => ForgeSoftware::GitLab,
        }
    }

    pub fn default_host(&self) -> &'static str {
        match self {
            Self::Codeberg => "codeberg.org",
            Self::Tildegit => "tildegit.org",
            Self::Forgejo => "git.2lp.in",
            Self::GitHub => "github.com",
            Self::GitLab => "gitlab.com",
        }
    }
}

impl std::fmt::Display for ForgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Codeberg => write!(f, "Codeberg"),
            Self::Tildegit => write!(f, "TildeGit"),
            Self::Forgejo => write!(f, "Forgejo"),
            Self::GitHub => write!(f, "GitHub"),
            Self::GitLab => write!(f, "GitLab"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoForgeInfo {
    pub provider: ForgeType,
    pub software: ForgeSoftware,
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl RepoForgeInfo {
    pub fn new(provider: ForgeType, host: String, owner: String, repo: String) -> Self {
        let software = provider.software();
        Self {
            provider,
            software,
            host,
            owner,
            repo,
        }
    }

    pub fn forge_name(&self) -> &str {
        match self.provider {
            ForgeType::Codeberg => "codeberg",
            ForgeType::Tildegit => "tildegit",
            ForgeType::Forgejo => "forgejo",
            ForgeType::GitHub => "github",
            ForgeType::GitLab => "gitlab",
        }
    }

    pub fn software_name(&self) -> &str {
        match self.software {
            ForgeSoftware::Forgejo => "forgejo",
            ForgeSoftware::GitHub => "github",
            ForgeSoftware::GitLab => "gitlab",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForgeComment {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeIssue {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub author: String,
    pub labels: Vec<String>,
    pub body: String,
    pub html_url: String,
    pub created_at: String,
    #[serde(default)]
    pub comments: Vec<ForgeComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgePullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub author: String,
    pub source_branch: String,
    pub target_branch: String,
    pub html_url: String,
}

/// Parses git remote URL into provider, host, owner, and repo
pub fn parse_remote_url(url: &str) -> Option<RepoForgeInfo> {
    let clean = url.trim();
    if clean.is_empty() {
        return None;
    }

    let clean_lower = clean.to_ascii_lowercase();
    let (host, path) = if clean_lower.starts_with("git@") {
        let stripped = &clean[4..];
        // SCP syntax: git@host:owner/repo.git or git@host/owner/repo.git
        if let Some((h, p)) = stripped.split_once(':') {
            (h, p)
        } else if let Some((h, p)) = stripped.split_once('/') {
            (h, p)
        } else {
            return None;
        }
    } else if clean_lower.starts_with("ssh://") {
        let stripped = &clean[6..];
        let without_user = if let Some((_, rest)) = stripped.split_once('@') {
            rest
        } else {
            stripped
        };
        let (h_port, p) = without_user.split_once('/')?;
        let host = if let Some((h, _)) = h_port.split_once(':') {
            h
        } else {
            h_port
        };
        (host, p)
    } else {
        let stripped = if clean_lower.starts_with("https://") {
            &clean[8..]
        } else if clean_lower.starts_with("http://") {
            &clean[7..]
        } else {
            return None;
        };
        let without_user = if let Some((_, rest)) = stripped.split_once('@') {
            rest
        } else {
            stripped
        };
        without_user.split_once('/')?
    };

    let clean_path = path.trim_start_matches('/').trim_end_matches(".git");
    let parts: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }

    let owner = parts[0].to_string();
    let repo = parts[1..].join("/");

    let host_lower = host.to_ascii_lowercase();
    let provider = if host_lower.contains("codeberg.org") {
        ForgeType::Codeberg
    } else if host_lower.contains("tildegit.org") {
        ForgeType::Tildegit
    } else if host_lower.contains("github.com") {
        ForgeType::GitHub
    } else if host_lower.contains("gitlab") || host_lower.contains("framagit") {
        ForgeType::GitLab
    } else {
        ForgeType::Forgejo
    };

    Some(RepoForgeInfo::new(provider, host.to_string(), owner, repo))
}

/// Detects the remote forge and coordinates from the git repository
pub fn detect_forge_from_repo(repo: &GitRepo) -> Result<RepoForgeInfo, String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Failed to run 'git remote get-url origin': {}", e))?;

    if !output.status.success() {
        return Err("No 'origin' remote found in repository".to_string());
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_remote_url(&url).ok_or_else(|| format!("Could not parse remote URL '{}'", url))
}

/// Helper: Convert a forge host to its canonical domain-based environment variable name.
/// E.g. "codeberg.org" -> "CODEBERG_ORG_TOKEN", "tildegit.org" -> "TILDEGIT_ORG_TOKEN", "github.com" -> "GITHUB_COM_TOKEN"
pub fn host_to_token_env_var(host: &str) -> String {
    let clean = host.trim().replace(['.', '-'], "_").to_ascii_uppercase();
    format!("{}_TOKEN", clean)
}

/// Resolves the authentication token for a specific forge host and provider
pub fn resolve_token_for_host(
    host: &str,
    provider: ForgeType,
    config: Option<&crate::config::Config>,
) -> Option<String> {
    let domain_var = host_to_token_env_var(host);

    // 1. Domain-based environment variable (e.g. CODEBERG_ORG_TOKEN, GITHUB_COM_TOKEN)
    if let Ok(tok) = std::env::var(&domain_var) {
        let trimmed = tok.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // 2. Domain-based token from loaded Config extra (.cccprc, ~/.env, .env)
    let fallback_cfg;
    let cfg = match config {
        Some(c) => c,
        None => {
            fallback_cfg = crate::config::Config::load_hierarchical(None);
            &fallback_cfg
        }
    };

    let domain_key = domain_var.to_ascii_lowercase();
    if let Some(tok) = cfg.extra.get(&domain_key) {
        if !tok.trim().is_empty() {
            return Some(tok.trim().to_string());
        }
    }

    // 3. Generic provider environment variables
    let provider_vars: &[&str] = match provider {
        ForgeType::Codeberg => &[
            "CODEBERG_TOKEN",
            "FORGEJO_TOKEN",
            "GITEA_TOKEN",
            "TEA_TOKEN",
        ],
        ForgeType::Tildegit => &[
            "TILDEGIT_TOKEN",
            "FORGEJO_TOKEN",
            "GITEA_TOKEN",
            "TEA_TOKEN",
        ],
        ForgeType::Forgejo => &[
            "GIT_2LP_IN_TOKEN",
            "FORGEJO_TOKEN",
            "GITEA_TOKEN",
            "TEA_TOKEN",
            "CODEBERG_TOKEN",
        ],
        ForgeType::GitHub => &["GITHUB_TOKEN", "GH_TOKEN"],
        ForgeType::GitLab => &["GITLAB_TOKEN", "GL_TOKEN"],
    };

    for var_name in provider_vars {
        if let Ok(tok) = std::env::var(var_name) {
            let trimmed = tok.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    // 4. Generic provider token from loaded Config extra (.cccprc, ~/.env, .env)
    for var_name in provider_vars {
        let k = var_name.to_ascii_lowercase();
        if let Some(tok) = cfg.extra.get(&k) {
            if !tok.trim().is_empty() {
                return Some(tok.trim().to_string());
            }
        }
    }

    None
}

/// Resolves the authentication token from environment variables based on the forge type
pub fn resolve_token(provider: ForgeType) -> Option<String> {
    resolve_token_for_host(provider.default_host(), provider, None)
}

pub struct ForgeClient {
    agent: ureq::Agent,
}

#[derive(Debug, Clone, Default)]
pub struct RepoMetadata {
    pub description: String,
    pub website: String,
    pub topics: Vec<String>,
    pub license: String,
}

impl Default for ForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgeClient {
    pub fn new() -> Self {
        let config = ureq::config::Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(10)))
            .build();
        let agent = ureq::Agent::new_with_config(config);
        Self { agent }
    }

    /// Attempt to fetch repository metadata (description, website, topics, license)
    pub fn fetch_repo_metadata(&self, info: &RepoForgeInfo) -> Option<RepoMetadata> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                format!(
                    "https://{}/api/v1/repos/{}/{}",
                    info.host, info.owner, info.repo
                )
            }
            ForgeType::GitHub => {
                format!("https://api.github.com/repos/{}/{}", info.owner, info.repo)
            }
            ForgeType::GitLab => {
                let enc = format!("{}%2F{}", info.owner, info.repo);
                format!("https://{}/api/v4/projects/{}", info.host, enc)
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        let resp = req.call().ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let body: serde_json::Value = resp.into_body().read_json().ok()?;
        let description = body
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let website = body
            .get("website")
            .or_else(|| body.get("homepage"))
            .or_else(|| body.get("web_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let topics = body
            .get("topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let license = body
            .get("license")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(spdx) = v.get("spdx_id").and_then(|s| s.as_str()) {
                    Some(spdx.to_string())
                } else {
                    v.get("name")
                        .and_then(|s| s.as_str())
                        .map(|name| name.to_string())
                }
            })
            .unwrap_or_default();

        Some(RepoMetadata {
            description,
            website,
            topics,
            license,
        })
    }

    /// Attempt to fetch open issues count for the repository directly via forge API
    pub fn fetch_open_issues_count(&self, info: &RepoForgeInfo) -> Result<usize, String> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                format!(
                    "https://{}/api/v1/repos/{}/{}",
                    info.host, info.owner, info.repo
                )
            }
            ForgeType::GitHub => {
                format!("https://api.github.com/repos/{}/{}", info.owner, info.repo)
            }
            ForgeType::GitLab => {
                let enc = format!("{}%2F{}", info.owner, info.repo);
                format!("https://{}/api/v4/projects/{}", info.host, enc)
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        match req.call() {
            Ok(resp) => {
                let status = resp.status();
                if status == 429 || status == 403 {
                    return Err(format!(
                        "Rate limit exceeded on {}. Configure an authentication token or retry later.",
                        info.host
                    ));
                }
                if resp.status().is_success() {
                    if let Ok(body) = resp.into_body().read_json::<serde_json::Value>() {
                        if let Some(count) = body.get("open_issues_count").and_then(|v| v.as_u64())
                        {
                            return Ok(count as usize);
                        }
                    }
                }
                // Fallback to fetch_issues if metadata didn't have the field
                self.fetch_issues(info, "open", 100).map(|v| v.len())
            }
            Err(e) => {
                // If metadata endpoint failed, try fallback to fetch_issues or propagate error
                self.fetch_issues(info, "open", 100)
                    .map(|v| v.len())
                    .map_err(|_| {
                        format!("Failed to fetch open issues count from {}: {}", api_url, e)
                    })
            }
        }
    }

    /// Attempt to fetch CI pipeline statuses for the repository ref
    pub fn fetch_ci_status(
        &self,
        info: &RepoForgeInfo,
        git_ref: Option<&str>,
    ) -> Result<Vec<crate::catalog::CiPipelineStatus>, String> {
        let r = git_ref.unwrap_or("main");
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                format!(
                    "https://{}/api/v1/repos/{}/{}/commits/{}/statuses",
                    info.host, info.owner, info.repo, r
                )
            }
            ForgeType::GitHub => {
                format!(
                    "https://api.github.com/repos/{}/{}/commits/{}/statuses",
                    info.owner, info.repo, r
                )
            }
            ForgeType::GitLab => {
                let enc = format!("{}%2F{}", info.owner, info.repo);
                format!(
                    "https://{}/api/v4/projects/{}/pipelines?ref={}&per_page=5",
                    info.host, enc, r
                )
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        let resp = req
            .call()
            .map_err(|e| format!("HTTP request to {} failed: {}", api_url, e))?;

        if !resp.status().is_success() {
            return Err(format!("Forge API returned status {}", resp.status()));
        }

        let body: serde_json::Value = resp
            .into_body()
            .read_json()
            .map_err(|e| format!("Failed to parse JSON from {}: {}", api_url, e))?;

        let mut pipelines = Vec::new();

        if let Some(arr) = body.as_array() {
            let mut seen = std::collections::HashSet::new();
            for item in arr {
                let name = item
                    .get("context")
                    .or_else(|| item.get("name"))
                    .or_else(|| item.get("ref"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("CI")
                    .to_string();

                if seen.contains(&name) {
                    continue;
                }
                seen.insert(name.clone());

                let status = item
                    .get("status")
                    .or_else(|| item.get("state"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let description = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let target_url = item
                    .get("target_url")
                    .or_else(|| item.get("web_url"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let updated_at = item
                    .get("updated_at")
                    .or_else(|| item.get("created_at"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                pipelines.push(crate::catalog::CiPipelineStatus {
                    name,
                    status,
                    description,
                    target_url,
                    updated_at,
                });
            }
        }

        Ok(pipelines)
    }

    /// Fetch issues from the specified forge repository
    pub fn fetch_issues(
        &self,
        info: &RepoForgeInfo,
        state: &str,
        limit: usize,
    ) -> Result<Vec<ForgeIssue>, String> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "open"
                };
                format!(
                    "https://{}/api/v1/repos/{}/{}/issues?state={}&type=issues&limit={}",
                    info.host, info.owner, info.repo, s, limit
                )
            }
            ForgeType::GitHub => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "open"
                };
                format!(
                    "https://api.github.com/repos/{}/{}/issues?state={}&per_page={}",
                    info.owner, info.repo, s, limit
                )
            }
            ForgeType::GitLab => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "opened"
                };
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                format!(
                    "https://{}/api/v4/projects/{}/issues?state={}&per_page={}",
                    info.host, enc_path, s, limit
                )
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        let mut resp = req
            .call()
            .map_err(|e| format!("HTTP request to {} failed: {}", api_url, e))?;
        let status = resp.status();
        if status == 429 || status == 403 {
            return Err(format!(
                "Rate limit exceeded on {}. Configure an authentication token or retry later.",
                info.host
            ));
        }
        if !status.is_success() {
            return Err(format!("Forge API returned error status {}", status));
        }

        let json_val: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse JSON response from {}: {}", info.host, e))?;

        parse_issues_json(&json_val, info.provider)
    }

    /// Fetch pull requests from the specified forge repository
    pub fn fetch_pull_requests(
        &self,
        info: &RepoForgeInfo,
        state: &str,
        limit: usize,
    ) -> Result<Vec<ForgePullRequest>, String> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "open"
                };
                format!(
                    "https://{}/api/v1/repos/{}/{}/pulls?state={}&limit={}",
                    info.host, info.owner, info.repo, s, limit
                )
            }
            ForgeType::GitHub => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "open"
                };
                format!(
                    "https://api.github.com/repos/{}/{}/pulls?state={}&per_page={}",
                    info.owner, info.repo, s, limit
                )
            }
            ForgeType::GitLab => {
                let s = if state == "all" {
                    "all"
                } else if state == "closed" {
                    "closed"
                } else {
                    "opened"
                };
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                format!(
                    "https://{}/api/v4/projects/{}/merge_requests?state={}&per_page={}",
                    info.host, enc_path, s, limit
                )
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        let mut resp = req
            .call()
            .map_err(|e| format!("HTTP request to {} failed: {}", api_url, e))?;
        let status = resp.status();
        if status == 429 || status == 403 {
            return Err(format!(
                "Rate limit exceeded on {}. Configure an authentication token or retry later.",
                info.host
            ));
        }
        if !status.is_success() {
            return Err(format!("Forge API returned error status {}", status));
        }

        let json_val: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse JSON response from {}: {}", info.host, e))?;

        parse_prs_json(&json_val, info.provider)
    }

    /// Create a new issue on the remote forge
    pub fn create_issue(
        &self,
        info: &RepoForgeInfo,
        title: &str,
        body: &str,
        _labels: &[String],
    ) -> Result<CreatedIssueInfo, String> {
        let token = resolve_token_for_host(&info.host, info.provider, None).ok_or_else(|| {
            format!(
                "No authentication token found for forge {} ({}). Set {} in ~/.env, .cccprc, or environment.",
                info.provider,
                info.host,
                host_to_token_env_var(&info.host)
            )
        })?;

        let (api_url, payload, auth_header) = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                let url = format!(
                    "https://{}/api/v1/repos/{}/{}/issues",
                    info.host, info.owner, info.repo
                );
                let p = serde_json::json!({
                    "title": title,
                    "body": body,
                });
                (url, p, ("Authorization", format!("token {}", token)))
            }
            ForgeType::GitHub => {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/issues",
                    info.owner, info.repo
                );
                let p = serde_json::json!({
                    "title": title,
                    "body": body,
                });
                (url, p, ("Authorization", format!("Bearer {}", token)))
            }
            ForgeType::GitLab => {
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                let url = format!("https://{}/api/v4/projects/{}/issues", info.host, enc_path);
                let p = serde_json::json!({
                    "title": title,
                    "description": body,
                });
                (url, p, ("PRIVATE-TOKEN", token))
            }
        };

        let req = self
            .agent
            .post(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json")
            .header(auth_header.0, &auth_header.1);

        let mut resp = req
            .send_json(payload)
            .map_err(|e| format!("HTTP POST request to {} failed: {}", api_url, e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_err = resp.body_mut().read_to_string().unwrap_or_default();
            return Err(format!("Forge returned HTTP {}: {}", status, body_err));
        }

        let json_val: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse response from {}: {}", info.host, e))?;

        let number = json_val
            .get("number")
            .or_else(|| json_val.get("iid"))
            .and_then(|n| n.as_u64())
            .unwrap_or(0);

        let html_url = json_val
            .get("html_url")
            .or_else(|| json_val.get("web_url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        Ok(CreatedIssueInfo {
            number,
            html_url,
            title: title.to_string(),
        })
    }

    /// Close an existing issue on the remote forge, optionally posting a resolution comment
    pub fn close_issue(
        &self,
        info: &RepoForgeInfo,
        issue_id: u64,
        comment: Option<&str>,
    ) -> Result<(), String> {
        let token = resolve_token_for_host(&info.host, info.provider, None).ok_or_else(|| {
            format!(
                "No authentication token found for forge {} ({}). Set {} in ~/.env, .cccprc, or environment.",
                info.provider,
                info.host,
                host_to_token_env_var(&info.host)
            )
        })?;

        // 1. Optionally post comment before or after closing
        if let Some(msg) = comment {
            let (comment_url, comment_payload, comment_auth) = match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    let url = format!(
                        "https://{}/api/v1/repos/{}/{}/issues/{}/comments",
                        info.host, info.owner, info.repo, issue_id
                    );
                    let p = serde_json::json!({ "body": msg });
                    (url, p, ("Authorization", format!("token {}", token)))
                }
                ForgeType::GitHub => {
                    let url = format!(
                        "https://api.github.com/repos/{}/{}/issues/{}/comments",
                        info.owner, info.repo, issue_id
                    );
                    let p = serde_json::json!({ "body": msg });
                    (url, p, ("Authorization", format!("Bearer {}", token)))
                }
                ForgeType::GitLab => {
                    let enc_path = format!("{}%2F{}", info.owner, info.repo);
                    let url = format!(
                        "https://{}/api/v4/projects/{}/issues/{}/notes",
                        info.host, enc_path, issue_id
                    );
                    let p = serde_json::json!({ "body": msg });
                    (url, p, ("PRIVATE-TOKEN", token.clone()))
                }
            };

            let req = self
                .agent
                .post(&comment_url)
                .header("User-Agent", "cccp/3.0.0")
                .header("Accept", "application/json")
                .header(comment_auth.0, &comment_auth.1);

            let _ = req.send_json(comment_payload);
        }

        // 2. Change state to closed
        let (api_url, payload, auth_header, http_method) = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                let url = format!(
                    "https://{}/api/v1/repos/{}/{}/issues/{}",
                    info.host, info.owner, info.repo, issue_id
                );
                let p = serde_json::json!({ "state": "closed" });
                (
                    url,
                    p,
                    ("Authorization", format!("token {}", token)),
                    "PATCH",
                )
            }
            ForgeType::GitHub => {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/issues/{}",
                    info.owner, info.repo, issue_id
                );
                let p = serde_json::json!({ "state": "closed" });
                (
                    url,
                    p,
                    ("Authorization", format!("Bearer {}", token)),
                    "PATCH",
                )
            }
            ForgeType::GitLab => {
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                let url = format!(
                    "https://{}/api/v4/projects/{}/issues/{}",
                    info.host, enc_path, issue_id
                );
                let p = serde_json::json!({ "state_event": "close" });
                (url, p, ("PRIVATE-TOKEN", token), "PUT")
            }
        };

        let req = if http_method == "PATCH" {
            self.agent.patch(&api_url)
        } else {
            self.agent.put(&api_url)
        }
        .header("User-Agent", "cccp/3.0.0")
        .header("Accept", "application/json")
        .header(auth_header.0, &auth_header.1);

        let mut resp = req
            .send_json(payload)
            .map_err(|e| format!("HTTP {} request to {} failed: {}", http_method, api_url, e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_err = resp.body_mut().read_to_string().unwrap_or_default();
            return Err(format!("Forge returned HTTP {}: {}", status, body_err));
        }

        Ok(())
    }

    /// Fetch comments for a specific issue from the remote forge
    pub fn fetch_issue_comments(
        &self,
        info: &RepoForgeInfo,
        issue_id: u64,
    ) -> Result<Vec<ForgeComment>, String> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                format!(
                    "https://{}/api/v1/repos/{}/{}/issues/{}/comments",
                    info.host, info.owner, info.repo, issue_id
                )
            }
            ForgeType::GitHub => {
                format!(
                    "https://api.github.com/repos/{}/{}/issues/{}/comments",
                    info.owner, info.repo, issue_id
                )
            }
            ForgeType::GitLab => {
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                format!(
                    "https://{}/api/v4/projects/{}/issues/{}/notes",
                    info.host, enc_path, issue_id
                )
            }
        };

        let mut req = self
            .agent
            .get(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json");

        if let Some(token) = resolve_token_for_host(&info.host, info.provider, None) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                    req = req.header("Authorization", &format!("token {}", token));
                }
                ForgeType::GitHub => {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
                ForgeType::GitLab => {
                    req = req.header("PRIVATE-TOKEN", &token);
                }
            }
        }

        let mut resp = req
            .call()
            .map_err(|e| format!("HTTP request to {} failed: {}", api_url, e))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!(
                "Forge comments API returned error status {}",
                status
            ));
        }

        let json_val: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse JSON response from {}: {}", info.host, e))?;

        parse_comments_json(&json_val)
    }

    /// Post a new comment to an existing issue on the remote forge
    pub fn create_comment(
        &self,
        info: &RepoForgeInfo,
        issue_id: u64,
        body: &str,
    ) -> Result<ForgeComment, String> {
        let token = resolve_token_for_host(&info.host, info.provider, None).ok_or_else(|| {
            format!(
                "No authentication token found for forge {} ({}). Set {} in ~/.env, .cccprc, or environment.",
                info.provider,
                info.host,
                host_to_token_env_var(&info.host)
            )
        })?;

        let (api_url, payload, auth_header) = match info.provider {
            ForgeType::Codeberg | ForgeType::Tildegit | ForgeType::Forgejo => {
                let url = format!(
                    "https://{}/api/v1/repos/{}/{}/issues/{}/comments",
                    info.host, info.owner, info.repo, issue_id
                );
                let p = serde_json::json!({ "body": body });
                (url, p, ("Authorization", format!("token {}", token)))
            }
            ForgeType::GitHub => {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/issues/{}/comments",
                    info.owner, info.repo, issue_id
                );
                let p = serde_json::json!({ "body": body });
                (url, p, ("Authorization", format!("Bearer {}", token)))
            }
            ForgeType::GitLab => {
                let enc_path = format!("{}%2F{}", info.owner, info.repo);
                let url = format!(
                    "https://{}/api/v4/projects/{}/issues/{}/notes",
                    info.host, enc_path, issue_id
                );
                let p = serde_json::json!({ "body": body });
                (url, p, ("PRIVATE-TOKEN", token))
            }
        };

        let req = self
            .agent
            .post(&api_url)
            .header("User-Agent", "cccp/3.0.0")
            .header("Accept", "application/json")
            .header(auth_header.0, &auth_header.1);

        let mut resp = req
            .send_json(payload)
            .map_err(|e| format!("HTTP POST request to {} failed: {}", api_url, e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_err = resp.body_mut().read_to_string().unwrap_or_default();
            return Err(format!("Forge returned HTTP {}: {}", status, body_err));
        }

        let json_val: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse response from {}: {}", info.host, e))?;

        let id = json_val.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let author = json_val
            .get("user")
            .and_then(|u| u.get("login").or_else(|| u.get("username")))
            .or_else(|| json_val.get("author").and_then(|a| a.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or("you")
            .to_string();
        let created_at = json_val
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(ForgeComment {
            id,
            author,
            body: body.to_string(),
            created_at,
        })
    }
}

/// Information returned after creating an issue on a remote forge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedIssueInfo {
    pub number: u64,
    pub html_url: String,
    pub title: String,
}

/// Helper: generate a suggested branch name from issue number and conventional commit title
pub fn suggest_branch_name(issue_num: u64, title: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let parts: Vec<&str> = clean.split('-').filter(|p| !p.is_empty()).collect();
    let slug = parts.join("-");
    if issue_num > 0 {
        format!("{}-{}", issue_num, slug)
    } else {
        slug
    }
}

/// Parses issues JSON array across different forge schemas
pub fn parse_issues_json(
    val: &serde_json::Value,
    _provider: ForgeType,
) -> Result<Vec<ForgeIssue>, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "Expected JSON array of issues".to_string())?;
    let mut issues = Vec::new();

    for item in arr {
        // On GitHub, filter out pull requests which are included in /issues
        if item.get("pull_request").is_some() && !item["pull_request"].is_null() {
            continue;
        }

        let number = item
            .get("number")
            .or_else(|| item.get("iid"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let state = item
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open")
            .to_string();

        let author = item
            .get("user")
            .and_then(|u| u.get("login").or_else(|| u.get("username")))
            .or_else(|| item.get("author").and_then(|a| a.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut labels = Vec::new();
        if let Some(l_arr) = item.get("labels").and_then(|l| l.as_array()) {
            for l in l_arr {
                if let Some(name) = l.as_str() {
                    labels.push(name.to_string());
                } else if let Some(name) = l.get("name").and_then(|n| n.as_str()) {
                    labels.push(name.to_string());
                }
            }
        }

        let body = item
            .get("body")
            .or_else(|| item.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let html_url = item
            .get("html_url")
            .or_else(|| item.get("web_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        issues.push(ForgeIssue {
            number,
            title,
            state,
            author,
            labels,
            body,
            html_url,
            created_at,
            comments: Vec::new(),
        });
    }

    Ok(issues)
}

/// Parses comments JSON array across different forge schemas
pub fn parse_comments_json(val: &serde_json::Value) -> Result<Vec<ForgeComment>, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "Expected JSON array of comments".to_string())?;
    let mut comments = Vec::new();
    for item in arr {
        let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let author = item
            .get("user")
            .and_then(|u| u.get("login").or_else(|| u.get("username")))
            .or_else(|| item.get("author").and_then(|a| a.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let body = item
            .get("body")
            .or_else(|| item.get("note"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        comments.push(ForgeComment {
            id,
            author,
            body,
            created_at,
        });
    }
    Ok(comments)
}

/// Parses pull requests JSON array across different forge schemas
pub fn parse_prs_json(
    val: &serde_json::Value,
    _provider: ForgeType,
) -> Result<Vec<ForgePullRequest>, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "Expected JSON array of pull requests".to_string())?;
    let mut prs = Vec::new();

    for item in arr {
        let number = item
            .get("number")
            .or_else(|| item.get("iid"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let state = item
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open")
            .to_string();

        let author = item
            .get("user")
            .and_then(|u| u.get("login").or_else(|| u.get("username")))
            .or_else(|| item.get("author").and_then(|a| a.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let source_branch = item
            .get("head")
            .and_then(|h| h.get("ref").or_else(|| h.get("label")))
            .or_else(|| item.get("source_branch"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let target_branch = item
            .get("base")
            .and_then(|b| b.get("ref").or_else(|| b.get("label")))
            .or_else(|| item.get("target_branch"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let html_url = item
            .get("html_url")
            .or_else(|| item.get("web_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        prs.push(ForgePullRequest {
            number,
            title,
            state,
            author,
            source_branch,
            target_branch,
            html_url,
        });
    }

    Ok(prs)
}

/// Print formatted issues table to terminal
pub fn display_issues(issues: &[ForgeIssue], theme: &Theme) {
    if issues.is_empty() {
        println!("{} No issues found.", theme.dim("•"));
        return;
    }

    println!("{}", theme.bold("★ Remote Forge Issues ★"));
    for issue in issues {
        let state_badge = if issue.state == "open" || issue.state == "opened" {
            theme.green("open")
        } else {
            theme.red("closed")
        };

        let labels_str = if issue.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", theme.cyan(&issue.labels.join(", ")))
        };

        println!(
            "  {} #{:<4} {} (@{}){}",
            state_badge,
            theme.gold_bold(&issue.number.to_string()),
            issue.title,
            issue.author,
            labels_str
        );
    }
}

/// Print formatted pull requests table to terminal
pub fn display_prs(prs: &[ForgePullRequest], theme: &Theme) {
    if prs.is_empty() {
        println!("{} No pull requests found.", theme.dim("•"));
        return;
    }

    println!("{}", theme.bold("★ Remote Forge Pull Requests ★"));
    for pr in prs {
        let state_badge = if pr.state == "open" || pr.state == "opened" {
            theme.green("open")
        } else {
            theme.red("closed")
        };

        println!(
            "  {} #{:<4} {} (@{}) [{} ➔ {}]",
            state_badge,
            theme.gold_bold(&pr.number.to_string()),
            pr.title,
            pr.author,
            theme.cyan(&pr.source_branch),
            theme.cyan(&pr.target_branch)
        );
    }
}
