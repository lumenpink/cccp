use cccp::forges::{host_to_token_env_var, ForgeType};

#[test]
fn test_forgejo_default_host_is_sovereign_forge() {
    assert_eq!(
        ForgeType::Forgejo.default_host(),
        "git.2lp.in",
        "ForgeType::Forgejo must default to git.2lp.in"
    );
}

#[test]
fn test_catalog_short_slug_resolves_to_sovereign_forge() {
    let info = cccp::catalog::parse_repo_identifier("lumen/nova-ferramenta")
        .expect("short slug should parse");
    assert_eq!(
        info.provider,
        ForgeType::Forgejo,
        "Short slug owner/repo must resolve to Forgejo"
    );
    assert_eq!(
        info.host, "git.2lp.in",
        "Short slug owner/repo must resolve to git.2lp.in"
    );
    assert_eq!(info.owner, "lumen");
    assert_eq!(info.repo, "nova-ferramenta");
}

#[test]
fn test_resolve_target_repo_workspace_defaults_to_sovereign_forge() {
    let res = cccp::catalog::resolve_target_repo_workspace("lumenpink/novo-projeto", None, None);
    assert!(res.is_some());
    let (info, _) = res.unwrap();
    assert_eq!(
        info.provider,
        ForgeType::Forgejo,
        "Target repo fallback must resolve to Forgejo"
    );
    assert_eq!(
        info.host, "git.2lp.in",
        "Target repo fallback must resolve to git.2lp.in"
    );
}

#[test]
fn test_token_env_var_for_sovereign_forge() {
    let var = host_to_token_env_var("git.2lp.in");
    assert_eq!(var, "GIT_2LP_IN_TOKEN");
}

#[test]
fn test_sovereign_forge_edge_cases_and_mutations() {
    // 1. Slug with .git suffix
    let info_git = cccp::catalog::parse_repo_identifier("lumen/ferramenta.git")
        .expect("short slug with .git should parse");
    assert_eq!(info_git.provider, ForgeType::Forgejo);
    assert_eq!(info_git.host, "git.2lp.in");
    assert_eq!(info_git.repo, "ferramenta");

    // 2. Explicit sovereign host uppercase
    let info_upper = cccp::catalog::parse_repo_identifier("GIT.2LP.IN/lumen/ferramenta")
        .expect("uppercase sovereign host should parse");
    assert_eq!(info_upper.provider, ForgeType::Forgejo);
    assert_eq!(info_upper.host, "GIT.2LP.IN");
}
