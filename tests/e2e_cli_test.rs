//! End-to-End (E2E) integration tests for the compiled cccp standalone binary.
//!
//! Validates actual execution of the compiled binary via `std::process::Command`,
//! simulating end-user workflows:
//! - Root commands and help
//! - Gosplan cultural registry
//! - Shell completions
//! - Repository scaffolding (`init`)
//! - Commit message linting (`lint`)
//! - Git hooks lifecycle (`hook --install` / `--uninstall`)
//! - Directory override (`-C / --dir`)
//! - Predictive Semantic Versioning (`version`)
//! - ANSI color suppression (`--no-color`)
//! - Configuration get and set (`config`)

use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn cccp_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cccp"))
}

// Helper to create an initialized git repository in a temp directory
fn setup_temp_git_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("failed to create temp dir");
    let status = Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir.path())
        .output()
        .expect("failed to execute git init");
    assert!(status.status.success(), "git init must succeed");

    // Configure user identity for the temp repository
    Command::new("git")
        .args(["config", "user.name", "Test Comrade"])
        .current_dir(dir.path())
        .output()
        .expect("git config user.name");
    Command::new("git")
        .args(["config", "user.email", "comrade@gosplan.su"])
        .current_dir(dir.path())
        .output()
        .expect("git config user.email");

    dir
}

#[test]
fn test_e2e_binary_version_and_help() {
    // Test --version
    let output = cccp_bin()
        .arg("--version")
        .output()
        .expect("failed to run cccp --version");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cccp 3.0.0"), "stdout was: {}", stdout);

    // Test --help
    let output = cccp_bin()
        .arg("--help")
        .output()
        .expect("failed to run cccp --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Conventional Commits Compliance Program"),
        "stdout was: {}",
        stdout
    );
    assert!(stdout.contains("commit"), "must list commit command");
    assert!(stdout.contains("version"), "must list version command");
    assert!(stdout.contains("hook"), "must list hook command");
}

#[test]
fn test_e2e_soviet_easter_eggs() {
    // Quote
    let output = cccp_bin()
        .args(["soviet", "--quote", "--no-color"])
        .output()
        .expect("failed to run cccp soviet --quote");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("—") || stdout.contains("Lenin") || stdout.contains("Marx"),
        "stdout was: {}",
        stdout
    );

    // Order 227
    let output = cccp_bin()
        .args(["soviet", "--order227", "--no-color"])
        .output()
        .expect("failed to run cccp soviet --order227");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("227") && stdout.contains("НИ ШАГУ НАЗАД!"),
        "stdout was: {}",
        stdout
    );

    // Anthem
    let output = cccp_bin()
        .args(["soviet", "--anthem", "--no-color"])
        .output()
        .expect("failed to run cccp soviet --anthem");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Союз нерушимый") || stdout.contains("Soviet Union"),
        "stdout was: {}",
        stdout
    );

    // Plan
    let output = cccp_bin()
        .args(["soviet", "--plan", "--no-color"])
        .output()
        .expect("failed to run cccp soviet --plan");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FIVE-YEAR PLAN"), "stdout was: {}", stdout);
}

#[test]
fn test_e2e_shell_completion() {
    // Bash
    let output = cccp_bin()
        .args(["completion", "bash"])
        .output()
        .expect("failed to run cccp completion bash");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("_cccp()"),
        "bash completion must define _cccp"
    );

    // Zsh
    let output = cccp_bin()
        .args(["completion", "zsh"])
        .output()
        .expect("failed to run cccp completion zsh");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("compdef _cccp"),
        "zsh completion must contain compdef"
    );

    // Fish
    let output = cccp_bin()
        .args(["completion", "fish"])
        .output()
        .expect("failed to run cccp completion fish");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("complete -c cccp"),
        "fish completion must register completions"
    );
}

#[test]
fn test_e2e_init_scaffolding() {
    let dir = tempdir().expect("failed to create temp dir");
    let output = cccp_bin()
        .args(["init", "gost-service", "--no-git", "--quiet"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run cccp init");
    assert!(output.status.success());

    let target_dir = dir.path().join("gost-service");
    assert!(
        target_dir.is_dir(),
        "created directory {} must exist",
        target_dir.display()
    );
    assert!(
        target_dir.join("README.md").is_file(),
        "README.md must exist"
    );
    assert!(target_dir.join("LICENSE").is_file(), "LICENSE must exist");
    assert!(
        target_dir.join("GOSTVER").is_file(),
        "GOSTVER declaration must exist"
    );
    assert!(
        target_dir.join("VERSION").is_file(),
        "VERSION metadata must exist"
    );
}

#[test]
fn test_e2e_lint_direct_messages() {
    // Valid feat message -> exit code 0
    let output = cccp_bin()
        .args(["lint", "-m", "feat(core): implement high-speed pipeline"])
        .output()
        .expect("failed to run cccp lint");
    assert!(
        output.status.success(),
        "valid commit must pass with code 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compliant"), "stdout was: {}", stdout);

    // Invalid message -> exit code 1
    let output = cccp_bin()
        .args(["lint", "-m", "invalid commit without type"])
        .output()
        .expect("failed to run cccp lint invalid");
    assert!(
        !output.status.success(),
        "invalid commit must fail with non-zero exit code"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rejected"), "stdout was: {}", stdout);
}

#[test]
fn test_e2e_hooks_installation_and_lifecycle() {
    let repo_dir = setup_temp_git_repo();
    let hooks_dir = repo_dir.path().join(".git").join("hooks");

    // 1. Install hooks
    let output = cccp_bin()
        .args(["hook", "--install", "--force"])
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run cccp hook --install");
    assert!(output.status.success(), "hook install must succeed");

    let commit_msg_hook = hooks_dir.join("commit-msg");
    assert!(
        commit_msg_hook.is_file(),
        "commit-msg hook must be installed"
    );
    let pre_push_hook = hooks_dir.join("pre-push");
    assert!(pre_push_hook.is_file(), "pre-push hook must be installed");

    // Verify hook content calls cccp
    let hook_content = fs::read_to_string(&commit_msg_hook).expect("read hook");
    assert!(
        hook_content.contains("cccp hook commit-msg"),
        "hook content must dispatch to cccp"
    );

    // 2. Uninstall hooks
    let output = cccp_bin()
        .args(["hook", "--uninstall"])
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to run cccp hook --uninstall");
    assert!(output.status.success(), "hook uninstall must succeed");

    assert!(
        !commit_msg_hook.is_file(),
        "commit-msg hook must be removed"
    );
    assert!(!pre_push_hook.is_file(), "pre-push hook must be removed");
}

#[test]
fn test_e2e_git_status_and_directory_override() {
    let repo_dir = setup_temp_git_repo();

    // Create a first commit
    let readme = repo_dir.path().join("README.md");
    fs::write(&readme, "# Test Repository\n").expect("write readme");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git commit");

    // Execute status with -C from an outside directory
    let outside_dir = tempdir().expect("outside temp dir");
    let output = cccp_bin()
        .args(["-C", repo_dir.path().to_str().unwrap(), "status"])
        .current_dir(outside_dir.path())
        .output()
        .expect("failed to run cccp -C <dir> status");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CCCP Repository Status"),
        "stdout was: {}",
        stdout
    );
    assert!(stdout.contains("clean"), "working tree must be clean");
}

#[test]
fn test_e2e_version_calculation() {
    let repo_dir = setup_temp_git_repo();

    // Create initial commit
    let readme = repo_dir.path().join("README.md");
    fs::write(&readme, "# Service\n").expect("write readme");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git commit");

    // Create a feature commit
    fs::write(&readme, "# Service\nFeature added.\n").expect("update readme");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "feat(api): add first endpoint"])
        .current_dir(repo_dir.path())
        .output()
        .expect("git commit");

    // Test version clean
    let output = cccp_bin()
        .args(["version", "-f", "clean"])
        .current_dir(repo_dir.path())
        .output()
        .expect("run cccp version -f clean");
    assert!(output.status.success());
    let clean_ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!clean_ver.is_empty(), "version must not be empty");

    // Test version json
    let output = cccp_bin()
        .args(["version", "-f", "json"])
        .current_dir(repo_dir.path())
        .output()
        .expect("run cccp version -f json");
    assert!(output.status.success());
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("output must be valid JSON");
    assert!(parsed.get("target_version").is_some());
}

#[test]
fn test_e2e_no_color_strips_ansi() {
    let output = cccp_bin()
        .args(["--no-color", "soviet", "--plan"])
        .output()
        .expect("run cccp --no-color soviet --plan");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "stdout must not contain ANSI escape sequences: {}",
        stdout
    );
}

#[test]
fn test_e2e_config_hierarchical_get_set() {
    let repo_dir = setup_temp_git_repo();

    // Set a config value in the repository
    let output = cccp_bin()
        .args(["config", "custom_test_key", "custom_test_val"])
        .current_dir(repo_dir.path())
        .output()
        .expect("run cccp config set");
    assert!(output.status.success());

    // Read back the config value
    let output = cccp_bin()
        .args(["config", "custom_test_key"])
        .current_dir(repo_dir.path())
        .output()
        .expect("run cccp config get");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("custom_test_val"), "stdout was: {}", stdout);

    // List config
    let output = cccp_bin()
        .args(["config", "-l"])
        .current_dir(repo_dir.path())
        .output()
        .expect("run cccp config list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("custom_test_key"), "stdout was: {}", stdout);
}
