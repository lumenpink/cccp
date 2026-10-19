use cccp::catalog::{
    add_remote_repo, load_catalog, remove_repo_from_catalog, remove_repo_from_catalog_ext,
};
use cccp::cli::{CatalogSubcommands, Cli, Commands};
use cccp::config::Config;
use clap::Parser;
use tempfile::tempdir;

#[test]
fn test_remove_repo_by_slug_name_and_alias() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

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
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

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
            Some(CatalogSubcommands::Remove { repo, .. }) => {
                assert_eq!(repo, "meu-repo");
            }
            _ => panic!("Expected CatalogSubcommands::Remove"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed_rm = Cli::parse_from(["cccp", "catalog", "rm", "outro-repo"]);
    match parsed_rm.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo, .. }) => {
                assert_eq!(repo, "outro-repo");
            }
            _ => panic!("Expected CatalogSubcommands::Remove via alias rm"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }
}

#[test]
fn test_remove_repo_case_insensitive() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    add_remote_repo(
        "git@codeberg.org:lumenpink/my-cased-repo.git",
        Some("My Cased Repo"),
        Some("Desc"),
        &["MyAlias".to_string()],
        Some(&config),
    )
    .expect("Add repo failed");

    // Remove usando alias com case diferente
    let (removed, _) = remove_repo_from_catalog("myalias", Some(&config))
        .expect("Remove by lowercase alias failed");
    assert_eq!(removed.name, "My Cased Repo");

    let entries = load_catalog(Some(&config), None);
    assert!(entries.is_empty());
}

fn setup_temp_git_repo(path: &std::path::Path, dirty: bool) {
    std::fs::create_dir_all(path).expect("Failed to create repo dir");
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(path)
        .output();
    std::fs::write(path.join("README.md"), "# Test Repo\n").unwrap();
    let _ = std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(path)
        .output();

    if dirty {
        std::fs::write(path.join("DIRTY.txt"), "untracked modification\n").unwrap();
    }
}

#[test]
fn test_cli_parsing_catalog_remove_delete_dir_flags() {
    let parsed1 = Cli::parse_from(["cccp", "catalog", "remove", "meu-repo", "--delete-dir"]);
    match parsed1.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo, delete_dir }) => {
                assert_eq!(repo, "meu-repo");
                assert!(delete_dir);
            }
            _ => panic!("Expected CatalogSubcommands::Remove with delete_dir"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed2 = Cli::parse_from(["cccp", "catalog", "rm", "outro-repo", "--rm-dir"]);
    match parsed2.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo, delete_dir }) => {
                assert_eq!(repo, "outro-repo");
                assert!(delete_dir);
            }
            _ => panic!("Expected CatalogSubcommands::Remove with rm-dir alias"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed3 = Cli::parse_from(["cccp", "catalog", "rm", "padrao"]);
    match parsed3.command {
        Some(Commands::Catalog(c)) => match c.action {
            Some(CatalogSubcommands::Remove { repo, delete_dir }) => {
                assert_eq!(repo, "padrao");
                assert!(!delete_dir);
            }
            _ => panic!("Expected CatalogSubcommands::Remove with delete_dir false"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }
}

#[test]
fn test_remove_repo_with_clean_dir_deletes_directory_and_catalog() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let repo_dir = tmp.path().join("CleanRepo");
    setup_temp_git_repo(&repo_dir, false);
    assert!(repo_dir.exists());

    add_remote_repo(
        "git@codeberg.org:lumenpink/CleanRepo.git",
        Some("CleanRepo"),
        Some("Desc"),
        &[],
        Some(&config),
    )
    .expect("Add repo failed");

    let initial = load_catalog(Some(&config), None);
    assert_eq!(initial.len(), 1);

    // Remove com delete_dir = true em repo limpo
    let (removed, _, deleted_path) = remove_repo_from_catalog_ext("CleanRepo", true, Some(&config))
        .expect("Clean repo removal should succeed");
    assert_eq!(removed.name, "CleanRepo");
    assert!(deleted_path.is_some());
    assert_eq!(deleted_path.unwrap(), repo_dir);

    // O diretório deve ter sido apagado fisicamente
    assert!(!repo_dir.exists(), "Directory should have been deleted");

    // O catálogo deve estar vazio
    let entries = load_catalog(Some(&config), None);
    assert!(entries.is_empty(), "Catalog should be empty");
}

#[test]
fn test_remove_repo_with_dirty_dir_fails_atomically() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let repo_dir = tmp.path().join("DirtyRepo");
    setup_temp_git_repo(&repo_dir, true); // true = dirty
    assert!(repo_dir.exists());

    add_remote_repo(
        "git@codeberg.org:lumenpink/DirtyRepo.git",
        Some("DirtyRepo"),
        Some("Desc"),
        &[],
        Some(&config),
    )
    .expect("Add repo failed");

    let initial = load_catalog(Some(&config), None);
    assert_eq!(initial.len(), 1);

    // Tentativa de remover com delete_dir = true em repo dirty deve FALHAR
    let res = remove_repo_from_catalog_ext("DirtyRepo", true, Some(&config));
    assert!(res.is_err(), "Operation MUST fail when repo is not clean");
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.to_lowercase().contains("clean")
            || err_msg.to_lowercase().contains("dirty")
            || err_msg.to_lowercase().contains("modified"),
        "Error message must mention cleanliness/modifications: {}",
        err_msg
    );

    // Validação da FALHA ATÔMICA:
    // 1. O diretório físico continua intacto
    assert!(repo_dir.exists(), "Dirty directory MUST NOT be deleted");
    assert!(
        repo_dir.join("DIRTY.txt").exists(),
        "Dirty files must be preserved"
    );

    // 2. O catálogo NÃO foi alterado
    let entries = load_catalog(Some(&config), None);
    assert_eq!(
        entries.len(),
        1,
        "Catalog entry MUST NOT be removed if operation failed"
    );
    assert_eq!(entries[0].name, "DirtyRepo");
}

#[test]
fn test_remove_remote_only_repo_with_delete_dir_flag() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // Repo apenas remoto (sem clone local)
    add_remote_repo(
        "git@codeberg.org:lumenpink/RemoteOnly.git",
        Some("RemoteOnly"),
        Some("Desc"),
        &[],
        Some(&config),
    )
    .expect("Add repo failed");

    let initial = load_catalog(Some(&config), None);
    assert_eq!(initial.len(), 1);

    // Remoção com delete_dir = true quando não há diretório local deve ter sucesso no catálogo e retornar None para purged_path
    let (removed, _, purged_path) = remove_repo_from_catalog_ext("RemoteOnly", true, Some(&config))
        .expect("Removal of remote-only repo should succeed");
    assert_eq!(removed.name, "RemoteOnly");
    assert!(purged_path.is_none());

    let entries = load_catalog(Some(&config), None);
    assert!(entries.is_empty());
}

#[test]
fn test_remove_repo_without_delete_dir_preserves_dirty_directory() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let catalog_path = tmp.path().join("catalog.json");
    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let repo_dir = tmp.path().join("DirtyRepoPreserved");
    setup_temp_git_repo(&repo_dir, true);
    assert!(repo_dir.exists());

    add_remote_repo(
        "git@codeberg.org:lumenpink/DirtyRepoPreserved.git",
        Some("DirtyRepoPreserved"),
        Some("Desc"),
        &[],
        Some(&config),
    )
    .expect("Add repo failed");

    // delete_dir = false deve remover do catálogo mantendo o diretório intacto mesmo dirty
    let (removed, _, purged_path) =
        remove_repo_from_catalog_ext("DirtyRepoPreserved", false, Some(&config))
            .expect("Removal without delete_dir should succeed");
    assert_eq!(removed.name, "DirtyRepoPreserved");
    assert!(purged_path.is_none());

    assert!(repo_dir.exists());
    assert!(repo_dir.join("DIRTY.txt").exists());

    let entries = load_catalog(Some(&config), None);
    assert!(entries.is_empty());
}
