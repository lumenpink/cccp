use cccp::profile::assemble_gost_specs;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_assemble_embeds_languages_and_purges_legacy_dirs() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join(".gost").join("languages")).unwrap();
    fs::create_dir_all(root.join(".rules")).unwrap();
    fs::write(
        root.join(".gost").join("languages").join("rust.md"),
        "# GOST Language Specification: Rust\n\n## Arquitetura de Domínio & SOLID Pragmático\n",
    )
    .unwrap();
    fs::write(root.join(".rules").join("legacy.toml"), "# legacy\n").unwrap();
    fs::write(root.join(".gost").join("rules.json"), "{}\n").unwrap();
    fs::write(root.join(".gost").join("PROFILE.md"), "# legacy\n").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    fs::write(root.join("VERSION"), "1.0.0\n").unwrap();

    let initial_gost = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
locale: "pt-br"
languages: ["rust"]
---

# Test Repo
"#;
    fs::write(root.join(".gost").join("GOST.md"), initial_gost).unwrap();

    let res = assemble_gost_specs(root);
    assert!(res.is_ok(), "assemble_gost_specs falhou: {:?}", res.err());

    let compiled_gost = fs::read_to_string(root.join(".gost").join("GOST.md")).unwrap();

    // 1. Deve embutir a especificação técnica de Rust no GOST.md
    assert!(
        compiled_gost.contains("GOST Language Specification: Rust")
            || compiled_gost.contains("Especificação Técnica da Stack: Rust")
            || compiled_gost.contains("Arquitetura de Domínio & SOLID Pragmático"),
        "GOST.md compilado deve conter a especificação técnica completa de Rust"
    );

    // 2. Não deve criar .rules/
    assert!(
        !root.join(".rules").exists(),
        "assemble_gost_specs não deve criar a pasta legada .rules/"
    );

    // 3. Não deve criar .gost/languages/
    assert!(
        !root.join(".gost").join("languages").exists(),
        "assemble_gost_specs não deve criar a pasta redundante .gost/languages/"
    );

    // 4. Não deve criar .gost/rules.json ou .gost/PROFILE.md
    assert!(
        !root.join(".gost").join("rules.json").exists(),
        "assemble_gost_specs não deve criar rules.json legado"
    );
    assert!(
        !root.join(".gost").join("PROFILE.md").exists(),
        "assemble_gost_specs não deve criar PROFILE.md legado"
    );

    // 5. Teste de Mutação: se a linguagem não for suportada, a compilação ainda deve ser resiliente
    let dir_mut = tempdir().expect("tempdir");
    let root_mut = dir_mut.path();
    fs::create_dir_all(root_mut.join(".gost")).unwrap();
    let gost_unknown = r#"---
standard: "lilya"
tier: "operational"
version: "2.1.0"
locale: "pt-br"
languages: ["desconhecida123"]
---
"#;
    fs::write(root_mut.join(".gost").join("GOST.md"), gost_unknown).unwrap();
    let res_mut = assemble_gost_specs(root_mut);
    assert!(
        res_mut.is_ok(),
        "assemble deve ser resiliente a stacks desconhecidas"
    );
    let content_mut = fs::read_to_string(root_mut.join(".gost").join("GOST.md")).unwrap();
    assert!(!content_mut.contains("ESPECIFICAÇÃO TÁTICA DA STACK: DESCONHECIDA123"));
}
