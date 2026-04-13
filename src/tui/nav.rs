use crate::tui::app::App;
use crate::tui::rows::Row;

impl App {
    pub fn rebuild_rows(&mut self) {
        self.rows = crate::tui::rows::build_rows(&self.files);
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
        }
    }

    pub fn move_cursor(&mut self, delta: isize) {
        let new = self.cursor as isize + delta;
        self.cursor = new.clamp(0, self.rows.len().saturating_sub(1) as isize) as usize;
        self.ensure_cursor_visible();
    }

    pub fn half_page_down(&mut self) { self.move_cursor((self.viewport_height / 2) as isize); }
    pub fn half_page_up(&mut self)   { self.move_cursor(-((self.viewport_height / 2) as isize)); }
    pub fn page_down(&mut self)      { self.move_cursor(self.viewport_height as isize); }
    pub fn page_up(&mut self)        { self.move_cursor(-(self.viewport_height as isize)); }

    pub fn goto_top(&mut self) {
        self.cursor = 0;
        self.ensure_cursor_visible();
    }

    pub fn goto_bottom(&mut self) {
        self.cursor = self.rows.len().saturating_sub(1);
        self.ensure_cursor_visible();
    }

    pub fn toggle_fold_hunk(&mut self) {
        let target = match &self.rows[self.cursor] {
            Row::FileHeader { file_index } => {
                let fi = *file_index;
                self.files[fi].collapsed = !self.files[fi].collapsed;
                Row::FileHeader { file_index: fi }
            }
            Row::HunkHeader { file_index, hunk_index } => {
                let (fi, hi) = (*file_index, *hunk_index);
                self.files[fi].hunks[hi].collapsed = !self.files[fi].hunks[hi].collapsed;
                Row::HunkHeader { file_index: fi, hunk_index: hi }
            }
            Row::Line { file_index, hunk_index, .. } => {
                let (fi, hi) = (*file_index, *hunk_index);
                self.files[fi].hunks[hi].collapsed = !self.files[fi].hunks[hi].collapsed;
                Row::HunkHeader { file_index: fi, hunk_index: hi }
            }
            Row::Binary { file_index } => Row::Binary { file_index: *file_index },
        };
        self.rebuild_rows();
        self.snap_cursor_to_header(target);
    }

    pub fn toggle_fold_file(&mut self) {
        let fi = self.current_file_index();
        self.files[fi].collapsed = !self.files[fi].collapsed;
        self.rebuild_rows();
        self.snap_cursor_to_header(Row::FileHeader { file_index: fi });
    }

    pub fn toggle_fold_all_hunks_in_file(&mut self) {
        let fi = self.current_file_index();
        let all_collapsed = self.files[fi].hunks.iter().all(|h| h.collapsed);
        for hunk in &mut self.files[fi].hunks {
            hunk.collapsed = !all_collapsed;
        }
        self.rebuild_rows();
        self.snap_cursor_to_header(Row::FileHeader { file_index: fi });
    }

    pub fn toggle_fold_all_files(&mut self) {
        let all_collapsed = self.files.iter().all(|f| f.collapsed);
        for file in &mut self.files {
            file.collapsed = !all_collapsed;
        }
        self.rebuild_rows();
        let fi = match self.rows.get(self.cursor) {
            Some(Row::FileHeader { file_index }) => *file_index,
            _ => 0,
        };
        self.snap_cursor_to_header(Row::FileHeader { file_index: fi });
    }

    fn snap_cursor_to_header(&mut self, target: Row) {
        let pos = self.rows.iter().position(|r| match (&target, r) {
            (Row::FileHeader { file_index: a }, Row::FileHeader { file_index: b }) => a == b,
            (Row::HunkHeader { file_index: fa, hunk_index: ha },
             Row::HunkHeader { file_index: fb, hunk_index: hb }) => fa == fb && ha == hb,
            _ => false,
        });
        if let Some(i) = pos {
            self.cursor = i;
            self.ensure_cursor_visible();
        }
    }

    pub fn jump_next_file(&mut self) {
        let current = self.current_file_index();
        for (i, row) in self.rows.iter().enumerate().skip(self.cursor + 1) {
            if let Row::FileHeader { file_index } = row {
                if *file_index != current {
                    self.cursor = i;
                    self.ensure_cursor_visible();
                    return;
                }
            }
        }
    }

    pub fn jump_prev_file(&mut self) {
        let current = self.current_file_index();
        for i in (0..self.cursor).rev() {
            if let Row::FileHeader { file_index } = &self.rows[i] {
                if *file_index != current {
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

    pub fn current_file_index(&self) -> usize {
        if self.rows.is_empty() { return 0; }
        match &self.rows[self.cursor] {
            Row::FileHeader { file_index } => *file_index,
            Row::HunkHeader { file_index, .. } => *file_index,
            Row::Line { file_index, .. } => *file_index,
            Row::Binary { file_index } => *file_index,
        }
    }

    pub fn ensure_cursor_visible(&mut self) {
        if self.viewport_height == 0 { return; }
        const SCROLLOFF: usize = 5;
        let top    = self.scroll_offset + SCROLLOFF;
        let bottom = self.scroll_offset + self.viewport_height.saturating_sub(1 + SCROLLOFF);
        if self.cursor < top {
            self.scroll_offset = self.cursor.saturating_sub(SCROLLOFF);
        } else if self.cursor > bottom {
            self.scroll_offset = self.cursor + 1 + SCROLLOFF - self.viewport_height;
        }
    }
}
