use cccp::audit::run_gost_audit;
use cccp::cultural::Theme;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_lua_rules_execution_with_gost_primitives() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    // Setup basic repository anatomy
    fs::write(p.join("GOSTVER"), "GOST_VERSION=\"3.0.0\"\n").unwrap();
    fs::write(p.join("VERSION"), "1.0.0\n").unwrap();
    fs::write(p.join("LICENSE"), "MIT\n").unwrap();
    fs::write(p.join("README.md"), "# Test Repo\n").unwrap();
    fs::write(p.join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(p.join(".gitignore"), "target/\n").unwrap();
    fs::write(p.join(".editorconfig"), "root = true\n").unwrap();

    // Create rules.toml with Lua scripts
    let rules_toml = r#"
[rules.TEST-001]
name = "CHECK-README-EXISTS"
severity = "error"
scope = "project"
description = "Checks that README.md exists"
script = '''
if not gost.file_exists("README.md") then
    return false, "ERR_NO_README", {}
end
return true
'''
[rules.TEST-001.messages]
ERR_NO_README = "README.md does not exist"

[rules.TEST-002]
name = "CHECK-NONEXISTENT-FAIL"
severity = "error"
scope = "project"
description = "Checks a non-existent file to verify failure and param interpolation"
script = '''
if not gost.file_exists("missing_file.txt") then
    return false, "ERR_FILE_MISSING", { target = "missing_file.txt" }
end
return true
'''
[rules.TEST-002.messages]
ERR_FILE_MISSING = "Target file '{target}' is completely missing"
"#;

    fs::write(p.join("rules.toml"), rules_toml).unwrap();

    let theme = Theme::with_color(false);
    let report = run_gost_audit(p, false, &theme);

    // Rule TEST-001 should pass
    let test_001 = report.checks.iter().find(|c| c.code == "TEST-001");
    assert!(test_001.is_some(), "TEST-001 check must be evaluated");
    assert_eq!(test_001.unwrap().status, cccp::audit::CheckStatus::Pass);

    // Rule TEST-002 should fail with interpolated parameter
    let test_002 = report.checks.iter().find(|c| c.code == "TEST-002");
    assert!(test_002.is_some(), "TEST-002 check must be evaluated");
    assert_eq!(test_002.unwrap().status, cccp::audit::CheckStatus::Fail);
    assert!(
        test_002.unwrap().detail.contains("missing_file.txt"),
        "Detail should interpolate '{{target}}' into 'missing_file.txt': got '{}'",
        test_002.unwrap().detail
    );

    // Test Mutation / Edge cases:
    // 1. Syntax error in rule script
    let syntax_err_toml = r#"
[rules.TEST-ERR]
name = "SYNTAX-ERROR-RULE"
script = "this is not valid lua code!!!"
"#;
    fs::write(p.join("rules.toml"), syntax_err_toml).unwrap();
    let err_report = run_gost_audit(p, false, &theme);
    let test_err = err_report.checks.iter().find(|c| c.code == "TEST-ERR");
    assert!(test_err.is_some());
    assert_eq!(test_err.unwrap().status, cccp::audit::CheckStatus::Fail);
    assert!(test_err.unwrap().detail.contains("Erro de execução no script Lua"));

    // 2. Disabled rule via .gost/config
    fs::create_dir_all(p.join(".gost")).unwrap();
    fs::write(p.join(".gost").join("config"), "DISABLED_RULES=\"TEST-ERR\"\n").unwrap();
    let skip_report = run_gost_audit(p, false, &theme);
    let test_skipped = skip_report.checks.iter().find(|c| c.code == "TEST-ERR");
    assert!(test_skipped.is_some());
    assert_eq!(test_skipped.unwrap().status, cccp::audit::CheckStatus::Skip);
    assert!(test_skipped.unwrap().detail.contains("dispensada por configuração"));
}
