use cccp::semver_calc::sync_cargo_toml_versions;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sync_single_crate_cargo_toml() {
    let dir = tempdir().unwrap();
    let cargo_path = dir.path().join("Cargo.toml");
    fs::write(
        &cargo_path,
        r#"[package]
name = "single-app"
version = "0.1.0-dev.1"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    let updated = sync_cargo_toml_versions(dir.path(), "0.1.0-dev.239+20261029.54f8b6b").unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0], std::path::PathBuf::from("Cargo.toml"));

    let content = fs::read_to_string(&cargo_path).unwrap();
    assert!(content.contains("version = \"0.1.0-dev.239+20261029.54f8b6b\""));
    assert!(content.contains("name = \"single-app\""));
}

#[test]
fn test_sync_workspace_members_cargo_toml() {
    let dir = tempdir().unwrap();
    let root_cargo = dir.path().join("Cargo.toml");
    fs::write(
        &root_cargo,
        r#"[workspace]
members = [
    "crates/pult-core",
    "crates/pult-tui",
]
resolver = "2"
"#,
    )
    .unwrap();

    let core_dir = dir.path().join("crates").join("pult-core");
    let tui_dir = dir.path().join("crates").join("pult-tui");
    fs::create_dir_all(&core_dir).unwrap();
    fs::create_dir_all(&tui_dir).unwrap();

    fs::write(
        core_dir.join("Cargo.toml"),
        r#"[package]
name = "pult-core"
version = "0.1.0-dev.1"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        tui_dir.join("Cargo.toml"),
        r#"[package]
name = "pult"
version = "0.1.0-dev.1"
edition = "2021"
"#,
    )
    .unwrap();

    let updated = sync_cargo_toml_versions(dir.path(), "0.1.0-dev.239").unwrap();
    assert_eq!(updated.len(), 2);

    let core_content = fs::read_to_string(core_dir.join("Cargo.toml")).unwrap();
    assert!(core_content.contains("version = \"0.1.0-dev.239\""));

    let tui_content = fs::read_to_string(tui_dir.join("Cargo.toml")).unwrap();
    assert!(tui_content.contains("version = \"0.1.0-dev.239\""));
}

#[test]
fn test_sync_workspace_package_cargo_toml() {
    let dir = tempdir().unwrap();
    let root_cargo = dir.path().join("Cargo.toml");
    fs::write(
        &root_cargo,
        r#"[workspace.package]
version = "0.1.0-dev.1"
edition = "2021"

[workspace]
members = ["crates/*"]
"#,
    )
    .unwrap();

    let updated = sync_cargo_toml_versions(dir.path(), "0.2.0").unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0], std::path::PathBuf::from("Cargo.toml"));

    let content = fs::read_to_string(&root_cargo).unwrap();
    assert!(content.contains("version = \"0.2.0\""));
}
