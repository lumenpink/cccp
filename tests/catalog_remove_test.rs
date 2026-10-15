use cccp::catalog::{add_remote_repo, load_catalog, remove_repo_from_catalog};
use cccp::cli::{CatalogSubcommands, Cli, Commands};
use cccp::config::Config;
use clap::Parser;
use tempfile::tempdir;

#[test]
fn test_remove_repo_by_slug_name_and_alias() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let mut config = Config::default();
    config.catalog_file = Some(catalog_path.to_string_lossy().to_string());

    // 1. Cadastra 2 repositórios de teste
    let (entry1, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/repo-um.git",
        Some("Repo Um"),
        Some("Desc Um"),
        &["um".to_string(), "primeiro".to_string()],
        Some(&config),
    )
    .expect("Add repo 1 failed");

    let (entry2, _, _) = add_remote_repo(
        "git@codeberg.org:lumenpink/repo-dois.git",
        Some("Repo Dois"),
        Some("Desc Dois"),
        &["dois".to_string()],
        Some(&config),
    )
    .expect("Add repo 2 failed");

    let initial = load_catalog(Some(&config), None);
    assert_eq!(initial.len(), 2);

    // 2. Remove repo 1 pelo alias "primeiro"
    let (removed1, _) =
        remove_repo_from_catalog("primeiro", Some(&config)).expect("Remove by alias failed");
    assert_eq!(removed1.slug, entry1.slug);
    assert_eq!(removed1.name, "Repo Um");

    let after_first = load_catalog(Some(&config), None);
    assert_eq!(after_first.len(), 1);
    assert_eq!(after_first[0].slug, entry2.slug);

    // 3. Remove repo 2 pelo slug
    let (removed2, _) =
        remove_repo_from_catalog(&entry2.slug, Some(&config)).expect("Remove by slug failed");
    assert_eq!(removed2.slug, entry2.slug);

    let after_second = load_catalog(Some(&config), None);
    assert!(after_second.is_empty(), "Catalog should be empty");
}

#[test]
fn test_remove_nonexistent_repo_returns_error() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let mut config = Config::default();
    config.catalog_file = Some(catalog_path.to_string_lossy().to_string());

    let res = remove_repo_from_catalog("fantasma", Some(&config));
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("fantasma"));
    assert!(err.contains("not found"));
}

#[test]
fn test_cli_parsing_catalog_remove_and_rm_alias() {
    let parsed_remove = Cli::parse_from(["cccp", "catalog", "remove", "meu-repo"]);
    match parsed_remove.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo }) => {
                assert_eq!(repo, "meu-repo");
            }
            _ => panic!("Expected CatalogSubcommands::Remove"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed_rm = Cli::parse_from(["cccp", "catalog", "rm", "outro-repo"]);
    match parsed_rm.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo }) => {
                assert_eq!(repo, "outro-repo");
            }
            _ => panic!("Expected CatalogSubcommands::Remove via alias rm"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }
}
