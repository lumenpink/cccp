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
        ..Default::default()
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
        ..Default::default()
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

#[test]
fn test_parse_repo_identifier() {
    // 1. Full HTTPS URL
    let res = catalog::parse_repo_identifier("https://github.com/rust-lang/cargo.git").unwrap();
    assert_eq!(res.provider, ForgeType::GitHub);
    assert_eq!(res.host, "github.com");
    assert_eq!(res.owner, "rust-lang");
    assert_eq!(res.repo, "cargo");

    // 2. Full SSH URL
    let res_ssh = catalog::parse_repo_identifier("git@codeberg.org:lumenpink/cccp.git").unwrap();
    assert_eq!(res_ssh.provider, ForgeType::Codeberg);
    assert_eq!(res_ssh.host, "codeberg.org");
    assert_eq!(res_ssh.owner, "lumenpink");
    assert_eq!(res_ssh.repo, "cccp");

    // 3. Slug with explicit host
    let res_host_slug = catalog::parse_repo_identifier("gitlab.com/freedesktop/systemd").unwrap();
    assert_eq!(res_host_slug.provider, ForgeType::GitLab);
    assert_eq!(res_host_slug.host, "gitlab.com");
    assert_eq!(res_host_slug.owner, "freedesktop");
    assert_eq!(res_host_slug.repo, "systemd");

    // 4. Short slug defaults to Codeberg
    let res_short = catalog::parse_repo_identifier("lumenpink/kvant").unwrap();
    assert_eq!(res_short.provider, ForgeType::Codeberg);
    assert_eq!(res_short.host, "codeberg.org");
    assert_eq!(res_short.owner, "lumenpink");
    assert_eq!(res_short.repo, "kvant");

    // 5. Invalid identifier
    assert!(catalog::parse_repo_identifier("").is_err());
    assert!(catalog::parse_repo_identifier("justaname").is_err());
}

#[test]
fn test_add_remote_repo_without_prior_clone() {
    let dir = tempdir().expect("Failed to create tempdir");
    let cat_path = dir.path().join("catalog.json");

    let mut config = cccp::config::Config::default();
    config.catalog_file = Some(cat_path.to_string_lossy().to_string());

    // Add first remote repo
    let (entry, path, is_updated) = catalog::add_remote_repo(
        "codeberg.org/lumenpink/newrepo",
        Some("New Repo"),
        Some("A brand new remote repository"),
        Some(&config),
    )
    .unwrap();

    assert_eq!(path, cat_path);
    assert!(!is_updated);
    assert_eq!(entry.name, "New Repo");
    assert_eq!(entry.forge, "codeberg");
    assert_eq!(entry.slug, "lumenpink/newrepo");
    assert_eq!(entry.description, "A brand new remote repository");

    // Verify it is on disk
    let loaded = catalog::load_catalog(Some(&config), None);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].slug, "lumenpink/newrepo");

    // Update existing repo entry
    let (updated_entry, _, was_updated) = catalog::add_remote_repo(
        "lumenpink/newrepo",
        Some("Updated Name"),
        Some("Updated Description"),
        Some(&config),
    )
    .unwrap();

    assert!(was_updated);
    assert_eq!(updated_entry.name, "Updated Name");
    assert_eq!(updated_entry.description, "Updated Description");

    let reloaded = catalog::load_catalog(Some(&config), None);
    assert_eq!(reloaded.len(), 1);
    assert_eq!(reloaded[0].name, "Updated Name");
}

#[test]
fn test_catalog_entry_serialization_full_fields() {
    let entry = CatalogEntry {
        name: "test-app".into(),
        forge: "codeberg".into(),
        slug: "user/test-app".into(),
        description: "Test application".into(),
        topics: vec!["cli".into(), "rust".into(), "gosplan".into()],
        license: "GPL-3.0".into(),
        website: "https://example.org".into(),
        is_upstream: false,
    };

    let serialized = serde_json::to_string_pretty(&vec![entry.clone()]).unwrap();
    assert!(serialized.contains(r#""topics": ["#));
    assert!(serialized.contains(r#""license": "GPL-3.0""#));
    assert!(serialized.contains(r#""website": "https://example.org""#));

    let deserialized: Vec<CatalogEntry> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.len(), 1);
    assert_eq!(deserialized[0], entry);
}

