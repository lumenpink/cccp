//! Red Test for Ticket #156: Expurgar libgit2 e unificar no Git CLI Nativo

use cccp::git::GitRepo;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cargo_toml_must_not_contain_git2_dependency() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path).expect("Failed to read Cargo.toml");

    // RED TEST: Deve falhar enquanto a dependência git2 ainda estiver no Cargo.toml
    let has_git2_dependency = content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("git2 =") || trimmed.starts_with("git2=") || trimmed == "git2"
    });

    assert!(
        !has_git2_dependency,
        "Cargo.toml must NOT contain git2 dependency! All Git operations must be native via CLI."
    );
}

#[test]
fn test_git_cli_native_empty_and_commit_walk() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("git init failed");

    Command::new("git")
        .args(["config", "user.name", "Native Auditor"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "auditor@gosplan.su"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "tag.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    // 1. Repositório vazio
    let empty_repo = GitRepo::open(Some(p)).expect("GitRepo::open empty");
    assert!(empty_repo.commits_since_tag.is_empty());
    assert!(!empty_repo.is_dirty);

    // 2. Commit e tag
    fs::write(p.join("file.txt"), "hello native git\n").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: initial commit"])
        .current_dir(p)
        .status()
        .unwrap();

    Command::new("git")
        .args(["tag", "-a", "v1.0.0", "-m", "release v1.0.0"])
        .current_dir(p)
        .status()
        .unwrap();

    fs::write(p.join("next.txt"), "next version\n").unwrap();
    Command::new("git")
        .args(["add", "next.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "fix: follow up fix"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(p)).expect("GitRepo::open with commits");
    assert_eq!(repo.branch.as_deref(), Some("main"));
    assert!(repo.latest_tag.is_some());
    assert_eq!(repo.latest_tag.as_ref().unwrap().name, "v1.0.0");
    assert_eq!(repo.latest_tag.as_ref().unwrap().version, "1.0.0");
    assert_eq!(repo.commits_since_tag.len(), 1);
    assert_eq!(repo.commits_since_tag[0].summary, "fix: follow up fix");
}

#[test]
fn test_git_cli_mutation_detached_head_and_untracked_files() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Detached Tester"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "detached@gosplan.su"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "tag.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    fs::write(p.join("c1.txt"), "c1\n").unwrap();
    Command::new("git")
        .args(["add", "c1.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: c1"])
        .current_dir(p)
        .status()
        .unwrap();

    let rev_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .unwrap();
    let c1_sha = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();

    // Detached checkout
    Command::new("git")
        .args(["checkout", &c1_sha])
        .current_dir(p)
        .status()
        .unwrap();

    let repo_detached = GitRepo::open(Some(p)).expect("open detached");
    assert_eq!(
        repo_detached.branch, None,
        "Detached HEAD must yield branch None"
    );

    // Untracked file does not mark repo as dirty
    fs::write(p.join("untracked.txt"), "untracked\n").unwrap();
    let repo_untracked = GitRepo::open(Some(p)).expect("open untracked");
    assert!(
        !repo_untracked.is_dirty,
        "Untracked files should not mark working tree as dirty"
    );

    // Tracked modified file marks repo as dirty
    fs::write(p.join("c1.txt"), "c1 modified\n").unwrap();
    let repo_dirty = GitRepo::open(Some(p)).expect("open dirty");
    assert!(
        repo_dirty.is_dirty,
        "Modified tracked file must mark working tree as dirty"
    );
}

#[test]
fn test_git_cli_mutation_ignores_non_semver_tags() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Tag Tester"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "tags@gosplan.su"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "tag.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    fs::write(p.join("file.txt"), "content\n").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(p)
        .status()
        .unwrap();

    // Create non-semver tag
    Command::new("git")
        .args(["tag", "nightly-build-99"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo_non_semver = GitRepo::open(Some(p)).expect("open with non-semver tag");
    assert!(
        repo_non_semver.latest_tag.is_none(),
        "Non-semver tags must be ignored"
    );

    // Create valid semver tag
    Command::new("git")
        .args(["tag", "v2.5.1"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo_valid = GitRepo::open(Some(p)).expect("open with semver tag");
    assert!(repo_valid.latest_tag.is_some());
    assert_eq!(repo_valid.latest_tag.unwrap().version, "2.5.1");
}
