pub mod draw;
pub mod fetch;

pub use draw::draw;

use crate::github::pr::PrReviewContext;
use crate::types::DiffFile;
use fetch::fetch_content;

pub const SCROLLOFF: usize = 5;

pub struct FileView {
    pub file_index: usize,
    pub showing_new: bool,
    pub scroll: usize,
    pub content: Option<Vec<String>>,
    pub pending_fetch: bool,
}

impl FileView {
    pub fn open(file_index: usize, showing_new: bool) -> Self {
        Self { file_index, showing_new, scroll: 0, content: None, pending_fetch: true }
    }

    pub fn resolve_pending(
        &mut self,
        files: &[DiffFile],
        pr_context: Option<&PrReviewContext>,
        cache: &mut std::collections::HashMap<(usize, bool), Vec<String>>,
    ) {
        if !self.pending_fetch { return; }
        self.pending_fetch = false;

        let key = (self.file_index, self.showing_new);
        if let Some(cached) = cache.get(&key) {
            self.content = Some(cached.clone());
            return;
        }

        let content = fetch_content(&files[self.file_index], self.showing_new, pr_context);
        if let Some(ref lines) = content {
            cache.insert(key, lines.clone());
        }
        self.content = content;
    }

    pub fn toggle_version(&mut self, viewport_height: usize) {
        self.showing_new = !self.showing_new;
        self.content = None;
        self.pending_fetch = true;
        self.clamp_scroll(viewport_height);
    }

    pub fn set_file(&mut self, file_index: usize) {
        self.file_index = file_index;
        self.content = None;
        self.pending_fetch = true;
        self.scroll = 0;
    }

    pub fn scroll_down(&mut self, n: usize, viewport_height: usize) {
        self.scroll = (self.scroll + n).min(self.max_scroll(viewport_height));
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn goto_top(&mut self) {
        self.scroll = 0;
    }

    pub fn goto_bottom(&mut self, viewport_height: usize) {
        self.scroll = self.max_scroll(viewport_height);
    }

    pub fn clamp_scroll(&mut self, viewport_height: usize) {
        self.scroll = self.scroll.min(self.max_scroll(viewport_height));
    }

    pub fn content_height(viewport_height: usize) -> usize {
        viewport_height.saturating_sub(1)
    }

    fn max_scroll(&self, viewport_height: usize) -> usize {
        let ch = Self::content_height(viewport_height);
        let total = self.content.as_ref().map(|c| c.len()).unwrap_or(0);
        total.saturating_sub(ch.saturating_sub(SCROLLOFF))
    }
}
