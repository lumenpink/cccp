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
