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

    #[test]
    fn is_incremented_matches_suffix() {
        assert!(is_incremented_version("project.2", "project", "."));
        assert!(is_incremented_version("project.. 100", "project", "."));
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
}
