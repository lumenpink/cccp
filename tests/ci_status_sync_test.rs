use cccp::catalog::{
    add_remote_repo, load_catalog, update_catalog_tickets, CatalogEntry, CiPipelineStatus,
};
use cccp::config::Config;
use cccp::forges::{ForgeClient, ForgeType, RepoForgeInfo};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ci_pipeline_status_struct_and_catalog_serde() {
    let json_data = r#"[
        {
            "name": "cccp",
            "forge": "codeberg",
            "slug": "lumenpink/cccp",
            "ci_status": [
                {
                    "name": "CI & Release / Rust Tests & Verification (push)",
                    "status": "success",
                    "description": "Tests passed successfully",
                    "target_url": "/lumenpink/cccp/actions/runs/140/jobs/0",
                    "updated_at": "2026-10-20T02:47:18+02:00"
                },
                {
                    "name": "CI & Release / SonarQube Analysis (push)",
                    "status": "pending",
                    "description": "Waiting to run",
                    "target_url": null,
                    "updated_at": null
                }
            ]
        }
    ]"#;

    let entries: Vec<CatalogEntry> =
        serde_json::from_str(json_data).expect("Deserialization of ci_status failed");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ci_status.len(), 2);
    assert_eq!(
        entries[0].ci_status[0].name,
        "CI & Release / Rust Tests & Verification (push)"
    );
    assert_eq!(entries[0].ci_status[0].status, "success");
    assert_eq!(
        entries[0].ci_status[0].description.as_deref(),
        Some("Tests passed successfully")
    );
    assert_eq!(entries[0].ci_status[1].status, "pending");

    let serialized = serde_json::to_string(&entries).expect("Serialization failed");
    assert!(serialized.contains("Rust Tests & Verification"));
}

#[test]
fn test_forge_client_fetch_ci_status_method_exists() {
    let client = ForgeClient::new();
    let info = RepoForgeInfo::new(
        ForgeType::Codeberg,
        "127.0.0.1:9".to_string(), // Unreachable host
        "testowner".to_string(),
        "testrepo".to_string(),
    );

    let res = client.fetch_ci_status(&info, Some("main"));
    assert!(
        res.is_err(),
        "Unreachable forge host should return an error gracefully"
    );
}

#[test]
fn test_forge_client_fetch_ci_status_across_forge_types() {
    let client = ForgeClient::new();
    for forge_type in [ForgeType::GitHub, ForgeType::GitLab, ForgeType::Forgejo] {
        let info = RepoForgeInfo::new(
            forge_type,
            "127.0.0.1:9".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );
        let res = client.fetch_ci_status(&info, Some("main"));
        assert!(res.is_err());
    }
}

#[test]
fn test_catalog_update_preserves_ci_status_on_forge_failure() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let (_entry, _, _) = add_remote_repo(
        "git@127.0.0.1:9/owner/repo.git",
        Some("Repo with CI"),
        Some("Desc"),
        &["ci-repo".to_string()],
        Some(&config),
    )
    .expect("Add repo failed");

    // Populate existing ci_status
    let mut catalog = load_catalog(Some(&config), None);
    catalog[0].ci_status = vec![CiPipelineStatus {
        name: "Build Pipeline".to_string(),
        status: "success".to_string(),
        description: Some("Previous run succeeded".to_string()),
        target_url: None,
        updated_at: Some("2026-10-19T20:00:00Z".to_string()),
    }];
    cccp::catalog::save_catalog(&catalog, Some(&config)).expect("Save failed");

    // Run catalog update tickets, which also attempts CI sync
    let _report = update_catalog_tickets(&["ci-repo".to_string()], false, Some(&config))
        .expect("update_catalog_tickets should succeed gracefully");

    // Existing ci_status should be preserved despite unreachable forge
    let reloaded = load_catalog(Some(&config), None);
    assert_eq!(reloaded.len(), 1);
    assert_eq!(reloaded[0].ci_status.len(), 1);
    assert_eq!(reloaded[0].ci_status[0].status, "success");
}

#[test]
fn test_ci_status_file_persistence() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let antigravity_dir = tmp.path().join(".antigravity");
    fs::create_dir_all(&antigravity_dir).expect("create dir");

    let ci_file = antigravity_dir.join("ci_status.json");
    let statuses = vec![CiPipelineStatus {
        name: "Linter".to_string(),
        status: "success".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];

    let json = serde_json::to_string_pretty(&statuses).expect("serialize");
    fs::write(&ci_file, json).expect("write");

    let read_back: Vec<CiPipelineStatus> =
        serde_json::from_str(&fs::read_to_string(&ci_file).unwrap()).expect("deserialize");
    assert_eq!(read_back.len(), 1);
    assert_eq!(read_back[0].name, "Linter");
}
