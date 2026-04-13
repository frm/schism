mod plain;
mod tty;

use std::io::{self, Write};

use crossterm::terminal;

use crate::render::syntax::Highlighter;
use crate::types::DiffFile;

pub fn render_pipe(files: &[DiffFile], is_tty: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let highlighter = Highlighter::new();
    let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);

    for (i, file) in files.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }
        render_file_header(&mut out, file, is_tty, term_width)?;

        let ext = Highlighter::extension_from_path(&file.path);
        for (h, hunk) in file.hunks.iter().enumerate() {
            if h > 0 {
                writeln!(out)?;
            }
            render_hunk_header(&mut out, hunk, is_tty)?;

            let lineno_width = 4;
            for line in &hunk.lines {
                render_line(&mut out, line, lineno_width, ext, &highlighter, is_tty, term_width)?;
            }

            render_hunk_footer(&mut out, is_tty)?;
        }
    }

    Ok(())
}

fn render_file_header(
    out: &mut impl Write,
    file: &DiffFile,
    is_tty: bool,
    term_width: usize,
) -> io::Result<()> {
    if is_tty {
        tty::render_file_header(out, file, term_width)
    } else {
        plain::render_file_header(out, file, term_width)
    }
}

fn render_hunk_header(
    out: &mut impl Write,
    hunk: &crate::types::Hunk,
    is_tty: bool,
) -> io::Result<()> {
    if is_tty {
        tty::render_hunk_header(out, hunk)
    } else {
        plain::render_hunk_header(out, hunk)
    }
}

fn render_hunk_footer(out: &mut impl Write, is_tty: bool) -> io::Result<()> {
    if is_tty {
        tty::render_hunk_footer(out)
    } else {
        plain::render_hunk_footer(out)
    }
}

fn render_line(
    out: &mut impl Write,
    line: &crate::types::DiffLine,
    lineno_width: usize,
    extension: &str,
    highlighter: &Highlighter,
    is_tty: bool,
    term_width: usize,
) -> io::Result<()> {
    if is_tty {
        tty::render_line(out, line, lineno_width, extension, highlighter, term_width)
    } else {
        plain::render_line(out, line, lineno_width)
    }
}
