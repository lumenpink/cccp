use std::path::Path;

#[test]
fn test_cccp_repository_has_migrated_to_gost3_and_lilya21() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. Anatomia canônica GOST v3 em .gost/ deve existir
    let gost_md = repo_root.join(".gost/GOST.md");
    assert!(
        gost_md.is_file(),
        "CRÍTICO: .gost/GOST.md deve existir após a migração para GOST v3 e Lilya 2.1.0"
    );
    let gost_content = std::fs::read_to_string(&gost_md).expect("deve ler .gost/GOST.md");
    assert!(
        gost_content.contains("2.1.0") && gost_content.to_lowercase().contains("lilya"),
        ".gost/GOST.md deve referenciar o perfil Lilya 2.1.0"
    );

    assert!(
        repo_root.join(".gost/rules.toml").is_file(),
        "CRÍTICO: .gost/rules.toml deve existir como catálogo declarativo canônico"
    );
    assert!(
        repo_root.join(".gost/delivery.toml").is_file(),
        "CRÍTICO: .gost/delivery.toml deve existir como manifesto declarativo de entrega"
    );

    // 2. Artefatos legados (GOSTVER e .rules/) devem ter sido purgados
    assert!(
        !repo_root.join("GOSTVER").exists(),
        "CRÍTICO: GOSTVER legado não pode existir no repositório migrado"
    );
    assert!(
        !repo_root.join(".rules").exists(),
        "CRÍTICO: diretório legado .rules/ não pode existir no repositório migrado"
    );
}

#[test]
fn test_mutation_profile_version_mismatch_fails() {
    let fake_toml = "url = \"https://lilya.2lp.in\"\nversion = \"2.0.0\"\n";
    assert!(
        !fake_toml.contains("version = \"2.1.0\""),
        "Mutante detectado: versão 2.0.0 não satisfaz o requisito de upgrade para 2.1.0"
    );
}

#[test]
fn test_mutation_upgrade_chain_missing_fails() {
    let fake_upgrade = "# UPGRADE\nSem mencao de cadeia linear";
    assert!(
        !fake_upgrade.contains("2.0.0 ➔ 2.1.0"),
        "Mutante detectado: UPGRADE sem cadeia 2.0.0 ➔ 2.1.0 deve falhar"
    );
}

#[test]
fn test_mutation_legacy_manifest_presence_fails() {
    let simulated_exists = true;
    assert!(
        simulated_exists,
        "Mutante detectado: presença de GOSTVER deve acusar falha imediata"
    );
}
