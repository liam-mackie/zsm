use crate::domain::DisplayItem;
use std::collections::HashMap;

use super::{DirectoryStore, SessionStore};

pub struct DisplayList;

impl DisplayList {
    pub fn build(
        sessions: &SessionStore,
        directories: &DirectoryStore,
        show_resurrectable: bool,
        separator: &str,
    ) -> Vec<DisplayItem> {
        let mut items = Vec::new();

        let session_name_to_dir: HashMap<&str, &str> = directories
            .directories()
            .iter()
            .map(|d| (d.session_name.as_str(), d.path.as_str()))
            .collect();

        for session in sessions.sessions() {
            let directory = find_matching_directory(
                &session.name,
                &session_name_to_dir,
                separator,
            );

            items.push(DisplayItem::ExistingSession {
                name: session.name.clone(),
                directory: directory.map(|s| s.to_string()),
                is_current: session.is_current,
            });
        }

        if show_resurrectable {
            for session in sessions.resurrectable() {
                items.push(DisplayItem::ResurrectableSession {
                    name: session.name.clone(),
                    duration: session.duration,
                });
            }
        }

        for dir in directories.directories() {
            items.push(DisplayItem::Directory {
                path: dir.path.clone(),
                session_name: dir.session_name.clone(),
            });
        }

        items
    }
}

fn find_matching_directory<'a>(
    session_name: &str,
    name_to_dir: &HashMap<&str, &'a str>,
    separator: &str,
) -> Option<&'a str> {
    if let Some(&path) = name_to_dir.get(session_name) {
        return Some(path);
    }

    for (&dir_name, &path) in name_to_dir {
        if is_incremented_version(session_name, dir_name, separator) {
            return Some(path);
        }
    }

    None
}

fn is_incremented_version(session_name: &str, base_name: &str, separator: &str) -> bool {
    if session_name.len() <= base_name.len() || !session_name.starts_with(base_name) {
        return false;
    }

    let remainder = &session_name[base_name.len()..];
    if !remainder.starts_with(separator) {
        return false;
    }

    let number_part = &remainder[separator.len()..];
    !number_part.is_empty() && number_part.parse::<u32>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Directory;
    use crate::naming::SessionNameGenerator;
    use std::time::Duration;

    fn make_generator() -> SessionNameGenerator {
        SessionNameGenerator::new(".".to_string(), vec![], crate::naming::DEFAULT_MAX_NAME_LENGTH)
    }

    fn setup_stores() -> (SessionStore, DirectoryStore) {
        (SessionStore::default(), DirectoryStore::default())
    }

    #[test]
    fn is_incremented_matches_suffix() {
        assert!(is_incremented_version("project.2", "project", "."));
        assert!(is_incremented_version("project.100", "project", "."));
    }

    #[test]
    fn is_incremented_rejects_non_numeric() {
        assert!(!is_incremented_version("project.foo", "project", "."));
        assert!(!is_incremented_version("project.2a", "project", "."));
    }

    #[test]
    fn is_incremented_rejects_different_base() {
        assert!(!is_incremented_version("other.2", "project", "."));
    }

    #[test]
    fn is_incremented_rejects_shorter() {
        assert!(!is_incremented_version("pro", "project", "."));
    }

    #[test]
    fn is_incremented_rejects_equal_length() {
        assert!(!is_incremented_version("project", "project", "."));
    }

    #[test]
    fn is_incremented_rejects_missing_separator() {
        assert!(!is_incremented_version("project2", "project", "."));
    }

    #[test]
    fn build_empty_stores() {
        let (sessions, directories) = setup_stores();
        let items = DisplayList::build(&sessions, &directories, false, ".");
        assert!(items.is_empty());
    }

    #[test]
    fn build_sessions_only() {
        let (mut sessions, directories) = setup_stores();
        sessions.update(vec![
            zellij_tile::prelude::SessionInfo {
                name: "session1".to_string(),
                is_current_session: true,
                ..Default::default()
            },
            zellij_tile::prelude::SessionInfo {
                name: "session2".to_string(),
                is_current_session: false,
                ..Default::default()
            },
        ]);
        let items = DisplayList::build(&sessions, &directories, false, ".");
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], DisplayItem::ExistingSession { name, is_current, .. } if name == "session1" && *is_current));
    }

    #[test]
    fn build_directories_only() {
        let (sessions, mut directories) = setup_stores();
        let dirs = vec![Directory {
            path: "/home/user/project".to_string(),
            ranking: 100.0,
            session_name: String::new(),
        }];
        directories.update(dirs, &make_generator());
        let items = DisplayList::build(&sessions, &directories, false, ".");
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], DisplayItem::Directory { path, .. } if path == "/home/user/project"));
    }

    #[test]
    fn build_includes_resurrectable_when_enabled() {
        let (mut sessions, directories) = setup_stores();
        sessions.update_resurrectable(vec![("dead".to_string(), Duration::from_secs(3600))]);
        let items = DisplayList::build(&sessions, &directories, true, ".");
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], DisplayItem::ResurrectableSession { name, .. } if name == "dead"));
    }

    #[test]
    fn build_excludes_resurrectable_when_disabled() {
        let (mut sessions, directories) = setup_stores();
        sessions.update_resurrectable(vec![("dead".to_string(), Duration::from_secs(3600))]);
        let items = DisplayList::build(&sessions, &directories, false, ".");
        assert!(items.is_empty());
    }

    #[test]
    fn build_matches_session_to_directory() {
        let (mut sessions, mut directories) = setup_stores();
        sessions.update(vec![zellij_tile::prelude::SessionInfo {
            name: "project".to_string(),
            is_current_session: false,
            ..Default::default()
        }]);
        directories.update(
            vec![Directory {
                path: "/home/user/project".to_string(),
                ranking: 100.0,
                session_name: String::new(),
            }],
            &make_generator(),
        );
        let items = DisplayList::build(&sessions, &directories, false, ".");
        if let DisplayItem::ExistingSession { directory, .. } = &items[0] {
            assert_eq!(directory.as_deref(), Some("/home/user/project"));
        } else {
            panic!("Expected ExistingSession");
        }
    }

    #[test]
    fn build_matches_incremented_session() {
        let (mut sessions, mut directories) = setup_stores();
        sessions.update(vec![zellij_tile::prelude::SessionInfo {
            name: "project.2".to_string(),
            is_current_session: false,
            ..Default::default()
        }]);
        directories.update(
            vec![Directory {
                path: "/home/user/project".to_string(),
                ranking: 100.0,
                session_name: String::new(),
            }],
            &make_generator(),
        );
        let items = DisplayList::build(&sessions, &directories, false, ".");
        if let DisplayItem::ExistingSession { directory, .. } = &items[0] {
            assert_eq!(directory.as_deref(), Some("/home/user/project"));
        } else {
            panic!("Expected ExistingSession");
        }
    }
}
