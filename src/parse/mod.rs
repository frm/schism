mod file;
mod hunk;

use crate::types::DiffFile;

pub fn parse_diff(input: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if !lines[i].starts_with("diff --git ") {
            i += 1;
            continue;
        }

        let (file, next) = file::parse_file(&lines, i);
        files.push(file);
        i = next;
    }

    files
}
