use std::io::{self, Write};

use crate::render::line::LineRenderer;
use crate::types::{DiffFile, DiffLine, Hunk};

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

    writeln!(out, " {} · {} · +{} -{}", path_display, status_word, added, removed)?;
    writeln!(out, "{}", "━".repeat(term_width))?;
    Ok(())
}

pub fn render_hunk_header(out: &mut impl Write, hunk: &Hunk) -> io::Result<()> {
    let (line_num, func_context) = LineRenderer::parse_hunk_context(&hunk.header);
    match func_context {
        Some(ctx) => writeln!(out, " ╭ L{} {}", line_num, ctx),
        None => writeln!(out, " ╭ L{}", line_num),
    }
}

pub fn render_hunk_footer(out: &mut impl Write) -> io::Result<()> {
    writeln!(out, " ╰")
}

pub fn render_line(
    out: &mut impl Write,
    line: &DiffLine,
    lineno_width: usize,
) -> io::Result<()> {
    let old_no = LineRenderer::format_lineno(line.old_lineno, lineno_width);
    let new_no = LineRenderer::format_lineno(line.new_lineno, lineno_width);
    let prefix = LineRenderer::line_prefix(&line.kind);
    writeln!(out, " │ {}{}{}{}", old_no, new_no, prefix, line.content)
}
