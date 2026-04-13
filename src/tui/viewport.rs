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

use crate::github::PrReviewContext;
use crate::render::syntax::Highlighter;
use crate::tui::app::App;
use crate::tui::draw;
use crate::tui::keys::{self, Action};
use crate::types::DiffFile;

/// Returns `None` on silent quit (q/Esc), `Some((files, body))` on Enter.
pub fn run(
    files: Vec<DiffFile>,
    show_tree: bool,
    pr_context: Option<PrReviewContext>,
    debug: bool,
) -> Result<Option<(Vec<DiffFile>, Option<String>)>> {
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

    let mut app = App::new(files, show_tree, pr_context);
    app.debug = debug;
    let highlighter = Highlighter::new();

    let result = run_loop(&mut terminal, &mut app, &highlighter);

    execute!(
        terminal.backend_mut(),
        PopKeyboardEnhancementFlags,
        DisableBracketedPaste,
        LeaveAlternateScreen,
    )?;
    disable_raw_mode()?;

    if let Some(debug_out) = app.debug_output.take() {
        eprintln!("{}", debug_out);
    }

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

        // Resolve pending file view fetches after rendering the loading state
        if app.file_view.as_ref().map(|fv| fv.pending_fetch).unwrap_or(false) {
            let pr = app.pr_context.as_ref();
            let files = &app.files;
            let cache = &mut app.file_content_cache;
            app.file_view.as_mut().unwrap().resolve_pending(files, pr, cache);
            continue;
        }

        match event::read()? {
            Event::Key(key) => match keys::handle_key(app, key) {
                Action::Continue => {}
                Action::Quit => return Ok(None),
                Action::QuitWithOutput => {
                    // PR mode: submit review via gh
                    if let (Some(ctx), Some(event)) = (&app.pr_context, app.review_event) {
                        let body = app.review_body.take().unwrap_or_default();
                        if app.debug {
                            let payload = crate::github::build_review_payload(&body, event, &ctx.metadata.head_ref_oid, &app.files);
                            let endpoint = format!(
                                "POST /repos/{}/{}/pulls/{}/reviews",
                                ctx.pr.owner, ctx.pr.repo, ctx.pr.number,
                            );
                            app.debug_output = Some(format!(
                                "{}\n{}",
                                endpoint,
                                serde_json::to_string_pretty(&payload).unwrap(),
                            ));
                            return Ok(None);
                        }
                        crate::github::submit_review(ctx, &body, event, &app.files)?;
                        return Ok(None);
                    }
                    // Normal mode: return files + body for stdout/json output
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
