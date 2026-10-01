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

#[test]
fn test_scan_and_add_from_src_batch() {
    let base_dir = tempdir().expect("Failed to create tempdir");
    let src_mock = base_dir.path().join("src");
    fs::create_dir_all(&src_mock).unwrap();

    // Repo 1: Codeberg
    let repo_a = src_mock.join("tool_a");
    let git_a = repo_a.join(".git");
    fs::create_dir_all(&git_a).unwrap();
    fs::write(
        git_a.join("config"),
        "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = https://codeberg.org/lumenpink/tool_a.git\n",
    )
    .unwrap();

    // Repo 2: GitHub
    let repo_b = src_mock.join("tool_b");
    let git_b = repo_b.join(".git");
    fs::create_dir_all(&git_b).unwrap();
    fs::write(
        git_b.join("config"),
        "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = git@github.com:someone/tool_b.git\n",
    )
    .unwrap();

    // Directory without git
    let not_git = src_mock.join("just_a_folder");
    fs::create_dir_all(&not_git).unwrap();

    // Plain file in src
    fs::write(src_mock.join("README.md"), "# Mock src root").unwrap();

    let cat_path = base_dir.path().join("catalog.json");
    let mut config = cccp::config::Config::default();
    config.catalog_file = Some(cat_path.to_string_lossy().to_string());

    // First scan: should add both repos
    let res = catalog::scan_and_add_from_src(&src_mock, Some(&config)).unwrap();
    assert_eq!(res.added.len(), 2);
    assert_eq!(res.skipped_existing.len(), 0);
    assert_eq!(res.skipped_not_git_or_remote.len(), 1); // just_a_folder

    let loaded = catalog::load_catalog(Some(&config), None);
    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().any(|e| e.slug == "lumenpink/tool_a" && e.forge == "codeberg"));
    assert!(loaded.iter().any(|e| e.slug == "someone/tool_b" && e.forge == "github"));

    // Second scan: idempotency check (all existing should be skipped)
    let res2 = catalog::scan_and_add_from_src(&src_mock, Some(&config)).unwrap();
    assert_eq!(res2.added.len(), 0);
    assert_eq!(res2.skipped_existing.len(), 2);

    let loaded2 = catalog::load_catalog(Some(&config), None);
    assert_eq!(loaded2.len(), 2);
}

#[test]
fn test_count_open_tickets() {
    let dir = tempdir().expect("Failed to create tempdir");
    let ag_dir = dir.path().join(".antigravity");
    fs::create_dir_all(&ag_dir).unwrap();

    let tickets_content = r#"# Active Tickets - testrepo
## [#1] feat: first ticket
- **Status**: `open`
## [#2] fix: second ticket
- **Status**: `open`
## [#3] chore: third ticket
- **Status**: `closed`
"#;
    fs::write(ag_dir.join("tickets.md"), tickets_content).unwrap();

    let count = catalog::count_open_tickets(dir.path());
    assert_eq!(count, 3);

    // Empty dir has 0 tickets
    let empty_dir = tempdir().expect("Failed to create tempdir");
    assert_eq!(catalog::count_open_tickets(empty_dir.path()), 0);
}

#[test]
fn test_format_catalog_table_rendering() {
    let summaries = vec![
        catalog::CatalogRepoSummary {
            name: "cccp".to_string(),
            forge: "codeberg".to_string(),
            slug: "lumenpink/cccp".to_string(),
            ticket_count: 5,
            is_clean: Some(true),
            unpushed: 2,
            status_label: "clean [↑2]".to_string(),
            local_path: Some(std::path::PathBuf::from("/home/user/src/cccp")),
        },
        catalog::CatalogRepoSummary {
            name: "remote_tool".to_string(),
            forge: "github".to_string(),
            slug: "org/remote_tool".to_string(),
            ticket_count: 0,
            is_clean: None,
            unpushed: 0,
            status_label: "remote".to_string(),
            local_path: None,
        },
    ];

    let theme = cccp::cultural::Theme::with_color(false);
    let table = catalog::format_catalog_table(&summaries, &theme);

    assert!(table.contains("REPOSITORY"));
    assert!(table.contains("FORGE"));
    assert!(table.contains("TICKETS"));
    assert!(table.contains("STATUS"));
    assert!(table.contains("LOCAL PATH"));
    assert!(table.contains("cccp"));
    assert!(table.contains("codeberg"));
    assert!(table.contains("clean [↑2]"));
    assert!(table.contains("remote_tool"));
    assert!(table.contains("github"));
    assert!(table.contains("remote"));
}

#[test]
fn test_filter_and_sort_catalog() {
    let base_summaries = vec![
        catalog::CatalogRepoSummary {
            name: "beta".to_string(),
            forge: "github".to_string(),
            slug: "org/beta".to_string(),
            ticket_count: 10,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: Some(std::path::PathBuf::from("/path/beta")),
        },
        catalog::CatalogRepoSummary {
            name: "alpha".to_string(),
            forge: "codeberg".to_string(),
            slug: "org/alpha".to_string(),
            ticket_count: 2,
            is_clean: Some(false),
            unpushed: 3,
            status_label: "dirty [↑3]".to_string(),
            local_path: Some(std::path::PathBuf::from("/path/alpha")),
        },
        catalog::CatalogRepoSummary {
            name: "gamma".to_string(),
            forge: "codeberg".to_string(),
            slug: "org/gamma".to_string(),
            ticket_count: 0,
            is_clean: None,
            unpushed: 0,
            status_label: "remote".to_string(),
            local_path: None,
        },
    ];

    // 1. Filter by status: clean
    let mut list1 = base_summaries.clone();
    let f1 = cccp::cli::CatalogFilterArgs {
        status: Some("clean".to_string()),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list1, &f1);
    assert_eq!(list1.len(), 1);
    assert_eq!(list1[0].name, "beta");

    // 2. Filter by status: dirty
    let mut list2 = base_summaries.clone();
    let f2 = cccp::cli::CatalogFilterArgs {
        status: Some("dirty".to_string()),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list2, &f2);
    assert_eq!(list2.len(), 1);
    assert_eq!(list2[0].name, "alpha");

    // 3. Filter by status: unpushed
    let mut list3 = base_summaries.clone();
    let f3 = cccp::cli::CatalogFilterArgs {
        status: Some("unpushed".to_string()),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list3, &f3);
    assert_eq!(list3.len(), 1);
    assert_eq!(list3[0].name, "alpha");

    // 4. Filter by status: remote
    let mut list4 = base_summaries.clone();
    let f4 = cccp::cli::CatalogFilterArgs {
        status: Some("remote".to_string()),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list4, &f4);
    assert_eq!(list4.len(), 1);
    assert_eq!(list4[0].name, "gamma");

    // 5. Filter by min_tickets
    let mut list5 = base_summaries.clone();
    let f5 = cccp::cli::CatalogFilterArgs {
        min_tickets: Some(5),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list5, &f5);
    assert_eq!(list5.len(), 1);
    assert_eq!(list5[0].name, "beta");

    // 6. Sort by tickets (descending by default)
    let mut list6 = base_summaries.clone();
    let f6 = cccp::cli::CatalogFilterArgs {
        sort: "tickets".to_string(),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list6, &f6);
    assert_eq!(list6[0].name, "beta");  // 10
    assert_eq!(list6[1].name, "alpha"); // 2
    assert_eq!(list6[2].name, "gamma"); // 0

    // 7. Sort by tickets with reverse
    let mut list7 = base_summaries.clone();
    let f7 = cccp::cli::CatalogFilterArgs {
        sort: "tickets".to_string(),
        reverse: true,
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list7, &f7);
    assert_eq!(list7[0].name, "gamma"); // 0
    assert_eq!(list7[1].name, "alpha"); // 2
    assert_eq!(list7[2].name, "beta");  // 10

    // 8. Sort by name (alphabetical)
    let mut list8 = base_summaries.clone();
    let f8 = cccp::cli::CatalogFilterArgs {
        sort: "name".to_string(),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut list8, &f8);
    assert_eq!(list8[0].name, "alpha");
    assert_eq!(list8[1].name, "beta");
    assert_eq!(list8[2].name, "gamma");
}

#[test]
fn test_resolve_target_repo_from_local_directory() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let local_repo = temp_root.path().join("local_tool");
    let git_dir = local_repo.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(
        git_dir.join("config"),
        "[remote \"origin\"]\n\turl = ssh://git@codeberg.org/lumenpink/local_tool.git\n",
    )
    .unwrap();

    let resolved = catalog::resolve_target_repo("local_tool", None, Some(temp_root.path()));
    assert!(resolved.is_some());
    let info = resolved.unwrap();
    assert_eq!(info.provider, ForgeType::Codeberg);
    assert_eq!(info.host, "codeberg.org");
    assert_eq!(info.owner, "lumenpink");
    assert_eq!(info.repo, "local_tool");
}





