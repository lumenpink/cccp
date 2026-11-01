use cccp::semver_calc::sync_project_manifests;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sync_package_json() {
    let dir = tempdir().unwrap();
    let pkg_path = dir.path().join("package.json");
    fs::write(
        &pkg_path,
        r#"{
  "name": "my-node-app",
  "version": "0.1.0-dev.1",
  "scripts": {
    "test": "node --test"
  }
}
"#,
    )
    .unwrap();

    let updated = sync_project_manifests(dir.path(), "0.1.0-dev.35+20261031.1234567").unwrap();
    assert!(updated.contains(&std::path::PathBuf::from("package.json")));

    let content = fs::read_to_string(&pkg_path).unwrap();
    assert!(content.contains("\"version\": \"0.1.0-dev.35+20261031.1234567\""));
    assert!(content.contains("\"name\": \"my-node-app\""));
}

#[test]
fn test_sync_composer_json_updates_existing_version() {
    let dir = tempdir().unwrap();
    let comp_path = dir.path().join("composer.json");
    fs::write(
        &comp_path,
        r#"{
    "name": "vendor/package",
    "version": "1.0.0",
    "require": {}
}
"#,
    )
    .unwrap();

    let updated = sync_project_manifests(dir.path(), "1.1.0").unwrap();
    assert!(updated.contains(&std::path::PathBuf::from("composer.json")));

    let content = fs::read_to_string(&comp_path).unwrap();
    assert!(content.contains("\"version\": \"1.1.0\""));
}

#[test]
fn test_sync_composer_json_injects_missing_version() {
    let dir = tempdir().unwrap();
    let comp_path = dir.path().join("composer.json");
    fs::write(
        &comp_path,
        r#"{
    "name": "lumenpink/fuxico",
    "description": "Fuxico service",
    "type": "project"
}
"#,
    )
    .unwrap();

    let updated = sync_project_manifests(dir.path(), "0.1.0-dev.10").unwrap();
    assert!(updated.contains(&std::path::PathBuf::from("composer.json")));

    let content = fs::read_to_string(&comp_path).unwrap();
    assert!(content.contains("\"version\": \"0.1.0-dev.10\""));
    assert!(content.contains("\"name\": \"lumenpink/fuxico\""));
}

#[test]
fn test_sync_nested_cargo_in_libs_directory() {
    let dir = tempdir().unwrap();
    let lib_dir = dir.path().join("libs").join("helper");
    fs::create_dir_all(&lib_dir).unwrap();
    let lib_cargo = lib_dir.join("Cargo.toml");
    fs::write(
        &lib_cargo,
        r#"[package]
name = "helper-lib"
version = "0.1.0-dev.1"
edition = "2021"
"#,
    )
    .unwrap();

    let updated = sync_project_manifests(dir.path(), "0.1.0-dev.240").unwrap();
    assert!(updated.contains(&std::path::PathBuf::from("libs/helper/Cargo.toml")));

    let content = fs::read_to_string(&lib_cargo).unwrap();
    assert!(content.contains("version = \"0.1.0-dev.240\""));
}

#[test]
fn test_sync_ignores_blacklisted_directories() {
    let dir = tempdir().unwrap();

    // Cria arquivos em pastas ignoradas
    let node_modules_pkg = dir
        .path()
        .join("node_modules")
        .join("dep")
        .join("package.json");
    let vendor_comp = dir.path().join("vendor").join("lib").join("composer.json");
    let target_cargo = dir.path().join("target").join("test").join("Cargo.toml");

    fs::create_dir_all(node_modules_pkg.parent().unwrap()).unwrap();
    fs::create_dir_all(vendor_comp.parent().unwrap()).unwrap();
    fs::create_dir_all(target_cargo.parent().unwrap()).unwrap();

    fs::write(&node_modules_pkg, r#"{"name": "dep", "version": "1.0.0"}"#).unwrap();
    fs::write(
        &vendor_comp,
        r#"{"name": "vendor/lib", "version": "1.0.0"}"#,
    )
    .unwrap();
    fs::write(
        &target_cargo,
        r#"[package]
name = "test"
version = "1.0.0"
"#,
    )
    .unwrap();

    let updated = sync_project_manifests(dir.path(), "2.0.0").unwrap();
    assert!(updated.is_empty());
}
