use crossterm::event::KeyEvent;

use crate::tui::app::{App, Focus};

mod body;
mod comment;
mod commit_picker;
mod diff;
mod filetree;
mod fileview;
mod fuzzy;
mod pr_description;
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
    if app.confirm_submit {
        match key.code {
            crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Char('y') => {
                app.confirm_submit = false;
                return Action::QuitWithOutput;
            }
            _ => {
                app.confirm_submit = false;
                return Action::Continue;
            }
        }
    }
    if app.show_pr_description {
        return pr_description::handle(app, key);
    }
    if app.commit_picker.is_some() {
        return commit_picker::handle(app, key);
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
