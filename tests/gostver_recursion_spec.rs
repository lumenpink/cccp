use cccp::cultural::Theme;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_gostver_without_profile_url_does_not_overflow_stack() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // Cenário Grundnorm / Repo sem perfil declarado
    let gostver_content = r#"# GOSTVER
GOST_VERSION="2.0.0"
GOST_PROFILE_URL=""
GOST_LANGUAGES="shell"
"#;
    fs::write(root.join("GOSTVER"), gostver_content).unwrap();

    // 1. read_gostver não deve estourar a pilha
    let (ver, prof, langs) = cccp::audit::read_gostver(root);
    assert_eq!(ver, "2.0.0");
    assert_eq!(prof, "None");
    assert_eq!(langs, "shell");

    // 2. run_gost_audit_full também deve executar de forma determinística
    let theme = Theme::init(false);
    let report = cccp::audit::run_gost_audit_full(root, false, false, None, None, &theme);
    assert_eq!(report.gost_version, "2.0.0");
    assert_eq!(report.profile, "None");
}
