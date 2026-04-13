use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let vh = app.viewport_height;
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.file_view.as_mut().unwrap().scroll_down(1, vh);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.file_view.as_mut().unwrap().scroll_up(1);
        }
        KeyCode::Char('d') if ctrl => {
            app.file_view.as_mut().unwrap().scroll_down(vh / 2, vh);
        }
        KeyCode::Char('u') if ctrl => {
            app.file_view.as_mut().unwrap().scroll_up(vh / 2);
        }
        KeyCode::Char('G') => {
            app.file_view.as_mut().unwrap().goto_bottom(vh);
        }
        KeyCode::Char('g') if app.pending_key == Some('g') => {
            app.pending_key = None;
            app.file_view.as_mut().unwrap().goto_top();
        }
        KeyCode::Char('g') => {
            app.pending_key = Some('g');
        }
        KeyCode::Char('J') => {
            let next = (app.file_view.as_ref().unwrap().file_index + 1)
                .min(app.files.len().saturating_sub(1));
            let pr = app.pr_context.as_ref();
            app.file_view.as_mut().unwrap().set_file(next, &app.files, pr);
        }
        KeyCode::Char('K') => {
            let prev = app.file_view.as_ref().unwrap().file_index.saturating_sub(1);
            let pr = app.pr_context.as_ref();
            app.file_view.as_mut().unwrap().set_file(prev, &app.files, pr);
        }
        KeyCode::Char('m') => {
            let pr = app.pr_context.as_ref();
            app.file_view.as_mut().unwrap().toggle_version(&app.files, vh, pr);
        }
        KeyCode::Char('f') | KeyCode::Esc | KeyCode::Char('q') => {
            let fi = app.file_view.as_ref().unwrap().file_index;
            app.file_view = None;
            app.jump_to_file(fi);
            let max_scroll = app.rows.len().saturating_sub(app.viewport_height);
            app.scroll_offset = app.cursor.saturating_sub(5).min(max_scroll);
        }
        _ => {}
    }

    Action::Continue
}
