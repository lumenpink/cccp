use cccp::profile::{
    assemble_rules_specs, parse_latest_version_from_catalog, read_profile_config,
    write_profile_config, ProfileConfig,
};
use tempfile::tempdir;

#[test]
fn test_read_and_write_profile_toml() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let cfg = ProfileConfig {
        url: "https://lilya.2lp.in".to_string(),
        version: "2.1.0".to_string(),
    };

    write_profile_config(root, &cfg).expect("write profile.toml");

    let read_back = read_profile_config(root).expect("read profile.toml");
    assert_eq!(read_back.url, "https://lilya.2lp.in");
    assert_eq!(read_back.version, "2.1.0");
}

#[test]
fn test_resolve_version_from_versions_json() {
    let sample_json = r#"{
        "standard": "lilya",
        "tier": "operational",
        "latest": "2.1.0",
        "versions": ["2.1.0", "2.0.0"]
    }"#;

    let latest = parse_latest_version_from_catalog(sample_json).expect("parse versions.json");
    assert_eq!(latest, "2.1.0");
}

#[test]
fn test_assemble_rules_into_dot_rules() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let cfg = ProfileConfig {
        url: "https://lilya.2lp.in".to_string(),
        version: "2.1.0".to_string(),
    };
    write_profile_config(root, &cfg).unwrap();

    // Assemble specifications
    let assembled = assemble_rules_specs(root).expect("assemble rules");
    assert!(!assembled.is_empty());

    let rules_dir = root.join(".rules");
    assert!(
        rules_dir.join("RULES.md").is_file(),
        "RULES.md must exist in .rules/"
    );
    assert!(
        rules_dir.join("RULES.toml").is_file(),
        "RULES.toml must exist in .rules/"
    );

    // Must NOT generate legacy files
    assert!(
        !rules_dir.join("rules.json").exists(),
        "rules.json must NOT be generated"
    );
    assert!(
        !rules_dir.join("PROFILE.md").exists(),
        "PROFILE.md must NOT be generated"
    );
    assert!(!root.join(".gost").exists(), ".gost must NOT be created");
}

#[test]
fn test_mutation_catalog_invalid_payload_returns_none() {
    let invalid_json = r#"{"invalid": "schema"}"#;
    assert!(parse_latest_version_from_catalog(invalid_json).is_none());
}

#[test]
fn test_mutation_missing_profile_config_returns_not_found() {
    let dir = tempdir().unwrap();
    assert!(read_profile_config(dir.path()).is_err());
}
