use cccp::catalog::{add_remote_repo, load_catalog, update_catalog_tickets, UpdateTicketsReport};
use cccp::cli::{CatalogSubcommands, Cli, Commands};
use cccp::config::Config;
use cccp::forges::{ForgeClient, ForgeType, RepoForgeInfo};
use clap::Parser;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_parsing_catalog_sync_alias() {
    // cccp catalog sync should be parsed as CatalogSubcommands::UpdateTickets
    let parsed_sync = Cli::parse_from(["cccp", "catalog", "sync"]);
    match parsed_sync.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::UpdateTickets { repos, all: _ }) => {
                assert!(repos.is_empty());
            }
            _ => panic!("Expected CatalogSubcommands::UpdateTickets via 'sync' alias"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed_sync_repos = Cli::parse_from(["cccp", "catalog", "sync", "repo1", "repo2"]);
    match parsed_sync_repos.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::UpdateTickets { repos, all }) => {
                assert!(!all);
                assert_eq!(repos, vec!["repo1", "repo2"]);
            }
            _ => panic!("Expected CatalogSubcommands::UpdateTickets with repos via 'sync'"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }
}

#[test]
fn test_cli_parsing_sync_catalog_tickets_flag() {
    let parsed = Cli::parse_from(["cccp", "sync", "--catalog-tickets"]);
    match parsed.command {
        Some(Commands::Sync(s)) => {
            assert!(s.catalog_tickets);
            assert!(!s.tickets);
            assert!(!s.agents);
            assert!(!s.catalog);
        }
        _ => panic!("Expected Commands::Sync"),
    }
}

#[test]
fn test_forge_client_fetch_open_issues_count_method() {
    let client = ForgeClient::new();
    let info = RepoForgeInfo::new(
        ForgeType::Codeberg,
        "127.0.0.1:9".to_string(), // Unreachable port
        "testowner".to_string(),
        "testrepo".to_string(),
    );

    // fetch_open_issues_count should return an Err or Ok, but the method MUST exist and be callable directly
    let res = client.fetch_open_issues_count(&info);
    assert!(
        res.is_err(),
        "Unreachable host should return an error gracefully"
    );
}

#[test]
fn test_fetch_open_issues_count_different_forges_error_handling() {
    let client = ForgeClient::new();

    for (forge_type, host) in [
        (ForgeType::GitHub, "127.0.0.1:9"),
        (ForgeType::GitLab, "127.0.0.1:9"),
        (ForgeType::Forgejo, "127.0.0.1:9"),
    ] {
        let info = RepoForgeInfo::new(
            forge_type,
            host.to_string(),
            "fakeowner".to_string(),
            "fakerepo".to_string(),
        );
        let res = client.fetch_open_issues_count(&info);
        assert!(
            res.is_err(),
            "Expected failure for unreachable host on {:?}",
            forge_type
        );
    }
}

#[test]
fn test_catalog_sync_tickets_remote_only_without_local_clone() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // Add a remote repository that does NOT have any local clone on disk
    let (entry, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/remote-uncloned-repo.git",
        Some("Remote Uncloned"),
        Some("Description"),
        &["uncloned".to_string()],
        Some(&config),
    )
    .expect("Add repo failed");

    // Pre-populate open_tickets as Some(5) to simulate cached state
    let mut catalog = load_catalog(Some(&config), None);
    catalog[0].open_tickets = Some(5);
    cccp::catalog::save_catalog(&catalog, Some(&config)).expect("Save failed");

    // Run update_catalog_tickets. Because it is remote and forge might be unreachable or 404 in test,
    // graceful degradation must preserve or update without corrupting catalog.
    let _report: UpdateTicketsReport =
        update_catalog_tickets(&["uncloned".to_string()], false, Some(&config))
            .expect("update_catalog_tickets should not panic");

    // Catalog should still be loadable and maintain integrity
    let reloaded = load_catalog(Some(&config), None);
    assert_eq!(reloaded.len(), 1);
    assert_eq!(reloaded[0].slug, entry.slug);
}

#[test]
fn test_catalog_sync_tickets_batch_isolation_and_persistence() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // 1. Repo local com 3 tickets no arquivo local
    let (entry1, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/batch-local-repo.git",
        Some("Batch Local"),
        Some("Desc 1"),
        &["batchlocal".to_string()],
        Some(&config),
    )
    .expect("Add repo 1 failed");

    let local_dir = tmp.path().join("batch-local-repo");
    fs::create_dir_all(local_dir.join(".antigravity")).expect("mkdir .antigravity");
    fs::create_dir_all(local_dir.join(".git")).expect("mkdir .git");
    fs::write(
        local_dir.join(".antigravity").join("tickets.md"),
        "## [#1] T1\n## [#2] T2\n## [#3] T3\n",
    )
    .expect("write tickets.md");

    // 2. Repo remoto sem clone e com forge fictícia/inexistente
    let (entry2, _, _) = add_remote_repo(
        "git@127.0.0.1:9/lumenpink/batch-unreachable-repo.git",
        Some("Batch Unreachable"),
        Some("Desc 2"),
        &["batchunreach".to_string()],
        Some(&config),
    )
    .expect("Add repo 2 failed");

    // Sincroniza ambos em lote com all = true
    let report: UpdateTicketsReport = update_catalog_tickets(&[], true, Some(&config))
        .expect("Batch update_catalog_tickets failed");

    // O repo 1 deve ter sido atualizado com sucesso via fallback local (3 tickets)
    let reloaded = load_catalog(Some(&config), None);
    let updated1 = reloaded.iter().find(|e| e.slug == entry1.slug).unwrap();
    let updated2 = reloaded.iter().find(|e| e.slug == entry2.slug).unwrap();

    assert_eq!(updated1.open_tickets, Some(3));
    // Repo 2 não possui clone nem forge online, deve estar na lista de falhas sem abortar o repo 1
    assert!(report.failed.iter().any(|(slug, _)| slug == &entry2.slug));
    assert_eq!(updated2.open_tickets, None);
}
