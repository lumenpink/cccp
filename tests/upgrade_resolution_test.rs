use cccp::profile::{parse_upgrade_frontmatter, resolve_upgrade_chain, UpgradePlan};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_upgrade_frontmatter() {
    let content = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
replaces: "2.0.0"
valid_since: "2.0.0"
breaking: true
---

# Upgrade para Lilya 2.1.0

## 1. Quebras de Contrato
- Purga de manifestos estaticos.

## 2. Mutação de Regras
- Novas regras operacionais.
"#;

    let (fm, body) = parse_upgrade_frontmatter(content).expect("deve parsear frontmatter com sucesso");
    assert_eq!(fm.standard, "lilya");
    assert_eq!(fm.tier, "operational");
    assert_eq!(fm.version, "2.1.0");
    assert_eq!(fm.replaces.as_deref(), Some("2.0.0"));
    assert_eq!(fm.valid_since, "2.0.0");
    assert!(fm.breaking);
    assert!(body.contains("Quebras de Contrato"));
}

#[test]
fn test_upgrade_fail_fast_below_valid_since() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    // Repositório com versão 1.8.0 instalada
    fs::create_dir_all(p.join(".rules")).unwrap();
    fs::write(
        p.join(".rules/profile.toml"),
        "url = \"https://lilya.2lp.in\"\nversion = \"1.8.0\"\n",
    )
    .unwrap();

    let target_upgrade_content = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
replaces: "2.0.0"
valid_since: "2.0.0"
breaking: true
---
Body delta.
"#;

    // Tentativa de upgrade direto a partir de 1.8.0 quando valid_since é 2.0.0
    let res = resolve_upgrade_chain(p, Some("2.1.0"), true, Some(target_upgrade_content));
    assert!(res.is_err(), "deve falhar imediatamente quando versao_instalada < valid_since");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("valid_since") || err_msg.contains("ERR_UPGRADE_BELOW_VALID_SINCE"),
        "mensagem deve indicar violação de valid_since: {}",
        err_msg
    );
}

#[test]
fn test_upgrade_dry_run_leaves_disk_untouched() {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();

    fs::create_dir_all(p.join(".rules")).unwrap();
    fs::write(
        p.join(".rules/profile.toml"),
        "url = \"https://lilya.2lp.in\"\nversion = \"2.0.0\"\n",
    )
    .unwrap();

    let target_upgrade_content = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
replaces: "2.0.0"
valid_since: "2.0.0"
breaking: true
---
Body delta 2.1.0.
"#;

    let res = resolve_upgrade_chain(p, Some("2.1.0"), true, Some(target_upgrade_content));
    assert!(res.is_ok(), "dry-run deve ser bem sucedido: {:?}", res.err());

    // Assegura que .gost/UPGRADE.md NÃO foi criado em dry-run
    assert!(!p.join(".gost/UPGRADE.md").exists(), ".gost/UPGRADE.md não deve ser criado em dry-run");

    // Assegura que .rules/profile.toml não foi alterado
    let cfg = fs::read_to_string(p.join(".rules/profile.toml")).unwrap();
    assert!(cfg.contains("2.0.0"), "profile.toml deve permanecer com 2.0.0 em dry-run");
}
