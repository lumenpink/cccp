//! Integration tests for Git manual command locks and origin parity check on push (Ticket #64)

use cccp::cultural::Theme;
use cccp::git::GitRepo;
use cccp::hook_runner::{run_pre_commit, run_pre_push};
use cccp::push::{check_and_reconcile_parity, ParityAction};
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tempfile::tempdir;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn scaffold_test_git_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    // git init
    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("git init");
    assert!(status.success());

    // git config
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

    // Initial commit
    fs::write(p.join("VERSION"), "0.1.0\n").unwrap();
    Command::new("git")
        .args(["add", "VERSION"])
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
fn test_manual_commit_blocked_without_maestro() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let repo_dir = scaffold_test_git_repo();
    let repo = GitRepo::open(Some(repo_dir.path())).expect("open repo");
    let theme = Theme::with_color(false);

    // Make sure CCCP_MAESTRO_ACTIVE and bypass variables are cleared
    std::env::remove_var("CCCP_MAESTRO_ACTIVE");
    std::env::remove_var("CCCP_BYPASS_LOCKS");
    std::env::remove_var("CI");

    let result = run_pre_commit(&repo, &theme, 0);
    assert!(
        result.is_ok(),
        "Hook runner execution returned unexpected error"
    );
    assert_eq!(
        result.unwrap(),
        1,
        "pre-commit hook must reject manual git commit without CCCP maestro"
    );
}

#[test]
fn test_commit_allowed_with_maestro_or_bypass() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let repo_dir = scaffold_test_git_repo();
    let repo = GitRepo::open(Some(repo_dir.path())).expect("open repo");
    let theme = Theme::with_color(false);

    // 1. With CCCP_MAESTRO_ACTIVE=1
    std::env::set_var("CCCP_MAESTRO_ACTIVE", "1");
    let res_maestro = run_pre_commit(&repo, &theme, 0);
    assert_eq!(
        res_maestro.unwrap(),
        0,
        "pre-commit must succeed when CCCP_MAESTRO_ACTIVE=1"
    );

    // 2. With CCCP_BYPASS_LOCKS=1
    std::env::remove_var("CCCP_MAESTRO_ACTIVE");
    std::env::set_var("CCCP_BYPASS_LOCKS", "1");
    let res_bypass = run_pre_commit(&repo, &theme, 0);
    assert_eq!(
        res_bypass.unwrap(),
        0,
        "pre-commit must succeed when CCCP_BYPASS_LOCKS=1"
    );

    std::env::remove_var("CCCP_BYPASS_LOCKS");
}

#[test]
fn test_manual_push_blocked_without_maestro() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let repo_dir = scaffold_test_git_repo();
    let repo = GitRepo::open(Some(repo_dir.path())).expect("open repo");
    let theme = Theme::with_color(false);
    let config = cccp::config::Config::default();

    std::env::remove_var("CCCP_MAESTRO_ACTIVE");
    std::env::remove_var("CCCP_BYPASS_LOCKS");
    std::env::remove_var("CI");

    // Standard pre-push stdin: <local ref> <local sha1> <remote ref> <remote sha1>
    let stdin_payload = "refs/heads/main 111111 refs/heads/main 000000\n";
    let result = run_pre_push(
        Some("origin"),
        Some("git@forge.su:repo.git"),
        Some(stdin_payload),
        &repo,
        &config,
        &theme,
        0,
    );

    assert!(
        result.is_ok(),
        "Hook runner execution returned unexpected error"
    );
    assert_eq!(
        result.unwrap(),
        1,
        "pre-push hook must reject manual git push without CCCP maestro"
    );

    // With CCCP_MAESTRO_ACTIVE=1
    std::env::set_var("CCCP_MAESTRO_ACTIVE", "1");
    let result_allowed = run_pre_push(
        Some("origin"),
        Some("git@forge.su:repo.git"),
        Some(stdin_payload),
        &repo,
        &config,
        &theme,
        0,
    );
    assert_eq!(
        result_allowed.unwrap(),
        0,
        "pre-push must pass when CCCP_MAESTRO_ACTIVE=1"
    );

    std::env::remove_var("CCCP_MAESTRO_ACTIVE");
}

#[test]
fn test_push_parity_tracking_branch_ahead() {
    let local_dir = scaffold_test_git_repo();
    let remote_dir = tempdir().expect("remote dir");

    // Create bare remote repository
    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path())
        .status()
        .unwrap();

    // Add remote to local
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Push initial branch
    Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Add another commit locally
    fs::write(local_dir.path().join("file.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: new local file"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    let theme = Theme::with_color(false);
    let action = check_and_reconcile_parity(local_dir.path(), &theme).expect("parity check");
    assert_eq!(action, ParityAction::ReadyToPush);
}

#[test]
fn test_push_auto_fast_forward_when_behind() {
    let local_dir = scaffold_test_git_repo();
    let remote_dir = tempdir().expect("remote dir");

    // Bare remote
    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path())
        .status()
        .unwrap();

    // Setup local tracking origin
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .current_dir(local_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Clone a second worker to simulate someone else pushing to origin
    let peer_dir = tempdir().expect("peer dir");
    Command::new("git")
        .args([
            "clone",
            remote_dir.path().to_str().unwrap(),
            peer_dir.path().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Peer Worker"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "peer@gost.su"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();

    fs::write(peer_dir.path().join("peer_file.txt"), "remote work").unwrap();
    Command::new("git")
        .args(["add", "peer_file.txt"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: peer remote commit"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();

    // Now local_dir is behind origin by 1 commit and ahead by 0 commits
    let theme = Theme::with_color(false);
    let action = check_and_reconcile_parity(local_dir.path(), &theme).expect("parity check");
    assert_eq!(action, ParityAction::FastForwarded);

    // Verify peer_file.txt is now present in local_dir after auto-fast-forward!
    assert!(local_dir.path().join("peer_file.txt").is_file());
}

#[test]
fn test_push_blocked_on_divergent_history() {
    let local_dir = scaffold_test_git_repo();
    let remote_dir = tempdir().expect("remote dir");

    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .current_dir(local_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    // Peer pushes a commit
    let peer_dir = tempdir().expect("peer dir");
    Command::new("git")
        .args([
            "clone",
            remote_dir.path().to_str().unwrap(),
            peer_dir.path().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Peer Worker"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "peer@gost.su"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    fs::write(peer_dir.path().join("remote.txt"), "remote").unwrap();
    Command::new("git")
        .args(["add", "remote.txt"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: peer remote commit"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(peer_dir.path())
        .status()
        .unwrap();

    // Local ALSO makes a different commit on main (divergence)
    fs::write(local_dir.path().join("local.txt"), "local").unwrap();
    Command::new("git")
        .args(["add", "local.txt"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: local commit"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    let theme = Theme::with_color(false);
    let res = check_and_reconcile_parity(local_dir.path(), &theme);
    assert!(
        res.is_err(),
        "Divergent history must return error / block push"
    );
    let err_msg = res.unwrap_err();
    let lower_msg = err_msg.to_lowercase();
    assert!(
        lower_msg.contains("divergência")
            || lower_msg.contains("divergent")
            || lower_msg.contains("behind"),
        "Error message should explain divergent branches: {}",
        err_msg
    );
}

#[test]
fn test_push_parity_already_up_to_date() {
    let local_dir = scaffold_test_git_repo();
    let remote_dir = tempdir().expect("remote dir");

    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path())
        .status()
        .unwrap();

    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(local_dir.path())
        .status()
        .unwrap();

    let theme = Theme::with_color(false);
    let action = check_and_reconcile_parity(local_dir.path(), &theme).expect("parity check");
    assert_eq!(action, ParityAction::AlreadyUpToDate);
}

#[test]
fn test_push_parity_no_remote_configured() {
    let local_dir = scaffold_test_git_repo();
    let theme = Theme::with_color(false);

    let action = check_and_reconcile_parity(local_dir.path(), &theme).expect("parity check");
    assert_eq!(action, ParityAction::NoRemote);
}

#[test]
fn test_ci_environment_bypasses_locks() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let repo_dir = scaffold_test_git_repo();
    let repo = GitRepo::open(Some(repo_dir.path())).expect("open repo");
    let theme = Theme::with_color(false);
    let config = cccp::config::Config::default();

    std::env::remove_var("CCCP_MAESTRO_ACTIVE");
    std::env::remove_var("CCCP_BYPASS_LOCKS");

    // GITHUB_ACTIONS=true
    std::env::set_var("GITHUB_ACTIONS", "true");
    let commit_res = run_pre_commit(&repo, &theme, 0);
    assert_eq!(commit_res.unwrap(), 0);

    let push_res = run_pre_push(
        Some("origin"),
        Some("git@forge.su:repo.git"),
        Some("refs/heads/main 111111 refs/heads/main 000000\n"),
        &repo,
        &config,
        &theme,
        0,
    );
    assert_eq!(push_res.unwrap(), 0);

    std::env::remove_var("GITHUB_ACTIONS");

    // FORGEJO_ACTIONS=true
    std::env::set_var("FORGEJO_ACTIONS", "true");
    let commit_res2 = run_pre_commit(&repo, &theme, 0);
    assert_eq!(commit_res2.unwrap(), 0);

    let push_res2 = run_pre_push(
        Some("origin"),
        Some("git@forge.su:repo.git"),
        Some("refs/heads/main 111111 refs/heads/main 000000\n"),
        &repo,
        &config,
        &theme,
        0,
    );
    assert_eq!(push_res2.unwrap(), 0);

    std::env::remove_var("FORGEJO_ACTIONS");
}
