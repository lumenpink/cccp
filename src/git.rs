//! Git repository inspection and commit reader using libgit2 (git2).
//! Ultra-fast, zero-subprocess reads of Git history, tags, and working tree with native SSL support.

use std::path::{Path, PathBuf};

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
        let repo = match path {
            Some(p) => git2::Repository::discover(p),
            None => git2::Repository::discover("."),
        }
        .map_err(|e| format!("Git repository not found: {}", e))?;

        let root_dir = repo
            .workdir()
            .map(|p| p.to_path_buf())
            .or_else(|| repo.path().parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // Resolve HEAD
        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => {
                // Empty repository without commits
                return Ok(Self {
                    root_dir,
                    branch: None,
                    head_id: String::new(),
                    head_short_id: String::new(),
                    head_date: format_date_now(),
                    latest_tag: None,
                    commits_since_tag: Vec::new(),
                    is_dirty: false,
                });
            }
        };

        let branch = head.shorthand().map(|s| s.to_string());

        let head_commit = match head.peel_to_commit() {
            Ok(c) => c,
            Err(_) => {
                return Ok(Self {
                    root_dir,
                    branch,
                    head_id: String::new(),
                    head_short_id: String::new(),
                    head_date: format_date_now(),
                    latest_tag: None,
                    commits_since_tag: Vec::new(),
                    is_dirty: false,
                });
            }
        };

        let head_id = head_commit.id().to_string();
        let head_short_id = head_id.chars().take(7).collect();

        // Format commit date YYYYMMDD
        let seconds = head_commit.time().seconds();
        let head_date = format_timestamp_yyyymmdd(seconds);

        // Collect all tags and find the latest valid SemVer tag reachable from HEAD
        let latest_tag = find_latest_semver_tag(&repo, head_commit.id())?;

        // Walk commits from HEAD backwards until latest tag commit
        let stop_id = latest_tag.as_ref().map(|t| t.commit_id.clone());
        let commits_since_tag = walk_commits_since(&repo, head_commit.id(), stop_id.as_deref())?;

        // Check if working tree is dirty (modified files)
        let is_dirty = check_is_dirty(&repo);

        Ok(Self {
            root_dir,
            branch,
            head_id,
            head_short_id,
            head_date,
            latest_tag,
            commits_since_tag,
            is_dirty,
        })
    }
}

/// Helper: find all tags and select the most recent valid SemVer tag reachable from HEAD
fn find_latest_semver_tag(
    repo: &git2::Repository,
    head_id: git2::Oid,
) -> Result<Option<TagInfo>, String> {
    let tag_names = match repo.tag_names(None) {
        Ok(names) => names,
        Err(_) => return Ok(None),
    };

    let mut candidate_tags: Vec<TagInfo> = Vec::new();

    for name_opt in tag_names.iter() {
        let raw_name = match name_opt {
            Some(n) => n,
            None => continue,
        };

        let trimmed_version = raw_name.strip_prefix('v').unwrap_or(raw_name).to_string();

        if is_valid_semver_syntax(&trimmed_version) {
            let ref_name = format!("refs/tags/{}", raw_name);
            let obj = match repo.revparse_single(&ref_name) {
                Ok(o) => o,
                Err(_) => continue,
            };

            let commit_id = match obj.peel_to_commit() {
                Ok(c) => c.id().to_string(),
                Err(_) => continue,
            };

            candidate_tags.push(TagInfo {
                name: raw_name.to_string(),
                version: trimmed_version,
                commit_id,
            });
        }
    }

    if candidate_tags.is_empty() {
        return Ok(None);
    }

    // Traverse commits backwards from HEAD; the first commit that matches a tag is the latest tag!
    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    let _ = revwalk.push(head_id);

    for oid_res in revwalk {
        let oid = match oid_res {
            Ok(id) => id,
            Err(_) => continue,
        };

        let oid_str = oid.to_string();

        let mut matches: Vec<TagInfo> = candidate_tags
            .iter()
            .filter(|t| t.commit_id == oid_str)
            .cloned()
            .collect();

        if !matches.is_empty() {
            // Sort matches so highest SemVer wins if multiple tags on same commit
            matches.sort_by(|a, b| compare_semver(&a.version, &b.version));
            return Ok(matches.pop());
        }
    }

    // Fallback if none reachable on HEAD branch history: pick highest SemVer tag overall
    candidate_tags.sort_by(|a, b| compare_semver(&a.version, &b.version));
    Ok(candidate_tags.pop())
}

/// Helper: walk commits from `start_id` backwards until `stop_id`
fn walk_commits_since(
    repo: &git2::Repository,
    start_id: git2::Oid,
    stop_id: Option<&str>,
) -> Result<Vec<CommitSummary>, String> {
    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    let _ = revwalk.push(start_id);

    let mut commits = Vec::new();

    for oid_res in revwalk {
        let oid = match oid_res {
            Ok(id) => id,
            Err(_) => break,
        };

        let oid_str = oid.to_string();

        if let Some(target) = stop_id {
            if oid_str == target {
                break;
            }
        }

        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let message = commit.message().unwrap_or("").to_string();
        let summary = commit.summary().unwrap_or("").to_string();
        let short_id = oid_str.chars().take(7).collect();

        commits.push(CommitSummary {
            id: oid_str,
            short_id,
            summary,
            message,
        });
    }

    Ok(commits)
}

/// Helper: check if working tree or index has uncommitted modifications
fn check_is_dirty(repo: &git2::Repository) -> bool {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(false);
    opts.include_ignored(false);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => !statuses.is_empty(),
        Err(_) => false,
    }
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

/// Fast, low-overhead inspection of remote URL from .git/config
pub fn get_remote_url(repo_dir: &Path) -> Option<String> {
    if let Ok(repo) = git2::Repository::discover(repo_dir) {
        if let Ok(remote) = repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return Some(url.to_string());
            }
        }
    }

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
    let output = std::process::Command::new("git")
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

    let output = std::process::Command::new("git")
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
        // Unix epoch 0 = 1970-01-01
        assert_eq!(format_timestamp_yyyymmdd(0), "19700101");
        // 1790726400 is 2026-09-30 00:00:00 UTC
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
