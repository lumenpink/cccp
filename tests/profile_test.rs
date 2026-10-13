use cccp::profile::{get_active_profile, resolve_profile_url, set_active_profile, KNOWN_PROFILES};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_profile_aliases() {
    assert_eq!(
        resolve_profile_url("lumen"),
        "https://gostlumen.2lp.in/profile.md"
    );
    assert_eq!(
        resolve_profile_url("default"),
        "https://gostlumen.2lp.in/profile.md"
    );
    assert_eq!(
        resolve_profile_url("kollontai"),
        "https://kollontai.2lp.in/profile.md"
    );
    assert_eq!(
        resolve_profile_url("https://custom.org/profile.md"),
        "https://custom.org/profile.md"
    );
}

#[test]
fn test_set_and_get_active_profile() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Cria GOSTVER inicial
    let initial_gostver =
        "GOST_VERSION=\"1.4.0\"\nGOST_PROFILE_URL=\"https://kollontai.2lp.in/profile.md\"\n";
    fs::write(root.join("GOSTVER"), initial_gostver).unwrap();

    let active = get_active_profile(root).unwrap();
    assert_eq!(active, "https://kollontai.2lp.in/profile.md");

    // Altera para lumen usando alias
    let updated_url = set_active_profile(root, "lumen").unwrap();
    assert_eq!(updated_url, "https://gostlumen.2lp.in/profile.md");

    let active_after = get_active_profile(root).unwrap();
    assert_eq!(active_after, "https://gostlumen.2lp.in/profile.md");

    // Verifica conteúdo preservado do arquivo
    let content = fs::read_to_string(root.join("GOSTVER")).unwrap();
    assert!(content.contains("GOST_VERSION=\"1.4.0\""));
    assert!(content.contains("GOST_PROFILE_URL=\"https://gostlumen.2lp.in/profile.md\""));
}

#[test]
fn test_known_profiles_list() {
    assert!(!KNOWN_PROFILES.is_empty());
    assert!(KNOWN_PROFILES.iter().any(|(alias, _, _)| *alias == "lumen"));
    assert!(KNOWN_PROFILES
        .iter()
        .any(|(alias, _, _)| *alias == "kollontai"));
}
