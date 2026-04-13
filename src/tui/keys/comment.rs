use crossterm::event::KeyEvent;

use crate::tui::app::App;
use crate::tui::comment::CommentTarget;
use crate::tui::editor::{apply_edit, editor_action, EditorAction};

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let cw = app.viewport_width.saturating_sub(crate::tui::comment::PREFIX_WIDTH);

    match editor_action(key) {
        EditorAction::InsertNewline => {
            app.comment_input.as_mut().unwrap().editor.insert_char('\n');
        }
        EditorAction::Save => {
            let input = app.comment_input.take().unwrap();
            let text = input.editor.text.trim().to_string();
            if !text.is_empty() {
                match input.target {
                    CommentTarget::Line { file_index, hunk_index, line_index } => {
                        app.files[file_index].hunks[hunk_index].lines[line_index].comment =
                            Some(crate::types::Comment { text });
                    }
                    CommentTarget::File { file_index } => {
                        app.files[file_index].comment = Some(crate::types::Comment { text });
                    }
                }
            }
        }
        EditorAction::Cancel => {
            app.comment_input = None;
        }
        EditorAction::Edit(edit) => {
            apply_edit(&mut app.comment_input.as_mut().unwrap().editor, edit, cw);
        }
        EditorAction::None => {}
    }

    Action::Continue
}
