use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::comment::render_editor_lines;
use crate::tui::editor::TextEditor;

const BG: Color = Color::Rgb(45, 45, 60);
const FG: Color = Color::Cyan;

pub struct BodyEditor {
    pub editor: TextEditor,
}

impl BodyEditor {
    pub fn new(existing: String) -> Self {
        Self { editor: TextEditor::with_text(existing) }
    }
}

pub fn draw(frame: &mut Frame, body: &BodyEditor, area: Rect) {
    let width = (area.width * 7 / 10).max(40).min(area.width);
    let height = (area.height * 45 / 100).max(8).min(area.height);
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let popup = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(FG))
        .title("──── Review body  Shift+Enter=newline · Enter=save · Esc=cancel ");

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Render editor lines into the inner area
    let editor_lines = render_editor_lines(
        &body.editor,
        inner.width as usize,
        "  ✎ ",
        "    ",
        BG,
    );

    // Pad remaining rows with empty background lines
    let mut all_lines = editor_lines;
    while all_lines.len() < inner.height as usize {
        let pad = inner.width as usize;
        all_lines.push(Line::from(Span::styled(
            " ".repeat(pad),
            Style::default().bg(BG),
        )));
    }

    let para = Paragraph::new(all_lines);
    frame.render_widget(para, inner);
}

/// Content width for the body overlay, given terminal width.
pub fn content_width(viewport_width: usize) -> usize {
    let popup_width = (viewport_width * 7 / 10).max(40).min(viewport_width);
    let inner_width = popup_width.saturating_sub(2); // borders
    inner_width.saturating_sub(4) // "  ✎ " / "    " prefix
}
