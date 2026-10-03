use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_install_script_help() {
    let output = Command::new("./install.sh")
        .arg("--help")
        .output()
        .expect("failed to execute ./install.sh --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--prefix"));
    assert!(stdout.contains("--bin-dir"));
    assert!(stdout.contains("--dest-dir"));
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn test_install_script_dry_run() {
    let output = Command::new("./install.sh")
        .arg("--dry-run")
        .output()
        .expect("failed to execute ./install.sh --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[DRY-RUN]"));
    assert!(stdout.contains("Dry-run completed successfully"));
}

#[test]
fn test_install_script_invalid_arg() {
    let output = Command::new("./install.sh")
        .arg("--unrecognized-option")
        .output()
        .expect("failed to execute ./install.sh with invalid arg");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown argument"));
}

#[test]
fn test_install_script_custom_bin_dir() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let target_bin_dir = temp_dir.path().join("bin");

    let output = Command::new("./install.sh")
        .arg("--bin-dir")
        .arg(&target_bin_dir)
        .output()
        .expect("failed to execute ./install.sh");

    assert!(output.status.success());
    let installed_binary = target_bin_dir.join("cccp");
    assert!(installed_binary.exists(), "Installed binary must exist");

    let verify_run = Command::new(&installed_binary)
        .arg("--version")
        .output()
        .expect("failed to run installed binary");

    assert!(verify_run.status.success());
    let version_out = String::from_utf8_lossy(&verify_run.stdout);
    assert!(version_out.contains("cccp"));
}
