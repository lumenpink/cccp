use cccp::audit::{run_gost_audit, CheckStatus};
use cccp::cultural::Theme;
use cccp::lua_rules::{disable_rules_in_config, enable_rules_in_config, read_disabled_rules};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_disable_and_enable_rules() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    // 1. Initially disabled rules list is empty
    assert_eq!(read_disabled_rules(p), Vec::<String>::new());

    // 2. Disable rules
    disable_rules_in_config(p, &["GOST-001".to_string(), "GOST-002".to_string()]).unwrap();
    let disabled = read_disabled_rules(p);
    assert_eq!(disabled, vec!["GOST-001", "GOST-002"]);

    // 3. Enable one rule back
    enable_rules_in_config(p, &["GOST-001".to_string()]).unwrap();
    let remaining = read_disabled_rules(p);
    assert_eq!(remaining, vec!["GOST-002"]);

    // 4. Test audit status is Skip for disabled rules
    fs::write(p.join("GOSTVER"), "GOST_VERSION=\"3.0.0\"\n").unwrap();
    fs::write(p.join("VERSION"), "1.0.0\n").unwrap();
    let rules_toml = r#"
[rules.GOST-002]
name = "TEST-DISABLED"
severity = "error"
script = "return false, 'ERR', {}"
"#;
    fs::write(p.join("rules.toml"), rules_toml).unwrap();

    let theme = Theme::with_color(false);
    let report = run_gost_audit(p, false, &theme);
    let check = report.checks.iter().find(|c| c.code == "GOST-002");
    assert!(check.is_some());
    assert_eq!(check.unwrap().status, CheckStatus::Skip);
}
