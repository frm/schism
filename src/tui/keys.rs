use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

pub enum Action {
    Continue,
    Quit,
    QuitWithOutput,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Handle pending 'g' for gg
    if app.pending_key == Some('g') {
        app.pending_key = None;
        if key.code == KeyCode::Char('g') {
            app.goto_top();
        }
        return Action::Continue;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Enter => Action::QuitWithOutput,
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_cursor(1);
            Action::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_cursor(-1);
            Action::Continue
        }
        KeyCode::Char('J') => {
            app.jump_next_file();
            Action::Continue
        }
        KeyCode::Char('K') => {
            app.jump_prev_file();
            Action::Continue
        }
        KeyCode::Char('n') => {
            app.jump_next_hunk();
            Action::Continue
        }
        KeyCode::Char('N') => {
            app.jump_prev_hunk();
            Action::Continue
        }
        KeyCode::Char('G') => {
            app.goto_bottom();
            Action::Continue
        }
        KeyCode::Char('g') => {
            app.pending_key = Some('g');
            Action::Continue
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_down();
            Action::Continue
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_up();
            Action::Continue
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.page_down();
            Action::Continue
        }
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.page_up();
            Action::Continue
        }
        KeyCode::Char(' ') | KeyCode::PageDown => {
            app.page_down();
            Action::Continue
        }
        KeyCode::PageUp => {
            app.page_up();
            Action::Continue
        }
        _ => Action::Continue,
    }
}
