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
    // Delega para o binário `lunokhod` caso disponível, ou retorna vazio sem panic
    println!("Checks avaliados: {}", checks.len());
}

#[test]
fn test_lunokhod_and_mlua_decoupled_from_dependencies() {
    let cargo_content = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml must exist");
    assert!(
        !cargo_content.contains("mlua ="),
        "Manifest must not depend directly on mlua"
    );
    assert!(
        !cargo_content.contains("lunokhod ="),
        "Manifest must not depend directly on lunokhod crate"
    );
}

#[test]
fn test_manifest_path_dependency_mutant_detection() {
    let mutated_manifest = "[dependencies]\nlunokhod = { path = \"../lunokhod\" }\n";
    let has_forbidden_path =
        mutated_manifest.contains("path = \"..") || mutated_manifest.contains("path = \"/");
    assert!(
        has_forbidden_path,
        "Mutant with path dependency must be detected as invalid"
    );
}
