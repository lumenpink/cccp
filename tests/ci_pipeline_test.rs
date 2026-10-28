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
