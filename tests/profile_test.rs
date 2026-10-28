use cccp::profile::{
    assemble_gost_specs, extract_extends_url, get_active_profile, resolve_profile_url,
    set_active_profile, KNOWN_PROFILES,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_profile_aliases() {
    assert_eq!(
        resolve_profile_url("lilya"),
        "https://lilya.2lp.in/profile.md"
    );
    assert_eq!(
        resolve_profile_url("lumen"),
        "https://lilya.2lp.in/profile.md"
    );
    assert_eq!(
        resolve_profile_url("default"),
        "https://lilya.2lp.in/profile.md"
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
fn test_extract_extends_url_from_yaml_or_frontmatter() {
    let content = r#"---
id: lumen
name: "Perfil Lumen"
extends: "https://kollontai.2lp.in/profile.md"
---

# Lumen Profile Body
"#;
    assert_eq!(
        extract_extends_url(content),
        Some("https://kollontai.2lp.in/profile.md".to_string())
    );

    let content_no_extends = r#"---
id: gost
---
# Base GOST
"#;
    assert_eq!(extract_extends_url(content_no_extends), None);
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

    // Altera para lilya usando alias
    let updated_url = set_active_profile(root, "lilya").unwrap();
    assert_eq!(updated_url, "https://lilya.2lp.in/profile.md");

    let active_after = get_active_profile(root).unwrap();
    assert_eq!(active_after, "https://lilya.2lp.in/profile.md");

    // Verifica conteúdo preservado do arquivo
    let content = fs::read_to_string(root.join("GOSTVER")).unwrap();
    assert!(content.contains("GOST_VERSION=\"1.4.0\""));
    assert!(content.contains("GOST_PROFILE_URL=\"https://lilya.2lp.in/profile.md\""));
}

#[test]
fn test_assemble_gost_specs_with_custom_md() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let gostver = "GOST_VERSION=\"1.4.0\"\nGOST_PROFILE_URL=\"https://lilya.2lp.in/profile.md\"\n";
    fs::write(root.join("GOSTVER"), gostver).unwrap();

    // Cria CUSTOM.md local
    let custom_md = "# Regras Customizadas do Repositório\n* Idioma Oficial: Espanhol (es)\n";
    fs::write(root.join("CUSTOM.md"), custom_md).unwrap();

    // Monta o arquivo .gost/GOST.md
    let assembled = assemble_gost_specs(root).unwrap();
    assert!(root.join(".gost/GOST.md").exists());

    let content = fs::read_to_string(root.join(".gost/GOST.md")).unwrap();
    assert!(content.contains("GOST — Padrão Técnico para Repositórios Abertos"));
    assert!(content.contains("Perfil de Implementação"));
    assert!(content.contains("CUSTOM.md"));
    assert!(content.contains("Idioma Oficial: Espanhol (es)"));
    assert!(assembled.contains("CUSTOM.md"));
}

#[test]
fn test_known_profiles_list() {
    assert!(!KNOWN_PROFILES.is_empty());
    assert!(KNOWN_PROFILES.iter().any(|(alias, _, _)| *alias == "lumen"));
    assert!(KNOWN_PROFILES
        .iter()
        .any(|(alias, _, _)| *alias == "kollontai"));
}
