use crate::types::DiffFile;

#[derive(Debug, Clone)]
pub enum Row {
    FileHeader { file_index: usize },
    HunkHeader { file_index: usize, hunk_index: usize },
    Line { file_index: usize, hunk_index: usize, line_index: usize },
    Binary { file_index: usize },
}

pub fn build_rows(files: &[DiffFile]) -> Vec<Row> {
    let mut rows = Vec::new();

    for (fi, file) in files.iter().enumerate() {
        rows.push(Row::FileHeader { file_index: fi });

        if file.collapsed { continue; }

        if file.binary {
            rows.push(Row::Binary { file_index: fi });
            continue;
        }

        for (hi, hunk) in file.hunks.iter().enumerate() {
            rows.push(Row::HunkHeader { file_index: fi, hunk_index: hi });

            if hunk.collapsed { continue; }

            for (li, _) in hunk.lines.iter().enumerate() {
                rows.push(Row::Line { file_index: fi, hunk_index: hi, line_index: li });
            }
        }
    }

    rows
}
