use crate::session::SessionItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Search result containing an item and match information
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched item
    pub item: SessionItem,
    /// Fuzzy match score
    pub score: i64,
    /// Character indices that matched the search term
    pub indices: Vec<usize>,
}

/// Handles fuzzy searching across sessions and directories
pub struct SearchEngine {
    /// Current search term
    search_term: String,
    /// Fuzzy matcher instance
    matcher: SkimMatcherV2,
    /// Current search results
    results: Vec<SearchResult>,
    /// Selected result index
    selected_index: Option<usize>,
    /// Whether we're currently searching
    is_searching: bool,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            matcher: SkimMatcherV2::default().use_cache(true),
            results: Vec::new(),
            selected_index: None,
            is_searching: false,
        }
    }
}

impl SearchEngine {
    /// Update search term and perform search
    pub fn update_search(&mut self, term: String, items: &[SessionItem]) {
        self.search_term = term;
        self.is_searching = !self.search_term.is_empty();

        if self.is_searching {
            self.perform_search(items);
        } else {
            self.results.clear();
            self.selected_index = None;
        }
    }

    /// Add character to search term
    pub fn add_char(&mut self, c: char, items: &[SessionItem]) {
        self.search_term.push(c);
        self.update_search(self.search_term.clone(), items);
    }

    /// Remove last character from search term
    pub fn backspace(&mut self, items: &[SessionItem]) {
        self.search_term.pop();
        self.update_search(self.search_term.clone(), items);
    }

    /// Clear search term
    pub fn clear(&mut self) {
        self.search_term.clear();
        self.results.clear();
        self.selected_index = None;
        self.is_searching = false;
    }

    /// Get current search term
    pub fn search_term(&self) -> &str {
        &self.search_term
    }

    /// Check if currently searching
    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    /// Get search results
    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }

    /// Get selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Move selection up
    pub fn move_selection_up(&mut self) {
        if let Some(selected) = self.selected_index.as_mut() {
            if *selected == 0 {
                *selected = self.results.len().saturating_sub(1);
            } else {
                *selected = selected.saturating_sub(1);
            }
        } else if !self.results.is_empty() {
            self.selected_index = Some(self.results.len().saturating_sub(1));
        }
    }

    /// Move selection down
    pub fn move_selection_down(&mut self) {
        if let Some(selected) = self.selected_index.as_mut() {
            if *selected == self.results.len().saturating_sub(1) {
                *selected = 0;
            } else {
                *selected += 1;
            }
        } else if !self.results.is_empty() {
            self.selected_index = Some(0);
        }
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> Option<&SessionItem> {
        self.selected_index
            .and_then(|i| self.results.get(i))
            .map(|result| &result.item)
    }

    /// Perform fuzzy search on items
    fn perform_search(&mut self, items: &[SessionItem]) {
        let mut matches = Vec::new();

        for item in items {
            // Create the display text that will actually be shown
            let display_text = Self::get_display_text_for_search(item);

            // Match against the actual display text
            if let Some((score, indices)) =
                self.matcher.fuzzy_indices(&display_text, &self.search_term)
            {
                matches.push(SearchResult {
                    item: item.clone(),
                    score,
                    indices,
                });
            }
        }

        // Sort results: sessions first, then by score
        matches.sort_by(|a, b| {
            let a_is_session = a.item.is_session() || a.item.is_resurrectable_session();
            let b_is_session = b.item.is_session() || b.item.is_resurrectable_session();

            match (a_is_session, b_is_session) {
                (true, false) => std::cmp::Ordering::Less, // a (session) comes first
                (false, true) => std::cmp::Ordering::Greater, // b (session) comes first
                _ => b.score.cmp(&a.score),                // Same type, sort by score
            }
        });

        self.results = matches;

        // Update selected index
        if self.results.is_empty() {
            self.selected_index = None;
        } else {
            match self.selected_index {
                Some(idx) if idx >= self.results.len() => {
                    self.selected_index = Some(self.results.len().saturating_sub(1));
                }
                None => {
                    self.selected_index = Some(0);
                }
                _ => {} // Keep current selection if valid
            }
        }
    }

    /// Get the display text used for searching (matches what's rendered)
    fn get_display_text_for_search(item: &SessionItem) -> String {
        match item {
            SessionItem::ExistingSession {
                name,
                directory,
                is_current,
            } => {
                let prefix = if *is_current { "● " } else { "○ " };
                format!("{}{} ({})", prefix, name, directory)
            }
            SessionItem::ResurrectableSession { name, duration } => {
                // For resurrectable sessions, we show the name and duration
                format!(
                    "↺ {} (created {} ago)",
                    name,
                    humantime::format_duration(*duration)
                )
            }
            SessionItem::Directory { path, .. } => {
                // For directories, we search the full path as displayed
                path.clone()
            }
        }
    }
}
