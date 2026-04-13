use crate::types::{DiffFile, FileStatus};

use super::hunk;

pub fn parse_file(lines: &[&str], start: usize) -> (DiffFile, usize) {
    let (new_path, old_path_str) = extract_paths(lines[start]);
    let (status, old_path, old_sha, new_sha, mut i) = parse_extended_headers(lines, start + 1, &new_path);

    if i < lines.len() && lines[i].starts_with("Binary files") {
        let mut file = build_file(new_path, old_path, &old_path_str, status, Vec::new(), old_sha, new_sha);
        file.binary = true;
        return (file, i + 1);
    }

    i = skip_file_markers(lines, i);

    let (hunks, end) = hunk::parse_hunks(lines, i);
    let file = build_file(new_path, old_path, &old_path_str, status, hunks, old_sha, new_sha);
    (file, end)
}

fn extract_paths(diff_line: &str) -> (String, String) {
    let parts: Vec<&str> = diff_line.splitn(2, " b/").collect();
    let new_path = parts.get(1).unwrap_or(&"").to_string();
    let old_path_str = parts[0]
        .strip_prefix("diff --git a/")
        .unwrap_or("")
        .to_string();
    (new_path, old_path_str)
}

fn parse_extended_headers(
    lines: &[&str],
    start: usize,
    new_path: &str,
) -> (FileStatus, Option<String>, Option<String>, Option<String>, usize) {
    let mut status = FileStatus::Modified;
    let mut old_path: Option<String> = None;
    let mut old_sha: Option<String> = None;
    let mut new_sha: Option<String> = None;
    let mut i = start;

    while i < lines.len()
        && !lines[i].starts_with("---")
        && !lines[i].starts_with("diff --git ")
        && !lines[i].starts_with("Binary ")
    {
        let line = lines[i];
        if line.starts_with("new file mode") {
            status = FileStatus::Added;
        } else if line.starts_with("deleted file mode") {
            status = FileStatus::Deleted;
        } else if let Some(from) = line.strip_prefix("rename from ") {
            status = FileStatus::Renamed;
            old_path = Some(from.to_string());
        } else if let Some(rest) = line.strip_prefix("index ") {
            // index <old sha>..<new sha> [mode]
            let parts: Vec<&str> = rest.splitn(2, "..").collect();
            if parts.len() == 2 {
                old_sha = Some(parts[0].to_string());
                new_sha = Some(parts[1].split_whitespace().next().unwrap_or("").to_string());
            }
        }
        i += 1;
    }

    // Fall back to comparing paths if no explicit rename header but paths differ
    if old_path.is_none() {
        let old_from_header = lines[start.saturating_sub(1)]
            .strip_prefix("diff --git a/")
            .and_then(|s| s.splitn(2, " b/").next())
            .unwrap_or("");
        if !old_from_header.is_empty() && old_from_header != new_path {
            old_path = Some(old_from_header.to_string());
        }
    }

    (status, old_path, old_sha, new_sha, i)
}

fn skip_file_markers(lines: &[&str], mut i: usize) -> usize {
    if i < lines.len() && lines[i].starts_with("---") {
        i += 1;
    }
    if i < lines.len() && lines[i].starts_with("+++") {
        i += 1;
    }
    i
}

fn build_file(
    new_path: String,
    old_path: Option<String>,
    old_path_str: &str,
    status: FileStatus,
    hunks: Vec<crate::types::Hunk>,
    old_sha: Option<String>,
    new_sha: Option<String>,
) -> DiffFile {
    let resolved_old_path = old_path.or_else(|| {
        if old_path_str != new_path {
            Some(old_path_str.to_string())
        } else {
            None
        }
    });

    DiffFile {
        path: new_path,
        old_path: resolved_old_path,
        status,
        hunks,
        collapsed: false,
        binary: false,
        comment: None,
        old_sha,
        new_sha,
    }
}
