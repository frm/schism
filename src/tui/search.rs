use crate::tui::rows::Row;
use crate::types::DiffFile;

pub struct SearchState {
    pub query: String,
    pub matches: Vec<usize>,
    pub current: usize,
    pub active_input: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self { query: String::new(), matches: Vec::new(), current: 0, active_input: true }
    }
}

pub fn find_matches(files: &[DiffFile], rows: &[Row], query: &str) -> Vec<usize> {
    if query.is_empty() { return Vec::new(); }
    let q = query.to_lowercase();
    rows.iter().enumerate().filter_map(|(i, row)| {
        let text = match row {
            Row::FileHeader { file_index } => files[*file_index].path.as_str(),
            Row::HunkHeader { file_index, hunk_index } =>
                files[*file_index].hunks[*hunk_index].header.as_str(),
            Row::Line { file_index, hunk_index, line_index } =>
                files[*file_index].hunks[*hunk_index].lines[*line_index].content.as_str(),
            Row::Binary { .. } => "",
        };
        if text.to_lowercase().contains(&q) { Some(i) } else { None }
    }).collect()
}
