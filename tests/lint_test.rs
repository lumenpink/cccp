//! Comprehensive unit & functional tests for GOST commit linting.
//! Validates types, scopes, subscopes, breaking change markers, length constraints, and comment filtering.

use cccp::config::Config;
use cccp::lint::{lint_commit_file, lint_commit_message};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_lint_all_standard_types() {
    let config = Config::default();
    let standard_types = [
        "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
        "revert",
    ];

    for t in standard_types {
        let msg = format!("{}: valid description for {}", t, t);
        let res = lint_commit_message(&msg, &config, false);
        assert!(
            res.is_valid,
            "Expected type '{}' to be valid, got errors: {:?}",
            t, res.errors
        );
    }
}

#[test]
fn test_lint_strict_types_vs_lenient() {
    // Lenient mode allows custom alphanumeric types
    let mut config = Config {
        strict_types: false,
        ..Default::default()
    };
    let res = lint_commit_message("custom-type: do something", &config, false);
    assert!(res.is_valid, "Lenient mode should allow custom type");

    // Invalid characters in type are rejected even in lenient mode
    let res_invalid = lint_commit_message("invalid type!: do something", &config, false);
    assert!(!res_invalid.is_valid);

    // Strict mode requires type to be in config.commit_types
    config.strict_types = true;
    let res_strict = lint_commit_message("custom-type: do something", &config, false);
    assert!(!res_strict.is_valid);
    assert!(res_strict.errors[0].contains("Invalid type 'custom-type'"));

    // Standard types still pass in strict mode
    let res_standard = lint_commit_message("feat: implement strict check", &config, false);
    assert!(res_standard.is_valid);
}

#[test]
fn test_lint_scopes_and_subscopes() {
    let mut config = Config::default();

    // 1. Valid single scope
    let res = lint_commit_message("feat(core): add feature", &config, false);
    assert!(res.is_valid);

    // 2. Subscope (scope/subscope)
    let res_sub = lint_commit_message("feat(api/auth): add jwt", &config, false);
    assert!(res_sub.is_valid);

    // 3. Subscopes disabled by config
    config.disable_subscopes = true;
    let res_sub_disabled = lint_commit_message("feat(api/auth): add jwt", &config, false);
    assert!(!res_sub_disabled.is_valid);
    assert!(res_sub_disabled.errors[0].contains("Subscopes are disabled"));
    config.disable_subscopes = false;

    // 4. Multiple scopes (comma-separated)
    let res_multi = lint_commit_message("fix(api,ui): align response", &config, false);
    assert!(res_multi.is_valid);

    // 5. Multiple scopes disabled by config
    config.disable_multiple_scopes = true;
    let res_multi_disabled = lint_commit_message("fix(api,ui): align response", &config, false);
    assert!(!res_multi_disabled.is_valid);
    assert!(res_multi_disabled.errors[0].contains("Multiple scopes are disabled"));
    config.disable_multiple_scopes = false;
}

#[test]
fn test_lint_strict_scope_enforcement() {
    let mut config = Config {
        strict_scopes: true,
        commit_scopes: vec!["core".to_string(), "cli".to_string()],
        allow_any_scope: false,
        ..Default::default()
    };

    // Allowed scope
    let res_ok = lint_commit_message("feat(core): update kernel", &config, false);
    assert!(res_ok.is_valid);

    // Disallowed scope
    let res_err = lint_commit_message("feat(unknown): update kernel", &config, false);
    assert!(!res_err.is_valid);
    assert!(res_err.errors[0].contains("Invalid scope 'unknown'"));

    // allow_any_scope overrides strict_scopes
    config.allow_any_scope = true;
    let res_override = lint_commit_message("feat(unknown): update kernel", &config, false);
    assert!(res_override.is_valid);
}

#[test]
fn test_lint_breaking_change_indicators() {
    let config = Config::default();

    // Bang on type
    let res_bang = lint_commit_message("feat!: drop legacy protocol", &config, false);
    assert!(res_bang.is_valid);

    // Bang on scope
    let res_bang_scope = lint_commit_message("feat(api)!: drop legacy protocol", &config, false);
    assert!(res_bang_scope.is_valid);

    // Multiline with BREAKING CHANGE footer
    let msg_footer = "feat(core): refactor engine\n\nBREAKING CHANGE: config schema changed";
    let res_footer = lint_commit_message(msg_footer, &config, false);
    assert!(res_footer.is_valid);

    // Multiline with BREAKING-CHANGE footer
    let msg_hyphen = "feat(core): refactor engine\n\nBREAKING-CHANGE: config schema changed";
    let res_hyphen = lint_commit_message(msg_hyphen, &config, false);
    assert!(res_hyphen.is_valid);
}

#[test]
fn test_lint_header_length_ceiling() {
    let config = Config::default();

    // Exactly 72 chars
    let exact_72 = format!("feat(core): {}", "a".repeat(60));
    assert_eq!(exact_72.chars().count(), 72);
    let res_72 = lint_commit_message(&exact_72, &config, true);
    assert!(res_72.is_valid);
    assert!(res_72.warnings.is_empty());

    // 73 chars: warning in non-strict mode
    let len_73 = format!("feat(core): {}", "a".repeat(61));
    assert_eq!(len_73.chars().count(), 73);
    let res_warn = lint_commit_message(&len_73, &config, false);
    assert!(res_warn.is_valid);
    assert_eq!(res_warn.warnings.len(), 1);
    assert!(res_warn.warnings[0].contains("exceeds 72 characters"));

    // 73 chars: error in strict mode
    let res_strict = lint_commit_message(&len_73, &config, true);
    assert!(!res_strict.is_valid);
    assert_eq!(res_strict.errors.len(), 1);
    assert!(res_strict.errors[0].contains("exceeds 72 characters"));
}

#[test]
fn test_lint_syntax_and_formatting_errors() {
    let config = Config::default();

    // 1. Empty message
    let r_empty = lint_commit_message("", &config, false);
    assert!(!r_empty.is_valid);
    assert!(r_empty.errors[0].contains("cannot be empty"));

    // 2. Whitespace only
    let r_ws = lint_commit_message("   \n\t  ", &config, false);
    assert!(!r_ws.is_valid);

    // 3. Missing colon separator
    let r_no_colon = lint_commit_message("feat add awesome feature", &config, false);
    assert!(!r_no_colon.is_valid);
    assert!(r_no_colon.errors[0].contains("must follow format"));

    // 4. Empty parentheses
    let r_empty_paren = lint_commit_message("feat(): missing scope", &config, false);
    assert!(!r_empty_paren.is_valid);
    assert!(r_empty_paren.errors[0].contains("Scope cannot be empty"));

    // 5. Unmatched parenthesis
    let r_unmatched = lint_commit_message("feat(core: missing closing paren", &config, false);
    assert!(!r_unmatched.is_valid);
    assert!(r_unmatched.errors[0].contains("Unmatched parenthesis"));

    // 6. Empty subject
    let r_empty_subj = lint_commit_message("feat(core):   ", &config, false);
    assert!(!r_empty_subj.is_valid);
    assert!(r_empty_subj.errors[0].contains("must have a subject description"));

    // 7. Trailing period in subject
    let r_dot = lint_commit_message("feat(core): do not add trailing period.", &config, false);
    assert!(!r_dot.is_valid);
    assert!(r_dot.errors[0].contains("must not end with a period/dot"));

    // 8. Missing blank line between subject and body
    let r_no_blank = lint_commit_message(
        "feat: add feature\nThis is immediately the body line",
        &config,
        false,
    );
    assert!(!r_no_blank.is_valid);
    assert!(r_no_blank.errors[0].contains("blank line is required"));
}

#[test]
fn test_lint_multiline_body_and_footers() {
    let config = Config::default();

    let full_commit = "\
feat(cli): introduce interactive commit wizard

Detailed explanation of how the wizard operates:
- Uses inquire for interactive prompts
- Formats standard Conventional Commits
- Validates 72-char summary ceiling

Closes #101
Fixes #102";

    let res = lint_commit_message(full_commit, &config, false);
    assert!(res.is_valid, "Expected valid commit: {:?}", res.errors);
    assert!(res.errors.is_empty());
}

#[test]
fn test_lint_commit_file_comment_stripping() {
    let config = Config::default();

    // Create temp commit message file with Git comments
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let content = "\
feat(git): add commit file verification

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch main
# Changes to be committed:
#	modified:   src/lint.rs
#";
    file.write_all(content.as_bytes()).expect("write failed");

    let res = lint_commit_file(file.path(), &config, false).expect("lint_commit_file failed");
    assert!(res.is_valid, "Expected valid after comments stripped");

    // File not found error
    let bad_path = std::path::Path::new("/nonexistent/file/path/COMMIT_MSG");
    let err = lint_commit_file(bad_path, &config, false);
    assert!(err.is_err());
}
