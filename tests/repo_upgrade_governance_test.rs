use std::path::Path;

#[test]
fn test_cccp_repository_has_migrated_to_gost3_and_lilya21() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. .rules/profile.toml deve existir e apontar para 2.1.0
    let profile_toml = repo_root.join(".rules/profile.toml");
    assert!(
        profile_toml.is_file(),
        "CRÍTICO: .rules/profile.toml deve existir após a migração para Lilya 2.1.0"
    );
    let profile_content =
        std::fs::read_to_string(&profile_toml).expect("deve ler .rules/profile.toml");
    assert!(
        profile_content.contains("version = \"2.1.0\""),
        ".rules/profile.toml deve declarar a versão 2.1.0"
    );

    // 2. .gost/UPGRADE.md deve existir e documentar o salto 2.0.0 ➔ 2.1.0
    let upgrade_md = repo_root.join(".gost/UPGRADE.md");
    assert!(
        upgrade_md.is_file(),
        "CRÍTICO: .gost/UPGRADE.md deve ser gerado pelo cccp upgrade"
    );
    let upgrade_content =
        std::fs::read_to_string(&upgrade_md).expect("deve ler .gost/UPGRADE.md");
    assert!(
        upgrade_content.contains("2.0.0 ➔ 2.1.0"),
        ".gost/UPGRADE.md deve relatar o elo de migração 2.0.0 ➔ 2.1.0"
    );

    // 3. GOSTVER deve ter sido purgado (Fail-Fast estrito sem resíduos)
    let legacy_gostver = repo_root.join("GOSTVER");
    assert!(
        !legacy_gostver.exists(),
        "CRÍTICO: GOSTVER legado não pode existir no repositório migrado"
    );
}
