use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::render::line::LineRenderer;
use crate::types::DiffFile;

const BG_INPUT: Color = Color::Rgb(30, 30, 20);
const BG_SAVED: Color = Color::Rgb(25, 25, 15);
const FG: Color = Color::Yellow;
pub const PREFIX_WIDTH: usize = 6; // " │  ✎ " etc. are all 6 display chars

pub struct CommentInput {
    pub text: String,
    /// Byte offset into `text`.
    pub cursor_pos: usize,
    pub file_index: usize,
    pub hunk_index: usize,
    pub line_index: usize,
}

impl CommentInput {
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

    /// Forward delete (fn+Delete on macOS).
    pub fn delete_forward(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        self.text.remove(self.cursor_pos);
        // cursor_pos stays — the char at that offset is now the next one
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
        // skip spaces/newlines, then skip non-spaces
        let a = before.trim_end_matches(|c: char| c == ' ' || c == '\n').len();
        let b = self.text[..a].trim_end_matches(|c: char| c != ' ' && c != '\n').len();
        self.cursor_pos = b;
    }

    pub fn move_word_right(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        let after = &self.text[self.cursor_pos..];
        // skip non-spaces, then skip spaces/newlines
        let word_end = after.len() - after.trim_start_matches(|c: char| c != ' ' && c != '\n').len();
        let rest = &after[word_end..];
        let space_end = rest.len() - rest.trim_start_matches(|c: char| c == ' ' || c == '\n').len();
        self.cursor_pos += word_end + space_end;
    }

    pub fn move_to_line_start(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let (row, _col) = cursor_in_wrapped(&offsets, self.cursor_pos);
        self.cursor_pos = offsets[row];
    }

    pub fn move_to_line_end(&mut self, content_width: usize) {
        let cw = content_width.max(1);
        let offsets = wrapped_offsets(&self.text, cw);
        let lines = wrap_lines(&self.text, cw);
        let (row, _col) = cursor_in_wrapped(&offsets, self.cursor_pos);
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

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos - 1;
    while !s.is_char_boundary(p) { p -= 1; }
    p
}

/// Flat byte offsets for each wrapped visual line — port of TS `wrappedOffsets`.
fn wrapped_offsets(text: &str, width: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut flat_pos = 0usize;
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            offsets.push(flat_pos);
        } else {
            let mut i = 0;
            let chars: Vec<char> = log_line.chars().collect();
            while i < chars.len() {
                // byte offset of chars[i] within log_line
                let byte_i: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
                offsets.push(flat_pos + byte_i);
                i += width;
            }
        }
        flat_pos += log_line.len() + 1; // +1 for '\n'
    }
    offsets
}

/// (row, col) of `cursor` in wrapped lines.
fn cursor_in_wrapped(offsets: &[usize], cursor: usize) -> (usize, usize) {
    let mut row = 0;
    for (i, &off) in offsets.iter().enumerate().rev() {
        if cursor >= off { row = i; break; }
    }
    (row, cursor - offsets[row])
}

/// Flat cursor position for (target_row, target_col), clamped to line length.
fn wrapped_to_cursor(text: &str, offsets: &[usize], row: usize, col: usize, width: usize) -> usize {
    let lines = wrap_lines(text, width);
    let clamped_row = row.min(lines.len().saturating_sub(1));
    let line_len = lines.get(clamped_row).map(|l: &&str| l.len()).unwrap_or(0);
    let clamped_col = col.min(line_len);
    offsets.get(clamped_row).copied().unwrap_or(text.len()) + clamped_col
}

/// All wrapped visual lines as string slices — mirrors TS `wrapText`.
fn wrap_lines<'a>(text: &'a str, width: usize) -> Vec<&'a str> {
    let mut out = Vec::new();
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            out.push("");
        } else {
            let mut start = 0;
            let mut col = 0;
            for (i, c) in log_line.char_indices() {
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

// ── rendering ────────────────────────────────────────────────────────────────

pub fn render_input(input: &CommentInput, width: usize) -> Vec<Line<'static>> {
    let cw = width.saturating_sub(PREFIX_WIDTH);
    if cw == 0 { return vec![]; }

    let lines = wrap_lines(&input.text, cw);
    let offsets = wrapped_offsets(&input.text, cw);
    let (cursor_row, cursor_col) = cursor_in_wrapped(&offsets, input.cursor_pos);

    // Empty text: one line with just the cursor
    if lines.is_empty() {
        let pad = cw.saturating_sub(1);
        return vec![Line::from(vec![
            Span::styled(" │  ✎ ".to_string(), Style::default().fg(FG).bg(BG_INPUT)),
            Span::styled("█".to_string(), Style::default().fg(Color::White).bg(BG_INPUT).add_modifier(Modifier::SLOW_BLINK)),
            Span::styled(" ".repeat(pad), Style::default().bg(BG_INPUT)),
        ])];
    }

    lines.iter().enumerate().map(|(i, &line)| {
        let prefix = if i == 0 { " │  ✎ " } else { " │    " };
        if i == cursor_row {
            let before = &line[..cursor_col.min(line.len())];
            let after_start = cursor_col.min(line.len());
            let char_at = line[after_start..].chars().next();
            let (cursor_span, after) = match char_at {
                Some(c) => {
                    let cursor_ch = &line[after_start..after_start + c.len_utf8()];
                    let rest = &line[after_start + c.len_utf8()..];
                    (
                        Span::styled(cursor_ch.to_string(), Style::default().fg(BG_INPUT).bg(Color::White)),
                        rest,
                    )
                }
                None => (
                    Span::styled("█".to_string(), Style::default().fg(Color::White).bg(BG_INPUT).add_modifier(Modifier::SLOW_BLINK)),
                    "",
                ),
            };
            let pad = cw.saturating_sub(before.chars().count() + 1 + after.chars().count());
            Line::from(vec![
                Span::styled(prefix.to_string(), Style::default().fg(FG).bg(BG_INPUT)),
                Span::styled(before.to_string(), Style::default().fg(Color::White).bg(BG_INPUT)),
                cursor_span,
                Span::styled(after.to_string(), Style::default().fg(Color::White).bg(BG_INPUT)),
                Span::styled(" ".repeat(pad), Style::default().bg(BG_INPUT)),
            ])
        } else {
            let pad = cw.saturating_sub(line.chars().count());
            Line::from(vec![
                Span::styled(prefix.to_string(), Style::default().fg(FG).bg(BG_INPUT)),
                Span::styled(line.to_string(), Style::default().fg(Color::White).bg(BG_INPUT)),
                Span::styled(" ".repeat(pad), Style::default().bg(BG_INPUT)),
            ])
        }
    }).collect()
}

pub fn render_saved(text: &str, width: usize) -> Vec<Line<'static>> {
    let cw = width.saturating_sub(PREFIX_WIDTH);
    if cw == 0 { return vec![]; }

    wrap_lines(text, cw).iter().enumerate().map(|(i, &line)| {
        let prefix = if i == 0 { " │  ↳ " } else { " │    " };
        let pad = cw.saturating_sub(line.chars().count());
        Line::from(vec![
            Span::styled(prefix.to_string(), Style::default().fg(FG).bg(BG_SAVED)),
            Span::styled(line.to_string(), Style::default().fg(Color::White).bg(BG_SAVED)),
            Span::styled(" ".repeat(pad), Style::default().bg(BG_SAVED)),
        ])
    }).collect()
}

// ── export ────────────────────────────────────────────────────────────────────

pub fn collect(files: &[DiffFile]) -> Option<String> {
    let mut output = String::new();

    for file in files {
        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
                    let prefix = LineRenderer::line_prefix(&line.kind);

                    if !output.is_empty() { output.push('\n'); }

                    output.push_str(&format!("{}:{}\n", file.path, lineno));
                    output.push_str(&format!("{}{}\n", prefix, line.content));
                    output.push_str(&comment.text);
                    output.push('\n');
                }
            }
        }
    }

    if output.is_empty() { None } else { Some(output) }
}
