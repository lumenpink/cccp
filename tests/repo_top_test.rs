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

    // CiStatus sorts by priority: Failure > Queued > Ok > None
    let res_status = apply_top_filters_and_sorting(&list, CiTopFilter::All, CiTopSort::CiStatus);
    assert_eq!(res_status[0].name, "alpha"); // Failure
    assert_eq!(res_status[1].name, "zeta"); // Ok
    assert_eq!(res_status[2].name, "mu"); // No queue / empty
}

#[test]
fn test_repo_top_state_navigation_and_help() {
    let mut state = RepoTopState::new(10);
    assert_eq!(state.filter, CiTopFilter::All);
    assert_eq!(state.sort, CiTopSort::ProcessOrder);
    assert!(!state.show_help);

    // Toggle help
    state.handle_action(RepoTopAction::ToggleHelp, 10);
    assert!(state.show_help);
    state.handle_action(RepoTopAction::ToggleHelp, 10);
    assert!(!state.show_help);

    // Cycle filter
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Ok);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Failure);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::Queued);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::NoQueue);
    state.handle_action(RepoTopAction::CycleFilter, 10);
    assert_eq!(state.filter, CiTopFilter::All);

    // Cycle sort
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::CiStatus);
    state.handle_action(RepoTopAction::CycleSort, 10);
    assert_eq!(state.sort, CiTopSort::RepoName);
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
}
