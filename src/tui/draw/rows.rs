use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::tui::draw::diff::render_diff_line;
use crate::tui::draw::headers::{render_file_header, render_hunk_header};
use crate::tui::rows::Row;

pub fn row_bg(is_cursor: bool, is_match: bool, is_current_match: bool) -> Color {
    if is_cursor             { Color::Rgb(30, 30, 50) }
    else if is_current_match { Color::Rgb(60, 50, 0) }
    else if is_match         { Color::Rgb(35, 30, 0) }
    else                     { Color::Reset }
}

pub fn render_row<'a>(
    row: &Row,
    app: &App,
    highlighter: &Highlighter,
    is_cursor: bool,
    is_match: bool,
    is_current_match: bool,
    match_query: Option<&'a str>,
    width: usize,
) -> Line<'a> {
    let bg = row_bg(is_cursor, is_match, is_current_match);
    match row {
        Row::FileHeader { file_index } =>
            render_file_header(app, *file_index, bg, width),
        Row::HunkHeader { file_index, hunk_index } =>
            render_hunk_header(app, *file_index, *hunk_index, bg, width),
        Row::Line { file_index, hunk_index, line_index } => {
            let query = if is_match || is_current_match { match_query } else { None };
            render_diff_line(app, highlighter, *file_index, *hunk_index, *line_index, bg, query, width)
        }
        Row::Binary { .. } => Line::from(Span::styled(
            "  Binary file not shown".to_string(),
            Style::default().fg(Color::DarkGray).bg(bg),
        )),
    }
}
