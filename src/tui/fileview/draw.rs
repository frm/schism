use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::tui::fileview::{FileView, SCROLLOFF};
use crate::tui::fileview::fetch::{changed_lines_new, changed_lines_old};

pub fn draw(frame: &mut Frame, fv: &FileView, app: &App, highlighter: &Highlighter) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let file           = &app.files[fv.file_index];
    let ext            = Highlighter::extension_from_path(&file.path);
    let changed        = if fv.showing_new { changed_lines_new(file) } else { changed_lines_old(file) };
    let width          = area.width as usize;
    let height         = area.height as usize;
    let content_height = FileView::content_height(height);

    // ── header ────────────────────────────────────────────────────────────────
    let (added, removed) = LineRenderer::file_stats(file);
    let status_word      = LineRenderer::status_word(&file.status);
    let version          = if fv.showing_new { "new" } else { "old" };
    let version_color    = if fv.showing_new { Color::Green } else { Color::Red };

    let pct = match &fv.content {
        Some(c) if c.len() > content_height => {
            let max = c.len().saturating_sub(content_height.saturating_sub(SCROLLOFF));
            format!(" {}%", (fv.scroll * 100 / max.max(1)).min(100))
        }
        _ => String::new(),
    };

    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };
    let used = 3 + path_display.len() + 1
        + 2 + status_word.len() + 1
        + 2 + format!("+{}", added).len() + 1 + format!("-{}", removed).len()
        + 2 + version.len() + 1
        + pct.len();
    let remaining = width.saturating_sub(used);

    let mut header_spans = vec![
        Span::styled(" ▾ ".to_string(),            Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", path_display), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("· ".to_string(),             Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", status_word),  Style::default().fg(Color::DarkGray)),
        Span::styled("· ".to_string(),             Style::default().fg(Color::DarkGray)),
        Span::styled(format!("+{}", added),        Style::default().fg(Color::Green)),
        Span::styled(" ".to_string(),              Style::default()),
        Span::styled(format!("-{}", removed),      Style::default().fg(Color::Red)),
        Span::styled(" [".to_string(),             Style::default().fg(Color::DarkGray)),
        Span::styled(version.to_string(),          Style::default().fg(version_color).add_modifier(Modifier::BOLD)),
        Span::styled("]".to_string(),              Style::default().fg(Color::DarkGray)),
    ];
    if remaining > 1 {
        header_spans.push(Span::styled(" ".to_string(), Style::default()));
        header_spans.push(Span::styled("━".repeat(remaining - 1), Style::default().fg(Color::DarkGray)));
    }
    if !pct.is_empty() {
        header_spans.push(Span::styled(pct, Style::default().fg(Color::DarkGray)));
    }

    // ── content ───────────────────────────────────────────────────────────────
    let mut lines = vec![Line::from(header_spans)];

    match &fv.content {
        None => {
            let msg = if fv.showing_new {
                " (could not read file)"
            } else {
                " (old version unavailable — no git SHA in diff)"
            };
            lines.push(Line::from(Span::styled(msg, Style::default().fg(Color::DarkGray))));
        }
        Some(content) => {
            let lineno_width = content.len().to_string().len().max(3);
            let start = fv.scroll.min(content.len());
            let end = (start + content_height).min(content.len());

            for (idx, raw_line) in content[start..end].iter().enumerate() {
                let lineno     = start + idx + 1;
                let is_changed = changed.contains(&(lineno as u32));
                let line_bg    = if is_changed {
                    if fv.showing_new { Color::Rgb(0, 30, 0) } else { Color::Rgb(40, 0, 0) }
                } else {
                    Color::Reset
                };

                let mut spans = vec![Span::styled(
                    format!(" {:>width$} ", lineno, width = lineno_width),
                    Style::default().fg(Color::DarkGray).bg(line_bg),
                )];
                for s in highlighter.highlight_line(raw_line, ext) {
                    let fg = Color::Rgb(s.fg.0, s.fg.1, s.fg.2);
                    let mut style = Style::default().fg(fg).bg(line_bg);
                    if s.bold   { style = style.add_modifier(Modifier::BOLD); }
                    if s.italic { style = style.add_modifier(Modifier::ITALIC); }
                    spans.push(Span::styled(s.text, style));
                }
                lines.push(Line::from(spans));
            }
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}
