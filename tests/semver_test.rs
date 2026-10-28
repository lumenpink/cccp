//! Exhaustive test suite for CCCP Predictive Semantic Versioning (SemVer 2.0.0).

use cccp::cli::BumpType;
use cccp::git::{CommitSummary, GitRepo, TagInfo};
use cccp::semver_calc::{
    apply_bump, calculate_version, detect_bump_from_commits, is_breaking_commit, is_feature_commit,
    write_version_file, BumpLevel,
};
use semver::Version;
use tempfile::tempdir;

fn mock_commit(summary: &str, body: Option<&str>) -> CommitSummary {
    let msg = match body {
        Some(b) => format!("{}\n\n{}", summary, b),
        None => summary.to_string(),
    };
    CommitSummary {
        id: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        short_id: "abcdef1".to_string(),
        summary: summary.to_string(),
        message: msg,
    }
}

fn mock_repo(tag: Option<(&str, &str)>, commits: Vec<CommitSummary>, is_dirty: bool) -> GitRepo {
    GitRepo {
        root_dir: std::path::PathBuf::from("/mock/repo"),
        branch: Some("main".to_string()),
        head_id: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        head_short_id: "abcdef1".to_string(),
        head_date: "20260930".to_string(),
        latest_tag: tag.map(|(name, ver)| TagInfo {
            name: name.to_string(),
            version: ver.to_string(),
            commit_id: "1111111111111111111111111111111111111111".to_string(),
        }),
        commits_since_tag: commits,
        is_dirty,
        head_parent_count: 1,
    }
}

#[test]
fn test_commit_classifiers() {
    let brk = mock_commit("feat!: breaking header", None);
    assert!(is_breaking_commit(&brk));
    assert!(is_feature_commit("feat: standard feature"));
    assert!(!is_feature_commit("fix: standard fix"));
}

#[test]
fn test_semver_no_prior_tags_bootstrap() {
    let commits = vec![
        mock_commit("fix: initial fix", None),
        mock_commit("chore: init repo", None),
    ];
    let repo = mock_repo(None, commits, false);

    let res = calculate_version(&repo, "0.1.0", false, None).expect("Calculation failed");
    assert_eq!(res.base_version, "0.1.0");
    assert_eq!(res.target_version, "0.1.1");
    assert_eq!(res.tag, "v0.1.1");
    assert_eq!(res.bump, BumpLevel::Patch);
    assert_eq!(res.commits_count, 2);
    assert_eq!(res.dev_version, "0.1.1-dev.2+20260930.abcdef1");
    assert!(!res.is_release);
}

#[test]
fn test_semver_patch_progression() {
    let commits = vec![
        mock_commit("fix: resolve off-by-one error", None),
        mock_commit("perf: optimize memory pool", None),
        mock_commit("docs: update architecture diagram", None),
        mock_commit("refactor: extract helper function", None),
    ];
    let repo = mock_repo(Some(("v1.2.0", "1.2.0")), commits, false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.base_version, "1.2.0");
    assert_eq!(res.target_version, "1.2.1");
    assert_eq!(res.tag, "v1.2.1");
    assert_eq!(res.bump, BumpLevel::Patch);
    assert_eq!(res.commits_count, 4);
    assert_eq!(res.dev_version, "1.2.1-dev.4+20260930.abcdef1");
}

#[test]
fn test_semver_minor_progression() {
    let commits = vec![
        mock_commit("docs: update guide", None),
        mock_commit("feat: add new CLI parser engine", None),
        mock_commit("fix: edge case handling", None),
    ];
    let repo = mock_repo(Some(("v1.2.0", "1.2.0")), commits, false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.base_version, "1.2.0");
    assert_eq!(res.target_version, "1.3.0");
    assert_eq!(res.tag, "v1.3.0");
    assert_eq!(res.bump, BumpLevel::Minor);
    assert_eq!(res.commits_count, 3);
    assert_eq!(res.dev_version, "1.3.0-dev.3+20260930.abcdef1");
}

#[test]
fn test_semver_major_progression_bang_indicator() {
    let commits = vec![
        mock_commit("fix: minor fix", None),
        mock_commit("feat(api)!: remove deprecated v1 endpoint", None),
    ];
    let repo = mock_repo(Some(("v1.2.0", "1.2.0")), commits, false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.base_version, "1.2.0");
    assert_eq!(res.target_version, "2.0.0");
    assert_eq!(res.tag, "v2.0.0");
    assert_eq!(res.bump, BumpLevel::Major);
    assert_eq!(res.commits_count, 2);
    assert_eq!(res.dev_version, "2.0.0-dev.2+20260930.abcdef1");
}

#[test]
fn test_semver_major_progression_breaking_change_footer() {
    let commits = vec![
        mock_commit(
            "refactor: overhaul state machine",
            Some("BREAKING CHANGE: All public callbacks require Context argument"),
        ),
        mock_commit("chore: update build script", None),
    ];
    let repo = mock_repo(Some(("v3.1.4", "3.1.4")), commits, false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.target_version, "4.0.0");
    assert_eq!(res.bump, BumpLevel::Major);
}

#[test]
fn test_semver_major_progression_breaking_hyphen_footer() {
    let commits = vec![mock_commit(
        "fix(core): alter data schema",
        Some("BREAKING-CHANGE: Schema version 2 is now mandatory"),
    )];
    let repo = mock_repo(Some(("v2.5.1", "2.5.1")), commits, false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.target_version, "3.0.0");
    assert_eq!(res.bump, BumpLevel::Major);
}

#[test]
fn test_semver_combined_weight_precedence() {
    // 5 chores, 3 fixes, 2 features, 1 breaking change -> MAJOR wins
    let mut commits = Vec::new();
    for i in 0..5 {
        commits.push(mock_commit(&format!("chore: maintenance {}", i), None));
    }
    for i in 0..3 {
        commits.push(mock_commit(&format!("fix: bugfix {}", i), None));
    }
    for i in 0..2 {
        commits.push(mock_commit(&format!("feat: feature {}", i), None));
    }
    commits.push(mock_commit("refactor!: revolutionary change", None));

    assert_eq!(detect_bump_from_commits(&commits), BumpLevel::Major);

    // Remove the breaking commit -> MINOR wins over patch
    commits.pop();
    assert_eq!(detect_bump_from_commits(&commits), BumpLevel::Minor);

    // Remove the feature commits -> PATCH wins over chores/docs
    commits.pop();
    commits.pop();
    assert_eq!(detect_bump_from_commits(&commits), BumpLevel::Patch);
}

#[test]
fn test_semver_pre_release_absorption_rule() {
    let base_major_dev = Version::parse("2.0.0-dev").unwrap();
    // In a 2.0.0-dev cycle, breaking change is absorbed into 2.0.0
    let bumped = apply_bump(&base_major_dev, BumpLevel::Major);
    assert_eq!(bumped.to_string(), "2.0.0");

    let base_minor_dev = Version::parse("2.1.0-dev").unwrap();
    // In a 2.1.0-dev cycle, feature is absorbed into 2.1.0
    let bumped = apply_bump(&base_minor_dev, BumpLevel::Minor);
    assert_eq!(bumped.to_string(), "2.1.0");

    // In a 2.1.0-dev cycle, breaking change forces next major 3.0.0
    let bumped = apply_bump(&base_minor_dev, BumpLevel::Major);
    assert_eq!(bumped.to_string(), "3.0.0");
}

#[test]
fn test_semver_on_release_tag_clean() {
    // 0 commits since tag, clean working tree
    let repo = mock_repo(Some(("v2.3.0", "2.3.0")), Vec::new(), false);

    let res = calculate_version(&repo, "0.0.1", false, None).expect("Calculation failed");
    assert_eq!(res.base_version, "2.3.0");
    assert_eq!(res.target_version, "2.3.0");
    assert_eq!(res.dev_version, "2.3.0");
    assert_eq!(res.tag, "v2.3.0");
    assert_eq!(res.bump, BumpLevel::None);
    assert_eq!(res.commits_count, 0);
    assert!(res.is_release);
}

#[test]
fn test_semver_manual_bump_overrides() {
    let commits = vec![mock_commit("fix: small fix", None)];
    let repo = mock_repo(Some(("v1.0.0", "1.0.0")), commits, false);

    // Override to Major
    let res_major = calculate_version(&repo, "0.0.1", false, Some(BumpType::Major)).unwrap();
    assert_eq!(res_major.target_version, "2.0.0");
    assert_eq!(res_major.bump, BumpLevel::Major);

    // Override to Minor
    let res_minor = calculate_version(&repo, "0.0.1", false, Some(BumpType::Minor)).unwrap();
    assert_eq!(res_minor.target_version, "1.1.0");
    assert_eq!(res_minor.bump, BumpLevel::Minor);

    // Override to Patch
    let res_patch = calculate_version(&repo, "0.0.1", false, Some(BumpType::Patch)).unwrap();
    assert_eq!(res_patch.target_version, "1.0.1");
    assert_eq!(res_patch.bump, BumpLevel::Patch);
}

#[test]
fn test_semver_no_v_formatting() {
    let commits = vec![mock_commit("feat: add feature", None)];
    let repo = mock_repo(Some(("1.2.0", "1.2.0")), commits, false);

    let res = calculate_version(&repo, "0.0.1", true, None).unwrap();
    assert_eq!(res.target_version, "1.3.0");
    assert_eq!(
        res.tag, "1.3.0",
        "Tag must not have 'v' prefix when no_v is true"
    );
}

#[test]
fn test_write_version_file_hermetic() {
    let dir = tempdir().expect("Failed to create tempdir");
    let version_content = "2.4.0-dev.15+20260930.abcdef1";

    write_version_file(dir.path(), version_content).expect("Failed to write VERSION file");

    let read_back =
        std::fs::read_to_string(dir.path().join("VERSION")).expect("Failed to read VERSION file");
    assert_eq!(read_back, format!("{}\n", version_content));
}
