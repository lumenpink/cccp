//! Git repository inspection and commit reader using Gitoxide (gix).
//! Ultra-fast, zero-subprocess, memory-mapped reads of Git history, tags, and working tree.

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
            Some(p) => gix::discover(p),
            None => gix::discover("."),
        }
        .map_err(|e| format!("Git repository not found: {}", e))?;

        let root_dir = repo
            .workdir()
            .map(|p| p.to_path_buf())
            .or_else(|| repo.git_dir().parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // Resolve HEAD
        let mut head = repo
            .head()
            .map_err(|e| format!("Failed to read HEAD: {}", e))?;

        let branch = head.referent_name().map(|n| n.shorten().to_string());

        let head_commit = match head.peel_to_commit() {
            Ok(c) => c,
            Err(_) => {
                // Empty repository without commits
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

        let head_id = head_commit.id.to_string();
        let head_short_id = head_id.chars().take(7).collect();

        // Format commit date YYYYMMDD
        let head_date = match head_commit.time() {
            Ok(t) => {
                let seconds = t.seconds;
                format_timestamp_yyyymmdd(seconds)
            }
            Err(_) => format_date_now(),
        };

        // Collect all tags and find the latest valid SemVer tag
        let latest_tag = find_latest_semver_tag(&repo, head_commit.id)?;

        // Walk commits from HEAD backwards until latest tag commit
        let stop_id = latest_tag.as_ref().map(|t| t.commit_id.clone());
        let commits_since_tag = walk_commits_since(&repo, head_commit.id, stop_id.as_deref())?;

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
    repo: &gix::Repository,
    _head_id: gix::ObjectId,
) -> Result<Option<TagInfo>, String> {
    let refs = match repo.references() {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let tags = match refs.tags() {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    let mut candidate_tags: Vec<TagInfo> = Vec::new();

    for tag_res in tags {
        let mut tag_ref = match tag_res {
            Ok(r) => r,
            Err(_) => continue,
        };

        let raw_name = tag_ref.name().shorten().to_string();
        let trimmed_version = raw_name.strip_prefix('v').unwrap_or(&raw_name).to_string();

        if is_valid_semver_syntax(&trimmed_version) {
            let commit_id = match tag_ref.peel_to_id() {
                Ok(id) => id.to_string(),
                Err(_) => continue,
            };

            candidate_tags.push(TagInfo {
                name: raw_name,
                version: trimmed_version,
                commit_id,
            });
        }
    }

    if candidate_tags.is_empty() {
        return Ok(None);
    }

    // Traverse commits backwards from HEAD; the first commit that matches a tag is the latest tag!
    let revwalk = repo.rev_walk([_head_id]);
    if let Ok(iter) = revwalk.all() {
        for info in iter.flatten() {
            let cid_str = info.id.to_string();
            if let Some(matching_tag) = candidate_tags.iter().find(|t| t.commit_id == cid_str) {
                return Ok(Some(matching_tag.clone()));
            }
        }
    }

    // Fallback: return the tag with the highest SemVer
    candidate_tags.sort_by(|a, b| compare_semver(&a.version, &b.version));
    Ok(candidate_tags.pop())
}

/// Walk commits from HEAD back to `stop_at_commit_id` (exclusive)
fn walk_commits_since(
    repo: &gix::Repository,
    head_id: gix::ObjectId,
    stop_at_commit_id: Option<&str>,
) -> Result<Vec<CommitSummary>, String> {
    let mut commits = Vec::new();
    let revwalk = repo.rev_walk([head_id]);
    let iter = match revwalk.all() {
        Ok(i) => i,
        Err(e) => return Err(format!("Failed to walk commits: {}", e)),
    };

    for commit_info in iter {
        let info = match commit_info {
            Ok(i) => i,
            Err(_) => break,
        };

        let id_str = info.id.to_string();
        if let Some(stop_id) = stop_at_commit_id {
            if id_str == stop_id {
                // Reached the tag commit; stop collecting
                break;
            }
        }

        let obj = match repo.find_commit(info.id) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let msg_raw = obj.message_raw().unwrap_or_default();
        let msg_str = String::from_utf8_lossy(msg_raw.as_ref()).to_string();
        let summary = msg_str.lines().next().unwrap_or("").trim().to_string();
        let short_id = id_str.chars().take(7).collect();

        commits.push(CommitSummary {
            id: id_str,
            short_id,
            summary,
            message: msg_str,
        });
    }

    Ok(commits)
}

/// Fast working tree check for modifications
fn check_is_dirty(repo: &gix::Repository) -> bool {
    // Check if git index / working tree has uncommitted modifications
    match repo.status(gix::progress::Discard) {
        Ok(status) => match status.into_index_worktree_iter(None) {
            Ok(iter) => iter.into_iter().next().is_some(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Simple SemVer syntax validator (e.g. 1.2.3, 0.0.1, 2.1.0-dev.1)
fn is_valid_semver_syntax(ver: &str) -> bool {
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() < 3 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_digit()) && parts[1].chars().all(|c| c.is_ascii_digit())
}

/// Basic SemVer comparison (major.minor.patch)
fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_nums = |s: &str| -> (u64, u64, u64) {
        let clean = s.split('-').next().unwrap_or(s);
        let mut p = clean.split('.');
        let major = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let minor = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let patch = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };
    parse_nums(a).cmp(&parse_nums(b))
}

fn format_timestamp_yyyymmdd(seconds: i64) -> String {
    // Days since unix epoch
    let days = seconds / 86400;
    // Civil day algorithm
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 142401) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}{:02}{:02}", y, m, d)
}

fn format_date_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    format_timestamp_yyyymmdd(now)
}

/// Extract the remote URL (origin or first remote) from a git repository directory
pub fn get_remote_url(repo_dir: &Path) -> Option<String> {
    // 1. Direct inspection of .git/config (zero subprocess overhead)
    let git_path = repo_dir.join(".git");
    let config_path = if git_path.is_dir() {
        Some(git_path.join("config"))
    } else if git_path.is_file() {
        // Worktree or submodule: gitdir: <path>
        if let Ok(content) = std::fs::read_to_string(&git_path) {
            let line = content.lines().next().unwrap_or("").trim();
            if let Some(target) = line.strip_prefix("gitdir:") {
                let target_path = target.trim();
                let p = if Path::new(target_path).is_absolute() {
                    PathBuf::from(target_path)
                } else {
                    repo_dir.join(target_path)
                };
                Some(p.join("config"))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref cfg_file) = config_path {
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
