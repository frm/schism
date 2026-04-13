use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::render::line::LineRenderer;
use crate::tui::app::App;

pub fn render_file_header<'a>(app: &App, file_index: usize, bg: Color, width: usize) -> Line<'a> {
    let file = &app.files[file_index];
    let status_word = LineRenderer::status_word(&file.status);
    let (added, removed) = LineRenderer::file_stats(file);
    let fold = if file.collapsed { "▸" } else { "▾" };

    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };

    let mut spans = vec![
        Span::styled(format!(" {} ", fold),           Style::default().fg(Color::DarkGray).bg(bg)),
        Span::styled(format!("{} ", path_display),    Style::default().fg(Color::White).bg(bg).add_modifier(Modifier::BOLD)),
        Span::styled("· ".to_string(),                Style::default().fg(Color::DarkGray).bg(bg)),
        Span::styled(format!("{} ", status_word),     Style::default().fg(Color::DarkGray).bg(bg)),
        Span::styled("· ".to_string(),                Style::default().fg(Color::DarkGray).bg(bg)),
        Span::styled(format!("+{}", added),           Style::default().fg(Color::Green).bg(bg)),
        Span::styled(" ".to_string(),                 Style::default().bg(bg)),
        Span::styled(format!("-{}", removed),         Style::default().fg(Color::Red).bg(bg)),
    ];

    let used = 3 + path_display.len() + 1 + 2 + status_word.len() + 1 + 2
        + format!("+{}", added).len() + 1 + format!("-{}", removed).len();
    let remaining = width.saturating_sub(used);
    if remaining > 2 {
        spans.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
        spans.push(Span::styled("━".repeat(remaining - 1), Style::default().fg(Color::DarkGray).bg(bg)));
    }

    Line::from(spans)
}

pub fn render_hunk_header<'a>(
    app: &App,
    file_index: usize,
    hunk_index: usize,
    bg: Color,
    width: usize,
) -> Line<'a> {
    let hunk = &app.files[file_index].hunks[hunk_index];
    let (line_num, func_context) = LineRenderer::parse_hunk_context(&hunk.header);
    let frame_char = if hunk.collapsed { " ▸ " } else { " ╭ " };

    let mut spans = vec![
        Span::styled(frame_char.to_string(),       Style::default().fg(Color::DarkGray).bg(bg)),
        Span::styled(format!("L{}", line_num),     Style::default().fg(Color::Cyan).bg(bg)),
    ];
    if let Some(ctx) = func_context {
        spans.push(Span::styled(format!(" {}", ctx), Style::default().fg(Color::DarkGray).bg(bg)));
    }
    let used = 3 + format!("L{}", line_num).len() + func_context.map(|c| c.len() + 1).unwrap_or(0);
    spans.push(Span::styled(" ".repeat(width.saturating_sub(used)), Style::default().bg(bg)));

    Line::from(spans)
}
