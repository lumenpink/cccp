use std::fs;
use std::path::PathBuf;

#[test]
fn test_readme_conforms_to_gost_50_lines_ceiling() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let readme_path = manifest_dir.join("README.md");
    let content = fs::read_to_string(&readme_path).expect("README.md must exist in root");

    let line_count = content.lines().count();
    assert!(
        line_count <= 50,
        "README.md excede o teto normativo GOST de 50 linhas (possui {} linhas)",
        line_count
    );

    // Deve preservar quickstart e documentação
    assert!(content.contains("cccp"), "README deve conter nome da ferramenta");
    assert!(
        content.contains("docs/"),
        "README deve referenciar o diretório docs/"
    );
    assert!(
        content.contains("https://cccp.2lp.in"),
        "README deve referenciar o site oficial cccp.2lp.in"
    );
}
