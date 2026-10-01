use cccp::cultural::Theme;
use cccp::deps::{cmd_deps_check, cmd_deps_list, get_catalog, get_system_pkg_mgr, version_ge};

#[test]
fn test_version_ge_strict_equality() {
    assert!(version_ge("2.25.0", "2.25.0"));
}

#[test]
fn test_version_ge_major_comparison() {
    assert!(version_ge("3.0.0", "2.25.0"));
    assert!(!version_ge("1.9.9", "2.25.0"));
}

#[test]
fn test_version_ge_minor_comparison() {
    assert!(version_ge("2.40.1", "2.25.0"));
    assert!(!version_ge("2.24.9", "2.25.0"));
}

#[test]
fn test_version_ge_patch_comparison() {
    assert!(version_ge("2.25.1", "2.25.0"));
    assert!(!version_ge("2.25.0", "2.25.1"));
}

#[test]
fn test_version_ge_strips_v_prefix() {
    assert!(version_ge("v2.30.0", "2.25.0"));
    assert!(version_ge("V2.30.0", "v2.25.0"));
}

#[test]
fn test_version_ge_development_builds() {
    assert!(version_ge("development", "0.9.0"));
    assert!(version_ge("dev.1", "0.1.0"));
    assert!(version_ge("master-build", "1.0.0"));
    assert!(version_ge("main", "2.0.0"));
}

#[test]
fn test_get_catalog_structure() {
    let catalog = get_catalog();
    assert!(catalog.len() >= 10);

    let git_dep = catalog
        .iter()
        .find(|d| d.name == "git")
        .expect("git must be in catalog");
    assert_eq!(git_dep.requirement, "Obrigatório");
    assert_eq!(git_dep.category, "core");

    let act_dep = catalog
        .iter()
        .find(|d| d.name == "act")
        .expect("act must be in catalog");
    assert_eq!(act_dep.category, "ci");

    let comrades = ["sv", "kv", "mk"];
    for c in comrades {
        let dep = catalog
            .iter()
            .find(|d| d.name == c)
            .unwrap_or_else(|| panic!("{} must be in catalog", c));
        assert_eq!(dep.category, "comrade");
    }
}

#[test]
fn test_get_system_pkg_mgr_returns_valid() {
    let pm = get_system_pkg_mgr();
    assert!(!pm.is_empty());
}

#[test]
fn test_cmd_deps_list_runs() {
    let theme = Theme::with_color(false);
    cmd_deps_list(&theme);
}

#[test]
fn test_cmd_deps_check_git() {
    let theme = Theme::with_color(false);
    let res = cmd_deps_check(&["git".to_string()], &theme);
    assert_eq!(res, Ok(true));
}
