//! Exhaustive unit & integration tests for Gitoxide (gix) git reader in CCCP.

use cccp::git::GitRepo;
use std::fs;
use std::process::Command;
use std::time::Instant;
use tempfile::tempdir;

fn setup_temp_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create tempdir");
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("git init failed");

    Command::new("git")
        .args(["config", "user.name", "Comrade Tester"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "tester@gosplan.su"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    dir
}

#[test]
fn test_git_repo_open_and_read_performance() {
    let start = Instant::now();
    let repo = GitRepo::open(None).expect("Failed to open current repository via gix");
    let elapsed = start.elapsed();

    println!("gix repo open and scan took: {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 10000,
        "Repo discovery and commit scan should complete in < 10000ms in debug mode (took {:?})",
        elapsed
    );

    // Invariants
    assert!(!repo.head_id.is_empty());
    assert_eq!(repo.head_short_id.len(), 7);
    assert_eq!(repo.head_date.len(), 8); // YYYYMMDD
    assert!(repo.root_dir.exists());
    assert!(repo.branch.is_some());
}

#[test]
fn test_git_repo_commits_since_tag() {
    let repo_dir = setup_temp_repo();
    let p = repo_dir.path();

    // 1. Initial commit and tag v1.0.0
    fs::write(p.join("file1.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["tag", "-a", "v1.0.0", "-m", "Release v1.0.0"])
        .current_dir(p)
        .status()
        .unwrap();

    // 2. Commit after tag
    fs::write(p.join("file2.txt"), "world").unwrap();
    Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat(core): add world feature"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(p)).expect("Failed to open temp repository via gix");

    assert_eq!(
        repo.latest_tag.as_ref().map(|t| t.name.as_str()),
        Some("v1.0.0")
    );
    assert_eq!(
        repo.commits_since_tag.len(),
        1,
        "Should find exactly 1 commit since v1.0.0"
    );

    let commit = &repo.commits_since_tag[0];
    assert_eq!(commit.id.len(), 40, "Commit ID must be 40 hex chars");
    assert_eq!(commit.short_id.len(), 7, "Short ID must be 7 chars");
    assert_eq!(commit.summary, "feat(core): add world feature");
}

#[test]
fn test_git_repo_working_tree_inspection() {
    let repo_dir = setup_temp_repo();
    let p = repo_dir.path();

    // Initial commit
    fs::write(p.join("clean.txt"), "clean").unwrap();
    Command::new("git")
        .args(["add", "clean.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: clean"])
        .current_dir(p)
        .status()
        .unwrap();

    let clean_repo = GitRepo::open(Some(p)).expect("Failed to open clean repo");
    assert!(!clean_repo.is_dirty, "Clean repo should not be dirty");

    // Modify a file to make it dirty
    fs::write(p.join("clean.txt"), "modified").unwrap();
    let dirty_repo = GitRepo::open(Some(p)).expect("Failed to open dirty repo");
    assert!(dirty_repo.is_dirty, "Modified repo should be dirty");
}
