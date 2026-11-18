use cccp::audit::{autodetect_languages, read_gostver};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_gostver_resolves_from_gost_md_frontmatter_without_gostver_file() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Sem GOSTVER na raiz
    assert!(!root.join("GOSTVER").exists());

    // Cria .gost/GOST.md com frontmatter
    let gost_dir = root.join(".gost");
    fs::create_dir_all(&gost_dir).unwrap();
    let gost_content = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
locale: "pt-br"
---

# Especificação
"#;
    fs::write(gost_dir.join("GOST.md"), gost_content).unwrap();

    // Cria Cargo.toml para autodetectar linguagem Rust
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    let (ver, prof, langs) = read_gostver(root);
    assert_eq!(ver, "2.1.0");
    assert!(prof.contains("lilya"));
    assert_eq!(langs, "rust");
}

#[test]
fn test_autodetect_languages() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Sem manifestos
    assert_eq!(autodetect_languages(root), "none");

    // Rust
    fs::write(root.join("Cargo.toml"), "").unwrap();
    assert_eq!(autodetect_languages(root), "rust");

    // Javascript
    fs::remove_file(root.join("Cargo.toml")).unwrap();
    fs::write(root.join("package.json"), "{}").unwrap();
    assert_eq!(autodetect_languages(root), "javascript");
}
