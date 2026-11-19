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

#[test]
fn test_mutation_gostver_profile_url_variants_and_error_handling() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // Variante 1: GOST_PROFILE_URL="none"
    fs::write(
        root.join("GOSTVER"),
        "GOST_VERSION=\"3.0.0\"\nGOST_PROFILE_URL=\"none\"\n",
    )
    .unwrap();
    let (_, prof, _) = cccp::audit::read_gostver(root);
    assert_eq!(prof, "None");

    let cfg_err = cccp::profile::read_profile_config(root);
    assert!(cfg_err.is_err(), "deve retornar Err para profile 'none'");

    // Variante 2: GOST_PROFILE_URL com espaços
    fs::write(
        root.join("GOSTVER"),
        "GOST_VERSION=\"1.5.0\"\nGOST_PROFILE_URL=\"   \"\n",
    )
    .unwrap();
    let (_, prof2, _) = cccp::audit::read_gostver(root);
    assert_eq!(prof2, "None");

    let cfg_err2 = cccp::profile::read_profile_config(root);
    assert!(
        cfg_err2.is_err(),
        "deve retornar Err para profile com espacos"
    );

    // Variante 3: Ausência de GOST_PROFILE_URL
    fs::write(root.join("GOSTVER"), "GOST_VERSION=\"1.2.0\"\n").unwrap();
    let (_, prof3, _) = cccp::audit::read_gostver(root);
    assert_eq!(prof3, "None");

    let cfg_err3 = cccp::profile::read_profile_config(root);
    assert!(
        cfg_err3.is_err(),
        "deve retornar Err na ausencia de GOST_PROFILE_URL"
    );
}
