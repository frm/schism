use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::tui::editor::TextEditor;
use crate::tui::wrap;

const BG_INPUT: Color = Color::Rgb(30, 30, 20);
const BG_SAVED: Color = Color::Rgb(25, 25, 15);
const FG: Color = Color::Yellow;
pub const PREFIX_WIDTH: usize = 6; // " │  ✎ " / " │  ↳ " / " │    " are all 6 chars

#[derive(Debug, Clone)]
pub enum CommentTarget {
    Line { file_index: usize, hunk_index: usize, line_index: usize },
    File { file_index: usize },
}

pub struct CommentInput {
    pub editor: TextEditor,
    pub target: CommentTarget,
}

impl CommentInput {
    pub fn for_line(file_index: usize, hunk_index: usize, line_index: usize, existing: String) -> Self {
        Self { editor: TextEditor::with_text(existing), target: CommentTarget::Line { file_index, hunk_index, line_index } }
    }

    pub fn for_file(file_index: usize, existing: String) -> Self {
        Self { editor: TextEditor::with_text(existing), target: CommentTarget::File { file_index } }
    }

}

// ── rendering ────────────────────────────────────────────────────────────────

pub fn render_input(input: &CommentInput, width: usize) -> Vec<Line<'static>> {
    render_editor_lines(&input.editor, width, " │  ✎ ", " │    ", BG_INPUT)
}

pub fn render_saved(text: &str, width: usize) -> Vec<Line<'static>> {
    let cw = width.saturating_sub(PREFIX_WIDTH);
    if cw == 0 { return vec![]; }

    wrap::wrap_lines(text, cw).iter().enumerate().map(|(i, &line)| {
        let prefix = if i == 0 { " │  ↳ " } else { " │    " };
        let pad = cw.saturating_sub(line.chars().count());
        Line::from(vec![
            Span::styled(prefix.to_string(), Style::default().fg(FG).bg(BG_SAVED)),
            Span::styled(line.to_string(), Style::default().fg(Color::White).bg(BG_SAVED)),
            Span::styled(" ".repeat(pad), Style::default().bg(BG_SAVED)),
        ])
    }).collect()
}

// ── shared render helper (used by body.rs too) ────────────────────────────────

pub fn render_editor_lines(
    ed: &TextEditor,
    width: usize,
    first_prefix: &str,
    cont_prefix: &str,
    bg: Color,
) -> Vec<Line<'static>> {
    let cw = width.saturating_sub(PREFIX_WIDTH);
    if cw == 0 { return vec![]; }

    let lines = wrap::wrap_lines(&ed.text, cw);
    let offsets = wrap::wrapped_offsets(&ed.text, cw);
    let (cursor_row, cursor_col) = wrap::cursor_in_wrapped(&offsets, ed.cursor_pos);

    if lines.is_empty() {
        let pad = cw.saturating_sub(1);
        return vec![Line::from(vec![
            Span::styled(first_prefix.to_string(), Style::default().fg(FG).bg(bg)),
            Span::styled("█".to_string(), Style::default().fg(Color::White).bg(bg)
                .add_modifier(ratatui::style::Modifier::SLOW_BLINK)),
            Span::styled(" ".repeat(pad), Style::default().bg(bg)),
        ])];
    }

    lines.iter().enumerate().map(|(i, &line)| {
        let prefix = if i == 0 { first_prefix } else { cont_prefix };
        if i == cursor_row {
            let before = &line[..cursor_col.min(line.len())];
            let (cursor_span, after) = wrap::render_cursor(line, cursor_col, bg);
            let pad = cw.saturating_sub(before.chars().count() + 1 + after.chars().count());
            Line::from(vec![
                Span::styled(prefix.to_string(), Style::default().fg(FG).bg(bg)),
                Span::styled(before.to_string(), Style::default().fg(Color::White).bg(bg)),
                cursor_span,
                Span::styled(after.to_string(), Style::default().fg(Color::White).bg(bg)),
                Span::styled(" ".repeat(pad), Style::default().bg(bg)),
            ])
        } else {
            let pad = cw.saturating_sub(line.chars().count());
            Line::from(vec![
                Span::styled(prefix.to_string(), Style::default().fg(FG).bg(bg)),
                Span::styled(line.to_string(), Style::default().fg(Color::White).bg(bg)),
                Span::styled(" ".repeat(pad), Style::default().bg(bg)),
            ])
        }
    }).collect()
}
