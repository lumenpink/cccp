use cccp::catalog::{CatalogEntry, CiPipelineStatus};
use cccp::forges::{ForgeClient, ForgeType, RepoForgeInfo};

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
