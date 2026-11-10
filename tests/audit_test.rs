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
fn test_agnostic_audit_evaluates_only_declarative_rules() {
    let temp = tempdir().expect("tempdir");
    let p = temp.path();
    let theme = Theme::with_color(false);

    let custom_rules = r#"
[rules.CUSTOM-001]
name = "PURE-CUSTOM-RULE"
severity = "error"
scope = "project"
description = "Verifies custom rule evaluation"
script = "return true"
"#;
    fs::write(p.join("rules.toml"), custom_rules).unwrap();

    let report = run_gost_audit(p, false, &theme);
    assert!(report.is_success, "Agnostic audit must pass with 1 valid rule");
    assert_eq!(report.checks.len(), 1, "Report must contain exactly 1 check from rules.toml");
    assert_eq!(report.checks[0].code, "CUSTOM-001");
    assert_eq!(report.checks[0].status, CheckStatus::Pass);

    let legacy_codes = [
        "GOST-16-GOSTVER",
        "GOST-01-LICENSE",
        "GOST-02-ANATOMY-ROOT",
        "GOST-20-SITE-TYPE-AND-SSOT-DOCS",
        "GOST-24-ZERO-BINARIES-IN-GIT",
        "GOST-26-METADATA-MANIFEST-SSOT",
        "GOST-27-DDD-HEXAGONAL-PURITY",
    ];
    for legacy in &legacy_codes {
        assert!(
            report.checks.iter().all(|c| c.code != *legacy),
            "Legacy hardcoded check '{}' must NOT be evaluated by agnostic engine",
            legacy
        );
    }
}

#[test]
fn test_agnostic_audit_without_rules_reports_no_rules_found() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::with_color(false);

    let report = run_gost_audit(temp.path(), false, &theme);
    assert!(!report.is_success, "Empty repo without rules must fail");
    assert!(
        report.checks.iter().any(|c| c.code == "NO-RULES-FOUND"),
        "Agnostic audit without rules must emit NO-RULES-FOUND"
    );
}

#[test]
fn test_agnostic_audit_mutation_lua_runtime_error_does_not_panic() {
    let temp = tempdir().expect("tempdir");
    let p = temp.path();
    let theme = Theme::with_color(false);

    let custom_rules = r#"
[rules.MUTANT-ERR]
name = "RUNTIME-ERROR-RULE"
severity = "error"
scope = "project"
description = "Verifica tratamento de erro em Lua"
script = "error('Falha intencional de runtime Lua')"
"#;
    fs::write(p.join("rules.toml"), custom_rules).unwrap();

    let report = run_gost_audit(p, false, &theme);
    assert!(!report.is_success, "Rule with runtime error must fail audit");
    let check = report
        .checks
        .iter()
        .find(|c| c.code == "MUTANT-ERR")
        .expect("MUTANT-ERR check must exist");
    assert_eq!(check.status, CheckStatus::Fail);
    assert!(check.detail.contains("Erro de execução no script Lua"));
}

#[test]
fn test_agnostic_audit_mutation_scope_isolation() {
    let temp = tempdir().expect("tempdir");
    let p = temp.path();
    let theme = Theme::with_color(false);

    let custom_rules = r#"
[rules.PROJECT-001]
name = "PROJECT-SCOPE-RULE"
severity = "error"
scope = "project"
description = "Regra de projeto"
script = "return true"

[rules.COMMIT-001]
name = "COMMIT-SCOPE-RULE"
severity = "error"
scope = "commit-msg"
description = "Regra de commit"
script = "return false"
"#;
    fs::write(p.join("rules.toml"), custom_rules).unwrap();

    // Auditoria padrão (escopo "project") não deve avaliar regras com scope="commit"
    let report = run_gost_audit(p, false, &theme);
    assert!(report.is_success);
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].code, "PROJECT-001");
    assert!(report.checks.iter().all(|c| c.code != "COMMIT-001"));
}

#[test]
fn test_gost_audit_empty_directory_fails() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    let report = run_gost_audit(temp.path(), false, &theme);
    assert!(!report.is_success, "Empty directory must fail GOST audit");
    assert_eq!(report.failures, 1);
    assert_eq!(report.percentage, 0);

    let no_rules = report
        .checks
        .iter()
        .find(|c| c.code == "NO-RULES-FOUND")
        .expect("NO-RULES-FOUND check must exist");
    assert_eq!(no_rules.status, CheckStatus::Fail);
}

#[test]
fn test_gost_audit_compliant_repo() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    let rules = r#"
[rules.RULE-001]
name = "CHECK-README"
severity = "error"
scope = "project"
description = "Verifica README"
script = "return gost.file_exists('README.md')"

[rules.RULE-002]
name = "CHECK-LICENSE"
severity = "error"
scope = "project"
description = "Verifica LICENSE"
script = "return gost.file_exists('LICENSE')"
"#;
    fs::write(temp.path().join("rules.toml"), rules).unwrap();
    fs::write(temp.path().join("README.md"), "# Compliant Repo\n").unwrap();
    fs::write(temp.path().join("LICENSE"), "GPL-3.0\n").unwrap();

    let report = run_gost_audit(temp.path(), false, &theme);
    assert_eq!(report.failures, 0, "No failures expected in compliant repo");
    assert!(report.is_success);
    assert_eq!(report.percentage, 100);
}

#[test]
fn test_gost_audit_strict_mode_rejects_warnings() {
    let temp = tempdir().expect("tempdir");
    let theme = Theme::init(true);

    let rules = r#"
[rules.RULE-WARN]
name = "OPTIONAL-DOC"
severity = "warning"
scope = "project"
description = "Aviso se documento opcional faltar"
script = "return false, 'Doc opcional ausente'"
"#;
    fs::write(temp.path().join("rules.toml"), rules).unwrap();

    // In non-strict mode, warnings are allowed
    let report_non_strict = run_gost_audit(temp.path(), false, &theme);
    assert!(report_non_strict.is_success);
    assert_eq!(report_non_strict.warnings, 1);

    // In strict mode, warnings cause failure
    let report_strict = run_gost_audit(temp.path(), true, &theme);
    assert!(!report_strict.is_success);
    assert_eq!(report_strict.warnings, 1);
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

    let rules = r#"
[rules.GIT-001]
name = "CHECK-BRANCH"
severity = "error"
scope = "project"
description = "Verifica se branch ativa e main"
script = "local b = gost.git_branch(); return b == 'main', 'Branch ativa: ' .. tostring(b)"
"#;
    fs::write(temp.path().join("rules.toml"), rules).unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(temp.path())
        .status()
        .unwrap();

    let report_strict = run_gost_audit(temp.path(), true, &theme);
    let check = report_strict
        .checks
        .iter()
        .find(|c| c.code == "GIT-001")
        .expect("GIT-001 check must exist");
    assert_eq!(check.status, CheckStatus::Pass);
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
