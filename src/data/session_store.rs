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
}
