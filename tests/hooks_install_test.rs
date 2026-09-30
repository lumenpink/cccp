use cccp::cultural::Theme;
use cccp::git::GitRepo;
use cccp::hooks_install::{
    generate_hook_stub, install_hooks, is_cccp_hook, resolve_hooks_dir, uninstall_hooks, HOOK_NAMES,
};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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

    dir
}

#[test]
fn test_generate_hook_stub_content() {
    let stub = generate_hook_stub(
        "commit-msg",
        std::path::Path::new("/home/comrade/src/repo"),
        "3.0.0",
    );
    assert!(stub.contains("# cccp-hook-version: 3.0.0"));
    assert!(stub.contains("command -v cccp"));
    assert!(stub.contains("$HOME/.local/bin/cccp"));
    assert!(stub.contains("$HOME/.cargo/bin/cccp"));
    assert!(stub.contains("/usr/local/bin/cccp"));
    assert!(stub.contains("/usr/bin/cccp"));
    assert!(stub.contains("/home/comrade/src/repo/target/release/cccp"));
    assert!(stub.contains("★ GOSPLAN ALERT: COMRADE! ★"));
    assert!(stub.contains("«НИ ШАГУ НАЗАД!» — Order No. 227"));
}

#[test]
fn test_install_fresh_repo_creates_all_hooks() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let hooks_dir = p.join(".git").join("hooks");
    let theme = Theme::init(true);

    install_hooks(&hooks_dir, p, false, &theme).expect("install_hooks failed");

    for name in &HOOK_NAMES {
        let hook_path = hooks_dir.join(name);
        assert!(hook_path.exists(), "Hook '{}' must exist", name);
        assert!(
            is_cccp_hook(&hook_path),
            "Hook '{}' must be detected as CCCP hook",
            name
        );

        #[cfg(unix)]
        {
            let metadata = fs::metadata(&hook_path).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(
                mode & 0o111,
                0o111,
                "Hook '{}' must have executable permissions",
                name
            );
        }
    }
}

#[test]
fn test_install_backs_up_foreign_hook_and_uninstall_restores_it() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let hooks_dir = p.join(".git").join("hooks");
    let theme = Theme::init(true);

    fs::create_dir_all(&hooks_dir).unwrap();

    // Create a pre-existing custom hook
    let custom_hook = hooks_dir.join("commit-msg");
    let custom_body = "#!/bin/sh\n# Custom husky script\necho 'running custom check'\nexit 0\n";
    fs::write(&custom_hook, custom_body).unwrap();

    // Install CCCP hooks
    install_hooks(&hooks_dir, p, false, &theme).expect("install_hooks failed");

    // commit-msg should now be a CCCP hook
    assert!(is_cccp_hook(&custom_hook));

    // commit-msg.bak should exist with original content
    let backup_hook = hooks_dir.join("commit-msg.bak");
    assert!(backup_hook.exists());
    let saved_backup = fs::read_to_string(&backup_hook).unwrap();
    assert_eq!(saved_backup, custom_body);

    // Uninstall CCCP hooks
    uninstall_hooks(&hooks_dir, &theme).expect("uninstall_hooks failed");

    // commit-msg must be restored to original custom script
    assert!(custom_hook.exists());
    assert!(!backup_hook.exists());
    let restored_body = fs::read_to_string(&custom_hook).unwrap();
    assert_eq!(restored_body, custom_body);
    assert!(!is_cccp_hook(&custom_hook));

    // Other hooks (pre-push, etc.) that had no backups should be removed
    assert!(!hooks_dir.join("pre-push").exists());
    assert!(!hooks_dir.join("post-commit").exists());
}

#[test]
fn test_install_idempotency_upgrade() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();
    let hooks_dir = p.join(".git").join("hooks");
    let theme = Theme::init(true);

    install_hooks(&hooks_dir, p, false, &theme).unwrap();
    // Run second time (upgrade)
    install_hooks(&hooks_dir, p, false, &theme).unwrap();

    // No .bak files should have been created since it was already CCCP
    for name in &HOOK_NAMES {
        assert!(!hooks_dir.join(format!("{}.bak", name)).exists());
    }
}

#[test]
fn test_resolve_hooks_dir_custom_core_hooks_path() {
    let repo_dir = setup_temp_git_repo();
    let p = repo_dir.path();

    // Configure core.hooksPath
    Command::new("git")
        .args(["config", "core.hooksPath", ".custom_hooks"])
        .current_dir(p)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(p)).unwrap();
    let resolved = resolve_hooks_dir(&repo, None);
    assert_eq!(resolved, p.join(".custom_hooks"));

    // CLI override takes precedence
    let cli_override = p.join("override_dir");
    let resolved_cli = resolve_hooks_dir(&repo, Some(&cli_override));
    assert_eq!(resolved_cli, cli_override);
}
