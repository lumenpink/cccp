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
fn test_ci_workflow_compiles_linux_amd64_gnu() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join(".forgejo/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("ci.yml must exist");
    assert!(
        workflow.contains("x86_64-unknown-linux-gnu.2.28"),
        "Workflow must compile x86_64-unknown-linux-gnu.2.28 for Linux AMD64"
    );
    assert!(
        workflow.contains("target/x86_64-unknown-linux-gnu/release/cccp"),
        "Workflow must package and publish x86_64-unknown-linux-gnu binary"
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
