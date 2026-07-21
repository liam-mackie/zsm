use std::path::PathBuf;
use zellij_tile::prelude::LayoutInfo;

use crate::app::actions::{Direction, SearchAction};
use crate::domain::DisplayItem;
use crate::search::{SearchEngine, SelectionState};

// ─── MainState ───────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MainState {
    pub search: SearchEngine,
    pub selection: SelectionState,
}

impl MainState {
    pub fn navigate(&mut self, direction: Direction) {
        if self.search.is_active() {
            match direction {
                Direction::Up => self.search.move_up(),
                Direction::Down => self.search.move_down(),
                Direction::Top => self.search.move_to_top(),
            }
        } else {
            match direction {
                Direction::Up => self.selection.move_up(),
                Direction::Down => self.selection.move_down(),
                Direction::Top => self.selection.select_top(),
            }
        }
    }

    pub fn handle_search(&mut self, action: SearchAction, items: &[DisplayItem]) {
        match action {
            SearchAction::AddChar(c) => self.search.add_char(c, items),
            SearchAction::Backspace => self.search.backspace(items),
            SearchAction::Clear => self.search.clear(),
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        if self.search.is_active() {
            self.search.selected_index()
        } else {
            self.selection.index()
        }
    }

    pub fn refresh_selection(&mut self, count: usize) {
        self.selection.update_count(count);
    }
}

// ─── NewSessionState ─────────────────────────────────────────────────────────

pub struct NewSessionState {
    pub name: String,
    pub folder: Option<PathBuf>,
    pub layout_search: String,
    pub layout_selection: SelectionState,
    pub entering_name: bool,
}

impl NewSessionState {
    pub fn new(name: String, folder: Option<PathBuf>, layout_count: usize) -> Self {
        let mut layout_selection = SelectionState::default();
        layout_selection.update_count(layout_count);
        Self {
            name,
            folder,
            layout_search: String::new(),
            layout_selection,
            entering_name: false,
        }
    }

    pub fn navigate(&mut self, direction: Direction) {
        if !self.entering_name {
            match direction {
                Direction::Up => self.layout_selection.move_up(),
                Direction::Down => self.layout_selection.move_down(),
                Direction::Top => self.layout_selection.select_top(),
            }
        }
    }

    pub fn handle_search(&mut self, action: SearchAction, layouts: &[LayoutInfo]) {
        match action {
            SearchAction::AddChar(c) => {
                self.layout_search.push(c);
                let count = self.filtered_layouts(layouts).len();
                self.layout_selection.update_count(count);
            }
            SearchAction::Backspace => {
                self.layout_search.pop();
                let count = self.filtered_layouts(layouts).len();
                self.layout_selection.update_count(count);
            }
            SearchAction::Clear => {
                self.layout_search.clear();
                // Reset count to full list so selection is valid if focus returns to layout selection.
                self.layout_selection.update_count(layouts.len());
                self.entering_name = true;
            }
        }
    }

    pub fn filtered_layouts(&self, layouts: &[LayoutInfo]) -> Vec<LayoutInfo> {
        if self.layout_search.is_empty() {
            layouts.to_vec()
        } else {
            let term = self.layout_search.to_lowercase();
            layouts
                .iter()
                .filter(|l| l.name().to_lowercase().contains(&term))
                .cloned()
                .collect()
        }
    }
}

// ─── ScreenState ─────────────────────────────────────────────────────────────

pub enum ScreenState {
    // Boxed: MainState (via SearchEngine's matcher) dwarfs NewSessionState
    Main(Box<MainState>),
    NewSession(NewSessionState),
}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState::Main(Box::default())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session_item(name: &str) -> DisplayItem {
        DisplayItem::ExistingSession {
            name: name.to_string(),
            directory: None,
            is_current: false,
        }
    }

    fn make_layout(name: &str) -> LayoutInfo {
        LayoutInfo::BuiltIn(name.to_string())
    }

    // ── MainState ──

    #[test]
    fn main_state_navigate_down_increments_selection() {
        let mut state = MainState::default();
        state.refresh_selection(3);
        state.navigate(Direction::Down);
        assert_eq!(state.selected_index(), Some(0));
        state.navigate(Direction::Down);
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn main_state_navigate_up_wraps() {
        let mut state = MainState::default();
        state.refresh_selection(3);
        state.navigate(Direction::Up);
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn main_state_search_active_navigate_moves_search_selection() {
        let mut state = MainState::default();
        let items = vec![make_session_item("aaa"), make_session_item("aab")];
        state.handle_search(SearchAction::AddChar('a'), &items);
        assert!(state.search.is_active());
        assert_eq!(state.selected_index(), Some(0));
        state.navigate(Direction::Down);
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn main_state_selected_index_none_when_empty() {
        let state = MainState::default();
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn main_state_search_add_char_filters() {
        let mut state = MainState::default();
        let items = vec![make_session_item("project"), make_session_item("other")];
        state.handle_search(SearchAction::AddChar('p'), &items);
        assert!(state.search.is_active());
        assert_eq!(state.search.results().len(), 1);
        assert_eq!(state.search.results()[0].item.display_name(), "project");
    }

    #[test]
    fn main_state_search_backspace_updates() {
        let mut state = MainState::default();
        let items = vec![make_session_item("project")];
        state.handle_search(SearchAction::AddChar('p'), &items);
        state.handle_search(SearchAction::Backspace, &items);
        assert!(!state.search.is_active());
    }

    #[test]
    fn main_state_search_clear_resets() {
        let mut state = MainState::default();
        let items = vec![make_session_item("project")];
        state.handle_search(SearchAction::AddChar('p'), &items);
        state.handle_search(SearchAction::Clear, &items);
        assert!(!state.search.is_active());
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn main_state_refresh_selection_updates_count() {
        let mut state = MainState::default();
        state.refresh_selection(5);
        state.navigate(Direction::Down);
        assert_eq!(state.selected_index(), Some(0));
    }

    // ── NewSessionState ──

    #[test]
    fn new_session_state_new_starts_in_layout_selection() {
        let state = NewSessionState::new("myapp".to_string(), None, 3);
        assert!(!state.entering_name);
        assert_eq!(state.name, "myapp");
        assert!(state.folder.is_none());
        assert!(state.layout_search.is_empty());
    }

    #[test]
    fn new_session_navigate_down_moves_layout_selection() {
        let mut state = NewSessionState::new("s".to_string(), None, 3);
        state.navigate(Direction::Down);
        assert_eq!(state.layout_selection.index(), Some(0));
        state.navigate(Direction::Down);
        assert_eq!(state.layout_selection.index(), Some(1));
    }

    #[test]
    fn new_session_navigate_ignored_when_entering_name() {
        let mut state = NewSessionState::new("s".to_string(), None, 3);
        state.entering_name = true;
        state.navigate(Direction::Down);
        assert_eq!(state.layout_selection.index(), None);
    }

    #[test]
    fn new_session_handle_search_adds_to_layout_search() {
        let layouts = vec![make_layout("default"), make_layout("compact")];
        let mut state = NewSessionState::new("s".to_string(), None, layouts.len());
        state.handle_search(SearchAction::AddChar('d'), &layouts);
        assert_eq!(state.layout_search, "d");
    }

    #[test]
    fn new_session_handle_search_clear_resets_to_name_entry() {
        let layouts = vec![make_layout("default")];
        let mut state = NewSessionState::new("s".to_string(), None, layouts.len());
        state.handle_search(SearchAction::AddChar('d'), &layouts);
        state.handle_search(SearchAction::Clear, &layouts);
        assert!(state.layout_search.is_empty());
        assert!(state.entering_name);
    }

    #[test]
    fn new_session_filtered_layouts_returns_all_when_empty_search() {
        let layouts = vec![make_layout("default"), make_layout("compact")];
        let state = NewSessionState::new("s".to_string(), None, layouts.len());
        let filtered = state.filtered_layouts(&layouts);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn new_session_filtered_layouts_filters_by_name() {
        let layouts = vec![make_layout("default"), make_layout("compact")];
        let mut state = NewSessionState::new("s".to_string(), None, layouts.len());
        state.layout_search = "comp".to_string();
        let filtered = state.filtered_layouts(&layouts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "compact");
    }

    #[test]
    fn new_session_filtered_layouts_case_insensitive() {
        let layouts = vec![make_layout("Default"), make_layout("compact")];
        let mut state = NewSessionState::new("s".to_string(), None, layouts.len());
        state.layout_search = "default".to_string();
        let filtered = state.filtered_layouts(&layouts);
        assert_eq!(filtered.len(), 1);
    }
}
