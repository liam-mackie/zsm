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
    // Search matches against this exact string, so render it verbatim or
    // highlight indices won't line up.
    pub fn display_text(&self) -> String {
        match self {
            Self::ExistingSession {
                name,
                directory,
                is_current,
            } => {
                let prefix = if *is_current { "● " } else { "○ " };
                match directory {
                    Some(dir) => format!("{}{} ({})", prefix, name, dir),
                    None => format!("{}{}", prefix, name),
                }
            }
            Self::ResurrectableSession { name, duration } => {
                format!("↺ {} ({})", name, humantime::format_duration(*duration))
            }
            Self::Directory { path, .. } => path.clone(),
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_existing_session() {
        let item = DisplayItem::ExistingSession {
            name: "my-session".to_string(),
            directory: Some("/path".to_string()),
            is_current: false,
        };
        assert_eq!(item.display_name(), "my-session");
    }

    #[test]
    fn display_name_resurrectable_session() {
        let item = DisplayItem::ResurrectableSession {
            name: "dead-session".to_string(),
            duration: Duration::from_secs(3600),
        };
        assert_eq!(item.display_name(), "dead-session");
    }

    #[test]
    fn display_name_directory() {
        let item = DisplayItem::Directory {
            path: "/home/user/project".to_string(),
            session_name: "project".to_string(),
        };
        assert_eq!(item.display_name(), "project");
    }

    #[test]
    fn is_session_existing() {
        let item = DisplayItem::ExistingSession {
            name: "test".to_string(),
            directory: None,
            is_current: false,
        };
        assert!(item.is_session());
    }

    #[test]
    fn is_session_resurrectable() {
        let item = DisplayItem::ResurrectableSession {
            name: "test".to_string(),
            duration: Duration::from_secs(0),
        };
        assert!(item.is_session());
    }

    #[test]
    fn is_session_directory() {
        let item = DisplayItem::Directory {
            path: "/path".to_string(),
            session_name: "name".to_string(),
        };
        assert!(!item.is_session());
    }

    #[test]
    fn is_current_true() {
        let item = DisplayItem::ExistingSession {
            name: "current".to_string(),
            directory: None,
            is_current: true,
        };
        assert!(item.is_current());
    }

    #[test]
    fn is_current_false() {
        let item = DisplayItem::ExistingSession {
            name: "not-current".to_string(),
            directory: None,
            is_current: false,
        };
        assert!(!item.is_current());
    }

    #[test]
    fn is_current_resurrectable() {
        let item = DisplayItem::ResurrectableSession {
            name: "test".to_string(),
            duration: Duration::from_secs(0),
        };
        assert!(!item.is_current());
    }

    #[test]
    fn is_current_directory() {
        let item = DisplayItem::Directory {
            path: "/path".to_string(),
            session_name: "name".to_string(),
        };
        assert!(!item.is_current());
    }
}
