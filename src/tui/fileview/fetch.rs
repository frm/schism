use std::collections::HashSet;
use std::process::Command;

use crate::github::pr::PrReviewContext;
use crate::types::{DiffFile, LineKind};

pub fn fetch_content(
    file: &DiffFile,
    new: bool,
    pr_context: Option<&PrReviewContext>,
) -> Option<Vec<String>> {
    // PR mode: fetch from GitHub
    if let Some(ctx) = pr_context {
        let ref_oid = if new {
            &ctx.metadata.head_ref_oid
        } else {
            &ctx.metadata.base_ref_oid
        };
        match crate::github::pr::fetch_file_content(&ctx.pr, &file.path, ref_oid) {
            Ok(text) => return Some(text.lines().map(|l| l.to_string()).collect()),
            Err(_) => return None,
        }
    }

    // Local mode: git cat-file or disk fallback
    let sha = if new { file.new_sha.as_deref() } else { file.old_sha.as_deref() };

    if let Some(sha) = sha {
        if let Ok(out) = Command::new("git").args(["cat-file", "blob", sha]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).into_owned();
                return Some(text.lines().map(|l| l.to_string()).collect());
            }
        }
    }

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
