use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::types::FileStatus;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let finder = match &app.fuzzy_finder {
        Some(f) => f,
        None => return,
    };

    let width = (area.width as f32 * 0.6).max(30.0) as u16;
    let result_height = finder.matches.len().max(1) as u16 + 1;
    let height = (result_height + 3).min(15).min(area.height - 2);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 3;

    let popup_area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(popup_area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Find file ");
    let input_line = Line::from(vec![
        Span::raw(" "),
        Span::styled(&finder.query[..finder.cursor_pos], Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)),
        Span::styled(&finder.query[finder.cursor_pos..], Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(input_line).block(input_block), chunks[0]);

    let max_results = chunks[1].height as usize;
    let result_lines: Vec<Line> = if finder.matches.is_empty() {
        let inner_width = chunks[1].width.saturating_sub(2) as usize;
        let msg = "No results";
        let pad = (inner_width.saturating_sub(msg.len())) / 2;
        vec![Line::from(Span::styled(
            format!("{}{}", " ".repeat(pad + 1), msg),
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        finder.matches.iter().take(max_results).enumerate().map(|(i, m)| {
            let file = &app.files[m.file_index];
            let is_selected = i == finder.selected;
            let style = if is_selected {
                Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 50)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let status_char = match file.status {
                FileStatus::Added => "A",
                FileStatus::Modified => "M",
                FileStatus::Deleted => "D",
                FileStatus::Renamed => "R",
            };
            Line::from(Span::styled(format!(" {} {}", status_char, file.path), style))
        }).collect()
    };

    let results = Paragraph::new(result_lines).block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(results, chunks[1]);
}
