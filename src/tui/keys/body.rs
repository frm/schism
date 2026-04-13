use crossterm::event::KeyEvent;

use crate::tui::app::App;
use crate::tui::editor::{apply_edit, editor_action, EditorAction};

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let cw = crate::tui::body::content_width(app.viewport_width);

    match editor_action(key) {
        EditorAction::InsertNewline => {
            app.body_editor.as_mut().unwrap().editor.insert_char('\n');
        }
        EditorAction::Save => {
            let text = app.body_editor.take().unwrap().editor.text.trim().to_string();
            app.review_body = if text.is_empty() { None } else { Some(text) };
        }
        EditorAction::Cancel => {
            app.body_editor = None;
        }
        EditorAction::Edit(edit) => {
            apply_edit(&mut app.body_editor.as_mut().unwrap().editor, edit, cw);
        }
        EditorAction::None => {}
    }

    Action::Continue
}
