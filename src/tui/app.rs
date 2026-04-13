use crate::tui::body::BodyEditor;
use crate::tui::comment::CommentInput;
use crate::tui::fileview::FileView;
use crate::tui::fuzzy::{FuzzyFinder, FuzzyMatch};
use crate::types::{DiffFile, DiffLine};

// ── tree types ────────────────────────────────────────────────────────────────

/// A node in the hierarchy tree. Dir nodes own their children.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    /// Full path (file) or path prefix (dir)
    pub path: String,
    pub is_dir: bool,
    pub expanded: bool,
    /// App::files index for file nodes; usize::MAX for dirs
    pub file_index: usize,
    pub depth: usize,
    pub children: Vec<TreeNode>,
}

/// A flat entry for cursor navigation — just references into the tree.
#[derive(Debug, Clone)]
pub struct FlatNode {
    pub path: String,
    pub is_dir: bool,
    pub file_index: usize,
    pub depth: usize,
    pub expanded: bool,
}

// ── app types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Row {
    FileHeader { file_index: usize },
    HunkHeader { file_index: usize, hunk_index: usize },
    Line { file_index: usize, hunk_index: usize, line_index: usize },
    Binary { file_index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Viewport,
    FileTree,
}

pub struct SearchState {
    pub query: String,
    pub matches: Vec<usize>,   // row indices
    pub current: usize,        // index into matches
    pub active_input: bool,    // true while typing
}

impl SearchState {
    pub fn new() -> Self {
        Self { query: String::new(), matches: Vec::new(), current: 0, active_input: true }
    }
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
    /// Hierarchy tree with expanded state
    pub tree_root: Vec<TreeNode>,
    /// Flattened visible nodes for rendering/navigation
    pub tree_flat: Vec<FlatNode>,
    /// Cursor index into tree_flat
    pub filetree_selected: usize,
    pub filetree_scroll: usize,
    pub focus: Focus,
    pub comment_input: Option<CommentInput>,
    pub fuzzy_finder: Option<FuzzyFinder>,
    pub body_editor: Option<BodyEditor>,
    pub review_body: Option<String>,
    pub file_view: Option<FileView>,
    pub search: Option<SearchState>,
    pub show_help: bool,
}

impl App {
    pub fn new(files: Vec<DiffFile>, show_filetree: bool) -> Self {
        let rows = build_rows(&files);
        let tree_root = build_tree(&files);
        let tree_flat = flatten_tree(&tree_root);
        Self {
            files,
            rows,
            cursor: 0,
            scroll_offset: 0,
            viewport_height: 0,
            viewport_width: 0,
            pending_key: None,
            show_filetree,
            tree_root,
            tree_flat,
            filetree_selected: 0,
            filetree_scroll: 0,
            focus: Focus::Viewport,
            comment_input: None,
            fuzzy_finder: None,
            body_editor: None,
            review_body: None,
            file_view: None,
            search: None,
            show_help: false,
        }
    }

    /// Toggle expand/collapse on the dir at filetree_selected.
    pub fn tree_toggle_expand(&mut self) {
        let path = match self.tree_flat.get(self.filetree_selected) {
            Some(n) if n.is_dir => n.path.clone(),
            _ => return,
        };
        toggle_node_expanded(&mut self.tree_root, &path);
        self.tree_flat = flatten_tree(&self.tree_root);
        self.filetree_selected = self.filetree_selected.min(self.tree_flat.len().saturating_sub(1));
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
        // Capture the header we want to land on after rebuild
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
        // Snap to the file we were on
        let fi = match self.rows.get(self.cursor) {
            Some(Row::FileHeader { file_index }) => *file_index,
            _ => 0,
        };
        self.snap_cursor_to_header(Row::FileHeader { file_index: fi });
    }

    /// After a fold rebuild, move cursor to the given header row and ensure it's visible.
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
        self.viewport_width
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
            Row::Line { file_index, hunk_index, line_index } =>
                Some(&self.files[*file_index].hunks[*hunk_index].lines[*line_index]),
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

// ── tree building ─────────────────────────────────────────────────────────────

fn build_tree(files: &[DiffFile]) -> Vec<TreeNode> {
    let mut root: Vec<TreeNode> = Vec::new();

    for (fi, file) in files.iter().enumerate() {
        let parts: Vec<&str> = file.path.split('/').collect();
        insert_node(&mut root, &parts, fi, 0);
    }

    sort_tree(&mut root);
    root
}

fn insert_node(nodes: &mut Vec<TreeNode>, parts: &[&str], file_index: usize, depth: usize) {
    let name = parts[0];
    let is_file = parts.len() == 1;

    if is_file {
        nodes.push(TreeNode {
            name: name.to_string(),
            path: build_path(nodes, name, depth),
            is_dir: false,
            expanded: false,
            file_index,
            depth,
            children: vec![],
        });
    } else {
        // Find or create the dir node
        let pos = nodes.iter().position(|n| n.is_dir && n.name == name);
        let pos = pos.unwrap_or_else(|| {
            let path = build_path(nodes, name, depth);
            nodes.push(TreeNode {
                name: name.to_string(),
                path,
                is_dir: true,
                expanded: true,
                file_index: usize::MAX,
                depth,
                children: vec![],
            });
            nodes.len() - 1
        });
        insert_node(&mut nodes[pos].children, &parts[1..], file_index, depth + 1);
    }
}

/// Build the path for a new node by looking at sibling nodes (they share the prefix).
fn build_path(siblings: &[TreeNode], name: &str, depth: usize) -> String {
    if depth == 0 {
        return name.to_string();
    }
    // All siblings share the same parent path — grab it from the first one
    if let Some(sib) = siblings.first() {
        let prefix: Vec<&str> = sib.path.split('/').take(depth).collect();
        return format!("{}/{}", prefix.join("/"), name);
    }
    name.to_string()
}

fn sort_tree(nodes: &mut Vec<TreeNode>) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    for n in nodes.iter_mut() {
        if n.is_dir { sort_tree(&mut n.children); }
    }
}

pub fn flatten_tree(nodes: &[TreeNode]) -> Vec<FlatNode> {
    let mut out = Vec::new();
    flatten_walk(nodes, &mut out);
    out
}

fn flatten_walk(nodes: &[TreeNode], out: &mut Vec<FlatNode>) {
    for n in nodes {
        out.push(FlatNode {
            path: n.path.clone(),
            is_dir: n.is_dir,
            file_index: n.file_index,
            depth: n.depth,
            expanded: n.expanded,
        });
        if n.is_dir && n.expanded {
            flatten_walk(&n.children, out);
        }
    }
}

fn toggle_node_expanded(nodes: &mut Vec<TreeNode>, path: &str) {
    for n in nodes.iter_mut() {
        if n.path == path {
            n.expanded = !n.expanded;
            return;
        }
        if n.is_dir {
            toggle_node_expanded(&mut n.children, path);
        }
    }
}

// ── row building ──────────────────────────────────────────────────────────────

fn build_rows(files: &[DiffFile]) -> Vec<Row> {
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
