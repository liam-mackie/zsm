use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::{LayoutInfo, Palette, SessionInfo};

use crate::data::{DirectoryStore, DisplayList, SessionStore};
use crate::domain::{Config, Directory, DisplayItem, TargetMode};
use crate::input::{InputContext, InputHandler};
use crate::integrations::{zellij_delete_dead_session, zellij_pipe_message_to_plugin};
use crate::naming::SessionNameGenerator;
use crate::target::{create_target, Target};

use super::actions::{Direction, SearchAction};
use super::screen_state::{NewSessionState, ScreenState};
use super::{Action, Screen};

pub struct AppState {
    config: Config,
    target: Box<dyn Target>,
    sessions: SessionStore,
    directories: DirectoryStore,
    screen: ScreenState,
    colors: Option<Palette>,
    error: Option<String>,
    current_session: Option<String>,
    layouts: Vec<LayoutInfo>,
    request_ids: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Config::default(),
            target: create_target(Default::default()),
            sessions: SessionStore::default(),
            directories: DirectoryStore::default(),
            screen: ScreenState::default(),
            colors: None,
            error: None,
            current_session: None,
            layouts: Vec::new(),
            request_ids: Vec::new(),
        }
    }
}

impl AppState {
    pub fn initialize(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_zellij_config(&configuration);
        self.target = create_target(self.config.mode);
    }

    pub fn update_sessions(&mut self, infos: Vec<SessionInfo>) {
        let prev_selected = self.selected_item().map(|i| i.display_name().to_string());
        for info in &infos {
            if info.is_current_session {
                self.current_session = Some(info.name.clone());
                self.layouts = info.available_layouts.clone();
                break;
            }
        }
        self.sessions.update(infos);
        self.refresh_selection(prev_selected);
    }

    pub fn update_resurrectable(&mut self, sessions: Vec<(String, std::time::Duration)>) {
        let prev_selected = self.selected_item().map(|i| i.display_name().to_string());
        self.sessions.update_resurrectable(sessions);
        self.refresh_selection(prev_selected);
    }

    pub fn update_directories(&mut self, directories: Vec<Directory>) {
        let prev_selected = self.selected_item().map(|i| i.display_name().to_string());
        let generator = SessionNameGenerator::new(
            self.config.session_separator.clone(),
            self.config.base_paths.clone(),
        );
        self.directories.update(directories, &generator);
        self.refresh_selection(prev_selected);
    }

    pub fn handle_key(&mut self, key: zellij_tile::prelude::KeyWithModifier) -> bool {
        if self.error.is_some() {
            self.error = None;
            return true;
        }

        let context = InputContext {
            is_searching: match &self.screen {
                ScreenState::Main(s) => s.search.is_active(),
                _ => false,
            },
            has_pending_deletion: self.sessions.pending_deletion().is_some(),
            entering_name: match &self.screen {
                ScreenState::NewSession(s) => s.entering_name,
                _ => true,
            },
        };

        let action = InputHandler::handle(key, self.screen(), &context);
        self.apply_action(action)
    }

    pub fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::Navigate(dir) => self.navigate(dir),
            Action::Select => self.handle_select(),
            Action::QuickCreate => self.handle_quick_create(),
            Action::Delete => self.handle_delete(),
            Action::ConfirmDelete => self.confirm_delete(),
            Action::CancelDelete => self.cancel_delete(),
            Action::Search(sa) => self.handle_search(sa),
            Action::GoToScreen(s) => self.go_to_screen(s),
            Action::OpenFilepicker => self.open_filepicker(),
            Action::ClearFolder => self.clear_folder(),
            Action::Refresh => self.request_refresh(),
            Action::Hide => {
                if !self.config.dev_mode {
                    self.target.hide();
                }
                false
            }
            Action::ToggleMode => {
                if self.config.dev_mode {
                    self.toggle_mode()
                } else {
                    false
                }
            }
            Action::None => false,
        }
    }

    fn navigate(&mut self, direction: Direction) -> bool {
        match &mut self.screen {
            ScreenState::Main(s) => s.navigate(direction),
            ScreenState::NewSession(s) => s.navigate(direction),
        }
        true
    }

    fn handle_select(&mut self) -> bool {
        match self.screen() {
            Screen::Main => self.select_main_item(),
            Screen::NewSession => self.create_session(),
        }
    }

    fn select_main_item(&mut self) -> bool {
        let item = self.selected_item();
        match item {
            Some(DisplayItem::ExistingSession { name, .. })
            | Some(DisplayItem::ResurrectableSession { name, .. }) => {
                self.target.switch_to(&name);
                if !self.config.dev_mode {
                    self.target.hide();
                }
            }
            Some(DisplayItem::Directory {
                path, session_name, ..
            }) => {
                let name = self
                    .sessions
                    .generate_incremented_name(&session_name, &self.config.session_separator);
                let layout_count = self.layouts.len();
                self.screen = ScreenState::NewSession(NewSessionState::new(
                    name,
                    Some(PathBuf::from(path)),
                    layout_count,
                ));
            }
            None => {}
        }
        true
    }

    fn create_session(&mut self) -> bool {
        // First pass: handle entering_name toggle
        if let ScreenState::NewSession(s) = &mut self.screen {
            if s.entering_name {
                s.entering_name = false;
                s.layout_selection.update_count(self.layouts.len());
                return true;
            }
        }

        // Second pass: extract values then act (avoids borrow conflict on self.screen)
        let (name, folder, layout) = if let ScreenState::NewSession(s) = &self.screen {
            let filtered = s.filtered_layouts(&self.layouts);
            let layout = s.layout_selection.index().and_then(|i| filtered.get(i).cloned());
            (s.name.clone(), s.folder.clone(), layout)
        } else {
            return true;
        };

        if let Some(folder) = folder {
            self.target.create(&name, &folder, layout);
            if !self.config.dev_mode {
                self.target.hide();
            }
        }

        self.screen = ScreenState::default();
        true
    }

    fn handle_quick_create(&mut self) -> bool {
        match self.screen() {
            Screen::Main => {
                if let Some(DisplayItem::Directory {
                    path, session_name, ..
                }) = self.selected_item()
                {
                    let name = self
                        .sessions
                        .generate_incremented_name(&session_name, &self.config.session_separator);
                    let layout = self.config.default_layout.as_ref().and_then(|n| {
                        self.layouts.iter().find(|l| l.name() == n).cloned()
                    });
                    self.target.create(&name, &PathBuf::from(path), layout);
                    if !self.config.dev_mode {
                        self.target.hide();
                    }
                }
            }
            Screen::NewSession => {
                let (name, folder) = if let ScreenState::NewSession(s) = &self.screen {
                    (s.name.clone(), s.folder.clone())
                } else {
                    return true;
                };
                if let Some(folder) = folder {
                    let layout = self.config.default_layout.as_ref().and_then(|n| {
                        self.layouts.iter().find(|l| l.name() == n).cloned()
                    });
                    self.target.create(&name, &folder, layout);
                    if !self.config.dev_mode {
                        self.target.hide();
                    }
                }
            }
        }
        true
    }

    fn handle_delete(&mut self) -> bool {
        if let Some(DisplayItem::ExistingSession { name, .. })
        | Some(DisplayItem::ResurrectableSession { name, .. }) = self.selected_item()
        {
            self.sessions.start_deletion(name);
        }
        true
    }

    fn confirm_delete(&mut self) -> bool {
        if let Some(name) = self.sessions.confirm_deletion() {
            let is_resurrectable = self.sessions.is_resurrectable(&name);
            let result = if is_resurrectable {
                zellij_delete_dead_session(&name);
                Ok(())
            } else {
                self.target.delete(&name)
            };
            if let Err(msg) = result {
                self.set_error(msg);
            }
        }
        true
    }

    fn cancel_delete(&mut self) -> bool {
        self.sessions.cancel_deletion();
        true
    }

    fn handle_search(&mut self, action: SearchAction) -> bool {
        match &mut self.screen {
            ScreenState::Main(s) => {
                let items = DisplayList::build(
                    &self.sessions,
                    &self.directories,
                    self.config.show_resurrectable_sessions,
                    &self.config.session_separator,
                );
                s.handle_search(action, &items);
            }
            ScreenState::NewSession(s) if s.entering_name => match action {
                SearchAction::AddChar(c) => s.name.push(c),
                SearchAction::Backspace => {
                    s.name.pop();
                }
                SearchAction::Clear => s.name.clear(),
            },
            ScreenState::NewSession(s) => {
                s.handle_search(action, &self.layouts);
            }
        }
        true
    }

    fn go_to_screen(&mut self, screen: Screen) -> bool {
        match screen {
            Screen::Main => {
                self.screen = ScreenState::default();
            }
            Screen::NewSession => {
                // Creates a blank NewSessionState (no folder). The intended flow is:
                // GoToScreen(NewSession) → OpenFilepicker → set_new_session_folder(Some(...)) → Select.
                // Pressing Enter before a folder is set is a silent no-op (no target.create call).
                let layout_count = self.layouts.len();
                self.screen = ScreenState::NewSession(NewSessionState::new(
                    String::new(),
                    None,
                    layout_count,
                ));
            }
        }
        true
    }

    fn open_filepicker(&mut self) -> bool {
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        self.request_ids.push(request_id.clone());
        // Cancelled filepickers never reply, so cap the backlog rather than leak.
        const MAX_PENDING_REQUESTS: usize = 8;
        if self.request_ids.len() > MAX_PENDING_REQUESTS {
            let excess = self.request_ids.len() - MAX_PENDING_REQUESTS;
            self.request_ids.drain(..excess);
        }

        let mut config = BTreeMap::new();
        config.insert("request_id".to_string(), request_id.clone());

        if let ScreenState::NewSession(s) = &self.screen {
            if let Some(folder) = &s.folder {
                config.insert("caller_cwd".to_string(), folder.to_string_lossy().to_string());
            }
        }

        let mut args = BTreeMap::new();
        args.insert("request_id".to_string(), request_id);

        zellij_pipe_message_to_plugin("filepicker", "filepicker", config, args, "Select folder...");
        true
    }

    fn clear_folder(&mut self) -> bool {
        if let ScreenState::NewSession(s) = &mut self.screen {
            s.folder = None;
        }
        true
    }

    fn request_refresh(&mut self) -> bool {
        true
    }

    fn toggle_mode(&mut self) -> bool {
        self.config.mode = match self.config.mode {
            TargetMode::Session => TargetMode::Tab,
            TargetMode::Tab => TargetMode::Session,
        };
        self.target = create_target(self.config.mode);
        true
    }

    fn refresh_selection(&mut self, prev_selected: Option<String>) {
        let items = DisplayList::build(
            &self.sessions,
            &self.directories,
            self.config.show_resurrectable_sessions,
            &self.config.session_separator,
        );
        if let ScreenState::Main(s) = &mut self.screen {
            // Re-run search with new data so results stay fresh and selection is preserved.
            s.search.re_search(&items);

            s.selection.update_count(items.len());
            // Restore selection by identity so data refreshes don't jump to a different item.
            if let Some(ref name) = prev_selected {
                if let Some(idx) = items.iter().position(|i| i.display_name() == name) {
                    s.selection.set_index(Some(idx));
                }
            }
        }
    }

    pub fn display_items(&self) -> Vec<DisplayItem> {
        match &self.screen {
            ScreenState::Main(s) => {
                if s.search.is_active() {
                    s.search.results().iter().map(|r| r.item.clone()).collect()
                } else {
                    DisplayList::build(
                        &self.sessions,
                        &self.directories,
                        self.config.show_resurrectable_sessions,
                        &self.config.session_separator,
                    )
                }
            }
            ScreenState::NewSession(_) => Vec::new(),
        }
    }

    pub fn selected_item(&self) -> Option<DisplayItem> {
        let items = self.display_items();
        let index = match &self.screen {
            ScreenState::Main(s) => s.selected_index(),
            ScreenState::NewSession(_) => return None,
        };
        index.and_then(|i| items.get(i).cloned())
    }

    pub fn filtered_layouts(&self) -> Vec<LayoutInfo> {
        match &self.screen {
            ScreenState::NewSession(s) => s.filtered_layouts(&self.layouts),
            _ => Vec::new(),
        }
    }

    // ─── Screen accessor ────────────────────────────────────────────────────

    pub fn screen(&self) -> Screen {
        match &self.screen {
            ScreenState::Main(_) => Screen::Main,
            ScreenState::NewSession(_) => Screen::NewSession,
        }
    }

    // ─── Shared accessors ───────────────────────────────────────────────────

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn sessions(&self) -> &SessionStore {
        &self.sessions
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn colors(&self) -> Option<Palette> {
        self.colors
    }

    pub fn set_colors(&mut self, colors: Palette) {
        self.colors = Some(colors);
    }

    // ─── Main screen accessors ──────────────────────────────────────────────

    /// Search term for the main screen search bar. Returns `""` if not on main screen.
    pub fn search_term(&self) -> &str {
        match &self.screen {
            ScreenState::Main(s) => s.search.term(),
            _ => "",
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        match &self.screen {
            ScreenState::Main(s) => s.selected_index(),
            _ => None,
        }
    }

    /// Returns fuzzy match character indices for a search result at the given position.
    /// Only populated when search is active.
    pub fn match_indices(&self, index: usize) -> Option<&[usize]> {
        match &self.screen {
            ScreenState::Main(s) if s.search.is_active() => {
                s.search.results().get(index).map(|r| r.indices.as_slice())
            }
            _ => None,
        }
    }

    // ─── NewSession screen accessors ────────────────────────────────────────

    pub fn entering_name(&self) -> bool {
        match &self.screen {
            ScreenState::NewSession(s) => s.entering_name,
            _ => false,
        }
    }

    pub fn new_session_name(&self) -> &str {
        match &self.screen {
            ScreenState::NewSession(s) => &s.name,
            _ => "",
        }
    }

    pub fn new_session_folder(&self) -> Option<&PathBuf> {
        match &self.screen {
            ScreenState::NewSession(s) => s.folder.as_ref(),
            _ => None,
        }
    }

    pub fn layout_search(&self) -> &str {
        match &self.screen {
            ScreenState::NewSession(s) => &s.layout_search,
            _ => "",
        }
    }

    pub fn layout_selection_index(&self) -> Option<usize> {
        match &self.screen {
            ScreenState::NewSession(s) => s.layout_selection.index(),
            _ => None,
        }
    }

    pub fn set_new_session_folder(&mut self, folder: Option<PathBuf>) {
        if let ScreenState::NewSession(s) = &mut self.screen {
            s.folder = folder;
        }
    }

    // ─── Request ID tracking ────────────────────────────────────────────────

    pub fn is_valid_request_id(&self, id: &str) -> bool {
        self.request_ids.contains(&id.to_string())
    }

    pub fn remove_request_id(&mut self, id: &str) {
        self.request_ids.retain(|r| r != id);
    }

    pub fn should_refresh(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Directory;
    use crate::target::MockTarget;

    fn make_app_with_mock() -> (AppState, MockTarget) {
        let mock = MockTarget::new();
        let state = AppState {
            target: Box::new(mock.clone()),
            ..Default::default()
        };
        (state, mock)
    }

    fn make_session_info(name: &str, is_current: bool) -> SessionInfo {
        SessionInfo {
            name: name.to_string(),
            is_current_session: is_current,
            ..Default::default()
        }
    }

    fn make_directory(path: &str, ranking: f64) -> Directory {
        Directory {
            path: path.to_string(),
            ranking,
            session_name: String::new(),
        }
    }

    // Screen and initialization tests
    #[test]
    fn default_screen_is_main() {
        let state = AppState::default();
        assert_eq!(state.screen(), Screen::Main);
    }

    #[test]
    fn initialize_sets_config() {
        let mut state = AppState::default();
        let mut config = BTreeMap::new();
        config.insert("session_separator".to_string(), "-".to_string());
        state.initialize(config);
        assert_eq!(state.config().session_separator, "-");
    }

    // Navigation tests
    #[test]
    fn navigate_down_updates_selection() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("s1", false),
            make_session_info("s2", false),
        ]);
        state.apply_action(Action::Navigate(Direction::Down));
        assert_eq!(state.selected_index(), Some(0));
        state.apply_action(Action::Navigate(Direction::Down));
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn navigate_up_updates_selection() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("s1", false),
            make_session_info("s2", false),
        ]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Navigate(Direction::Up));
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn selection_preserved_across_session_update() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("alpha", false),
            make_session_info("beta", false),
            make_session_info("gamma", false),
        ]);
        // Select "beta" (index 0=alpha, 1=beta)
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Navigate(Direction::Down));
        assert_eq!(state.selected_item().unwrap().display_name(), "beta");

        // Re-push sessions in a different order — "beta" should stay selected
        state.update_sessions(vec![
            make_session_info("gamma", false),
            make_session_info("alpha", false),
            make_session_info("beta", false),
        ]);
        assert_eq!(state.selected_item().unwrap().display_name(), "beta");
    }

    #[test]
    fn sessions_current_first_rest_preserve_order() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("zebra", false),
            make_session_info("alpha", true),
            make_session_info("middle", false),
        ]);
        let items = state.display_items();
        assert_eq!(items[0].display_name(), "alpha"); // current, pinned first
        assert_eq!(items[1].display_name(), "zebra"); // original order preserved
        assert_eq!(items[2].display_name(), "middle");
    }

    // Selection tests
    #[test]
    fn selecting_session_switches_and_hides() {
        let (mut state, mock) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("target-session", false)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Select);
        assert_eq!(mock.switched_to_sessions(), vec!["target-session"]);
        assert_eq!(mock.hide_count(), 1);
    }

    #[test]
    fn selecting_directory_goes_to_new_session_screen() {
        let (mut state, _) = make_app_with_mock();
        state.update_directories(vec![make_directory("/home/user/project", 100.0)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Select);
        assert_eq!(state.screen(), Screen::NewSession);
        assert!(!state.new_session_name().is_empty());
    }

    // Deletion tests
    #[test]
    fn delete_action_starts_pending_deletion() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("to-delete", false)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Delete);
        assert_eq!(state.sessions().pending_deletion(), Some("to-delete"));
    }

    #[test]
    fn confirm_delete_calls_target_delete() {
        let (mut state, mock) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("to-delete", false)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Delete);
        state.apply_action(Action::ConfirmDelete);
        assert_eq!(mock.deleted_sessions(), vec!["to-delete"]);
    }

    #[test]
    fn cancel_delete_clears_pending() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("to-delete", false)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Delete);
        state.apply_action(Action::CancelDelete);
        assert!(state.sessions().pending_deletion().is_none());
    }

    #[test]
    fn confirm_delete_sets_error_on_target_failure() {
        struct FailTarget;
        impl crate::target::Target for FailTarget {
            fn create(&self, _name: &str, _directory: &std::path::Path, _layout: Option<zellij_tile::prelude::LayoutInfo>) {}
            fn switch_to(&self, _name: &str) {}
            fn delete(&self, _name: &str) -> Result<(), String> {
                Err("Tab deletion is not supported".to_string())
            }
            fn hide(&self) {}
        }

        let mut state = AppState {
            target: Box::new(FailTarget),
            ..Default::default()
        };
        state.update_sessions(vec![make_session_info("to-delete", false)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Delete);
        state.apply_action(Action::ConfirmDelete);
        assert!(state.error().is_some());
        assert!(state.error().unwrap().contains("not supported"));
    }

    // Search tests
    #[test]
    fn search_filters_display_items() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("project", false),
            make_session_info("other", false),
        ]);
        state.apply_action(Action::Search(SearchAction::AddChar('p')));
        assert!(!state.search_term().is_empty());
        let items = state.display_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].display_name(), "project");
    }

    #[test]
    fn search_clear_resets() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("test", false)]);
        state.apply_action(Action::Search(SearchAction::AddChar('t')));
        assert!(!state.search_term().is_empty());
        state.apply_action(Action::Search(SearchAction::Clear));
        assert!(state.search_term().is_empty());
    }

    // Screen transition tests
    #[test]
    fn go_to_screen_changes_screen() {
        let (mut state, _) = make_app_with_mock();
        assert_eq!(state.screen(), Screen::Main);
        state.apply_action(Action::GoToScreen(Screen::NewSession));
        assert_eq!(state.screen(), Screen::NewSession);
    }

    #[test]
    fn go_to_main_clears_new_session_state() {
        let (mut state, _) = make_app_with_mock();
        state.update_directories(vec![make_directory("/home/test", 100.0)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::Select);
        assert_eq!(state.screen(), Screen::NewSession);
        assert!(!state.new_session_name().is_empty());
        state.apply_action(Action::GoToScreen(Screen::Main));
        assert_eq!(state.screen(), Screen::Main);
        assert!(state.new_session_name().is_empty());
    }

    // Hide test
    #[test]
    fn hide_calls_target_hide() {
        let (mut state, mock) = make_app_with_mock();
        state.apply_action(Action::Hide);
        assert_eq!(mock.hide_count(), 1);
    }

    // Toggle mode tests
    #[test]
    fn toggle_mode_switches_session_to_tab_in_dev_mode() {
        let (mut state, _) = make_app_with_mock();
        let mut config = BTreeMap::new();
        config.insert("dev_mode".to_string(), "true".to_string());
        state.initialize(config);
        assert_eq!(state.config().mode, TargetMode::Session);
        state.apply_action(Action::ToggleMode);
        assert_eq!(state.config().mode, TargetMode::Tab);
    }

    #[test]
    fn toggle_mode_switches_tab_to_session_in_dev_mode() {
        let (mut state, _) = make_app_with_mock();
        let mut config = BTreeMap::new();
        config.insert("dev_mode".to_string(), "true".to_string());
        config.insert("mode".to_string(), "tab".to_string());
        state.initialize(config);
        assert_eq!(state.config().mode, TargetMode::Tab);
        state.apply_action(Action::ToggleMode);
        assert_eq!(state.config().mode, TargetMode::Session);
    }

    #[test]
    fn toggle_mode_ignored_when_not_in_dev_mode() {
        let (mut state, _) = make_app_with_mock();
        assert_eq!(state.config().mode, TargetMode::Session);
        state.apply_action(Action::ToggleMode);
        assert_eq!(state.config().mode, TargetMode::Session);
    }

    // Quick create tests
    #[test]
    fn quick_create_on_directory_creates_session() {
        let (mut state, mock) = make_app_with_mock();
        state.update_directories(vec![make_directory("/home/user/project", 100.0)]);
        state.apply_action(Action::Navigate(Direction::Down));
        state.apply_action(Action::QuickCreate);
        let created = mock.created_sessions();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].0, "project");
        assert_eq!(mock.hide_count(), 1);
    }

    // Display items tests
    #[test]
    fn display_items_combines_sessions_and_directories() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![make_session_info("session1", true)]);
        state.update_directories(vec![make_directory("/home/dir", 100.0)]);
        let items = state.display_items();
        assert_eq!(items.len(), 2);
    }

    // Error handling tests
    #[test]
    fn set_error_stores_error() {
        let (mut state, _) = make_app_with_mock();
        state.set_error("Test error".to_string());
        assert_eq!(state.error(), Some("Test error"));
    }

    #[test]
    fn handle_key_clears_error() {
        let (mut state, _) = make_app_with_mock();
        state.set_error("Test error".to_string());
        state.handle_key(zellij_tile::prelude::KeyWithModifier::new(
            zellij_tile::prelude::BareKey::Esc,
        ));
        assert!(state.error().is_none());
    }

    // Selected item tests
    #[test]
    fn selected_item_returns_correct() {
        let (mut state, _) = make_app_with_mock();
        state.update_sessions(vec![
            make_session_info("first", false),
            make_session_info("second", false),
        ]);
        state.apply_action(Action::Navigate(Direction::Down));
        let selected = state.selected_item();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().display_name(), "first");
    }

    #[test]
    fn selected_item_none_when_empty() {
        let state = AppState::default();
        assert!(state.selected_item().is_none());
    }
}
