//! Unit and integration tests for CCCP GOST Standards Compliance & Quality Audit Engine.

use cccp::audit::{read_gostver, run_gost_audit, CheckStatus};
use cccp::cultural::Theme;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_gostver_parsing() {
    let temp = tempdir().expect("tempdir");
    let content = r#"
# GOSTVER
GOST_VERSION="1.1.0"
GOST_PROFILE_URL="profiles/lumen.md"
GOST_LANGUAGES="rust,shell"
"#;
    fs::write(temp.path().join("GOSTVER"), content).unwrap();

    let (ver, prof, langs) = read_gostver(temp.path());
    assert_eq!(ver, "1.1.0");
    assert_eq!(prof, "profiles/lumen.md");
    assert_eq!(langs, "rust,shell");
}

#[test]
fn test_read_gostver_missing_file() {
    let temp = tempdir().expect("tempdir");
    let (ver, prof, langs) = read_gostver(temp.path());
    assert_eq!(ver, "Não declarado");
    assert_eq!(prof, "Nenhum");
    assert_eq!(langs, "Nenhuma");
}

#[test]
fn test_gost_audit_empty_directory_fails() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    let report = run_gost_audit(temp.path(), false, &theme);
    assert!(!report.is_success, "Empty directory must fail GOST audit");
    assert!(report.failures > 0);
    assert!(report.percentage < 50);

    // Verify individual checks
    let gostver_check = report
        .checks
        .iter()
        .find(|c| c.code == "GOST-16-GOSTVER")
        .unwrap();
    assert_eq!(gostver_check.status, CheckStatus::Fail);

    let license_check = report
        .checks
        .iter()
        .find(|c| c.code == "GOST-01-LICENSE")
        .unwrap();
    assert_eq!(license_check.status, CheckStatus::Fail);
}

#[test]
fn test_gost_audit_compliant_repo() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    // Setup compliant files
    fs::write(temp.path().join("GOSTVER"), "GOST_VERSION=\"1.1.0\"\n").unwrap();
    fs::write(temp.path().join("LICENSE"), "CC0-1.0 Public Domain\n").unwrap();
    fs::write(temp.path().join("README.md"), "# Compliant Repo\n").unwrap();
    fs::write(temp.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(temp.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(temp.path().join(".editorconfig"), "root = true\n").unwrap();

    let workflows_dir = temp.path().join(".forgejo/workflows");
    fs::create_dir_all(&workflows_dir).unwrap();
    fs::write(workflows_dir.join("ci.yaml"), "name: CI\n").unwrap();

    let report = run_gost_audit(temp.path(), false, &theme);
    assert_eq!(report.failures, 0, "No failures expected in compliant repo");
    assert!(report.is_success);
    assert!(report.percentage >= 75);
}

#[test]
fn test_gost_audit_strict_mode_rejects_warnings() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    // Setup base files without CI workflows (which produces a warning)
    fs::write(temp.path().join("GOSTVER"), "GOST_VERSION=\"1.1.0\"\n").unwrap();
    fs::write(temp.path().join("LICENSE"), "MIT\n").unwrap();
    fs::write(temp.path().join("README.md"), "# Repo\n").unwrap();
    fs::write(temp.path().join("VERSION"), "0.1.0\n").unwrap();
    fs::write(temp.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(temp.path().join(".editorconfig"), "root = true\n").unwrap();

    // In non-strict mode, warnings are allowed
    let report_non_strict = run_gost_audit(temp.path(), false, &theme);
    assert!(report_non_strict.is_success);
    assert!(report_non_strict.warnings > 0);

    // In strict mode, warnings cause failure
    let report_strict = run_gost_audit(temp.path(), true, &theme);
    assert!(!report_strict.is_success);
}
