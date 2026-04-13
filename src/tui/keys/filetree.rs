use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, Focus};

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.filetree_selected < app.tree_flat.len().saturating_sub(1) {
                app.filetree_selected += 1;
            }
            Action::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.filetree_selected = app.filetree_selected.saturating_sub(1);
            Action::Continue
        }
        KeyCode::Enter | KeyCode::Char('o') => {
            if let Some(node) = app.tree_flat.get(app.filetree_selected) {
                if node.is_dir {
                    app.tree_toggle_expand();
                } else {
                    app.jump_to_file(node.file_index);
                    app.focus = Focus::Viewport;
                }
            }
            Action::Continue
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(node) = app.tree_flat.get(app.filetree_selected) {
                if node.is_dir && node.expanded {
                    app.tree_toggle_expand();
                } else if node.depth > 0 {
                    let depth = node.depth;
                    let path = node.path.clone();
                    let parent: String = path.split('/').take(depth).collect::<Vec<_>>().join("/");
                    if let Some(i) = app.tree_flat.iter().position(|n| n.is_dir && n.path == parent) {
                        app.filetree_selected = i;
                    }
                }
            }
            Action::Continue
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.focus = Focus::Viewport;
            Action::Continue
        }
        KeyCode::Char('t') | KeyCode::Esc => {
            app.focus = Focus::Viewport;
            Action::Continue
        }
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Continue,
    }
}
