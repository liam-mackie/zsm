use crate::domain::Directory;
use crate::naming::SessionNameGenerator;

#[derive(Debug, Default)]
pub struct DirectoryStore {
    directories: Vec<Directory>,
}

impl DirectoryStore {
    pub fn update(&mut self, mut directories: Vec<Directory>, generator: &SessionNameGenerator) {
        generator.generate_names(&mut directories);
        directories.sort();
        self.directories = directories;
    }

    pub fn directories(&self) -> &[Directory] {
        &self.directories
    }

    pub fn find_by_session_name(&self, session_name: &str) -> Option<&Directory> {
        self.directories
            .iter()
            .find(|d| d.session_name == session_name)
    }

    pub fn find_by_path(&self, path: &str) -> Option<&Directory> {
        self.directories.iter().find(|d| d.path == path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_generator() -> SessionNameGenerator {
        SessionNameGenerator::new(".".to_string(), vec![])
    }

    fn make_dir(path: &str, ranking: f64) -> Directory {
        Directory {
            path: path.to_string(),
            ranking,
            session_name: String::new(),
        }
    }

    #[test]
    fn update_sorts_by_ranking_descending() {
        let mut store = DirectoryStore::default();
        let dirs = vec![
            make_dir("/low", 10.0),
            make_dir("/high", 100.0),
            make_dir("/medium", 50.0),
        ];
        store.update(dirs, &make_generator());
        assert_eq!(store.directories()[0].path, "/high");
        assert_eq!(store.directories()[1].path, "/medium");
        assert_eq!(store.directories()[2].path, "/low");
    }

    #[test]
    fn update_generates_session_names() {
        let mut store = DirectoryStore::default();
        let dirs = vec![make_dir("/home/user/project", 100.0)];
        store.update(dirs, &make_generator());
        assert_eq!(store.directories()[0].session_name, "project");
    }

    #[test]
    fn find_by_session_name_returns_correct_directory() {
        let mut store = DirectoryStore::default();
        let dirs = vec![
            make_dir("/home/user/alpha", 100.0),
            make_dir("/home/user/beta", 50.0),
        ];
        store.update(dirs, &make_generator());
        let found = store.find_by_session_name("beta");
        assert!(found.is_some());
        assert_eq!(found.unwrap().path, "/home/user/beta");
    }

    #[test]
    fn find_by_session_name_returns_none_for_unknown() {
        let mut store = DirectoryStore::default();
        store.update(vec![make_dir("/home/test", 100.0)], &make_generator());
        assert!(store.find_by_session_name("unknown").is_none());
    }

    #[test]
    fn find_by_path_returns_correct_directory() {
        let mut store = DirectoryStore::default();
        let dirs = vec![
            make_dir("/home/user/alpha", 100.0),
            make_dir("/home/user/beta", 50.0),
        ];
        store.update(dirs, &make_generator());
        let found = store.find_by_path("/home/user/beta");
        assert!(found.is_some());
        assert_eq!(found.unwrap().session_name, "beta");
    }

    #[test]
    fn find_by_path_returns_none_for_unknown() {
        let mut store = DirectoryStore::default();
        store.update(vec![make_dir("/home/test", 100.0)], &make_generator());
        assert!(store.find_by_path("/unknown/path").is_none());
    }

    #[test]
    fn empty_store() {
        let store = DirectoryStore::default();
        assert!(store.directories().is_empty());
        assert!(store.find_by_session_name("any").is_none());
        assert!(store.find_by_path("/any").is_none());
    }
}
