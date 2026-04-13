use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::render::line::LineRenderer;
use crate::tui::app::App;
use crate::types::FileStatus;

const FG_DIM:      Color = Color::Rgb(120, 120, 120);
const FG_DIR:      Color = Color::Rgb(180, 180, 180);
const FG_FILE:     Color = Color::Rgb(200, 200, 200);
const FG_CURRENT:  Color = Color::Yellow;
const FG_CURSOR:   Color = Color::Cyan;
const FG_COMMENT:  Color = Color::Rgb(180, 140, 60);

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let height = area.height as usize;

    let current_path = app.files.get(app.current_file_index())
        .map(|f| f.path.as_str())
        .unwrap_or("");

    // Auto-scroll to keep cursor visible
    let mut scroll = app.filetree_scroll;
    if app.filetree_selected >= scroll + height.saturating_sub(1) {
        scroll = app.filetree_selected + 1 - height.max(1);
    }
    if app.filetree_selected < scroll {
        scroll = app.filetree_selected;
    }

    let visible = app.tree_flat
        .iter()
        .enumerate()
        .skip(scroll)
        .take(height);

    let mut lines: Vec<Line> = visible.map(|(abs_idx, node)| {
        let is_cursor  = abs_idx == app.filetree_selected;
        let is_current = !node.is_dir && node.file_index < app.files.len()
            && app.files[node.file_index].path == current_path;

        let indent = "  ".repeat(node.depth);

        let (icon, icon_color) = if node.is_dir {
            if node.expanded { ("▾ ", FG_DIR) } else { ("▸ ", FG_DIR) }
        } else {
            ("  ", FG_FILE)
        };

        let has_comments = !node.is_dir && node.file_index < app.files.len() && {
            let f = &app.files[node.file_index];
            f.hunks.iter().any(|h| h.lines.iter().any(|l| l.comment.is_some()))
        };

        let file_meta = if !node.is_dir && node.file_index < app.files.len() {
            let f = &app.files[node.file_index];
            let status_char = match f.status {
                FileStatus::Added    => Some(("A", Color::Green)),
                FileStatus::Deleted  => Some(("D", Color::Red)),
                FileStatus::Renamed  => Some(("R", Color::Cyan)),
                FileStatus::Modified => Some(("M", Color::Rgb(180, 140, 60))),
            };
            let (added, removed) = LineRenderer::file_stats(f);
            Some((status_char, added, removed))
        } else {
            None
        };

        let name_color = if is_current {
            FG_CURRENT
        } else if node.is_dir {
            FG_DIR
        } else {
            FG_FILE
        };

        let cursor_indicator = if is_cursor { "▶" } else { " " };
        let cursor_color = if is_cursor { FG_CURSOR } else { Color::Reset };

        let mut spans: Vec<Span> = vec![
            Span::styled(cursor_indicator.to_string(), Style::default().fg(cursor_color)),
            Span::raw(indent),
            Span::styled(icon.to_string(), Style::default().fg(icon_color)),
        ];

        let name_style = if is_current {
            Style::default().fg(name_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(name_color)
        };
        spans.push(Span::styled(node.path.split('/').last().unwrap_or(&node.path).to_string(), name_style));

        if let Some((status_char, added, removed)) = file_meta {
            if let Some((sc, sc_color)) = status_char {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(sc.to_string(), Style::default().fg(sc_color)));
            }
            spans.push(Span::styled(
                format!(" +{}", added),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::styled(
                format!(" -{}", removed),
                Style::default().fg(Color::Red),
            ));
        }

        if has_comments {
            spans.push(Span::styled(" ●".to_string(), Style::default().fg(FG_COMMENT)));
        }

        Line::from(spans)
    }).collect();

    // Pad remaining rows
    while lines.len() < height.saturating_sub(2) {
        lines.push(Line::from(""));
    }

    // Stats footer
    let total = app.files.len();
    let added: usize = app.files.iter()
        .flat_map(|f| &f.hunks)
        .flat_map(|h| &h.lines)
        .filter(|l| l.kind == crate::types::LineKind::Added)
        .count();
    let removed: usize = app.files.iter()
        .flat_map(|f| &f.hunks)
        .flat_map(|h| &h.lines)
        .filter(|l| l.kind == crate::types::LineKind::Removed)
        .count();

    lines.push(Line::from(Span::styled(
        format!(" {} files", total),
        Style::default().fg(FG_DIM),
    )));
    lines.push(Line::from(vec![
        Span::styled(format!(" +{}", added),  Style::default().fg(Color::Green)),
        Span::styled(format!(" -{}", removed), Style::default().fg(Color::Red)),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}
