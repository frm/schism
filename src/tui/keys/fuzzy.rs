use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Esc => {
            app.fuzzy_finder = None;
        }
        KeyCode::Char('c') if ctrl => {
            app.fuzzy_finder = None;
        }
        KeyCode::Enter => {
            let fi = app.fuzzy_finder.as_ref()
                .and_then(|f| f.matches.get(f.selected))
                .map(|m| m.file_index);
            app.fuzzy_finder = None;
            if let Some(fi) = fi {
                app.jump_to_file(fi);
            }
        }
        KeyCode::Down => {
            if let Some(f) = &mut app.fuzzy_finder {
                if f.selected < f.matches.len().saturating_sub(1) {
                    f.selected += 1;
                }
            }
        }
        KeyCode::Char('n') if ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                if f.selected < f.matches.len().saturating_sub(1) {
                    f.selected += 1;
                }
            }
        }
        KeyCode::Up => {
            if let Some(f) = &mut app.fuzzy_finder {
                f.selected = f.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('k') if ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                f.selected = f.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('p') if ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                f.selected = f.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('u') if ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                f.query.clear();
                f.cursor_pos = 0;
            }
            crate::tui::fuzzy::update_matches(app);
        }
        KeyCode::Char('w') if ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                let pos = f.cursor_pos;
                let trimmed = f.query[..pos].trim_end_matches(' ');
                let word_start = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                f.query.drain(word_start..pos);
                f.cursor_pos = word_start;
            }
            crate::tui::fuzzy::update_matches(app);
        }
        KeyCode::Backspace => {
            if let Some(f) = &mut app.fuzzy_finder {
                if f.cursor_pos > 0 {
                    f.query.remove(f.cursor_pos - 1);
                    f.cursor_pos -= 1;
                }
            }
            crate::tui::fuzzy::update_matches(app);
        }
        KeyCode::Char(c) if !ctrl => {
            if let Some(f) = &mut app.fuzzy_finder {
                f.query.insert(f.cursor_pos, c);
                f.cursor_pos += 1;
            }
            crate::tui::fuzzy::update_matches(app);
        }
        _ => {}
    }

    Action::Continue
}
