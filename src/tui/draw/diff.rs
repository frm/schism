use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::types::LineKind;

/// Slightly lighten a background colour for the inline match highlight.
pub fn match_highlight_bg(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b) => Color::Rgb(r.saturating_add(25), g.saturating_add(20), b.saturating_add(10)),
        Color::Reset => Color::Rgb(50, 45, 20),
        other => other,
    }
}

pub fn render_diff_line<'a>(
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
    let prefix_color = match diff_line.kind {
        LineKind::Added   => Color::Green,
        LineKind::Removed => Color::Red,
        LineKind::Context => Color::DarkGray,
    };

    let mut spans = vec![
        Span::styled(" │ ".to_string(),              Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", comment_marker), Style::default().fg(Color::Yellow).bg(line_bg)),
        Span::styled(old_no,                         Style::default().fg(Color::DarkGray).bg(line_bg)),
        Span::styled(new_no,                         Style::default().fg(Color::DarkGray).bg(line_bg)),
        Span::styled(prefix.to_string(),             Style::default().fg(prefix_color).bg(line_bg)),
    ];

    let content = &diff_line.content;
    let match_range: Option<(usize, usize)> = match_query.and_then(|q| {
        let lc = content.to_lowercase();
        let lq = q.to_lowercase();
        lc.find(&lq).map(|start| (start, start + lq.len()))
    });
    let match_bg = match_range.map(|_| match_highlight_bg(line_bg));

    let mut content_len = 0;
    let mut byte_pos = 0usize;
    for span in highlighter.highlight_line(content, ext) {
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
                let pre_end    = ms.clamp(span_start, span_end);
                let post_start = me.clamp(span_start, span_end);
                if pre_end > span_start {
                    spans.push(Span::styled(span.text[..pre_end - span_start].to_string(), base_style(line_bg)));
                }
                if post_start > pre_end {
                    spans.push(Span::styled(span.text[pre_end - span_start..post_start - span_start].to_string(), base_style(mbg)));
                }
                if post_start < span_end {
                    spans.push(Span::styled(span.text[post_start - span_start..].to_string(), base_style(line_bg)));
                }
            }
            _ => spans.push(Span::styled(span.text, base_style(line_bg))),
        }
    }

    let used = 3 + 2 + lineno_width + 1 + lineno_width + 1 + 1 + content_len;
    if width.saturating_sub(used) > 0 {
        spans.push(Span::styled(" ".repeat(width.saturating_sub(used)), Style::default().bg(line_bg)));
    }

    Line::from(spans)
}
