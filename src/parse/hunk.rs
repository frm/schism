use crate::types::{DiffLine, Hunk, LineKind};

pub fn parse_hunks(lines: &[&str], mut i: usize) -> (Vec<Hunk>, usize) {
    let mut hunks = Vec::new();

    while i < lines.len() && !lines[i].starts_with("diff --git ") {
        if lines[i].starts_with("@@") {
            let (hunk, next) = parse_hunk(lines, i);
            hunks.push(hunk);
            i = next;
        } else {
            i += 1;
        }
    }

    (hunks, i)
}

fn parse_hunk(lines: &[&str], start: usize) -> (Hunk, usize) {
    let header = lines[start].to_string();
    let (old_start, old_count, new_start, new_count) = parse_header(&header);
    let (diff_lines, end) = parse_lines(lines, start + 1, old_start, new_start);

    (
        Hunk {
            header,
            old_start,
            old_count,
            new_start,
            new_count,
            lines: diff_lines,
            collapsed: false,
        },
        end,
    )
}

fn parse_lines(
    lines: &[&str],
    start: usize,
    old_start: u32,
    new_start: u32,
) -> (Vec<DiffLine>, usize) {
    let mut diff_lines = Vec::new();
    let mut old_lineno = old_start;
    let mut new_lineno = new_start;
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("diff --git ") || line.starts_with("@@") {
            break;
        }

        if line.starts_with("\\ No newline at end of file") {
            i += 1;
            continue;
        }

        match classify_line(line) {
            Some((kind, content)) => {
                let (old_no, new_no) = assign_linenos(&kind, &mut old_lineno, &mut new_lineno);
                diff_lines.push(DiffLine {
                    kind,
                    content,
                    old_lineno: old_no,
                    new_lineno: new_no,
                    comment: None,
                });
            }
            None => {}
        }

        i += 1;
    }

    (diff_lines, i)
}

fn classify_line(line: &str) -> Option<(LineKind, String)> {
    if line.starts_with('+') {
        Some((LineKind::Added, line[1..].to_string()))
    } else if line.starts_with('-') {
        Some((LineKind::Removed, line[1..].to_string()))
    } else if line.starts_with(' ') || line.is_empty() {
        let content = if line.is_empty() {
            String::new()
        } else {
            line[1..].to_string()
        };
        Some((LineKind::Context, content))
    } else {
        None
    }
}

fn assign_linenos(
    kind: &LineKind,
    old_lineno: &mut u32,
    new_lineno: &mut u32,
) -> (Option<u32>, Option<u32>) {
    match kind {
        LineKind::Added => {
            let n = *new_lineno;
            *new_lineno += 1;
            (None, Some(n))
        }
        LineKind::Removed => {
            let n = *old_lineno;
            *old_lineno += 1;
            (Some(n), None)
        }
        LineKind::Context => {
            let old = *old_lineno;
            let new = *new_lineno;
            *old_lineno += 1;
            *new_lineno += 1;
            (Some(old), Some(new))
        }
    }
}

fn parse_header(line: &str) -> (u32, u32, u32, u32) {
    let trimmed = line.trim_start_matches("@@ ").trim_end();
    let trimmed = trimmed.split(" @@").next().unwrap_or(trimmed);
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    let (old_start, old_count) = parse_range(parts.first().unwrap_or(&"-0,0"));
    let (new_start, new_count) = parse_range(parts.get(1).unwrap_or(&"+0,0"));

    (old_start, old_count, new_start, new_count)
}

fn parse_range(range: &str) -> (u32, u32) {
    let range = range.trim_start_matches(['-', '+']);
    if let Some((start, count)) = range.split_once(',') {
        (start.parse().unwrap_or(0), count.parse().unwrap_or(0))
    } else {
        (range.parse().unwrap_or(0), 1)
    }
}
