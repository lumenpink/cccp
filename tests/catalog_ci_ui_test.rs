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
        Some(Commands::Catalog(c)) => match c.action {
            Some(cccp::cli::CatalogSubcommands::List(filter)) => {
                assert_eq!(filter.ci.as_deref(), Some("fail"));
                assert!(filter.ci_details);
            }
            _ => panic!("Expected CatalogSubcommands::List"),
        },
        _ => panic!("Expected Commands::Catalog"),
    }

    let parsed_root = Cli::parse_from(["cccp", "catalog", "--ci", "success", "--ci-details"]);
    match parsed_root.command {
        Some(Commands::Catalog(c)) => {
            assert_eq!(c.filter.ci.as_deref(), Some("success"));
            assert!(c.filter.ci_details);
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

#[test]
fn test_format_ci_summary_multi_pipelines_and_running() {
    use cccp::catalog::format_ci_summary;

    // Empty
    let (text, style) = format_ci_summary(&[]);
    assert_eq!(text, "-");
    assert_eq!(style, "dim");

    // Single pass
    let (text, style) = format_ci_summary(&[CiPipelineStatus {
        name: "test".to_string(),
        status: "success".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }]);
    assert_eq!(text, "✓ pass");
    assert_eq!(style, "green");

    // Multiple passes
    let (text, style) = format_ci_summary(&[
        CiPipelineStatus {
            name: "test".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "lint".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ]);
    assert_eq!(text, "✓ pass (2)");
    assert_eq!(style, "green");

    // Multiple with 1 failure
    let (text, style) = format_ci_summary(&[
        CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "lint".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ]);
    assert_eq!(text, "✗ fail (1/2)");
    assert_eq!(style, "red");

    // Multiple with 1 running, none failing
    let (text, style) = format_ci_summary(&[
        CiPipelineStatus {
            name: "build".to_string(),
            status: "running".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "lint".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ]);
    assert_eq!(text, "⏳ pending (1/2)");
    assert_eq!(style, "yellow");
}

#[test]
fn test_sort_by_ci_status_order_and_reverse() {
    let make_repo = |name: &str, st: Option<&str>| CatalogRepoSummary {
        name: name.to_string(),
        ci_status: match st {
            Some(s) => vec![CiPipelineStatus {
                name: "job".to_string(),
                status: s.to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            None => Vec::new(),
        },
        ..Default::default()
    };

    let mut list = vec![
        make_repo("repo-none", None),
        make_repo("repo-pass", Some("success")),
        make_repo("repo-fail", Some("failure")),
        make_repo("repo-pending", Some("pending")),
    ];

    let sort_ci = CatalogFilterArgs {
        sort: "ci".to_string(),
        ..Default::default()
    };

    filter_and_sort_summaries(&mut list, &sort_ci);
    let names: Vec<&str> = list.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["repo-fail", "repo-pending", "repo-pass", "repo-none"]
    );

    let sort_ci_rev = CatalogFilterArgs {
        sort: "ci".to_string(),
        reverse: true,
        ..Default::default()
    };

    filter_and_sort_summaries(&mut list, &sort_ci_rev);
    let rev_names: Vec<&str> = list.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        rev_names,
        vec!["repo-none", "repo-pass", "repo-pending", "repo-fail"]
    );
}

#[test]
fn test_filter_by_ci_status_variants() {
    let list = vec![
        CatalogRepoSummary {
            name: "pass-repo".to_string(),
            ci_status: vec![CiPipelineStatus {
                name: "test".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "pending-repo".to_string(),
            ci_status: vec![CiPipelineStatus {
                name: "test".to_string(),
                status: "pending".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            ..Default::default()
        },
        CatalogRepoSummary {
            name: "none-repo".to_string(),
            ci_status: Vec::new(),
            ..Default::default()
        },
    ];

    let mut filtered_pass = list.clone();
    filter_and_sort_summaries(
        &mut filtered_pass,
        &CatalogFilterArgs {
            ci: Some("pass".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(filtered_pass.len(), 1);
    assert_eq!(filtered_pass[0].name, "pass-repo");

    let mut filtered_pending = list.clone();
    filter_and_sort_summaries(
        &mut filtered_pending,
        &CatalogFilterArgs {
            ci: Some("pending".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(filtered_pending.len(), 1);
    assert_eq!(filtered_pending[0].name, "pending-repo");

    let mut filtered_none = list.clone();
    filter_and_sort_summaries(
        &mut filtered_none,
        &CatalogFilterArgs {
            ci: Some("none".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(filtered_none.len(), 1);
    assert_eq!(filtered_none[0].name, "none-repo");
}

#[test]
fn test_format_catalog_ci_details_output() {
    use cccp::catalog::format_catalog_ci_details;

    let theme = Theme::init(false);
    let summaries = vec![CatalogRepoSummary {
        name: "my-app".to_string(),
        forge: "codeberg".to_string(),
        slug: "lumenpink/my-app".to_string(),
        ci_status: vec![CiPipelineStatus {
            name: "Continuous Integration".to_string(),
            status: "success".to_string(),
            description: Some("Build succeeded".to_string()),
            target_url: Some("https://codeberg.org/ci/1".to_string()),
            updated_at: Some("2026-10-20T05:00:00Z".to_string()),
        }],
        ..Default::default()
    }];

    let details = format_catalog_ci_details(&summaries, &theme);
    assert!(details.contains("CI Pipeline Details:"));
    assert!(details.contains("my-app"));
    assert!(details.contains("lumenpink/my-app"));
    assert!(details.contains("Continuous Integration"));
    assert!(details.contains("Build succeeded"));
    assert!(details.contains("https://codeberg.org/ci/1"));
    assert!(details.contains("2026-10-20T05:00:00Z"));

    let empty_details = format_catalog_ci_details(&[], &theme);
    assert!(empty_details.contains("No CI pipelines recorded"));
}
