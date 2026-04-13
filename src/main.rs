mod export;
mod parse;
mod render;
mod tui;
mod types;

use std::io::Read;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "schism", about = "Terminal diff reviewer")]
struct Cli {
    /// Pretty-print mode (no TUI)
    #[arg(long)]
    no_pager: bool,

    /// Output file for markdown export
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,

    /// Output review as JSON
    #[arg(long)]
    json: bool,

    /// Start with the file tree open
    #[arg(long)]
    tree: bool,
}

/// Strip ANSI escape sequences (e.g. colour codes git adds when color.pager is on).
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // consume until a letter (the final byte of a CSI sequence)
                while let Some(&d) = chars.peek() {
                    chars.next();
                    if d.is_ascii_alphabetic() { break; }
                }
            }
            // skip other escape sequences too
        } else {
            out.push(c);
        }
    }
    out
}

fn read_piped_stdin() -> Result<String> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;
    let input = strip_ansi(&raw);

    // Replace stdin fd with /dev/tty so crossterm can read keyboard events.
    //
    // 1. Close the consumed pipe on fd 0
    // 2. dup2 /dev/tty onto fd 0
    // 3. Leak the File so the underlying fd stays valid
    unsafe { libc::close(libc::STDIN_FILENO) };

    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    let tty_fd = std::os::unix::io::AsRawFd::as_raw_fd(&tty);
    if tty_fd != libc::STDIN_FILENO {
        if unsafe { libc::dup2(tty_fd, libc::STDIN_FILENO) } == -1 {
            anyhow::bail!("dup2 failed: {}", std::io::Error::last_os_error());
        }
        // tty will be dropped (closing tty_fd), but fd 0 is now an independent copy
    } else {
        // tty got assigned fd 0 directly — don't let Drop close it
        std::mem::forget(tty);
    }

    Ok(input)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let is_piped = !std::io::IsTerminal::is_terminal(&std::io::stdin());

    let input = if is_piped {
        read_piped_stdin()?
    } else {
        return Ok(());
    };

    if input.is_empty() {
        return Ok(());
    }

    let files = parse::parse_diff(&input);

    if cli.no_pager {
        let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
        render::pipe::render_pipe(&files, is_tty)?;
    } else {
        if let Some((files, review_body)) = tui::viewport::run(files, cli.tree)? {
            if cli.json {
                let review = export::json::Review {
                    body: review_body.as_deref(),
                    files: &files,
                };
                print!("{}", export::json::format_json(&review));
            } else {
                let output = tui::comment::collect(&files, review_body.as_deref());
                if let Some(s) = output { print!("{}", s); }
            }
        }
    }

    Ok(())
}
