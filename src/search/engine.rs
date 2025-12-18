use crate::domain::DisplayItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use super::SelectionState;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub item: DisplayItem,
    pub score: i64,
    pub indices: Vec<usize>,
}

pub struct SearchEngine {
    term: String,
    matcher: SkimMatcherV2,
    results: Vec<SearchResult>,
    selection: SelectionState,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            term: String::new(),
            matcher: SkimMatcherV2::default().use_cache(true),
            results: Vec::new(),
            selection: SelectionState::default(),
        }
    }
}

impl SearchEngine {
    pub fn search(&mut self, items: &[DisplayItem]) {
        if self.term.is_empty() {
            self.results.clear();
            self.selection.update_count(0);
            return;
        }

        self.results = items
            .iter()
            .filter_map(|item| {
                let text = Self::search_text(item);
                self.matcher
                    .fuzzy_indices(&text, &self.term)
                    .map(|(score, indices)| SearchResult {
                        item: item.clone(),
                        score,
                        indices,
                    })
            })
            .collect();

        self.results.sort_by(|a, b| {
            let a_session = a.item.is_session();
            let b_session = b.item.is_session();
            match (a_session, b_session) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.score.cmp(&a.score),
            }
        });

        self.selection.update_count(self.results.len());
        if self.selection.index().is_none() && !self.results.is_empty() {
            self.selection.select_first();
        }
    }

    pub fn add_char(&mut self, c: char, items: &[DisplayItem]) {
        self.term.push(c);
        self.search(items);
    }

    pub fn backspace(&mut self, items: &[DisplayItem]) {
        self.term.pop();
        self.search(items);
    }

    pub fn clear(&mut self) {
        self.term.clear();
        self.results.clear();
        self.selection.clear();
    }

    pub fn term(&self) -> &str {
        &self.term
    }

    pub fn is_active(&self) -> bool {
        !self.term.is_empty()
    }

    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selection.index()
    }

    pub fn selected_item(&self) -> Option<&DisplayItem> {
        self.selection
            .index()
            .and_then(|i| self.results.get(i))
            .map(|r| &r.item)
    }

    pub fn move_up(&mut self) {
        self.selection.move_up();
    }

    pub fn move_down(&mut self) {
        self.selection.move_down();
    }

    fn search_text(item: &DisplayItem) -> String {
        match item {
            DisplayItem::ExistingSession {
                name,
                directory,
                is_current,
            } => {
                let prefix = if *is_current { "● " } else { "○ " };
                let dir = directory.as_deref().unwrap_or("");
                format!("{}{} ({})", prefix, name, dir)
            }
            DisplayItem::ResurrectableSession { name, duration } => {
                format!(
                    "↺ {} (created {} ago)",
                    name,
                    humantime::format_duration(*duration)
                )
            }
            DisplayItem::Directory { path, .. } => path.clone(),
        }
    }
}
