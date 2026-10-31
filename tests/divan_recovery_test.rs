//! Tests for Divan recovery flow after AI triage or forge creation failure (Ticket #111)

use cccp::editor::{
    extract_prompt_from_buffer, format_divan_prompt_template, parse_forge_failure_action,
    parse_triage_failure_action, DivanRecoveryAction,
};

#[test]
fn test_format_divan_prompt_template_preserves_previous_content_on_recovery() {
    let target = "lumen/cccp";
    let template_fresh = format_divan_prompt_template(target, None);
    assert!(template_fresh.contains("The Divan — Target: lumen/cccp"));
    assert!(!template_fresh.contains("Bug no auth"));

    let previous_prompt = "Bug no auth: token expirado ao conectar com forge";
    let template_recovered = format_divan_prompt_template(target, Some(previous_prompt));
    assert!(template_recovered.contains("The Divan — Target: lumen/cccp"));
    assert!(template_recovered.contains(previous_prompt));

    let extracted = extract_prompt_from_buffer(&template_recovered);
    assert_eq!(extracted, previous_prompt);
}

#[test]
fn test_extract_prompt_from_buffer_strips_comments_and_whitespace() {
    let raw = "# Comment line 1\n# Comment line 2\n\nFix responsive layout in terminal\n# trailing comment\nAdditional details\n";
    let extracted = extract_prompt_from_buffer(raw);
    assert_eq!(
        extracted,
        "Fix responsive layout in terminal\nAdditional details"
    );
}

#[test]
fn test_parse_triage_failure_action_defaults_to_reopen_editor() {
    // Pressing 'e' or Enter ('\n') or unrecognized key must return ReopenInEditor
    assert_eq!(
        parse_triage_failure_action('e'),
        DivanRecoveryAction::ReopenInEditor
    );
    assert_eq!(
        parse_triage_failure_action('\n'),
        DivanRecoveryAction::ReopenInEditor
    );
    assert_eq!(
        parse_triage_failure_action('\r'),
        DivanRecoveryAction::ReopenInEditor
    );
    assert_eq!(
        parse_triage_failure_action('x'),
        DivanRecoveryAction::ReopenInEditor
    );

    // 'q' must quit cleanly
    assert_eq!(parse_triage_failure_action('q'), DivanRecoveryAction::Quit);
}

#[test]
fn test_parse_forge_failure_action_defaults_to_retry_in_editor() {
    // Default must be ReopenInEditor, preventing silent skipping or blind submission
    assert_eq!(
        parse_forge_failure_action('r'),
        DivanRecoveryAction::ReopenInEditor
    );
    assert_eq!(
        parse_forge_failure_action('\n'),
        DivanRecoveryAction::ReopenInEditor
    );
    assert_eq!(
        parse_forge_failure_action('\r'),
        DivanRecoveryAction::ReopenInEditor
    );

    // Explicit 's' must skip ticket
    assert_eq!(parse_forge_failure_action('s'), DivanRecoveryAction::Skip);

    // Explicit 'q' must quit
    assert_eq!(parse_forge_failure_action('q'), DivanRecoveryAction::Quit);
}
