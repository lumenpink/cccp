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
