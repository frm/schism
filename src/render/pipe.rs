use std::io::{self, Write};

use crossterm::style::{Color, Stylize};
use crossterm::terminal;

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::types::{DiffFile, FileStatus, Hunk, LineKind};

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
            render_hunk(&mut out, hunk, ext, &highlighter, is_tty, term_width)?;
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
    let status_word = match file.status {
        FileStatus::Added => "added",
        FileStatus::Modified => "modified",
        FileStatus::Deleted => "deleted",
        FileStatus::Renamed => "renamed",
    };

    let (added, removed) = file_stats(file);

    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };

    if !is_tty {
        writeln!(out, " {} · {} · +{} -{}", path_display, status_word, added, removed)?;
        writeln!(out, "{}", "━".repeat(term_width))?;
        return Ok(());
    }

    // Path bright white, separator and status dim, stats colored
    write!(out, " {}", path_display.clone().white().bold())?;
    write!(out, "{}", " · ".dark_grey())?;
    write!(out, "{}", status_word.dark_grey())?;
    write!(out, "{}", " · ".dark_grey())?;
    write!(out, "{}", format!("+{}", added).green())?;
    write!(out, " ")?;
    writeln!(out, "{}", format!("-{}", removed).red())?;

    let separator: String = "━".repeat(term_width);
    writeln!(out, "{}", separator.dark_cyan())?;

    Ok(())
}

fn render_hunk(
    out: &mut impl Write,
    hunk: &Hunk,
    extension: &str,
    highlighter: &Highlighter,
    is_tty: bool,
    term_width: usize,
) -> io::Result<()> {
    let (line_num, func_context) = parse_hunk_context(&hunk.header);

    // Hunk open: ╭ L## function_context
    if !is_tty {
        match func_context {
            Some(ctx) => writeln!(out, " ╭ L{} {}", line_num, ctx)?,
            None => writeln!(out, " ╭ L{}", line_num)?,
        }
    } else {
        write!(out, "{}", " ╭ ".dark_grey())?;
        write!(out, "{}", format!("L{}", line_num).cyan())?;
        if let Some(ctx) = func_context {
            write!(out, " {}", ctx.dark_grey())?;
        }
        writeln!(out)?;
    }

    // Lines
    let lineno_width = 4;
    for line in &hunk.lines {
        render_line(out, line, lineno_width, extension, highlighter, is_tty, term_width)?;
    }

    // Hunk close: ╰
    if is_tty {
        writeln!(out, "{}", " ╰".dark_grey())?;
    } else {
        writeln!(out, " ╰")?;
    }

    Ok(())
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
    let old_no = LineRenderer::format_lineno(line.old_lineno, lineno_width);
    let new_no = LineRenderer::format_lineno(line.new_lineno, lineno_width);
    let prefix = LineRenderer::line_prefix(&line.kind);

    if !is_tty {
        writeln!(out, " │ {}{}{}{}", old_no, new_no, prefix, line.content)?;
        return Ok(());
    }

    let bg = match line.kind {
        LineKind::Added => Some(Color::Rgb { r: 0, g: 35, b: 0 }),
        LineKind::Removed => Some(Color::Rgb { r: 45, g: 0, b: 0 }),
        LineKind::Context => None,
    };

    // Frame character
    write!(out, "{}", " │ ".dark_grey())?;

    // Line numbers
    let old_styled = old_no.clone().dark_grey();
    let new_styled = new_no.clone().dark_grey();

    if let Some(bg_color) = bg {
        write!(out, "{}", old_styled.on(bg_color))?;
        write!(out, "{}", new_styled.on(bg_color))?;
    } else {
        write!(out, "{}{}", old_styled, new_styled)?;
    }

    // Prefix
    let prefix_color = LineRenderer::prefix_color(&line.kind);
    let prefix_styled = prefix.with(prefix_color);
    if let Some(bg_color) = bg {
        write!(out, "{}", prefix_styled.on(bg_color))?;
    } else {
        write!(out, "{}", prefix_styled)?;
    }

    // Syntax-highlighted content
    let spans = highlighter.highlight_line(&line.content, extension);
    for span in &spans {
        let fg = Color::Rgb {
            r: span.fg.0,
            g: span.fg.1,
            b: span.fg.2,
        };
        let styled = span.text.clone().with(fg);
        if let Some(bg_color) = bg {
            write!(out, "{}", styled.on(bg_color))?;
        } else {
            write!(out, "{}", styled)?;
        }
    }

    // Fill remaining width with background
    if let Some(bg_color) = bg {
        // Calculate how much we've written: " │ " (3) + old_no + new_no + prefix (1) + content
        let content_len = line.content.len();
        let written = 3 + lineno_width + 1 + lineno_width + 1 + 1 + content_len;
        if written < term_width {
            let padding = " ".repeat(term_width - written);
            write!(out, "{}", padding.on(bg_color))?;
        }
    }

    writeln!(out)?;
    Ok(())
}

fn file_stats(file: &DiffFile) -> (usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    for hunk in &file.hunks {
        for line in &hunk.lines {
            match line.kind {
                LineKind::Added => added += 1,
                LineKind::Removed => removed += 1,
                LineKind::Context => {}
            }
        }
    }
    (added, removed)
}

fn parse_hunk_context(header: &str) -> (u32, Option<&str>) {
    // "@@ -old,count +new,count @@ optional function context"
    let after_at = header.split(" @@ ").nth(1).unwrap_or("");
    let func_context = if after_at.is_empty() {
        // Try splitting on "@@" without trailing space
        let alt = header.split("@@").nth(2).map(|s| s.trim());
        alt.filter(|s| !s.is_empty())
    } else {
        Some(after_at.trim())
    };

    // Extract new-side start line number
    let new_start = header
        .split('+')
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    (new_start, func_context)
}
