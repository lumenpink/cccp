use cccp::editor::Editor;

#[test]
fn test_editor_soft_wrap_default_state() {
    let editor = Editor::new("Default Wrap Test", "Uma linha qualquer de teste");
    assert!(
        editor.soft_wrap,
        "Editor must initialize with soft_wrap active by default"
    );
}

#[test]
fn test_editor_toggle_soft_wrap_state_and_offset_reconciliation() {
    let mut editor = Editor::new("Toggle Test", "Linha longa para testar alternancia de wrap");
    assert!(editor.soft_wrap);

    // Set horizontal offset while wrap is on (simulate state)
    editor.col_offset = 15;

    // Toggle off: soft_wrap becomes false, visual_row_offset is reset
    let state_off = editor.toggle_soft_wrap();
    assert!(!state_off);
    assert!(!editor.soft_wrap);
    assert_eq!(editor.visual_row_offset, 0);

    // While wrap is off, user might scroll horizontally
    editor.col_offset = 25;

    // Toggle on: soft_wrap becomes true, col_offset must be zeroed to avoid distorting wrapped chunks
    let state_on = editor.toggle_soft_wrap();
    assert!(state_on);
    assert!(editor.soft_wrap);
    assert_eq!(
        editor.col_offset, 0,
        "col_offset must be reset to 0 when toggling soft_wrap on"
    );
}

#[test]
fn test_editor_toggle_soft_wrap_repeatedly() {
    let mut editor = Editor::new("Repeated Toggle", "Linha 1\nLinha 2");
    assert!(editor.soft_wrap);

    for _ in 0..5 {
        assert!(!editor.toggle_soft_wrap());
        assert!(editor.toggle_soft_wrap());
    }
}
