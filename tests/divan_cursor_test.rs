use cccp::editor::Editor;

#[test]
fn test_position_cursor_after_comments_divan_template() {
    let content = "# The Divan — Target: test\n# Describe the issue, bug, or feature request below.\n# Lines starting with '#' will be ignored.\n# Press Ctrl+S (or F2) to submit to AI triage, or Ctrl+Q (Esc) to cancel.\n\n";
    let mut editor = Editor::new("Test", content);
    editor.position_cursor_after_comments();

    // Lines 0..3 are comments starting with '#'
    // Line 4 is the blank line separating comments from prompt text
    assert_eq!(editor.cursor_y, 4);
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_position_cursor_after_comments_no_comments() {
    let content = "Primeira linha de texto\nSegunda linha";
    let mut editor = Editor::new("Test", content);
    editor.position_cursor_after_comments();

    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_position_cursor_after_comments_only_comments() {
    let content = "# Apenas comentario 1\n# Apenas comentario 2";
    let mut editor = Editor::new("Test", content);
    editor.position_cursor_after_comments();

    // Must ensure an editable blank line after comments exists and position cursor on it
    assert_eq!(editor.cursor_y, 2);
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.rows.len(), 3);
    assert_eq!(editor.rows[2], "");
}

#[test]
fn test_position_cursor_after_comments_empty_buffer() {
    let mut editor = Editor::new("Test", "");
    editor.position_cursor_after_comments();

    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_edit_prompt_signature() {
    // Calling edit_prompt in non-terminal mode should return Ok(Some(...)) with the content
    let content = "# Header comment\n\nMeu prompt";
    let res = cccp::editor::edit_prompt("Test Prompt", content);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Some(content.to_string()));
}
