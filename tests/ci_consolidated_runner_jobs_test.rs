use cccp::catalog::{CatalogRepoSummary, CiPipelineStatus};
use cccp::forges::{extract_repo_slug_from_action_url, merge_live_runner_jobs, ForgeActionRunnerJob};

#[test]
fn test_parse_forge_action_runner_job_json() {
    let json_data = r#"[
      {
        "id": 9215,
        "run_id": 1582,
        "attempt": 1,
        "handle": "277c86ca-f12c-40f5-9fc4-89667b71f734",
        "repo_id": 21,
        "owner_id": 1,
        "name": "Checks, Tests & Quality Gates",
        "html_url": "https://gambi.2lp.in/lumen/cccp/actions/runs/26/jobs/0/attempt/1",
        "needs": null,
        "runs_on": ["ubuntu-latest"],
        "task_id": 0,
        "status": "waiting"
      },
      {
        "id": 9201,
        "run_id": 1575,
        "attempt": 1,
        "handle": "93376df5-1a38-4554-b513-3529bf17d818",
        "repo_id": 25,
        "owner_id": 1,
        "name": "Build & Push gost-ci-flutter",
        "html_url": "https://gambi.2lp.in/lumen/gost-ci/actions/runs/23/jobs/9/attempt/1",
        "needs": null,
        "runs_on": ["ubuntu-latest"],
        "task_id": 1626,
        "status": "running"
      }
    ]"#;

    let jobs: Vec<ForgeActionRunnerJob> =
        serde_json::from_str(json_data).expect("falha ao desserializar ForgeActionRunnerJob");
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].id, 9215);
    assert_eq!(jobs[0].status, "waiting");
    assert_eq!(jobs[1].id, 9201);
    assert_eq!(jobs[1].status, "running");
}

#[test]
fn test_extract_repo_slug_from_action_url() {
    let url1 = "https://gambi.2lp.in/lumen/cccp/actions/runs/26/jobs/0/attempt/1";
    assert_eq!(
        extract_repo_slug_from_action_url(url1),
        Some("lumen/cccp".to_string())
    );

    let url2 = "https://codeberg.org/forge-user/my-project/actions/runs/5/jobs/1";
    assert_eq!(
        extract_repo_slug_from_action_url(url2),
        Some("forge-user/my-project".to_string())
    );

    let invalid = "https://example.com/not-an-action";
    assert_eq!(extract_repo_slug_from_action_url(invalid), None);
}

#[test]
fn test_merge_live_runner_jobs() {
    let mut summaries = vec![
        CatalogRepoSummary {
            name: "cccp".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/cccp".to_string(),
            aliases: vec![],
            ticket_count: 1,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: None,
            gost_version: Some("2.0.0".to_string()),
            ci_status: vec![CiPipelineStatus {
                name: "old-build".to_string(),
                status: "success".to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }],
            is_mirror: false,
        },
        CatalogRepoSummary {
            name: "banca".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/banca".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(true),
            unpushed: 0,
            status_label: "clean".to_string(),
            local_path: None,
            gost_version: Some("2.0.0".to_string()),
            ci_status: vec![],
            is_mirror: false,
        },
    ];

    let live_jobs = vec![
        ForgeActionRunnerJob {
            id: 100,
            run_id: 50,
            attempt: 1,
            handle: "h1".to_string(),
            repo_id: 1,
            owner_id: 1,
            name: "CI Gate".to_string(),
            html_url: "https://gambi.2lp.in/lumen/cccp/actions/runs/50/jobs/1/attempt/1".to_string(),
            status: "running".to_string(),
        },
        ForgeActionRunnerJob {
            id: 101,
            run_id: 51,
            attempt: 1,
            handle: "h2".to_string(),
            repo_id: 2,
            owner_id: 1,
            name: "Deploy".to_string(),
            html_url: "https://gambi.2lp.in/lumen/banca/actions/runs/51/jobs/2/attempt/1".to_string(),
            status: "waiting".to_string(),
        },
    ];

    merge_live_runner_jobs(&mut summaries, &live_jobs);

    // cccp must now include running job
    let cccp = summaries.iter().find(|r| r.slug == "lumen/cccp").unwrap();
    assert!(cccp.ci_status.iter().any(|p| p.status == "running" && p.name == "CI Gate"));

    // banca must now include waiting job
    let banca = summaries.iter().find(|r| r.slug == "lumen/banca").unwrap();
    assert!(banca.ci_status.iter().any(|p| p.status == "waiting" && p.name == "Deploy"));
}
