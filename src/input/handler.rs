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
            BareKey::Char('m') if key.has_modifiers(&[KeyModifier::Ctrl]) => Action::ToggleMode,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn key(bare: BareKey) -> KeyWithModifier {
        KeyWithModifier::new(bare)
    }

    fn char_key(c: char) -> KeyWithModifier {
        KeyWithModifier::new(BareKey::Char(c))
    }

    fn ctrl_key(c: char) -> KeyWithModifier {
        KeyWithModifier::new_with_modifiers(
            BareKey::Char(c),
            BTreeSet::from([KeyModifier::Ctrl]),
        )
    }

    fn ctrl_enter() -> KeyWithModifier {
        KeyWithModifier::new_with_modifiers(BareKey::Enter, BTreeSet::from([KeyModifier::Ctrl]))
    }

    // Main screen navigation tests
    #[test]
    fn arrow_up_navigates_up() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Up), Screen::Main, &context);
        assert_eq!(action, Action::Navigate(Direction::Up));
    }

    #[test]
    fn arrow_down_navigates_down() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Down), Screen::Main, &context);
        assert_eq!(action, Action::Navigate(Direction::Down));
    }

    #[test]
    fn enter_selects() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Enter), Screen::Main, &context);
        assert_eq!(action, Action::Select);
    }

    #[test]
    fn ctrl_enter_quick_creates() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_enter(), Screen::Main, &context);
        assert_eq!(action, Action::QuickCreate);
    }

    #[test]
    fn delete_starts_deletion() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Delete), Screen::Main, &context);
        assert_eq!(action, Action::Delete);
    }

    // Search input tests
    #[test]
    fn char_input_adds_to_search() {
        let context = InputContext::default();
        let action = InputHandler::handle(char_key('a'), Screen::Main, &context);
        assert_eq!(action, Action::Search(SearchAction::AddChar('a')));
    }

    #[test]
    fn backspace_removes_from_search() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Backspace), Screen::Main, &context);
        assert_eq!(action, Action::Search(SearchAction::Backspace));
    }

    #[test]
    fn escape_clears_search_when_searching() {
        let context = InputContext {
            is_searching: true,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Esc), Screen::Main, &context);
        assert_eq!(action, Action::Search(SearchAction::Clear));
    }

    #[test]
    fn escape_hides_when_not_searching() {
        let context = InputContext {
            is_searching: false,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Esc), Screen::Main, &context);
        assert_eq!(action, Action::Hide);
    }

    #[test]
    fn ctrl_c_hides() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_key('c'), Screen::Main, &context);
        assert_eq!(action, Action::Hide);
    }

    #[test]
    fn ctrl_r_refreshes() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_key('r'), Screen::Main, &context);
        assert_eq!(action, Action::Refresh);
    }

    #[test]
    fn ctrl_m_toggles_mode() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_key('m'), Screen::Main, &context);
        assert_eq!(action, Action::ToggleMode);
    }

    // Deletion confirmation tests
    #[test]
    fn y_confirms_deletion() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(char_key('y'), Screen::Main, &context);
        assert_eq!(action, Action::ConfirmDelete);
    }

    #[test]
    fn uppercase_y_confirms_deletion() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(char_key('Y'), Screen::Main, &context);
        assert_eq!(action, Action::ConfirmDelete);
    }

    #[test]
    fn n_cancels_deletion() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(char_key('n'), Screen::Main, &context);
        assert_eq!(action, Action::CancelDelete);
    }

    #[test]
    fn escape_cancels_deletion() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Esc), Screen::Main, &context);
        assert_eq!(action, Action::CancelDelete);
    }

    #[test]
    fn deletion_mode_ignores_navigation() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Down), Screen::Main, &context);
        assert_eq!(action, Action::None);
    }

    #[test]
    fn deletion_mode_ignores_other_chars() {
        let context = InputContext {
            has_pending_deletion: true,
            ..Default::default()
        };
        let action = InputHandler::handle(char_key('x'), Screen::Main, &context);
        assert_eq!(action, Action::None);
    }

    // NewSession screen tests
    #[test]
    fn new_session_enter_selects() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Enter), Screen::NewSession, &context);
        assert_eq!(action, Action::Select);
    }

    #[test]
    fn new_session_ctrl_f_opens_filepicker() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_key('f'), Screen::NewSession, &context);
        assert_eq!(action, Action::OpenFilepicker);
    }

    #[test]
    fn new_session_ctrl_c_clears_folder() {
        let context = InputContext::default();
        let action = InputHandler::handle(ctrl_key('c'), Screen::NewSession, &context);
        assert_eq!(action, Action::ClearFolder);
    }

    #[test]
    fn new_session_escape_with_empty_name_goes_to_main() {
        let context = InputContext {
            session_name_empty: true,
            entering_name: true,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Esc), Screen::NewSession, &context);
        assert_eq!(action, Action::GoToScreen(Screen::Main));
    }

    #[test]
    fn new_session_escape_during_layout_clears_search() {
        let context = InputContext {
            entering_name: false,
            ..Default::default()
        };
        let action = InputHandler::handle(key(BareKey::Esc), Screen::NewSession, &context);
        assert_eq!(action, Action::Search(SearchAction::Clear));
    }

    #[test]
    fn new_session_navigation_works() {
        let context = InputContext::default();
        let action = InputHandler::handle(key(BareKey::Up), Screen::NewSession, &context);
        assert_eq!(action, Action::Navigate(Direction::Up));
    }

    #[test]
    fn new_session_char_adds_to_search() {
        let context = InputContext::default();
        let action = InputHandler::handle(char_key('t'), Screen::NewSession, &context);
        assert_eq!(action, Action::Search(SearchAction::AddChar('t')));
    }
}
