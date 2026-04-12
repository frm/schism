/// Generic text editor state — shared by inline comments and the body overlay.
pub struct TextEditor {
    pub text: String,
    /// Byte offset into `text`.
    pub cursor_pos: usize,
}

impl TextEditor {
    pub fn new() -> Self {
        Self { text: String::new(), cursor_pos: 0 }
    }

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
            .map(|i| i + 1)
            .unwrap_or(0);
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

// ── wrap helpers ──────────────────────────────────────────────────────────────

pub fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos - 1;
    while !s.is_char_boundary(p) { p -= 1; }
    p
}

/// Flat byte offset of the start of each wrapped visual line.
pub fn wrapped_offsets(text: &str, width: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut flat_pos = 0usize;
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            offsets.push(flat_pos);
        } else {
            let chars: Vec<char> = log_line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let byte_i: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
                offsets.push(flat_pos + byte_i);
                i += width;
            }
        }
        flat_pos += log_line.len() + 1;
    }
    offsets
}

/// `(row, col)` of `cursor` within wrapped lines.
pub fn cursor_in_wrapped(offsets: &[usize], cursor: usize) -> (usize, usize) {
    let mut row = 0;
    for (i, &off) in offsets.iter().enumerate().rev() {
        if cursor >= off { row = i; break; }
    }
    (row, cursor - offsets[row])
}

/// Byte offset for `(target_row, target_col)`, clamped to line length.
pub fn wrapped_to_cursor(text: &str, offsets: &[usize], row: usize, col: usize, width: usize) -> usize {
    let lines = wrap_lines(text, width);
    let clamped_row = row.min(lines.len().saturating_sub(1));
    let line_len = lines.get(clamped_row).map(|l: &&str| l.len()).unwrap_or(0);
    offsets.get(clamped_row).copied().unwrap_or(text.len()) + col.min(line_len)
}

/// All wrapped visual lines as string slices.
pub fn wrap_lines<'a>(text: &'a str, width: usize) -> Vec<&'a str> {
    let mut out = Vec::new();
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            out.push("");
        } else {
            let mut start = 0;
            let mut col = 0;
            for (i, _c) in log_line.char_indices() {
                if col == width {
                    out.push(&log_line[start..i]);
                    start = i;
                    col = 0;
                }
                col += 1;
            }
            out.push(&log_line[start..]);
        }
    }
    out
}

/// Render the cursor span for a given line at the given column.
/// Returns `(cursor_span, after_str)`.
pub fn render_cursor<'a>(
    line: &'a str,
    col: usize,
    bg: ratatui::style::Color,
) -> (ratatui::text::Span<'static>, &'a str) {
    let after_start = col.min(line.len());
    let char_at = line[after_start..].chars().next();
    match char_at {
        Some(c) => {
            let cursor_ch = line[after_start..after_start + c.len_utf8()].to_string();
            let rest = &line[after_start + c.len_utf8()..];
            (
                ratatui::text::Span::styled(
                    cursor_ch,
                    ratatui::style::Style::default().fg(bg).bg(ratatui::style::Color::White),
                ),
                rest,
            )
        }
        None => (
            ratatui::text::Span::styled(
                "█".to_string(),
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White)
                    .bg(bg)
                    .add_modifier(ratatui::style::Modifier::SLOW_BLINK),
            ),
            "",
        ),
    }
}
