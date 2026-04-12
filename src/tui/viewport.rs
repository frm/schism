use std::fs::File;

use anyhow::Result;
use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event},
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

pub fn run(files: Vec<DiffFile>) -> Result<Option<String>> {
    let tty = File::options().read(true).write(true).open("/dev/tty")?;
    let backend = CrosstermBackend::new(tty);

    enable_raw_mode()?;
    let mut terminal = Terminal::new(backend)?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableBracketedPaste)?;

    let mut app = App::new(files);
    let highlighter = Highlighter::new();

    let result = run_loop(&mut terminal, &mut app, &highlighter);

    execute!(terminal.backend_mut(), DisableBracketedPaste, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<File>>,
    app: &mut App,
    highlighter: &Highlighter,
) -> Result<Option<String>> {
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
                Action::QuitWithOutput => return Ok(comment::collect(&app.files)),
            },
            Event::Paste(text) => {
                if let Some(ref mut input) = app.comment_input {
                    input.insert_str(&text);
                }
            }
            _ => {}
        }
    }
}
