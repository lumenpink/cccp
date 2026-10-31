use std::process::Command;
use tempfile::tempdir;

fn cccp_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cccp"))
}

#[test]
fn test_ticket_divan_with_repo_option_outside_git_repo() {
    let empty_dir = tempdir().expect("Failed to create empty temp dir");

    // Execute 'cccp ticket divan -r lumen/cccp --dry-run' outside any git repo
    let output = cccp_bin()
        .args(["ticket", "divan", "-r", "lumen/cccp", "--dry-run", "test prompt triage"])
        .current_dir(empty_dir.path())
        .output()
        .expect("Failed to execute cccp");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must not fail with 'Git repository not found' from premature GitRepo::open
    assert!(
        !stderr.contains("Git repository not found"),
        "Must not fail with 'Git repository not found' when --repo is provided. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );
}

#[test]
fn test_ticket_divan_with_positional_repo_outside_git_repo() {
    let empty_dir = tempdir().expect("Failed to create empty temp dir");

    // Execute 'cccp ticket divan lumen/cccp "prompt"' outside any git repo
    let output = cccp_bin()
        .args(["ticket", "divan", "lumen/cccp", "--dry-run", "test positional prompt"])
        .current_dir(empty_dir.path())
        .output()
        .expect("Failed to execute cccp");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stderr.contains("Git repository not found"),
        "Must not fail with 'Git repository not found' when positional repo is provided. stderr:\n{}\nstdout:\n{}",
        stderr,
        stdout
    );
}

#[test]
fn test_ticket_divan_without_repo_outside_git_repo_shows_clear_error() {
    let empty_dir = tempdir().expect("Failed to create empty temp dir");

    // Execute 'cccp ticket divan --dry-run' with no repo outside git repo
    let output = cccp_bin()
        .args(["ticket", "divan", "--dry-run"])
        .current_dir(empty_dir.path())
        .output()
        .expect("Failed to execute cccp");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Git repository not found"),
        "Must not fail with raw git error when divan validation can explain missing repo. got:\n{}",
        stderr
    );
    assert!(
        stderr.contains("No target repository specified"),
        "Expected clear validation error when no repo specified outside git repo, got:\n{}",
        stderr
    );
    assert!(!output.status.success());
}
