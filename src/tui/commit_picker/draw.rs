use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let picker = match &app.commit_picker {
        Some(p) => p,
        None => return,
    };

    let width = (area.width as f32 * 0.7).max(40.0) as u16;
    let result_height = picker.filtered.len().max(1) as u16 + 1;
    let height = (result_height + 3).min(20).min(area.height - 2);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 3;

    let popup_area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(popup_area);

    // Search input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Find commit ");
    let input_line = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            &picker.query[..picker.cursor_pos],
            Style::default().fg(Color::White),
        ),
        Span::styled(
            "█",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(
            &picker.query[picker.cursor_pos..],
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(input_line).block(input_block), chunks[0]);

    // Results
    let max_results = chunks[1].height as usize;
    let result_lines: Vec<Line> = if picker.filtered.is_empty() {
        let inner_width = chunks[1].width.saturating_sub(2) as usize;
        let msg = "No commits";
        let pad = (inner_width.saturating_sub(msg.len())) / 2;
        vec![Line::from(Span::styled(
            format!("{}{}", " ".repeat(pad + 1), msg),
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        picker
            .filtered
            .iter()
            .take(max_results)
            .enumerate()
            .map(|(i, &ci)| {
                let commit = &picker.commits[ci];
                let is_selected = i == picker.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(30, 30, 50))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let sha_short = &commit.sha[..7.min(commit.sha.len())];
                Line::from(Span::styled(
                    format!(" {} · {} · {}", sha_short, commit.author, commit.message),
                    style,
                ))
            })
            .collect()
    };

    let results = Paragraph::new(result_lines).block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(results, chunks[1]);
}
