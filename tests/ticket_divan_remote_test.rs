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
        .args([
            "ticket",
            "divan",
            "-r",
            "lumen/cccp",
            "--dry-run",
            "test prompt triage",
        ])
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
        .args([
            "ticket",
            "divan",
            "lumen/cccp",
            "--dry-run",
            "test positional prompt",
        ])
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

#[test]
fn test_mutation_ticket_divan_nonexistent_repo_error_message() {
    let empty_dir = tempdir().expect("Failed to create empty temp dir");

    // Execute 'cccp ticket divan -r non_existent_repo_slug --dry-run'
    let output = cccp_bin()
        .args([
            "ticket",
            "divan",
            "-r",
            "non_existent_repo_slug_xyz",
            "--dry-run",
            "prompt",
        ])
        .current_dir(empty_dir.path())
        .output()
        .expect("Failed to execute cccp");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(!stderr.contains("Git repository not found"));
    assert!(
        stderr.contains("not found in catalog or workspaces"),
        "Expected error message indicating repo not found in catalog, got:\n{}",
        stderr
    );
}

#[test]
fn test_mutation_divan_args_resolve_repo_and_prompt_combinations() {
    use cccp::cli::DivanArgs;

    // 1. --repo with no positional args
    let args1 = DivanArgs {
        repo: Some("owner/repo".to_string()),
        ..Default::default()
    };
    let (r1, p1) = args1.resolve_repo_and_prompt();
    assert_eq!(r1.as_deref(), Some("owner/repo"));
    assert_eq!(p1, None);

    // 2. --repo with single positional prompt
    let args2 = DivanArgs {
        repo: Some("owner/repo".to_string()),
        target: Some("fix login bug".to_string()),
        ..Default::default()
    };
    let (r2, p2) = args2.resolve_repo_and_prompt();
    assert_eq!(r2.as_deref(), Some("owner/repo"));
    assert_eq!(p2.as_deref(), Some("fix login bug"));

    // 3. Single positional without whitespace -> inferred as repo
    let args3 = DivanArgs {
        target: Some("custom-app".to_string()),
        ..Default::default()
    };
    let (r3, p3) = args3.resolve_repo_and_prompt();
    assert_eq!(r3.as_deref(), Some("custom-app"));
    assert_eq!(p3, None);

    // 4. Single positional with whitespace -> inferred as prompt
    let args4 = DivanArgs {
        target: Some("fix login bug on mobile".to_string()),
        ..Default::default()
    };
    let (r4, p4) = args4.resolve_repo_and_prompt();
    assert_eq!(r4, None);
    assert_eq!(p4.as_deref(), Some("fix login bug on mobile"));
}
