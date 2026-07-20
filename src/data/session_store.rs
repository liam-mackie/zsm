use crate::domain::{ResurrectableSession, Session};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use zellij_tile::prelude::SessionInfo;

/// Zellij learns about other sessions by polling their metadata cache files,
/// so a live session can transiently vanish from one SessionUpdate and
/// reappear in the next. Keep a session listed until it has been absent from
/// this many consecutive updates (roughly one update per second).
const MISSED_UPDATES_BEFORE_DROP: u8 = 3;

#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: Vec<Session>,
    // Consecutive updates each currently-listed session has been absent from.
    // Sessions present in the latest update have no entry.
    missed: HashMap<String, u8>,
    resurrectable: Vec<ResurrectableSession>,
    pending_deletion: Option<String>,
}

impl SessionStore {
    pub fn update(&mut self, session_infos: Vec<SessionInfo>) {
        let fresh: Vec<Session> = session_infos
            .into_iter()
            .map(|info| Session {
                name: info.name,
                is_current: info.is_current_session,
            })
            .collect();
        let fresh_names: HashSet<String> = fresh.iter().map(|s| s.name.clone()).collect();

        let mut merged = fresh;
        let mut missed_next: HashMap<String, u8> = HashMap::new();

        for (old_index, old) in std::mem::take(&mut self.sessions).into_iter().enumerate() {
            if fresh_names.contains(&old.name) {
                continue;
            }
            let missed = self.missed.get(&old.name).copied().unwrap_or(0) + 1;
            if missed >= MISSED_UPDATES_BEFORE_DROP {
                continue;
            }
            missed_next.insert(old.name.clone(), missed);
            merged.insert(old_index.min(merged.len()), old);
        }

        self.sessions = merged;
        self.missed = missed_next;

        // Pin current session first, preserve Zellij's order for the rest.
        if let Some(pos) = self.sessions.iter().position(|s| s.is_current) {
            if pos > 0 {
                let current = self.sessions.remove(pos);
                self.sessions.insert(0, current);
            }
        }
    }

    pub fn update_resurrectable(&mut self, sessions: Vec<(String, Duration)>) {
        // Dead is authoritative: stop extending grace to a session zellij now
        // reports as resurrectable.
        let dead: HashSet<&str> = sessions.iter().map(|(name, _)| name.as_str()).collect();
        let missed = &self.missed;
        self.sessions
            .retain(|s| !missed.contains_key(&s.name) || !dead.contains(s.name.as_str()));
        self.missed.retain(|name, _| !dead.contains(name.as_str()));

        // The reverse also holds: a session listed as live shouldn't show a
        // dead duplicate.
        self.resurrectable = sessions
            .into_iter()
            .filter(|(name, _)| !self.sessions.iter().any(|s| &s.name == name))
            .map(|(name, duration)| ResurrectableSession { name, duration })
            .collect();
    }

    /// Immediate removal for sessions the user deleted through zsm — waiting
    /// for the next SessionUpdate (or worse, ghost grace) would show a
    /// just-deleted session as alive.
    pub fn remove_session(&mut self, name: &str) {
        self.sessions.retain(|s| s.name != name);
        self.missed.remove(name);
        self.resurrectable.retain(|s| s.name != name);
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
        let store = SessionStore {
            sessions: vec![Session {
                name: "project".to_string(),
                is_current: false,
            }],
            ..Default::default()
        };
        assert_eq!(store.generate_incremented_name("project", "."), "project.2");
    }

    #[test]
    fn generate_incremented_finds_next_available() {
        let store = SessionStore {
            sessions: vec![
                Session {
                    name: "project".to_string(),
                    is_current: false,
                },
                Session {
                    name: "project.2".to_string(),
                    is_current: false,
                },
            ],
            ..Default::default()
        };
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

    // Flap-tolerance tests: zellij's view of other sessions is eventually
    // consistent, so single-update dropouts must not empty the listing.

    fn names(store: &SessionStore) -> Vec<&str> {
        store.sessions().iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn transiently_missing_session_stays_listed() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("other", false),
        ]);
        store.update(vec![make_session_info("current", true)]);
        assert_eq!(names(&store), vec!["current", "other"]);
    }

    #[test]
    fn session_missing_repeatedly_is_dropped() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("other", false),
        ]);
        for _ in 0..3 {
            store.update(vec![make_session_info("current", true)]);
        }
        assert_eq!(names(&store), vec!["current"]);
    }

    #[test]
    fn reappearing_session_resets_miss_count() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("other", false),
        ]);
        store.update(vec![make_session_info("current", true)]);
        store.update(vec![
            make_session_info("current", true),
            make_session_info("other", false),
        ]);
        // Two more misses shouldn't drop it — the counter restarted.
        store.update(vec![make_session_info("current", true)]);
        store.update(vec![make_session_info("current", true)]);
        assert_eq!(names(&store), vec!["current", "other"]);
    }

    #[test]
    fn ghost_keeps_list_position() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("middle", false),
            make_session_info("last", false),
        ]);
        store.update(vec![
            make_session_info("current", true),
            make_session_info("last", false),
        ]);
        assert_eq!(names(&store), vec!["current", "middle", "last"]);
    }

    #[test]
    fn ghost_removed_when_it_turns_up_dead() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("other", false),
        ]);
        // "other" goes missing (ghosted), then the same update cycle reports
        // it resurrectable.
        store.update(vec![make_session_info("current", true)]);
        store.update_resurrectable(vec![("other".to_string(), Duration::from_secs(1))]);
        assert_eq!(names(&store), vec!["current"]);
        assert!(store.is_resurrectable("other"));
    }

    #[test]
    fn live_session_suppresses_dead_duplicate() {
        let mut store = SessionStore::default();
        store.update(vec![make_session_info("project", true)]);
        store.update_resurrectable(vec![("project".to_string(), Duration::from_secs(1))]);
        assert_eq!(names(&store), vec!["project"]);
        assert!(store.resurrectable().is_empty());
    }

    #[test]
    fn remove_session_is_immediate() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("current", true),
            make_session_info("doomed", false),
        ]);
        store.update_resurrectable(vec![("dead".to_string(), Duration::from_secs(1))]);
        store.remove_session("doomed");
        store.remove_session("dead");
        assert_eq!(names(&store), vec!["current"]);
        assert!(store.resurrectable().is_empty());
        // A ghost of the removed session must not reappear on the next flap.
        store.update(vec![make_session_info("current", true)]);
        assert_eq!(names(&store), vec!["current"]);
    }

    #[test]
    fn current_session_pinned_first_with_ghosts() {
        let mut store = SessionStore::default();
        store.update(vec![
            make_session_info("other", false),
            make_session_info("current", true),
        ]);
        store.update(vec![make_session_info("current", true)]);
        assert_eq!(names(&store)[0], "current");
    }
}
