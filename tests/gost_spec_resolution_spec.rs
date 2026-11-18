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

#[test]
fn test_setup_gost_environment_creates_anatomy_and_purges_legacy_gostver() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let theme = cccp::cultural::Theme::with_color(false);

    // Cria GOSTVER legado
    fs::write(root.join("GOSTVER"), "GOST_VERSION=\"1.0.0\"\n").unwrap();
    assert!(root.join("GOSTVER").exists());

    // Cria Cargo.toml
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let res = cccp::profile::setup_gost_environment(root, None, false, &theme);
    assert!(res.is_ok());

    // GOSTVER legado deve ter sido purgado
    assert!(!root.join("GOSTVER").exists());

    // Diretório .gost/ e artefatos devem existir
    assert!(root.join(".gost/metadata.json").exists());
    assert!(root.join(".gost/delivery.toml").exists());
}

#[test]
fn test_cli_parsing_gost_setup() {
    use cccp::cli::{Cli, Commands, ProfileCommand};
    use clap::Parser;

    let args = Cli::parse_from(["cccp", "gost", "setup", "--force"]);
    match args.command {
        Some(Commands::Profile(ProfileCommand::Setup(s))) => {
            assert!(s.force);
        }
        _ => panic!("Expected Commands::Profile(ProfileCommand::Setup)"),
    }
}

#[test]
fn test_mutation_autodetect_languages_python_and_php() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("composer.json"), "{}").unwrap();
    assert_eq!(autodetect_languages(root), "php");

    fs::remove_file(root.join("composer.json")).unwrap();
    fs::write(root.join("pyproject.toml"), "").unwrap();
    assert_eq!(autodetect_languages(root), "python");
}

#[test]
fn test_mutation_extract_frontmatter_handles_whitespace_and_single_quotes() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let gost_dir = root.join(".gost");
    fs::create_dir_all(&gost_dir).unwrap();

    let content = r#"---
standard: 'kollontai'
version: '3.0.0'
---
# Documento
"#;
    fs::write(gost_dir.join("GOST.md"), content).unwrap();

    let (ver, prof, _) = read_gostver(root);
    assert_eq!(ver, "3.0.0");
    assert_eq!(prof, "kollontai");
}
