//! Unit and integration tests for CCCP GOST Standards Compliance & Quality Audit Engine.

use cccp::audit::{read_gostver, run_gost_audit, CheckStatus};
use cccp::cultural::Theme;
use std::fs;
use std::process::Command;
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

    let site_dir = temp.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    fs::write(site_dir.join("index.html"), "<html></html>\n").unwrap();

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

    let site_dir = temp.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    fs::write(site_dir.join("index.html"), "<html></html>\n").unwrap();

    // In non-strict mode, warnings are allowed
    let report_non_strict = run_gost_audit(temp.path(), false, &theme);
    assert!(report_non_strict.is_success);
    assert!(report_non_strict.warnings > 0);

    // In strict mode, warnings cause failure
    let report_strict = run_gost_audit(temp.path(), true, &theme);
    assert!(!report_strict.is_success);
}

#[test]
fn test_gost_audit_chores_exempt_from_ticket_requirement() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .status()
        .expect("git init failed");
    assert!(status.success());

    Command::new("git")
        .args(["config", "user.name", "Auditor"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "auditor@gosplan.su"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    fs::write(temp.path().join("GOSTVER"), "GOST_VERSION=\"2.0.0\"\n").unwrap();
    fs::write(temp.path().join("LICENSE"), "GPL-3.0\n").unwrap();
    fs::write(temp.path().join("README.md"), "# Test Repo\n").unwrap();
    fs::write(temp.path().join("VERSION"), "0.1.0\n").unwrap();
    fs::write(temp.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(temp.path().join(".editorconfig"), "root = true\n").unwrap();

    let workflows_dir = temp.path().join(".forgejo/workflows");
    fs::create_dir_all(&workflows_dir).unwrap();
    fs::write(workflows_dir.join("ci.yml"), "name: CI\n").unwrap();

    let site_dir = temp.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    fs::write(site_dir.join("index.html"), "<html></html>\n").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial repository structure"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    fs::write(temp.path().join("NOTES.md"), "just notes\n").unwrap();
    Command::new("git")
        .args(["add", "NOTES.md"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore(docs): update notes"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    let report_strict = run_gost_audit(temp.path(), true, &theme);
    let check_04 = report_strict
        .checks
        .iter()
        .find(|c| c.code == "GOST-04-TICKETS-COMMITS")
        .unwrap();
    assert_eq!(check_04.status, cccp::audit::CheckStatus::Pass);
    assert!(check_04.detail.contains("chores isentos"));

    // Mutation: adding an invalid non-chore commit without ticket reference must FAIL
    fs::write(temp.path().join("feat.txt"), "some feature\n").unwrap();
    Command::new("git")
        .args(["add", "feat.txt"])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feat: add feature without ticket"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    let report_mutant = run_gost_audit(temp.path(), true, &theme);
    let check_04_mutant = report_mutant
        .checks
        .iter()
        .find(|c| c.code == "GOST-04-TICKETS-COMMITS")
        .unwrap();
    assert_eq!(
        check_04_mutant.status,
        cccp::audit::CheckStatus::Fail,
        "Non-chore commit without ticket must fail GOST-04"
    );
}

fn check_workflow_has_permissions(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "permissions:" || trimmed.starts_with("permissions:")
    })
}

#[test]
fn test_forgejo_workflows_no_permissions() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workflows_dir = manifest_dir.join(".forgejo/workflows");
    if !workflows_dir.exists() {
        return;
    }

    for entry in fs::read_dir(&workflows_dir).unwrap().flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "yml" || ext == "yaml" {
            let content = fs::read_to_string(&path).unwrap();
            let has_permissions = check_workflow_has_permissions(&content);
            assert!(
                !has_permissions,
                "Workflow {:?} contains unsupported 'permissions:' directive",
                path.file_name().unwrap()
            );
        }
    }
}

#[test]
fn test_forgejo_workflows_mutation_permissions_detected() {
    let mutant_top = r#"
name: CI
on: [push]
permissions:
  contents: write
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    assert!(
        check_workflow_has_permissions(mutant_top),
        "Mutant with top-level permissions must be detected"
    );

    let mutant_job = r#"
name: CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
"#;
    assert!(
        check_workflow_has_permissions(mutant_job),
        "Mutant with job-level permissions must be detected"
    );

    let clean = r#"
name: CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    assert!(
        !check_workflow_has_permissions(clean),
        "Clean workflow without permissions must pass"
    );
}
