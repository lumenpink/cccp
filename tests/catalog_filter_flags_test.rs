use cccp::catalog::{filter_and_sort_summaries, CatalogRepoSummary};
use cccp::cli::{CatalogFilterArgs, Cli, Commands, RepoSubcommands};
use clap::Parser;

#[test]
fn test_cli_parsing_dirty_flag() {
    let parsed = Cli::parse_from(["cccp", "repo", "list", "--dirty"]);
    match parsed.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::List(filter)) => {
                assert!(filter.dirty);
                assert!(!filter.unpushed);
            }
            _ => panic!("Expected RepoSubcommands::List"),
        },
        _ => panic!("Expected Commands::Repo"),
    }
}

#[test]
fn test_cli_parsing_unpushed_flag_and_alias() {
    let parsed = Cli::parse_from(["cccp", "repo", "list", "--unpushed"]);
    match parsed.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::List(filter)) => {
                assert!(!filter.dirty);
                assert!(filter.unpushed);
            }
            _ => panic!("Expected RepoSubcommands::List"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    let parsed_alias = Cli::parse_from(["cccp", "repo", "list", "--pending-push"]);
    match parsed_alias.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::List(filter)) => {
                assert!(filter.unpushed);
            }
            _ => panic!("Expected RepoSubcommands::List with --pending-push"),
        },
        _ => panic!("Expected Commands::Repo"),
    }
}

#[test]
fn test_cli_conflict_dirty_and_unpushed() {
    let result = Cli::try_parse_from(["cccp", "repo", "list", "--dirty", "--unpushed"]);
    assert!(
        result.is_err(),
        "Using --dirty and --unpushed together must be rejected"
    );
}

#[test]
fn test_filter_and_sort_dirty_and_unpushed() {
    let summaries = vec![
        CatalogRepoSummary {
            name: "repo-clean-synced".to_string(),
            forge: "codeberg".to_string(),
            slug: "lumenpink/repo-clean-synced".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: None,
            gost_version: None,
            ci_status: Vec::new(),
            is_mirror: false,
        },
        CatalogRepoSummary {
            name: "repo-clean-unpushed".to_string(),
            forge: "codeberg".to_string(),
            slug: "lumenpink/repo-clean-unpushed".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(true),
            unpushed: 2,
            status_label: "clean [↑2]".to_string(),
            local_path: None,
            gost_version: None,
            ci_status: Vec::new(),
            is_mirror: false,
        },
        CatalogRepoSummary {
            name: "repo-dirty".to_string(),
            forge: "codeberg".to_string(),
            slug: "lumenpink/repo-dirty".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(false),
            unpushed: 0,
            status_label: "dirty [1*]".to_string(),
            local_path: None,
            gost_version: None,
            ci_status: Vec::new(),
            is_mirror: false,
        },
    ];

    // Testa filtro --dirty
    let mut list_dirty = summaries.clone();
    let filter_dirty = CatalogFilterArgs {
        dirty: true,
        ..Default::default()
    };
    filter_and_sort_summaries(&mut list_dirty, &filter_dirty);
    assert_eq!(list_dirty.len(), 1);
    assert_eq!(list_dirty[0].name, "repo-dirty");

    // Testa filtro --unpushed
    let mut list_unpushed = summaries.clone();
    let filter_unpushed = CatalogFilterArgs {
        unpushed: true,
        ..Default::default()
    };
    filter_and_sort_summaries(&mut list_unpushed, &filter_unpushed);
    assert_eq!(list_unpushed.len(), 1);
    assert_eq!(list_unpushed[0].name, "repo-clean-unpushed");
}

#[test]
fn test_catalog_filter_args_merge_preserves_dirty_and_unpushed() {
    let base = CatalogFilterArgs {
        dirty: true,
        ..Default::default()
    };
    let sub = CatalogFilterArgs {
        unpushed: false,
        min_tickets: Some(2),
        ..Default::default()
    };
    let merged = base.merge(&sub);
    assert!(merged.dirty);
    assert_eq!(merged.min_tickets, Some(2));

    let base2 = CatalogFilterArgs::default();
    let sub2 = CatalogFilterArgs {
        unpushed: true,
        ..Default::default()
    };
    let merged2 = base2.merge(&sub2);
    assert!(merged2.unpushed);
    assert!(!merged2.dirty);
}
