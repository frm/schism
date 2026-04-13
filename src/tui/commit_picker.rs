pub mod draw;

use crate::github::pr::PrCommit;

pub struct CommitPicker {
    pub commits: Vec<PrCommit>,
    pub query: String,
    pub cursor_pos: usize,
    pub selected: usize,
    pub filtered: Vec<usize>,
}

impl CommitPicker {
    pub fn new(commits: Vec<PrCommit>) -> Self {
        let filtered = (0..commits.len()).collect();
        Self { commits, query: String::new(), cursor_pos: 0, selected: 0, filtered }
    }

    pub fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commits.len()).collect();
        } else {
            let q = self.query.to_lowercase();
            self.filtered = self.commits.iter().enumerate()
                .filter(|(_, c)| {
                    c.message.to_lowercase().contains(&q)
                        || c.sha.contains(&q)
                        || c.author.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = 0;
    }
}
