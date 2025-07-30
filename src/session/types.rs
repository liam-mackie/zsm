
/// Represents different types of items that can be displayed in the session list
#[derive(Debug, Clone)]
pub enum SessionItem {
    /// An existing Zellij session
    ExistingSession {
        name: String,
        directory: String,
        is_current: bool,
    },
    /// A resurrectable session that can be restored
    ResurrectableSession {
        name: String,
        duration: std::time::Duration,
    },
    /// A zoxide directory that can be used to create a new session
    Directory {
        path: String,
        session_name: String,
    },
}

impl SessionItem {
    /// Check if this is an existing session
    pub fn is_session(&self) -> bool {
        matches!(self, SessionItem::ExistingSession { .. })
    }
    pub fn is_resurrectable_session(&self) -> bool {
        matches!(self, SessionItem::ResurrectableSession { .. })
    }
}

/// Actions that can be performed on sessions
#[derive(Debug, Clone)]
pub enum SessionAction {
    /// Switch to an existing session
    Switch(String),
    /// Kill an existing session
    Kill(String),
}