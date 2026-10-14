use cccp::act::{
    act_ci_gate, check_prerequisites, find_workflow, format_duration, run_act_log, run_act_status,
    sanitize_workflow_if_needed, RunnerPauseGuard,
};
use cccp::cultural::Theme;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0), "0m 00s");
    assert_eq!(format_duration(47), "0m 47s");
    assert_eq!(format_duration(94), "1m 34s");
    assert_eq!(format_duration(3605), "60m 05s");
}

#[test]
fn test_find_workflow_locations() {
    let temp = tempdir().expect("Failed to create tempdir");
    let root = temp.path();

    // 1. None when empty
    assert!(find_workflow(root).is_none());

    // 2. Finds Forgejo workflow
    let forgejo_dir = root.join(".forgejo/workflows");
    fs::create_dir_all(&forgejo_dir).unwrap();
    let forgejo_wf = forgejo_dir.join("ci.yml");
    fs::write(&forgejo_wf, "name: CI\n").unwrap();

    let found = find_workflow(root);
    assert_eq!(found, Some(forgejo_wf.clone()));

    // 3. Priority of forgejo over github
    let github_dir = root.join(".github/workflows");
    fs::create_dir_all(&github_dir).unwrap();
    let github_wf = github_dir.join("ci.yml");
    fs::write(&github_wf, "name: GitHub CI\n").unwrap();

    let found_priority = find_workflow(root);
    assert_eq!(found_priority, Some(forgejo_wf));
}

#[test]
fn test_sanitize_workflow_if_needed() {
    let temp = tempdir().expect("Failed to create tempdir");
    let wf_path = temp.path().join("ci.yml");

    fs::write(
        &wf_path,
        "jobs:\n  test:\n    steps:\n      - uses: https://github.com/actions/checkout@v4\n",
    )
    .unwrap();

    let (clean_path, tmp) = sanitize_workflow_if_needed(&wf_path);
    assert!(tmp.is_some());
    let content = fs::read_to_string(&clean_path).unwrap();
    assert!(!content.contains("https://github.com/"));
    assert!(content.contains("uses: actions/checkout@v4"));

    // Workflow without https URL should not be modified
    let clean_wf_path = temp.path().join("clean.yml");
    fs::write(&clean_wf_path, "uses: actions/checkout@v4\n").unwrap();
    let (same_path, no_tmp) = sanitize_workflow_if_needed(&clean_wf_path);
    assert!(no_tmp.is_none());
    assert_eq!(same_path, clean_wf_path);
}

#[test]
fn test_check_prerequisites_missing_binary() {
    let (act_ok, _) = check_prerequisites(Some("/nonexistent/path/to/act_xyz"));
    assert!(!act_ok);
}

#[test]
fn test_act_ci_gate_bypass_directive() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let theme = Theme::with_color(false);

    std::env::set_var("CCCP_ACT_SKIP", "1");
    let res = act_ci_gate(temp.path(), "origin", "main", &theme, 0);
    std::env::remove_var("CCCP_ACT_SKIP");

    assert_eq!(res, Ok(true));
}

#[test]
fn test_act_ci_gate_no_workflow_skips() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let theme = Theme::with_color(false);

    std::env::remove_var("CCCP_ACT_SKIP");
    std::env::remove_var("ACT_SKIP");

    let res = act_ci_gate(temp.path(), "origin", "main", &theme, 0);
    assert_eq!(res, Ok(true));
}

#[test]
fn test_act_ci_gate_graceful_degradation_on_missing_act() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let theme = Theme::with_color(false);

    let wf_dir = temp.path().join(".forgejo/workflows");
    fs::create_dir_all(&wf_dir).unwrap();
    fs::write(wf_dir.join("ci.yml"), "name: CI\n").unwrap();

    std::env::set_var("CCCP_ACT_BIN", "/nonexistent/act_binary_runner");
    let res = act_ci_gate(temp.path(), "origin", "main", &theme, 0);
    std::env::remove_var("CCCP_ACT_BIN");

    // Must degrade gracefully and allow push without error
    assert_eq!(res, Ok(true));
}

#[test]
fn test_act_ci_gate_success_execution_and_logging() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let root = temp.path();
    let theme = Theme::with_color(false);

    // Initialize git repository in temp dir
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@gosplan.su"])
        .current_dir(root)
        .status()
        .unwrap();

    fs::write(root.join("test.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-q", "-m", "chore: init"])
        .current_dir(root)
        .status()
        .unwrap();

    let wf_dir = root.join(".forgejo/workflows");
    fs::create_dir_all(&wf_dir).unwrap();
    fs::write(wf_dir.join("ci.yml"), "name: CI\n").unwrap();

    // Create mock act runner that succeeds
    let mock_act = root.join("mock_act.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(
            &mock_act,
            "#!/bin/sh\necho '[test/Run tests] ⭐ Run cargo test'\necho 'Success!'\nexit 0\n",
        )
        .unwrap();
        let mut perms = fs::metadata(&mock_act).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_act, perms).unwrap();
    }

    std::env::set_var("CCCP_ACT_BIN", mock_act.to_str().unwrap());
    std::env::set_var("SHELLSPEC", "1"); // non-interactive

    let res = act_ci_gate(root, "origin", "main", &theme, 0);

    std::env::remove_var("CCCP_ACT_BIN");
    std::env::remove_var("SHELLSPEC");

    assert_eq!(res, Ok(true));

    // Verify log file was recorded in .git/act-logs
    let log_dir = root.join(".git/act-logs");
    assert!(log_dir.is_dir());
    let log_count = fs::read_dir(&log_dir).unwrap().count();
    assert!(log_count >= 1);

    // Verify run_act_log and run_act_status works
    assert!(run_act_log(root, false, &theme).is_ok());
    run_act_status(root);
}

#[test]
fn test_act_ci_gate_failure_rejection() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let root = temp.path();
    let theme = Theme::with_color(false);

    Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@gosplan.su"])
        .current_dir(root)
        .status()
        .unwrap();

    fs::write(root.join("test.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(root)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-q", "-m", "chore: init"])
        .current_dir(root)
        .status()
        .unwrap();

    let wf_dir = root.join(".forgejo/workflows");
    fs::create_dir_all(&wf_dir).unwrap();
    fs::write(wf_dir.join("ci.yml"), "name: CI\n").unwrap();

    // Create mock act runner that fails
    let mock_act = root.join("mock_act_fail.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(
            &mock_act,
            "#!/bin/sh\necho '##[error] cargo test failed: 2 assertion errors'\nexit 1\n",
        )
        .unwrap();
        let mut perms = fs::metadata(&mock_act).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_act, perms).unwrap();
    }

    std::env::set_var("CCCP_ACT_BIN", mock_act.to_str().unwrap());
    std::env::set_var("SHELLSPEC", "1"); // non-interactive

    let res = act_ci_gate(root, "origin", "main", &theme, 0);

    std::env::remove_var("CCCP_ACT_BIN");
    std::env::remove_var("SHELLSPEC");

    // Gate should fail, rejecting push
    assert_eq!(res, Ok(false));
}

#[test]
fn test_runner_pause_guard_bypass_env() {
    let _guard = ENV_LOCK.lock().unwrap();
    let theme = Theme::with_color(false);

    std::env::set_var("CCCP_NO_PAUSE_RUNNER", "1");
    let guard = RunnerPauseGuard::new(&theme);
    std::env::remove_var("CCCP_NO_PAUSE_RUNNER");

    assert!(guard.paused_containers().is_empty());
}

#[test]
fn test_runner_pause_guard_missing_docker() {
    let _guard = ENV_LOCK.lock().unwrap();
    let theme = Theme::with_color(false);

    let guard = RunnerPauseGuard::with_docker_bin(&theme, "/nonexistent/docker/bin");
    assert!(guard.paused_containers().is_empty());
}

#[test]
fn test_runner_pause_guard_lifecycle_mock_docker() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let theme = Theme::with_color(false);

    let log_file = temp.path().join("docker_calls.log");
    let mock_docker = temp.path().join("mock_docker.sh");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = format!(
            r#"#!/bin/sh
cmd="$1"
sub="$2"
echo "$@" >> "{}"
if [ "$cmd" = "ps" ]; then
    echo "forgejo-runner"
fi
exit 0
"#,
            log_file.display()
        );
        fs::write(&mock_docker, script).unwrap();
        let mut perms = fs::metadata(&mock_docker).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_docker, perms).unwrap();
    }

    {
        let guard = RunnerPauseGuard::with_docker_bin(&theme, mock_docker.to_str().unwrap());
        assert_eq!(guard.paused_containers(), &["forgejo-runner"]);

        let log = fs::read_to_string(&log_file).unwrap_or_default();
        assert!(log.contains("pause forgejo-runner"));
    }

    // After guard is dropped, unpause must have been invoked
    let log_after = fs::read_to_string(&log_file).unwrap_or_default();
    assert!(log_after.contains("unpause forgejo-runner"));
}

#[test]
fn test_runner_pause_guard_docker_failure_graceful() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().expect("Failed to create tempdir");
    let theme = Theme::with_color(false);

    let mock_docker = temp.path().join("mock_docker_fail.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(&mock_docker, "#!/bin/sh\nexit 1\n").unwrap();
        let mut perms = fs::metadata(&mock_docker).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_docker, perms).unwrap();
    }

    let guard = RunnerPauseGuard::with_docker_bin(&theme, mock_docker.to_str().unwrap());
    assert!(guard.paused_containers().is_empty());
}
