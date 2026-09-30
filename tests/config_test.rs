//! Exhaustive unit & integration tests for CCCP Gosplan hierarchical configuration.

use cccp::config::{find_local_config_path, parse_key_value_lines, write_config_key, Config};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_default_configuration_invariants() {
    let cfg = Config::default();

    // Verify default commit types
    let expected_types = [
        "feat", "fix", "perf", "refactor", "revert", "chore", "build", "ci", "docs", "ops",
        "style", "test", "merge",
    ];
    for t in expected_types {
        assert!(
            cfg.commit_types.iter().any(|x| x == t),
            "Default commit_types must include '{}'",
            t
        );
    }

    // Verify default scopes & subscopes
    assert!(cfg.commit_scopes.iter().any(|s| s == "core"));
    assert!(cfg.commit_scopes.iter().any(|s| s == "cli"));
    assert!(cfg.commit_subscopes.iter().any(|s| s == "utils"));

    // Verify default policy flags
    assert!(cfg.strict_types, "strict_types must default to true");
    assert!(!cfg.strict_scopes, "strict_scopes must default to false");
    assert!(
        !cfg.strict_subscopes,
        "strict_subscopes must default to false"
    );
    assert!(cfg.allow_any_scope, "allow_any_scope must default to true");
    assert!(
        cfg.allow_any_subscope,
        "allow_any_subscope must default to true"
    );
    assert!(!cfg.disable_subscopes);
    assert!(!cfg.disable_multiple_scopes);
    assert_eq!(cfg.default_base_version, "0.0.1");
    assert!(!cfg.no_v);
    assert_eq!(cfg.tickets_file, ".antigravity/tickets.md");
    assert_eq!(cfg.agents_md_file, ".antigravity/AGENTS.md");
    assert!(cfg.extra.is_empty());
}

#[test]
fn test_key_value_line_parser_robustness() {
    let raw = r#"
        # Leading comment
        ; Semicolon comment
        types = feat, fix, docs, refactor
        strict_types = 0
        export ALLOW_ANY_SCOPE = "false"
        default-base-version = '1.2.3'
        NO-V = true
        custom_directive = Maintain high Soviet standards
        empty_val =
        url = https://codeberg.org/lumenpink/cccp?ref=main&test=1
    "#;

    let kvs = parse_key_value_lines(raw);
    assert_eq!(kvs.len(), 8);

    let mut cfg = Config::default();
    cfg.merge_from_str(raw);

    assert_eq!(cfg.commit_types, vec!["feat", "fix", "docs", "refactor"]);
    assert!(!cfg.strict_types);
    assert!(!cfg.allow_any_scope);
    assert!(cfg.strict_scopes);
    assert_eq!(cfg.default_base_version, "1.2.3");
    assert!(cfg.no_v);
    assert_eq!(
        cfg.extra.get("custom_directive"),
        Some(&"Maintain high Soviet standards".to_string())
    );
    assert_eq!(
        cfg.extra.get("url"),
        Some(&"https://codeberg.org/lumenpink/cccp?ref=main&test=1".to_string())
    );
}

#[test]
fn test_malformed_and_edge_case_lines() {
    let malformed = r#"
        This line has no equals sign
        = Missing key
        # Valid comment
        ;; Another comment
        types = feat
        invalid line without separator
        trailing_whitespace_key   =    spaced_value   
    "#;

    let mut cfg = Config::default();
    cfg.merge_from_str(malformed);

    assert_eq!(cfg.commit_types, vec!["feat"]);
    assert_eq!(
        cfg.extra.get("trailing_whitespace_key"),
        Some(&"spaced_value".to_string())
    );
}

#[test]
fn test_boolean_parsing_variants() {
    let mut cfg = Config::default();

    cfg.set_key_value("strict_types", "1");
    assert!(cfg.strict_types);
    cfg.set_key_value("strict_types", "0");
    assert!(!cfg.strict_types);

    cfg.set_key_value("strict_types", "true");
    assert!(cfg.strict_types);
    cfg.set_key_value("strict_types", "false");
    assert!(!cfg.strict_types);

    cfg.set_key_value("strict_types", "yes");
    assert!(cfg.strict_types);
    cfg.set_key_value("strict_types", "no");
    assert!(!cfg.strict_types);

    cfg.set_key_value("strict_types", "on");
    assert!(cfg.strict_types);
    cfg.set_key_value("strict_types", "off");
    assert!(!cfg.strict_types);

    // Invalid boolean preserves current default
    cfg.set_key_value("strict_types", "invalid_choice");
    assert!(!cfg.strict_types);
}

#[test]
fn test_scope_policy_flags_cascade() {
    let mut cfg = Config::default();

    // Disabling allow_any_scope forces strict_scopes to true
    cfg.set_key_value("allow_any_scope", "0");
    assert!(!cfg.allow_any_scope);
    assert!(cfg.strict_scopes);

    // Re-enabling allow_any_scope
    cfg.set_key_value("allow_any_scope", "1");
    assert!(cfg.allow_any_scope);

    // Disabling allow_any_subscope forces strict_subscopes to true
    cfg.set_key_value("allow_any_subscope", "false");
    assert!(!cfg.allow_any_subscope);
    assert!(cfg.strict_subscopes);
}

#[test]
fn test_four_layer_precedence_cascade() {
    let mut cfg = Config::default();
    // Layer 1: Default
    assert_eq!(cfg.default_base_version, "0.0.1");
    assert!(cfg.strict_types);
    assert_eq!(cfg.tickets_file, ".antigravity/tickets.md");

    // Layer 2: Global configuration
    let global_content = r#"
        default_base_version = 1.0.0
        strict_types = 0
        tickets_file = .global_tickets.md
        global_custom = global_value
    "#;
    cfg.merge_from_str(global_content);
    assert_eq!(cfg.default_base_version, "1.0.0");
    assert!(!cfg.strict_types);
    assert_eq!(cfg.tickets_file, ".global_tickets.md");
    assert_eq!(
        cfg.extra.get("global_custom"),
        Some(&"global_value".to_string())
    );

    // Layer 3: Local repository .cccprc (overrides global)
    let local_content = r#"
        default_base_version = 2.0.0
        tickets_file = .local/tickets.md
        local_custom = local_value
    "#;
    cfg.merge_from_str(local_content);
    assert_eq!(cfg.default_base_version, "2.0.0"); // overridden by local
    assert!(!cfg.strict_types); // preserved from global
    assert_eq!(cfg.tickets_file, ".local/tickets.md"); // overridden by local
    assert_eq!(
        cfg.extra.get("global_custom"),
        Some(&"global_value".to_string())
    ); // preserved
    assert_eq!(
        cfg.extra.get("local_custom"),
        Some(&"local_value".to_string())
    );

    // Layer 4: Environment variables override file layers
    std::env::set_var("CCCP_DEFAULT_BASE_VERSION", "3.5.0");
    std::env::set_var("STRICT_TYPES", "1");
    cfg.merge_from_env();
    std::env::remove_var("CCCP_DEFAULT_BASE_VERSION");
    std::env::remove_var("STRICT_TYPES");

    assert_eq!(cfg.default_base_version, "3.5.0"); // overridden by ENV
    assert!(cfg.strict_types); // overridden by ENV
    assert_eq!(cfg.tickets_file, ".local/tickets.md"); // untouched by ENV
}

#[test]
fn test_local_config_file_discovery_upwards() {
    let dir = tempdir().expect("Failed to create tempdir");
    let root = dir.path();

    // Create root/.cccprc
    let cccprc_path = root.join(".cccprc");
    fs::write(&cccprc_path, "default_base_version = 5.0.0\n").expect("Failed to write .cccprc");

    // Create deeply nested dir
    let nested = root.join("src").join("deep").join("nested");
    fs::create_dir_all(&nested).expect("Failed to create nested dir");

    // Search upwards from nested dir
    let found = find_local_config_path(Some(&nested));
    assert_eq!(found, Some(cccprc_path));
}

#[test]
fn test_local_config_discovery_respects_git_boundary() {
    let dir = tempdir().expect("Failed to create tempdir");
    let root = dir.path();

    // Create .git boundary in root
    fs::create_dir(root.join(".git")).expect("Failed to create .git dir");

    // Do NOT create .cccprc in root
    let sub = root.join("sub");
    fs::create_dir(&sub).expect("Failed to create sub");

    // Discovery should stop at git root and return None
    let found = find_local_config_path(Some(&sub));
    assert_eq!(found, None);
}

#[test]
fn test_write_config_key_in_place_update() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file = dir.path().join("subdir").join(".cccprc");

    // Initial write creates file and directory
    write_config_key(&file, "types", "feat, fix, chore").expect("Failed to write initial key");
    write_config_key(&file, "strict_types", "1").expect("Failed to write strict_types");

    let content = fs::read_to_string(&file).expect("Failed to read file");
    assert!(content.contains("types = feat, fix, chore"));
    assert!(content.contains("strict_types = 1"));

    // Update existing key in place
    write_config_key(&file, "strict_types", "0").expect("Failed to update key");
    let updated = fs::read_to_string(&file).expect("Failed to read updated file");

    assert!(updated.contains("strict_types = 0"));
    assert!(!updated.contains("strict_types = 1"));
    assert!(updated.contains("types = feat, fix, chore"));
}
