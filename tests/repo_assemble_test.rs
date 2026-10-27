//! Integration tests for selective repo assemble and clean working tree guard (Ticket #65)

use cccp::catalog::{assemble_catalog_repo, assemble_repo, CatalogEntry};
use cccp::cultural::Theme;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn scaffold_test_repo(_name: &str) -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Soviet Tester"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "tester@gost.su"])
        .current_dir(p)
        .status()
        .unwrap();

    fs::write(
        p.join("GOSTVER"),
        "GOST_VERSION=\"2.0.0\"\nGOST_PROFILE_URL=\"https://lilya.2lp.in/profile.md\"\n",
    )
    .unwrap();

    fs::write(p.join("VERSION"), "0.1.0\n").unwrap();
    fs::create_dir_all(p.join(".gost")).unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(p)
        .status()
        .unwrap();

    dir
}

#[test]
fn test_assemble_blocked_when_working_tree_dirty() {
    let repo_dir = scaffold_test_repo("dirty-repo");
    let theme = Theme::with_color(false);

    // Make working tree dirty
    fs::write(repo_dir.path().join("dirty.txt"), "uncommitted work").unwrap();

    let res = assemble_repo(repo_dir.path(), false, false, &theme);
    assert!(res.is_err(), "Dirty working tree must block assemble");
    let err = res.unwrap_err();
    let lower = err.to_lowercase();
    assert!(
        lower.contains("dirty") || lower.contains("modificações") || lower.contains("pendentes"),
        "Error should mention uncommitted changes: {}",
        err
    );
}

#[test]
fn test_assemble_single_repo_success() {
    let repo_dir = scaffold_test_repo("clean-repo");
    let theme = Theme::with_color(false);

    let res = assemble_repo(repo_dir.path(), false, false, &theme);
    assert!(
        res.is_ok(),
        "Assemble on clean repo must succeed: {:?}",
        res.err()
    );

    // Verify .gost/GOST.md was assembled
    assert!(repo_dir.path().join(".gost/GOST.md").is_file());
}

#[test]
fn test_assemble_repo_not_found() {
    let theme = Theme::with_color(false);
    let entries: Vec<CatalogEntry> = Vec::new();
    let res = assemble_catalog_repo(&entries, "nonexistent-repo", false, false, None, &theme);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("não encontrado"));
}

#[test]
fn test_assemble_auto_commit_creates_commit() {
    let repo_dir = scaffold_test_repo("auto-commit-repo");
    let theme = Theme::with_color(false);

    let res = assemble_repo(repo_dir.path(), false, true, &theme);
    assert!(
        res.is_ok(),
        "Assemble with auto-commit must succeed: {:?}",
        res.err()
    );

    // Verify last commit
    let output = Command::new("git")
        .args(["log", "-n", "1", "--oneline"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(
        log.contains("docs(gost)") || log.contains("sincronizar") || log.contains("atualizar"),
        "Commit must be created for gost assembly: {}",
        log
    );
}

#[test]
fn test_assemble_dry_run_leaves_fs_untouched() {
    let repo_dir = scaffold_test_repo("dry-run-repo");
    let theme = Theme::with_color(false);

    let res = assemble_repo(repo_dir.path(), true, false, &theme);
    assert!(res.is_ok());
    assert!(!repo_dir.path().join(".gost/GOST.md").exists(), "dry-run must not create files");
}

#[test]
fn test_assemble_blocked_when_staged_changes() {
    let repo_dir = scaffold_test_repo("staged-repo");
    let theme = Theme::with_color(false);

    fs::write(repo_dir.path().join("staged.txt"), "staged").unwrap();
    Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(repo_dir.path())
        .status()
        .unwrap();

    let res = assemble_repo(repo_dir.path(), false, false, &theme);
    assert!(res.is_err(), "Staged changes must also block assemble");
}

#[test]
fn test_assemble_idempotent_no_duplicate_commit() {
    let repo_dir = scaffold_test_repo("idempotent-repo");
    let theme = Theme::with_color(false);

    // First assemble with auto_commit
    let res1 = assemble_repo(repo_dir.path(), false, true, &theme);
    assert!(res1.is_ok());

    let count1_out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();
    let count1: usize = String::from_utf8_lossy(&count1_out.stdout).trim().parse().unwrap();

    // Second assemble with auto_commit immediately after (no changes to GOST.md)
    let res2 = assemble_repo(repo_dir.path(), false, true, &theme);
    assert!(res2.is_ok());

    let count2_out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();
    let count2: usize = String::from_utf8_lossy(&count2_out.stdout).trim().parse().unwrap();

    assert_eq!(count1, count2, "Idempotent assemble must not create duplicate commits");
}
