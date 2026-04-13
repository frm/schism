use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;
use crate::tui::search::find_matches;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.search = None;
        }
        KeyCode::Enter => {
            if let Some(s) = &mut app.search {
                s.active_input = false;
                s.matches = find_matches(&app.files, &app.rows, &s.query);
                s.current = 0;
                if !s.matches.is_empty() {
                    let first = s.matches.iter().position(|&r| r >= app.cursor).unwrap_or(0);
                    s.current = first;
                    let dest = s.matches[first];
                    app.cursor = dest;
                    app.ensure_cursor_visible();
                }
            }
        }
        KeyCode::Backspace => {
            if let Some(s) = &mut app.search {
                s.query.pop();
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(s) = &mut app.search {
                s.query.push(c);
            }
        }
        _ => {}
    }

    Action::Continue
}

pub fn handle_pending_g(app: &mut App, key: KeyEvent) -> Action {
    app.pending_key = None;
    if key.code == KeyCode::Char('g') {
        app.goto_top();
    }
    Action::Continue
}
