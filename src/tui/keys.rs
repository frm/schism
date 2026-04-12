use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, Focus, Row};
use crate::tui::body::BodyEditor;
use crate::tui::comment::CommentInput;
use crate::tui::editor::TextEditor;

pub enum Action {
    Continue,
    Quit,
    QuitWithOutput,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Body editor mode — full overlay, same keys as comment input
    if app.body_editor.is_some() {
        let cw = crate::tui::body::content_width(app.viewport_width);
        match editor_action(key) {
            EditorAction::InsertNewline => {
                app.body_editor.as_mut().unwrap().editor.insert_char('\n');
            }
            EditorAction::Save => {
                let body = app.body_editor.take().unwrap();
                let text = body.editor.text.trim().to_string();
                app.review_body = if text.is_empty() { None } else { Some(text) };
            }
            EditorAction::Cancel => {
                app.body_editor = None;
            }
            EditorAction::Edit(edit) => apply_edit(&mut app.body_editor.as_mut().unwrap().editor, edit, cw),
            EditorAction::None => {}
        }
        return Action::Continue;
    }

    // Comment input mode
    if app.comment_input.is_some() {
        let cw = app.viewport_width.saturating_sub(crate::tui::comment::PREFIX_WIDTH);
        match editor_action(key) {
            EditorAction::InsertNewline => {
                app.comment_input.as_mut().unwrap().editor.insert_char('\n');
            }
            EditorAction::Save => {
                let input = app.comment_input.take().unwrap();
                let text = input.editor.text.trim().to_string();
                if !text.is_empty() {
                    app.files[input.file_index].hunks[input.hunk_index].lines[input.line_index]
                        .comment = Some(crate::types::Comment { text });
                }
            }
            EditorAction::Cancel => {
                app.comment_input = None;
            }
            EditorAction::Edit(edit) => apply_edit(&mut app.comment_input.as_mut().unwrap().editor, edit, cw),
            EditorAction::None => {}
        }
        return Action::Continue;
    }
    // Fuzzy finder mode
    if app.fuzzy_finder.is_some() {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            // Close
            KeyCode::Esc => {
                app.fuzzy_finder = None;
            }
            KeyCode::Char('c') if ctrl => {
                app.fuzzy_finder = None;
            }
            // Confirm — plain Enter only (Ctrl+J comes in as Enter too, treat same)
            KeyCode::Enter => {
                let fi = app.fuzzy_finder.as_ref()
                    .and_then(|f| f.matches.get(f.selected))
                    .map(|m| m.file_index);
                app.fuzzy_finder = None;
                if let Some(fi) = fi {
                    app.jump_to_file(fi);
                }
            }
            // Navigate down: Ctrl+J (arrives as Enter, handled above), Ctrl+N, or Down
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
            // Navigate up: Ctrl+K, Ctrl+P, or Up
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
            // Clear entire query
            KeyCode::Char('u') if ctrl => {
                if let Some(f) = &mut app.fuzzy_finder {
                    f.query.clear();
                    f.cursor_pos = 0;
                }
                crate::tui::fuzzy::update_matches(app);
            }
            // Delete word backwards
            KeyCode::Char('w') if ctrl => {
                if let Some(f) = &mut app.fuzzy_finder {
                    // trim trailing space then delete back to next space
                    let pos = f.cursor_pos;
                    let trimmed = f.query[..pos].trim_end_matches(' ');
                    let word_start = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                    f.query.drain(word_start..pos);
                    f.cursor_pos = word_start;
                }
                crate::tui::fuzzy::update_matches(app);
            }
            // Delete char backwards
            KeyCode::Backspace => {
                if let Some(f) = &mut app.fuzzy_finder {
                    if f.cursor_pos > 0 {
                        f.query.remove(f.cursor_pos - 1);
                        f.cursor_pos -= 1;
                    }
                }
                crate::tui::fuzzy::update_matches(app);
            }
            // Type a character (no modifier)
            KeyCode::Char(c) if !ctrl => {
                if let Some(f) = &mut app.fuzzy_finder {
                    f.query.insert(f.cursor_pos, c);
                    f.cursor_pos += 1;
                }
                crate::tui::fuzzy::update_matches(app);
            }
            _ => {}
        }
        return Action::Continue;
    }

    // File tree navigation
    if app.focus == Focus::FileTree && app.show_filetree {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if app.filetree_selected < app.tree_flat.len().saturating_sub(1) {
                    app.filetree_selected += 1;
                }
                return Action::Continue;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.filetree_selected = app.filetree_selected.saturating_sub(1);
                return Action::Continue;
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
                return Action::Continue;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // Collapse dir or move to parent
                if let Some(node) = app.tree_flat.get(app.filetree_selected) {
                    if node.is_dir && node.expanded {
                        app.tree_toggle_expand();
                    } else if node.depth > 0 {
                        // Move cursor to parent dir
                        let depth = node.depth;
                        let path = node.path.clone();
                        let parent: String = path.split('/').take(depth).collect::<Vec<_>>().join("/");
                        if let Some(i) = app.tree_flat.iter().position(|n| n.is_dir && n.path == parent) {
                            app.filetree_selected = i;
                        }
                    }
                }
                return Action::Continue;
            }
            KeyCode::Char('l') | KeyCode::Right => {
                app.focus = Focus::Viewport;
                return Action::Continue;
            }
            KeyCode::Char('t') | KeyCode::Esc => {
                app.focus = Focus::Viewport;
                return Action::Continue;
            }
            KeyCode::Char('q') => return Action::Quit,
            _ => return Action::Continue,
        }
    }

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
        KeyCode::Char('c') => {
            if let Row::Line { file_index, hunk_index, line_index } = &app.rows[app.cursor] {
                let existing = app.files[*file_index].hunks[*hunk_index].lines[*line_index]
                    .comment.as_ref().map(|c| c.text.clone()).unwrap_or_default();
                app.comment_input = Some(CommentInput::new(*file_index, *hunk_index, *line_index, existing));
            }
            Action::Continue
        }
        KeyCode::Char('b') => {
            let existing = app.review_body.clone().unwrap_or_default();
            app.body_editor = Some(BodyEditor::new(existing));
            Action::Continue
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.half_page_down();
            Action::Continue
        }
        KeyCode::Char('d') if app.pending_key == Some('d') => {
            app.pending_key = None;
            if let Some(line) = app.current_line_mut() {
                line.comment = None;
            }
            Action::Continue
        }
        KeyCode::Char('d') => {
            app.pending_key = Some('d');
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
        KeyCode::Char('t') => {
            app.show_filetree = !app.show_filetree;
            if !app.show_filetree {
                app.focus = Focus::Viewport;
            }
            Action::Continue
        }
        KeyCode::Char('h') | KeyCode::Left if app.show_filetree => {
            app.focus = Focus::FileTree;
            Action::Continue
        }
        KeyCode::Char('z') => {
            app.toggle_fold_hunk();
            Action::Continue
        }
        KeyCode::Char('Z') => {
            app.toggle_fold_file();
            Action::Continue
        }
        KeyCode::Tab => {
            app.toggle_fold_all_hunks_in_file();
            Action::Continue
        }
        KeyCode::BackTab => {
            app.toggle_fold_all_files();
            Action::Continue
        }
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::tui::fuzzy::open(app);
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

// ── shared editor input ───────────────────────────────────────────────────────

enum Edit {
    InsertChar(char),
    InsertStr(String),
    Backspace,
    DeleteForward,
    DeleteWordBack,
    DeleteToLineStart,
    MoveLeft, MoveRight,
    MoveWordLeft, MoveWordRight,
    MoveUp, MoveDown,
    MoveLineStart, MoveLineEnd,
}

enum EditorAction {
    InsertNewline,
    Save,
    Cancel,
    Edit(Edit),
    None,
}

fn editor_action(key: KeyEvent) -> EditorAction {
    let ctrl  = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt   = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Enter if shift                 => EditorAction::InsertNewline,
        KeyCode::Enter                          => EditorAction::Save,
        KeyCode::Esc                            => EditorAction::Cancel,
        KeyCode::Char('w') if ctrl              => EditorAction::Edit(Edit::DeleteWordBack),
        KeyCode::Char('u') if ctrl              => EditorAction::Edit(Edit::DeleteToLineStart),
        KeyCode::Backspace if ctrl || alt       => EditorAction::Edit(Edit::DeleteWordBack),
        KeyCode::Backspace                      => EditorAction::Edit(Edit::Backspace),
        KeyCode::Delete                         => EditorAction::Edit(Edit::DeleteForward),
        KeyCode::Home                           => EditorAction::Edit(Edit::MoveLineStart),
        KeyCode::End                            => EditorAction::Edit(Edit::MoveLineEnd),
        KeyCode::Left  if alt                   => EditorAction::Edit(Edit::MoveWordLeft),
        KeyCode::Right if alt                   => EditorAction::Edit(Edit::MoveWordRight),
        KeyCode::Char('b') if alt               => EditorAction::Edit(Edit::MoveWordLeft),
        KeyCode::Char('f') if alt               => EditorAction::Edit(Edit::MoveWordRight),
        KeyCode::Left                           => EditorAction::Edit(Edit::MoveLeft),
        KeyCode::Right                          => EditorAction::Edit(Edit::MoveRight),
        KeyCode::Up                             => EditorAction::Edit(Edit::MoveUp),
        KeyCode::Down                           => EditorAction::Edit(Edit::MoveDown),
        KeyCode::Char(c) if !ctrl && !alt       => EditorAction::Edit(Edit::InsertChar(c)),
        _                                       => EditorAction::None,
    }
}

fn apply_edit(ed: &mut TextEditor, edit: Edit, cw: usize) {
    match edit {
        Edit::InsertChar(c)      => ed.insert_char(c),
        Edit::InsertStr(s)       => ed.insert_str(&s),
        Edit::Backspace          => ed.backspace(),
        Edit::DeleteForward      => ed.delete_forward(),
        Edit::DeleteWordBack     => ed.delete_word_back(),
        Edit::DeleteToLineStart  => ed.delete_to_line_start(),
        Edit::MoveLeft           => ed.move_left(),
        Edit::MoveRight          => ed.move_right(),
        Edit::MoveWordLeft       => ed.move_word_left(),
        Edit::MoveWordRight      => ed.move_word_right(),
        Edit::MoveUp             => ed.move_up(cw),
        Edit::MoveDown           => ed.move_down(cw),
        Edit::MoveLineStart      => ed.move_to_line_start(cw),
        Edit::MoveLineEnd        => ed.move_to_line_end(cw),
    }
}
