use std::io::{self, Write};

use crossterm::style::{Color, Stylize};

use crate::render::line::LineRenderer;
use crate::render::syntax::Highlighter;
use crate::types::{DiffFile, DiffLine, Hunk, LineKind};

pub fn render_file_header(
    out: &mut impl Write,
    file: &DiffFile,
    term_width: usize,
) -> io::Result<()> {
    let status_word = LineRenderer::status_word(&file.status);
    let (added, removed) = LineRenderer::file_stats(file);
    let path_display = match &file.old_path {
        Some(old) => format!("{} → {}", old, file.path),
        None => file.path.clone(),
    };

    write!(out, " {}", path_display.clone().white().bold())?;
    write!(out, "{}", " · ".dark_grey())?;
    write!(out, "{}", status_word.dark_grey())?;
    write!(out, "{}", " · ".dark_grey())?;
    write!(out, "{}", format!("+{}", added).green())?;
    write!(out, " ")?;
    writeln!(out, "{}", format!("-{}", removed).red())?;
    writeln!(out, "{}", "━".repeat(term_width).dark_cyan())?;
    Ok(())
}

pub fn render_hunk_header(out: &mut impl Write, hunk: &Hunk) -> io::Result<()> {
    let (line_num, func_context) = LineRenderer::parse_hunk_context(&hunk.header);
    write!(out, "{}", " ╭ ".dark_grey())?;
    write!(out, "{}", format!("L{}", line_num).cyan())?;
    if let Some(ctx) = func_context {
        write!(out, " {}", ctx.dark_grey())?;
    }
    writeln!(out)?;
    Ok(())
}

pub fn render_hunk_footer(out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{}", " ╰".dark_grey())
}

pub fn render_line(
    out: &mut impl Write,
    line: &DiffLine,
    lineno_width: usize,
    extension: &str,
    highlighter: &Highlighter,
    term_width: usize,
) -> io::Result<()> {
    let old_no = LineRenderer::format_lineno(line.old_lineno, lineno_width);
    let new_no = LineRenderer::format_lineno(line.new_lineno, lineno_width);
    let prefix = LineRenderer::line_prefix(&line.kind);

    let bg = match line.kind {
        LineKind::Added => Some(Color::Rgb { r: 0, g: 35, b: 0 }),
        LineKind::Removed => Some(Color::Rgb { r: 45, g: 0, b: 0 }),
        LineKind::Context => None,
    };

    write!(out, "{}", " │ ".dark_grey())?;

    let old_styled = old_no.clone().dark_grey();
    let new_styled = new_no.clone().dark_grey();
    if let Some(bg_color) = bg {
        write!(out, "{}", old_styled.on(bg_color))?;
        write!(out, "{}", new_styled.on(bg_color))?;
    } else {
        write!(out, "{}{}", old_styled, new_styled)?;
    }

    let prefix_color = LineRenderer::prefix_color(&line.kind);
    let prefix_styled = prefix.with(prefix_color);
    if let Some(bg_color) = bg {
        write!(out, "{}", prefix_styled.on(bg_color))?;
    } else {
        write!(out, "{}", prefix_styled)?;
    }

    let spans = highlighter.highlight_line(&line.content, extension);
    for span in &spans {
        let fg = Color::Rgb { r: span.fg.0, g: span.fg.1, b: span.fg.2 };
        let styled = span.text.clone().with(fg);
        if let Some(bg_color) = bg {
            write!(out, "{}", styled.on(bg_color))?;
        } else {
            write!(out, "{}", styled)?;
        }
    }

    if let Some(bg_color) = bg {
        let content_len = line.content.len();
        let written = 3 + lineno_width + 1 + lineno_width + 1 + 1 + content_len;
        if written < term_width {
            write!(out, "{}", " ".repeat(term_width - written).on(bg_color))?;
        }
    }

    writeln!(out)?;
    Ok(())
}
