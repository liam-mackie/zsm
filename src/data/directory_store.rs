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
