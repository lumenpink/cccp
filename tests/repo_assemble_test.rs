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
    assert!(
        !repo_dir.path().join(".gost/GOST.md").exists(),
        "dry-run must not create files"
    );
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
    let count1: usize = String::from_utf8_lossy(&count1_out.stdout)
        .trim()
        .parse()
        .unwrap();

    // Second assemble with auto_commit immediately after (no changes to GOST.md)
    let res2 = assemble_repo(repo_dir.path(), false, true, &theme);
    assert!(res2.is_ok());

    let count2_out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(repo_dir.path())
        .output()
        .unwrap();
    let count2: usize = String::from_utf8_lossy(&count2_out.stdout)
        .trim()
        .parse()
        .unwrap();

    assert_eq!(
        count1, count2,
        "Idempotent assemble must not create duplicate commits"
    );
}

#[test]
fn test_assemble_updates_outdated_gostver_and_syncs_gost_artifacts() {
    let repo_dir = scaffold_test_repo("outdated-gostver-repo");
    let p = repo_dir.path();
    let theme = Theme::with_color(false);

    // Overwrite GOSTVER with an outdated version
    fs::write(
        p.join("GOSTVER"),
        "GOST_VERSION=\"1.1.0\"\nGOST_PROFILE_URL=\"profiles/lumen.md\"\nGOST_LANGUAGES=\"shell\"\n",
    )
    .unwrap();

    // Commit change so tree is clean
    Command::new("git")
        .args(["commit", "-am", "chore: set outdated gostver"])
        .current_dir(p)
        .status()
        .unwrap();

    let res = assemble_repo(p, false, false, &theme);
    assert!(res.is_ok(), "assemble_repo must succeed: {:?}", res.err());

    // GOSTVER must be updated to canonical 2.0.0
    let gostver_content = fs::read_to_string(p.join("GOSTVER")).unwrap();
    assert!(
        gostver_content.contains("GOST_VERSION=\"2.0.0\""),
        "GOSTVER must be updated to 2.0.0 during assemble, but was: {}",
        gostver_content
    );

    // .gost/GOST.md must exist
    assert!(p.join(".gost/GOST.md").is_file());

    // .gost/languages/shell.md must be synchronized
    assert!(
        p.join(".gost/languages/shell.md").is_file(),
        "Language spec for shell must be synchronized during assemble"
    );
}

#[test]
fn test_batch_assemble_propagates_gostver_update_and_handles_partial_failure() {
    use cccp::catalog::batch_assemble_gost_repos;

    let repo1 = scaffold_test_repo("batch-repo-1");
    let repo2 = scaffold_test_repo("batch-repo-2");
    let repo3 = scaffold_test_repo("batch-repo-3");
    let theme = Theme::with_color(false);

    let p1 = repo1.path();
    let p2 = repo2.path();
    let p3 = repo3.path();

    // Repo 1 has old GOSTVER 1.0.0
    fs::write(
        p1.join("GOSTVER"),
        "GOST_VERSION=\"1.0.0\"\nGOST_PROFILE_URL=\"profiles/lumen.md\"\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "chore: old gostver"])
        .current_dir(p1)
        .status()
        .unwrap();

    // Repo 2 has a dirty working tree
    fs::write(p2.join("uncommitted.txt"), "dirty").unwrap();

    // Repo 3 has old GOSTVER 1.2.0
    fs::write(
        p3.join("GOSTVER"),
        "GOST_VERSION=\"1.2.0\"\nGOST_PROFILE_URL=\"profiles/lumen.md\"\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "chore: old gostver"])
        .current_dir(p3)
        .status()
        .unwrap();

    let repos = [
        ("batch-repo-1", Some(p1)),
        ("batch-repo-2", Some(p2)),
        ("batch-repo-3", Some(p3)),
    ];

    let report = batch_assemble_gost_repos(&repos, false, &theme);

    // Repo 1 and 3 should assemble successfully
    assert!(report.assembled.contains(&"batch-repo-1".to_string()));
    assert!(report.assembled.contains(&"batch-repo-3".to_string()));

    // Repo 2 should fail due to dirty working tree without corrupting others
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, "batch-repo-2");

    // Repo 1 and 3 GOSTVER must now be 2.0.0
    let g1 = fs::read_to_string(p1.join("GOSTVER")).unwrap();
    assert!(g1.contains("GOST_VERSION=\"2.0.0\""));

    let g3 = fs::read_to_string(p3.join("GOSTVER")).unwrap();
    assert!(g3.contains("GOST_VERSION=\"2.0.0\""));
}
