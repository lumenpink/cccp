use cccp::cultural::Theme;
use cccp::newrepo::{cmd_newrepo, generate_license_text, NewrepoOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_newrepo_scaffolding_defaults() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("my-test-proj");
    let theme = Theme::init(true);

    let opts = NewrepoOptions {
        name: Some("my-test-proj".to_string()),
        root: Some(root.clone()),
        sync: false, // test offline/hermetic
        quiet: true,
        ..NewrepoOptions::default()
    };

    let res = cmd_newrepo(opts, &theme);
    assert!(res.is_ok());

    // Check all files
    assert!(root.join("GOSTVER").is_file());
    assert!(root.join(".editorconfig").is_file());
    assert!(root.join(".gitignore").is_file());
    assert!(root.join("VERSION").is_file());
    assert!(root.join("CHANGELOG.md").is_file());
    assert!(root.join("LICENSE").is_file());
    assert!(root.join("README.md").is_file());
    assert!(root.join("site/_headers").is_file());
    assert!(root.join("site/index.html").is_file());
    assert!(root.join(".forgejo/workflows/ci.yml").is_file());
    assert!(root.join(".git").is_dir());

    // Content checks
    let gostver = fs::read_to_string(root.join("GOSTVER")).unwrap();
    assert!(gostver.contains("GOST_VERSION=\"2.0.0\""));
    assert!(gostver.contains("GOST_PROFILE_URL=\"profiles/lumen.md\""));
    assert!(gostver.contains("GOST_LANGUAGES=\"shell\""));

    let version = fs::read_to_string(root.join("VERSION")).unwrap();
    assert_eq!(version.trim(), "0.1.0-dev.1");

    let editorconfig = fs::read_to_string(root.join(".editorconfig")).unwrap();
    assert!(editorconfig.contains("indent_size = 2"));

    let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/target/"));
    assert!(gitignore.contains(".antigravity/"));

    let changelog = fs::read_to_string(root.join("CHANGELOG.md")).unwrap();
    assert!(changelog.contains("Keep a Changelog"));

    let license = fs::read_to_string(root.join("LICENSE")).unwrap();
    assert!(license.contains("GNU GENERAL PUBLIC LICENSE"));

    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    assert!(readme.contains("# my-test-proj"));
    assert!(readme.contains("**Versão GOST:** `2.0.0`"));
    assert!(readme.contains("shell"));

    let ci_content = fs::read_to_string(root.join(".forgejo/workflows/ci.yml")).unwrap();
    assert!(ci_content.contains("runs-on: ubuntu-latest"));
    assert!(!ci_content.contains("runs-on: native"));

    let headers = fs::read_to_string(root.join("site/_headers")).unwrap();
    assert!(headers.contains("X-Content-Type-Options: nosniff"));

    let site_index = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(site_index.contains("my-test-proj — Documentação"));
    assert!(site_index.contains("<span class=\"badge\">GOST v2.0.0</span>"));

    let ci = fs::read_to_string(root.join(".forgejo/workflows/ci.yml")).unwrap();
    assert!(ci.contains("cccp check --strict"));
}

#[test]
fn test_newrepo_custom_options() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("api-rust");
    let theme = Theme::init(true);

    let opts = NewrepoOptions {
        name: Some("api-rust".to_string()),
        root: Some(root.clone()),
        desc: Some("Central telemetry service".to_string()),
        lang: "rust,shell".to_string(),
        license: "MIT".to_string(),
        profile: "profiles/custom.md".to_string(),
        sync: false,
        quiet: true,
        ..NewrepoOptions::default()
    };

    let res = cmd_newrepo(opts, &theme);
    assert!(res.is_ok());

    let gostver = fs::read_to_string(root.join("GOSTVER")).unwrap();
    assert!(gostver.contains("GOST_LANGUAGES=\"rust,shell\""));
    assert!(gostver.contains("GOST_PROFILE_URL=\"profiles/custom.md\""));

    let license = fs::read_to_string(root.join("LICENSE")).unwrap();
    assert!(license.contains("MIT License"));
    assert!(license.contains("api-rust"));

    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    assert!(readme.contains("Central telemetry service"));
    assert!(readme.contains("rust,shell"));

    let site_index = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(site_index.contains("Central telemetry service"));
    assert!(site_index.contains("MIT"));
}

#[test]
fn test_newrepo_flags_no_git_and_no_ci() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("no-git-proj");
    let theme = Theme::init(true);

    let opts = NewrepoOptions {
        name: Some("no-git-proj".to_string()),
        root: Some(root.clone()),
        git: false,
        ci: false,
        sync: false,
        quiet: true,
        ..NewrepoOptions::default()
    };

    let res = cmd_newrepo(opts, &theme);
    assert!(res.is_ok());

    assert!(!root.join(".git").exists());
    assert!(!root.join(".forgejo/workflows/ci.yml").exists());
    assert!(!root.join(".forgejo/workflows/ci.yaml").exists());
    assert!(root.join("GOSTVER").is_file());
    assert!(root.join("README.md").is_file());
}

#[test]
fn test_newrepo_idempotency() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("existing-proj");
    fs::create_dir_all(&root).unwrap();

    let custom_readme = root.join("README.md");
    fs::write(&custom_readme, "# MY CUSTOM README PRESERVED\n").unwrap();

    let theme = Theme::init(true);
    let opts = NewrepoOptions {
        name: Some("existing-proj".to_string()),
        root: Some(root.clone()),
        sync: false,
        quiet: true,
        ..NewrepoOptions::default()
    };

    let res = cmd_newrepo(opts, &theme);
    assert!(res.is_ok());

    let content = fs::read_to_string(&custom_readme).unwrap();
    assert_eq!(content, "# MY CUSTOM README PRESERVED\n");

    // Other files created
    assert!(root.join("GOSTVER").is_file());
    assert!(root.join("VERSION").is_file());
}

#[test]
fn test_generate_license_text() {
    let mit = generate_license_text("MIT", "my-app");
    assert!(mit.contains("MIT License"));
    assert!(mit.contains("my-app"));

    let cc0 = generate_license_text("CC0-1.0", "my-app");
    assert!(cc0.contains("CC0 1.0 Universal"));

    let cc_by_sa = generate_license_text("CC-BY-SA-4.0", "my-app");
    assert!(cc_by_sa.contains("Creative Commons Attribution-ShareAlike 4.0"));

    let gpl = generate_license_text("GPL-3.0", "my-app");
    assert!(gpl.contains("GNU GENERAL PUBLIC LICENSE"));

    let apache = generate_license_text("Apache-2.0", "my-app");
    assert!(apache.contains("Apache License"));

    let custom = generate_license_text("BSL-1.0", "my-app");
    assert!(custom.contains("Software License: BSL-1.0"));
    assert!(custom.contains("my-app"));
}

#[test]
fn test_newrepo_validation_error_missing_name_and_root() {
    let theme = Theme::init(true);
    let opts = NewrepoOptions {
        name: None,
        root: None,
        ..NewrepoOptions::default()
    };
    let res = cmd_newrepo(opts, &theme);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Informe o nome do repositório"));
}
