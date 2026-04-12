use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::tui::app::{App, Row};
use crate::types::LineKind;

pub fn draw(frame: &mut Frame, app: &App, highlighter: &Highlighter) {
    let area = frame.area();
    let width = area.width as usize;
    let visible_rows = area.height as usize;
    let end = (app.scroll_offset + visible_rows).min(app.rows.len());

    let mut lines: Vec<Line> = Vec::new();

    for i in app.scroll_offset..end {
        let is_cursor = i == app.cursor;
        let line = render_row(&app.rows[i], app, highlighter, is_cursor, width);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_row<'a>(
    row: &Row,
    app: &App,
    highlighter: &Highlighter,
    is_cursor: bool,
    width: usize,
) -> Line<'a> {
    match row {
        Row::FileHeader { file_index } => render_file_header(app, *file_index, is_cursor, width),
        Row::HunkHeader { file_index, hunk_index } => {
            render_hunk_header(app, *file_index, *hunk_index, is_cursor, width)
        }
        Row::Line { file_index, hunk_index, line_index } => {
            render_diff_line(app, highlighter, *file_index, *hunk_index, *line_index, is_cursor, width)
        }
    }
}

fn render_file_header<'a>(app: &App, file_index: usize, is_cursor: bool, width: usize) -> Line<'a> {
    let file = &app.files[file_index];
    let status_word = LineRenderer::status_word(&file.status);
    let (added, removed) = LineRenderer::file_stats(file);

    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };

    let cursor_bg = if is_cursor { Color::Rgb(30, 30, 50) } else { Color::Reset };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        format!(" {} ", path_display),
        Style::default().fg(Color::White).bg(cursor_bg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        "· ".to_string(),
        Style::default().fg(Color::DarkGray).bg(cursor_bg),
    ));
    spans.push(Span::styled(
        format!("{} ", status_word),
        Style::default().fg(Color::DarkGray).bg(cursor_bg),
    ));
    spans.push(Span::styled(
        "· ".to_string(),
        Style::default().fg(Color::DarkGray).bg(cursor_bg),
    ));
    spans.push(Span::styled(
        format!("+{}", added),
        Style::default().fg(Color::Green).bg(cursor_bg),
    ));
    spans.push(Span::styled(
        " ".to_string(),
        Style::default().bg(cursor_bg),
    ));
    spans.push(Span::styled(
        format!("-{}", removed),
        Style::default().fg(Color::Red).bg(cursor_bg),
    ));

    // Fill remaining width with separator character
    let used = 1 + path_display.len() + 1 + 2 + status_word.len() + 1 + 2
        + format!("+{}", added).len() + 1 + format!("-{}", removed).len();
    let remaining = width.saturating_sub(used);
    if remaining > 2 {
        spans.push(Span::styled(
            " ".to_string(),
            Style::default().bg(cursor_bg),
        ));
        spans.push(Span::styled(
            "━".repeat(remaining - 1),
            Style::default().fg(Color::DarkGray).bg(cursor_bg),
        ));
    }

    Line::from(spans)
}

fn render_hunk_header<'a>(
    app: &App,
    file_index: usize,
    hunk_index: usize,
    is_cursor: bool,
    width: usize,
) -> Line<'a> {
    let hunk = &app.files[file_index].hunks[hunk_index];
    let (line_num, func_context) = LineRenderer::parse_hunk_context(&hunk.header);
    let cursor_bg = if is_cursor { Color::Rgb(30, 30, 50) } else { Color::Reset };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " ╭ ".to_string(),
        Style::default().fg(Color::DarkGray).bg(cursor_bg),
    ));
    spans.push(Span::styled(
        format!("L{}", line_num),
        Style::default().fg(Color::Cyan).bg(cursor_bg),
    ));
    if let Some(ctx) = func_context {
        spans.push(Span::styled(
            format!(" {}", ctx),
            Style::default().fg(Color::DarkGray).bg(cursor_bg),
        ));
    }

    let used: usize = 3 + format!("L{}", line_num).len()
        + func_context.map(|c| c.len() + 1).unwrap_or(0);
    let remaining = width.saturating_sub(used);
    spans.push(Span::styled(
        " ".repeat(remaining),
        Style::default().bg(cursor_bg),
    ));

    Line::from(spans)
}

fn render_diff_line<'a>(
    app: &App,
    highlighter: &Highlighter,
    file_index: usize,
    hunk_index: usize,
    line_index: usize,
    is_cursor: bool,
    width: usize,
) -> Line<'a> {
    let file = &app.files[file_index];
    let diff_line = &file.hunks[hunk_index].lines[line_index];
    let ext = Highlighter::extension_from_path(&file.path);

    let lineno_width = 4;
    let old_no = LineRenderer::format_lineno(diff_line.old_lineno, lineno_width);
    let new_no = LineRenderer::format_lineno(diff_line.new_lineno, lineno_width);
    let prefix = LineRenderer::line_prefix(&diff_line.kind);

    let line_bg = match (diff_line.kind.clone(), is_cursor) {
        (LineKind::Added, true) => Color::Rgb(0, 60, 0),
        (LineKind::Added, false) => Color::Rgb(0, 35, 0),
        (LineKind::Removed, true) => Color::Rgb(70, 0, 0),
        (LineKind::Removed, false) => Color::Rgb(45, 0, 0),
        (LineKind::Context, true) => Color::Rgb(30, 30, 50),
        (LineKind::Context, false) => Color::Reset,
    };

    let comment_marker = if diff_line.comment.is_some() { "●" } else { " " };

    let mut spans = Vec::new();

    // Frame
    spans.push(Span::styled(
        " │ ".to_string(),
        Style::default().fg(Color::DarkGray),
    ));

    // Comment marker
    spans.push(Span::styled(
        format!("{} ", comment_marker),
        Style::default().fg(Color::Yellow).bg(line_bg),
    ));

    // Line numbers
    spans.push(Span::styled(
        old_no.clone(),
        Style::default().fg(Color::DarkGray).bg(line_bg),
    ));
    spans.push(Span::styled(
        new_no.clone(),
        Style::default().fg(Color::DarkGray).bg(line_bg),
    ));

    // Prefix
    let prefix_color = match diff_line.kind {
        LineKind::Added => Color::Green,
        LineKind::Removed => Color::Red,
        LineKind::Context => Color::DarkGray,
    };
    spans.push(Span::styled(
        prefix.to_string(),
        Style::default().fg(prefix_color).bg(line_bg),
    ));

    // Syntax-highlighted content
    let hl_spans = highlighter.highlight_line(&diff_line.content, ext);
    let mut content_len = 0;
    for span in hl_spans {
        let fg = Color::Rgb(span.fg.0, span.fg.1, span.fg.2);
        let mut style = Style::default().fg(fg).bg(line_bg);
        if span.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if span.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        content_len += span.text.len();
        spans.push(Span::styled(span.text, style));
    }

    // Fill remaining width with background
    let used = 3 + 2 + lineno_width + 1 + lineno_width + 1 + 1 + content_len;
    let remaining = width.saturating_sub(used);
    if remaining > 0 {
        spans.push(Span::styled(
            " ".repeat(remaining),
            Style::default().bg(line_bg),
        ));
    }

    Line::from(spans)
}

pub fn collect_comments(app: &App) -> Option<String> {
    let mut output = String::new();

    for file in &app.files {
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
