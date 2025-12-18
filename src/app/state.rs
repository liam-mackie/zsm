use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::{LayoutInfo, Palette, SessionInfo};

use crate::data::{DirectoryStore, DisplayList, SessionStore};
use crate::domain::{Config, Directory, DisplayItem};
use crate::input::{InputContext, InputHandler};
use crate::integrations::{zellij_delete_dead_session, zellij_pipe_message_to_plugin};
use crate::naming::SessionNameGenerator;
use crate::search::{SearchEngine, SelectionState};
use crate::target::{create_target, Target};

use super::actions::{Direction, SearchAction};
use super::{Action, Screen};

pub struct AppState {
    config: Config,
    target: Box<dyn Target>,
    sessions: SessionStore,
    directories: DirectoryStore,
    search: SearchEngine,
    selection: SelectionState,
    screen: Screen,
    colors: Option<Palette>,
    error: Option<String>,
    current_session: Option<String>,
    layouts: Vec<LayoutInfo>,
    new_session_name: String,
    new_session_folder: Option<PathBuf>,
    layout_search: String,
    layout_selection: SelectionState,
    entering_name: bool,
    request_ids: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Config::default(),
            target: create_target(Default::default()),
            sessions: SessionStore::default(),
            directories: DirectoryStore::default(),
            search: SearchEngine::default(),
            selection: SelectionState::default(),
            screen: Screen::default(),
            colors: None,
            error: None,
            current_session: None,
            layouts: Vec::new(),
            new_session_name: String::new(),
            new_session_folder: None,
            layout_search: String::new(),
            layout_selection: SelectionState::default(),
            entering_name: true,
            request_ids: Vec::new(),
        }
    }
}

impl AppState {
    pub fn initialize(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_zellij_config(&configuration);
        self.target = create_target(self.config.mode);
    }

    pub fn set_colors(&mut self, colors: Palette) {
        self.colors = Some(colors);
    }

    pub fn colors(&self) -> Option<Palette> {
        self.colors
    }

    pub fn update_sessions(&mut self, infos: Vec<SessionInfo>) {
        for info in &infos {
            if info.is_current_session {
                self.current_session = Some(info.name.clone());
                self.layouts = info.available_layouts.clone();
                break;
            }
        }
        self.sessions.update(infos);
        self.refresh_selection();
    }

    pub fn update_directories(&mut self, directories: Vec<Directory>) {
        let generator = SessionNameGenerator::new(
            self.config.session_separator.clone(),
            self.config.base_paths.clone(),
        );
        self.directories.update(directories, &generator);
        self.refresh_selection();
    }

    pub fn handle_key(&mut self, key: zellij_tile::prelude::KeyWithModifier) -> bool {
        if self.error.is_some() {
            self.error = None;
            return true;
        }

        let context = InputContext {
            is_searching: self.search.is_active(),
            has_pending_deletion: self.sessions.pending_deletion().is_some(),
            session_name_empty: self.new_session_name.is_empty(),
            entering_name: self.entering_name,
        };

        let action = InputHandler::handle(key, self.screen, &context);
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
                self.target.hide();
                false
            }
            Action::None => false,
        }
    }

    fn navigate(&mut self, direction: Direction) -> bool {
        match self.screen {
            Screen::Main => {
                if self.search.is_active() {
                    match direction {
                        Direction::Up => self.search.move_up(),
                        Direction::Down => self.search.move_down(),
                    }
                } else {
                    match direction {
                        Direction::Up => self.selection.move_up(),
                        Direction::Down => self.selection.move_down(),
                    }
                }
            }
            Screen::NewSession if !self.entering_name => match direction {
                Direction::Up => self.layout_selection.move_up(),
                Direction::Down => self.layout_selection.move_down(),
            },
            _ => {}
        }
        true
    }

    fn handle_select(&mut self) -> bool {
        match self.screen {
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
                self.target.hide();
            }
            Some(DisplayItem::Directory {
                path, session_name, ..
            }) => {
                let name = self
                    .sessions
                    .generate_incremented_name(&session_name, &self.config.session_separator);
                self.new_session_name = name;
                self.new_session_folder = Some(PathBuf::from(path));
                self.entering_name = false;
                self.layout_selection.update_count(self.layouts.len());
                self.screen = Screen::NewSession;
            }
            None => {}
        }
        true
    }

    fn create_session(&mut self) -> bool {
        if self.entering_name {
            self.entering_name = false;
            self.layout_selection.update_count(self.layouts.len());
            return true;
        }

        let layout = self
            .layout_selection
            .index()
            .and_then(|i| self.filtered_layouts().get(i).cloned());

        if let Some(folder) = &self.new_session_folder {
            self.target
                .create(&self.new_session_name, folder, layout);
            self.target.hide();
        }

        self.screen = Screen::Main;
        true
    }

    fn handle_quick_create(&mut self) -> bool {
        match self.screen {
            Screen::Main => {
                if let Some(DisplayItem::Directory {
                    path, session_name, ..
                }) = self.selected_item()
                {
                    let name = self
                        .sessions
                        .generate_incremented_name(&session_name, &self.config.session_separator);

                    let layout = self.config.default_layout.as_ref().and_then(|name| {
                        self.layouts.iter().find(|l| l.name() == name).cloned()
                    });

                    self.target.create(&name, &PathBuf::from(path), layout);
                    self.target.hide();
                }
            }
            Screen::NewSession => {
                if let Some(folder) = &self.new_session_folder {
                    let layout = self.config.default_layout.as_ref().and_then(|name| {
                        self.layouts.iter().find(|l| l.name() == name).cloned()
                    });

                    self.target
                        .create(&self.new_session_name, folder, layout);
                    self.target.hide();
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
            if is_resurrectable {
                zellij_delete_dead_session(&name);
            } else {
                self.target.delete(&name);
            }
        }
        true
    }

    fn cancel_delete(&mut self) -> bool {
        self.sessions.cancel_deletion();
        true
    }

    fn handle_search(&mut self, action: SearchAction) -> bool {
        let items = self.display_items();
        match self.screen {
            Screen::Main => match action {
                SearchAction::AddChar(c) => self.search.add_char(c, &items),
                SearchAction::Backspace => self.search.backspace(&items),
                SearchAction::Clear => self.search.clear(),
            },
            Screen::NewSession if self.entering_name => match action {
                SearchAction::AddChar(c) => self.new_session_name.push(c),
                SearchAction::Backspace => {
                    self.new_session_name.pop();
                }
                SearchAction::Clear => self.new_session_name.clear(),
            },
            Screen::NewSession => match action {
                SearchAction::AddChar(c) => {
                    self.layout_search.push(c);
                    self.update_layout_selection();
                }
                SearchAction::Backspace => {
                    self.layout_search.pop();
                    self.update_layout_selection();
                }
                SearchAction::Clear => {
                    self.layout_search.clear();
                    self.entering_name = true;
                }
            },
        }
        true
    }

    fn go_to_screen(&mut self, screen: Screen) -> bool {
        self.screen = screen;
        if screen == Screen::Main {
            self.new_session_name.clear();
            self.new_session_folder = None;
            self.layout_search.clear();
            self.entering_name = true;
        }
        true
    }

    fn open_filepicker(&mut self) -> bool {
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        self.request_ids.push(request_id.clone());

        let mut config = BTreeMap::new();
        config.insert("request_id".to_string(), request_id.clone());

        if let Some(folder) = &self.new_session_folder {
            config.insert("caller_cwd".to_string(), folder.to_string_lossy().to_string());
        }

        let mut args = BTreeMap::new();
        args.insert("request_id".to_string(), request_id);

        zellij_pipe_message_to_plugin("filepicker", "filepicker", config, args, "Select folder...");

        true
    }

    fn clear_folder(&mut self) -> bool {
        self.new_session_folder = None;
        true
    }

    fn request_refresh(&mut self) -> bool {
        true
    }

    fn refresh_selection(&mut self) {
        let count = self.display_items().len();
        self.selection.update_count(count);
    }

    fn update_layout_selection(&mut self) {
        let count = self.filtered_layouts().len();
        self.layout_selection.update_count(count);
    }

    pub fn display_items(&self) -> Vec<DisplayItem> {
        if self.search.is_active() {
            self.search.results().iter().map(|r| r.item.clone()).collect()
        } else {
            DisplayList::build(
                &self.sessions,
                &self.directories,
                self.config.show_resurrectable_sessions,
                &self.config.session_separator,
            )
        }
    }

    pub fn selected_item(&self) -> Option<DisplayItem> {
        let items = self.display_items();
        let index = if self.search.is_active() {
            self.search.selected_index()
        } else {
            self.selection.index()
        };
        index.and_then(|i| items.get(i).cloned())
    }

    pub fn filtered_layouts(&self) -> Vec<LayoutInfo> {
        if self.layout_search.is_empty() {
            self.layouts.clone()
        } else {
            self.layouts
                .iter()
                .filter(|l| {
                    l.name()
                        .to_lowercase()
                        .contains(&self.layout_search.to_lowercase())
                })
                .cloned()
                .collect()
        }
    }

    // Accessors for rendering
    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn sessions(&self) -> &SessionStore {
        &self.sessions
    }

    pub fn search(&self) -> &SearchEngine {
        &self.search
    }

    pub fn selected_index(&self) -> Option<usize> {
        if self.search.is_active() {
            self.search.selected_index()
        } else {
            self.selection.index()
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn new_session_name(&self) -> &str {
        &self.new_session_name
    }

    pub fn new_session_folder(&self) -> Option<&PathBuf> {
        self.new_session_folder.as_ref()
    }

    pub fn layout_search(&self) -> &str {
        &self.layout_search
    }

    pub fn layout_selection_index(&self) -> Option<usize> {
        self.layout_selection.index()
    }

    pub fn entering_name(&self) -> bool {
        self.entering_name
    }

    pub fn is_valid_request_id(&self, id: &str) -> bool {
        self.request_ids.contains(&id.to_string())
    }

    pub fn remove_request_id(&mut self, id: &str) {
        self.request_ids.retain(|r| r != id);
    }

    pub fn set_new_session_folder(&mut self, folder: Option<PathBuf>) {
        self.new_session_folder = folder;
    }

    pub fn should_refresh(&self) -> bool {
        true
    }
}
