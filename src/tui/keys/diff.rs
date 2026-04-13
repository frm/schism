use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, Focus};
use crate::tui::body::BodyEditor;
use crate::tui::comment::CommentInput;
use crate::tui::commit_picker::CommitPicker;
use crate::tui::fileview::FileView;
use crate::tui::rows::Row;
use crate::tui::search::SearchState;

use super::Action;

pub fn handle(app: &mut App, key: KeyEvent) -> Action {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Char('/') => {
            app.search = Some(SearchState::new());
            Action::Continue
        }
        KeyCode::Char('?') => {
            app.show_help = true;
            Action::Continue
        }
        KeyCode::Char('D') if app.pr_context.is_some() => {
            app.show_pr_description = true;
            app.pr_description_scroll = 0;
            Action::Continue
        }
        KeyCode::Char('C') if app.pr_context.is_some() => {
            if let Some(ctx) = &app.pr_context {
                app.commit_picker = Some(CommitPicker::new(ctx.commits.clone()));
            }
            Action::Continue
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.search.is_some() {
                app.search = None;
                return Action::Continue;
            }
            Action::Quit
        }
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
            if let Some(s) = &mut app.search {
                if !s.matches.is_empty() {
                    s.current = (s.current + 1) % s.matches.len();
                    let dest = s.matches[s.current];
                    app.cursor = dest;
                    app.ensure_cursor_visible();
                    return Action::Continue;
                }
            }
            app.jump_next_hunk();
            Action::Continue
        }
        KeyCode::Char('N') => {
            if let Some(s) = &mut app.search {
                if !s.matches.is_empty() {
                    s.current = s.current.checked_sub(1).unwrap_or(s.matches.len() - 1);
                    let dest = s.matches[s.current];
                    app.cursor = dest;
                    app.ensure_cursor_visible();
                    return Action::Continue;
                }
            }
            app.jump_prev_hunk();
            Action::Continue
        }
        KeyCode::Char('d') if ctrl => {
            app.half_page_down();
            Action::Continue
        }
        KeyCode::Char('f') if ctrl => {
            app.page_down();
            Action::Continue
        }
        KeyCode::Char('b') if ctrl => {
            app.page_up();
            Action::Continue
        }
        KeyCode::Char('c') => {
            match &app.rows[app.cursor] {
                Row::Line { file_index, hunk_index, line_index } => {
                    let existing = app.files[*file_index].hunks[*hunk_index].lines[*line_index]
                        .comment.as_ref().map(|c| c.text.clone()).unwrap_or_default();
                    app.comment_input = Some(CommentInput::for_line(*file_index, *hunk_index, *line_index, existing));
                }
                Row::FileHeader { file_index } => {
                    let existing = app.files[*file_index].comment.as_ref()
                        .map(|c| c.text.clone()).unwrap_or_default();
                    app.comment_input = Some(CommentInput::for_file(*file_index, existing));
                }
                _ => {}
            }
            Action::Continue
        }
        KeyCode::Char('b') => {
            let existing = app.review_body.clone().unwrap_or_default();
            app.body_editor = Some(BodyEditor::new(existing));
            Action::Continue
        }
        KeyCode::Char('f') => {
            let fi = app.current_file_index();
            app.file_view = Some(FileView::open(fi, true));
            Action::Continue
        }
        KeyCode::Char('F') => {
            let fi = app.current_file_index();
            app.file_view = Some(FileView::open(fi, false));
            Action::Continue
        }
        KeyCode::Char('d') if app.pending_key == Some('d') => {
            app.pending_key = None;
            match &app.rows[app.cursor] {
                Row::FileHeader { file_index } => {
                    app.files[*file_index].comment = None;
                }
                _ => {
                    if let Some(line) = app.current_line_mut() {
                        line.comment = None;
                    }
                }
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
        KeyCode::Char('u') if ctrl => {
            app.half_page_up();
            Action::Continue
        }
        KeyCode::Char('t') => {
            app.show_filetree = !app.show_filetree;
            app.focus = if app.show_filetree { Focus::FileTree } else { Focus::Viewport };
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
        KeyCode::Char('p') if ctrl => {
            crate::tui::fuzzy::open(app);
            Action::Continue
        }
        KeyCode::Char(' ') => {
            app.toggle_fold_hunk();
            Action::Continue
        }
        KeyCode::PageDown => {
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
