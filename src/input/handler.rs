use crate::app::{Action, Screen};
use crate::app::actions::{Direction, SearchAction};
use zellij_tile::prelude::{BareKey, KeyModifier, KeyWithModifier};

pub struct InputHandler;

impl InputHandler {
    pub fn handle(key: KeyWithModifier, screen: Screen, context: &InputContext) -> Action {
        if context.has_pending_deletion {
            return Self::handle_deletion_confirmation(key);
        }

        match screen {
            Screen::Main => Self::handle_main_screen(key, context.is_searching),
            Screen::NewSession => Self::handle_new_session_screen(key, context),
        }
    }

    fn handle_deletion_confirmation(key: KeyWithModifier) -> Action {
        match key.bare_key {
            BareKey::Char('y') | BareKey::Char('Y') if key.has_no_modifiers() => {
                Action::ConfirmDelete
            }
            BareKey::Char('n') | BareKey::Char('N') | BareKey::Esc if key.has_no_modifiers() => {
                Action::CancelDelete
            }
            _ => Action::None,
        }
    }

    fn handle_main_screen(key: KeyWithModifier, is_searching: bool) -> Action {
        match key.bare_key {
            BareKey::Up if key.has_no_modifiers() => Action::Navigate(Direction::Up),
            BareKey::Down if key.has_no_modifiers() => Action::Navigate(Direction::Down),
            BareKey::Enter if key.has_no_modifiers() => Action::Select,
            BareKey::Enter if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::QuickCreate,
            BareKey::Delete if key.has_no_modifiers() => Action::Delete,
            BareKey::Char(c) if key.has_no_modifiers() && c != '\n' => {
                Action::Search(SearchAction::AddChar(c))
            }
            BareKey::Backspace if key.has_no_modifiers() => Action::Search(SearchAction::Backspace),
            BareKey::Esc if key.has_no_modifiers() => {
                if is_searching {
                    Action::Search(SearchAction::Clear)
                } else {
                    Action::Hide
                }
            }
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::Hide,
            BareKey::Char('r') if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::Refresh,
            _ => Action::None,
        }
    }

    fn handle_new_session_screen(key: KeyWithModifier, context: &InputContext) -> Action {
        match key.bare_key {
            BareKey::Enter if key.has_no_modifiers() => Action::Select,
            BareKey::Enter if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::QuickCreate,
            BareKey::Esc if key.has_no_modifiers() => {
                if context.session_name_empty && context.entering_name {
                    Action::GoToScreen(Screen::Main)
                } else if context.entering_name {
                    Action::GoToScreen(Screen::Main)
                } else {
                    Action::Search(SearchAction::Clear)
                }
            }
            BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::OpenFilepicker,
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::ClearFolder,
            BareKey::Up if key.has_no_modifiers() => Action::Navigate(Direction::Up),
            BareKey::Down if key.has_no_modifiers() => Action::Navigate(Direction::Down),
            BareKey::Char(c) if key.has_no_modifiers() && c != '\n' => {
                Action::Search(SearchAction::AddChar(c))
            }
            BareKey::Backspace if key.has_no_modifiers() => Action::Search(SearchAction::Backspace),
            _ => Action::None,
        }
    }
}

pub struct InputContext {
    pub is_searching: bool,
    pub has_pending_deletion: bool,
    pub session_name_empty: bool,
    pub entering_name: bool,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            is_searching: false,
            has_pending_deletion: false,
            session_name_empty: true,
            entering_name: true,
        }
    }
}
