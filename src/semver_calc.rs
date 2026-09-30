//! Predictive Semantic Versioning (SemVer 2.0.0) Engine.
//! Calculates next target release version and development build metadata
//! based on Git history and Conventional Commits.

use crate::cli::BumpType;
use crate::git::{CommitSummary, GitRepo};
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BumpLevel {
    Major,
    Minor,
    Patch,
    None,
}

impl std::fmt::Display for BumpLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Calculated version result with full SemVer 2.0.0 metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PredictedVersion {
    /// Baseline version (from last tag or config default base)
    pub base_version: String,
    /// Next clean target release version (e.g. "2.4.0")
    pub target_version: String,
    /// Full development version (e.g. "2.4.0-dev.53+20260930.0ffc534")
    pub dev_version: String,
    /// Formatted tag name (e.g. "v2.4.0" or "2.4.0")
    pub tag: String,
    /// Calculated or overridden bump level
    pub bump: BumpLevel,
    /// Commits accumulated since the baseline tag
    pub commits_count: usize,
    /// Whether HEAD is exactly on a release tag with 0 new commits
    pub is_release: bool,
}

/// Calculate predictive Semantic Version from Git repository state
pub fn calculate_version(
    repo: &GitRepo,
    default_base: &str,
    no_v: bool,
    bump_override: Option<BumpType>,
) -> Result<PredictedVersion, String> {
    let base_str = repo
        .latest_tag
        .as_ref()
        .map(|t| t.version.as_str())
        .unwrap_or(default_base);

    let base_semver = Version::parse(base_str)
        .map_err(|e| format!("Invalid base SemVer '{}': {}", base_str, e))?;

    let commits_count = repo.commits_since_tag.len();
    let is_on_tag = repo.latest_tag.is_some() && commits_count == 0 && !repo.is_dirty;

    // Detect bump from commits unless explicitly overridden
    let bump = match bump_override {
        Some(BumpType::Major) => BumpLevel::Major,
        Some(BumpType::Minor) => BumpLevel::Minor,
        Some(BumpType::Patch) => BumpLevel::Patch,
        None => {
            if commits_count == 0 {
                BumpLevel::None
            } else {
                detect_bump_from_commits(&repo.commits_since_tag)
            }
        }
    };

    let target_semver = if is_on_tag {
        base_semver.clone()
    } else {
        apply_bump(&base_semver, bump)
    };

    let target_version = target_semver.to_string();
    let tag = if no_v {
        target_version.clone()
    } else {
        format!("v{}", target_version)
    };

    let dev_version = if is_on_tag {
        target_version.clone()
    } else {
        let mut dev_ver = target_semver.clone();
        let pre_str = format!("dev.{}", commits_count);
        let build_str = format!("{}.{}", repo.head_date, repo.head_short_id);

        dev_ver.pre =
            Prerelease::new(&pre_str).map_err(|e| format!("Invalid prerelease metadata: {}", e))?;
        dev_ver.build =
            BuildMetadata::new(&build_str).map_err(|e| format!("Invalid build metadata: {}", e))?;

        dev_ver.to_string()
    };

    Ok(PredictedVersion {
        base_version: base_str.to_string(),
        target_version,
        dev_version,
        tag,
        bump,
        commits_count,
        is_release: is_on_tag,
    })
}

/// Analyze commit history to determine required SemVer bump magnitude
pub fn detect_bump_from_commits(commits: &[CommitSummary]) -> BumpLevel {
    let mut has_breaking = false;
    let mut has_feat = false;

    for commit in commits {
        // Check breaking change indicators
        if is_breaking_commit(commit) {
            has_breaking = true;
            break; // Highest bump reached
        }

        // Check feature commits
        if is_feature_commit(&commit.summary) {
            has_feat = true;
        }
    }

    if has_breaking {
        BumpLevel::Major
    } else if has_feat {
        BumpLevel::Minor
    } else {
        BumpLevel::Patch
    }
}

/// Check if a commit triggers a BREAKING CHANGE (Major bump)
pub fn is_breaking_commit(commit: &CommitSummary) -> bool {
    let summary = &commit.summary;

    // 1. Indicator `!` before colon in header: e.g. "feat!:", "fix(api)!:"
    if let Some((prefix, _)) = summary.split_once(':') {
        if prefix.ends_with('!') {
            return true;
        }
    }

    // 2. Footer or body contains `BREAKING CHANGE:` or `BREAKING-CHANGE:`
    for line in commit.message.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("BREAKING CHANGE:") || trimmed.starts_with("BREAKING-CHANGE:") {
            return true;
        }
    }

    false
}

/// Check if a commit summary indicates a feature (Minor bump)
pub fn is_feature_commit(summary: &str) -> bool {
    let trimmed = summary.trim();
    if trimmed.starts_with("feat:") {
        return true;
    }
    if trimmed.starts_with("feat(") {
        if let Some((_scope, rest)) = trimmed.split_once(')') {
            return rest.starts_with(':') || rest.starts_with("!:");
        }
    }
    false
}

/// Apply bump rules adhering to SemVer 2.0.0
pub fn apply_bump(base: &Version, bump: BumpLevel) -> Version {
    let mut next = Version::new(base.major, base.minor, base.patch);

    // If base is already a pre-release (e.g. 2.0.0-dev)
    if !base.pre.is_empty() {
        match bump {
            BumpLevel::Major => {
                if base.minor == 0 && base.patch == 0 {
                    // Pre-release major cycle (e.g. 2.0.0-alpha) absorbs breaking changes
                    next.major = base.major;
                    next.minor = 0;
                    next.patch = 0;
                } else {
                    next.major = base.major + 1;
                    next.minor = 0;
                    next.patch = 0;
                }
            }
            BumpLevel::Minor => {
                if base.patch == 0 {
                    next.major = base.major;
                    next.minor = base.minor;
                    next.patch = 0;
                } else {
                    next.major = base.major;
                    next.minor = base.minor + 1;
                    next.patch = 0;
                }
            }
            BumpLevel::Patch | BumpLevel::None => {
                next.major = base.major;
                next.minor = base.minor;
                next.patch = base.patch;
            }
        }
    } else {
        match bump {
            BumpLevel::Major => {
                next.major = base.major + 1;
                next.minor = 0;
                next.patch = 0;
            }
            BumpLevel::Minor => {
                next.minor = base.minor + 1;
                next.patch = 0;
            }
            BumpLevel::Patch => {
                next.patch = base.patch + 1;
            }
            BumpLevel::None => {}
        }
    }

    next
}

/// Write calculated version string to the repository's VERSION file
pub fn write_version_file(repo_root: &Path, version_str: &str) -> std::io::Result<()> {
    let version_path = repo_root.join("VERSION");
    fs::write(version_path, format!("{}\n", version_str.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaking_commit_detection() {
        let c1 = CommitSummary {
            id: "1".into(),
            short_id: "1".into(),
            summary: "feat!: remove old endpoint".into(),
            message: "feat!: remove old endpoint\n\nDetails".into(),
        };
        assert!(is_breaking_commit(&c1));

        let c2 = CommitSummary {
            id: "2".into(),
            short_id: "2".into(),
            summary: "fix(core)!: change return type".into(),
            message: "fix(core)!: change return type".into(),
        };
        assert!(is_breaking_commit(&c2));

        let c3 = CommitSummary {
            id: "3".into(),
            short_id: "3".into(),
            summary: "refactor: restructure models".into(),
            message: "refactor: restructure models\n\nBREAKING CHANGE: User struct changed".into(),
        };
        assert!(is_breaking_commit(&c3));

        let c4 = CommitSummary {
            id: "4".into(),
            short_id: "4".into(),
            summary: "feat: normal feature".into(),
            message: "feat: normal feature".into(),
        };
        assert!(!is_breaking_commit(&c4));
    }

    #[test]
    fn test_feature_commit_detection() {
        assert!(is_feature_commit("feat: add cli"));
        assert!(is_feature_commit("feat(ui): add button"));
        assert!(is_feature_commit("feat(core)!: breaking feature"));
        assert!(!is_feature_commit("fix: resolve bug"));
        assert!(!is_feature_commit("chore: update deps"));
    }

    #[test]
    fn test_apply_bump_progression() {
        let v = Version::new(1, 2, 0);

        // Patch bump
        let patch = apply_bump(&v, BumpLevel::Patch);
        assert_eq!(patch.to_string(), "1.2.1");

        // Minor bump
        let minor = apply_bump(&v, BumpLevel::Minor);
        assert_eq!(minor.to_string(), "1.3.0");

        // Major bump
        let major = apply_bump(&v, BumpLevel::Major);
        assert_eq!(major.to_string(), "2.0.0");
    }
}
