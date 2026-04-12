use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::render::line::LineRenderer;
use crate::tui::app::App;
use crate::types::DiffFile;

pub struct CommentInput {
    pub text: String,
    pub cursor_pos: usize,
    pub file_index: usize,
    pub hunk_index: usize,
    pub line_index: usize,
}

pub fn render_input<'a>(input: &CommentInput, width: usize) -> Line<'a> {
    let prefix = " │  ✎ ";
    let text = format!("{}{}", prefix, input.text);
    let cursor = "█";
    let remaining = width.saturating_sub(text.chars().count() + 1);

    Line::from(vec![
        Span::styled(text, Style::default().fg(Color::Yellow).bg(Color::Rgb(30, 30, 20))),
        Span::styled(
            cursor.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(30, 30, 20))
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(" ".repeat(remaining), Style::default().bg(Color::Rgb(30, 30, 20))),
    ])
}

pub fn render_saved<'a>(text: &str, width: usize) -> Line<'a> {
    let prefix = " │  ↳ ";
    let content = format!("{}{}", prefix, text);
    let remaining = width.saturating_sub(content.chars().count());

    Line::from(vec![
        Span::styled(content, Style::default().fg(Color::Yellow).bg(Color::Rgb(25, 25, 15))),
        Span::styled(" ".repeat(remaining), Style::default().bg(Color::Rgb(25, 25, 15))),
    ])
}

pub fn collect(files: &[DiffFile]) -> Option<String> {
    let mut output = String::new();

    for file in files {
        for hunk in &file.hunks {
            for line in &hunk.lines {
                if let Some(comment) = &line.comment {
                    let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
                    let prefix = LineRenderer::line_prefix(&line.kind);

                    if !output.is_empty() {
                        output.push('\n');
                    }

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
