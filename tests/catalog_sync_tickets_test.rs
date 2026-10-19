use cccp::catalog::{
    add_remote_repo, load_catalog, update_catalog_tickets, UpdateTicketsReport,
};
use cccp::cli::{CatalogSubcommands, Cli, Commands};
use cccp::config::Config;
use cccp::forges::{ForgeClient, ForgeType, RepoForgeInfo};
use clap::Parser;
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
    assert!(res.is_err(), "Unreachable host should return an error gracefully");
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
