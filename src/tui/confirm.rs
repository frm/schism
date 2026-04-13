use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::github::pr::{collect_review_comments, ReviewEvent};
use crate::tui::app::App;
use crate::tui::pr_description::render_markdown;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let event = match app.review_event {
        Some(e) => e,
        None => return,
    };

    let width  = (area.width as f32 * 0.6).max(50.0) as u16;
    let height = (area.height as f32 * 0.6).max(12.0) as u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);
    frame.render_widget(Clear, popup);

    let content_height = height.saturating_sub(2) as usize;
    let inner_width = width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    // Action badge
    let (label, bg) = match event {
        ReviewEvent::Comment => (" comment ", Color::Rgb(40, 40, 60)),
        ReviewEvent::Approve => (" approve ", Color::Rgb(0, 50, 0)),
        ReviewEvent::RequestChanges => (" request changes ", Color::Rgb(60, 30, 0)),
    };
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            label.to_string(),
            Style::default().fg(Color::White).bg(bg).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Review body
    let body = app.review_body.as_deref().unwrap_or("");
    if !body.is_empty() {
        let body_lines = render_markdown(body, inner_width.saturating_sub(2));
        for bl in body_lines {
            lines.push(bl);
        }
    } else {
        lines.push(Line::from(Span::styled(
            " (no review body)".to_string(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Separator
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(" {}", "─".repeat(inner_width.saturating_sub(2))),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    // Inline comments
    let comments = collect_review_comments(&app.files);
    if comments.is_empty() {
        lines.push(Line::from(Span::styled(
            " (no inline comments)".to_string(),
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!(" {} comment{}", comments.len(), if comments.len() == 1 { "" } else { "s" }),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

            for c in &comments {
            let side_color = if c.side == "RIGHT" { Color::Green } else { Color::Red };
            let side_char = if c.side == "RIGHT" { "+" } else { "-" };
            let line_bg = if c.side == "RIGHT" { Color::Rgb(0, 35, 0) } else { Color::Rgb(45, 0, 0) };

            // path:line
            lines.push(Line::from(vec![
                Span::styled(" ".to_string(), Style::default()),
                Span::styled(
                    c.path.clone(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(":{}", c.line),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // +/- line content
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} {}", side_char, c.content),
                    Style::default().fg(side_color).bg(line_bg),
                ),
            ]));

            // comment text (markdown rendered)
            let md_lines = render_markdown(&c.body, inner_width.saturating_sub(2));
            for md_line in md_lines {
                lines.push(md_line);
            }
            lines.push(Line::from(""));
        }
    }

    // Clamp to content height
    let visible: Vec<Line> = lines.into_iter().take(content_height).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(" Submit review ", Style::default().fg(Color::White)))
        .title_bottom(Span::styled(
            " Enter=submit · any key=cancel ",
            Style::default().fg(Color::DarkGray),
        ));

    frame.render_widget(Paragraph::new(visible).block(block), popup);
}
