use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::wrap::{prev_char_boundary, wrapped_offsets, cursor_in_wrapped, wrapped_to_cursor, wrap_lines};

/// Generic text editor state — shared by inline comments and the body overlay.
pub struct TextEditor {
    pub text: String,
    /// Byte offset into `text`.
    pub cursor_pos: usize,
}

impl TextEditor {
    pub fn with_text(text: String) -> Self {
        let cursor_pos = text.len();
        Self { text, cursor_pos }
    }

    // ── mutation ──────────────────────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.text.insert_str(self.cursor_pos, s);
        self.cursor_pos += s.len();
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos == 0 { return; }
        let pos = prev_char_boundary(&self.text, self.cursor_pos);
        self.text.remove(pos);
        self.cursor_pos = pos;
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        self.text.remove(self.cursor_pos);
    }

    pub fn delete_word_back(&mut self) {
        if self.cursor_pos == 0 { return; }
        let before = &self.text[..self.cursor_pos];
        let trimmed = before.trim_end_matches(|c: char| c == ' ' || c == '\n');
        let new_end = trimmed.rfind(|c: char| c == ' ' || c == '\n')
            .map(|i| i + 1).unwrap_or(0);
        self.text.drain(new_end..self.cursor_pos);
        self.cursor_pos = new_end;
    }

    pub fn delete_to_line_start(&mut self) {
        let line_start = self.text[..self.cursor_pos]
            .rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.text.drain(line_start..self.cursor_pos);
        self.cursor_pos = line_start;
    }

    // ── movement ──────────────────────────────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.cursor_pos == 0 { return; }
        self.cursor_pos = prev_char_boundary(&self.text, self.cursor_pos);
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        let c = self.text[self.cursor_pos..].chars().next().unwrap();
        self.cursor_pos += c.len_utf8();
    }

    pub fn move_word_left(&mut self) {
        if self.cursor_pos == 0 { return; }
        let before = &self.text[..self.cursor_pos];
        let a = before.trim_end_matches(|c: char| c == ' ' || c == '\n').len();
        let b = self.text[..a].trim_end_matches(|c: char| c != ' ' && c != '\n').len();
        self.cursor_pos = b;
    }

    pub fn move_word_right(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        let after = &self.text[self.cursor_pos..];
        let word_end = after.len() - after.trim_start_matches(|c: char| c != ' ' && c != '\n').len();
        let rest = &after[word_end..];
        let space_end = rest.len() - rest.trim_start_matches(|c: char| c == ' ' || c == '\n').len();
        self.cursor_pos += word_end + space_end;
    }

    pub fn move_to_line_start(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let (row, _) = cursor_in_wrapped(&offsets, self.cursor_pos);
        self.cursor_pos = offsets[row];
    }

    pub fn move_to_line_end(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let lines = wrap_lines(&self.text, cw);
        let (row, _) = cursor_in_wrapped(&offsets, self.cursor_pos);
        let line_len = lines.get(row).map(|l| l.len()).unwrap_or(0);
        self.cursor_pos = offsets[row] + line_len;
    }

    pub fn move_up(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let (row, col) = cursor_in_wrapped(&offsets, self.cursor_pos);
        if row > 0 {
            self.cursor_pos = wrapped_to_cursor(&self.text, &offsets, row - 1, col, cw);
        }
    }

    pub fn move_down(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let (row, col) = cursor_in_wrapped(&offsets, self.cursor_pos);
        self.cursor_pos = wrapped_to_cursor(&self.text, &offsets, row + 1, col, cw);
    }
}

// ── key mapping ───────────────────────────────────────────────────────────────

pub enum Edit {
    InsertChar(char),
    Backspace,
    DeleteForward,
    DeleteWordBack,
    DeleteToLineStart,
    MoveLeft, MoveRight,
    MoveWordLeft, MoveWordRight,
    MoveUp, MoveDown,
    MoveLineStart, MoveLineEnd,
}

pub enum EditorAction {
    InsertNewline,
    Save,
    Cancel,
    Edit(Edit),
    None,
}

pub fn editor_action(key: KeyEvent) -> EditorAction {
    let ctrl  = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt   = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Enter if shift                 => EditorAction::InsertNewline,
        KeyCode::Enter                          => EditorAction::Save,
        KeyCode::Esc                            => EditorAction::Cancel,
        KeyCode::Char('w') if ctrl              => EditorAction::Edit(Edit::DeleteWordBack),
        KeyCode::Char('u') if ctrl              => EditorAction::Edit(Edit::DeleteToLineStart),
        KeyCode::Backspace if ctrl || alt       => EditorAction::Edit(Edit::DeleteWordBack),
        KeyCode::Backspace                      => EditorAction::Edit(Edit::Backspace),
        KeyCode::Delete                         => EditorAction::Edit(Edit::DeleteForward),
        KeyCode::Home                           => EditorAction::Edit(Edit::MoveLineStart),
        KeyCode::End                            => EditorAction::Edit(Edit::MoveLineEnd),
        KeyCode::Left  if alt                   => EditorAction::Edit(Edit::MoveWordLeft),
        KeyCode::Right if alt                   => EditorAction::Edit(Edit::MoveWordRight),
        KeyCode::Char('b') if alt               => EditorAction::Edit(Edit::MoveWordLeft),
        KeyCode::Char('f') if alt               => EditorAction::Edit(Edit::MoveWordRight),
        KeyCode::Left                           => EditorAction::Edit(Edit::MoveLeft),
        KeyCode::Right                          => EditorAction::Edit(Edit::MoveRight),
        KeyCode::Up                             => EditorAction::Edit(Edit::MoveUp),
        KeyCode::Down                           => EditorAction::Edit(Edit::MoveDown),
        KeyCode::Char(c) if !ctrl && !alt       => EditorAction::Edit(Edit::InsertChar(c)),
        _                                       => EditorAction::None,
    }
}

pub fn apply_edit(ed: &mut TextEditor, edit: Edit, cw: usize) {
    match edit {
        Edit::InsertChar(c)      => ed.insert_char(c),
        Edit::Backspace          => ed.backspace(),
        Edit::DeleteForward      => ed.delete_forward(),
        Edit::DeleteWordBack     => ed.delete_word_back(),
        Edit::DeleteToLineStart  => ed.delete_to_line_start(),
        Edit::MoveLeft           => ed.move_left(),
        Edit::MoveRight          => ed.move_right(),
        Edit::MoveWordLeft       => ed.move_word_left(),
        Edit::MoveWordRight      => ed.move_word_right(),
        Edit::MoveUp             => ed.move_up(cw),
        Edit::MoveDown           => ed.move_down(cw),
        Edit::MoveLineStart      => ed.move_to_line_start(cw),
        Edit::MoveLineEnd        => ed.move_to_line_end(cw),
    }
}
