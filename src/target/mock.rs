use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zellij_tile::prelude::LayoutInfo;

use super::Target;

#[derive(Clone)]
pub struct MockTarget {
    inner: Rc<MockTargetInner>,
}

struct MockTargetInner {
    created: RefCell<Vec<(String, PathBuf, Option<LayoutInfo>)>>,
    switched_to: RefCell<Vec<String>>,
    deleted: RefCell<Vec<String>>,
    hidden: RefCell<usize>,
}

impl MockTarget {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(MockTargetInner {
                created: RefCell::new(vec![]),
                switched_to: RefCell::new(vec![]),
                deleted: RefCell::new(vec![]),
                hidden: RefCell::new(0),
            }),
        }
    }

    pub fn created_sessions(&self) -> Vec<(String, PathBuf, Option<LayoutInfo>)> {
        self.inner.created.borrow().clone()
    }

    pub fn switched_to_sessions(&self) -> Vec<String> {
        self.inner.switched_to.borrow().clone()
    }

    pub fn deleted_sessions(&self) -> Vec<String> {
        self.inner.deleted.borrow().clone()
    }

    pub fn hide_count(&self) -> usize {
        *self.inner.hidden.borrow()
    }

    pub fn reset(&self) {
        self.inner.created.borrow_mut().clear();
        self.inner.switched_to.borrow_mut().clear();
        self.inner.deleted.borrow_mut().clear();
        *self.inner.hidden.borrow_mut() = 0;
    }
}

impl Default for MockTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for MockTarget {
    fn create(&self, name: &str, directory: &Path, layout: Option<LayoutInfo>) {
        self.inner
            .created
            .borrow_mut()
            .push((name.to_string(), directory.to_path_buf(), layout));
    }

    fn switch_to(&self, name: &str) {
        self.inner.switched_to.borrow_mut().push(name.to_string());
    }

    fn delete(&self, name: &str) {
        self.inner.deleted.borrow_mut().push(name.to_string());
    }

    fn hide(&self) {
        *self.inner.hidden.borrow_mut() += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_created_sessions() {
        let mock = MockTarget::new();
        mock.create("test", Path::new("/home/test"), None);

        let created = mock.created_sessions();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].0, "test");
        assert_eq!(created[0].1, PathBuf::from("/home/test"));
    }

    #[test]
    fn tracks_switched_sessions() {
        let mock = MockTarget::new();
        mock.switch_to("session1");
        mock.switch_to("session2");

        let switched = mock.switched_to_sessions();
        assert_eq!(switched, vec!["session1", "session2"]);
    }

    #[test]
    fn tracks_deleted_sessions() {
        let mock = MockTarget::new();
        mock.delete("old-session");

        let deleted = mock.deleted_sessions();
        assert_eq!(deleted, vec!["old-session"]);
    }

    #[test]
    fn tracks_hide_calls() {
        let mock = MockTarget::new();
        mock.hide();
        mock.hide();

        assert_eq!(mock.hide_count(), 2);
    }

    #[test]
    fn reset_clears_all_tracking() {
        let mock = MockTarget::new();
        mock.create("test", Path::new("/test"), None);
        mock.switch_to("session");
        mock.delete("old");
        mock.hide();

        mock.reset();

        assert!(mock.created_sessions().is_empty());
        assert!(mock.switched_to_sessions().is_empty());
        assert!(mock.deleted_sessions().is_empty());
        assert_eq!(mock.hide_count(), 0);
    }
}
