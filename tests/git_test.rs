//! Exhaustive unit & integration tests for Gitoxide (gix) git reader in CCCP.

use cccp::git::GitRepo;
use std::time::Instant;

#[test]
fn test_git_repo_open_and_read_performance() {
    let start = Instant::now();
    let repo = GitRepo::open(None).expect("Failed to open current repository via gix");
    let elapsed = start.elapsed();

    println!("gix repo open and scan took: {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 250,
        "Repo discovery and commit scan should complete in < 250ms in debug mode (took {:?})",
        elapsed
    );

    // Invariants
    assert!(!repo.head_id.is_empty());
    assert_eq!(repo.head_short_id.len(), 7);
    assert_eq!(repo.head_date.len(), 8); // YYYYMMDD
    assert!(repo.root_dir.exists());
    assert_eq!(repo.branch.as_deref(), Some("rust"));
}

#[test]
fn test_git_repo_commits_since_tag() {
    let repo = GitRepo::open(None).expect("Failed to open current repository via gix");

    // We have commits on branch `rust` since last release tag
    assert!(
        !repo.commits_since_tag.is_empty(),
        "Should find commits since previous release tag"
    );

    for commit in &repo.commits_since_tag {
        assert_eq!(commit.id.len(), 40, "Commit ID must be 40 hex chars");
        assert_eq!(commit.short_id.len(), 7, "Short ID must be 7 chars");
        assert!(!commit.summary.is_empty(), "Summary must not be empty");
        assert!(!commit.message.is_empty(), "Message must not be empty");
    }
}

#[test]
fn test_git_repo_working_tree_inspection() {
    let repo = GitRepo::open(None).expect("Failed to open current repository via gix");
    // We have modified/uncommitted changes in current working tree
    assert!(repo.is_dirty, "Working tree should be detected as dirty");
}
