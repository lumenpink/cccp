use cccp::ddtdd::infer_ticket_from_branch;
use std::path::Path;

#[test]
fn test_branch_inference_supports_ci_build_style() {
    assert_eq!(
        infer_ticket_from_branch("ci/6-test-ci"),
        Some((6, "ci".to_string(), "test-ci".to_string()))
    );
    assert_eq!(
        infer_ticket_from_branch("build/12-test-build"),
        Some((12, "build".to_string(), "test-build".to_string()))
    );
    assert_eq!(
        infer_ticket_from_branch("style/4-test-style"),
        Some((4, "style".to_string(), "test-style".to_string()))
    );
}

#[test]
fn test_lunokhod_engine_delegation() {
    let checks = cccp::lua_rules::evaluate_lua_rules(Path::new("."), &[]);
    // Se o repositório não tiver rules.toml local na raiz, retorna lista vazia mas inicializa sem erro
    println!("Checks avaliados: {}", checks.len());
}

#[test]
fn test_lunokhod_dependency_is_not_path_dependent() {
    let cargo_content = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");
    assert!(
        !cargo_content.contains("lunokhod = { path ="),
        "Manifest contains forbidden path dependency for lunokhod"
    );
    assert!(
        cargo_content.contains("registry = \"gambi\""),
        "Manifest must consume lunokhod via registry = \"gambi\""
    );
}
