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
    assert_eq!(ver, "Undeclared");
    assert_eq!(prof, "None");
    assert_eq!(langs, "None");
}

#[test]
fn test_read_gostver_from_dot_rules_profile_toml() {
    let temp = tempdir().expect("tempdir");
    let rules_dir = temp.path().join(".rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let content = r#"
url = "https://lilya.2lp.in"
version = "2.1.0"
"#;
    fs::write(rules_dir.join("profile.toml"), content).unwrap();

    let (ver, prof, _langs) = read_gostver(temp.path());
    assert_eq!(ver, "2.1.0");
    assert_eq!(prof, "https://lilya.2lp.in");
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
    assert!(
        report.is_success,
        "Agnostic audit must pass with 1 valid rule"
    );
    assert_eq!(
        report.checks.len(),
        1,
        "Report must contain exactly 1 check from rules.toml"
    );
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
    assert!(
        !report.is_success,
        "Rule with runtime error must fail audit"
    );
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

#[test]
fn test_audit_evaluates_rules_from_dot_rules() {
    let temp = tempdir().expect("tempdir");
    let p = temp.path();
    let rules_dir = p.join(".rules");
    fs::create_dir_all(&rules_dir).unwrap();

    let custom_rules = r#"
[rules.LOCAL-001]
name = "Local Test Rule"
description = "Verifies custom rule in .rules/RULES.toml"
severity = "error"
scope = "project"
script = "return true"
"#;
    fs::write(rules_dir.join("RULES.toml"), custom_rules).unwrap();

    let theme = Theme::with_color(false);
    let report = run_gost_audit(p, false, &theme);
    assert_eq!(
        report.checks.len(),
        1,
        "Audit must evaluate exactly 1 rule from .rules/RULES.toml"
    );
    assert_eq!(report.checks[0].code, "LOCAL-001");
    assert_eq!(report.checks[0].status, CheckStatus::Pass);
}

#[test]
fn test_read_gostver_legacy_gostver_precedence() {
    let temp = tempdir().expect("tempdir");
    let p = temp.path();

    // Create both GOSTVER and .rules/profile.toml
    let gostver_content = r#"
GOST_VERSION="1.9.9"
GOST_PROFILE_URL="profiles/legacy.md"
"#;
    fs::write(p.join("GOSTVER"), gostver_content).unwrap();

    let rules_dir = p.join(".rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let profile_toml = r#"
url = "https://lilya.2lp.in"
version = "2.1.0"
"#;
    fs::write(rules_dir.join("profile.toml"), profile_toml).unwrap();

    let (ver, prof, _) = read_gostver(p);
    assert_eq!(
        ver, "1.9.9",
        "GOSTVER legacy must take precedence if explicitly present"
    );
    assert_eq!(prof, "profiles/legacy.md");
}

#[test]
fn test_render_audit_report_in_english() {
    let theme = Theme::with_color(false);
    let report = cccp::audit::AuditReport {
        gost_version: "2.1.0".to_string(),
        profile: "https://example.org/profile".to_string(),
        languages: "rust".to_string(),
        checks: vec![cccp::audit::AuditCheck {
            code: "TEST-001",
            title: "Test Check",
            detail: "Conforming with guideline".to_string(),
            status: CheckStatus::Pass,
        }],
        passed: 1,
        warnings: 0,
        failures: 0,
        percentage: 100,
        is_success: true,
    };

    let rendered = cccp::audit::render_audit_report(&report, false, &theme);
    assert!(
        rendered.contains("☭ [RULES] NORMATIVE CONFORMANCE AUDIT"),
        "Header must be in English and agnostic: {}",
        rendered
    );
    assert!(
        rendered.contains("Standard: 2.1.0 | Profile: https://example.org/profile"),
        "Metadata must use English labels: {}",
        rendered
    );
    assert!(
        rendered.contains("Total checks: 1 | Conforming: 1"),
        "Summary must use English labels: {}",
        rendered
    );
    assert!(
        rendered.contains("Compliance Index: 100%"),
        "Compliance index must use English labels: {}",
        rendered
    );
}

#[test]
fn test_build_canonical_rules_urls_deterministic() {
    let (md_url, toml_url) =
        cccp::profile::build_canonical_rules_urls("https://example.org/rules/", "1.2.0");
    assert_eq!(md_url, "https://example.org/rules/1.2.0/RULES.md");
    assert_eq!(toml_url, "https://example.org/rules/1.2.0/RULES.toml");
}

#[test]
fn test_mutation_build_canonical_rules_urls_rejects_trailing_slash_inconsistencies() {
    let (md1, toml1) =
        cccp::profile::build_canonical_rules_urls("https://custom.org/spec", "0.9.1");
    let (md2, toml2) =
        cccp::profile::build_canonical_rules_urls("https://custom.org/spec/", "0.9.1");
    assert_eq!(md1, md2);
    assert_eq!(toml1, toml2);
    assert_eq!(md1, "https://custom.org/spec/0.9.1/RULES.md");
    assert_eq!(toml1, "https://custom.org/spec/0.9.1/RULES.toml");
}

#[test]
fn test_mutation_get_rules_candidate_urls_arbitrary_profile() {
    let candidates =
        cccp::profile::get_rules_candidate_urls("https://mycompany.com/standards/v2/RULES.md");
    assert_eq!(
        candidates,
        vec!["https://mycompany.com/standards/v2/RULES.toml".to_string()]
    );

    let candidates_dir =
        cccp::profile::get_rules_candidate_urls("https://mycompany.com/standards/v2/");
    assert_eq!(
        candidates_dir,
        vec!["https://mycompany.com/standards/v2/RULES.toml".to_string()]
    );
}

#[test]
fn test_render_audit_report_failures_only_filters_out_passes() {
    let theme = Theme::with_color(false);
    let report = cccp::audit::AuditReport {
        gost_version: "2.1.0".to_string(),
        profile: "https://example.org/profile".to_string(),
        languages: "rust".to_string(),
        checks: vec![
            cccp::audit::AuditCheck {
                code: "PASS-001",
                title: "Passing Check",
                detail: "Conforming with guideline".to_string(),
                status: CheckStatus::Pass,
            },
            cccp::audit::AuditCheck {
                code: "FAIL-001",
                title: "Failing Check",
                detail: "Violation detected".to_string(),
                status: CheckStatus::Fail,
            },
        ],
        passed: 1,
        warnings: 0,
        failures: 1,
        percentage: 50,
        is_success: false,
    };

    let rendered = cccp::audit::render_audit_report_with_options(&report, true, true, &theme);
    assert!(
        rendered.contains("FAIL-001"),
        "Failures-only mode must include failed checks: {}",
        rendered
    );
    assert!(
        !rendered.contains("PASS-001"),
        "Failures-only mode must NOT include passed checks: {}",
        rendered
    );
    assert!(
        !rendered.contains("[PASS]"),
        "Failures-only mode must NOT render [PASS] badges: {}",
        rendered
    );
}

#[test]
fn test_mutation_render_audit_report_failures_only_all_passing() {
    let theme = Theme::with_color(false);
    let report = cccp::audit::AuditReport {
        gost_version: "2.1.0".to_string(),
        profile: "https://example.org/profile".to_string(),
        languages: "rust".to_string(),
        checks: vec![
            cccp::audit::AuditCheck {
                code: "PASS-001",
                title: "Passing 1",
                detail: "OK".to_string(),
                status: CheckStatus::Pass,
            },
            cccp::audit::AuditCheck {
                code: "PASS-002",
                title: "Passing 2",
                detail: "OK".to_string(),
                status: CheckStatus::Pass,
            },
        ],
        passed: 2,
        warnings: 0,
        failures: 0,
        percentage: 100,
        is_success: true,
    };

    let rendered = cccp::audit::render_audit_report_with_options(&report, true, true, &theme);
    assert!(
        rendered.contains("All 2 checks passed (100% compliance)"),
        "100% passing audit in failures-only mode must produce clean single line success: {}",
        rendered
    );
    assert!(
        !rendered.contains("NORMATIVE CONFORMANCE AUDIT"),
        "Passing audit should not display full audit header in failures-only mode"
    );
}

#[test]
fn test_mutation_render_audit_report_failures_only_renders_warnings_in_strict() {
    let theme = Theme::with_color(false);
    let report = cccp::audit::AuditReport {
        gost_version: "2.1.0".to_string(),
        profile: "https://example.org/profile".to_string(),
        languages: "rust".to_string(),
        checks: vec![
            cccp::audit::AuditCheck {
                code: "PASS-001",
                title: "Passing 1",
                detail: "OK".to_string(),
                status: CheckStatus::Pass,
            },
            cccp::audit::AuditCheck {
                code: "WARN-001",
                title: "Warning 1",
                detail: "Mild divergence".to_string(),
                status: CheckStatus::Warn,
            },
        ],
        passed: 1,
        warnings: 1,
        failures: 0,
        percentage: 50,
        is_success: false,
    };

    let rendered = cccp::audit::render_audit_report_with_options(&report, true, true, &theme);
    assert!(
        rendered.contains("WARN-001"),
        "Warnings must be displayed in failures-only when strict: {}",
        rendered
    );
    assert!(
        !rendered.contains("PASS-001"),
        "Passed checks must still be suppressed"
    );
}


