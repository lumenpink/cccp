use cccp::cli::CzArgs;
use cccp::config::Config;
use cccp::cultural::Theme;
use cccp::cz::{compose_commit, format_conventional_commit};
use cccp::lint::lint_commit_message;

#[test]
fn test_format_conventional_commit_standard() {
    let msg = format_conventional_commit(
        "feat",
        Some("cli"),
        false,
        "implement interactive wizard",
        None,
        None,
        None,
    );
    assert_eq!(msg, "feat(cli): implement interactive wizard");
}

#[test]
fn test_format_conventional_commit_breaking() {
    let msg = format_conventional_commit(
        "feat",
        Some("config"),
        true,
        "rework cascading priority order",
        Some("Priority is now Defaults < Global < Local < ENV < CLI"),
        Some("Old config keys are deprecated"),
        Some("#73"),
    );
    let expected = "feat(config)!: rework cascading priority order\n\n\
                    Priority is now Defaults < Global < Local < ENV < CLI\n\n\
                    BREAKING CHANGE: Old config keys are deprecated\n\n\
                    Closes #73";
    assert_eq!(msg, expected);
}

#[test]
fn test_format_conventional_commit_custom_issue_syntax() {
    let msg = format_conventional_commit(
        "fix",
        None,
        false,
        "correct crash on null byte",
        None,
        None,
        Some("fixes #99"),
    );
    assert_eq!(msg, "fix: correct crash on null byte\n\nfixes #99");
}

#[test]
fn test_compose_non_interactive_success() {
    let args = CzArgs {
        commit_type: Some("feat".into()),
        scope: Some("semver".into()),
        message: Some("predict SemVer 2.0.0 from commits".into()),
        body: Some("Uses gix revwalk and semver crate".into()),
        dry_run: true,
        ..Default::default()
    };
    let config = Config::default();
    let theme = Theme::init(true);

    let res = compose_commit(&args, &config, &theme).expect("compose failed");
    assert_eq!(res.commit_type, "feat");
    assert_eq!(res.scope.as_deref(), Some("semver"));
    assert_eq!(res.subject, "predict SemVer 2.0.0 from commits");
    assert!(!res.is_breaking);

    // Verify it passes strict linter
    let lint_res = lint_commit_message(&res.formatted_message, &config, true);
    assert!(
        lint_res.is_valid,
        "Composed message must pass strict linter: {:?}",
        lint_res.errors
    );
}

#[test]
fn test_compose_non_interactive_breaking() {
    let args = CzArgs {
        commit_type: Some("refactor".into()),
        scope: Some("api".into()),
        message: Some("remove legacy endpoints".into()),
        body: None,
        breaking: true,
        dry_run: true,
        ..Default::default()
    };
    let config = Config::default();
    let theme = Theme::init(true);

    let res = compose_commit(&args, &config, &theme).expect("compose failed");
    assert_eq!(
        res.formatted_message,
        "refactor(api)!: remove legacy endpoints"
    );

    let lint_res = lint_commit_message(&res.formatted_message, &config, true);
    assert!(lint_res.is_valid);
}

#[test]
fn test_compose_non_interactive_missing_type_errors() {
    let args = CzArgs {
        commit_type: None,
        scope: Some("core".into()),
        message: Some("missing type".into()),
        dry_run: true,
        ..Default::default()
    };
    let config = Config::default();
    let theme = Theme::init(true);

    let res = compose_commit(&args, &config, &theme);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Commit type must be specified"));
}

#[test]
fn test_compose_non_interactive_invalid_subject_dot() {
    let args = CzArgs {
        commit_type: Some("fix".into()),
        message: Some("subject ending with dot.".into()),
        dry_run: true,
        ..Default::default()
    };
    let config = Config::default();
    let theme = Theme::init(true);

    let res = compose_commit(&args, &config, &theme);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("must not end with a period"));
}

#[test]
fn test_composed_commit_linter_compliance_all_types() {
    let types = [
        "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
        "revert",
    ];
    let config = Config::default();
    let theme = Theme::init(true);

    for t in types {
        let args = CzArgs {
            commit_type: Some(t.to_string()),
            scope: Some("core".to_string()),
            message: Some(format!("testing type {}", t)),
            body: Some("Testing body lines".to_string()),
            dry_run: true,
            ..Default::default()
        };
        let res = compose_commit(&args, &config, &theme).unwrap();
        let lint_res = lint_commit_message(&res.formatted_message, &config, true);
        assert!(
            lint_res.is_valid,
            "Type '{}' failed linter: {:?}",
            t, lint_res.errors
        );
    }
}
