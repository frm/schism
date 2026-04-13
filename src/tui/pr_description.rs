use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::markdown;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let body = match &app.pr_context {
        Some(ctx) => &ctx.metadata.body,
        None => return,
    };

    let width  = (area.width as f32 * 0.7) as u16;
    let height = (area.height as f32 * 0.8) as u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);
    frame.render_widget(Clear, popup);

    let content_width = width.saturating_sub(4) as usize;
    let content_height = height.saturating_sub(2) as usize;
    let rendered = markdown::render(body, content_width);

    let max_scroll = rendered.len().saturating_sub(content_height);
    let start = app.pr_description_scroll.min(max_scroll);
    let end = (start + content_height).min(rendered.len());
    let visible: Vec<Line> = rendered[start..end].to_vec();

    let pct = if rendered.len() > content_height {
        format!(" {}% ", (start * 100 / max_scroll.max(1)).min(100))
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(" PR Description ", Style::default().fg(Color::White)))
        .title_bottom(Span::styled(pct, Style::default().fg(Color::DarkGray)));

    frame.render_widget(Paragraph::new(visible).block(block), popup);
}
