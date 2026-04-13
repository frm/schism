/// Byte offset of the previous char boundary before `pos`.
pub fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos - 1;
    while !s.is_char_boundary(p) { p -= 1; }
    p
}

/// Flat byte offset of the start of each wrapped visual line.
pub fn wrapped_offsets(text: &str, width: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut flat_pos = 0usize;
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            offsets.push(flat_pos);
        } else {
            let chars: Vec<char> = log_line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let byte_i: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
                offsets.push(flat_pos + byte_i);
                i += width;
            }
        }
        flat_pos += log_line.len() + 1;
    }
    offsets
}

/// `(row, col)` of `cursor` within wrapped lines.
pub fn cursor_in_wrapped(offsets: &[usize], cursor: usize) -> (usize, usize) {
    let mut row = 0;
    for (i, &off) in offsets.iter().enumerate().rev() {
        if cursor >= off { row = i; break; }
    }
    (row, cursor - offsets[row])
}

/// Byte offset for `(target_row, target_col)`, clamped to line length.
pub fn wrapped_to_cursor(text: &str, offsets: &[usize], row: usize, col: usize, width: usize) -> usize {
    let lines = wrap_lines(text, width);
    let clamped_row = row.min(lines.len().saturating_sub(1));
    let line_len = lines.get(clamped_row).map(|l: &&str| l.len()).unwrap_or(0);
    offsets.get(clamped_row).copied().unwrap_or(text.len()) + col.min(line_len)
}

/// All wrapped visual lines as string slices.
pub fn wrap_lines<'a>(text: &'a str, width: usize) -> Vec<&'a str> {
    let mut out = Vec::new();
    for log_line in text.split('\n') {
        if log_line.is_empty() {
            out.push("");
        } else {
            let mut start = 0;
            let mut col = 0;
            for (i, _c) in log_line.char_indices() {
                if col == width {
                    out.push(&log_line[start..i]);
                    start = i;
                    col = 0;
                }
                col += 1;
            }
            out.push(&log_line[start..]);
        }
    }
    out
}

/// Render the cursor span for a given line at the given column.
/// Returns `(cursor_span, after_str)`.
pub fn render_cursor<'a>(
    line: &'a str,
    col: usize,
    bg: ratatui::style::Color,
) -> (ratatui::text::Span<'static>, &'a str) {
    let after_start = col.min(line.len());
    let char_at = line[after_start..].chars().next();
    match char_at {
        Some(c) => {
            let cursor_ch = line[after_start..after_start + c.len_utf8()].to_string();
            let rest = &line[after_start + c.len_utf8()..];
            (
                ratatui::text::Span::styled(
                    cursor_ch,
                    ratatui::style::Style::default().fg(bg).bg(ratatui::style::Color::White),
                ),
                rest,
            )
        }
        None => (
            ratatui::text::Span::styled(
                "█".to_string(),
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White)
                    .bg(bg)
                    .add_modifier(ratatui::style::Modifier::SLOW_BLINK),
            ),
            "",
        ),
    }
}
