use cccp::forges::{
    host_to_token_env_var, parse_issues_json, parse_prs_json, parse_remote_url,
    resolve_token_for_host, ForgeType,
};
use serde_json::json;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_parse_remote_url_codeberg_ssh() {
    let info = parse_remote_url("git@codeberg.org:lumenpink/cccp.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Codeberg);
    assert_eq!(info.host, "codeberg.org");
    assert_eq!(info.owner, "lumenpink");
    assert_eq!(info.repo, "cccp");
}

#[test]
fn test_parse_remote_url_codeberg_https() {
    let info = parse_remote_url("https://codeberg.org/lumenpink/cccp.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Codeberg);
    assert_eq!(info.host, "codeberg.org");
    assert_eq!(info.owner, "lumenpink");
    assert_eq!(info.repo, "cccp");
}

#[test]
fn test_parse_remote_url_ssh_protocol_with_port() {
    let info =
        parse_remote_url("ssh://git@codeberg.org:2222/lumenpink/cccp.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Codeberg);
    assert_eq!(info.host, "codeberg.org");
    assert_eq!(info.owner, "lumenpink");
    assert_eq!(info.repo, "cccp");
}

#[test]
fn test_parse_remote_url_github() {
    let info = parse_remote_url("git@github.com:rust-lang/rust.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::GitHub);
    assert_eq!(info.host, "github.com");
    assert_eq!(info.owner, "rust-lang");
    assert_eq!(info.repo, "rust");

    let https_info = parse_remote_url("https://github.com/rust-lang/cargo").expect("parse failed");
    assert_eq!(https_info.provider, ForgeType::GitHub);
    assert_eq!(https_info.owner, "rust-lang");
    assert_eq!(https_info.repo, "cargo");
}

#[test]
fn test_parse_remote_url_gitlab() {
    let info =
        parse_remote_url("https://gitlab.com/gitlab-org/gitlab-runner.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::GitLab);
    assert_eq!(info.host, "gitlab.com");
    assert_eq!(info.owner, "gitlab-org");
    assert_eq!(info.repo, "gitlab-runner");
}

#[test]
fn test_parse_remote_url_self_hosted_forgejo() {
    let info =
        parse_remote_url("https://git.internal.lan/devs/secret-service.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Forgejo);
    assert_eq!(info.host, "git.internal.lan");
    assert_eq!(info.owner, "devs");
    assert_eq!(info.repo, "secret-service");
}

#[test]
fn test_parse_remote_url_tildegit_ssh() {
    let info = parse_remote_url("git@tildegit.org:lumen/install_i3.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Tildegit);
    assert_eq!(info.software, cccp::forges::ForgeSoftware::Forgejo);
    assert_eq!(info.host, "tildegit.org");
    assert_eq!(info.owner, "lumen");
    assert_eq!(info.repo, "install_i3");
    assert_eq!(info.forge_name(), "tildegit");
    assert_eq!(info.software_name(), "forgejo");
}

#[test]
fn test_parse_remote_url_tildegit_https() {
    let info = parse_remote_url("https://tildegit.org/lumen/install_i3.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Tildegit);
    assert_eq!(info.software, cccp::forges::ForgeSoftware::Forgejo);
    assert_eq!(info.host, "tildegit.org");
    assert_eq!(info.owner, "lumen");
    assert_eq!(info.repo, "install_i3");
}

#[test]
fn test_parse_remote_url_tildegit_subdomain_and_port() {
    let info = parse_remote_url("ssh://git@git.tildegit.org:2222/lumen/install_i3.git")
        .expect("parse failed");
    assert_eq!(info.provider, ForgeType::Tildegit);
    assert_eq!(info.software, cccp::forges::ForgeSoftware::Forgejo);
    assert_eq!(info.host, "git.tildegit.org");
    assert_eq!(info.owner, "lumen");
    assert_eq!(info.repo, "install_i3");
    assert_eq!(info.forge_name(), "tildegit");
    assert_eq!(info.software_name(), "forgejo");
}

#[test]
fn test_parse_remote_url_tildegit_case_insensitive() {
    let info = parse_remote_url("HTTPS://TILDEGIT.ORG/Lumen/Install_i3.git").expect("parse failed");
    assert_eq!(info.provider, ForgeType::Tildegit);
    assert_eq!(info.software, cccp::forges::ForgeSoftware::Forgejo);
    assert_eq!(info.owner, "Lumen");
    assert_eq!(info.repo, "Install_i3");
}

#[test]
fn test_resolve_token_for_tildegit_domain_precedence() {
    let _lock = ENV_LOCK.lock().unwrap();
    let empty_cfg = cccp::config::Config::default();
    std::env::set_var("TILDEGIT_ORG_TOKEN", "domain_tok_111");
    std::env::set_var("TILDEGIT_TOKEN", "generic_tok_222");
    assert_eq!(
        resolve_token_for_host("tildegit.org", ForgeType::Tildegit, Some(&empty_cfg)),
        Some("domain_tok_111".into())
    );
    std::env::remove_var("TILDEGIT_ORG_TOKEN");
    assert_eq!(
        resolve_token_for_host("tildegit.org", ForgeType::Tildegit, Some(&empty_cfg)),
        Some("generic_tok_222".into())
    );
    std::env::remove_var("TILDEGIT_TOKEN");
}

#[test]
fn test_parse_remote_url_invalid() {
    assert!(parse_remote_url("not_a_valid_url").is_none());
    assert!(parse_remote_url("").is_none());
    assert!(parse_remote_url("https://github.com/").is_none());
}

#[test]
fn test_resolve_token_environment() {
    let _lock = ENV_LOCK.lock().unwrap();
    let empty_cfg = cccp::config::Config::default();
    std::env::set_var("GITHUB_TOKEN", "gh_secret_token_456");
    assert_eq!(
        resolve_token_for_host("mock-github.com", ForgeType::GitHub, Some(&empty_cfg)),
        Some("gh_secret_token_456".into())
    );

    std::env::set_var("GITLAB_TOKEN", "gl_secret_token_789");
    assert_eq!(
        resolve_token_for_host("mock-gitlab.com", ForgeType::GitLab, Some(&empty_cfg)),
        Some("gl_secret_token_789".into())
    );

    std::env::set_var("TILDEGIT_TOKEN", "tg_secret_token_321");
    assert_eq!(
        resolve_token_for_host("mock-tildegit.org", ForgeType::Tildegit, Some(&empty_cfg)),
        Some("tg_secret_token_321".into())
    );

    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("GITLAB_TOKEN");
    std::env::remove_var("TILDEGIT_TOKEN");
}

#[test]
fn test_resolve_token_reads_home_env_when_config_none() {
    let _lock = ENV_LOCK.lock().unwrap();
    let saved_cb = std::env::var("CODEBERG_TOKEN").ok();
    let saved_fj = std::env::var("FORGEJO_TOKEN").ok();
    let saved_gt = std::env::var("GITEA_TOKEN").ok();
    let saved_tea = std::env::var("TEA_TOKEN").ok();
    let saved_cb_org = std::env::var("CODEBERG_ORG_TOKEN").ok();
    let saved_mock = std::env::var("MOCK_HOME_ENV_ORG_TOKEN").ok();
    let saved_g2lp = std::env::var("GIT_2LP_IN_TOKEN").ok();

    std::env::remove_var("CODEBERG_TOKEN");
    std::env::remove_var("FORGEJO_TOKEN");
    std::env::remove_var("GITEA_TOKEN");
    std::env::remove_var("TEA_TOKEN");
    std::env::remove_var("CODEBERG_ORG_TOKEN");
    std::env::remove_var("MOCK_HOME_ENV_ORG_TOKEN");
    std::env::remove_var("GIT_2LP_IN_TOKEN");

    let mut config = cccp::config::Config::default();
    config
        .extra
        .insert("codeberg_token".to_string(), "ci_test_token".to_string());
    let token = resolve_token_for_host("mock-home-env.org", ForgeType::Codeberg, Some(&config));

    if let Some(v) = saved_cb {
        std::env::set_var("CODEBERG_TOKEN", v);
    }
    if let Some(v) = saved_fj {
        std::env::set_var("FORGEJO_TOKEN", v);
    }
    if let Some(v) = saved_gt {
        std::env::set_var("GITEA_TOKEN", v);
    }
    if let Some(v) = saved_tea {
        std::env::set_var("TEA_TOKEN", v);
    }
    if let Some(v) = saved_cb_org {
        std::env::set_var("CODEBERG_ORG_TOKEN", v);
    }
    if let Some(v) = saved_mock {
        std::env::set_var("MOCK_HOME_ENV_ORG_TOKEN", v);
    }
    if let Some(v) = saved_g2lp {
        std::env::set_var("GIT_2LP_IN_TOKEN", v);
    }

    assert_eq!(token.as_deref(), Some("ci_test_token"));
}

#[test]
fn test_parse_issues_json_forgejo() {
    let json_data = json!([
        {
            "number": 83,
            "title": "[rust] feat(forges): HTTP client",
            "state": "open",
            "user": {
                "login": "lumenpink",
                "username": "lumenpink"
            },
            "labels": [
                { "name": "rust" },
                { "name": "feature" }
            ],
            "body": "Issue description here",
            "html_url": "https://codeberg.org/lumenpink/cccp/issues/83",
            "created_at": "2026-09-30T09:00:00Z"
        }
    ]);

    let issues = parse_issues_json(&json_data, ForgeType::Codeberg).expect("parse failed");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].number, 83);
    assert_eq!(issues[0].title, "[rust] feat(forges): HTTP client");
    assert_eq!(issues[0].author, "lumenpink");
    assert_eq!(issues[0].labels, vec!["rust", "feature"]);
    assert_eq!(issues[0].body, "Issue description here");
}

#[test]
fn test_parse_issues_json_github_filters_pull_requests() {
    let json_data = json!([
        {
            "number": 10,
            "title": "Legitimate Issue",
            "state": "open",
            "user": { "login": "octocat" },
            "labels": [{ "name": "bug" }],
            "body": "Bug report",
            "html_url": "https://github.com/octocat/repo/issues/10",
            "pull_request": null
        },
        {
            "number": 11,
            "title": "PR in disguise",
            "state": "open",
            "user": { "login": "octocat" },
            "labels": [],
            "body": "PR body",
            "html_url": "https://github.com/octocat/repo/pull/11",
            "pull_request": {
                "url": "https://api.github.com/repos/octocat/repo/pulls/11"
            }
        }
    ]);

    let issues = parse_issues_json(&json_data, ForgeType::GitHub).expect("parse failed");
    // Only item 10 should be parsed; item 11 (PR) must be ignored
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].number, 10);
    assert_eq!(issues[0].title, "Legitimate Issue");
}

#[test]
fn test_parse_prs_json_unified() {
    let json_data = json!([
        {
            "number": 42,
            "title": "Add Stakhanovite worker mode",
            "state": "open",
            "user": { "login": "stakhanov" },
            "head": { "ref": "feat/stakhanov" },
            "base": { "ref": "main" },
            "html_url": "https://codeberg.org/lumenpink/cccp/pulls/42"
        }
    ]);

    let prs = parse_prs_json(&json_data, ForgeType::Codeberg).expect("parse failed");
    assert_eq!(prs.len(), 1);
    assert_eq!(prs[0].number, 42);
    assert_eq!(prs[0].author, "stakhanov");
    assert_eq!(prs[0].source_branch, "feat/stakhanov");
    assert_eq!(prs[0].target_branch, "main");
}

#[test]
fn test_host_to_token_env_var() {
    assert_eq!(host_to_token_env_var("codeberg.org"), "CODEBERG_ORG_TOKEN");
    assert_eq!(host_to_token_env_var("tildegit.org"), "TILDEGIT_ORG_TOKEN");
    assert_eq!(host_to_token_env_var("github.com"), "GITHUB_COM_TOKEN");
    assert_eq!(
        host_to_token_env_var("git.my-domain.net"),
        "GIT_MY_DOMAIN_NET_TOKEN"
    );
}

#[test]
fn test_resolve_token_for_host_domain_precedence() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("TEST_FORGE_NET_TOKEN", "domain_secret_123");
    let resolved = resolve_token_for_host("test.forge.net", ForgeType::Codeberg, None);
    assert_eq!(resolved.as_deref(), Some("domain_secret_123"));
    std::env::remove_var("TEST_FORGE_NET_TOKEN");
}

#[test]
fn test_resolve_token_for_host_from_config_extra() {
    let mut config = cccp::config::Config::default();
    config.extra.insert(
        "custom_myforge_org_token".to_string(),
        "my_cb_token_abc".to_string(),
    );

    let resolved = resolve_token_for_host("custom.myforge.org", ForgeType::Codeberg, Some(&config));
    assert_eq!(resolved.as_deref(), Some("my_cb_token_abc"));
}

#[test]
fn test_close_issue_missing_token_returns_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    let saved_cb = std::env::var("CODEBERG_TOKEN").ok();
    let saved_fj = std::env::var("FORGEJO_TOKEN").ok();
    let saved_gt = std::env::var("GITEA_TOKEN").ok();
    let saved_tea = std::env::var("TEA_TOKEN").ok();
    let saved_cb_org = std::env::var("CODEBERG_ORG_TOKEN").ok();
    std::env::remove_var("CODEBERG_TOKEN");
    std::env::remove_var("FORGEJO_TOKEN");
    std::env::remove_var("GITEA_TOKEN");
    std::env::remove_var("TEA_TOKEN");
    std::env::remove_var("CODEBERG_ORG_TOKEN");

    let client = cccp::forges::ForgeClient::new();
    let info = cccp::forges::RepoForgeInfo::new(
        ForgeType::GitLab,
        "unknown-forge-domain.example".to_string(),
        "testowner".to_string(),
        "testrepo".to_string(),
    );

    let result = client.close_issue(&info, 133, Some("Closing note"));

    if let Some(v) = saved_cb {
        std::env::set_var("CODEBERG_TOKEN", v);
    }
    if let Some(v) = saved_fj {
        std::env::set_var("FORGEJO_TOKEN", v);
    }
    if let Some(v) = saved_gt {
        std::env::set_var("GITEA_TOKEN", v);
    }
    if let Some(v) = saved_tea {
        std::env::set_var("TEA_TOKEN", v);
    }
    if let Some(v) = saved_cb_org {
        std::env::set_var("CODEBERG_ORG_TOKEN", v);
    }

    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("No authentication token found"));
}

#[test]
fn test_resolve_token_from_tea_config_yaml() {
    let sample_yaml = r#"
logins:
    - name: codeberg
      url: https://codeberg.org
      token: token_cb_123
      ssh_host: codeberg.org
    - name: gambi
      url: https://gambi.2lp.in
      token: token_gambi_456
      ssh_host: gambi.2lp.in
"#;
    let tok = cccp::forges::parse_tea_token_from_yaml(sample_yaml, "gambi.2lp.in");
    assert_eq!(tok.as_deref(), Some("token_gambi_456"));

    let tok_name = cccp::forges::parse_tea_token_from_yaml(sample_yaml, "codeberg");
    assert_eq!(tok_name.as_deref(), Some("token_cb_123"));

    let tok_missing = cccp::forges::parse_tea_token_from_yaml(sample_yaml, "nonexistent.host");
    assert_eq!(tok_missing, None);

    // Edge cases: trailing slashes, quotes and empty content
    let yaml_edge = r#"
logins:
    - name: 'quoted_forge'
      url: 'https://quoted.example.org/'
      token: "secret_quoted_token"
      ssh_host: "quoted.example.org"
"#;
    let tok_quoted = cccp::forges::parse_tea_token_from_yaml(yaml_edge, "quoted.example.org");
    assert_eq!(tok_quoted.as_deref(), Some("secret_quoted_token"));
    assert_eq!(
        cccp::forges::parse_tea_token_from_yaml("", "anything"),
        None
    );
}

#[test]
fn test_parse_consolidated_commit_status_json() {
    let raw_json = serde_json::json!({
        "state": "failure",
        "sha": "49619afac2a19bcd7fee2eb58cb8f3977a8fe97d",
        "total_count": 2,
        "statuses": [
            {
                "id": 7,
                "status": "failure",
                "target_url": "/lumen/lacraia/actions/runs/28/jobs/0",
                "description": "Failing after 3m14s",
                "context": "CI/CD & Canonical Release to Banca / Checks, Tests & Quality Gates (push)",
                "created_at": "2026-11-02T11:29:08-03:00",
                "updated_at": "2026-11-02T11:29:08-03:00"
            },
            {
                "id": 19,
                "status": "skipped",
                "target_url": "/lumen/lacraia/actions/runs/28/jobs/3",
                "description": "Has been skipped",
                "context": "CI/CD & Canonical Release to Banca / Trigger Pratilera Store Update in Banca (push)",
                "created_at": "2026-11-02T11:30:06-03:00",
                "updated_at": "2026-11-02T11:30:06-03:00"
            }
        ]
    });

    let pipelines = cccp::forges::parse_ci_status_json(&raw_json);
    assert_eq!(pipelines.len(), 2);
    assert_eq!(pipelines[0].status, "failure");
    assert_eq!(
        pipelines[0].description.as_deref(),
        Some("Failing after 3m14s")
    );
    assert_eq!(
        pipelines[0].updated_at.as_deref(),
        Some("2026-11-02T11:29:08-03:00")
    );
    assert_eq!(pipelines[1].status, "skipped");
    assert_eq!(
        pipelines[1].updated_at.as_deref(),
        Some("2026-11-02T11:30:06-03:00")
    );
}

#[test]
fn test_merge_live_runner_jobs_preserves_timestamp() {
    let mut summaries = vec![cccp::catalog::CatalogRepoSummary {
        name: "mandu".to_string(),
        slug: "lumen/mandu".to_string(),
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
            name: "CI/CD & Canonical Release to Banca / Checks, Tests & Quality Gates (push)"
                .to_string(),
            status: "waiting".to_string(),
            description: Some("Waiting to run".to_string()),
            target_url: None,
            updated_at: Some("2026-11-02T17:46:48-03:00".to_string()),
        }],
    }];

    let live_jobs = vec![cccp::forges::ForgeActionRunnerJob {
        id: 9682,
        run_id: 1752,
        attempt: 1,
        handle: "5d05b838-f222-4fd6-96bc-c0fb4ee5ae5e".to_string(),
        repo_id: 36,
        owner_id: 1,
        name: "Checks, Tests & Quality Gates".to_string(),
        html_url: "https://gambi.2lp.in/lumen/mandu/actions/runs/86/jobs/0/attempt/1".to_string(),
        status: "running".to_string(),
    }];

    cccp::forges::merge_live_runner_jobs(&mut summaries, &live_jobs);

    assert_eq!(summaries[0].ci_status.len(), 1);
    assert_eq!(summaries[0].ci_status[0].status, "running");
    assert_eq!(
        summaries[0].ci_status[0].updated_at.as_deref(),
        Some("2026-11-02T17:46:48-03:00")
    );
}

#[test]
fn test_mutation_parse_ci_status_json_edge_cases() {
    // 1. Empty object
    let empty_obj = serde_json::json!({});
    assert!(cccp::forges::parse_ci_status_json(&empty_obj).is_empty());

    // 2. Empty array
    let empty_arr = serde_json::json!([]);
    assert!(cccp::forges::parse_ci_status_json(&empty_arr).is_empty());

    // 3. Flat legacy array with default fallback to "CI"
    let legacy_arr = serde_json::json!([
        {
            "status": "success",
            "description": "Passed all tests",
            "created_at": "2026-11-02T10:00:00Z"
        }
    ]);
    let res = cccp::forges::parse_ci_status_json(&legacy_arr);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "CI");
    assert_eq!(res[0].status, "success");
    assert_eq!(res[0].updated_at.as_deref(), Some("2026-11-02T10:00:00Z"));

    // 4. Duplicate context names are deduplicated preserving the first occurrence
    let dup_json = serde_json::json!({
        "state": "pending",
        "statuses": [
            {
                "context": "build",
                "status": "success",
                "description": "First build"
            },
            {
                "context": "build",
                "status": "pending",
                "description": "Stale build"
            }
        ]
    });
    let dedup = cccp::forges::parse_ci_status_json(&dup_json);
    assert_eq!(dedup.len(), 1);
    assert_eq!(dedup[0].status, "success");
    assert_eq!(dedup[0].description.as_deref(), Some("First build"));
}

#[test]
fn test_forge_name_returns_instance_host_prefix_for_forgejo() {
    let info = cccp::forges::RepoForgeInfo::new(
        cccp::forges::ForgeType::Forgejo,
        "gambi.2lp.in".to_string(),
        "lumen".to_string(),
        "lacraia".to_string(),
    );
    assert_eq!(info.forge_name(), "gambi");

    let entry = cccp::catalog::CatalogEntry {
        name: "lacraia".to_string(),
        forge: "forgejo".to_string(),
        slug: "lumen/lacraia".to_string(),
        aliases: vec![],
        description: "Test".to_string(),
        topics: vec![],
        license: "MIT".to_string(),
        website: "".to_string(),
        is_upstream: false,
        is_mirror: false,
        open_tickets: None,
        ci_status: vec![],
        canonical_url: Some("https://gambi.2lp.in/lumen/lacraia".to_string()),
        server_host: Some("gambi.2lp.in".to_string()),
    };

    let summaries = cccp::catalog::summarize_catalog(&[entry]);
    assert_eq!(summaries[0].forge, "gambi");
}

#[test]
fn test_mutation_forge_name_edge_cases() {
    // 1. Host with git. prefix
    let info_git = cccp::forges::RepoForgeInfo::new(
        cccp::forges::ForgeType::Forgejo,
        "git.2lp.in".to_string(),
        "lumen".to_string(),
        "repo".to_string(),
    );
    assert_eq!(info_git.forge_name(), "2lp");

    // 2. Host with port
    let info_port = cccp::forges::RepoForgeInfo::new(
        cccp::forges::ForgeType::Forgejo,
        "gambi.2lp.in:8443".to_string(),
        "lumen".to_string(),
        "repo".to_string(),
    );
    assert_eq!(info_port.forge_name(), "gambi");

    // 3. Host with no dot
    let info_local = cccp::forges::RepoForgeInfo::new(
        cccp::forges::ForgeType::Forgejo,
        "localhost".to_string(),
        "lumen".to_string(),
        "repo".to_string(),
    );
    assert_eq!(info_local.forge_name(), "localhost");

    // 4. Summarize catalog with canonical_url fallback
    let entry_url = cccp::catalog::CatalogEntry {
        name: "test-repo".to_string(),
        forge: "forgejo".to_string(),
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
        canonical_url: Some("https://git.internal.org/lumen/test-repo".to_string()),
        server_host: None,
    };
    let sum_url = cccp::catalog::summarize_catalog(&[entry_url]);
    assert_eq!(sum_url[0].forge, "internal");
}

#[test]
fn test_merge_live_runner_jobs_reconciles_stale_running_ghosts() {
    let mut summaries = vec![
        cccp::catalog::CatalogRepoSummary {
            name: "lacraia".to_string(),
            slug: "lumen/lacraia".to_string(),
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
                name: "CI/CD & Sovereign Release to Banca / Trigger Pratilera (push)".to_string(),
                status: "running".to_string(),
                description: Some("Has started running".to_string()),
                target_url: None,
                updated_at: Some("2026-11-02T11:30:06-03:00".to_string()),
            }],
        },
        cccp::catalog::CatalogRepoSummary {
            name: "gost-ci".to_string(),
            slug: "lumen/gost-ci".to_string(),
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
                name: "Build & Push gost-ci-deploy".to_string(),
                status: "success".to_string(),
                description: Some("Successful".to_string()),
                target_url: None,
                updated_at: Some("2026-11-02T07:52:10-03:00".to_string()),
            }],
        },
    ];

    let live_jobs = vec![cccp::forges::ForgeActionRunnerJob {
        id: 9671,
        run_id: 1751,
        attempt: 1,
        handle: "8ede3783-f41c-4ac0-afe4-1e9fc6d31e2f".to_string(),
        repo_id: 25,
        owner_id: 1,
        name: "Build & Push gost-ci-deploy".to_string(),
        html_url: "https://gambi.2lp.in/lumen/gost-ci/actions/runs/32/jobs/2/attempt/1".to_string(),
        status: "running".to_string(),
    }];

    cccp::forges::merge_live_runner_jobs(&mut summaries, &live_jobs);

    // gost-ci must be running
    assert_eq!(summaries[1].ci_status[0].status, "running");

    // lacraia had a stale running status on disk, but has zero running jobs on runners
    // It must be reconciled away from "running"
    assert_ne!(
        summaries[0].ci_status[0].status, "running",
        "Stale running status on lacraia must be reconciled away from running"
    );
}

#[test]
fn test_mutation_reconcile_when_all_jobs_waiting() {
    let mut summaries = vec![cccp::catalog::CatalogRepoSummary {
        name: "mandu".to_string(),
        slug: "lumen/mandu".to_string(),
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
            name: "Trigger".to_string(),
            status: "running".to_string(),
            description: None,
            target_url: None,
            updated_at: None,
        }],
    }];

    // Live jobs contain only waiting jobs
    let live_jobs = vec![cccp::forges::ForgeActionRunnerJob {
        id: 1,
        run_id: 1,
        attempt: 1,
        handle: "h".to_string(),
        repo_id: 36,
        owner_id: 1,
        name: "Checks".to_string(),
        html_url: "https://gambi.2lp.in/lumen/mandu/actions/runs/1/jobs/0/attempt/1".to_string(),
        status: "waiting".to_string(),
    }];

    cccp::forges::merge_live_runner_jobs(&mut summaries, &live_jobs);

    // The stale running job "Trigger" must have been downgraded from "running"
    assert_ne!(summaries[0].ci_status[0].status, "running");
    let has_any_running = summaries[0]
        .ci_status
        .iter()
        .any(|p| p.status.eq_ignore_ascii_case("running"));
    assert!(!has_any_running);
}
