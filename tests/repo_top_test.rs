use cccp::catalog::{CatalogRepoSummary, CiPipelineStatus};
use cccp::cultural::Theme;
use cccp::repo_top::{
    apply_repo_ci_update, apply_top_filters_and_sorting, render_top_view, CiTopFilter, CiTopSort,
    RepoTopAction, RepoTopState,
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
fn test_active_runner_job_precedence_over_stale_commit_failure() {
    let mut repo = make_sample_repo(
        "gost-ci",
        "lumen/gost-ci",
        vec![CiPipelineStatus {
            name: "ci".to_string(),
            status: "running".to_string(),
            description: Some("Runner Job #123".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );

    // Stale commit status update arrives with "failure"
    let stale_update = vec![CiPipelineStatus {
        name: "ci".to_string(),
        status: "failure".to_string(),
        description: Some("Old failure on commit".to_string()),
        target_url: None,
        updated_at: None,
    }];

    apply_repo_ci_update(&mut repo, stale_update);

    // Active runner job MUST NOT be clobbered by the old failure
    let ci_pipeline = repo.ci_status.iter().find(|p| p.name == "ci").unwrap();
    assert_eq!(
        ci_pipeline.status, "running",
        "Active runner job must maintain 'running' status and not be clobbered by stale failure"
    );
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

    // Cycle sort: starts at CiStatus -> RecentUpdate -> ProcessOrder -> RepoName -> CiStatus
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::RecentUpdate);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::ProcessOrder);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::RepoName);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::CiStatus);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::RecentUpdate);

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
    assert_eq!(CiTopSort::RecentUpdate.label(), "recent");
    assert_eq!(format!("{}", CiTopSort::CiStatus), "status");

    // Valid parses
    assert_eq!(
        CiTopSort::from_str("recent").unwrap(),
        CiTopSort::RecentUpdate
    );
    assert_eq!(
        CiTopSort::from_str("updated").unwrap(),
        CiTopSort::RecentUpdate
    );
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

#[test]
fn test_render_top_view_displays_syncing_spinner_and_convergence_badge() {
    let theme = Theme::with_color(false);
    let mut state = RepoTopState::new(10);
    let items = vec![make_sample_repo("omega", "org/omega", vec![])];

    // 1. Initial idle state (not syncing, not converged)
    assert!(!state.is_syncing);
    assert!(!state.has_converged);
    let initial_view = render_top_view(&items, &state, &theme, 80, 24);
    assert!(!initial_view.contains("sincronizando"));
    assert!(!initial_view.contains("convergente"));

    // 2. Active syncing state with spinner
    state.is_syncing = true;
    state.spinner_frame = 2;
    let syncing_view = render_top_view(&items, &state, &theme, 80, 24);
    assert!(
        syncing_view.contains("sincronizando"),
        "Header must display syncing indicator while is_syncing is true. View:\n{}",
        syncing_view
    );

    // 3. Converged state when syncing completes
    state.is_syncing = false;
    state.has_converged = true;
    let converged_view = render_top_view(&items, &state, &theme, 80, 24);
    assert!(
        converged_view.contains("convergente"),
        "Header must display convergence badge when has_converged is true. View:\n{}",
        converged_view
    );
    assert!(!converged_view.contains("sincronizando"));
}

#[test]
fn test_mutation_repo_top_sync_frames_and_messages() {
    use cccp::repo_top::{TopBackgroundMsg, TOP_SPINNER_FRAMES};

    assert_eq!(TOP_SPINNER_FRAMES.len(), 10);
    for frame in TOP_SPINNER_FRAMES {
        assert_eq!(
            frame.chars().count(),
            1,
            "Spinner frame must be single character width"
        );
    }

    let theme = Theme::with_color(false);
    let mut state = RepoTopState::new(10);
    let items = vec![make_sample_repo("omega", "org/omega", vec![])];

    // Verify syncing takes precedence over has_converged
    state.is_syncing = true;
    state.has_converged = true;
    for frame_idx in 0..TOP_SPINNER_FRAMES.len() * 2 {
        state.spinner_frame = frame_idx;
        let view = render_top_view(&items, &state, &theme, 90, 24);
        let expected_frame = TOP_SPINNER_FRAMES[frame_idx % TOP_SPINNER_FRAMES.len()];
        assert!(view.contains(expected_frame));
        assert!(view.contains("sincronizando"));
        assert!(!view.contains("convergente"));
    }

    // Verify TopBackgroundMsg construction and clones
    let msg1 = TopBackgroundMsg::SyncStarted;
    let msg2 = TopBackgroundMsg::RepoUpdate {
        slug: "org/test".to_string(),
        ci_status: Some(vec![]),
        status_label: Some("clean".to_string()),
        unpushed: Some(0),
        is_clean: Some(true),
    };
    let msg3 = TopBackgroundMsg::SyncCompleted;

    match msg1.clone() {
        TopBackgroundMsg::SyncStarted => {}
        _ => panic!("Expected SyncStarted"),
    }
    match msg2.clone() {
        TopBackgroundMsg::RepoUpdate { slug, unpushed, .. } => {
            assert_eq!(slug, "org/test");
            assert_eq!(unpushed, Some(0));
        }
        _ => panic!("Expected RepoUpdate"),
    }
    match msg3.clone() {
        TopBackgroundMsg::SyncCompleted => {}
        _ => panic!("Expected SyncCompleted"),
    }

    let msg4 = TopBackgroundMsg::RunnerJobsUpdated(vec![]);
    match msg4.clone() {
        TopBackgroundMsg::RunnerJobsUpdated(jobs) => {
            assert!(jobs.is_empty());
        }
        _ => panic!("Expected RunnerJobsUpdated"),
    }
}

#[test]
fn test_mutation_apply_repo_ci_update_edge_cases() {
    let mut repo = make_sample_repo(
        "complex-ci",
        "lumen/complex-ci",
        vec![
            CiPipelineStatus {
                name: "lint".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            },
            CiPipelineStatus {
                name: "test".to_string(),
                status: "running".to_string(),
                description: Some("Job #42".to_string()),
                target_url: Some("https://ci/42".to_string()),
                updated_at: None,
            },
            CiPipelineStatus {
                name: "deploy".to_string(),
                status: "waiting".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            },
        ],
    );

    // 1. Empty new_ci does not clear pipelines
    apply_repo_ci_update(&mut repo, vec![]);
    assert_eq!(repo.ci_status.len(), 3);

    // 2. new_ci updates "lint" to failure, tries to update "test" to failure, and has a new "e2e"
    let update = vec![
        CiPipelineStatus {
            name: "lint".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(), // Stale failure!
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "e2e".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ];

    apply_repo_ci_update(&mut repo, update);

    // "lint" was not active runner, so it correctly becomes "failure"
    let lint = repo.ci_status.iter().find(|p| p.name == "lint").unwrap();
    assert_eq!(lint.status, "failure");

    // "test" WAS active runner, so it MUST STAY "running"
    let test = repo.ci_status.iter().find(|p| p.name == "test").unwrap();
    assert_eq!(test.status, "running");
    assert_eq!(test.description.as_deref(), Some("Job #42"));
    assert_eq!(test.target_url.as_deref(), Some("https://ci/42"));

    // "deploy" was active runner not present in update, so it was preserved/inserted
    let deploy = repo.ci_status.iter().find(|p| p.name == "deploy").unwrap();
    assert_eq!(deploy.status, "waiting");

    // "e2e" was new, so it was included
    let e2e = repo.ci_status.iter().find(|p| p.name == "e2e").unwrap();
    assert_eq!(e2e.status, "success");
}

#[test]
fn test_mutation_pipeline_matches_job_name_and_joaju_pdf_queue_scenario() {
    use cccp::catalog::format_ci_summary;
    use cccp::forges::{merge_live_runner_jobs, pipeline_matches_job_name, ForgeActionRunnerJob};

    // 1. Verify matching behavior on composite contexts
    assert!(pipeline_matches_job_name(
        "CI, Quality & Release / Rust — Checks, Tests & Quality Gates (push)",
        "Rust — Checks, Tests & Quality Gates"
    ));
    assert!(pipeline_matches_job_name(
        "Rust — Checks, Tests & Quality Gates",
        "CI, Quality & Release / Rust — Checks, Tests & Quality Gates (push)"
    ));
    assert!(pipeline_matches_job_name(
        "Build & Lint / Test Suite (pull_request)",
        "Test Suite"
    ));
    assert!(pipeline_matches_job_name("check", "check"));
    assert!(!pipeline_matches_job_name("deploy-prod", "test-suite"));
    assert!(pipeline_matches_job_name(
        "CI/CD & Sovereign Release to Banca / Checks, Tests & Quality Gates (push)",
        "Checks, Tests & Quality Gates"
    ));
    assert!(pipeline_matches_job_name(
        "CI / CD Pipeline / Rust Test, Lint & E2E (GOST-CI) (push)",
        "Rust Test, Lint & E2E (GOST-CI)"
    ));

    // 2. Scenario joaju-pdf: Stale failure + Live waiting runner job
    let mut repos = vec![make_sample_repo(
        "joaju-pdf",
        "lumen/joaju-pdf",
        vec![CiPipelineStatus {
            name: "CI, Quality & Release / Rust — Checks, Tests & Quality Gates (push)".to_string(),
            status: "failure".to_string(),
            description: Some("Workflow failed on previous run".to_string()),
            target_url: Some("https://forge/run/10".to_string()),
            updated_at: None,
        }],
    )];

    let runner_jobs = vec![ForgeActionRunnerJob {
        id: 999,
        run_id: 11,
        attempt: 1,
        handle: "test".to_string(),
        repo_id: 1,
        owner_id: 1,
        name: "Rust — Checks, Tests & Quality Gates".to_string(),
        status: "waiting".to_string(),
        html_url: "https://gambi.2lp.in/lumen/joaju-pdf/actions/runs/11/jobs/999".to_string(),
    }];

    merge_live_runner_jobs(&mut repos, &runner_jobs);

    // Verify pipeline was matched and updated in-place (no duplication)
    assert_eq!(
        repos[0].ci_status.len(),
        1,
        "Must update existing pipeline instead of duplicating"
    );
    assert_eq!(repos[0].ci_status[0].status, "waiting");
    assert_eq!(
        repos[0].ci_status[0].target_url.as_deref(),
        Some("https://gambi.2lp.in/lumen/joaju-pdf/actions/runs/11/jobs/999")
    );

    // Verify UI summary displays "~ waiting" with yellow styling, not failure
    let (summary_text, summary_style) = format_ci_summary(&repos[0].ci_status);
    assert_eq!(summary_text, "~ waiting");
    assert_eq!(summary_style, "yellow");

    // Verify filters: must be included in Queued, not in Failure
    let queued_repos =
        apply_top_filters_and_sorting(&repos, CiTopFilter::Queued, CiTopSort::ProcessOrder);
    assert_eq!(
        queued_repos.len(),
        1,
        "Repo in queue must be matched by Queued filter"
    );
    assert_eq!(queued_repos[0].name, "joaju-pdf");

    let fail_repos =
        apply_top_filters_and_sorting(&repos, CiTopFilter::Failure, CiTopSort::ProcessOrder);
    assert_eq!(
        fail_repos.len(),
        0,
        "Repo in queue must not be treated as Failure"
    );
}

#[test]
fn test_running_repositories_prioritized_at_absolute_top_over_waiting_and_others() {
    let r_waiting = make_sample_repo(
        "waiting-repo",
        "org/waiting-repo",
        vec![CiPipelineStatus {
            name: "test".to_string(),
            status: "waiting".to_string(),
            description: Some("Runner Job #10 on run #1".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );

    let r_running = make_sample_repo(
        "running-repo",
        "org/running-repo",
        vec![CiPipelineStatus {
            name: "build".to_string(),
            status: "running".to_string(),
            description: Some("Runner Job #20 on run #2".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );

    let r_blocked = make_sample_repo(
        "blocked-repo",
        "org/blocked-repo",
        vec![CiPipelineStatus {
            name: "deploy".to_string(),
            status: "blocked".to_string(),
            description: Some("Blocked by required conditions".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );

    let r_fail = make_sample_repo(
        "fail-repo",
        "org/fail-repo",
        vec![CiPipelineStatus {
            name: "check".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );

    let r_ok = make_sample_repo(
        "ok-repo",
        "org/ok-repo",
        vec![CiPipelineStatus {
            name: "lint".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    );

    let list = vec![
        r_ok.clone(),
        r_fail.clone(),
        r_blocked.clone(),
        r_waiting.clone(),
        r_running.clone(),
    ];

    let sorted = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::CiStatus);

    assert_eq!(
        sorted[0].name, "running-repo",
        "Running repos must be at the absolute top"
    );
    assert_eq!(
        sorted[1].name, "waiting-repo",
        "Waiting repos come immediately after running"
    );
    assert_eq!(sorted[2].name, "blocked-repo");
    assert_eq!(sorted[3].name, "fail-repo");
    assert_eq!(sorted[4].name, "ok-repo");
}

#[test]
fn test_mutation_repo_ci_priority_edge_cases_and_precedence() {
    use cccp::repo_top::repo_ci_priority;

    // 1. Running trumps everything (failure, waiting, ok)
    let p_running_mix = vec![
        CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "build".to_string(),
            status: "running".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ];
    assert_eq!(repo_ci_priority(&p_running_mix), 0);

    // 2. Waiting trumps failure
    let p_waiting_mix = vec![
        CiPipelineStatus {
            name: "test".to_string(),
            status: "failure".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "build".to_string(),
            status: "waiting".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
    ];
    assert_eq!(repo_ci_priority(&p_waiting_mix), 1);

    // 3. Blocked trumps failure
    let p_blocked = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "blocked".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(repo_ci_priority(&p_blocked), 2);

    // 4. Failure priority
    let p_fail = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "failure".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(repo_ci_priority(&p_fail), 3);

    // 5. Ok priority
    let p_ok = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "success".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(repo_ci_priority(&p_ok), 4);

    // 6. Empty priority
    assert_eq!(repo_ci_priority(&[]), 5);
}

#[test]
fn test_format_ci_elapsed_time_and_iso_parsing() {
    use cccp::repo_top::format_ci_elapsed_time;

    // Fixed base time: 2026-11-02T05:00:00Z -> epoch 1793595600
    let base_epoch = 1793595600u64;

    // 1. Active running job with Z timestamp 14 minutes ago
    let ci_running = vec![CiPipelineStatus {
        name: "build".to_string(),
        status: "running".to_string(),
        description: Some("Has started running".to_string()),
        target_url: None,
        updated_at: Some("2026-11-02T05:00:00Z".to_string()),
    }];
    let now_14m = base_epoch + 14 * 60;
    assert_eq!(format_ci_elapsed_time(&ci_running, now_14m), "14m");

    // 2. Active waiting job with timezone offset (-03:00) 45 seconds ago
    // 2026-11-02T02:00:00-03:00 == 2026-11-02T05:00:00Z
    let ci_waiting = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "waiting".to_string(),
        description: Some("Waiting to run".to_string()),
        target_url: None,
        updated_at: Some("2026-11-02T02:00:00-03:00".to_string()),
    }];
    let now_45s = base_epoch + 45;
    assert_eq!(format_ci_elapsed_time(&ci_waiting, now_45s), "45s");

    // 3. Active job 1h15m ago
    let now_1h15m = base_epoch + 75 * 60;
    assert_eq!(format_ci_elapsed_time(&ci_waiting, now_1h15m), "1h15m");

    // 4. Completed job with "Successful in 14m21s"
    let ci_done = vec![CiPipelineStatus {
        name: "release".to_string(),
        status: "success".to_string(),
        description: Some("Successful in 14m21s".to_string()),
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(format_ci_elapsed_time(&ci_done, now_14m), "14m21s");

    // 5. Empty or missing
    assert_eq!(format_ci_elapsed_time(&[], now_14m), "-");
}

#[test]
fn test_format_ci_elapsed_time_resilient_to_missing_first_timestamp() {
    use cccp::repo_top::format_ci_elapsed_time;

    let now = 1793620800; // 2026-11-02 12:00:00 UTC
    let ci = vec![
        CiPipelineStatus {
            name: "Checks, Tests & Quality Gates".to_string(),
            status: "running".to_string(),
            description: Some("Runner Job #9594 on run #40".to_string()),
            target_url: None,
            updated_at: None, // Missing timestamp on first item
        },
        CiPipelineStatus {
            name: "CI/CD & Sovereign Release to Banca / Checks, Tests & Quality Gates (push)"
                .to_string(),
            status: "running".to_string(),
            description: Some("Has started running".to_string()),
            target_url: None,
            updated_at: Some("2026-11-02T11:45:00Z".to_string()), // 15m ago
        },
    ];

    let result = format_ci_elapsed_time(&ci, now);
    assert_eq!(result, "15m");
}

#[test]
fn test_mutation_pipeline_matches_job_name_ci_cd_slashes_and_time_resilience() {
    use cccp::forges::pipeline_matches_job_name;
    use cccp::repo_top::format_ci_elapsed_time;

    assert!(pipeline_matches_job_name(
        "CI/CD Pipeline / Build, Test & Release / Checks (push)",
        "Checks"
    ));
    assert!(pipeline_matches_job_name("CI/CD / Lint", "Lint"));
    assert!(!pipeline_matches_job_name("CI/CD / Lint", "Build"));

    let now = 1793620800;
    let ci = vec![
        CiPipelineStatus {
            name: "Job A".to_string(),
            status: "waiting".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "Job B".to_string(),
            status: "waiting".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        },
        CiPipelineStatus {
            name: "Job C".to_string(),
            status: "waiting".to_string(),
            description: None,
            target_url: None,
            updated_at: Some("2026-11-02T11:50:00Z".to_string()), // 10m
        },
    ];
    assert_eq!(format_ci_elapsed_time(&ci, now), "10m");
}

#[test]
fn test_render_top_view_includes_tempo_column() {
    let repo = make_sample_repo(
        "mandu",
        "lumen/mandu",
        vec![CiPipelineStatus {
            name: "build".to_string(),
            status: "running".to_string(),
            description: Some("Runner Job #82 on run #1".to_string()),
            target_url: None,
            updated_at: Some("2026-11-02T05:00:00Z".to_string()),
        }],
    );

    let theme = Theme::with_color(false);
    let state = RepoTopState::new(10);
    let rendered = render_top_view(&[repo], &state, &theme, 80, 24);

    assert!(
        rendered.contains("TEMPO"),
        "Top view column headers must include TEMPO"
    );
}

#[test]
fn test_mutation_iso_parsing_and_duration_concise_boundary_cases() {
    use cccp::repo_top::{format_duration_concise, parse_iso8601_to_epoch};

    // 1. Boundary duration formatting
    assert_eq!(format_duration_concise(0), "0s");
    assert_eq!(format_duration_concise(59), "59s");
    assert_eq!(format_duration_concise(60), "1m");
    assert_eq!(format_duration_concise(3599), "59m");
    assert_eq!(format_duration_concise(3600), "1h00m");
    assert_eq!(format_duration_concise(3665), "1h01m");
    assert_eq!(format_duration_concise(86399), "23h59m");
    assert_eq!(format_duration_concise(86400), "1d0h");
    assert_eq!(format_duration_concise(90000), "1d1h");

    // 2. ISO-8601 parsing mutations
    assert!(parse_iso8601_to_epoch("").is_none());
    assert!(parse_iso8601_to_epoch("not-a-timestamp").is_none());
    assert!(parse_iso8601_to_epoch("2026-13-01T00:00:00Z").is_none()); // invalid month
    assert!(parse_iso8601_to_epoch("2026-00-01T00:00:00Z").is_none());
    assert!(parse_iso8601_to_epoch("2026-11-32T00:00:00Z").is_none()); // invalid day

    // 3. Fractional seconds with timezone
    let with_millis = parse_iso8601_to_epoch("2026-11-02T05:00:00.123Z");
    let without_millis = parse_iso8601_to_epoch("2026-11-02T05:00:00Z");
    assert_eq!(with_millis, without_millis);

    // 4. Positive and negative timezone offsets
    // UTC 05:00 == +02:00 at 07:00
    let with_pos_tz = parse_iso8601_to_epoch("2026-11-02T07:00:00+02:00");
    assert_eq!(with_pos_tz, without_millis);
}

#[test]
fn test_auto_sync_catalog_ci_status_in_memory_only() {
    let mut cat = vec![cccp::catalog::CatalogEntry {
        name: "test-repo".to_string(),
        forge: "gambi".to_string(),
        slug: "lumen/test-repo".to_string(),
        aliases: vec![],
        description: "Test".to_string(),
        topics: vec![],
        license: "MIT".to_string(),
        website: "".to_string(),
        is_upstream: false,
        is_mirror: false,
        open_tickets: None,
        ci_status: vec![],
        canonical_url: None,
        server_host: None,
    }];

    let summaries = vec![cccp::catalog::CatalogRepoSummary {
        name: "test-repo".to_string(),
        slug: "lumen/test-repo".to_string(),
        forge: "gambi".to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        is_mirror: false,
        ci_status: vec![cccp::catalog::CiPipelineStatus {
            name: "Build & Test".to_string(),
            status: "success".to_string(),
            description: Some("Passed".to_string()),
            target_url: None,
            updated_at: Some("2026-11-02T19:00:00Z".to_string()),
        }],
    }];

    // Passing None for target_path must update in memory without writing to disk
    let modified = cccp::repo_top::auto_sync_catalog_ci_status(&mut cat, &summaries, None);

    assert!(modified);
    assert_eq!(cat[0].ci_status.len(), 1);
    assert_eq!(cat[0].ci_status[0].status, "success");
    assert_eq!(cat[0].ci_status[0].name, "Build & Test");
}

#[test]
fn test_auto_sync_catalog_ci_status_persists_to_isolated_file() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let target_file = tmp_dir.path().join("catalog.json");

    let mut cat = vec![cccp::catalog::CatalogEntry {
        name: "isolated-repo".to_string(),
        forge: "gambi".to_string(),
        slug: "lumen/isolated-repo".to_string(),
        aliases: vec![],
        description: "Isolated".to_string(),
        topics: vec![],
        license: "MIT".to_string(),
        website: "".to_string(),
        is_upstream: false,
        is_mirror: false,
        open_tickets: None,
        ci_status: vec![],
        canonical_url: None,
        server_host: None,
    }];

    let summaries = vec![cccp::catalog::CatalogRepoSummary {
        name: "isolated-repo".to_string(),
        slug: "lumen/isolated-repo".to_string(),
        forge: "gambi".to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        is_mirror: false,
        ci_status: vec![cccp::catalog::CiPipelineStatus {
            name: "CI Check".to_string(),
            status: "success".to_string(),
            description: Some("Passed".to_string()),
            target_url: None,
            updated_at: Some("2026-11-02T20:00:00Z".to_string()),
        }],
    }];

    let modified =
        cccp::repo_top::auto_sync_catalog_ci_status(&mut cat, &summaries, Some(&target_file));

    assert!(modified);
    assert!(target_file.is_file());
    let saved_content = std::fs::read_to_string(&target_file).expect("read saved file");
    assert!(saved_content.contains("isolated-repo"));
    assert!(saved_content.contains("CI Check"));
}

#[test]
fn test_mutation_auto_sync_edge_cases_and_unmodified_state() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let target_file = tmp_dir.path().join("catalog_edge.json");

    // 1. Empty cat or summaries
    let mut empty_cat: Vec<cccp::catalog::CatalogEntry> = vec![];
    let empty_summaries: Vec<cccp::catalog::CatalogRepoSummary> = vec![];
    assert!(!cccp::repo_top::auto_sync_catalog_ci_status(
        &mut empty_cat,
        &empty_summaries,
        Some(&target_file)
    ));
    assert!(!target_file.exists());

    // 2. Unmodified state (identical ci_status)
    let ci = vec![cccp::catalog::CiPipelineStatus {
        name: "test".to_string(),
        status: "success".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    let mut cat = vec![cccp::catalog::CatalogEntry {
        name: "repo1".to_string(),
        forge: "gambi".to_string(),
        slug: "Lumen/Repo1".to_string(),
        aliases: vec![],
        description: "".to_string(),
        topics: vec![],
        license: "".to_string(),
        website: "".to_string(),
        is_upstream: false,
        is_mirror: false,
        open_tickets: None,
        ci_status: ci.clone(),
        canonical_url: None,
        server_host: None,
    }];
    let summaries = vec![cccp::catalog::CatalogRepoSummary {
        name: "repo1".to_string(),
        slug: "lumen/repo1".to_string(),
        forge: "gambi".to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: None,
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        is_mirror: false,
        ci_status: ci,
    }];
    // When statuses are already identical, returns false and does not write to disk
    assert!(!cccp::repo_top::auto_sync_catalog_ci_status(
        &mut cat,
        &summaries,
        Some(&target_file)
    ));
    assert!(!target_file.exists());
}

#[test]
fn test_repo_top_sort_recent_update() {
    let r1 = CatalogRepoSummary {
        name: "older-repo".to_string(),
        slug: "lumen/older-repo".to_string(),
        forge: "gambi".to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        is_mirror: false,
        ci_status: vec![CiPipelineStatus {
            name: "CI".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: Some("2026-11-01T10:00:00-03:00".to_string()),
        }],
    };
    let r2 = CatalogRepoSummary {
        name: "newer-repo".to_string(),
        slug: "lumen/newer-repo".to_string(),
        forge: "gambi".to_string(),
        aliases: vec![],
        ticket_count: 0,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean".to_string(),
        local_path: None,
        gost_version: None,
        is_mirror: false,
        ci_status: vec![CiPipelineStatus {
            name: "CI".to_string(),
            status: "success".to_string(),
            description: None,
            target_url: None,
            updated_at: Some("2026-11-03T20:00:00-03:00".to_string()),
        }],
    };

    let sorted =
        apply_top_filters_and_sorting(&[r1, r2], CiTopFilter::All, CiTopSort::RecentUpdate);
    assert_eq!(sorted[0].name, "newer-repo");
    assert_eq!(sorted[1].name, "older-repo");
}

#[test]
fn test_sanitize_ghost_pending_statuses() {
    let mut summaries = vec![
        make_sample_repo(
            "banca",
            "lumen/banca",
            vec![CiPipelineStatus {
                name: "CI".to_string(),
                status: "pending".to_string(),
                description: Some("Successful in 2m56s".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
        make_sample_repo(
            "navalha",
            "lumen/navalha",
            vec![CiPipelineStatus {
                name: "CI".to_string(),
                status: "pending".to_string(),
                description: Some("Failing after 34s".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
    ];

    cccp::repo_top::sanitize_pipeline_statuses(&mut summaries);

    assert_eq!(summaries[0].ci_status[0].status, "success");
    assert_eq!(summaries[1].ci_status[0].status, "failure");
}

#[test]
fn test_mutation_sanitize_pipeline_statuses_edge_cases() {
    let mut summaries = vec![
        make_sample_repo(
            "r1",
            "lumen/r1",
            vec![CiPipelineStatus {
                name: "Job".to_string(),
                status: "running".to_string(),
                description: Some("PASSED in 10s".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
        make_sample_repo(
            "r2",
            "lumen/r2",
            vec![CiPipelineStatus {
                name: "Job".to_string(),
                status: "pending".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
        ),
    ];

    cccp::repo_top::sanitize_pipeline_statuses(&mut summaries);
    // Description without "success" or "fail" leaves status unmodified
    assert_eq!(summaries[0].ci_status[0].status, "running");
    assert_eq!(summaries[1].ci_status[0].status, "pending");
}

#[test]
fn test_mutation_format_ci_summary_badges() {
    use cccp::catalog::format_ci_summary;

    let p_pass = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "success".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(format_ci_summary(&p_pass).1, "green");

    let p_fail = vec![CiPipelineStatus {
        name: "test".to_string(),
        status: "failure".to_string(),
        description: None,
        target_url: None,
        updated_at: None,
    }];
    assert_eq!(format_ci_summary(&p_fail).1, "red");
}

#[test]
fn test_sanitize_pipeline_statuses_expurges_falsely_waiting_and_blocked() {
    let mut summaries = vec![
        make_sample_repo(
            "r1",
            "lumen/r1",
            vec![CiPipelineStatus {
                name: "Job1".to_string(),
                status: "waiting".to_string(),
                description: Some("Successful in 1m54s".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
        make_sample_repo(
            "r2",
            "lumen/r2",
            vec![CiPipelineStatus {
                name: "Job2".to_string(),
                status: "waiting".to_string(),
                description: Some("Blocked by required conditions".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
        make_sample_repo(
            "r3",
            "lumen/r3",
            vec![CiPipelineStatus {
                name: "Job3".to_string(),
                status: "pending".to_string(),
                description: Some("Has been skipped".to_string()),
                target_url: None,
                updated_at: None,
            }],
        ),
    ];

    cccp::repo_top::sanitize_pipeline_statuses(&mut summaries);
    assert_eq!(summaries[0].ci_status[0].status, "success");
    assert_eq!(summaries[1].ci_status[0].status, "skipped");
    assert_eq!(summaries[2].ci_status[0].status, "skipped");
}

#[test]
fn test_apply_repo_ci_update_does_not_revive_stale_waiting_from_disk() {
    let mut repo = make_sample_repo(
        "r1",
        "lumen/r1",
        vec![CiPipelineStatus {
            name: "Build".to_string(),
            status: "waiting".to_string(),
            description: Some("Old stale waiting".to_string()),
            target_url: None,
            updated_at: None,
        }],
    );

    let fresh_ci = vec![CiPipelineStatus {
        name: "Build".to_string(),
        status: "success".to_string(),
        description: Some("Successful in 30s".to_string()),
        target_url: None,
        updated_at: None,
    }];

    cccp::repo_top::apply_repo_ci_update(&mut repo, fresh_ci);
    assert_eq!(repo.ci_status.len(), 1);
    assert_eq!(repo.ci_status[0].status, "success");
}

#[test]
fn test_mutation_sanitize_pipeline_statuses_boundary_cases() {
    // 1. Empty slice does not panic
    let mut empty: Vec<CatalogRepoSummary> = vec![];
    cccp::repo_top::sanitize_pipeline_statuses(&mut empty);
    assert!(empty.is_empty());

    // 2. Pipelines without descriptions or with already clean status
    let mut clean = vec![make_sample_repo(
        "clean-repo",
        "lumen/clean-repo",
        vec![
            CiPipelineStatus {
                name: "p1".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            },
            CiPipelineStatus {
                name: "p2".to_string(),
                status: "failure".to_string(),
                description: Some("Failed in 5s".to_string()),
                target_url: None,
                updated_at: None,
            },
        ],
    )];
    cccp::repo_top::sanitize_pipeline_statuses(&mut clean);
    assert_eq!(clean[0].ci_status[0].status, "success");
    assert_eq!(clean[0].ci_status[1].status, "failure");
}

#[test]
fn test_prioritize_forge_sync_targets_limits_and_orders_by_priority() {
    let cat = vec![
        cccp::catalog::CatalogEntry {
            name: "r1".to_string(),
            forge: "gambi".to_string(),
            slug: "lumen/r1".to_string(),
            aliases: vec![],
            description: "".to_string(),
            topics: vec![],
            license: "".to_string(),
            website: "".to_string(),
            is_upstream: false,
            is_mirror: false,
            open_tickets: None,
            ci_status: vec![],
            canonical_url: None,
            server_host: Some("gambi.2lp.in".to_string()),
        },
        cccp::catalog::CatalogEntry {
            name: "r2".to_string(),
            forge: "gambi".to_string(),
            slug: "lumen/r2".to_string(),
            aliases: vec![],
            description: "".to_string(),
            topics: vec![],
            license: "".to_string(),
            website: "".to_string(),
            is_upstream: false,
            is_mirror: false,
            open_tickets: None,
            ci_status: vec![CiPipelineStatus {
                name: "ci".to_string(),
                status: "running".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            canonical_url: None,
            server_host: Some("gambi.2lp.in".to_string()),
        },
        cccp::catalog::CatalogEntry {
            name: "r3".to_string(),
            forge: "gambi".to_string(),
            slug: "lumen/r3".to_string(),
            aliases: vec![],
            description: "".to_string(),
            topics: vec![],
            license: "".to_string(),
            website: "".to_string(),
            is_upstream: false,
            is_mirror: false,
            open_tickets: None,
            ci_status: vec![],
            canonical_url: None,
            server_host: Some("gambi.2lp.in".to_string()),
        },
    ];

    let targets = cccp::repo_top::prioritize_forge_sync_targets(&cat, 2);
    assert_eq!(targets.len(), 2);
    // r2 has active CI running, so it MUST be prioritized first
    assert_eq!(targets[0].0, "lumen/r2");
}
