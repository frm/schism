use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;
use crate::types::{FileStatus, LineKind};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        " Files",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (i, file) in app.files.iter().enumerate() {
        let is_selected = i == app.filetree_selected;
        let is_current = i == app.current_file_index();

        let (status_char, status_color) = match file.status {
            FileStatus::Added => ("A", Color::Green),
            FileStatus::Modified => ("M", Color::Yellow),
            FileStatus::Deleted => ("D", Color::Red),
            FileStatus::Renamed => ("R", Color::Cyan),
        };

        let has_comments = file.hunks.iter().any(|h| h.lines.iter().any(|l| l.comment.is_some()));
        let comment_marker = if has_comments { " ●" } else { "" };

        let filename = file.path.rsplit('/').next().unwrap_or(&file.path);
        let text = format!(" {} {}{}", status_char, filename, comment_marker);

        let mut style = Style::default().fg(status_color);
        if is_selected {
            style = style.bg(Color::Rgb(30, 30, 50));
        }
        if is_current {
            style = style.add_modifier(Modifier::BOLD);
        }

        lines.push(Line::from(Span::styled(text, style)));
    }

    // Stats at bottom
    let total = app.files.len();
    let added: usize = app.files.iter()
        .flat_map(|f| &f.hunks)
        .flat_map(|h| &h.lines)
        .filter(|l| l.kind == LineKind::Added)
        .count();
    let removed: usize = app.files.iter()
        .flat_map(|f| &f.hunks)
        .flat_map(|h| &h.lines)
        .filter(|l| l.kind == LineKind::Removed)
        .count();

    while lines.len() < area.height.saturating_sub(2) as usize {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        format!(" {} files", total),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(vec![
        Span::styled(format!(" +{}", added), Style::default().fg(Color::Green)),
        Span::styled(format!(" -{}", removed), Style::default().fg(Color::Red)),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}
