use crate::domain::{Directory, DisplayItem};

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
        let dirs = directories.directories();

        for session in sessions.sessions() {
            let directory = find_matching_directory(&session.name, dirs, separator);

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

// Truncating names to fit zellij's socket budget can collapse two distinct
// directories onto the same session name (a project root and a same-named folder
// nested inside it). Prefer the shallowest path: a bare basename belongs to the
// top-level directory, not a nested namesake. Tie-breaks make it independent of
// zoxide's ordering.
fn find_matching_directory<'a>(
    session_name: &str,
    directories: &'a [Directory],
    separator: &str,
) -> Option<&'a str> {
    let exact = directories.iter().filter(|d| d.session_name == session_name);
    if let Some(dir) = shallowest(exact) {
        return Some(dir.path.as_str());
    }

    let incremented = directories
        .iter()
        .filter(|d| is_incremented_version(session_name, &d.session_name, separator));
    shallowest(incremented).map(|d| d.path.as_str())
}

fn shallowest<'a>(dirs: impl Iterator<Item = &'a Directory>) -> Option<&'a Directory> {
    dirs.min_by(|a, b| {
        path_depth(&a.path)
            .cmp(&path_depth(&b.path))
            .then_with(|| a.path.len().cmp(&b.path.len()))
            .then_with(|| a.path.cmp(&b.path))
    })
}

fn path_depth(path: &str) -> usize {
    path.split('/').filter(|s| !s.is_empty()).count()
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
    fn colliding_truncated_names_match_shallowest_directory() {
        // The 22-char basename overruns the 21-char budget, so root and nested
        // both truncate to the same name; the session must still resolve to root.
        let root = "/Users/liammackie/g/octopus-argocd-gateway";
        let nested = "/Users/liammackie/g/octopus-argocd-gateway/charts/octopus-argocd-gateway";
        let gen = SessionNameGenerator::new(".".to_string(), vec![], 21);

        for (root_rank, nested_rank) in [(100.0, 50.0), (50.0, 100.0)] {
            let (mut sessions, mut directories) = setup_stores();
            directories.update(
                vec![
                    Directory {
                        path: root.to_string(),
                        ranking: root_rank,
                        session_name: String::new(),
                    },
                    Directory {
                        path: nested.to_string(),
                        ranking: nested_rank,
                        session_name: String::new(),
                    },
                ],
                &gen,
            );
            let names: Vec<&str> = directories
                .directories()
                .iter()
                .map(|d| d.session_name.as_str())
                .collect();
            assert_eq!(names[0], names[1], "expected a name collision to test against");

            let session_name = names[0].to_string();
            sessions.update(vec![zellij_tile::prelude::SessionInfo {
                name: session_name,
                is_current_session: false,
                ..Default::default()
            }]);

            let items = DisplayList::build(&sessions, &directories, false, ".");
            let matched = items.iter().find_map(|i| match i {
                DisplayItem::ExistingSession { directory, .. } => Some(directory.as_deref()),
                _ => None,
            });
            assert_eq!(matched, Some(Some(root)));
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
