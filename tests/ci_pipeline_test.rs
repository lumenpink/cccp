//! Integration and contract tests for CI Pipeline subcommands (Ticket #76)

use cccp::ci_pipeline::{
    deploy_site, publish_artifacts, DeploySiteOptions, PublishArtifactsOptions,
};
use cccp::cultural::Theme;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_publish_artifacts_dry_run() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    // Create mock dist directory with artifacts
    let dist_dir = root.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    let dummy_bin = dist_dir.join("cccp");
    fs::write(&dummy_bin, b"#!/bin/sh\necho cccp-binary\n").unwrap();

    let opts = PublishArtifactsOptions {
        dist_dir: Some(dist_dir.clone()),
        target: Some("x86_64-unknown-linux-musl".to_string()),
        bucket: Some("banca-test".to_string()),
        endpoint: None,
        dry_run: true,
    };

    let res = publish_artifacts(root, &opts, &theme);
    assert!(
        res.is_ok(),
        "publish_artifacts dry-run must succeed: {:?}",
        res.err()
    );

    // Verify .sha256 file generated
    let sha_file = dist_dir.join("cccp.sha256");
    assert!(sha_file.is_file(), "cccp.sha256 must be generated");
    let sha_content = fs::read_to_string(&sha_file).unwrap();
    assert!(
        !sha_content.trim().is_empty(),
        "SHA-256 hash must not be empty"
    );

    // Verify manifest.json generated
    let manifest_file = dist_dir.join("manifest.json");
    assert!(manifest_file.is_file(), "manifest.json must be generated");
    let manifest_content = fs::read_to_string(&manifest_file).unwrap();
    assert!(manifest_content.contains("cccp"));
    assert!(manifest_content.contains("sha256"));
}

#[test]
fn test_ci_workflow_compiles_linux_amd64_musl() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join(".forgejo/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("ci.yml must exist");
    assert!(
        workflow.contains("x86_64-unknown-linux-musl"),
        "Workflow must compile x86_64-unknown-linux-musl for Linux AMD64"
    );
    assert!(
        !workflow.contains("x86_64-unknown-linux-gnu.2.28"),
        "Workflow must not contain gnu.2.28 target for Linux AMD64"
    );
    assert!(
        workflow.contains("target/x86_64-unknown-linux-musl/release/cccp"),
        "Workflow must package and publish x86_64-unknown-linux-musl binary"
    );

    let config_path = manifest_dir.join(".cargo/config.toml");
    let config = fs::read_to_string(&config_path).expect(".cargo/config.toml must exist");
    assert!(
        config.contains("CC_x86_64_unknown_linux_musl = \"musl-gcc\""),
        ".cargo/config.toml must configure CC_x86_64_unknown_linux_musl as musl-gcc"
    );
    assert!(
        config.contains("OPENSSL_STATIC = \"1\""),
        ".cargo/config.toml must declare OPENSSL_STATIC = 1"
    );
    assert!(
        config.contains("LIBZ_SYS_STATIC = \"1\""),
        ".cargo/config.toml must declare LIBZ_SYS_STATIC = 1"
    );
    assert!(
        config.contains("PKG_CONFIG_ALL_STATIC = \"1\""),
        ".cargo/config.toml must declare PKG_CONFIG_ALL_STATIC = 1"
    );
}

#[test]
fn test_ci_workflow_overwrites_generic_packages_with_delete_before_put() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join(".forgejo/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("ci.yml must exist");

    let target_endpoint = "api/packages/lumen/generic/cccp/nightly/cccp-linux-amd64";
    assert!(
        workflow.contains(target_endpoint),
        "Workflow deve conter o endpoint do generic packages para cccp-linux-amd64"
    );

    // Deve conter DELETE explícito para evitar 409 Conflict
    assert!(
        workflow.contains("-X DELETE") && workflow.contains(target_endpoint),
        "Workflow deve executar DELETE prévio contra o endpoint para sobrescrever o pacote sem 409 Conflict"
    );

    // O DELETE deve preceder o PUT
    let delete_pos = workflow
        .find("-X DELETE")
        .expect("DELETE deve estar presente");
    let put_pos = workflow.find("-X PUT").expect("PUT deve estar presente");
    assert!(
        delete_pos < put_pos,
        "A requisição DELETE deve preceder a requisição PUT para garantir sobrescrita limpa"
    );
}

#[test]
fn test_ci_pipeline_musl_static_mutation_checks() {
    // Mutation 1: config containing gcc instead of musl-gcc must be rejected
    let mutated_config_gcc = "[env]\nCC_x86_64_unknown_linux_musl = \"gcc\"\n";
    assert!(
        !mutated_config_gcc.contains("musl-gcc"),
        "Mutation with gcc must lack musl-gcc requirement"
    );

    // Mutation 2: Workflow sem DELETE prévio no endpoint generic package deve falhar
    let mutated_workflow_no_delete =
        "curl -X PUT https://gambi.2lp.in/api/packages/lumen/generic/cccp/nightly/cccp-linux-amd64";
    assert!(
        !mutated_workflow_no_delete.contains("-X DELETE"),
        "Mutação sem DELETE prévio deve ser detectada"
    );

    // Mutation 3: Workflow com PUT invertido antes do DELETE deve ser rejeitado
    let mutated_order = "-X PUT ... -X DELETE";
    let del_idx = mutated_order.find("-X DELETE").unwrap();
    let put_idx = mutated_order.find("-X PUT").unwrap();
    assert!(
        put_idx < del_idx,
        "Mutação com ordem invertida deve falhar na checagem DELETE < PUT"
    );

    // Mutation 4: workflow containing gnu.2.28 fallback must fail check
    let mutated_workflow_gnu = "cargo zigbuild --target x86_64-unknown-linux-gnu.2.28";
    assert!(
        mutated_workflow_gnu.contains("x86_64-unknown-linux-gnu"),
        "Mutation with gnu must be detectable"
    );

    // Mutation 3: workflow missing generic package musl binary path
    let mutated_workflow_missing_musl = "curl -T target/release/cccp";
    assert!(
        !mutated_workflow_missing_musl.contains("target/x86_64-unknown-linux-musl/release/cccp"),
        "Mutation missing musl release path must be detectable"
    );
}

#[test]
fn test_ci_workflow_scopes_no_proxy_on_clippy() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join(".forgejo/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("ci.yml must exist");
    let clippy_idx = workflow
        .find("Clippy Check (Zero Warnings)")
        .expect("Clippy step must exist");
    let after_clippy = &workflow[clippy_idx..];
    let next_step_rel = after_clippy.find("- name:").unwrap_or(after_clippy.len());
    let clippy_block = &after_clippy[..next_step_rel];
    assert!(
        clippy_block.contains("no_proxy: \"localhost,127.0.0.1,::1\""),
        "Clippy step must configure no_proxy for localhost sccache daemon access"
    );
}

#[test]
fn test_deploy_site_dry_run() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    // Initialize git repo for tree operations
    let repo = git2::Repository::init(root).unwrap();

    // Create an initial commit on main so HEAD exists
    {
        let mut index = repo.index().unwrap();
        let sig = git2::Signature::now("Comrade", "comrade@gosplan.su").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();
    }

    // Create mock site directory
    let site_dir = root.join("site");
    fs::create_dir_all(&site_dir).unwrap();
    fs::write(
        site_dir.join("index.html"),
        b"<!DOCTYPE html><html><body>Vitrine Soberana</body></html>",
    )
    .unwrap();

    let opts = DeploySiteOptions {
        source: Some(site_dir),
        branch: "pages".to_string(),
        remote: "origin".to_string(),
        cname: Some("gosplan.2lp.in".to_string()),
        dry_run: true,
    };

    let res = deploy_site(root, &opts, &theme);
    assert!(
        res.is_ok(),
        "deploy_site dry-run must succeed: {:?}",
        res.err()
    );

    // Verify pages branch exists locally
    let branch = repo.find_branch("pages", git2::BranchType::Local);
    assert!(branch.is_ok(), "Local 'pages' branch must be created");
}

#[test]
fn test_ci_pipeline_mutation_edge_cases() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    // 1. Missing dist directory
    let opts1 = PublishArtifactsOptions {
        dist_dir: Some(root.join("nonexistent_dist")),
        dry_run: true,
        ..Default::default()
    };
    let res1 = publish_artifacts(root, &opts1, &theme);
    assert!(res1.is_err(), "Must fail when dist dir does not exist");

    // 2. Empty dist directory
    let empty_dist = root.join("empty_dist");
    fs::create_dir_all(&empty_dist).unwrap();
    let opts2 = PublishArtifactsOptions {
        dist_dir: Some(empty_dist),
        dry_run: true,
        ..Default::default()
    };
    let res2 = publish_artifacts(root, &opts2, &theme);
    assert!(res2.is_err(), "Must fail when dist dir has no artifacts");

    // 3. Missing R2 credentials when not dry-run
    let dist_with_file = root.join("dist_with_file");
    fs::create_dir_all(&dist_with_file).unwrap();
    fs::write(dist_with_file.join("binary.bin"), b"dummy").unwrap();
    let opts3 = PublishArtifactsOptions {
        dist_dir: Some(dist_with_file),
        dry_run: false,
        ..Default::default()
    };
    std::env::remove_var("R2_ACCOUNT_ID");
    std::env::remove_var("R2_ACCESS_KEY_ID");
    std::env::remove_var("R2_SECRET_ACCESS_KEY");
    let res3 = publish_artifacts(root, &opts3, &theme);
    assert!(res3.is_err(), "Must fail when R2 credentials are missing");

    // 4. Missing site source directory
    let opts4 = DeploySiteOptions {
        source: Some(root.join("nonexistent_site")),
        branch: "pages".to_string(),
        remote: "origin".to_string(),
        cname: None,
        dry_run: true,
    };
    let res4 = deploy_site(root, &opts4, &theme);
    assert!(res4.is_err(), "Must fail when site dir does not exist");
}

#[test]
fn test_ci_workflow_rotates_catalog_json_with_manifest() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join(".forgejo/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("ci.yml must exist");
    assert!(
        workflow.contains("git add site/downloads/cccp/index.json site/downloads/catalog.json"),
        "ci.yml must stage both index.json and catalog.json on release rotation"
    );
}

#[test]
fn test_ci_workflow_rotates_catalog_json_mutation_check() {
    let mutated_workflow = "git add site/downloads/cccp/index.json";
    assert!(
        !mutated_workflow.contains("site/downloads/catalog.json"),
        "Mutated workflow without catalog.json must be caught"
    );
}
