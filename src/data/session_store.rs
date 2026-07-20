use crate::domain::{ResurrectableSession, Session};
use std::collections::HashSet;
use std::time::Duration;
use zellij_tile::prelude::SessionInfo;

#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: Vec<Session>,
    resurrectable: Vec<ResurrectableSession>,
    pending_deletion: Option<String>,
}

impl SessionStore {
    pub fn update(&mut self, session_infos: Vec<SessionInfo>) {
        self.sessions = session_infos
            .into_iter()
            .map(|info| Session {
                name: info.name,
                is_current: info.is_current_session,
            })
            .collect();
        // Pin current session first, preserve Zellij's order for the rest.
        if let Some(pos) = self.sessions.iter().position(|s| s.is_current) {
            if pos > 0 {
                let current = self.sessions.remove(pos);
                self.sessions.insert(0, current);
            }
        }
    }

    pub fn update_resurrectable(&mut self, sessions: Vec<(String, Duration)>) {
        self.resurrectable = sessions
            .into_iter()
            .map(|(name, duration)| ResurrectableSession { name, duration })
            .collect();
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub fn resurrectable(&self) -> &[ResurrectableSession] {
        &self.resurrectable
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.sessions.iter().find(|s| s.is_current)
    }

    pub fn start_deletion(&mut self, name: String) {
        self.pending_deletion = Some(name);
    }

    pub fn pending_deletion(&self) -> Option<&str> {
        self.pending_deletion.as_deref()
    }

    pub fn confirm_deletion(&mut self) -> Option<String> {
        self.pending_deletion.take()
    }

    pub fn cancel_deletion(&mut self) {
        self.pending_deletion = None;
    }

    pub fn is_resurrectable(&self, name: &str) -> bool {
        self.resurrectable.iter().any(|s| s.name == name)
    }

    pub fn generate_incremented_name(&self, base_name: &str, separator: &str) -> String {
        let existing: HashSet<&str> = self
            .sessions
            .iter()
            .map(|s| s.name.as_str())
            .chain(self.resurrectable.iter().map(|s| s.name.as_str()))
            .collect();

        if !existing.contains(base_name) {
            return base_name.to_string();
        }

        for counter in 2..=1000 {
            let candidate = format!("{}{}{}", base_name, separator, counter);
            if !existing.contains(candidate.as_str()) {
                return candidate;
            }
        }

        format!("{}{}{}", base_name, separator, uuid::Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session_info(name: &str, is_current: bool) -> SessionInfo {
        SessionInfo {
            name: name.to_string(),
            is_current_session: is_current,
            ..Default::default()
        }
    }

    #[test]
    fn generate_incremented_returns_base_when_available() {
        let store = SessionStore::default();
        assert_eq!(store.generate_incremented_name("project", "."), "project");
    }

    #[test]
    fn generate_incremented_adds_suffix_when_taken() {
        let mut store = SessionStore::default();
        store.sessions = vec![Session {
            name: "project".to_string(),
            is_current: false,
        }];
        assert_eq!(store.generate_incremented_name("project", "."), "project.2");
    }

    #[test]
    fn generate_incremented_finds_next_available() {
        let mut store = SessionStore::default();
        store.sessions = vec![
            Session {
                name: "project".to_string(),
                is_current: false,
            },
            Session {
                name: "project.2".to_string(),
                is_current: false,
            },
        ];
        assert_eq!(store.generate_incremented_name("project", "."), "project.3");
    }

    #[test]
    fn generate_incremented_considers_resurrectable() {
        let mut store = SessionStore::default();
        store.update_resurrectable(vec![("project".to_string(), Duration::from_secs(0))]);
        assert_eq!(store.generate_incremented_name("project", "."), "project.2");
    }

    #[test]
    fn update_from_session_info() {
        let mut store = SessionStore::default();
        let infos = vec![
            make_session_info("session1", false),
            make_session_info("session2", true),
        ];
        store.update(infos);
        assert_eq!(store.sessions().len(), 2);
        // Current session pinned first, rest in original order
        assert_eq!(store.sessions()[0].name, "session2");
        assert!(store.sessions()[0].is_current);
        assert_eq!(store.sessions()[1].name, "session1");
    }

    #[test]
    fn update_preserves_order_for_non_current() {
        let mut store = SessionStore::default();
        let infos = vec![
            make_session_info("zebra", false),
            make_session_info("alpha", true),
            make_session_info("middle", false),
        ];
        store.update(infos);
        // Current pinned first, rest preserve Zellij's order
        assert_eq!(store.sessions()[0].name, "alpha");
        assert_eq!(store.sessions()[1].name, "zebra");
        assert_eq!(store.sessions()[2].name, "middle");
    }

    #[test]
    fn current_session_returns_correct() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("inactive", false),
            make_session_info("current", true),
        ]);
        let current = store.current_session();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "current");
    }

    #[test]
    fn current_session_returns_none_when_no_current() {
        let mut store = SessionStore::default();
        store.update(vec![make_session_info("only", false)]);
        assert!(store.current_session().is_none());
    }

    #[test]
    fn deletion_workflow_start() {
        let mut store = SessionStore::default();
        assert!(store.pending_deletion().is_none());
        store.start_deletion("test".to_string());
        assert_eq!(store.pending_deletion(), Some("test"));
    }

    #[test]
    fn deletion_workflow_confirm() {
        let mut store = SessionStore::default();
        store.start_deletion("test".to_string());
        let confirmed = store.confirm_deletion();
        assert_eq!(confirmed, Some("test".to_string()));
        assert!(store.pending_deletion().is_none());
    }

    #[test]
    fn deletion_workflow_cancel() {
        let mut store = SessionStore::default();
        store.start_deletion("test".to_string());
        store.cancel_deletion();
        assert!(store.pending_deletion().is_none());
    }

    #[test]
    fn is_resurrectable_true() {
        let mut store = SessionStore::default();
        store.update_resurrectable(vec![("dead-session".to_string(), Duration::from_secs(3600))]);
        assert!(store.is_resurrectable("dead-session"));
    }

    #[test]
    fn is_resurrectable_false() {
        let mut store = SessionStore::default();
        store.update_resurrectable(vec![("other".to_string(), Duration::from_secs(0))]);
        assert!(!store.is_resurrectable("unknown"));
    }

    #[test]
    fn resurrectable_sessions() {
        let mut store = SessionStore::default();
        store.update_resurrectable(vec![
            ("dead1".to_string(), Duration::from_secs(100)),
            ("dead2".to_string(), Duration::from_secs(200)),
        ]);
        assert_eq!(store.resurrectable().len(), 2);
        assert_eq!(store.resurrectable()[0].name, "dead1");
    }
}
