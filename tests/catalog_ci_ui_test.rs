use cccp::catalog::{
    filter_and_sort_summaries, format_catalog_table, CatalogRepoSummary, CiPipelineStatus,
};
use cccp::cli::{CatalogFilterArgs, Cli, Commands};
use cccp::cultural::Theme;
use clap::Parser;

#[test]
fn test_cli_parsing_ci_filter_and_details_flags() {
    let parsed = Cli::parse_from(["cccp", "catalog", "list", "--ci", "fail", "--ci-details"]);
    match parsed.command {
        Some(Commands::Catalog(c)) => {
            let filter = c.filter;
            assert_eq!(filter.ci.as_deref(), Some("fail"));
            assert!(filter.ci_details);
        }
        _ => panic!("Expected Commands::Catalog"),
    }
}

#[test]
fn test_catalog_repo_summary_ci_status_field_and_table_column() {
    let summary_success = CatalogRepoSummary {
        name: "repo-success".to_string(),
        forge: "codeberg".to_string(),
        slug: "lumenpink/repo-success".to_string(),
        aliases: Vec::new(),
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: Some("1.4.0".to_string()),
        ci_status: vec![CiPipelineStatus {
            name: "Tests".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: Some("2026-10-20T00:00:00Z".to_string()),
        }],
    };

    let summary_fail = CatalogRepoSummary {
        name: "repo-fail".to_string(),
        forge: "codeberg".to_string(),
        slug: "lumenpink/repo-fail".to_string(),
        aliases: Vec::new(),
        ticket_count: 1,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        ci_status: vec![CiPipelineStatus {
            name: "Build".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    };

    let summary_none = CatalogRepoSummary {
        name: "repo-none".to_string(),
        forge: "codeberg".to_string(),
        slug: "lumenpink/repo-none".to_string(),
        aliases: Vec::new(),
        ticket_count: 0,
        is_clean: None,
        unpushed: 0,
        status_label: "remote".to_string(),
        local_path: None,
        gost_version: None,
        ci_status: Vec::new(),
    };

    let theme = Theme::init(false);
    let table = format_catalog_table(&[summary_success, summary_fail, summary_none], &theme);

    // Table MUST contain the CI column
    assert!(table.contains("CI"), "Table header must contain CI column");
    assert!(
        table.contains("pass") || table.contains("✓"),
        "Must render success indicator"
    );
    assert!(
        table.contains("fail") || table.contains("✗"),
        "Must render failure indicator"
    );
}

#[test]
fn test_filter_summaries_by_ci_status() {
    let mut summaries = vec![
        CatalogRepoSummary {
            name: "success-app".to_string(),
            ci_status: vec![CiPipelineStatus {
                name: "CI".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "fail-app".to_string(),
            ci_status: vec![CiPipelineStatus {
                name: "CI".to_string(),
                status: "failure".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            ..Default::default()
        },
    ];

    let filter_fail = CatalogFilterArgs {
        ci: Some("fail".to_string()),
        ..Default::default()
    };

    filter_and_sort_summaries(&mut summaries, &filter_fail);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "fail-app");
}
