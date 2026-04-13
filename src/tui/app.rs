use std::collections::HashMap;

use crate::github::pr::{PrReviewContext, ReviewEvent};
use crate::tui::commit_picker::CommitPicker;
use crate::tui::body::BodyEditor;
use crate::tui::comment::CommentInput;
use crate::tui::fileview::FileView;
use crate::tui::fuzzy::FuzzyFinder;
use crate::tui::rows::{Row, build_rows};
use crate::tui::search::SearchState;
use crate::tui::tree::{TreeNode, FlatNode, build_tree, flatten_tree};
use crate::types::{DiffFile, DiffLine};

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
    pub tree_root: Vec<TreeNode>,
    pub tree_flat: Vec<FlatNode>,
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
    pub pr_context: Option<PrReviewContext>,
    pub show_pr_description: bool,
    pub pr_description_scroll: usize,
    pub commit_picker: Option<CommitPicker>,
    pub file_content_cache: HashMap<(usize, bool), Vec<String>>,
    pub review_event: Option<ReviewEvent>,
    pub debug: bool,
    pub debug_output: Option<String>,
}

impl App {
    pub fn new(files: Vec<DiffFile>, show_filetree: bool, pr_context: Option<PrReviewContext>) -> Self {
        let rows = build_rows(&files);
        let tree_root = build_tree(&files);
        let tree_flat = flatten_tree(&tree_root);
        let is_pr = pr_context.is_some();
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
            pr_context,
            show_pr_description: false,
            pr_description_scroll: 0,
            commit_picker: None,
            file_content_cache: HashMap::new(),
            review_event: if is_pr { Some(ReviewEvent::Comment) } else { None },
            debug: false,
            debug_output: None,
        }
    }

    pub fn tree_toggle_expand(&mut self) {
        let path = match self.tree_flat.get(self.filetree_selected) {
            Some(n) if n.is_dir => n.path.clone(),
            _ => return,
        };
        crate::tui::tree::toggle_node_expanded(&mut self.tree_root, &path);
        self.tree_flat = flatten_tree(&self.tree_root);
        self.filetree_selected = self.filetree_selected.min(self.tree_flat.len().saturating_sub(1));
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
