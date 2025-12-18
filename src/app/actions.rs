#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Navigate(Direction),
    Select,
    QuickCreate,
    Delete,
    ConfirmDelete,
    CancelDelete,
    Search(SearchAction),
    GoToScreen(super::Screen),
    OpenFilepicker,
    ClearFolder,
    Refresh,
    Hide,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchAction {
    AddChar(char),
    Backspace,
    Clear,
}
