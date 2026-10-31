use cccp::catalog::{CatalogRepoSummary, CiPipelineStatus};
use cccp::cultural::Theme;
use cccp::repo_top::{
    apply_top_filters_and_sorting, render_top_view, CiTopFilter, CiTopSort, RepoTopAction,
    RepoTopState,
};

fn make_sample_repo(name: &str, slug: &str, ci: Vec<CiPipelineStatus>) -> CatalogRepoSummary {
    CatalogRepoSummary {
        name: name.to_string(),
        forge: "forgejo".to_string(),
        slug: slug.to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: Some("2.0.0".to_string()),
        ci_status: ci,
        is_mirror: false,
    }
}

#[test]
fn test_ci_top_filters() {
    let repo_ok = make_sample_repo(
        "alpha",
        "org/alpha",
        vec![CiPipelineStatus {
            name: "build".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let repo_fail = make_sample_repo(
        "beta",
        "org/beta",
        vec![CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let repo_queued = make_sample_repo(
        "gamma",
        "org/gamma",
        vec![CiPipelineStatus {
            name: "deploy".to_string(),
            status: "running".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let repo_empty = make_sample_repo("delta", "org/delta", vec![]);

    let list = vec![
        repo_ok.clone(),
        repo_fail.clone(),
        repo_queued.clone(),
        repo_empty.clone(),
    ];

    // All
    let res_all = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::ProcessOrder);
    assert_eq!(res_all.len(), 4);

    // FailOrPending
    let res_fop =
        apply_top_filters_and_sorting(&list, CiTopFilter::FailOrPending, CiTopSort::ProcessOrder);
    assert_eq!(res_fop.len(), 2);
    assert!(res_fop.iter().any(|r| r.name == "beta")); // fail
    assert!(res_fop.iter().any(|r| r.name == "gamma")); // queued

    // Ok
    let res_ok = apply_top_filters_and_sorting(&list, CiTopFilter::Ok, CiTopSort::ProcessOrder);
    assert_eq!(res_ok.len(), 1);
    assert_eq!(res_ok[0].name, "alpha");

    // Failure
    let res_fail =
        apply_top_filters_and_sorting(&list, CiTopFilter::Failure, CiTopSort::ProcessOrder);
    assert_eq!(res_fail.len(), 1);
    assert_eq!(res_fail[0].name, "beta");

    // Queued
    let res_queue =
        apply_top_filters_and_sorting(&list, CiTopFilter::Queued, CiTopSort::ProcessOrder);
    assert_eq!(res_queue.len(), 1);
    assert_eq!(res_queue[0].name, "gamma");

    // NoQueue
    let res_no_queue =
        apply_top_filters_and_sorting(&list, CiTopFilter::NoQueue, CiTopSort::ProcessOrder);
    assert_eq!(res_no_queue.len(), 2);
    assert!(res_no_queue.iter().any(|r| r.name == "alpha"));
    assert!(res_no_queue.iter().any(|r| r.name == "delta"));
}

#[test]
fn test_ci_top_sorting() {
    let r1 = make_sample_repo(
        "zeta",
        "org/zeta",
        vec![CiPipelineStatus {
            name: "build".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let r2 = make_sample_repo(
        "alpha",
        "org/alpha",
        vec![CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let r3 = make_sample_repo("mu", "org/mu", vec![]);

    let list = vec![r1.clone(), r2.clone(), r3.clone()];

    // ProcessOrder keeps original order
    let res_process =
        apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::ProcessOrder);
    assert_eq!(res_process[0].name, "zeta");
    assert_eq!(res_process[1].name, "alpha");
    assert_eq!(res_process[2].name, "mu");

    // RepoName sorts alphabetically
    let res_name = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::RepoName);
    assert_eq!(res_name[0].name, "alpha");
    assert_eq!(res_name[1].name, "mu");
    assert_eq!(res_name[2].name, "zeta");

    // CiStatus sorts by priority: Queued > Failure > Ok > None
    let res_status = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::CiStatus);
    assert_eq!(res_status[0].name, "alpha"); // Failure in this 3-repo list
    assert_eq!(res_status[1].name, "zeta"); // Ok
    assert_eq!(res_status[2].name, "mu"); // No queue / empty
}

#[test]
fn test_ci_top_priority_and_queue_order() {
    let r_ok = make_sample_repo(
        "zeta",
        "org/zeta",
        vec![CiPipelineStatus {
            name: "build".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let r_fail = make_sample_repo(
        "beta",
        "org/beta",
        vec![CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );
    let r_queued1 = make_sample_repo(
        "gamma1",
        "org/gamma1",
        vec![CiPipelineStatus {
            name: "deploy".to_string(),
            status: "running".to_string(),
            description: Some("Runner Job #10".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );
    let r_queued2 = make_sample_repo(
        "gamma2",
        "org/gamma2",
        vec![CiPipelineStatus {
            name: "deploy".to_string(),
            status: "waiting".to_string(),
            description: Some("Runner Job #20".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );
    let r_none = make_sample_repo("omega", "org/omega", vec![]);

    let list = vec![
        r_none.clone(),
        r_ok.clone(),
        r_queued2.clone(),
        r_fail.clone(),
        r_queued1.clone(),
    ];

    let sorted = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::CiStatus);
    // 1st & 2nd: Pending/Queued (gamma1 earlier in runner queue than gamma2)
    assert_eq!(sorted[0].name, "gamma1");
    assert_eq!(sorted[1].name, "gamma2");
    // 3rd: Failure
    assert_eq!(sorted[2].name, "beta");
    // 4th: Pass / Ok
    assert_eq!(sorted[3].name, "zeta");
    // 5th: None
    assert_eq!(sorted[4].name, "omega");
}

#[test]
fn test_ci_top_filter_parsing() {
    use std::str::FromStr;
    assert_eq!(
        CiTopFilter::from_str("fail+pending").unwrap(),
        CiTopFilter::FailOrPending
    );
    assert_eq!(
        CiTopFilter::from_str("pending+fail").unwrap(),
        CiTopFilter::FailOrPending
    );
    assert_eq!(
        CiTopFilter::from_str("issues").unwrap(),
        CiTopFilter::FailOrPending
    );
    assert_eq!(
        CiTopFilter::from_str("alertas").unwrap(),
        CiTopFilter::FailOrPending
    );
    assert_eq!(
        CiTopFilter::from_str("fail-pending").unwrap(),
        CiTopFilter::FailOrPending
    );
}

#[test]
fn test_repo_top_state_navigation_and_help() {
    let mut state = RepoTopState::new(10);
    assert_eq!(state.filter, CiTopFilter::All);
    assert_eq!(state.sort, CiTopSort::CiStatus);
    assert!(!state.show_help);

    // Toggle help
    state.handle_action(RepoTopAction::ToggleHelp, 10);
    assert!(state.show_help);
    state.handle_action(RepoTopAction::ToggleHelp, 10);
    assert!(!state.show_help);

    // Cycle filter: All -> FailOrPending -> Queued -> Failure -> Ok -> NoQueue -> All
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::FailOrPending);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Queued);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Failure);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Ok);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::NoQueue);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::All);

    // Cycle sort: starts at CiStatus -> ProcessOrder -> RepoName -> CiStatus
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::ProcessOrder);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::RepoName);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::CiStatus);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::ProcessOrder);

    // Navigation bounds
    state.handle_action(RepoTopAction::CursorDown, 3);
    assert_eq!(state.selected_index, 1);
    state.handle_action(RepoTopAction::CursorDown, 3);
    assert_eq!(state.selected_index, 2);
    state.handle_action(RepoTopAction::CursorDown, 3);
    assert_eq!(state.selected_index, 2); // clamped at max (total - 1)

    state.handle_action(RepoTopAction::CursorUp, 3);
    assert_eq!(state.selected_index, 1);
    state.handle_action(RepoTopAction::CursorUp, 3);
    assert_eq!(state.selected_index, 0);
    state.handle_action(RepoTopAction::CursorUp, 3);
    assert_eq!(state.selected_index, 0); // clamped at 0
}

#[test]
fn test_render_top_view_output() {
    let theme = Theme::init(true);
    let state = RepoTopState::new(10);
    let r1 = make_sample_repo("omega", "org/omega", vec![]);
    let items = vec![r1];

    let rendered = render_top_view(&items, &state, &theme, 80, 24);
    assert!(rendered.contains("CCCP REPO TOP"));
    assert!(rendered.contains("omega"));
    assert!(rendered.contains("Ctrl+H"));

    let mut help_state = state.clone();
    help_state.show_help = true;
    let help_rendered = render_top_view(&items, &help_state, &theme, 80, 24);
    assert!(help_rendered.contains("Ajuda"));
    assert!(help_rendered.contains("Ctrl+H"));
    assert!(help_rendered.contains("Ctrl+S"));

    // Empty list rendering
    let empty_rendered = render_top_view(&[], &state, &theme, 80, 24);
    assert!(empty_rendered.contains("Nenhum repositório"));

    // Long names truncation
    let long_repo = make_sample_repo(
        "super-extremely-long-repository-name",
        "super-org/super-extremely-long-repository-slug",
        vec![],
    );
    let long_rendered = render_top_view(&[long_repo], &state, &theme, 80, 24);
    assert!(long_rendered.contains('…'));
}

#[test]
fn test_mutation_filters_parsing_and_labels() {
    use std::str::FromStr;

    // Filter labels and display
    assert_eq!(CiTopFilter::All.label(), "all");
    assert_eq!(CiTopFilter::FailOrPending.label(), "fail+pending");
    assert_eq!(CiTopFilter::Ok.label(), "ok");
    assert_eq!(CiTopFilter::Failure.label(), "fail");
    assert_eq!(CiTopFilter::Queued.label(), "queued");
    assert_eq!(CiTopFilter::NoQueue.label(), "no-queue");
    assert_eq!(format!("{}", CiTopFilter::All), "all");
    assert_eq!(format!("{}", CiTopFilter::FailOrPending), "fail+pending");

    // Valid parses
    assert_eq!(CiTopFilter::from_str("all").unwrap(), CiTopFilter::All);
    assert_eq!(CiTopFilter::from_str("todos").unwrap(), CiTopFilter::All);
    assert_eq!(CiTopFilter::from_str("ok").unwrap(), CiTopFilter::Ok);
    assert_eq!(CiTopFilter::from_str("pass").unwrap(), CiTopFilter::Ok);
    assert_eq!(CiTopFilter::from_str("success").unwrap(), CiTopFilter::Ok);
    assert_eq!(CiTopFilter::from_str("fail").unwrap(), CiTopFilter::Failure);
    assert_eq!(
        CiTopFilter::from_str("failure").unwrap(),
        CiTopFilter::Failure
    );
    assert_eq!(
        CiTopFilter::from_str("falha").unwrap(),
        CiTopFilter::Failure
    );
    assert_eq!(
        CiTopFilter::from_str("error").unwrap(),
        CiTopFilter::Failure
    );
    assert_eq!(
        CiTopFilter::from_str("queued").unwrap(),
        CiTopFilter::Queued
    );
    assert_eq!(CiTopFilter::from_str("queue").unwrap(), CiTopFilter::Queued);
    assert_eq!(CiTopFilter::from_str("fila").unwrap(), CiTopFilter::Queued);
    assert_eq!(
        CiTopFilter::from_str("running").unwrap(),
        CiTopFilter::Queued
    );
    assert_eq!(
        CiTopFilter::from_str("pending").unwrap(),
        CiTopFilter::Queued
    );
    assert_eq!(
        CiTopFilter::from_str("noqueue").unwrap(),
        CiTopFilter::NoQueue
    );
    assert_eq!(
        CiTopFilter::from_str("no-queue").unwrap(),
        CiTopFilter::NoQueue
    );
    assert_eq!(CiTopFilter::from_str("none").unwrap(), CiTopFilter::NoQueue);
    assert_eq!(
        CiTopFilter::from_str("sem-fila").unwrap(),
        CiTopFilter::NoQueue
    );

    // Invalid parses
    assert!(CiTopFilter::from_str("invalid").is_err());
    assert!(CiTopFilter::from_str("").is_err());
}

#[test]
fn test_mutation_sort_parsing_and_labels() {
    use std::str::FromStr;

    // Sort labels and display
    assert_eq!(CiTopSort::ProcessOrder.label(), "process");
    assert_eq!(CiTopSort::CiStatus.label(), "status");
    assert_eq!(CiTopSort::RepoName.label(), "name");
    assert_eq!(format!("{}", CiTopSort::CiStatus), "status");

    // Valid parses
    assert_eq!(
        CiTopSort::from_str("process").unwrap(),
        CiTopSort::ProcessOrder
    );
    assert_eq!(
        CiTopSort::from_str("process-order").unwrap(),
        CiTopSort::ProcessOrder
    );
    assert_eq!(
        CiTopSort::from_str("ordem").unwrap(),
        CiTopSort::ProcessOrder
    );
    assert_eq!(CiTopSort::from_str("status").unwrap(), CiTopSort::CiStatus);
    assert_eq!(CiTopSort::from_str("ci").unwrap(), CiTopSort::CiStatus);
    assert_eq!(CiTopSort::from_str("estado").unwrap(), CiTopSort::CiStatus);
    assert_eq!(CiTopSort::from_str("name").unwrap(), CiTopSort::RepoName);
    assert_eq!(CiTopSort::from_str("nome").unwrap(), CiTopSort::RepoName);
    assert_eq!(CiTopSort::from_str("repo").unwrap(), CiTopSort::RepoName);

    // Invalid parses
    assert!(CiTopSort::from_str("invalid").is_err());
    assert!(CiTopSort::from_str("").is_err());
}

#[test]
fn test_mutation_edge_cases_scrolling_and_clamping() {
    // Interval clamped to at least 1
    let state_zero = RepoTopState::new(0);
    assert_eq!(state_zero.interval_secs, 1);

    let mut state = RepoTopState::new(5);

    // Empty list navigation
    state.handle_action(RepoTopAction::CursorDown, 0);
    assert_eq!(state.selected_index, 0);
    state.handle_action(RepoTopAction::CursorUp, 0);
    assert_eq!(state.selected_index, 0);
    state.handle_action(RepoTopAction::PageDown, 0);
    assert_eq!(state.selected_index, 0);
    state.handle_action(RepoTopAction::PageUp, 0);
    assert_eq!(state.selected_index, 0);

    // Page navigation with items
    state.handle_action(RepoTopAction::PageDown, 25);
    assert_eq!(state.selected_index, 10);
    state.handle_action(RepoTopAction::PageDown, 25);
    assert_eq!(state.selected_index, 20);
    state.handle_action(RepoTopAction::PageDown, 25);
    assert_eq!(state.selected_index, 24); // clamped at 24 (25 - 1)

    state.handle_action(RepoTopAction::PageUp, 25);
    assert_eq!(state.selected_index, 14);
    state.handle_action(RepoTopAction::PageUp, 25);
    assert_eq!(state.selected_index, 4);
    state.handle_action(RepoTopAction::PageUp, 25);
    assert_eq!(state.selected_index, 0);

    // Refresh and Quit actions do not mutate indices
    state.handle_action(RepoTopAction::Refresh, 10);
    assert_eq!(state.selected_index, 0);
    state.handle_action(RepoTopAction::Quit, 10);
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_mutation_extract_runner_job_id_edge_cases() {
    use cccp::repo_top::extract_runner_job_id;

    assert_eq!(extract_runner_job_id(&[]), None);

    // Not queued status (e.g. success) must not extract job id
    let ci_success = vec![CiPipelineStatus {
        name: "build".to_string(),
        status: "success".to_string(),
        description: Some("Runner Job #123 on run #4".to_string()),
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(extract_runner_job_id(&ci_success), None);

    // No digits after Runner Job #
    let ci_no_digits = vec![CiPipelineStatus {
        name: "build".to_string(),
        status: "waiting".to_string(),
        description: Some("Runner Job #abc on run #4".to_string()),
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(extract_runner_job_id(&ci_no_digits), None);

    // Valid runner job extraction
    let ci_valid = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "running".to_string(),
        description: Some("Runner Job #9942 on run #100".to_string()),
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(extract_runner_job_id(&ci_valid), Some(9942));
}

#[test]
fn test_mutation_fail_or_pending_filter_mixed() {
    let repo_mixed = make_sample_repo(
        "mixed",
        "org/mixed",
        vec![
            CiPipelineStatus {
                name: "test".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            },
            CiPipelineStatus {
                name: "deploy".to_string(),
                status: "failure".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            },
        ],
    );
    let list = vec![repo_mixed];
    let res = apply_top_filters_and_sorting(&list, CiTopFilter::FailOrPending, CiTopSort::CiStatus);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "mixed");
}
