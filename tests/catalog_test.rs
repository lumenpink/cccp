use cccp::catalog::{self, CatalogEntry};
use cccp::forges::ForgeType;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_catalog_entry_to_forge_info() {
    let github_entry = CatalogEntry {
        name: "aura".into(),
        forge: "github".into(),
        slug: "Canimandra/aura".into(),
        description: "Medical ledger".into(),
        is_upstream: false,
    };
    let info = github_entry.to_forge_info();
    assert_eq!(info.provider, ForgeType::GitHub);
    assert_eq!(info.host, "github.com");
    assert_eq!(info.owner, "Canimandra");
    assert_eq!(info.repo, "aura");

    let codeberg_entry = CatalogEntry {
        name: "cccp".into(),
        forge: "codeberg".into(),
        slug: "lumenpink/cccp".into(),
        description: "Compliance tool".into(),
        is_upstream: false,
    };
    let info2 = codeberg_entry.to_forge_info();
    assert_eq!(info2.provider, ForgeType::Codeberg);
    assert_eq!(info2.host, "codeberg.org");
    assert_eq!(info2.owner, "lumenpink");
    assert_eq!(info2.repo, "cccp");
}

#[test]
fn test_catalog_load_and_resolve() {
    let dir = tempdir().expect("Failed to create tempdir");
    let catalog_json = r#"[
        {
            "name": "kvant",
            "forge": "codeberg",
            "slug": "lumenpink/kvant",
            "description": "Micro terminal text editor"
        },
        {
            "name": "aura",
            "forge": "github",
            "slug": "Canimandra/aura",
            "description": "Medical ecosystem"
        }
    ]"#;

    let cat_path = dir.path().join("catalog.json");
    fs::write(&cat_path, catalog_json).expect("Failed to write catalog.json");

    let loaded = catalog::load_catalog(None, Some(dir.path()));
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "kvant");
    assert_eq!(loaded[1].name, "aura");

    // Resolve by slug
    let res_slug = catalog::resolve_target_repo("Canimandra/aura", None, Some(dir.path())).unwrap();
    assert_eq!(res_slug.provider, ForgeType::GitHub);
    assert_eq!(res_slug.owner, "Canimandra");
    assert_eq!(res_slug.repo, "aura");

    // Resolve by name
    let res_name = catalog::resolve_target_repo("kvant", None, Some(dir.path())).unwrap();
    assert_eq!(res_name.provider, ForgeType::Codeberg);
    assert_eq!(res_name.owner, "lumenpink");
    assert_eq!(res_name.repo, "kvant");

    // Fallback owner/repo
    let res_fallback = catalog::resolve_target_repo("acme/project", None, Some(dir.path())).unwrap();
    assert_eq!(res_fallback.provider, ForgeType::Codeberg);
    assert_eq!(res_fallback.owner, "acme");
    assert_eq!(res_fallback.repo, "project");

    // Remote URL
    let res_url =
        catalog::resolve_target_repo("https://github.com/org/repo.git", None, Some(dir.path())).unwrap();
    assert_eq!(res_url.provider, ForgeType::GitHub);
    assert_eq!(res_url.owner, "org");
    assert_eq!(res_url.repo, "repo");
}

#[test]
fn test_catalog_path_from_config_and_env_precedence() {
    let dir = tempdir().expect("Failed to create tempdir");
    let custom_cat = dir.path().join("my_custom_catalog.json");
    fs::write(
        &custom_cat,
        r#"[{"name": "custom_proj", "forge": "codeberg", "slug": "custom/proj"}]"#,
    )
    .unwrap();

    let mut config = cccp::config::Config::default();
    config.catalog_file = Some(custom_cat.to_string_lossy().to_string());

    let found = catalog::find_catalog_path(Some(&config), None).unwrap();
    assert_eq!(found, custom_cat);

    let loaded = catalog::load_catalog(Some(&config), None);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "custom_proj");

    let resolved = catalog::resolve_target_repo("custom_proj", Some(&config), None).unwrap();
    assert_eq!(resolved.owner, "custom");
    assert_eq!(resolved.repo, "proj");
}
