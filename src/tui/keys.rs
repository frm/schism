use crossterm::event::KeyEvent;

use crate::tui::app::{App, Focus};

mod body;
mod comment;
mod diff;
mod filetree;
mod fileview;
mod fuzzy;
mod search;

pub enum Action {
    Continue,
    Quit,
    QuitWithOutput,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    if app.show_help {
        app.show_help = false;
        return Action::Continue;
    }
    if app.file_view.is_some() {
        return fileview::handle(app, key);
    }
    if app.body_editor.is_some() {
        return body::handle(app, key);
    }
    if app.comment_input.is_some() {
        return comment::handle(app, key);
    }
    if app.fuzzy_finder.is_some() {
        return fuzzy::handle(app, key);
    }
    if matches!(&app.search, Some(s) if s.active_input) {
        return search::handle(app, key);
    }
    if app.focus == Focus::FileTree && app.show_filetree {
        return filetree::handle(app, key);
    }
    if app.pending_key == Some('g') {
        return search::handle_pending_g(app, key);
    }
    diff::handle(app, key)
}
