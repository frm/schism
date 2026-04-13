use crossterm::style::Color;

use crate::types::{DiffFile, LineKind};

pub struct LineRenderer;

impl LineRenderer {
    pub fn format_lineno(lineno: Option<u32>, width: usize) -> String {
        match lineno {
            Some(n) => format!("{:>width$} ", n, width = width),
            None => format!("{:>width$} ", "", width = width),
        }
    }

    pub fn line_prefix(kind: &LineKind) -> &'static str {
        match kind {
            LineKind::Added => "+",
            LineKind::Removed => "-",
            LineKind::Context => " ",
        }
    }

    pub fn prefix_color(kind: &LineKind) -> Color {
        match kind {
            LineKind::Added => Color::Green,
            LineKind::Removed => Color::Red,
            LineKind::Context => Color::DarkGrey,
        }
    }

    pub fn file_stats(file: &DiffFile) -> (usize, usize) {
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

    pub fn parse_hunk_context(header: &str) -> (u32, Option<&str>) {
        let after_at = header.split(" @@ ").nth(1).unwrap_or("");
        let func_context = if after_at.is_empty() {
            let alt = header.split("@@").nth(2).map(|s| s.trim());
            alt.filter(|s| !s.is_empty())
        } else {
            Some(after_at.trim())
        };

        let new_start = header
            .split('+')
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        (new_start, func_context)
    }

    pub fn status_word(status: &crate::types::FileStatus) -> &'static str {
        match status {
            crate::types::FileStatus::Added => "added",
            crate::types::FileStatus::Modified => "modified",
            crate::types::FileStatus::Deleted => "deleted",
            crate::types::FileStatus::Renamed => "renamed",
        }
    }
}
