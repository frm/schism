use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::render::line::LineRenderer;
use crate::tui::editor::{self, TextEditor};
use crate::types::DiffFile;

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

    pub fn file_index(&self) -> usize {
        match self.target {
            CommentTarget::Line { file_index, .. } => file_index,
            CommentTarget::File { file_index } => file_index,
        }
    }
}

// ── rendering ────────────────────────────────────────────────────────────────

pub fn render_input(input: &CommentInput, width: usize) -> Vec<Line<'static>> {
    render_editor_lines(&input.editor, width, " │  ✎ ", " │    ", BG_INPUT)
}

pub fn render_saved(text: &str, width: usize) -> Vec<Line<'static>> {
    let cw = width.saturating_sub(PREFIX_WIDTH);
    if cw == 0 { return vec![]; }

    editor::wrap_lines(text, cw).iter().enumerate().map(|(i, &line)| {
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

pub fn collect(files: &[DiffFile], body: Option<&str>) -> Option<String> {
    let mut comments = String::new();

    for file in files {
        // File-level comment
        if let Some(comment) = &file.comment {
            if !comments.is_empty() { comments.push('\n'); }
            comments.push_str(&format!("{}\n", file.path));
            comments.push_str(&comment.text);
            comments.push('\n');
        }

        // Line-level comments
        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
                    let prefix = LineRenderer::line_prefix(&line.kind);

                    if !comments.is_empty() { comments.push('\n'); }

                    comments.push_str(&format!("{}:{}\n", file.path, lineno));
                    comments.push_str(&format!("{}{}\n", prefix, line.content));
                    comments.push_str(&comment.text);
                    comments.push('\n');
                }
            }
        }
    }

    match (body, comments.is_empty()) {
        (None, true)           => None,
        (None, false)          => Some(comments),
        (Some(b), true)        => Some(b.to_string()),
        (Some(b), false)       => Some(format!("{}\n\n{}", b, comments)),
    }
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

    let lines = editor::wrap_lines(&ed.text, cw);
    let offsets = editor::wrapped_offsets(&ed.text, cw);
    let (cursor_row, cursor_col) = editor::cursor_in_wrapped(&offsets, ed.cursor_pos);

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
            let (cursor_span, after) = editor::render_cursor(line, cursor_col, bg);
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
