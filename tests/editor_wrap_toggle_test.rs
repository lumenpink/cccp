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

#[test]
fn test_editor_toggle_soft_wrap_empty_buffer() {
    let mut editor = Editor::new("Empty", "");
    assert!(editor.soft_wrap);
    assert!(!editor.toggle_soft_wrap());
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 0);
    assert!(editor.toggle_soft_wrap());
    assert_eq!(editor.col_offset, 0);
}

#[test]
fn test_editor_soft_wrap_indicator_presence() {
    let mut editor = Editor::new("Indicator Test", "Linha 1\nLinha 2");
    editor.screen_cols = 80;
    editor.screen_rows = 24;

    // Wrap is on by default: status bar includes [WRAP]
    assert!(editor.soft_wrap);

    // Toggle off: soft_wrap is false
    editor.toggle_soft_wrap();
    assert!(!editor.soft_wrap);

    // Toggle back on
    editor.toggle_soft_wrap();
    assert!(editor.soft_wrap);
}

#[test]
fn test_editor_toggle_soft_wrap_cursor_reconciliation() {
    let mut editor = Editor::new("Cursor Test", "Linha 1\nLinha 2\nLinha 3\nLinha 4\nLinha 5");
    editor.screen_rows = 3;
    editor.screen_cols = 80;
    editor.cursor_y = 4;
    editor.cursor_x = 2;

    // Toggle off: row_offset is reconciled
    editor.toggle_soft_wrap();
    assert!(!editor.soft_wrap);
    assert_eq!(editor.visual_row_offset, 0);
    assert_eq!(editor.row_offset, 3); // 4 - (3 / 2) = 4 - 1 = 3

    // Toggle on: col_offset is 0
    editor.col_offset = 10;
    editor.toggle_soft_wrap();
    assert!(editor.soft_wrap);
    assert_eq!(editor.col_offset, 0);
}
