use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => { app.pr_description_scroll += 1; }
        KeyCode::Char('k') | KeyCode::Up   => { app.pr_description_scroll = app.pr_description_scroll.saturating_sub(1); }
        KeyCode::Char('d') if ctrl         => { app.pr_description_scroll += 10; }
        KeyCode::Char('u') if ctrl         => { app.pr_description_scroll = app.pr_description_scroll.saturating_sub(10); }
        KeyCode::Char('G')                 => { app.pr_description_scroll = usize::MAX; }
        KeyCode::Char('g')                 => { app.pr_description_scroll = 0; }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('D') => {
            app.show_pr_description = false;
        }
        _ => {}
    }

    Action::Continue
}
