use cccp::cli::{Cli, Commands, TicketAction};
use cccp::config::Config;
use clap::Parser;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_top_level_divan_repo_flag_and_positional() {
    // 1. Top-level divan with --repo flag
    let cli = Cli::try_parse_from(["cccp", "divan", "--repo", "lumen/lauda"])
        .expect("should parse cccp divan --repo lumen/lauda");
    match cli.command {
        Some(Commands::Divan(divan)) => {
            let (repo, prompt) = divan.resolve_repo_and_prompt();
            assert_eq!(repo.as_deref(), Some("lumen/lauda"));
            assert_eq!(prompt, None);
        }
        _ => panic!("Expected Commands::Divan"),
    }

    // 2. Top-level divan with -r flag and prompt
    let cli = Cli::try_parse_from([
        "cccp",
        "divan",
        "-r",
        "lumen/lauda",
        "fix broken parser autolink",
    ])
    .expect("should parse cccp divan -r with prompt");
    match cli.command {
        Some(Commands::Divan(divan)) => {
            let (repo, prompt) = divan.resolve_repo_and_prompt();
            assert_eq!(repo.as_deref(), Some("lumen/lauda"));
            assert_eq!(prompt.as_deref(), Some("fix broken parser autolink"));
        }
        _ => panic!("Expected Commands::Divan"),
    }

    // 3. Top-level divan with positional repo only
    let cli = Cli::try_parse_from(["cccp", "divan", "subversao/core"])
        .expect("should parse cccp divan positional repo");
    match cli.command {
        Some(Commands::Divan(divan)) => {
            let (repo, prompt) = divan.resolve_repo_and_prompt();
            assert_eq!(repo.as_deref(), Some("subversao/core"));
            assert_eq!(prompt, None);
        }
        _ => panic!("Expected Commands::Divan"),
    }

    // 4. Ticket divan alias with --repo flag
    let cli = Cli::try_parse_from(["cccp", "ticket", "divan", "--repo", "custom-app"])
        .expect("should parse ticket divan --repo");
    match cli.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Divan(divan)) => {
                let (repo, prompt) = divan.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("custom-app"));
                assert_eq!(prompt, None);
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_resolve_target_repo_workspace_resolution() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create a catalog with a local workspace
    let repo_dir = root.join("my-cool-project");
    fs::create_dir_all(repo_dir.join(".git")).unwrap();

    let catalog_json = r#"[
        {
            "name": "my-cool-project",
            "forge": "https://gambi.2lp.in",
            "slug": "lumen/my-cool-project",
            "aliases": ["mcp"],
            "description": "Test Repo"
        }
    ]"#;
    let catalog_path = root.join("catalog.json");
    fs::write(&catalog_path, catalog_json).unwrap();

    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    // Resolve target repo and its workspace directory via catalog helper
    let resolved = cccp::catalog::resolve_target_repo_workspace("mcp", Some(&config), Some(root));

    assert!(
        resolved.is_some(),
        "Expected target repo to resolve workspace"
    );
    let (info, local_path) = resolved.unwrap();
    assert_eq!(info.repo, "my-cool-project");
    assert_eq!(info.owner, "lumen");
    assert_eq!(local_path, Some(repo_dir));

    // Case-insensitivity check
    let resolved_upper =
        cccp::catalog::resolve_target_repo_workspace("MCP", Some(&config), Some(root));
    assert!(resolved_upper.is_some());
    assert_eq!(resolved_upper.unwrap().0.repo, "my-cool-project");

    // Empty and whitespace input
    assert!(cccp::catalog::resolve_target_repo_workspace("", Some(&config), Some(root)).is_none());
    assert!(
        cccp::catalog::resolve_target_repo_workspace("   ", Some(&config), Some(root)).is_none()
    );

    // Non-existent target
    assert!(cccp::catalog::resolve_target_repo_workspace(
        "non-existent-repo-xyz",
        Some(&config),
        Some(root)
    )
    .is_none());
}
