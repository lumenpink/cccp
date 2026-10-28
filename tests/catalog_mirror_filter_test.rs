use cccp::catalog::{filter_and_sort_summaries, summarize_catalog, CatalogEntry};
use cccp::cli::CatalogFilterArgs;

#[test]
fn test_catalog_entry_deserializes_is_mirror() {
    let json = r#"[
        {
            "name": "canonical-project",
            "forge": "forgejo",
            "slug": "lumen/canonical-project",
            "is_mirror": false
        },
        {
            "name": "mirror-from-upstream",
            "forge": "forgejo",
            "slug": "lumen/mirror-from-upstream",
            "mirror": true
        },
        {
            "name": "direct-mirror-flag",
            "forge": "forgejo",
            "slug": "lumen/direct-mirror-flag",
            "is_mirror": true
        }
    ]"#;

    let entries: Vec<CatalogEntry> = serde_json::from_str(json).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(!entries[0].is_mirror);
    assert!(entries[1].is_mirror);
    assert!(entries[2].is_mirror);
}

#[test]
fn test_filter_and_sort_summaries_hides_mirrors_by_default() {
    let entries = vec![
        CatalogEntry {
            name: "repo-canonical".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/repo-canonical".to_string(),
            is_mirror: false,
            ..Default::default()
        },
        CatalogEntry {
            name: "repo-mirror".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/repo-mirror".to_string(),
            is_mirror: true,
            ..Default::default()
        },
    ];

    let mut summaries = summarize_catalog(&entries);
    assert_eq!(summaries.len(), 2);
    assert!(!summaries[0].is_mirror);
    assert!(summaries[1].is_mirror);

    // 1. Filtro padrão (include_mirrors = false) deve omitir o espelho
    let default_filter = CatalogFilterArgs::default();
    filter_and_sort_summaries(&mut summaries, &default_filter);

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "repo-canonical");

    // 2. Com include_mirrors = true, o espelho deve ser mantido
    let mut summaries_with_mirrors = summarize_catalog(&entries);
    let include_filter = CatalogFilterArgs {
        include_mirrors: true,
        ..Default::default()
    };

    filter_and_sort_summaries(&mut summaries_with_mirrors, &include_filter);
    assert_eq!(summaries_with_mirrors.len(), 2);
}

#[test]
fn test_filter_args_merge_preserves_include_mirrors() {
    let base = CatalogFilterArgs {
        include_mirrors: true,
        ..Default::default()
    };
    let sub = CatalogFilterArgs::default();
    assert!(base.merge(&sub).include_mirrors);

    let base2 = CatalogFilterArgs::default();
    let sub2 = CatalogFilterArgs {
        include_mirrors: true,
        ..Default::default()
    };
    assert!(base2.merge(&sub2).include_mirrors);
}

#[test]
fn test_dirty_and_status_filters_respect_mirror_omission() {
    use cccp::catalog::CatalogRepoSummary;

    let mut summaries = vec![
        CatalogRepoSummary {
            name: "canonical-dirty".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/canonical-dirty".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(false),
            unpushed: 0,
            status_label: "dirty [1*]".to_string(),
            local_path: None,
            gost_version: None,
            ci_status: Vec::new(),
            is_mirror: false,
        },
        CatalogRepoSummary {
            name: "mirror-dirty".to_string(),
            forge: "forgejo".to_string(),
            slug: "lumen/mirror-dirty".to_string(),
            aliases: vec![],
            ticket_count: 0,
            is_clean: Some(false),
            unpushed: 0,
            status_label: "dirty [1*]".to_string(),
            local_path: None,
            gost_version: None,
            ci_status: Vec::new(),
            is_mirror: true,
        },
    ];

    // Mesmo com --dirty, o espelho dirty DEVE ser ocultado sem --include-mirrors
    let dirty_filter = CatalogFilterArgs {
        dirty: true,
        include_mirrors: false,
        ..Default::default()
    };
    filter_and_sort_summaries(&mut summaries, &dirty_filter);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "canonical-dirty");
}
