//! Predictive Semantic Versioning (SemVer 2.0.0) Engine.
//! Calculates next target release version and development build metadata
//! based on Git history and Conventional Commits.

use crate::cli::BumpType;
use crate::git::{CommitSummary, GitRepo};
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Write calculated version string to the repository's VERSION file and synchronize Cargo.toml if present
pub fn write_version_file(repo_root: &Path, version_str: &str) -> std::io::Result<()> {
    let version_path = repo_root.join("VERSION");
    fs::write(version_path, format!("{}\n", version_str.trim()))?;
    let _ = sync_cargo_toml_versions(repo_root, version_str);
    Ok(())
}

/// Updates `version = "..."` in a single Cargo.toml file under `[package]` or `[workspace.package]`.
/// Returns true if the file was modified.
pub fn update_cargo_toml_file(file_path: &Path, new_version: &str) -> Result<bool, String> {
    if !file_path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Falha ao ler {}: {}", file_path.display(), e))?;

    let mut new_lines = Vec::new();
    let mut modified = false;
    let mut in_target_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_target_section = trimmed == "[package]" || trimmed == "[workspace.package]";
            new_lines.push(line.to_string());
            continue;
        }

        if in_target_section && trimmed.starts_with("version") {
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0].trim() == "version" {
                let leading_ws = &line[..line.len() - line.trim_start().len()];
                let new_line = format!("{}version = \"{}\"", leading_ws, new_version.trim());
                if line != new_line {
                    modified = true;
                }
                new_lines.push(new_line);
                continue;
            }
        }

        new_lines.push(line.to_string());
    }

    if modified {
        let mut final_content = new_lines.join("\n");
        if content.ends_with('\n') {
            final_content.push('\n');
        }
        fs::write(file_path, final_content)
            .map_err(|e| format!("Falha ao escrever {}: {}", file_path.display(), e))?;
    }

    Ok(modified)
}

/// Sincroniza o arquivo VERSION com os arquivos Cargo.toml do repositório (pacote único ou workspace).
/// Retorna a lista de caminhos relativos dos arquivos modificados.
pub fn sync_cargo_toml_versions(repo_root: &Path, new_version: &str) -> Result<Vec<PathBuf>, String> {
    let root_cargo = repo_root.join("Cargo.toml");
    if !root_cargo.is_file() {
        return Ok(Vec::new());
    }

    let mut modified_files = Vec::new();

    // 1. Atualizar o Cargo.toml raiz se tiver [package] ou [workspace.package]
    if update_cargo_toml_file(&root_cargo, new_version)? {
        modified_files.push(PathBuf::from("Cargo.toml"));
    }

    // 2. Verificar se é workspace e possui membros declarados
    let root_content = fs::read_to_string(&root_cargo)
        .map_err(|e| format!("Falha ao ler Cargo.toml raiz: {}", e))?;

    if let Ok(toml_val) = root_content.parse::<toml::Value>() {
        if let Some(workspace) = toml_val.get("workspace") {
            if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
                for member in members {
                    if let Some(member_str) = member.as_str() {
                        let member_dir = repo_root.join(member_str);
                        if member_dir.is_dir() {
                            let member_cargo = member_dir.join("Cargo.toml");
                            if member_cargo.is_file() && update_cargo_toml_file(&member_cargo, new_version)? {
                                if let Ok(rel) = member_cargo.strip_prefix(repo_root) {
                                    modified_files.push(rel.to_path_buf());
                                } else {
                                    modified_files.push(PathBuf::from(member_str).join("Cargo.toml"));
                                }
                            }
                        } else if member_str.ends_with("/*") {
                            let base_rel = &member_str[..member_str.len() - 2];
                            let base_dir = repo_root.join(base_rel);
                            if let Ok(entries) = fs::read_dir(base_dir) {
                                for entry in entries.flatten() {
                                    let sub_dir = entry.path();
                                    if sub_dir.is_dir() {
                                        let member_cargo = sub_dir.join("Cargo.toml");
                                        if member_cargo.is_file() && update_cargo_toml_file(&member_cargo, new_version)? {
                                            if let Ok(rel) = member_cargo.strip_prefix(repo_root) {
                                                modified_files.push(rel.to_path_buf());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(modified_files)
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
