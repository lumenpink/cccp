use cccp::commit_exec::{execute_commit, has_staged_changes, CommitOptions};
use cccp::cultural::Theme;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

fn setup_temp_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("Failed to create tempdir");
    let p = dir.path();

    // git init
    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("git init failed");
    assert!(status.success());

    // git config user
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
fn test_has_staged_changes_detection() {
    let repo = setup_temp_repo();
    let p = repo.path();

    // Initially nothing staged
    assert!(!has_staged_changes(Some(p)).unwrap());

    // Create untracked file
    let file_path = p.join("README.md");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "# Soviet Plan").unwrap();

    // Untracked file is still not staged
    assert!(!has_staged_changes(Some(p)).unwrap());

    // Stage the file
    let status = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(p)
        .status()
        .unwrap();
    assert!(status.success());

    // Now has staged changes!
    assert!(has_staged_changes(Some(p)).unwrap());
}

#[test]
fn test_execute_commit_dry_run() {
    let repo = setup_temp_repo();
    let p = repo.path();
    let theme = Theme::init(true);

    let file_path = p.join("plan.txt");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "Five year plan").unwrap();

    Command::new("git")
        .args(["add", "plan.txt"])
        .current_dir(p)
        .status()
        .unwrap();

    let opts = CommitOptions {
        all: false,
        sign: false,
        no_sign: true,
        dry_run: true,
        force: false,
        dir: Some(p.to_path_buf()),
    };

    let code = execute_commit("feat(plan): initial draft", &opts, &theme).unwrap();
    assert_eq!(code, 0);

    // Git log should be empty (no commits yet)
    let log_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(!log_output.status.success());
}

#[test]
fn test_execute_commit_empty_stage_rejection() {
    let repo = setup_temp_repo();
    let p = repo.path();
    let theme = Theme::init(true);

    let opts = CommitOptions {
        all: false,
        sign: false,
        no_sign: true,
        dry_run: false,
        force: false,
        dir: Some(p.to_path_buf()),
    };

    let code = execute_commit("feat(plan): nothing staged", &opts, &theme).unwrap();
    // Must return 1 (warning: no changes staged) without executing commit
    assert_eq!(code, 1);
}

#[test]
fn test_execute_commit_success_verifies_message() {
    let repo = setup_temp_repo();
    let p = repo.path();
    let theme = Theme::init(true);

    let file_path = p.join("decree.txt");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "Decree 227").unwrap();

    Command::new("git")
        .args(["add", "decree.txt"])
        .current_dir(p)
        .status()
        .unwrap();

    let msg = "feat(decree): not a step back\n\nFull description of the decree.\n\nCloses #227";
    let opts = CommitOptions {
        all: false,
        sign: false,
        no_sign: true,
        dry_run: false,
        force: false,
        dir: Some(p.to_path_buf()),
    };

    let code = execute_commit(msg, &opts, &theme).unwrap();
    assert_eq!(code, 0);

    // Verify commit in git log
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%B"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(output.status.success());
    let recorded_msg = String::from_utf8_lossy(&output.stdout);
    assert_eq!(recorded_msg.trim(), msg.trim());
}

#[test]
fn test_execute_commit_with_all_flag() {
    let repo = setup_temp_repo();
    let p = repo.path();
    let theme = Theme::init(true);

    // Create and commit an initial file
    let file_path = p.join("main.rs");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "fn main() {{}}").unwrap();

    Command::new("git")
        .args(["add", "main.rs"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: init", "--no-gpg-sign"])
        .current_dir(p)
        .status()
        .unwrap();

    // Modify tracked file without git add
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "fn main() {{ println!(\"Gosplan\"); }}").unwrap();

    // Confirm nothing staged in index
    assert!(!has_staged_changes(Some(p)).unwrap());

    // Execute with all = true
    let opts = CommitOptions {
        all: true,
        sign: false,
        no_sign: true,
        dry_run: false,
        force: false,
        dir: Some(p.to_path_buf()),
    };

    let code = execute_commit("feat(core): add printing logic", &opts, &theme).unwrap();
    assert_eq!(code, 0);

    // Verify commit message
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .current_dir(p)
        .output()
        .unwrap();
    let recorded_subject = String::from_utf8_lossy(&output.stdout);
    assert_eq!(recorded_subject.trim(), "feat(core): add printing logic");
}
