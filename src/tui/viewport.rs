use std::fs::File;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::tui::comment;
use crate::tui::draw;
use crate::tui::keys::{self, Action};
use crate::types::DiffFile;

/// Returns `None` on silent quit (q/Esc), `Some((files, body))` on Enter.
pub fn run(files: Vec<DiffFile>, show_tree: bool) -> Result<Option<(Vec<DiffFile>, Option<String>)>> {
    let tty = File::options().read(true).write(true).open("/dev/tty")?;
    let backend = CrosstermBackend::new(tty);

    enable_raw_mode()?;
    let mut terminal = Terminal::new(backend)?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableBracketedPaste,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        ),
    )?;

    let mut app = App::new(files, show_tree);
    let highlighter = Highlighter::new();

    let result = run_loop(&mut terminal, &mut app, &highlighter);

    execute!(
        terminal.backend_mut(),
        PopKeyboardEnhancementFlags,
        DisableBracketedPaste,
        LeaveAlternateScreen,
    )?;
    disable_raw_mode()?;

    result

}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<File>>,
    app: &mut App,
    highlighter: &Highlighter,
) -> Result<Option<(Vec<DiffFile>, Option<String>)>> {
    loop {
        terminal.draw(|frame| {
            app.viewport_height = frame.area().height as usize;
            app.viewport_width = frame.area().width as usize;
            draw::draw(frame, app, highlighter);
        })?;

        match event::read()? {
            Event::Key(key) => match keys::handle_key(app, key) {
                Action::Continue => {}
                Action::Quit => return Ok(None),
                Action::QuitWithOutput => {
                    let body = app.review_body.take();
                    let files = std::mem::take(&mut app.files);
                    return Ok(Some((files, body)));
                }
            },
            Event::Paste(text) => {
                if let Some(ref mut input) = app.comment_input {
                    input.editor.insert_str(&text);
                } else if let Some(ref mut body) = app.body_editor {
                    body.editor.insert_str(&text);
                }
            }
            _ => {}
        }
    }
}
