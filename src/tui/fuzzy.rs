mod draw;

pub use draw::draw;

use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher, Utf32Str,
};

use crate::tui::app::App;

pub struct FuzzyMatch {
    pub file_index: usize,
    pub score: u32,
}

pub struct FuzzyFinder {
    pub query: String,
    pub cursor_pos: usize,
    pub selected: usize,
    pub matches: Vec<FuzzyMatch>,
}

pub fn open(app: &mut App) {
    let matches = app.files.iter().enumerate()
        .map(|(i, _)| FuzzyMatch { file_index: i, score: 0 })
        .collect();

    app.fuzzy_finder = Some(FuzzyFinder {
        query: String::new(),
        cursor_pos: 0,
        selected: 0,
        matches,
    });
}

pub fn update_matches(app: &mut App) {
    let query = match &app.fuzzy_finder {
        Some(f) => f.query.clone(),
        None => return,
    };

    if query.is_empty() {
        let matches = app.files.iter().enumerate()
            .map(|(i, _)| FuzzyMatch { file_index: i, score: 0 })
            .collect();
        if let Some(f) = &mut app.fuzzy_finder {
            f.matches = matches;
            f.selected = 0;
        }
        return;
    }

    let atom = Atom::new(
        &query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());

    let mut matches: Vec<FuzzyMatch> = app.files.iter().enumerate().filter_map(|(i, file)| {
        let mut buf = Vec::new();
        let haystack = Utf32Str::new(&file.path, &mut buf);
        atom.score(haystack, &mut matcher)
            .map(|score| FuzzyMatch { file_index: i, score: score as u32 })
    }).collect();

    matches.sort_by(|a, b| b.score.cmp(&a.score));

    if let Some(f) = &mut app.fuzzy_finder {
        f.selected = 0;
        f.matches = matches;
    }
}
