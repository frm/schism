use std::process::Command;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::types::DiffFile;

const SCROLLOFF: usize = 5;

pub struct FileView {
    pub file_index: usize,
    pub showing_new: bool,
    pub scroll: usize,
    pub content: Option<Vec<String>>,
}

impl FileView {
    pub fn open(file_index: usize, showing_new: bool, files: &[DiffFile]) -> Self {
        let content = fetch_content(&files[file_index], showing_new);
        Self { file_index, showing_new, scroll: 0, content }
    }

    pub fn toggle_version(&mut self, files: &[DiffFile]) {
        self.showing_new = !self.showing_new;
        self.content = fetch_content(&files[self.file_index], self.showing_new);
        // keep scroll position — the user stays in roughly the same region
    }

    pub fn set_file(&mut self, file_index: usize, files: &[DiffFile]) {
        self.file_index = file_index;
        self.content = fetch_content(&files[file_index], self.showing_new);
        self.scroll = 0;
    }

    pub fn scroll_down(&mut self, n: usize, viewport_height: usize) {
        let max = self.max_scroll(viewport_height);
        self.scroll = (self.scroll + n).min(max);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn goto_top(&mut self) {
        self.scroll = 0;
    }

    pub fn goto_bottom(&mut self, viewport_height: usize) {
        self.scroll = self.max_scroll(viewport_height);
    }

    fn content_height(viewport_height: usize) -> usize {
        viewport_height.saturating_sub(1) // 1 for header
    }

    fn max_scroll(&self, viewport_height: usize) -> usize {
        let ch = Self::content_height(viewport_height);
        let total = self.content.as_ref().map(|c| c.len()).unwrap_or(0);
        total.saturating_sub(ch.saturating_sub(SCROLLOFF))
    }
}

fn fetch_content(file: &DiffFile, new: bool) -> Option<Vec<String>> {
    let sha = if new { file.new_sha.as_deref() } else { file.old_sha.as_deref() };

    if let Some(sha) = sha {
        if let Ok(out) = Command::new("git").args(["cat-file", "blob", sha]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).into_owned();
                return Some(text.lines().map(|l| l.to_string()).collect());
            }
        }
    }

    // Fallback: read from disk for new version
    if new {
        if let Ok(text) = std::fs::read_to_string(&file.path) {
            return Some(text.lines().map(|l| l.to_string()).collect());
        }
    }

    None
}

// ── changed line sets ─────────────────────────────────────────────────────────

fn changed_lines_new(file: &DiffFile) -> std::collections::HashSet<u32> {
    file.hunks.iter().flat_map(|h| &h.lines)
        .filter_map(|l| if l.kind != crate::types::LineKind::Context { l.new_lineno } else { None })
        .collect()
}

fn changed_lines_old(file: &DiffFile) -> std::collections::HashSet<u32> {
    file.hunks.iter().flat_map(|h| &h.lines)
        .filter_map(|l| if l.kind != crate::types::LineKind::Context { l.old_lineno } else { None })
        .collect()
}

// ── rendering ─────────────────────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, fv: &FileView, app: &App, highlighter: &Highlighter) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let file = &app.files[fv.file_index];
    let ext  = Highlighter::extension_from_path(&file.path);
    let changed = if fv.showing_new { changed_lines_new(file) } else { changed_lines_old(file) };

    let width          = area.width as usize;
    let height         = area.height as usize;
    let content_height = FileView::content_height(height);

    // ── header (same style as diff file header) ────────────────────────────────
    let (added, removed) = crate::render::line::LineRenderer::file_stats(file);
    let status_word = crate::render::line::LineRenderer::status_word(&file.status);
    let version = if fv.showing_new { "new" } else { "old" };
    let version_color = if fv.showing_new { Color::Green } else { Color::Red };

    let pct = match &fv.content {
        Some(c) if c.len() > content_height => {
            let max = c.len().saturating_sub(content_height.saturating_sub(SCROLLOFF));
            format!(" {}%", (fv.scroll * 100 / max.max(1)).min(100))
        }
        _ => String::new(),
    };

    // Calculate used width for ━ fill (same accounting as render_file_header)
    // " ▾ " → " f " (3) + path (1 space after) + "· " + status + " " + "· " + "+N" + " " + "-N" + " [version]" + pct
    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };
    let used = 3 + path_display.len() + 1
        + 2 + status_word.len() + 1
        + 2 + format!("+{}", added).len() + 1 + format!("-{}", removed).len()
        + 2 + version.len() + 1  // " [version] "
        + pct.len();
    let remaining = width.saturating_sub(used);

    let mut header_spans = vec![
        Span::styled(" ▾ ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", path_display), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("· ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", status_word), Style::default().fg(Color::DarkGray)),
        Span::styled("· ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("+{}", added), Style::default().fg(Color::Green)),
        Span::styled(" ".to_string(), Style::default()),
        Span::styled(format!("-{}", removed), Style::default().fg(Color::Red)),
        Span::styled(" [".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(version.to_string(), Style::default().fg(version_color).add_modifier(Modifier::BOLD)),
        Span::styled("]".to_string(), Style::default().fg(Color::DarkGray)),
    ];
    if remaining > 1 {
        header_spans.push(Span::styled(" ".to_string(), Style::default()));
        header_spans.push(Span::styled("━".repeat(remaining - 1), Style::default().fg(Color::DarkGray)));
    }
    if !pct.is_empty() {
        header_spans.push(Span::styled(pct, Style::default().fg(Color::DarkGray)));
    }
    let header = Line::from(header_spans);

    // ── content ───────────────────────────────────────────────────────────────
    let mut lines = vec![header];

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
            let start = fv.scroll;
            let end   = (start + content_height).min(content.len());

            for (idx, raw_line) in content[start..end].iter().enumerate() {
                let lineno     = start + idx + 1;
                let is_changed = changed.contains(&(lineno as u32));

                let line_bg = if is_changed {
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
