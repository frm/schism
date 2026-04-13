use crossterm::event::{KeyCode, KeyEvent};

use crate::github::ReviewEvent;
use crate::tui::app::App;
use crate::tui::editor::{apply_edit, editor_action, EditorAction};

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    // Tab cycles review action (only in PR mode)
    if key.code == KeyCode::Tab && app.pr_context.is_some() {
        app.review_event = Some(match app.review_event {
            None | Some(ReviewEvent::Comment) => ReviewEvent::Approve,
            Some(ReviewEvent::Approve) => ReviewEvent::RequestChanges,
            Some(ReviewEvent::RequestChanges) => ReviewEvent::Comment,
        });
        return Action::Continue;
    }

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
