pub mod diff;
pub mod headers;
pub mod rows;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::tui::comment;
use crate::tui::rows::Row;

pub fn draw(frame: &mut Frame, app: &App, highlighter: &Highlighter) {
    let full_area = frame.area();
    let area = if let Some(ref ctx) = app.pr_context {
        let bar = Rect::new(full_area.x, full_area.y, full_area.width, 1);
        let content = Rect::new(full_area.x, full_area.y + 1, full_area.width, full_area.height.saturating_sub(1));

        let info = Style::default().fg(Color::Cyan);
        let mut spans = vec![
            Span::styled(format!(" PR #{}", ctx.pr.number), info),
            Span::styled(" · ", info),
            Span::styled(ctx.metadata.author.clone(), info),
            Span::styled(" · ", info),
            Span::styled(format!("{} ← {}", ctx.metadata.base_branch, ctx.metadata.head_branch), info),
            Span::styled(" · ", info),
            Span::styled(ctx.metadata.title.clone(), info),
        ];

        let used: usize = spans.iter().map(|s| s.content.len()).sum();

        if let Some(event) = app.review_event {
            let (label, bg) = match event {
                crate::github::pr::ReviewEvent::Comment => (" comment ", Color::Rgb(40, 40, 60)),
                crate::github::pr::ReviewEvent::Approve => (" approve ", Color::Rgb(0, 50, 0)),
                crate::github::pr::ReviewEvent::RequestChanges => (" request changes ", Color::Rgb(60, 30, 0)),
            };
            let pad = (bar.width as usize).saturating_sub(used + label.len());
            spans.push(Span::styled(" ".repeat(pad), Style::default()));
            spans.push(Span::styled(
                label.to_string(),
                Style::default().fg(Color::White).bg(bg).add_modifier(Modifier::BOLD),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), bar);
        content
    } else {
        full_area
    };

    if app.show_filetree {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)])
            .split(area);

        crate::tui::filetree::draw(frame, app, chunks[0]);
        draw_viewport(frame, app, highlighter, chunks[1]);
    } else {
        draw_viewport(frame, app, highlighter, area);
    }

    if app.fuzzy_finder.is_some() {
        crate::tui::fuzzy::draw(frame, app, full_area);
    }
    if let Some(ref body) = app.body_editor {
        let action_label = app.review_event.map(|e| e.label());
        crate::tui::body::draw(frame, body, full_area, action_label);
    }
    if let Some(ref fv) = app.file_view {
        crate::tui::fileview::draw(frame, fv, app, highlighter);
    }
    if app.commit_picker.is_some() {
        crate::tui::commit_picker::draw::draw(frame, app, full_area);
    }
    if app.show_pr_description {
        crate::tui::pr_description::draw(frame, app, full_area);
    }
    if app.confirm_submit {
        crate::tui::confirm::draw(frame, app, full_area);
    }
    if app.show_help {
        crate::tui::help::draw(frame, app, full_area);
    }
}

fn draw_viewport(frame: &mut Frame, app: &App, highlighter: &Highlighter, area: Rect) {
    let search_bar_height = if app.search.is_some() { 1usize } else { 0 };
    let width = area.width as usize;
    let visible_rows = area.height as usize - search_bar_height;
    let end = (app.scroll_offset + visible_rows).min(app.rows.len());

    let comment_row_idx = app.comment_input.as_ref().and_then(|c| {
        use crate::tui::comment::CommentTarget;
        app.rows.iter().position(|r| match (&c.target, r) {
            (CommentTarget::Line { file_index: fi, hunk_index: hi, line_index: li },
             Row::Line { file_index, hunk_index, line_index }) =>
                fi == file_index && hi == hunk_index && li == line_index,
            (CommentTarget::File { file_index: fi }, Row::FileHeader { file_index }) =>
                fi == file_index,
            _ => false,
        })
    });

    let mut lines: Vec<Line> = Vec::new();

    for i in app.scroll_offset..end {
        if lines.len() >= visible_rows { break; }

        let is_cursor = i == app.cursor;
        let is_match = app.search.as_ref()
            .filter(|s| !s.active_input && !s.query.is_empty())
            .map(|s| s.matches.contains(&i))
            .unwrap_or(false);
        let is_current_match = app.search.as_ref()
            .filter(|s| !s.active_input)
            .and_then(|s| s.matches.get(s.current))
            .map(|&m| m == i)
            .unwrap_or(false);
        let query = app.search.as_ref()
            .filter(|s| !s.active_input && !s.query.is_empty())
            .map(|s| s.query.as_str());

        lines.push(rows::render_row(
            &app.rows[i], app, highlighter,
            is_cursor, is_match, is_current_match, query, width,
        ));

        let active_input = app.comment_input.as_ref().filter(|_| Some(i) == comment_row_idx);
        match &app.rows[i] {
            Row::FileHeader { file_index } => {
                let fi = *file_index;
                if let Some(ref input) = active_input {
                    for cl in comment::render_input(input, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                } else if let Some(ref c) = app.files[fi].comment {
                    for cl in comment::render_saved(&c.text, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                }
            }
            Row::Line { file_index, hunk_index, line_index } => {
                let diff_line = &app.files[*file_index].hunks[*hunk_index].lines[*line_index];
                if let Some(ref input) = active_input {
                    for cl in comment::render_input(input, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                } else if let Some(ref c) = diff_line.comment {
                    for cl in comment::render_saved(&c.text, width) {
                        if lines.len() >= visible_rows { break; }
                        lines.push(cl);
                    }
                }
            }
            _ => {}
        }
    }

    let content_area = if search_bar_height > 0 {
        Rect::new(area.x, area.y, area.width, area.height - 1)
    } else {
        area
    };
    frame.render_widget(Paragraph::new(lines), content_area);

    if let Some(ref s) = app.search {
        let bar_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let text = if s.active_input {
            format!("/{}", s.query)
        } else if s.matches.is_empty() {
            format!("/{} [no matches]", s.query)
        } else {
            format!("/{} [{}/{}]", s.query, s.current + 1, s.matches.len())
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" {}", text), Style::default().fg(Color::White)))),
            bar_area,
        );
    }
}
