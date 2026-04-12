use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, Focus, Row};
use crate::tui::comment::CommentInput;

pub enum Action {
    Continue,
    Quit,
    QuitWithOutput,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Comment input mode — captures all keys
    if app.comment_input.is_some() {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt  = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let viewport_width = app.viewport_width();
        let content_width = viewport_width.saturating_sub(crate::tui::comment::PREFIX_WIDTH);

        match key.code {
            KeyCode::Enter if shift => {
                app.comment_input.as_mut().unwrap().insert_char('\n');
            }
            KeyCode::Enter => {
                let input = app.comment_input.take().unwrap();
                if !input.text.is_empty() {
                    app.files[input.file_index].hunks[input.hunk_index].lines[input.line_index]
                        .comment = Some(crate::types::Comment { text: input.text });
                }
            }
            KeyCode::Esc => {
                app.comment_input = None;
            }
            // Ctrl+W — delete word back
            KeyCode::Char('w') if ctrl => {
                app.comment_input.as_mut().unwrap().delete_word_back();
            }
            // Ctrl+U — delete to start of line
            KeyCode::Char('u') if ctrl => {
                app.comment_input.as_mut().unwrap().delete_to_line_start();
            }
            // Ctrl+Backspace or Alt+Backspace — delete word back (macOS Cmd+Delete)
            KeyCode::Backspace if ctrl || alt => {
                app.comment_input.as_mut().unwrap().delete_word_back();
            }
            KeyCode::Backspace => {
                app.comment_input.as_mut().unwrap().backspace();
            }
            // fn+Delete = forward delete
            KeyCode::Delete => {
                app.comment_input.as_mut().unwrap().delete_forward();
            }
            KeyCode::Home => {
                app.comment_input.as_mut().unwrap().move_to_line_start(content_width);
            }
            KeyCode::End => {
                app.comment_input.as_mut().unwrap().move_to_line_end(content_width);
            }
            // Alt+Left / Alt+Right — move by word
            KeyCode::Left if alt => {
                app.comment_input.as_mut().unwrap().move_word_left();
            }
            KeyCode::Right if alt => {
                app.comment_input.as_mut().unwrap().move_word_right();
            }
            KeyCode::Left => {
                app.comment_input.as_mut().unwrap().move_left();
            }
            KeyCode::Right => {
                app.comment_input.as_mut().unwrap().move_right();
            }
            KeyCode::Up => {
                app.comment_input.as_mut().unwrap().move_up(content_width);
            }
            KeyCode::Down => {
                app.comment_input.as_mut().unwrap().move_down(content_width);
            }
            KeyCode::Char(c) if !ctrl && !alt => {
                app.comment_input.as_mut().unwrap().insert_char(c);
            }
            _ => {}
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
                if app.filetree_selected < app.files.len().saturating_sub(1) {
                    app.filetree_selected += 1;
                }
                return Action::Continue;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.filetree_selected = app.filetree_selected.saturating_sub(1);
                return Action::Continue;
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                app.jump_to_file(app.filetree_selected);
                app.focus = Focus::Viewport;
                return Action::Continue;
            }
            KeyCode::Char('t') => {
                app.show_filetree = false;
                app.focus = Focus::Viewport;
                return Action::Continue;
            }
            KeyCode::Char('q') | KeyCode::Esc => return Action::Quit,
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
                    .comment
                    .as_ref()
                    .map(|c| c.text.clone())
                    .unwrap_or_default();
                app.comment_input = Some(CommentInput {
                    text: existing.clone(),
                    cursor_pos: existing.len(),
                    file_index: *file_index,
                    hunk_index: *hunk_index,
                    line_index: *line_index,
                });
            }
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
            app.filetree_selected = app.current_file_index();
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
