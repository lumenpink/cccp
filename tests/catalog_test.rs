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

    let tildegit_entry = CatalogEntry {
        name: "install_i3".into(),
        forge: "tildegit".into(),
        slug: "lumen/install_i3".into(),
        description: "Setup i3wm".into(),
        is_upstream: false,
        ..Default::default()
    };
    let info3 = tildegit_entry.to_forge_info();
    assert_eq!(info3.provider, ForgeType::Tildegit);
    assert_eq!(info3.software, cccp::forges::ForgeSoftware::Forgejo);
    assert_eq!(info3.host, "tildegit.org");
    assert_eq!(info3.owner, "lumen");
    assert_eq!(info3.repo, "install_i3");
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
    let res_fallback =
        catalog::resolve_target_repo("acme/project", None, Some(dir.path())).unwrap();
    assert_eq!(res_fallback.provider, ForgeType::Codeberg);
    assert_eq!(res_fallback.owner, "acme");
    assert_eq!(res_fallback.repo, "project");

    // Remote URL
    let res_url =
        catalog::resolve_target_repo("https://github.com/org/repo.git", None, Some(dir.path()))
            .unwrap();
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

    let config = cccp::config::Config {
        catalog_file: Some(custom_cat.to_string_lossy().to_string()),
        ..Default::default()
    };

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

    let config = cccp::config::Config {
        catalog_file: Some(cat_path.to_string_lossy().to_string()),
        ..Default::default()
    };

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
        aliases: vec!["app".into(), "cli".into()],
        description: "Test application".into(),
        topics: vec!["cli".into(), "rust".into(), "gosplan".into()],
        license: "GPL-3.0".into(),
        website: "https://example.org".into(),
        is_upstream: false,
    };

    let serialized = serde_json::to_string_pretty(&vec![entry.clone()]).unwrap();
    assert!(serialized.contains(r#""aliases": ["#));
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
    let config = cccp::config::Config {
        catalog_file: Some(cat_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // First scan: should add both repos
    let res = catalog::scan_and_add_from_src(&src_mock, Some(&config)).unwrap();
    assert_eq!(res.added.len(), 2);
    assert_eq!(res.skipped_existing.len(), 0);
    assert_eq!(res.skipped_not_git_or_remote.len(), 1); // just_a_folder

    let loaded = catalog::load_catalog(Some(&config), None);
    assert_eq!(loaded.len(), 2);
    assert!(loaded
        .iter()
        .any(|e| e.slug == "lumenpink/tool_a" && e.forge == "codeberg"));
    assert!(loaded
        .iter()
        .any(|e| e.slug == "someone/tool_b" && e.forge == "github"));

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
            aliases: vec!["core".to_string()],
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
            aliases: vec![],
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
            aliases: vec![],
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
            aliases: vec![],
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
            aliases: vec![],
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
    assert_eq!(list6[0].name, "beta"); // 10
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
    assert_eq!(list7[2].name, "beta"); // 10

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

#[test]
fn test_catalog_alias_deserialization_variants() {
    let json_data = r#"[
        {
            "name": "project-single-alias",
            "forge": "github",
            "slug": "org/proj-single",
            "alias": "app"
        },
        {
            "name": "project-multiple-aliases",
            "forge": "codeberg",
            "slug": "org/proj-multi",
            "aliases": ["cli", "terminal"]
        },
        {
            "name": "project-both-fields",
            "forge": "gitlab",
            "slug": "org/proj-both",
            "alias": "web",
            "aliases": ["frontend", "ui"]
        },
        {
            "name": "project-no-alias",
            "forge": "codeberg",
            "slug": "org/proj-none"
        }
    ]"#;

    let entries: Vec<CatalogEntry> =
        serde_json::from_str(json_data).expect("Failed to deserialize");
    assert_eq!(entries.len(), 4);

    assert_eq!(entries[0].aliases, vec!["app"]);
    assert_eq!(entries[1].aliases, vec!["cli", "terminal"]);
    assert_eq!(entries[2].aliases, vec!["web", "frontend", "ui"]);
    assert!(entries[3].aliases.is_empty());
}

#[test]
fn test_catalog_resolve_by_alias_success() {
    let dir = tempdir().expect("Failed to create tempdir");
    let catalog_json = r#"[
        {
            "name": "kvant",
            "forge": "codeberg",
            "slug": "lumenpink/kvant",
            "alias": "editor"
        },
        {
            "name": "joaju",
            "forge": "codeberg",
            "slug": "lumenpink/joaju",
            "aliases": ["portal", "hub"]
        }
    ]"#;

    let cat_path = dir.path().join("catalog.json");
    fs::write(&cat_path, catalog_json).expect("Failed to write catalog.json");

    // Resolve by alias "editor"
    let resolved = catalog::resolve_target_repo("editor", None, Some(dir.path()));
    assert!(resolved.is_some());
    let info = resolved.unwrap();
    assert_eq!(info.owner, "lumenpink");
    assert_eq!(info.repo, "kvant");

    // Resolve by alias "portal" (from aliases array)
    let resolved_portal = catalog::resolve_target_repo("portal", None, Some(dir.path()));
    assert!(resolved_portal.is_some());
    let info_portal = resolved_portal.unwrap();
    assert_eq!(info_portal.owner, "lumenpink");
    assert_eq!(info_portal.repo, "joaju");

    // Resolve by alias "HUB" (case-insensitive)
    let resolved_hub = catalog::resolve_target_repo("HUB", None, Some(dir.path()));
    assert!(resolved_hub.is_some());
    let info_hub = resolved_hub.unwrap();
    assert_eq!(info_hub.owner, "lumenpink");
    assert_eq!(info_hub.repo, "joaju");
}

#[test]
fn test_catalog_resolve_alias_collision() {
    let dir = tempdir().expect("Failed to create tempdir");
    let catalog_json = r#"[
        {
            "name": "repo-one",
            "forge": "codeberg",
            "slug": "org/repo-one",
            "alias": "shared"
        },
        {
            "name": "repo-two",
            "forge": "github",
            "slug": "org/repo-two",
            "aliases": ["shared", "secondary"]
        }
    ]"#;

    let cat_path = dir.path().join("catalog.json");
    fs::write(&cat_path, catalog_json).expect("Failed to write catalog.json");

    // Attempting to resolve ambiguous alias "shared" should return None due to collision
    let resolved = catalog::resolve_target_repo("shared", None, Some(dir.path()));
    assert!(
        resolved.is_none(),
        "Collision on alias must return None for safety"
    );

    // Unique alias on repo-two still resolves correctly
    let resolved_secondary = catalog::resolve_target_repo("secondary", None, Some(dir.path()));
    assert!(resolved_secondary.is_some());
    assert_eq!(resolved_secondary.unwrap().repo, "repo-two");
}

#[test]
fn test_catalog_resolve_name_or_slug_takes_precedence_over_alias() {
    let dir = tempdir().expect("Failed to create tempdir");
    let catalog_json = r#"[
        {
            "name": "target-app",
            "forge": "codeberg",
            "slug": "org/target-app"
        },
        {
            "name": "other-app",
            "forge": "github",
            "slug": "org/other-app",
            "alias": "target-app"
        }
    ]"#;

    let cat_path = dir.path().join("catalog.json");
    fs::write(&cat_path, catalog_json).expect("Failed to write catalog.json");

    // "target-app" matches repo 1 by exact name, which has precedence over repo 2's alias
    let resolved = catalog::resolve_target_repo("target-app", None, Some(dir.path()));
    assert!(resolved.is_some());
    let info = resolved.unwrap();
    assert_eq!(info.repo, "target-app");
    assert_eq!(info.provider, ForgeType::Codeberg);
}

#[test]
fn test_catalog_cli_aliases() {
    use cccp::cli::{Cli, Commands};
    use clap::Parser;

    let parsed_cat = Cli::parse_from(["cccp", "cat"]);
    assert!(matches!(parsed_cat.command, Some(Commands::Catalog(_))));

    let parsed_c = Cli::parse_from(["cccp", "c"]);
    assert!(matches!(parsed_c.command, Some(Commands::Catalog(_))));

    let parsed_full = Cli::parse_from(["cccp", "catalog"]);
    assert!(matches!(parsed_full.command, Some(Commands::Catalog(_))));
}

#[test]
fn test_batch_push_clean_repos_selection_and_safety() {
    use cccp::catalog::{batch_push_clean_repos, CatalogRepoSummary};
    use cccp::cultural::Theme;
    use std::path::PathBuf;

    let theme = Theme::with_color(false);

    let summaries = vec![
        CatalogRepoSummary {
            name: "clean-ahead".into(),
            is_clean: Some(true),
            unpushed: 2,
            local_path: Some(PathBuf::from("/mock/clean-ahead")),
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "dirty-ahead".into(),
            is_clean: Some(false),
            unpushed: 1,
            local_path: Some(PathBuf::from("/mock/dirty-ahead")),
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "clean-synced".into(),
            is_clean: Some(true),
            unpushed: 0,
            local_path: Some(PathBuf::from("/mock/clean-synced")),
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "remote-only".into(),
            is_clean: None,
            unpushed: 0,
            local_path: None,
            ..Default::default()
        },
    ];

    let report_dry = batch_push_clean_repos(&summaries, true, &theme);
    assert_eq!(report_dry.dry_run, vec!["clean-ahead"]);
    assert_eq!(report_dry.skipped_dirty, vec!["dirty-ahead"]);
    assert_eq!(report_dry.skipped_up_to_date, vec!["clean-synced"]);
    assert_eq!(report_dry.skipped_remote, vec!["remote-only"]);
    assert!(report_dry.pushed.is_empty());
}

#[test]
fn test_batch_push_report_formatting_and_edge_cases() {
    use cccp::catalog::{format_batch_push_report, BatchPushReport};
    use cccp::cultural::Theme;

    let theme = Theme::with_color(false);

    let mut report = BatchPushReport::default();
    report.dry_run.push("repo-dry".into());
    report.pushed.push("repo-pushed".into());
    report
        .failed
        .push(("repo-failed".into(), "network error".into()));
    report.skipped_dirty.push("repo-dirty".into());
    report.skipped_up_to_date.push("repo-synced".into());
    report.skipped_remote.push("repo-remote".into());

    let formatted = format_batch_push_report(&report, &theme);
    assert!(formatted.contains("Gosplan Pre-Push Batch Report"));
    assert!(formatted.contains("[DRY-RUN]"));
    assert!(formatted.contains("repo-dry"));
    assert!(formatted.contains("Pushed successfully: (1)"));
    assert!(formatted.contains("repo-pushed"));
    assert!(formatted.contains("Push failed: (1)"));
    assert!(formatted.contains("repo-failed: network error"));
    assert!(formatted.contains("Skipped dirty working trees"));
    assert!(formatted.contains("repo-dirty"));
    assert!(formatted.contains("Up-to-date"));
    assert!(formatted.contains("repo-synced"));
    assert!(formatted.contains("Remote only"));
    assert!(formatted.contains("repo-remote"));

    // Case: Empty / all up to date
    let empty_report = BatchPushReport::default();
    let empty_formatted = format_batch_push_report(&empty_report, &theme);
    assert!(empty_formatted.contains("All local repositories are up-to-date with remotes."));
}

#[test]
fn test_batch_push_execution_failure_handling() {
    use cccp::catalog::{batch_push_clean_repos, CatalogRepoSummary};
    use cccp::cultural::Theme;
    use std::path::PathBuf;

    let theme = Theme::with_color(false);

    // Non-existent directory should cleanly fail when trying to execute git push
    let summaries = vec![CatalogRepoSummary {
        name: "fail-repo".into(),
        is_clean: Some(true),
        unpushed: 1,
        local_path: Some(PathBuf::from("/nonexistent/path/for/failure/test")),
        ..Default::default()
    }];

    let report = batch_push_clean_repos(&summaries, false, &theme);
    assert!(report.pushed.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, "fail-repo");
}

#[test]
fn test_batch_assemble_gost_selection_and_dry_run() {
    use cccp::catalog::batch_assemble_gost_repos;
    use cccp::cultural::Theme;
    use tempfile::tempdir;

    let theme = Theme::with_color(false);

    // Repo 1: Local with GOSTVER
    let dir_gost = tempdir().expect("tempdir");
    std::fs::write(dir_gost.path().join("GOSTVER"), "1.4.0\n").expect("write GOSTVER");

    // Repo 2: Local without GOSTVER
    let dir_nogost = tempdir().expect("tempdir");

    let repos = vec![
        ("repo-gost", Some(dir_gost.path())),
        ("repo-nogost", Some(dir_nogost.path())),
        ("repo-remote", None),
    ];

    let report_dry = batch_assemble_gost_repos(&repos, true, &theme);
    assert_eq!(report_dry.dry_run, vec!["repo-gost"]);
    assert_eq!(report_dry.skipped_no_gostver, vec!["repo-nogost"]);
    assert_eq!(report_dry.skipped_remote, vec!["repo-remote"]);
    assert!(report_dry.assembled.is_empty());
}

#[test]
fn test_batch_assemble_gost_execution_and_file_creation() {
    use cccp::catalog::batch_assemble_gost_repos;
    use cccp::cultural::Theme;
    use tempfile::tempdir;

    let theme = Theme::with_color(false);

    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join("GOSTVER"), "1.4.0\n").expect("write GOSTVER");

    let repos = vec![("repo-exec", Some(dir.path()))];

    let report = batch_assemble_gost_repos(&repos, false, &theme);
    assert_eq!(report.assembled, vec!["repo-exec"]);
    assert!(report.failed.is_empty());
    assert!(report.dry_run.is_empty());

    let target_file = dir.path().join(".gost/GOST.md");
    assert!(target_file.is_file());
    let content = std::fs::read_to_string(target_file).expect("read GOST.md");
    assert!(content.contains("ГОСТ / GOSPLAN"));
}

#[test]
fn test_batch_assemble_gost_report_formatting() {
    use cccp::catalog::{format_batch_assemble_report, BatchAssembleReport};
    use cccp::cultural::Theme;

    let theme = Theme::with_color(false);

    let mut report = BatchAssembleReport::default();
    report.dry_run.push("repo-dry".into());
    report.assembled.push("repo-assembled".into());
    report
        .failed
        .push(("repo-fail".into(), "disk error".into()));
    report.skipped_no_gostver.push("repo-nogost".into());
    report.skipped_remote.push("repo-remote".into());

    let formatted = format_batch_assemble_report(&report, &theme);
    assert!(formatted.contains("Gosplan Batch GOST Assembly Report"));
    assert!(formatted.contains("[DRY-RUN]"));
    assert!(formatted.contains("repo-dry"));
    assert!(formatted.contains("Assembled .gost/GOST.md successfully: (1)"));
    assert!(formatted.contains("repo-assembled"));
    assert!(formatted.contains("Failed to assemble GOST: (1)"));
    assert!(formatted.contains("repo-fail: disk error"));
    assert!(formatted.contains("Skipped (no GOSTVER defined): (1)"));
    assert!(formatted.contains("repo-nogost"));
    assert!(formatted.contains("Remote only / unmanaged locally: (1)"));
    assert!(formatted.contains("repo-remote"));

    // Case: Empty report
    let empty_report = BatchAssembleReport::default();
    let empty_formatted = format_batch_assemble_report(&empty_report, &theme);
    assert!(empty_formatted.contains("No catalog repositories found to assemble."));
}

#[test]
fn test_extract_gost_version() {
    let dir = tempdir().expect("Failed to create tempdir");
    let path = dir.path();

    // 1. No GOSTVER file
    assert_eq!(catalog::extract_gost_version(path), None);

    // 2. GOSTVER with double quotes
    let gostver_file = path.join("GOSTVER");
    fs::write(&gostver_file, "# Header\nGOST_VERSION=\"1.4.0\"\nGOST_PROFILE_URL=\"https://example.com\"\n").unwrap();
    assert_eq!(catalog::extract_gost_version(path), Some("1.4.0".to_string()));

    // 3. GOSTVER with single quotes
    fs::write(&gostver_file, "GOST_VERSION='2.0.1'\n").unwrap();
    assert_eq!(catalog::extract_gost_version(path), Some("2.0.1".to_string()));

    // 4. GOSTVER unquoted
    fs::write(&gostver_file, "GOST_VERSION=0.9.0\n").unwrap();
    assert_eq!(catalog::extract_gost_version(path), Some("0.9.0".to_string()));

    // 5. GOSTVER without GOST_VERSION
    fs::write(&gostver_file, "# Only comment\nFOO=bar\n").unwrap();
    assert_eq!(catalog::extract_gost_version(path), None);
}

#[test]
fn test_catalog_table_gost_version_column_and_sorting() {
    let summaries = vec![
        catalog::CatalogRepoSummary {
            name: "repo_c".to_string(),
            forge: "codeberg".to_string(),
            slug: "org/repo_c".to_string(),
            aliases: vec![],
            ticket_count: 1,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: Some(std::path::PathBuf::from("/path/c")),
            gost_version: Some("1.4.0".to_string()),
        },
        catalog::CatalogRepoSummary {
            name: "repo_a".to_string(),
            forge: "github".to_string(),
            slug: "org/repo_a".to_string(),
            aliases: vec![],
            ticket_count: 2,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: Some(std::path::PathBuf::from("/path/a")),
            gost_version: Some("1.2.0".to_string()),
        },
        catalog::CatalogRepoSummary {
            name: "repo_b".to_string(),
            forge: "codeberg".to_string(),
            slug: "org/repo_b".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: None,
            unpushed: 0,
            status_label: "remote".to_string(),
            local_path: None,
            gost_version: None,
        },
    ];

    let theme = cccp::cultural::Theme::with_color(false);
    let table = catalog::format_catalog_table(&summaries, &theme);

    // Verify header and column contents
    assert!(table.contains("GOST"), "Header should contain GOST column");
    assert!(table.contains("1.4.0"), "Table should display 1.4.0");
    assert!(table.contains("1.2.0"), "Table should display 1.2.0");
    assert!(table.contains("-"), "Table should display '-' for missing gost version");

    // Test sort by gost
    let mut sortable = summaries.clone();
    let filter = cccp::cli::CatalogFilterArgs {
        sort: "gost".to_string(),
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut sortable, &filter);
    // Expected order: repo_a (1.2.0), repo_c (1.4.0), repo_b (None)
    assert_eq!(sortable[0].name, "repo_a");
    assert_eq!(sortable[1].name, "repo_c");
    assert_eq!(sortable[2].name, "repo_b");

    // Test reverse sort by gost
    let mut sortable_rev = summaries.clone();
    let filter_rev = cccp::cli::CatalogFilterArgs {
        sort: "gost".to_string(),
        reverse: true,
        ..Default::default()
    };
    catalog::filter_and_sort_summaries(&mut sortable_rev, &filter_rev);
    assert_eq!(sortable_rev[0].name, "repo_b");
    assert_eq!(sortable_rev[1].name, "repo_c");
    assert_eq!(sortable_rev[2].name, "repo_a");
}

