use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayItem {
    ExistingSession {
        name: String,
        directory: Option<String>,
        is_current: bool,
    },
    ResurrectableSession {
        name: String,
        duration: Duration,
    },
    Directory {
        path: String,
        session_name: String,
    },
}

impl DisplayItem {
    pub fn display_name(&self) -> &str {
        match self {
            Self::ExistingSession { name, .. } => name,
            Self::ResurrectableSession { name, .. } => name,
            Self::Directory { session_name, .. } => session_name,
        }
    }

    pub fn is_session(&self) -> bool {
        matches!(
            self,
            Self::ExistingSession { .. } | Self::ResurrectableSession { .. }
        )
    }

    pub fn is_current(&self) -> bool {
        matches!(self, Self::ExistingSession { is_current: true, .. })
    }
}
