use crate::tui::body::BodyEditor;
use crate::tui::comment::CommentInput;
use crate::tui::fuzzy::{FuzzyFinder, FuzzyMatch};
use crate::types::{DiffFile, DiffLine};

#[derive(Debug, Clone)]
pub enum Row {
    FileHeader { file_index: usize },
    HunkHeader { file_index: usize, hunk_index: usize },
    Line { file_index: usize, hunk_index: usize, line_index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Viewport,
    FileTree,
}

pub struct App {
    pub files: Vec<DiffFile>,
    pub rows: Vec<Row>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub viewport_width: usize,
    pub pending_key: Option<char>,
    pub show_filetree: bool,
    pub filetree_selected: usize,
    pub focus: Focus,
    pub comment_input: Option<CommentInput>,
    pub fuzzy_finder: Option<FuzzyFinder>,
    pub body_editor: Option<BodyEditor>,
    pub review_body: Option<String>,
}

impl App {
    pub fn new(files: Vec<DiffFile>) -> Self {
        let rows = build_rows(&files);
        Self {
            files,
            rows,
            cursor: 0,
            scroll_offset: 0,
            viewport_height: 0,
            viewport_width: 0,
            pending_key: None,
            show_filetree: false,
            filetree_selected: 0,
            focus: Focus::Viewport,
            comment_input: None,
            fuzzy_finder: None,
            body_editor: None,
            review_body: None,
        }
    }

    pub fn rebuild_rows(&mut self) {
        self.rows = build_rows(&self.files);
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
        }
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let new = self.cursor as isize + delta;
        self.cursor = new.clamp(0, self.rows.len().saturating_sub(1) as isize) as usize;
        self.ensure_cursor_visible();
    }

    pub fn half_page_down(&mut self) {
        self.move_cursor((self.viewport_height / 2) as isize);
    }

    pub fn half_page_up(&mut self) {
        self.move_cursor(-((self.viewport_height / 2) as isize));
    }

    pub fn page_down(&mut self) {
        self.move_cursor(self.viewport_height as isize);
    }

    pub fn page_up(&mut self) {
        self.move_cursor(-(self.viewport_height as isize));
    }

    pub fn goto_top(&mut self) {
        self.cursor = 0;
        self.ensure_cursor_visible();
    }

    pub fn goto_bottom(&mut self) {
        self.cursor = self.rows.len().saturating_sub(1);
        self.ensure_cursor_visible();
    }

    pub fn toggle_fold_hunk(&mut self) {
        match &self.rows[self.cursor] {
            Row::FileHeader { file_index } => {
                let fi = *file_index;
                self.files[fi].collapsed = !self.files[fi].collapsed;
            }
            Row::HunkHeader { file_index, hunk_index } => {
                let (fi, hi) = (*file_index, *hunk_index);
                self.files[fi].hunks[hi].collapsed = !self.files[fi].hunks[hi].collapsed;
            }
            Row::Line { file_index, hunk_index, .. } => {
                let (fi, hi) = (*file_index, *hunk_index);
                self.files[fi].hunks[hi].collapsed = !self.files[fi].hunks[hi].collapsed;
            }
        }
        self.rebuild_rows();
    }

    pub fn toggle_fold_file(&mut self) {
        let fi = self.current_file_index();
        self.files[fi].collapsed = !self.files[fi].collapsed;
        self.rebuild_rows();
    }

    pub fn toggle_fold_all_hunks_in_file(&mut self) {
        let fi = self.current_file_index();
        let all_collapsed = self.files[fi].hunks.iter().all(|h| h.collapsed);
        for hunk in &mut self.files[fi].hunks {
            hunk.collapsed = !all_collapsed;
        }
        self.rebuild_rows();
    }

    pub fn toggle_fold_all_files(&mut self) {
        let all_collapsed = self.files.iter().all(|f| f.collapsed);
        for file in &mut self.files {
            file.collapsed = !all_collapsed;
        }
        self.rebuild_rows();
    }

    pub fn jump_next_file(&mut self) {
        let current_file = self.current_file_index();
        for (i, row) in self.rows.iter().enumerate().skip(self.cursor + 1) {
            if let Row::FileHeader { file_index } = row {
                if *file_index != current_file {
                    self.cursor = i;
                    self.ensure_cursor_visible();
                    return;
                }
            }
        }
    }

    pub fn jump_prev_file(&mut self) {
        let current_file = self.current_file_index();
        for i in (0..self.cursor).rev() {
            if let Row::FileHeader { file_index } = &self.rows[i] {
                if *file_index != current_file {
                    self.cursor = i;
                    self.ensure_cursor_visible();
                    return;
                }
            }
        }
    }

    pub fn jump_next_hunk(&mut self) {
        for (i, row) in self.rows.iter().enumerate().skip(self.cursor + 1) {
            if matches!(row, Row::HunkHeader { .. }) {
                self.cursor = i;
                self.ensure_cursor_visible();
                return;
            }
        }
    }

    pub fn jump_prev_hunk(&mut self) {
        for i in (0..self.cursor).rev() {
            if matches!(&self.rows[i], Row::HunkHeader { .. }) {
                self.cursor = i;
                self.ensure_cursor_visible();
                return;
            }
        }
    }

    pub fn jump_to_file(&mut self, file_index: usize) {
        for (i, row) in self.rows.iter().enumerate() {
            if let Row::FileHeader { file_index: fi } = row {
                if *fi == file_index {
                    self.cursor = i;
                    self.ensure_cursor_visible();
                    return;
                }
            }
        }
    }

    pub fn viewport_width(&self) -> usize {
        // viewport_height is set per-frame; width isn't tracked but we can approximate.
        // Actually store it properly:
        self.viewport_width
    }

    pub fn current_file_index(&self) -> usize {
        match &self.rows[self.cursor] {
            Row::FileHeader { file_index } => *file_index,
            Row::HunkHeader { file_index, .. } => *file_index,
            Row::Line { file_index, .. } => *file_index,
        }
    }

    pub fn ensure_cursor_visible(&mut self) {
        if self.viewport_height == 0 {
            return;
        }
        let scrolloff: usize = 5;
        let top = self.scroll_offset + scrolloff;
        let bottom = self.scroll_offset + self.viewport_height.saturating_sub(1 + scrolloff);

        if self.cursor < top {
            self.scroll_offset = self.cursor.saturating_sub(scrolloff);
        } else if self.cursor > bottom {
            self.scroll_offset = self.cursor + 1 + scrolloff - self.viewport_height;
        }
    }

    pub fn current_line(&self) -> Option<&DiffLine> {
        match &self.rows[self.cursor] {
            Row::Line { file_index, hunk_index, line_index } => {
                Some(&self.files[*file_index].hunks[*hunk_index].lines[*line_index])
            }
            _ => None,
        }
    }

    pub fn current_line_mut(&mut self) -> Option<&mut DiffLine> {
        match &self.rows[self.cursor] {
            Row::Line { file_index, hunk_index, line_index } => {
                let (fi, hi, li) = (*file_index, *hunk_index, *line_index);
                Some(&mut self.files[fi].hunks[hi].lines[li])
            }
            _ => None,
        }
    }
}

fn build_rows(files: &[DiffFile]) -> Vec<Row> {
    let mut rows = Vec::new();

    for (fi, file) in files.iter().enumerate() {
        rows.push(Row::FileHeader { file_index: fi });

        if file.collapsed {
            continue;
        }

        for (hi, hunk) in file.hunks.iter().enumerate() {
            rows.push(Row::HunkHeader {
                file_index: fi,
                hunk_index: hi,
            });

            if hunk.collapsed {
                continue;
            }

            for (li, _) in hunk.lines.iter().enumerate() {
                rows.push(Row::Line {
                    file_index: fi,
                    hunk_index: hi,
                    line_index: li,
                });
            }
        }
    }

    rows
}
