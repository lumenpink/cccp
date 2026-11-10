//! Git repository inspection and commit reader using native Git CLI.
//! Robust, zero-C-ABI reads of Git history, tags, and working tree.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Detailed Git inspection summary
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub root_dir: PathBuf,
    pub branch: Option<String>,
    pub head_id: String,
    pub head_short_id: String,
    pub head_date: String, // YYYYMMDD
    pub latest_tag: Option<TagInfo>,
    pub commits_since_tag: Vec<CommitSummary>,
    pub is_dirty: bool,
    pub head_parent_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo {
    /// Full tag name (e.g. "v1.2.0" or "2.0.0")
    pub name: String,
    /// Stripped version string (e.g. "1.2.0")
    pub version: String,
    /// Commit SHA target
    pub commit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSummary {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub message: String,
}

impl GitRepo {
    /// Discover git repository starting from `path` (or current directory)
    pub fn open(path: Option<&Path>) -> Result<Self, String> {
        let start_dir = path.unwrap_or_else(|| Path::new("."));

        // Discover repository root
        let root_out = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(start_dir)
            .output()
            .map_err(|e| format!("Falha ao invocar git rev-parse: {}", e))?;

        if !root_out.status.success() {
            return Err(format!(
                "Git repository not found in '{}'",
                start_dir.display()
            ));
        }

        let root_dir_str = String::from_utf8_lossy(&root_out.stdout).trim().to_string();
        let root_dir = PathBuf::from(root_dir_str);

        // Check if repository has any commits (resolve HEAD)
        let head_out = Command::new("git")
            .args(["rev-parse", "--verify", "HEAD"])
            .current_dir(&root_dir)
            .output()
            .map_err(|e| format!("Falha ao invocar git rev-parse HEAD: {}", e))?;

        if !head_out.status.success() {
            // Empty repository without commits
            let branch = resolve_branch(&root_dir);
            let is_dirty = check_is_dirty(&root_dir);

            return Ok(Self {
                root_dir,
                branch,
                head_id: String::new(),
                head_short_id: String::new(),
                head_date: format_date_now(),
                latest_tag: None,
                commits_since_tag: Vec::new(),
                is_dirty,
                head_parent_count: 0,
            });
        }

        let head_id = String::from_utf8_lossy(&head_out.stdout).trim().to_string();
        let head_short_id = head_id.chars().take(7).collect();

        // Branch
        let branch = resolve_branch(&root_dir);

        // Date of HEAD commit in YYYYMMDD
        let head_date = get_commit_date_yyyymmdd(&root_dir, &head_id);

        // Count parents of HEAD
        let head_parent_count = get_parent_count(&root_dir, &head_id);

        // Find latest valid SemVer tag reachable from HEAD
        let latest_tag = find_latest_semver_tag(&root_dir, &head_id)?;

        // Walk commits from HEAD backwards until latest tag commit
        let stop_id = latest_tag.as_ref().map(|t| t.commit_id.clone());
        let commits_since_tag = walk_commits_since(&root_dir, stop_id.as_deref())?;

        // Check if working tree is dirty
        let is_dirty = check_is_dirty(&root_dir);

        Ok(Self {
            root_dir,
            branch,
            head_id,
            head_short_id,
            head_date,
            latest_tag,
            commits_since_tag,
            is_dirty,
            head_parent_count,
        })
    }

    /// Read up to `limit` recent commit messages from HEAD
    pub fn get_recent_commits(&self, limit: usize) -> Vec<String> {
        let output = Command::new("git")
            .args(["log", &format!("-{}", limit), "--format=%s"])
            .current_dir(&self.root_dir)
            .output();

        match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            _ => Vec::new(),
        }
    }
}

fn resolve_branch(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(root)
        .output()
        .ok()?;

    if out.status.success() {
        let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !b.is_empty() {
            return Some(b);
        }
    }

    let out2 = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;

    if out2.status.success() {
        let b = String::from_utf8_lossy(&out2.stdout).trim().to_string();
        if !b.is_empty() && b != "HEAD" {
            return Some(b);
        }
    }

    None
}

fn get_commit_date_yyyymmdd(root: &Path, commit_id: &str) -> String {
    let out = Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%cd",
            "--date=format:%Y%m%d",
            commit_id,
        ])
        .current_dir(root)
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.len() == 8 {
                return s;
            }
        }
        _ => {}
    }

    format_date_now()
}

fn get_parent_count(root: &Path, commit_id: &str) -> usize {
    let out = Command::new("git")
        .args(["rev-list", "--parents", "-n", "1", commit_id])
        .current_dir(root)
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let line = String::from_utf8_lossy(&o.stdout);
            let hashes: Vec<&str> = line.split_whitespace().collect();
            if hashes.len() > 1 {
                hashes.len() - 1
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn check_is_dirty(root: &Path) -> bool {
    let out = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=no"])
        .current_dir(root)
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            !stdout.trim().is_empty()
        }
        _ => false,
    }
}

/// Helper: find all tags and select the most recent valid SemVer tag reachable from HEAD
fn find_latest_semver_tag(root: &Path, head_id: &str) -> Result<Option<TagInfo>, String> {
    let out = Command::new("git")
        .args(["tag", "--list"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Falha ao listar tags: {}", e))?;

    if !out.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut candidate_tags: Vec<TagInfo> = Vec::new();

    for line in stdout.lines() {
        let raw_name = line.trim();
        if raw_name.is_empty() {
            continue;
        }

        let trimmed_version = raw_name.strip_prefix('v').unwrap_or(raw_name).to_string();
        if is_valid_semver_syntax(&trimmed_version) {
            let rev_out = Command::new("git")
                .args([
                    "rev-parse",
                    "-q",
                    "--verify",
                    &format!("refs/tags/{}^{{commit}}", raw_name),
                ])
                .current_dir(root)
                .output();

            if let Ok(rev) = rev_out {
                if rev.status.success() {
                    let commit_id = String::from_utf8_lossy(&rev.stdout).trim().to_string();
                    if !commit_id.is_empty() {
                        candidate_tags.push(TagInfo {
                            name: raw_name.to_string(),
                            version: trimmed_version,
                            commit_id,
                        });
                    }
                }
            }
        }
    }

    if candidate_tags.is_empty() {
        return Ok(None);
    }

    // Traverse commits backwards from HEAD; the first commit that matches a tag is the latest tag!
    let log_out = Command::new("git")
        .args(["rev-list", head_id])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Falha ao percorrer commits: {}", e))?;

    if log_out.status.success() {
        let rev_list = String::from_utf8_lossy(&log_out.stdout);
        for commit_line in rev_list.lines() {
            let commit_sha = commit_line.trim();
            if commit_sha.is_empty() {
                continue;
            }

            let mut matches: Vec<TagInfo> = candidate_tags
                .iter()
                .filter(|t| t.commit_id == commit_sha)
                .cloned()
                .collect();

            if !matches.is_empty() {
                matches.sort_by(|a, b| compare_semver(&a.version, &b.version));
                return Ok(matches.pop());
            }
        }
    }

    // Fallback: pick highest SemVer tag overall
    candidate_tags.sort_by(|a, b| compare_semver(&a.version, &b.version));
    Ok(candidate_tags.pop())
}

/// Helper: walk commits from `start_id` backwards until `stop_id`
fn walk_commits_since(
    root: &Path,
    stop_id: Option<&str>,
) -> Result<Vec<CommitSummary>, String> {
    let range = match stop_id {
        Some(stop) => format!("{}..HEAD", stop),
        None => "HEAD".to_string(),
    };

    let out = Command::new("git")
        .args(["log", &range, "--format=%H%x1f%h%x1f%s%x1f%B%x1e"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Falha ao ler log de commits: {}", e))?;

    if !out.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut commits = Vec::new();

    for record in stdout.split('\x1e') {
        let trimmed_record = record.trim();
        if trimmed_record.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed_record.split('\x1f').collect();
        if parts.len() >= 4 {
            let id = parts[0].trim().to_string();
            let short_id = parts[1].trim().to_string();
            let summary = parts[2].trim().to_string();
            let message = parts[3].to_string();

            commits.push(CommitSummary {
                id,
                short_id,
                summary,
                message,
            });
        }
    }

    Ok(commits)
}

/// Verify if a version string matches standard SemVer syntax (e.g. 1.2.3 or 1.2.3-alpha)
fn is_valid_semver_syntax(ver: &str) -> bool {
    let mut parts = ver.split('+');
    let base_and_prerelease = parts.next().unwrap_or(ver);

    let mut core_parts = base_and_prerelease.split('-');
    let core = core_parts.next().unwrap_or(base_and_prerelease);

    let numbers: Vec<&str> = core.split('.').collect();
    if numbers.len() != 3 {
        return false;
    }

    numbers
        .iter()
        .all(|n| n.chars().all(|c| c.is_ascii_digit()))
}

/// Compare two SemVer strings (using semver crate or fallback)
fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    if let (Ok(ver_a), Ok(ver_b)) = (semver::Version::parse(a), semver::Version::parse(b)) {
        ver_a.cmp(&ver_b)
    } else {
        a.cmp(b)
    }
}

/// Format unix timestamp as YYYYMMDD
fn format_timestamp_yyyymmdd(seconds: i64) -> String {
    let secs_per_day = 86400i64;
    let days = seconds.div_euclid(secs_per_day);

    let l = days + 68569 + 2440588;
    let n = 4 * l / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = 4000 * (l + 1) / 1461001;
    let l = l - 1461 * i / 4 + 31;
    let j = 80 * l / 2447;
    let day = l - 2447 * j / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;

    format!("{:04}{:02}{:02}", year, month, day)
}

fn format_date_now() -> String {
    format_timestamp_yyyymmdd(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
}

/// Fast, low-overhead inspection of remote URL from .git/config or git CLI
pub fn get_remote_url(repo_dir: &Path) -> Option<String> {
    // 1. Direct read of .git/config without subprocess
    let git_dir = repo_dir.join(".git");
    let config_path = if git_dir.is_dir() {
        Some(git_dir.join("config"))
    } else if git_dir.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_dir) {
            if let Some(gitdir_line) = content.lines().find(|l| l.starts_with("gitdir:")) {
                let rel_path = gitdir_line["gitdir:".len()..].trim();
                let actual_git_dir = repo_dir.join(rel_path);
                Some(actual_git_dir.join("config"))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(cfg_file) = config_path {
        if let Ok(content) = std::fs::read_to_string(cfg_file) {
            let mut in_origin = false;
            let mut fallback_url = None;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_origin = trimmed.eq_ignore_ascii_case(r#"[remote "origin"]"#);
                } else if let Some(url_part) = trimmed.strip_prefix("url =") {
                    let u = url_part.trim().to_string();
                    if in_origin {
                        return Some(u);
                    }
                    if fallback_url.is_none() {
                        fallback_url = Some(u);
                    }
                }
            }

            if let Some(u) = fallback_url {
                return Some(u);
            }
        }
    }

    // 2. Fallback to git subprocess if config was not directly readable
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return Some(stdout);
        }
    }

    None
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalRepoStatus {
    pub is_clean: bool,
    pub ahead: usize,
    pub behind: usize,
    pub branch: Option<String>,
    pub modified_files: usize,
}

/// Inspect local git status (cleanliness, ahead/behind upstream count, branch)
pub fn inspect_local_status(repo_dir: &Path) -> Option<LocalRepoStatus> {
    if !repo_dir.join(".git").exists() {
        return None;
    }

    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "-b"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();

    let mut branch = None;
    let mut ahead = 0;
    let mut behind = 0;

    if let Some(branch_line) = lines.next() {
        let clean = branch_line.trim_start_matches('#').trim();
        if let Some(bracket_idx) = clean.find('[') {
            let info_part = &clean[bracket_idx..];
            if let Some(ahead_match) = info_part.split("ahead ").nth(1) {
                ahead = ahead_match
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
            if let Some(behind_match) = info_part.split("behind ").nth(1) {
                behind = behind_match
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        let bname = clean
            .split("...")
            .next()
            .unwrap_or(clean)
            .split_whitespace()
            .next()
            .unwrap_or("");
        if !bname.is_empty() {
            branch = Some(bname.to_string());
        }
    }

    let modified_files = lines.count();
    let is_clean = modified_files == 0;

    Some(LocalRepoStatus {
        is_clean,
        ahead,
        behind,
        branch,
        modified_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_syntax_validator() {
        assert!(is_valid_semver_syntax("1.0.0"));
        assert!(is_valid_semver_syntax("0.1.0-alpha"));
        assert!(is_valid_semver_syntax("2.4.1+build.123"));
        assert!(!is_valid_semver_syntax("invalid"));
        assert!(!is_valid_semver_syntax("1.0"));
    }

    #[test]
    fn test_semver_ordering() {
        assert_eq!(compare_semver("1.0.0", "2.0.0"), std::cmp::Ordering::Less);
        assert_eq!(
            compare_semver("1.2.0", "1.1.9"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(compare_semver("2.1.3", "2.1.3"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_timestamp_formatting() {
        assert_eq!(format_timestamp_yyyymmdd(0), "19700101");
        let date = format_timestamp_yyyymmdd(1790726400);
        assert_eq!(date, "20260930");
    }

    #[test]
    fn test_git_repo_open_current_directory() {
        let repo = GitRepo::open(None).expect("Failed to open current git repo");
        assert!(!repo.head_id.is_empty(), "head_id must not be empty");
        assert_eq!(repo.head_short_id.len(), 7);
        assert_eq!(repo.head_date.len(), 8);
    }
}
