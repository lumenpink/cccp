//! Integration tests for canonical URL, server host, and automatic domain/custom aliases (Ticket #69)

use cccp::catalog::{enrich_catalog_entry, extract_domain_prefix, CatalogEntry};
use cccp::config::Config;

#[test]
fn test_catalog_entry_canonical_url_and_server_host() {
    let mut entry = CatalogEntry {
        name: "CCCP Maestro".to_string(),
        forge: "https://gambi.2lp.in".to_string(),
        slug: "lumen/cccp".to_string(),
        ..Default::default()
    };

    enrich_catalog_entry(&mut entry, None);

    assert_eq!(
        entry.canonical_url.as_deref(),
        Some("https://gambi.2lp.in/lumen/cccp")
    );
    assert_eq!(entry.server_host.as_deref(), Some("gambi.2lp.in"));
}

#[test]
fn test_domain_based_automatic_alias_rules() {
    assert_eq!(
        extract_domain_prefix("gambi.2lp.in"),
        Some("gambi".to_string())
    );
    assert_eq!(
        extract_domain_prefix("git.subversao.su"),
        Some("subversao".to_string())
    );
    assert_eq!(
        extract_domain_prefix("codeberg.org"),
        Some("codeberg".to_string())
    );

    let mut entry1 = CatalogEntry {
        name: "CCCP".to_string(),
        forge: "https://gambi.2lp.in".to_string(),
        slug: "lumen/cccp".to_string(),
        ..Default::default()
    };
    enrich_catalog_entry(&mut entry1, None);

    assert!(
        entry1.aliases.contains(&"gambi:cccp".to_string()),
        "Aliases should contain gambi:cccp, got {:?}",
        entry1.aliases
    );
    assert!(
        entry1.aliases.contains(&"gambi/cccp".to_string()),
        "Aliases should contain gambi/cccp, got {:?}",
        entry1.aliases
    );

    let mut entry2 = CatalogEntry {
        name: "Kernel".to_string(),
        forge: "https://git.subversao.su".to_string(),
        slug: "soviet/kernel".to_string(),
        ..Default::default()
    };
    enrich_catalog_entry(&mut entry2, None);

    assert!(
        entry2.aliases.contains(&"subversao:kernel".to_string()),
        "Aliases should contain subversao:kernel, got {:?}",
        entry2.aliases
    );
}

#[test]
fn test_custom_aliases_from_config() {
    let mut config = Config::default();
    config
        .repo_aliases
        .insert("lumen/cccp".to_string(), vec!["my-maestro".to_string()]);

    let mut entry = CatalogEntry {
        name: "CCCP Maestro".to_string(),
        forge: "https://gambi.2lp.in".to_string(),
        slug: "lumen/cccp".to_string(),
        ..Default::default()
    };

    enrich_catalog_entry(&mut entry, Some(&config));

    assert!(
        entry.aliases.contains(&"my-maestro".to_string()),
        "Aliases should contain custom alias my-maestro, got {:?}",
        entry.aliases
    );
}

#[test]
fn test_ssh_and_port_host_extraction() {
    let mut entry = CatalogEntry {
        name: "Self-Hosted".to_string(),
        forge: "http://git.local.internal:8080".to_string(),
        slug: "team/service".to_string(),
        ..Default::default()
    };
    enrich_catalog_entry(&mut entry, None);
    assert_eq!(entry.server_host.as_deref(), Some("git.local.internal"));
    assert_eq!(
        extract_domain_prefix("git.local.internal"),
        Some("local".to_string())
    );
    assert!(entry.aliases.contains(&"local:service".to_string()));
}

#[test]
fn test_domain_prefix_edge_cases() {
    assert_eq!(extract_domain_prefix(""), None);
    assert_eq!(extract_domain_prefix("localhost"), Some("localhost".to_string()));
    assert_eq!(extract_domain_prefix("git"), Some("git".to_string()));
    assert_eq!(extract_domain_prefix("GIT.COMPANY.COM"), Some("company".to_string()));
}
