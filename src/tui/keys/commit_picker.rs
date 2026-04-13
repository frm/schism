use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::github::pr;
use crate::tui::app::App;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Esc => {
            app.commit_picker = None;
        }
        KeyCode::Char('c') if ctrl => {
            app.commit_picker = None;
        }
        KeyCode::Enter => {
            let selected = app.commit_picker.as_ref().and_then(|p| {
                p.filtered.get(p.selected).map(|&i| p.commits[i].sha.clone())
            });
            let pr_ref = app.pr_context.as_ref().map(|ctx| ctx.pr.clone());

            app.commit_picker = None;

            if let (Some(sha), Some(pr)) = (selected, pr_ref) {
                match pr::fetch_commit_diff(&pr, &sha) {
                    Ok(diff) => {
                        let files = crate::parse::parse_diff(&diff);
                        app.files = files;
                        app.rebuild_rows();
                        app.cursor = 0;
                        app.scroll_offset = 0;
                    }
                    Err(_) => {}
                }
            }
        }
        KeyCode::Down => {
            if let Some(p) = &mut app.commit_picker {
                if p.selected < p.filtered.len().saturating_sub(1) {
                    p.selected += 1;
                }
            }
        }
        KeyCode::Char('n') if ctrl => {
            if let Some(p) = &mut app.commit_picker {
                if p.selected < p.filtered.len().saturating_sub(1) {
                    p.selected += 1;
                }
            }
        }
        KeyCode::Up => {
            if let Some(p) = &mut app.commit_picker {
                p.selected = p.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('p') if ctrl => {
            if let Some(p) = &mut app.commit_picker {
                p.selected = p.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('k') if ctrl => {
            if let Some(p) = &mut app.commit_picker {
                p.selected = p.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('u') if ctrl => {
            if let Some(p) = &mut app.commit_picker {
                p.query.clear();
                p.cursor_pos = 0;
                p.update_filter();
            }
        }
        KeyCode::Backspace => {
            if let Some(p) = &mut app.commit_picker {
                if p.cursor_pos > 0 {
                    p.query.remove(p.cursor_pos - 1);
                    p.cursor_pos -= 1;
                    p.update_filter();
                }
            }
        }
        KeyCode::Char(c) if !ctrl => {
            if let Some(p) = &mut app.commit_picker {
                p.query.insert(p.cursor_pos, c);
                p.cursor_pos += 1;
                p.update_filter();
            }
        }
        _ => {}
    }

    Action::Continue
}
