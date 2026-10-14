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

#[test]
fn test_extract_closes_ids_canonical_syntax() {
    use cccp::hook_runner::extract_closes_ids;

    let text1 = "feat(core): implement stakhanovite worker (closes #133)";
    assert_eq!(extract_closes_ids(text1), vec![133]);

    let text2 = "fix: resolve deadlock (closes #42) and (closes #43)\nMultiple lines (closes #99)";
    assert_eq!(extract_closes_ids(text2), vec![42, 43, 99]);

    // Invalid non-canonical syntax should NOT be extracted
    let text3 = "fix: close #123, closes 456, fix #789 without parens";
    assert_eq!(extract_closes_ids(text3), Vec::<u64>::new());
}

#[test]
fn test_hook_pre_commit_updates_version_and_stages() {
    use cccp::hook_runner::run_pre_commit;

    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let theme = Theme::init(true);

    // Initial commit
    fs::write(p.join("VERSION"), "0.1.0-dev.1\n").unwrap();
    Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial setup"])
        .current_dir(p)
        .status()
        .unwrap();

    // Stage a new change
    fs::write(p.join("file.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(p)).expect("open repo");
    let code = run_pre_commit(&repo, &theme, 0).expect("pre-commit hook failed");
    assert_eq!(code, 0);

    // VERSION file must have been updated to dev.2
    let version_content = fs::read_to_string(p.join("VERSION")).unwrap();
    assert!(
        version_content.contains("dev.2"),
        "Expected VERSION to contain dev.2, got: {}",
        version_content
    );

    // VERSION must be in git index (staged)
    let staged = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(p)
        .output()
        .unwrap();
    let staged_str = String::from_utf8_lossy(&staged.stdout);
    assert!(
        staged_str.contains("VERSION"),
        "Expected VERSION to be staged, got: {}",
        staged_str
    );
}

#[test]
fn test_reconcile_merge_version_behavior() {
    use cccp::hook_runner::reconcile_merge_version;

    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let theme = Theme::init(true);

    // Base commit
    fs::write(p.join("VERSION"), "0.1.0\n").unwrap();
    Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["tag", "-a", "v0.1.0", "-m", "release 0.1.0"])
        .current_dir(p)
        .status()
        .unwrap();

    // Second commit
    fs::write(p.join("a.txt"), "a").unwrap();
    Command::new("git")
        .args(["add", "a.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: add a"])
        .current_dir(p)
        .status()
        .unwrap();

    // Now repo has 1 commit since v0.1.0, so calculated dev version is 0.2.0-dev.1+...
    // But VERSION file still has "0.1.0"
    let repo = GitRepo::open(Some(p)).expect("open repo");
    let result = reconcile_merge_version(&repo, &theme).expect("reconcile failed");
    assert!(
        result.is_some(),
        "Expected reconcile to update version and create commit"
    );

    let version_content = fs::read_to_string(p.join("VERSION")).unwrap();
    assert!(version_content.contains("0.2.0-dev.2"));

    // Check that a commit was created with chore(version)
    let log = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .current_dir(p)
        .output()
        .unwrap();
    let subject = String::from_utf8_lossy(&log.stdout);
    assert!(subject.starts_with("chore(version):"));

    // If we call reconcile again when it already matches, it should return Ok(None) and not create another commit
    let repo2 = GitRepo::open(Some(p)).expect("open repo");
    let result2 = reconcile_merge_version(&repo2, &theme).expect("reconcile failed");
    assert!(result2.is_none());
}
