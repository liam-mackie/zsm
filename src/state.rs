use std::collections::{BTreeMap, HashSet};
use std::time::Duration;
use zellij_tile::prelude::*;

use crate::config::Config;
use crate::new_session_info::NewSessionInfo;
use crate::session::{SessionAction, SessionItem, SessionManager};
use crate::zoxide::{SearchEngine, ZoxideDirectory};

/// The main plugin state
#[derive(Default)]
pub struct PluginState {
    /// Plugin configuration
    config: Config,
    /// Session manager
    session_manager: SessionManager,
    /// Zoxide directories (managed separately from sessions)
    zoxide_directories: Vec<ZoxideDirectory>,
    /// Search engine for fuzzy finding
    search_engine: SearchEngine,
    /// New session creation component
    new_session_info: NewSessionInfo,
    /// Current active screen
    active_screen: ActiveScreen,
    /// Error message to display
    error: Option<String>,
    /// Color scheme
    colors: Option<Palette>,
    /// Current session name
    current_session_name: Option<String>,
    /// Request IDs for plugin communication
    request_ids: Vec<String>,
    /// Selected index in main list (when not searching)
    selected_index: Option<usize>,
}

/// Represents the different screens in the plugin
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ActiveScreen {
    /// Main screen showing zoxide directories and sessions
    #[default]
    Main,
    /// New session creation screen
    NewSession,
}

impl PluginState {
    /// Initialize plugin with configuration
    pub fn initialize(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_zellij_config(&configuration);
    }

    /// Update session information with stability tracking
    /// Returns true if the session list actually changed
    pub fn update_sessions(&mut self, sessions: Vec<SessionInfo>) -> bool {
        // Store current session name and layouts from the incoming data
        for session in &sessions {
            if session.is_current_session {
                self.current_session_name = Some(session.name.clone());
                self.new_session_info
                    .update_layout_list(session.available_layouts.clone());
                break;
            }
        }

        // Use stable update that handles Zellij's inconsistent data
        let changed = self.session_manager.update_sessions_stable(sessions);
        if changed {
            self.update_search_if_needed();
        }
        changed
    }

    /// Update session information for resurrectable sessions
    /// Returns true if the resurrectable session list actually changed
    pub fn update_resurrectable_sessions(
        &mut self,
        resurrectable_sessions: Vec<(String, Duration)>,
    ) -> bool {
        let changed = self
            .session_manager
            .update_resurrectable_stable(resurrectable_sessions);
        if changed {
            self.update_search_if_needed();
        }
        changed
    }

    /// Update zoxide directories (managed separately from sessions)
    pub fn update_zoxide_directories(&mut self, directories: Vec<ZoxideDirectory>) {
        self.zoxide_directories = directories;
        self.update_search_if_needed();
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyWithModifier) -> bool {
        // Clear error on any key press
        if self.error.is_some() {
            self.error = None;
            return true;
        }

        // Handle session deletion confirmation
        if let Some(session_name) = self
            .session_manager
            .pending_deletion()
            .map(|s| s.to_string())
        {
            return self.handle_deletion_confirmation(key, &session_name);
        }

        match self.active_screen {
            ActiveScreen::Main => self.handle_main_screen_key(key),
            ActiveScreen::NewSession => self.handle_new_session_key(key),
        }
    }

    /// Get current screen
    pub fn active_screen(&self) -> ActiveScreen {
        self.active_screen
    }

    /// Get items to display (combined sessions and zoxide directories)
    pub fn display_items(&self) -> Vec<SessionItem> {
        if self.search_engine.is_searching() {
            // Return items from search results
            self.search_engine
                .results()
                .iter()
                .map(|result| result.item.clone())
                .collect()
        } else {
            self.combined_items()
        }
    }

    /// Combine sessions and zoxide directories for display
    fn combined_items(&self) -> Vec<SessionItem> {
        let mut items = Vec::new();
        let mut added_session_names = HashSet::new();

        // First, add existing sessions that match zoxide directories (including incremented ones)
        for session in self.session_manager.sessions() {
            if let Some(zoxide_dir) = self.find_matching_zoxide_dir(&session.name) {
                items.push(SessionItem::ExistingSession {
                    name: session.name.clone(),
                    directory: zoxide_dir.directory.clone(),
                    is_current: session.is_current_session,
                });
                added_session_names.insert(session.name.clone());
            } else if self.config.show_all_sessions {
                // Session didn't match any zoxide dir, but show_all_sessions is enabled
                items.push(SessionItem::ExistingSession {
                    name: session.name.clone(),
                    directory: String::new(),
                    is_current: session.is_current_session,
                });
                added_session_names.insert(session.name.clone());
            }
        }

        // Add resurrectable sessions if configured to show them
        if self.config.show_resurrectable_sessions {
            for (name, duration) in self.session_manager.resurrectable_sessions() {
                // Skip if already added as existing session
                if added_session_names.contains(name) {
                    continue;
                }

                let matches_zoxide = self.find_matching_zoxide_dir(name).is_some();
                if matches_zoxide || self.config.show_all_sessions {
                    items.push(SessionItem::ResurrectableSession {
                        name: name.clone(),
                        duration: *duration,
                    });
                }
            }
        }

        // Then add all zoxide directories (always show directories, even if sessions exist)
        for dir in &self.zoxide_directories {
            items.push(SessionItem::Directory {
                path: dir.directory.clone(),
                session_name: dir.session_name.clone(),
            });
        }

        items
    }

    /// Check if session name is an incremented version of base name
    fn is_incremented_session(&self, session_name: &str, base_name: &str) -> bool {
        if session_name.len() <= base_name.len() || !session_name.starts_with(base_name) {
            return false;
        }

        let remainder = &session_name[base_name.len()..];
        if !remainder.starts_with(&self.config.session_separator) {
            return false;
        }

        let number_part = &remainder[self.config.session_separator.len()..];
        number_part.parse::<u32>().is_ok() && !number_part.is_empty()
    }

    /// Check if a session name matches any zoxide directory (exact or incremented name)
    /// Returns the matching directory if found
    fn find_matching_zoxide_dir(&self, session_name: &str) -> Option<&ZoxideDirectory> {
        self.zoxide_directories.iter().find(|zoxide_dir| {
            session_name == zoxide_dir.session_name
                || self.is_incremented_session(session_name, &zoxide_dir.session_name)
        })
    }

    /// Get search engine (for UI rendering)
    pub fn search_engine(&self) -> &SearchEngine {
        &self.search_engine
    }

    /// Get new session info (for UI rendering)
    pub fn new_session_info(&self) -> &NewSessionInfo {
        &self.new_session_info
    }

    /// Get session manager (for UI rendering)
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get selected index for main screen
    pub fn selected_index(&self) -> Option<usize> {
        if self.search_engine.is_searching() {
            self.search_engine.selected_index()
        } else {
            self.selected_index
        }
    }

    /// Get colors
    pub fn colors(&self) -> Option<Palette> {
        self.colors
    }

    /// Set colors
    pub fn set_colors(&mut self, colors: Palette) {
        self.colors = Some(colors);
    }

    /// Show error message
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Get current error
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<SessionItem> {
        if self.search_engine.is_searching() {
            self.search_engine.selected_item().cloned()
        } else {
            let items = self.display_items();
            self.selected_index.and_then(|i| items.get(i).cloned())
        }
    }

    /// Handle main screen key input
    fn handle_main_screen_key(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Up if key.has_no_modifiers() => {
                self.move_selection_up();
                true
            }
            BareKey::Down if key.has_no_modifiers() => {
                self.move_selection_down();
                true
            }
            BareKey::Enter if key.has_no_modifiers() => {
                self.handle_item_selection();
                true
            }
            BareKey::Enter if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                self.handle_quick_session_creation();
                true
            }
            BareKey::Delete if key.has_no_modifiers() => {
                self.handle_delete_key();
                true
            }
            BareKey::Char(c) if key.has_no_modifiers() && c != '\n' => {
                let items = self.combined_items(); // Always use full item list, not search results
                self.search_engine.add_char(c, &items);
                true
            }
            BareKey::Backspace if key.has_no_modifiers() => {
                let items = self.combined_items(); // Always use full item list, not search results
                self.search_engine.backspace(&items);
                true
            }
            BareKey::Esc if key.has_no_modifiers() => {
                if self.search_engine.is_searching() {
                    self.search_engine.clear();
                    true
                } else {
                    hide_self();
                    false
                }
            }
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                hide_self();
                false
            }
            BareKey::Char('r') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                // reload zoxide directories
                self.fetch_zoxide_directories();
                true
            }
            _ => false,
        }
    }

    /// Handle new session screen key input
    fn handle_new_session_key(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Enter if key.has_no_modifiers() => {
                // Handle session creation
                self.new_session_info
                    .handle_selection(&self.current_session_name);
                self.active_screen = ActiveScreen::Main;
                true
            }
            BareKey::Enter if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                // Quick session creation with default layout
                if self.new_session_info.name().len() >= 108 {
                    self.set_error("Session name must be shorter than 108 bytes".to_string());
                } else if self.new_session_info.name().contains('/') {
                    self.set_error("Session name cannot contain '/'".to_string());
                } else {
                    self.new_session_info.handle_quick_session_creation(
                        &self.current_session_name,
                        &self.config.default_layout,
                    );
                    self.active_screen = ActiveScreen::Main;
                }
                true
            }
            BareKey::Esc if key.has_no_modifiers() => {
                // Special handling for Esc when entering session name - go back to main
                if self.new_session_info.entering_new_session_name()
                    && self.new_session_info.name().is_empty()
                {
                    self.active_screen = ActiveScreen::Main;
                } else {
                    // Let NewSessionInfo handle its own escape logic
                    self.new_session_info.handle_key(key);
                }
                true
            }
            BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                // Handle filepicker
                self.launch_filepicker();
                true
            }
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                // Clear session folder - don't delegate to NewSessionInfo
                self.new_session_info.set_folder(None);
                true
            }
            _ => {
                // Delegate other keys to NewSessionInfo component
                self.new_session_info.handle_key(key);
                true
            }
        }
    }

    /// Handle deletion confirmation
    fn handle_deletion_confirmation(&mut self, key: KeyWithModifier, _session_name: &str) -> bool {
        match key.bare_key {
            BareKey::Char('y') | BareKey::Char('Y') if key.has_no_modifiers() => {
                self.session_manager.confirm_deletion();
                true
            }
            BareKey::Char('n') | BareKey::Char('N') | BareKey::Esc if key.has_no_modifiers() => {
                self.session_manager.cancel_deletion();
                true
            }
            _ => false,
        }
    }

    /// Move selection up
    fn move_selection_up(&mut self) {
        if self.search_engine.is_searching() {
            self.search_engine.move_selection_up();
        } else {
            let items_len = self.display_items().len();
            if items_len == 0 {
                return;
            }

            if let Some(selected) = self.selected_index.as_mut() {
                if *selected == 0 {
                    *selected = items_len.saturating_sub(1);
                } else {
                    *selected = selected.saturating_sub(1);
                }
            } else {
                self.selected_index = Some(items_len.saturating_sub(1));
            }
        }
    }

    /// Move selection down
    fn move_selection_down(&mut self) {
        if self.search_engine.is_searching() {
            self.search_engine.move_selection_down();
        } else {
            let items_len = self.display_items().len();
            if items_len == 0 {
                return;
            }

            if let Some(selected) = self.selected_index.as_mut() {
                if *selected == items_len.saturating_sub(1) {
                    *selected = 0;
                } else {
                    *selected += 1;
                }
            } else {
                self.selected_index = Some(0);
            }
        }
    }

    /// Handle item selection (Enter key)
    fn handle_item_selection(&mut self) {
        // Get the selected item data before any mutable borrows
        let selected_item_data = self.selected_item().map(|item| match item {
            SessionItem::ExistingSession { name, .. } => (true, name, String::new()),
            SessionItem::Directory {
                session_name, path, ..
            } => (false, session_name, path),
            SessionItem::ResurrectableSession { name, .. } => (true, name, String::new()),
        });

        if let Some((is_session, name, path)) = selected_item_data {
            if is_session {
                // Switch to existing session
                self.session_manager
                    .execute_action(SessionAction::Switch(name));
                hide_self();
            } else {
                // Create new session with incremented name
                let incremented_name = self
                    .session_manager
                    .generate_incremented_name(&name, &self.config.session_separator);

                // Set up new session creation
                self.new_session_info.set_name(&incremented_name);
                self.new_session_info
                    .set_folder(Some(std::path::PathBuf::from(&path)));
                self.new_session_info.advance_to_layout_selection();
                self.active_screen = ActiveScreen::NewSession;
            }
        }
    }

    /// Handle delete key
    fn handle_delete_key(&mut self) {
        // Get the selected item data before any mutable borrows
        let selected_session_name = self.selected_item().and_then(|item| match item {
            SessionItem::ExistingSession { name, .. } => Some(name),
            SessionItem::ResurrectableSession { name, .. } => Some(name),
            _ => None,
        });

        if let Some(session_name) = selected_session_name {
            self.session_manager.start_deletion(session_name);
        }
    }

    /// Update search if currently searching
    fn update_search_if_needed(&mut self) {
        if self.search_engine.is_searching() {
            let term = self.search_engine.search_term().to_string();
            let items = self.combined_items(); // Always use full item list, not search results
            self.search_engine.update_search(term, &items);
        }
    }

    /// Launch filepicker for new session folder selection
    fn launch_filepicker(&mut self) {
        use uuid::Uuid;
        use zellij_tile::prelude::{pipe_message_to_plugin, MessageToPlugin};

        let request_id = Uuid::new_v4();
        let mut config = BTreeMap::new();
        let mut args = BTreeMap::new();

        self.request_ids.push(request_id.to_string());

        // we insert this into the config so that a new plugin will be opened (the plugin's
        // uniqueness is determined by its name/url as well as its config)
        config.insert("request_id".to_owned(), request_id.to_string());

        // Start filepicker at the current session folder if set
        if let Some(folder) = self.new_session_info.new_session_folder() {
            config.insert(
                "caller_cwd".to_owned(),
                folder.to_string_lossy().to_string(),
            );
        }

        // we also insert this into the args so that the plugin will have an easier access to it
        args.insert("request_id".to_owned(), request_id.to_string());

        pipe_message_to_plugin(
            MessageToPlugin::new("filepicker")
                .with_plugin_url("filepicker")
                .with_plugin_config(config)
                .new_plugin_instance_should_have_pane_title("Select folder for the new session...")
                .with_args(args),
        );
    }

    /// Check if a request ID is valid (exists in our request list)
    pub fn is_valid_request_id(&self, request_id: &str) -> bool {
        self.request_ids.contains(&request_id.to_string())
    }

    /// Remove a request ID from our tracking list
    pub fn remove_request_id(&mut self, request_id: &str) {
        self.request_ids.retain(|id| id != request_id);
    }

    /// Set new session folder
    pub fn set_new_session_folder(&mut self, folder: Option<std::path::PathBuf>) {
        self.new_session_info.set_folder(folder);
    }

    /// Handle quick session creation from main screen
    fn handle_quick_session_creation(&mut self) {
        use zellij_tile::prelude::{switch_session_with_cwd, switch_session_with_layout};

        // Get the selected item data or search term
        let (session_name, session_folder) = if let Some(selected_item) = self.selected_item() {
            match selected_item {
                SessionItem::ExistingSession { name, .. } => {
                    // Switch to existing session
                    switch_session_with_cwd(Some(&name), None);
                    hide_self();
                    return;
                }
                SessionItem::ResurrectableSession { name, .. } => {
                    switch_session_with_cwd(Some(&name), None);
                    hide_self();
                    return;
                }
                SessionItem::Directory {
                    session_name, path, ..
                } => {
                    let incremented_name = self
                        .session_manager
                        .generate_incremented_name(&session_name, &self.config.session_separator);
                    (incremented_name, Some(std::path::PathBuf::from(path)))
                }
            }
        } else {
            self.set_error("Please select a directory".to_string());
            return;
        };

        // Validate session name
        if session_name.len() >= 108 {
            self.set_error("Session name must be shorter than 108 bytes".to_string());
            return;
        }
        if session_name.contains('/') {
            self.set_error("Session name cannot contain '/'".to_string());
            return;
        }

        // Check if session name is different from current session
        if Some(&session_name) == self.current_session_name.as_ref() {
            self.set_error("Cannot create session with same name as current session".to_string());
            return;
        }

        // Create session with default layout if configured
        match &self.config.default_layout {
            Some(layout_name) => {
                // Find the layout by name from current session's available layouts
                if let Some(current_session) = self
                    .session_manager
                    .sessions()
                    .iter()
                    .find(|s| s.is_current_session)
                {
                    let layout_info = current_session
                        .available_layouts
                        .iter()
                        .find(|layout| layout.name() == layout_name)
                        .cloned();

                    match layout_info {
                        Some(layout) => {
                            switch_session_with_layout(Some(&session_name), layout, session_folder);
                        }
                        None => {
                            // Defined layout not found, create without layout
                            switch_session_with_cwd(Some(&session_name), session_folder);
                        }
                    }
                } else {
                    // No current session info, cannot retrieve layouts, create without layout
                    switch_session_with_cwd(Some(&session_name), session_folder);
                }
            }
            None => {
                // No default layout configured, create without layout
                switch_session_with_cwd(Some(&session_name), session_folder);
            }
        }

        hide_self();
    }
}
