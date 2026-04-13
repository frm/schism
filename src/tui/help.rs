use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const KEYS: &[(&str, &str)] = &[
    ("Navigation",       ""),
    ("j / k",            "Move cursor"),
    ("J / K",            "Jump to next/prev file"),
    ("n / N",            "Next/prev hunk (or search match)"),
    ("gg / G",           "Top / bottom"),
    ("Ctrl+D / U",       "Half page down/up"),
    ("Ctrl+F / B",       "Full page down/up"),
    ("",                 ""),
    ("Folding",          ""),
    ("z / Space",        "Toggle fold hunk"),
    ("Z",                "Toggle fold file"),
    ("Tab",              "Toggle fold all hunks in file"),
    ("Shift+Tab",        "Toggle fold all files"),
    ("",                 ""),
    ("Commenting",       ""),
    ("c",                "Add/edit comment on line or file"),
    ("dd",               "Delete comment"),
    ("b",                "Edit review body"),
    ("",                 ""),
    ("File viewer",      ""),
    ("f",                "Open file (new version) / close"),
    ("F",                "Open file (old version)"),
    ("m",                "Toggle old/new in file view"),
    ("J / K",            "Next/prev file in file view"),
    ("",                 ""),
    ("Tools",            ""),
    ("t",                "Toggle file tree"),
    ("Ctrl+P",           "Fuzzy file finder"),
    ("/",                "Search"),
    ("?",                "Toggle this help"),
    ("",                 ""),
    ("Exit",             ""),
    ("Enter",            "Quit and output comments"),
    ("q / Esc",          "Quit silently"),
];

pub fn draw(frame: &mut Frame, area: Rect) {
    let width  = 52u16.min(area.width.saturating_sub(4));
    let height = (KEYS.len() as u16 + 2).min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup);

    let lines: Vec<Line> = KEYS.iter().map(|(key, desc)| {
        if desc.is_empty() && key.is_empty() {
            Line::from("")
        } else if desc.is_empty() {
            // Section header
            Line::from(Span::styled(
                format!(" {}", key),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(vec![
                Span::styled(
                    format!(" {:<18}", key),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    desc.to_string(),
                    Style::default().fg(Color::White),
                ),
            ])
        }
    }).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" ? keybindings ", Style::default().fg(Color::White)));

    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
