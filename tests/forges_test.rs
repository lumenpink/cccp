use cccp::forges::{
    host_to_token_env_var, parse_issues_json, parse_prs_json, parse_remote_url,
    resolve_token_for_host, ForgeType,
};
use serde_json::json;

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
    let saved_cb = std::env::var("CODEBERG_TOKEN").ok();
    let saved_fj = std::env::var("FORGEJO_TOKEN").ok();
    let saved_gt = std::env::var("GITEA_TOKEN").ok();
    let saved_tea = std::env::var("TEA_TOKEN").ok();
    let saved_cb_org = std::env::var("CODEBERG_ORG_TOKEN").ok();
    let saved_mock = std::env::var("MOCK_HOME_ENV_ORG_TOKEN").ok();

    std::env::remove_var("CODEBERG_TOKEN");
    std::env::remove_var("FORGEJO_TOKEN");
    std::env::remove_var("GITEA_TOKEN");
    std::env::remove_var("TEA_TOKEN");
    std::env::remove_var("CODEBERG_ORG_TOKEN");
    std::env::remove_var("MOCK_HOME_ENV_ORG_TOKEN");

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
