use cccp::config::Config;
use cccp::cultural::Theme;
use cccp::git::GitRepo;
use cccp::hook_runner::{run_commit_msg, run_post_commit, run_pre_push, run_reference_transaction};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup_temp_git_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create tempdir");
    let p = dir.path();

    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("git init failed");
    assert!(status.success());

    Command::new("git")
        .args(["config", "user.name", "Gosplan Auditor"])
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

    dir
}

#[test]
fn test_hook_commit_msg_valid() {
    let dir = tempdir().unwrap();
    let msg_file = dir.path().join("COMMIT_EDITMSG");
    fs::write(&msg_file, "feat(hook): fast hook runner implementation\n").unwrap();

    let config = Config::default();
    let theme = Theme::init(true);

    let code = run_commit_msg(&msg_file, &config, &theme, 0).expect("hook failed");
    assert_eq!(code, 0);
}

#[test]
fn test_hook_commit_msg_invalid() {
    let dir = tempdir().unwrap();
    let msg_file = dir.path().join("COMMIT_EDITMSG");
    fs::write(&msg_file, "bad commit message with trailing dot.\n").unwrap();

    let config = Config::default();
    let theme = Theme::init(true);

    let code = run_commit_msg(&msg_file, &config, &theme, 0).expect("hook failed");
    assert_eq!(code, 1);
}

#[test]
fn test_hook_commit_msg_missing_file_errors() {
    let dir = tempdir().unwrap();
    let msg_file = dir.path().join("NON_EXISTENT");

    let config = Config::default();
    let theme = Theme::init(true);

    let res = run_commit_msg(&msg_file, &config, &theme, 0);
    assert!(res.is_err());
}

#[test]
fn test_hook_pre_push_clean_matching_tag() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let theme = Theme::init(true);
    let config = Config::default();

    // Create VERSION file with 1.0.0
    fs::write(p.join("VERSION"), "1.0.0\n").unwrap();
    Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: release 1.0.0"])
        .current_dir(p)
        .status()
        .unwrap();

    let head_sha = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(p)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();

    let stdin_input = format!(
        "refs/heads/main {} refs/tags/v1.0.0 0000000000000000000000000000000000000000\n",
        head_sha
    );

    let repo = GitRepo::open(Some(p)).unwrap();
    let code = run_pre_push(
        Some("origin"),
        Some("git@codeberg.org:test/repo.git"),
        Some(&stdin_input),
        &repo,
        &config,
        &theme,
        0,
    )
    .unwrap();

    assert_eq!(code, 0);
}

#[test]
fn test_hook_pre_push_rejects_tag_missing_version_file() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let theme = Theme::init(true);
    let config = Config::default();

    // Commit file without VERSION
    fs::write(p.join("README.md"), "# Hello\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(p)
        .status()
        .unwrap();

    let head_sha = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(p)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();

    let stdin_input = format!(
        "refs/heads/main {} refs/tags/v1.0.0 0000000000000000000000000000000000000000\n",
        head_sha
    );

    let repo = GitRepo::open(Some(p)).unwrap();
    let code = run_pre_push(
        Some("origin"),
        Some("git@codeberg.org:test/repo.git"),
        Some(&stdin_input),
        &repo,
        &config,
        &theme,
        0,
    )
    .unwrap();

    // Must be rejected (code 1)
    assert_eq!(code, 1);
}

#[test]
fn test_hook_pre_push_rejects_tag_with_dev_version() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let theme = Theme::init(true);
    let config = Config::default();

    fs::write(p.join("VERSION"), "1.0.0-dev.5+hash\n").unwrap();
    Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: dev commit"])
        .current_dir(p)
        .status()
        .unwrap();

    let head_sha = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(p)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();

    let stdin_input = format!(
        "refs/heads/main {} refs/tags/v1.0.0 0000000000000000000000000000000000000000\n",
        head_sha
    );

    let repo = GitRepo::open(Some(p)).unwrap();
    let code = run_pre_push(
        Some("origin"),
        Some("git@codeberg.org:test/repo.git"),
        Some(&stdin_input),
        &repo,
        &config,
        &theme,
        0,
    )
    .unwrap();

    assert_eq!(code, 1);
}

#[test]
fn test_hook_post_commit_and_reference_transaction() {
    let theme = Theme::init(true);

    assert_eq!(run_post_commit(&theme, 0).unwrap(), 0);
    assert_eq!(
        run_reference_transaction(Some("committed"), &theme, 0).unwrap(),
        0
    );
}
