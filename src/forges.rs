//! Git Forge API client and remote URL detector (Codeberg, Forgejo, GitHub, GitLab).
//! Pure Rust standalone HTTPS via `ureq` + `rustls` + `webpki-roots`.

use crate::cultural::Theme;
use crate::git::GitRepo;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeType {
    Codeberg,
    Forgejo,
    GitHub,
    GitLab,
}

impl std::fmt::Display for ForgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Codeberg => write!(f, "Codeberg"),
            Self::Forgejo => write!(f, "Forgejo"),
            Self::GitHub => write!(f, "GitHub"),
            Self::GitLab => write!(f, "GitLab"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoForgeInfo {
    pub provider: ForgeType,
    pub host: String,
    pub owner: String,
    pub repo: String,
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

    let (host, path) = if let Some(stripped) = clean.strip_prefix("git@") {
        // SCP syntax: git@host:owner/repo.git or git@host/owner/repo.git
        if let Some((h, p)) = stripped.split_once(':') {
            (h, p)
        } else if let Some((h, p)) = stripped.split_once('/') {
            (h, p)
        } else {
            return None;
        }
    } else if let Some(stripped) = clean.strip_prefix("ssh://") {
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
        let stripped = clean
            .strip_prefix("https://")
            .or_else(|| clean.strip_prefix("http://"))?;
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
    } else if host_lower.contains("github.com") {
        ForgeType::GitHub
    } else if host_lower.contains("gitlab") || host_lower.contains("framagit") {
        ForgeType::GitLab
    } else {
        ForgeType::Forgejo
    };

    Some(RepoForgeInfo {
        provider,
        host: host.to_string(),
        owner,
        repo,
    })
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

/// Resolves the authentication token from environment variables based on the forge type
pub fn resolve_token(provider: ForgeType) -> Option<String> {
    match provider {
        ForgeType::Codeberg | ForgeType::Forgejo => std::env::var("CODEBERG_TOKEN")
            .or_else(|_| std::env::var("FORGEJO_TOKEN"))
            .or_else(|_| std::env::var("GITEA_TOKEN"))
            .ok(),
        ForgeType::GitHub => std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .ok(),
        ForgeType::GitLab => std::env::var("GITLAB_TOKEN")
            .or_else(|_| std::env::var("GL_TOKEN"))
            .ok(),
    }
}

pub struct ForgeClient {
    agent: ureq::Agent,
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

    /// Fetch issues from the specified forge repository
    pub fn fetch_issues(
        &self,
        info: &RepoForgeInfo,
        state: &str,
        limit: usize,
    ) -> Result<Vec<ForgeIssue>, String> {
        let api_url = match info.provider {
            ForgeType::Codeberg | ForgeType::Forgejo => {
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

        if let Some(token) = resolve_token(info.provider) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Forgejo => {
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
            ForgeType::Codeberg | ForgeType::Forgejo => {
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

        if let Some(token) = resolve_token(info.provider) {
            match info.provider {
                ForgeType::Codeberg | ForgeType::Forgejo => {
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
        let token = resolve_token(info.provider).ok_or_else(|| {
            format!(
                "No authentication token found for forge {} ({}). Set CODEBERG_TOKEN, GITHUB_TOKEN, or GITLAB_TOKEN in ~/.env or environment.",
                info.provider, info.host
            )
        })?;

        let (api_url, payload, auth_header) = match info.provider {
            ForgeType::Codeberg | ForgeType::Forgejo => {
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
        });
    }

    Ok(issues)
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
