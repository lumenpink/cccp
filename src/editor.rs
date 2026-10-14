//! CCCP Internal Minimalist Text Editor (inspired by Kvant / kv)
//! 100% pure in-process terminal microeditor with full keyboard navigation and zero external dependencies.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualLine {
    pub logical_row: usize,
    pub char_start: usize,
    pub char_len: usize,
}

pub struct Editor {
    pub title: String,
    pub rows: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub row_offset: usize,
    pub col_offset: usize,
    pub visual_row_offset: usize,
    pub screen_cols: usize,
    pub screen_rows: usize,
    pub dirty: bool,
    pub soft_wrap: bool,
    pub status_msg: String,
}

impl Editor {
    pub fn new(title: &str, initial_content: &str) -> Self {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let screen_rows = if rows > 2 { (rows - 2) as usize } else { 1 };
        let mut lines: Vec<String> = initial_content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            title: title.to_string(),
            rows: lines,
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            col_offset: 0,
            visual_row_offset: 0,
            screen_cols: cols as usize,
            screen_rows,
            dirty: false,
            soft_wrap: true,
            status_msg: format!("CCCP Editor — {} | Ctrl+S Save | Ctrl+Q Cancel", title),
        }
    }

    pub fn to_content(&self) -> String {
        let mut content = self.rows.join("\n");
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.rows.len() {
            self.rows.push(String::new());
        }

        let row = &mut self.rows[self.cursor_y];
        let byte_idx = char_to_byte_index(row, self.cursor_x);

        row.insert(byte_idx, c);
        self.cursor_x += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        if self.cursor_y >= self.rows.len() {
            self.rows.push(String::new());
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.dirty = true;
            return;
        }

        let row = &mut self.rows[self.cursor_y];
        let byte_idx = char_to_byte_index(row, self.cursor_x);

        let remainder = row.split_off(byte_idx);
        self.cursor_y += 1;
        self.rows.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        self.dirty = true;
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_x == 0 && self.cursor_y == 0 {
            return;
        }

        if self.cursor_x > 0 {
            let row = &mut self.rows[self.cursor_y];
            let (prev_byte_idx, _) = prev_char_byte_indices(row, self.cursor_x);

            row.remove(prev_byte_idx);
            self.cursor_x -= 1;
            self.dirty = true;
        } else if self.cursor_y > 0 {
            let current_row = self.rows.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.rows[self.cursor_y].chars().count();
            self.rows[self.cursor_y].push_str(&current_row);
            self.dirty = true;
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_y >= self.rows.len() {
            return;
        }

        let row_len = self.rows[self.cursor_y].chars().count();
        if self.cursor_x < row_len {
            let row = &mut self.rows[self.cursor_y];
            let byte_idx = char_to_byte_index(row, self.cursor_x);
            row.remove(byte_idx);
            self.dirty = true;
        } else if self.cursor_y + 1 < self.rows.len() {
            let next_row = self.rows.remove(self.cursor_y + 1);
            self.rows[self.cursor_y].push_str(&next_row);
            self.dirty = true;
        }
    }

    pub fn kill_to_end_of_line(&mut self) {
        if self.cursor_y >= self.rows.len() {
            return;
        }
        let row = &mut self.rows[self.cursor_y];
        let byte_idx = char_to_byte_index(row, self.cursor_x);

        if byte_idx < row.len() {
            row.truncate(byte_idx);
            self.dirty = true;
        } else if self.cursor_y + 1 < self.rows.len() {
            let next_row = self.rows.remove(self.cursor_y + 1);
            self.rows[self.cursor_y].push_str(&next_row);
            self.dirty = true;
        }
    }

    pub fn compute_visual_lines(&self) -> Vec<VisualLine> {
        let mut visual_lines = Vec::new();
        let cols = if self.screen_cols == 0 {
            80
        } else {
            self.screen_cols
        };

        for (log_idx, row) in self.rows.iter().enumerate() {
            let num_chars = row.chars().count();
            if !self.soft_wrap || num_chars <= cols {
                visual_lines.push(VisualLine {
                    logical_row: log_idx,
                    char_start: 0,
                    char_len: num_chars,
                });
            } else {
                let mut start = 0;
                while start < num_chars {
                    let len = (num_chars - start).min(cols);
                    visual_lines.push(VisualLine {
                        logical_row: log_idx,
                        char_start: start,
                        char_len: len,
                    });
                    start += len;
                }
            }
        }

        if visual_lines.is_empty() {
            visual_lines.push(VisualLine {
                logical_row: 0,
                char_start: 0,
                char_len: 0,
            });
        }

        visual_lines
    }

    pub fn cursor_to_visual_pos(&self, visual_lines: &[VisualLine]) -> (usize, usize) {
        if visual_lines.is_empty() {
            return (0, 0);
        }

        let mut candidate_v_idx = 0;
        let mut candidate_v_col = 0;

        for (v_idx, vl) in visual_lines.iter().enumerate() {
            if vl.logical_row == self.cursor_y {
                if self.cursor_x >= vl.char_start {
                    let is_last_chunk_for_row = v_idx + 1 == visual_lines.len()
                        || visual_lines[v_idx + 1].logical_row != self.cursor_y;

                    if self.cursor_x < vl.char_start + vl.char_len || is_last_chunk_for_row {
                        let col = (self.cursor_x - vl.char_start).min(vl.char_len);
                        return (v_idx, col);
                    }
                }
                candidate_v_idx = v_idx;
                candidate_v_col = self.cursor_x.saturating_sub(vl.char_start);
            }
        }

        (candidate_v_idx, candidate_v_col)
    }

    pub fn move_cursor(&mut self, key: KeyCode) {
        if self.soft_wrap {
            let vlines = self.compute_visual_lines();
            let (curr_v_row, curr_v_col) = self.cursor_to_visual_pos(&vlines);

            match key {
                KeyCode::Left => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    } else if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                        self.cursor_x = self.rows[self.cursor_y].chars().count();
                    }
                }
                KeyCode::Right => {
                    let line_len = if self.cursor_y < self.rows.len() {
                        self.rows[self.cursor_y].chars().count()
                    } else {
                        0
                    };
                    if self.cursor_x < line_len {
                        self.cursor_x += 1;
                    } else if self.cursor_y + 1 < self.rows.len() {
                        self.cursor_y += 1;
                        self.cursor_x = 0;
                    }
                }
                KeyCode::Up => {
                    if curr_v_row > 0 {
                        let target_v_row = curr_v_row - 1;
                        let target_vl = &vlines[target_v_row];
                        self.cursor_y = target_vl.logical_row;
                        let clamped_col = curr_v_col.min(target_vl.char_len);
                        self.cursor_x = target_vl.char_start + clamped_col;
                    }
                }
                KeyCode::Down => {
                    if curr_v_row + 1 < vlines.len() {
                        let target_v_row = curr_v_row + 1;
                        let target_vl = &vlines[target_v_row];
                        self.cursor_y = target_vl.logical_row;
                        let clamped_col = curr_v_col.min(target_vl.char_len);
                        self.cursor_x = target_vl.char_start + clamped_col;
                    }
                }
                KeyCode::Home => {
                    self.cursor_x = 0;
                }
                KeyCode::End => {
                    if self.cursor_y < self.rows.len() {
                        self.cursor_x = self.rows[self.cursor_y].chars().count();
                    }
                }
                KeyCode::PageUp => {
                    let target_v_row = curr_v_row.saturating_sub(self.screen_rows);
                    let target_vl = &vlines[target_v_row];
                    self.cursor_y = target_vl.logical_row;
                    let clamped_col = curr_v_col.min(target_vl.char_len);
                    self.cursor_x = target_vl.char_start + clamped_col;
                }
                KeyCode::PageDown => {
                    let target_v_row =
                        (curr_v_row + self.screen_rows).min(vlines.len().saturating_sub(1));
                    let target_vl = &vlines[target_v_row];
                    self.cursor_y = target_vl.logical_row;
                    let clamped_col = curr_v_col.min(target_vl.char_len);
                    self.cursor_x = target_vl.char_start + clamped_col;
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Left => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    } else if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                        self.cursor_x = self.rows[self.cursor_y].chars().count();
                    }
                }
                KeyCode::Right => {
                    let line_len = if self.cursor_y < self.rows.len() {
                        self.rows[self.cursor_y].chars().count()
                    } else {
                        0
                    };
                    if self.cursor_x < line_len {
                        self.cursor_x += 1;
                    } else if self.cursor_y + 1 < self.rows.len() {
                        self.cursor_y += 1;
                        self.cursor_x = 0;
                    }
                }
                KeyCode::Up => {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.cursor_y + 1 < self.rows.len() {
                        self.cursor_y += 1;
                    }
                }
                KeyCode::Home => {
                    self.cursor_x = 0;
                }
                KeyCode::End => {
                    if self.cursor_y < self.rows.len() {
                        self.cursor_x = self.rows[self.cursor_y].chars().count();
                    }
                }
                KeyCode::PageUp => {
                    self.cursor_y = self.cursor_y.saturating_sub(self.screen_rows);
                }
                KeyCode::PageDown => {
                    self.cursor_y =
                        (self.cursor_y + self.screen_rows).min(self.rows.len().saturating_sub(1));
                }
                _ => {}
            }
        }
    }

    pub fn clamp_cursor(&mut self) {
        if self.cursor_y >= self.rows.len() {
            self.cursor_y = if self.rows.is_empty() {
                0
            } else {
                self.rows.len() - 1
            };
        }
        let line_len = if self.rows.is_empty() {
            0
        } else {
            self.rows[self.cursor_y].chars().count()
        };
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }

        if self.soft_wrap {
            let vlines = self.compute_visual_lines();
            let (v_row, _) = self.cursor_to_visual_pos(&vlines);
            if v_row < self.visual_row_offset {
                self.visual_row_offset = v_row;
            }
            if v_row >= self.visual_row_offset + self.screen_rows {
                self.visual_row_offset = v_row - self.screen_rows + 1;
            }
            self.col_offset = 0;
        } else {
            if self.cursor_y < self.row_offset {
                self.row_offset = self.cursor_y;
            }
            if self.cursor_y >= self.row_offset + self.screen_rows {
                self.row_offset = self.cursor_y - self.screen_rows + 1;
            }

            if self.cursor_x < self.col_offset {
                self.col_offset = self.cursor_x;
            }
            if self.cursor_x >= self.col_offset + self.screen_cols {
                self.col_offset = self.cursor_x - self.screen_cols + 1;
            }
        }
    }

    pub fn render(&mut self) -> io::Result<()> {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        self.screen_cols = cols as usize;
        self.screen_rows = if rows > 2 { (rows - 2) as usize } else { 1 };
        self.clamp_cursor();

        let mut output = String::with_capacity(self.screen_cols * rows as usize * 2);
        // Hide cursor and position at top-left
        output.push_str("\x1b[?25l\x1b[H");

        let (cursor_screen_y, cursor_screen_x) = if self.soft_wrap {
            let vlines = self.compute_visual_lines();
            let (v_row, v_col) = self.cursor_to_visual_pos(&vlines);

            // 1. Text rows (visual)
            for y in 0..self.screen_rows {
                let v_idx = y + self.visual_row_offset;
                if v_idx < vlines.len() {
                    let vl = &vlines[v_idx];
                    let row_str = &self.rows[vl.logical_row];
                    let chars: Vec<char> = row_str.chars().collect();
                    if vl.char_len > 0 && vl.char_start < chars.len() {
                        let end = (vl.char_start + vl.char_len).min(chars.len());
                        let slice: String = chars[vl.char_start..end].iter().collect();
                        output.push_str(&slice);
                    }
                } else {
                    output.push('~');
                }
                output.push_str("\x1b[K\r\n");
            }

            (v_row.saturating_sub(self.visual_row_offset) + 1, v_col + 1)
        } else {
            // 1. Text rows (logical with horizontal scroll)
            for y in 0..self.screen_rows {
                let file_row = y + self.row_offset;
                if file_row < self.rows.len() {
                    let row_str = &self.rows[file_row];
                    let chars: Vec<char> = row_str.chars().collect();
                    if chars.len() > self.col_offset {
                        let end_idx = (self.col_offset + self.screen_cols).min(chars.len());
                        let slice: String = chars[self.col_offset..end_idx].iter().collect();
                        output.push_str(&slice);
                    }
                } else {
                    output.push('~');
                }
                output.push_str("\x1b[K\r\n");
            }

            (
                self.cursor_y.saturating_sub(self.row_offset) + 1,
                self.cursor_x.saturating_sub(self.col_offset) + 1,
            )
        };

        // 2. Status bar (reverse video)
        output.push_str("\x1b[7m");
        let dirty_flag = if self.dirty { " [+]" } else { "" };
        let wrap_flag = if self.soft_wrap { " [WRAP]" } else { "" };
        let left_status = format!(
            " ★ [CCCP EDITOR]{} | {}{} - {} lines",
            wrap_flag,
            self.title,
            dirty_flag,
            self.rows.len()
        );
        let right_status = format!("Ln {}, Col {} ", self.cursor_y + 1, self.cursor_x + 1);

        let pad_len = if self.screen_cols > left_status.len() + right_status.len() {
            self.screen_cols - left_status.len() - right_status.len()
        } else {
            0
        };

        output.push_str(&left_status);
        for _ in 0..pad_len {
            output.push(' ');
        }
        output.push_str(&right_status);
        output.push_str("\x1b[0m\r\n");

        // 3. Help footer (persistent shortcuts)
        output.push_str("\x1b[1;30;47m");
        let help =
            " Ctrl+S Save  Ctrl+Q / Esc Cancel  Alt+W Wrap  Arrows Navigate  Ctrl+K Cut Line ";
        output.push_str(help);
        if self.screen_cols > help.len() {
            for _ in 0..(self.screen_cols - help.len()) {
                output.push(' ');
            }
        }
        output.push_str("\x1b[0m");

        // 4. Place hardware cursor
        output.push_str(&format!(
            "\x1b[{};{}H\x1b[?25h",
            cursor_screen_y, cursor_screen_x
        ));

        let mut stdout = io::stdout();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}

/// Edit text interactively using the built-in minimalist terminal editor.
/// Returns Ok(Some(new_text)) if saved (Ctrl+S / F2), or Ok(None) if canceled (Ctrl+Q / Esc).
pub fn edit_text(title: &str, initial_content: &str) -> Result<Option<String>, String> {
    if !io::stdin().is_terminal() {
        return Ok(Some(initial_content.to_string()));
    }

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;
    enable_raw_mode().map_err(|e| format!("Failed to enable terminal raw mode: {}", e))?;

    let mut editor = Editor::new(title, initial_content);

    let result = loop {
        if let Err(e) = editor.render() {
            break Err(format!("Editor render error: {}", e));
        }

        match event::read() {
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => match (code, modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) | (KeyCode::F(2), _) => {
                    break Ok(Some(editor.to_content()));
                }
                (KeyCode::Char('q'), KeyModifiers::CONTROL)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                | (KeyCode::Esc, _) => {
                    break Ok(None);
                }
                (KeyCode::Char('w'), KeyModifiers::ALT) => {
                    editor.soft_wrap = !editor.soft_wrap;
                }
                (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                    editor.kill_to_end_of_line();
                }
                (KeyCode::Char(c), _) => {
                    editor.insert_char(c);
                }
                (KeyCode::Enter, _) => {
                    editor.insert_newline();
                }
                (KeyCode::Backspace, _) => {
                    editor.delete_char_backward();
                }
                (KeyCode::Delete, _) => {
                    editor.delete_char_forward();
                }
                (
                    KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::PageUp
                    | KeyCode::PageDown,
                    _,
                ) => {
                    editor.move_cursor(code);
                }
                _ => {}
            },
            Ok(Event::Resize(cols, rows)) => {
                editor.screen_cols = cols as usize;
                editor.screen_rows = if rows > 2 { (rows - 2) as usize } else { 1 };
            }
            Err(e) => {
                break Err(format!("Failed to read terminal event: {}", e));
            }
            _ => {}
        }
    };

    let _ = disable_raw_mode();
    let _ = execute!(stdout, LeaveAlternateScreen);
    result
}

/// Open a file in the built-in terminal editor, saving changes if confirmed (Ctrl+S / F2).
/// If `file_path` is None, opens a scratchpad and prints content to stdout upon save.
pub fn edit_file_or_scratch(file_path: Option<&std::path::Path>) -> Result<bool, String> {
    let (title, initial_content) = if let Some(path) = file_path {
        let content = if path.exists() {
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?
        } else {
            String::new()
        };
        (path.display().to_string(), content)
    } else {
        ("*scratch*".to_string(), String::new())
    };

    match edit_text(&title, &initial_content)? {
        Some(saved_content) => {
            if let Some(path) = file_path {
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }
                std::fs::write(path, saved_content)
                    .map_err(|e| format!("Failed to save file '{}': {}", path.display(), e))?;
            } else {
                print!("{}", saved_content);
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

fn char_to_byte_index(s: &str, char_target: usize) -> usize {
    let mut byte_idx = 0;
    for (idx, ch) in s.chars().enumerate() {
        if idx == char_target {
            break;
        }
        byte_idx += ch.len_utf8();
    }
    byte_idx
}

fn prev_char_byte_indices(s: &str, char_target: usize) -> (usize, usize) {
    let mut prev_byte = 0;
    let mut byte_idx = 0;
    for (idx, ch) in s.chars().enumerate() {
        if idx == char_target {
            break;
        }
        prev_byte = byte_idx;
        byte_idx += ch.len_utf8();
    }
    (prev_byte, byte_idx)
}
