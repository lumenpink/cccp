//! Integration tests for remote and upstream repository isolation against local mutations (Ticket #66)

use cccp::catalog::{assemble_catalog_repo, batch_assemble_gost, CatalogEntry};
use cccp::cultural::Theme;

#[test]
fn test_assemble_blocked_for_upstream_repo() {
    let theme = Theme::with_color(false);

    let upstream_entry = CatalogEntry {
        name: "Upstream Library".to_string(),
        slug: "upstream-lib".to_string(),
        is_upstream: true,
        ..Default::default()
    };

    let entries = vec![upstream_entry];
    let res = assemble_catalog_repo(&entries, "upstream-lib", false, false, None, &theme);
    assert!(
        res.is_err(),
        "Upstream repository must be isolated against assemble"
    );
    let err = res.unwrap_err();
    let lower = err.to_lowercase();
    assert!(
        lower.contains("upstream") && (lower.contains("isolado") || lower.contains("protegido")),
        "Error message must explain upstream isolation: {}",
        err
    );
}

#[test]
fn test_batch_assemble_skips_upstream_repos() {
    let theme = Theme::with_color(false);

    let upstream_entry = CatalogEntry {
        name: "Upstream Batch".to_string(),
        slug: "upstream-batch".to_string(),
        is_upstream: true,
        ..Default::default()
    };

    let entries = vec![upstream_entry];
    let report = batch_assemble_gost(&entries, false, &theme);

    assert!(
        report
            .skipped_remote
            .contains(&"Upstream Batch".to_string()),
        "Upstream repository must be skipped in batch assemble: {:?}",
        report
    );
    assert!(
        report.assembled.is_empty(),
        "No assemble should run for upstream repositories"
    );
}

#[test]
fn test_assemble_blocked_for_upstream_repo_by_alias() {
    let theme = Theme::with_color(false);

    let upstream_entry = CatalogEntry {
        name: "Upstream Official".to_string(),
        slug: "org/upstream-official".to_string(),
        aliases: vec!["official-lib".to_string()],
        is_upstream: true,
        ..Default::default()
    };

    let entries = vec![upstream_entry];
    let res = assemble_catalog_repo(&entries, "official-lib", false, false, None, &theme);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("upstream") && err.contains("isolado"));
}

#[test]
fn test_non_upstream_without_clone_reports_missing_workspace() {
    let theme = Theme::with_color(false);

    let remote_entry = CatalogEntry {
        name: "Remote Only".to_string(),
        slug: "user/remote-only".to_string(),
        is_upstream: false,
        ..Default::default()
    };

    let entries = vec![remote_entry];
    let res = assemble_catalog_repo(&entries, "remote-only", false, false, None, &theme);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("workspace local clonado"));
}
