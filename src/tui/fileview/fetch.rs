use std::collections::HashSet;
use std::process::Command;

use crate::types::{DiffFile, LineKind};

pub fn fetch_content(file: &DiffFile, new: bool) -> Option<Vec<String>> {
    let sha = if new { file.new_sha.as_deref() } else { file.old_sha.as_deref() };

    if let Some(sha) = sha {
        if let Ok(out) = Command::new("git").args(["cat-file", "blob", sha]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).into_owned();
                return Some(text.lines().map(|l| l.to_string()).collect());
            }
        }
    }

    // Fallback: read from disk for new version
    if new {
        if let Ok(text) = std::fs::read_to_string(&file.path) {
            return Some(text.lines().map(|l| l.to_string()).collect());
        }
    }

    None
}

pub fn changed_lines_new(file: &DiffFile) -> HashSet<u32> {
    file.hunks.iter().flat_map(|h| &h.lines)
        .filter_map(|l| if l.kind != LineKind::Context { l.new_lineno } else { None })
        .collect()
}

pub fn changed_lines_old(file: &DiffFile) -> HashSet<u32> {
    file.hunks.iter().flat_map(|h| &h.lines)
        .filter_map(|l| if l.kind != LineKind::Context { l.old_lineno } else { None })
        .collect()
}
