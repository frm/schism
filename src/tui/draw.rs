use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::tui::app::{App, Row};
use crate::tui::comment;
use crate::types::LineKind;

pub fn draw(frame: &mut Frame, app: &App, highlighter: &Highlighter) {
    let area = frame.area();

    if app.show_filetree {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)])
            .split(area);

        crate::tui::filetree::draw(frame, app, chunks[0]);
        draw_viewport(frame, app, highlighter, chunks[1]);
    } else {
        draw_viewport(frame, app, highlighter, area);
    }

    // Overlays
    if app.fuzzy_finder.is_some() {
        crate::tui::fuzzy::draw(frame, app, area);
    }
    if let Some(ref body) = app.body_editor {
        crate::tui::body::draw(frame, body, area);
    }
    if let Some(ref fv) = app.file_view {
        crate::tui::fileview::draw(frame, fv, app, highlighter);
    }
    if app.show_help {
        crate::tui::help::draw(frame, area);
    }
}

fn draw_viewport(frame: &mut Frame, app: &App, highlighter: &Highlighter, area: Rect) {
    // Reserve the last row for the search bar when active
    let search_bar_height = if app.search.is_some() { 1usize } else { 0 };
    let width = area.width as usize;
    let visible_rows = area.height as usize - search_bar_height;
    let end = (app.scroll_offset + visible_rows).min(app.rows.len());

    let comment_row_idx = app.comment_input.as_ref().and_then(|c| {
        use crate::tui::comment::CommentTarget;
        app.rows.iter().position(|r| match (&c.target, r) {
            (CommentTarget::Line { file_index: fi, hunk_index: hi, line_index: li },
             Row::Line { file_index, hunk_index, line_index }) =>
                fi == file_index && hi == hunk_index && li == line_index,
            (CommentTarget::File { file_index: fi }, Row::FileHeader { file_index }) =>
                fi == file_index,
            _ => false,
        })
    });

    let mut lines: Vec<Line> = Vec::new();

    for i in app.scroll_offset..end {
        if lines.len() >= visible_rows {
            break;
        }
        let is_cursor = i == app.cursor;
        let is_match = app.search.as_ref()
            .filter(|s| !s.active_input && !s.query.is_empty())
            .map(|s| s.matches.contains(&i))
            .unwrap_or(false);
        let is_current_match = app.search.as_ref()
            .filter(|s| !s.active_input)
            .and_then(|s| s.matches.get(s.current))
            .map(|&m| m == i)
            .unwrap_or(false);
        let query = app.search.as_ref()
            .filter(|s| !s.active_input && !s.query.is_empty())
            .map(|s| s.query.as_str());
        let line = render_row(&app.rows[i], app, highlighter, is_cursor, is_match, is_current_match, query, width);
        lines.push(line);

        // Render active comment input or saved comment below this row
        let active_input = app.comment_input.as_ref().filter(|_| Some(i) == comment_row_idx);
        match &app.rows[i] {
            Row::FileHeader { file_index } => {
                let fi = *file_index;
                if let Some(ref input) = active_input {
                    for cl in comment::render_input(input, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                } else if let Some(ref c) = app.files[fi].comment {
                    for cl in comment::render_saved(&c.text, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                }
            }
            Row::Line { file_index, hunk_index, line_index } => {
                let diff_line = &app.files[*file_index].hunks[*hunk_index].lines[*line_index];
                if let Some(ref input) = active_input {
                    for cl in comment::render_input(input, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                } else if let Some(ref c) = diff_line.comment {
                    for cl in comment::render_saved(&c.text, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                }
            }
            _ => {}
        }
    }

    let content_area = if search_bar_height > 0 {
        Rect::new(area.x, area.y, area.width, area.height - 1)
    } else {
        area
    };
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, content_area);

    // Search bar
    if let Some(ref s) = app.search {
        let bar_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let text = if s.active_input {
            format!("/{}", s.query)
        } else if s.matches.is_empty() {
            format!("/{} [no matches]", s.query)
        } else {
            format!("/{} [{}/{}]", s.query, s.current + 1, s.matches.len())
        };
        let style = Style::default().fg(Color::White).bg(Color::Reset);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" {}", text), style))),
            bar_area,
        );
    }
}

fn row_bg(is_cursor: bool, is_match: bool, is_current_match: bool) -> Color {
    if is_cursor           { Color::Rgb(30, 30, 50) }
    else if is_current_match { Color::Rgb(60, 50, 0) }   // bright gold for current
    else if is_match       { Color::Rgb(35, 30, 0) }      // dim gold for other matches
    else                   { Color::Reset }
}

fn render_row<'a>(
    row: &Row,
    app: &App,
    highlighter: &Highlighter,
    is_cursor: bool,
    is_match: bool,
    is_current_match: bool,
    match_query: Option<&'a str>,
    width: usize,
) -> Line<'a> {
    let bg = row_bg(is_cursor, is_match, is_current_match);
    match row {
        Row::FileHeader { file_index } => render_file_header(app, *file_index, bg, width),
        Row::HunkHeader { file_index, hunk_index } => {
            render_hunk_header(app, *file_index, *hunk_index, bg, width)
        }
        Row::Line { file_index, hunk_index, line_index } => {
            let query = if is_match || is_current_match { match_query } else { None };
            render_diff_line(app, highlighter, *file_index, *hunk_index, *line_index, bg, query, width)
        }
    }
}

fn render_file_header<'a>(app: &App, file_index: usize, bg: Color, width: usize) -> Line<'a> {
    let file = &app.files[file_index];
    let status_word = LineRenderer::status_word(&file.status);
    let (added, removed) = LineRenderer::file_stats(file);
    let fold = if file.collapsed { "▸" } else { "▾" };

    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        format!(" {} ", fold),
        Style::default().fg(Color::DarkGray).bg(bg),
    ));
    spans.push(Span::styled(
        format!("{} ", path_display),
        Style::default().fg(Color::White).bg(bg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        "· ".to_string(),
        Style::default().fg(Color::DarkGray).bg(bg),
    ));
    spans.push(Span::styled(
        format!("{} ", status_word),
        Style::default().fg(Color::DarkGray).bg(bg),
    ));
    spans.push(Span::styled(
        "· ".to_string(),
        Style::default().fg(Color::DarkGray).bg(bg),
    ));
    spans.push(Span::styled(
        format!("+{}", added),
        Style::default().fg(Color::Green).bg(bg),
    ));
    spans.push(Span::styled(
        " ".to_string(),
        Style::default().bg(bg),
    ));
    spans.push(Span::styled(
        format!("-{}", removed),
        Style::default().fg(Color::Red).bg(bg),
    ));

    // Fill remaining width with separator character
    // " ▾ " (3) + path + " " (1) + "· " (2) + status + " " (1) + "· " (2) + +N + " " (1) + -N
    let used = 3 + path_display.len() + 1 + 2 + status_word.len() + 1 + 2
        + format!("+{}", added).len() + 1 + format!("-{}", removed).len();
    let remaining = width.saturating_sub(used);
    if remaining > 2 {
        spans.push(Span::styled(
            " ".to_string(),
            Style::default().bg(bg),
        ));
        spans.push(Span::styled(
            "━".repeat(remaining - 1),
            Style::default().fg(Color::DarkGray).bg(bg),
        ));
    }

    Line::from(spans)
}

fn render_hunk_header<'a>(
    app: &App,
    file_index: usize,
    hunk_index: usize,
    bg: Color,
    width: usize,
) -> Line<'a> {
    let hunk = &app.files[file_index].hunks[hunk_index];
    let (line_num, func_context) = LineRenderer::parse_hunk_context(&hunk.header);
    let frame_char = if hunk.collapsed { " ▸ " } else { " ╭ " };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        frame_char.to_string(),
        Style::default().fg(Color::DarkGray).bg(bg),
    ));
    spans.push(Span::styled(
        format!("L{}", line_num),
        Style::default().fg(Color::Cyan).bg(bg),
    ));
    if let Some(ctx) = func_context {
        spans.push(Span::styled(
            format!(" {}", ctx),
            Style::default().fg(Color::DarkGray).bg(bg),
        ));
    }

    let used: usize = 3 + format!("L{}", line_num).len()
        + func_context.map(|c| c.len() + 1).unwrap_or(0);
    let remaining = width.saturating_sub(used);
    spans.push(Span::styled(
        " ".repeat(remaining),
        Style::default().bg(bg),
    ));

    Line::from(spans)
}

/// Slightly darken a background colour for the inline match highlight.
fn match_highlight_bg(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(25),
            g.saturating_add(20),
            b.saturating_add(10),
        ),
        Color::Reset => Color::Rgb(50, 45, 20),
        other => other,
    }
}

fn render_diff_line<'a>(
    app: &App,
    highlighter: &Highlighter,
    file_index: usize,
    hunk_index: usize,
    line_index: usize,
    bg: Color,
    match_query: Option<&str>,
    width: usize,
) -> Line<'a> {
    let file = &app.files[file_index];
    let diff_line = &file.hunks[hunk_index].lines[line_index];
    let ext = Highlighter::extension_from_path(&file.path);

    let lineno_width = 4;
    let old_no = LineRenderer::format_lineno(diff_line.old_lineno, lineno_width);
    let new_no = LineRenderer::format_lineno(diff_line.new_lineno, lineno_width);
    let prefix = LineRenderer::line_prefix(&diff_line.kind);

    // Blend line kind colour with the row background (cursor/match/etc)
    let line_bg = match (&diff_line.kind, bg) {
        (LineKind::Added,   Color::Rgb(30, 30, 50)) => Color::Rgb(0, 60, 0),
        (LineKind::Added,   Color::Reset)            => Color::Rgb(0, 35, 0),
        (LineKind::Added,   other)                   => other,
        (LineKind::Removed, Color::Rgb(30, 30, 50)) => Color::Rgb(70, 0, 0),
        (LineKind::Removed, Color::Reset)            => Color::Rgb(45, 0, 0),
        (LineKind::Removed, other)                   => other,
        (LineKind::Context, other)                   => other,
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

    // Syntax-highlighted content, with optional inline match highlight
    let hl_spans = highlighter.highlight_line(&diff_line.content, ext);
    let content = &diff_line.content;
    // Find the byte range of the first case-insensitive match
    let match_range: Option<(usize, usize)> = match_query.and_then(|q| {
        let lc = content.to_lowercase();
        let lq = q.to_lowercase();
        lc.find(&lq).map(|start| (start, start + lq.len()))
    });
    let match_bg = match_range.map(|_| match_highlight_bg(line_bg));

    let mut content_len = 0;
    let mut byte_pos = 0usize;
    for span in hl_spans {
        let fg = Color::Rgb(span.fg.0, span.fg.1, span.fg.2);
        let span_start = byte_pos;
        let span_end   = byte_pos + span.text.len();
        byte_pos = span_end;
        content_len += span.text.len();

        let base_style = |bg_c: Color| {
            let mut s = Style::default().fg(fg).bg(bg_c);
            if span.bold   { s = s.add_modifier(Modifier::BOLD); }
            if span.italic { s = s.add_modifier(Modifier::ITALIC); }
            s
        };

        match (match_range, match_bg) {
            (Some((ms, me)), Some(mbg)) if span_end > ms && span_start < me => {
                // This syntect span overlaps the match — split into up to 3 segments
                let pre_end  = ms.clamp(span_start, span_end);
                let post_start = me.clamp(span_start, span_end);

                if pre_end > span_start {
                    spans.push(Span::styled(
                        span.text[span_start - span_start..pre_end - span_start].to_string(),
                        base_style(line_bg),
                    ));
                }
                if post_start > pre_end {
                    spans.push(Span::styled(
                        span.text[pre_end - span_start..post_start - span_start].to_string(),
                        base_style(mbg),
                    ));
                }
                if post_start < span_end {
                    spans.push(Span::styled(
                        span.text[post_start - span_start..].to_string(),
                        base_style(line_bg),
                    ));
                }
            }
            _ => spans.push(Span::styled(span.text, base_style(line_bg))),
        }
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


