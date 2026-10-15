use cccp::catalog::{
    add_remote_repo, load_catalog, update_catalog_tickets, CatalogEntry, UpdateTicketsReport,
};
use cccp::cli::{CatalogSubcommands, Cli, Commands};
use cccp::config::Config;
use clap::Parser;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_parsing_catalog_update_tickets_and_alias() {
    let parsed_all = Cli::parse_from(["cccp", "catalog", "update-tickets", "--all"]);
    match parsed_all.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::UpdateTickets { repos, all }) => {
                assert!(all);
                assert!(repos.is_empty());
            }
            _ => panic!("Expected CatalogSubcommands::UpdateTickets"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed_alias = Cli::parse_from(["cccp", "catalog", "sync-tickets", "repo-a", "repo-b"]);
    match parsed_alias.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::UpdateTickets { repos, all }) => {
                assert!(!all);
                assert_eq!(repos, vec!["repo-a", "repo-b"]);
            }
            _ => panic!("Expected CatalogSubcommands::UpdateTickets via sync-tickets"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }
}

#[test]
fn test_catalog_entry_open_tickets_serde() {
    let json_data = r#"[
        {
            "name": "repo-tickets",
            "forge": "codeberg",
            "slug": "lumenpink/repo-tickets",
            "open_tickets": 7
        },
        {
            "name": "repo-legacy-tickets",
            "forge": "codeberg",
            "slug": "lumenpink/repo-legacy-tickets",
            "tickets": 3
        }
    ]"#;

    let entries: Vec<CatalogEntry> =
        serde_json::from_str(json_data).expect("Deserialization failed");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].open_tickets, Some(7));
    assert_eq!(entries[1].open_tickets, Some(3));

    let serialized = serde_json::to_string(&entries).expect("Serialization failed");
    assert!(serialized.contains("\"open_tickets\":7"));
    assert!(serialized.contains("\"open_tickets\":3"));
}

#[test]
fn test_update_catalog_tickets_fallback_and_persistence() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // Cadastra 2 repositórios
    let (entry1, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/local-sync-repo.git",
        Some("Local Sync"),
        Some("Desc 1"),
        &["localsync".to_string()],
        Some(&config),
    )
    .expect("Add repo 1 failed");

    let (entry2, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/remote-sync-repo.git",
        Some("Remote Sync"),
        Some("Desc 2"),
        &["remotesync".to_string()],
        Some(&config),
    )
    .expect("Add repo 2 failed");

    // Simula um clone local para o primeiro repositório com 2 tickets
    let local_dir = tmp.path().join("local-sync-repo");
    fs::create_dir_all(local_dir.join(".antigravity")).expect("mkdir .antigravity");
    fs::create_dir_all(local_dir.join(".git")).expect("mkdir .git");
    fs::write(
        local_dir.join(".antigravity").join("tickets.md"),
        "## [#1] Ticket um\n## [#2] Ticket dois\n",
    )
    .expect("write tickets.md");

    // Executa update_catalog_tickets apenas para o entry1
    let report: UpdateTicketsReport =
        update_catalog_tickets(&["localsync".to_string()], false, Some(&config))
            .expect("update_catalog_tickets failed");

    assert_eq!(report.updated.len(), 1);
    assert_eq!(report.updated[0].slug, entry1.slug);

    // Recarrega o catálogo e valida que open_tickets foi atualizado e salvo
    let reloaded = load_catalog(Some(&config), None);
    let updated1 = reloaded.iter().find(|e| e.slug == entry1.slug).unwrap();
    let updated2 = reloaded.iter().find(|e| e.slug == entry2.slug).unwrap();

    assert_eq!(updated1.open_tickets, Some(2));
    assert_eq!(updated2.open_tickets, None);
}

#[test]
fn test_update_catalog_tickets_not_found_repo() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    add_remote_repo(
        "git@codeberg.org:lumenpink/present-repo.git",
        Some("Present Repo"),
        Some("Desc"),
        &["present".to_string()],
        Some(&config),
    )
    .expect("Add repo failed");

    let report = update_catalog_tickets(&["inexistente".to_string()], false, Some(&config))
        .expect("Should return report");
    assert!(report.updated.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, "inexistente");
}

#[test]
fn test_update_catalog_tickets_empty_catalog() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let res = update_catalog_tickets(&[], true, Some(&config));
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("empty"));
}
