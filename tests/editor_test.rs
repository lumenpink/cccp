use cccp::editor::Editor;

#[test]
fn test_editor_buffer_manipulation() {
    let mut editor = Editor::new("Test Buffer", "Line 1\nLine 2");
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0], "Line 1");
    assert_eq!(editor.rows[1], "Line 2");

    // Insert character at (0, 0)
    editor.insert_char('A');
    assert_eq!(editor.rows[0], "ALine 1");
    assert_eq!(editor.cursor_x, 1);
    assert!(editor.dirty);

    // Insert newline
    editor.insert_newline();
    assert_eq!(editor.rows.len(), 3);
    assert_eq!(editor.rows[0], "A");
    assert_eq!(editor.rows[1], "Line 1");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);

    // Delete backward merges line 1 into line 0
    editor.delete_char_backward();
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0], "ALine 1");
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 1);

    // Move to end and kill to end
    editor.cursor_x = 2;
    editor.kill_to_end_of_line();
    assert_eq!(editor.rows[0], "AL");

    // Content serialization
    let content = editor.to_content();
    assert_eq!(content, "AL\nLine 2\n");
}

#[test]
fn test_editor_cursor_clamping() {
    let mut editor = Editor::new("Test Clamping", "Short\nMuch longer line here");
    editor.cursor_y = 1;
    editor.cursor_x = 20;

    // Moving up to shorter line clamps cursor_x
    editor.move_cursor(crossterm::event::KeyCode::Up);
    assert_eq!(editor.cursor_y, 0);
    editor.clamp_cursor();
    assert_eq!(editor.cursor_x, 5);
}

#[test]
fn test_editor_cli_parsing() {
    use cccp::cli::{Cli, Commands};
    use clap::Parser;
    use std::path::PathBuf;

    let parsed_edit = Cli::parse_from(["cccp", "edit", "README.md"]);
    match parsed_edit.command {
        Some(Commands::Editor(args)) => {
            assert_eq!(args.file, Some(PathBuf::from("README.md")));
        }
        _ => panic!("Expected Commands::Editor"),
    }

    let parsed_editor = Cli::parse_from(["cccp", "editor", "src/main.rs"]);
    match parsed_editor.command {
        Some(Commands::Editor(args)) => {
            assert_eq!(args.file, Some(PathBuf::from("src/main.rs")));
        }
        _ => panic!("Expected Commands::Editor"),
    }

    let parsed_scratch = Cli::parse_from(["cccp", "editor"]);
    match parsed_scratch.command {
        Some(Commands::Editor(args)) => {
            assert_eq!(args.file, None);
        }
        _ => panic!("Expected Commands::Editor"),
    }
}

#[test]
fn test_editor_buffer_file_io() {
    use tempfile::NamedTempFile;

    let temp = NamedTempFile::new().expect("Failed to create tempfile");
    std::fs::write(temp.path(), "Initial content\nSecond line").expect("Write failed");

    let content = std::fs::read_to_string(temp.path()).expect("Read failed");
    let mut editor = Editor::new("Test IO", &content);
    assert_eq!(editor.rows.len(), 2);
    assert_eq!(editor.rows[0], "Initial content");

    editor.insert_char('!');
    let new_content = editor.to_content();
    std::fs::write(temp.path(), &new_content).expect("Write back failed");

    let saved = std::fs::read_to_string(temp.path()).expect("Read failed");
    assert_eq!(saved, "!Initial content\nSecond line\n");
}

#[test]
fn test_editor_soft_wrap_visual_lines_computation() {
    let content = "0123456789ABCDEFGHIJ01234\n\nHello";
    let mut editor = Editor::new("Test Wrap", content);
    editor.screen_cols = 10;
    editor.soft_wrap = true;

    let vlines = editor.compute_visual_lines();
    assert_eq!(vlines.len(), 5);

    // Line 0 chunks
    assert_eq!(vlines[0].logical_row, 0);
    assert_eq!(vlines[0].char_start, 0);
    assert_eq!(vlines[0].char_len, 10);

    assert_eq!(vlines[1].logical_row, 0);
    assert_eq!(vlines[1].char_start, 10);
    assert_eq!(vlines[1].char_len, 10);

    assert_eq!(vlines[2].logical_row, 0);
    assert_eq!(vlines[2].char_start, 20);
    assert_eq!(vlines[2].char_len, 5);

    // Line 1 (empty)
    assert_eq!(vlines[3].logical_row, 1);
    assert_eq!(vlines[3].char_start, 0);
    assert_eq!(vlines[3].char_len, 0);

    // Line 2 (5 chars)
    assert_eq!(vlines[4].logical_row, 2);
    assert_eq!(vlines[4].char_start, 0);
    assert_eq!(vlines[4].char_len, 5);

    // Without soft wrap: 1 visual line per logical line
    editor.soft_wrap = false;
    let no_wrap_lines = editor.compute_visual_lines();
    assert_eq!(no_wrap_lines.len(), 3);
}

#[test]
fn test_editor_soft_wrap_cursor_mapping_and_navigation() {
    let content = "0123456789ABCDEFGHIJ01234\nSecond";
    let mut editor = Editor::new("Test Nav", content);
    editor.screen_cols = 10;
    editor.soft_wrap = true;

    let vlines = editor.compute_visual_lines();

    // Map (y:0, x:0) -> visual line 0, visual col 0
    editor.cursor_y = 0;
    editor.cursor_x = 0;
    assert_eq!(editor.cursor_to_visual_pos(&vlines), (0, 0));

    // Map (y:0, x:15) -> visual line 1, visual col 5
    editor.cursor_x = 15;
    assert_eq!(editor.cursor_to_visual_pos(&vlines), (1, 5));

    // Map (y:0, x:22) -> visual line 2, visual col 2
    editor.cursor_x = 22;
    assert_eq!(editor.cursor_to_visual_pos(&vlines), (2, 2));

    // Navigation: Down inside long logical line
    editor.cursor_y = 0;
    editor.cursor_x = 3; // (visual 0, col 3)
    editor.move_cursor(crossterm::event::KeyCode::Down);
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 13); // (visual 1, col 3)

    editor.move_cursor(crossterm::event::KeyCode::Down);
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 23); // (visual 2, col 3)

    // Up inside long logical line
    editor.move_cursor(crossterm::event::KeyCode::Up);
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 13); // back to (visual 1, col 3)

    // Buffer preservation: to_content() has no inserted newlines
    let out = editor.to_content();
    assert_eq!(out, "0123456789ABCDEFGHIJ01234\nSecond\n");
}

#[test]
fn test_editor_soft_wrap_mutation_edge_cases() {
    // 1. Exact boundary (len == screen_cols)
    let mut editor = Editor::new("Boundary", "1234567890");
    editor.screen_cols = 10;
    editor.soft_wrap = true;
    let vlines = editor.compute_visual_lines();
    assert_eq!(
        vlines.len(),
        1,
        "Exact screen_cols width must produce exactly 1 chunk"
    );
    assert_eq!(vlines[0].char_len, 10);

    // 2. Off-by-one boundary (len == screen_cols + 1)
    let mut editor_plus = Editor::new("Boundary + 1", "12345678901");
    editor_plus.screen_cols = 10;
    editor_plus.soft_wrap = true;
    let vlines_plus = editor_plus.compute_visual_lines();
    assert_eq!(
        vlines_plus.len(),
        2,
        "screen_cols + 1 must produce 2 chunks"
    );
    assert_eq!(vlines_plus[0].char_len, 10);
    assert_eq!(vlines_plus[1].char_len, 1);

    // 3. UTF-8 multibyte characters (accents, emojis, cyrillic)
    // 10 chars: "Café ☭ pão" (contains multibyte é, ☭, ã)
    let utf8_text = "Café ☭ pão!"; // 11 chars
    let mut editor_utf8 = Editor::new("UTF8", utf8_text);
    editor_utf8.screen_cols = 5;
    editor_utf8.soft_wrap = true;
    let vlines_utf8 = editor_utf8.compute_visual_lines();
    assert_eq!(vlines_utf8.len(), 3); // 5, 5, 1
    assert_eq!(vlines_utf8[0].char_len, 5);
    assert_eq!(vlines_utf8[1].char_len, 5);
    assert_eq!(vlines_utf8[2].char_len, 1);

    // 4. PageDown and PageUp navigation with soft wrap
    let mut editor_page = Editor::new(
        "PageNav",
        "Line 0\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6",
    );
    editor_page.screen_cols = 80;
    editor_page.screen_rows = 3;
    editor_page.soft_wrap = true;
    editor_page.cursor_y = 0;
    editor_page.cursor_x = 0;

    editor_page.move_cursor(crossterm::event::KeyCode::PageDown);
    assert_eq!(editor_page.cursor_y, 3);

    editor_page.move_cursor(crossterm::event::KeyCode::PageUp);
    assert_eq!(editor_page.cursor_y, 0);

    // 5. Clamping and rendering does not panic with empty lines or wrapped lines
    let mut empty_editor = Editor::new("Empty", "");
    empty_editor.screen_cols = 20;
    empty_editor.screen_rows = 5;
    empty_editor.soft_wrap = true;
    empty_editor.clamp_cursor();
    assert_eq!(empty_editor.cursor_x, 0);
    assert_eq!(empty_editor.cursor_y, 0);
}

#[test]
fn test_parse_ticket_buffer_valid() {
    let raw = "feat(cli): integrar revisao e submissao de tickets\n\n## Description\nDetalhes do ticket aqui.\n";
    let (title, body) =
        cccp::editor::parse_ticket_buffer(raw).expect("Must parse valid ticket buffer");
    assert_eq!(title, "feat(cli): integrar revisao e submissao de tickets");
    assert_eq!(body, "## Description\nDetalhes do ticket aqui.");
}

#[test]
fn test_parse_ticket_buffer_without_blank_separator() {
    let raw = "fix(divan): corrigir fluxo direto\n## Description\nCorpo direto sem espaco.";
    let (title, body) =
        cccp::editor::parse_ticket_buffer(raw).expect("Must parse buffer without blank separator");
    assert_eq!(title, "fix(divan): corrigir fluxo direto");
    assert_eq!(body, "## Description\nCorpo direto sem espaco.");
}

#[test]
fn test_parse_ticket_buffer_errors() {
    // Empty buffer
    assert!(cccp::editor::parse_ticket_buffer("").is_err());
    assert!(cccp::editor::parse_ticket_buffer("   \n\n  ").is_err());

    // Title only, empty body
    assert!(cccp::editor::parse_ticket_buffer("feat(editor): only title\n\n").is_err());
    assert!(cccp::editor::parse_ticket_buffer("feat(editor): only title").is_err());
}

#[test]
fn test_validate_ticket_submission_conventional_commits() {
    let config = cccp::config::Config::default();

    // Valid conventional commit title and non-empty body
    assert!(cccp::editor::validate_ticket_submission(
        "feat(cli): submit tickets from editor",
        "## Description\nValid body content",
        &config
    )
    .is_ok());

    // Invalid non-conventional title
    let res_invalid_title = cccp::editor::validate_ticket_submission(
        "Invalid title without conventional commit prefix",
        "## Description\nValid body content",
        &config,
    );
    assert!(
        res_invalid_title.is_err(),
        "Non-conventional title must be rejected"
    );

    // Empty body
    let res_empty_body = cccp::editor::validate_ticket_submission(
        "feat(cli): submit tickets from editor",
        "   \n  ",
        &config,
    );
    assert!(res_empty_body.is_err(), "Empty body must be rejected");
}

#[test]
fn test_editor_contextual_ticket_labels() {
    let mut editor = Editor::new("Ticket #138", "feat(cli): title\n\nbody");
    editor.set_header_prefix("CCCP TICKET EDITOR");
    editor.set_save_label("Submit");

    assert_eq!(editor.header_prefix, "CCCP TICKET EDITOR");
    assert_eq!(editor.save_label, "Submit");
}

#[test]
fn test_parse_and_validate_mutation_edge_cases() {
    let config = cccp::config::Config::default();

    // 1. Multiple leading blank lines before title
    let raw_leading = "\n\n   \nfeat(core): title after leading blanks\n\nBody content";
    let (t1, b1) =
        cccp::editor::parse_ticket_buffer(raw_leading).expect("Should handle leading blank lines");
    assert_eq!(t1, "feat(core): title after leading blanks");
    assert_eq!(b1, "Body content");

    // 2. Multiple blank lines separating title and body
    let raw_multiblank = "feat(ui): cool title\n\n\n\nParagraph 1\n\nParagraph 2";
    let (t2, b2) = cccp::editor::parse_ticket_buffer(raw_multiblank)
        .expect("Should handle multiple blank lines");
    assert_eq!(t2, "feat(ui): cool title");
    assert_eq!(b2, "Paragraph 1\n\nParagraph 2");

    // 3. UTF-8 accents and symbols in title and body
    let raw_utf8 =
        "feat(i18n): suporte a caracteres acentuados ☭\n\nDescrição detalhada com acentuação.";
    let (t3, b3) = cccp::editor::parse_ticket_buffer(raw_utf8).expect("Should handle utf8");
    assert!(cccp::editor::validate_ticket_submission(&t3, &b3, &config).is_ok());

    // 4. Breaking change exclamation mark in title
    let raw_breaking = "feat(api)!: breaking change endpoint\n\nDetails of breaking change";
    let (t4, b4) = cccp::editor::parse_ticket_buffer(raw_breaking).expect("Should parse breaking");
    assert!(cccp::editor::validate_ticket_submission(&t4, &b4, &config).is_ok());

    // 5. Subject ending with period must be rejected by validator
    let res_dot = cccp::editor::validate_ticket_submission(
        "feat(api): title ends with dot.",
        "Body",
        &config,
    );
    assert!(
        res_dot.is_err(),
        "Subject ending with dot must fail validation"
    );

    // 6. Empty scope parentheses 'feat(): title' must be rejected
    let res_empty_scope =
        cccp::editor::validate_ticket_submission("feat(): title", "Body", &config);
    assert!(res_empty_scope.is_err(), "Empty scope must fail validation");

    // 7. Unmatched parenthesis 'feat(scope: title' must be rejected
    let res_unmatched =
        cccp::editor::validate_ticket_submission("feat(scope: title", "Body", &config);
    assert!(
        res_unmatched.is_err(),
        "Unmatched scope paren must fail validation"
    );
}
