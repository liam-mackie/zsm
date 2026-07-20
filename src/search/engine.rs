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

        // Capture before rebuilding results — old index into old results.
        let prev_selected = self
            .selected_item()
            .map(|item| item.display_name().to_string());

        self.results = items
            .iter()
            .filter_map(|item| {
                let text = item.display_text();
                self.matcher
                    .fuzzy_indices(&text, &self.term)
                    .map(|(score, indices)| SearchResult {
                        item: item.clone(),
                        score,
                        indices,
                    })
            })
            .collect();

        // Sessions before directories. Sessions keep their input order (stable sort
        // returns Equal). Directories sorted by fuzzy score, then alphabetically.
        self.results.sort_by(|a, b| {
            let a_session = a.item.is_session();
            let b_session = b.item.is_session();
            match (a_session, b_session) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => std::cmp::Ordering::Equal,
                (false, false) => b
                    .score
                    .cmp(&a.score)
                    .then_with(|| a.item.display_name().cmp(b.item.display_name())),
            }
        });

        self.selection.update_count(self.results.len());

        // Try to preserve the previously selected item across re-searches.
        if let Some(ref name) = prev_selected {
            if let Some(idx) = self
                .results
                .iter()
                .position(|r| r.item.display_name() == name)
            {
                self.selection.set_index(Some(idx));
                return;
            }
        }

        // No previous selection or it was filtered out — default to first non-current.
        if !self.results.is_empty() {
            let first_non_current = self
                .results
                .iter()
                .position(|r| !r.item.is_current());
            self.selection
                .set_index(Some(first_non_current.unwrap_or(0)));
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

    pub fn re_search(&mut self, items: &[DisplayItem]) {
        if self.is_active() {
            let prev_name = self
                .selected_item()
                .map(|item| item.display_name().to_string());
            self.search(items);
            if let Some(name) = prev_name {
                if let Some(idx) = self
                    .results
                    .iter()
                    .position(|r| r.item.display_name() == name)
                {
                    self.selection.set_index(Some(idx));
                }
            }
        }
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

    pub fn move_to_top(&mut self) {
        self.selection.select_top();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_session(name: &str) -> DisplayItem {
        DisplayItem::ExistingSession {
            name: name.to_string(),
            directory: None,
            is_current: false,
        }
    }

    fn make_directory(path: &str, session_name: &str) -> DisplayItem {
        DisplayItem::Directory {
            path: path.to_string(),
            session_name: session_name.to_string(),
        }
    }

    fn make_resurrectable(name: &str) -> DisplayItem {
        DisplayItem::ResurrectableSession {
            name: name.to_string(),
            duration: Duration::from_secs(3600),
        }
    }

    #[test]
    fn empty_search_returns_empty_results() {
        let engine = SearchEngine::default();
        assert!(engine.results().is_empty());
        assert!(!engine.is_active());
    }

    #[test]
    fn search_without_term_clears_results() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("project")];
        engine.search(&items);
        assert!(engine.results().is_empty());
    }

    #[test]
    fn search_finds_matching_session() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("project"), make_session("other")];
        engine.add_char('p', &items);
        engine.add_char('r', &items);
        assert_eq!(engine.results().len(), 1);
        assert_eq!(engine.results()[0].item.display_name(), "project");
    }

    #[test]
    fn search_finds_matching_directory() {
        let mut engine = SearchEngine::default();
        let items = vec![make_directory("/home/user/project", "project")];
        engine.add_char('h', &items);
        engine.add_char('o', &items);
        engine.add_char('m', &items);
        assert_eq!(engine.results().len(), 1);
    }

    #[test]
    fn fuzzy_matching_works() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("my-project")];
        engine.add_char('m', &items);
        engine.add_char('p', &items);
        assert_eq!(engine.results().len(), 1);
    }

    #[test]
    fn sessions_ranked_above_directories() {
        let mut engine = SearchEngine::default();
        let items = vec![
            make_directory("/home/project", "project"),
            make_session("project"),
        ];
        engine.add_char('p', &items);
        assert!(engine.results()[0].item.is_session());
    }

    #[test]
    fn backspace_updates_results() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("abc"), make_session("xyz")];
        engine.add_char('x', &items);
        assert_eq!(engine.results().len(), 1);
        engine.backspace(&items);
        assert!(engine.results().is_empty());
        assert!(!engine.is_active());
    }

    #[test]
    fn clear_resets_state() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("test")];
        engine.add_char('t', &items);
        assert!(engine.is_active());
        engine.clear();
        assert!(!engine.is_active());
        assert!(engine.results().is_empty());
        assert_eq!(engine.term(), "");
    }

    #[test]
    fn selection_auto_selects_first() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("aaa"), make_session("aab")];
        engine.add_char('a', &items);
        assert_eq!(engine.selected_index(), Some(0));
    }

    #[test]
    fn selection_skips_current_session() {
        let mut engine = SearchEngine::default();
        let current = DisplayItem::ExistingSession {
            name: "current-session".to_string(),
            directory: None,
            is_current: true,
        };
        let items = vec![current, make_session("other-session")];
        engine.add_char('s', &items);
        assert_eq!(engine.results().len(), 2);
        // Should select "other-session", not the current one
        assert_eq!(
            engine.selected_item().unwrap().display_name(),
            "other-session"
        );
    }

    #[test]
    fn selection_falls_back_to_current_if_only_match() {
        let mut engine = SearchEngine::default();
        let current = DisplayItem::ExistingSession {
            name: "current-session".to_string(),
            directory: None,
            is_current: true,
        };
        let items = vec![current];
        engine.add_char('c', &items);
        assert_eq!(engine.selected_index(), Some(0));
    }

    #[test]
    fn selection_preserved_when_current_filtered_out() {
        let mut engine = SearchEngine::default();
        let current = DisplayItem::ExistingSession {
            name: "current-web".to_string(),
            directory: None,
            is_current: true,
        };
        let other = make_session("api-server");
        // Both match 'e' — current at 0, other at 1. Auto-selects 1 (first non-current).
        let items = vec![current, other];
        engine.add_char('e', &items);
        assert_eq!(
            engine.selected_item().unwrap().display_name(),
            "api-server"
        );

        // Type more to filter out current — "api-server" should stay selected
        engine.add_char('r', &items);
        assert_eq!(
            engine.selected_item().unwrap().display_name(),
            "api-server"
        );
    }

    #[test]
    fn move_to_top_selects_first() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("aa"), make_session("ab"), make_session("ac")];
        engine.add_char('a', &items);
        engine.move_down();
        engine.move_down();
        assert_eq!(engine.selected_index(), Some(2));
        engine.move_to_top();
        assert_eq!(engine.selected_index(), Some(0));
    }

    #[test]
    fn move_down_updates_selection() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("aa"), make_session("ab"), make_session("ac")];
        engine.add_char('a', &items);
        engine.move_down();
        assert_eq!(engine.selected_index(), Some(1));
    }

    #[test]
    fn move_up_updates_selection() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("aa"), make_session("ab"), make_session("ac")];
        engine.add_char('a', &items);
        engine.move_down();
        engine.move_down();
        engine.move_up();
        assert_eq!(engine.selected_index(), Some(1));
    }

    #[test]
    fn selected_item_returns_correct_item() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("first"), make_session("second")];
        engine.add_char('f', &items);
        assert_eq!(engine.selected_item().unwrap().display_name(), "first");
    }

    #[test]
    fn search_matches_resurrectable_sessions() {
        let mut engine = SearchEngine::default();
        let items = vec![make_resurrectable("dead-project")];
        engine.add_char('d', &items);
        engine.add_char('e', &items);
        assert_eq!(engine.results().len(), 1);
    }

    #[test]
    fn no_match_returns_empty() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("project")];
        engine.add_char('x', &items);
        engine.add_char('y', &items);
        engine.add_char('z', &items);
        assert!(engine.results().is_empty());
    }

    #[test]
    fn term_accumulates_characters() {
        let mut engine = SearchEngine::default();
        let items: Vec<DisplayItem> = vec![];
        engine.add_char('a', &items);
        engine.add_char('b', &items);
        engine.add_char('c', &items);
        assert_eq!(engine.term(), "abc");
    }

    #[test]
    fn sessions_preserve_input_order_in_search() {
        let mut engine = SearchEngine::default();
        let items = vec![
            make_session("z-session"),
            make_session("a-session"),
            make_session("m-session"),
        ];
        engine.add_char('s', &items);
        engine.add_char('e', &items);
        let names: Vec<&str> = engine.results().iter().map(|r| r.item.display_name()).collect();
        // Sessions keep their input order, not sorted by score or alphabetically
        assert_eq!(names, vec!["z-session", "a-session", "m-session"]);
    }

    #[test]
    fn directories_sorted_by_score_in_search() {
        let mut engine = SearchEngine::default();
        let items = vec![
            make_directory("/home/user/zzz-project", "zzz-project"),
            make_directory("/home/user/project", "project"),
        ];
        // "pro" should match "project" better than "zzz-project"
        engine.add_char('p', &items);
        engine.add_char('r', &items);
        engine.add_char('o', &items);
        if engine.results().len() == 2 {
            assert!(engine.results()[0].score >= engine.results()[1].score);
        }
    }

    #[test]
    fn re_search_updates_results_preserves_selection() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("alpha"), make_session("apex")];
        engine.add_char('a', &items);
        // Select second result
        engine.move_down();
        let selected_name = engine.selected_item().unwrap().display_name().to_string();

        // Re-search with an additional item
        let new_items = vec![
            make_session("alpha"),
            make_session("apex"),
            make_session("another"),
        ];
        engine.re_search(&new_items);

        // Previously selected item should still be selected
        assert_eq!(
            engine.selected_item().unwrap().display_name(),
            selected_name,
        );
    }

    #[test]
    fn re_search_noop_when_inactive() {
        let mut engine = SearchEngine::default();
        let items = vec![make_session("test")];
        engine.re_search(&items);
        assert!(engine.results().is_empty());
    }
}
