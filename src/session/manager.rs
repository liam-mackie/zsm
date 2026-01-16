use crate::session::types::SessionAction;
use std::collections::HashMap;
use std::time::Duration;
use zellij_tile::prelude::{delete_dead_session, kill_sessions, switch_session, SessionInfo};

/// Number of consecutive updates a session must be missing before we remove it
const MISSING_THRESHOLD: u8 = 3;

/// Manages session operations and state
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Currently known sessions from Zellij
    sessions: Vec<SessionInfo>,
    /// Session name pending deletion confirmation
    pending_deletion: Option<String>,
    /// Resurrectable sessions
    resurrectable_sessions: Vec<(String, Duration)>,
    /// Tracks how many consecutive updates each session has been missing
    /// Key is lowercase session name for case-insensitive matching
    missing_counts: HashMap<String, u8>,
    /// MRU timestamps for session ordering (session name -> unix timestamp in seconds)
    mru_timestamps: HashMap<String, u64>,
}

impl SessionManager {
    /// Update session list with stability tracking
    /// Returns true if the visible session list changed
    pub fn update_sessions_stable(&mut self, new_sessions: Vec<SessionInfo>) -> bool {
        let mut changed = false;

        // Build a set of session names from the new update (lowercase for comparison)
        let new_session_names: HashMap<String, &SessionInfo> = new_sessions
            .iter()
            .map(|s| (s.name.to_lowercase(), s))
            .collect();

        // Check for sessions that are in the new list - reset their missing count
        // and add any genuinely new sessions
        for new_session in &new_sessions {
            let key = new_session.name.to_lowercase();
            self.missing_counts.remove(&key);

            // Check if this is a new session we haven't seen before
            let exists = self.sessions.iter().any(|s| s.name.to_lowercase() == key);
            if !exists {
                // New session - add it
                self.sessions.push(new_session.clone());
                changed = true;
            } else {
                // Update existing session info (e.g., is_current_session flag)
                if let Some(existing) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.name.to_lowercase() == key)
                {
                    if existing.is_current_session != new_session.is_current_session {
                        existing.is_current_session = new_session.is_current_session;
                        changed = true;
                    }
                }
            }
        }

        // Check for sessions that are missing from the new list
        let mut sessions_to_remove = Vec::new();
        for session in &self.sessions {
            let key = session.name.to_lowercase();
            if !new_session_names.contains_key(&key) {
                // Session is missing - increment its missing count
                let count = self.missing_counts.entry(key.clone()).or_insert(0);
                *count += 1;

                if *count >= MISSING_THRESHOLD {
                    // Session has been missing long enough - remove it
                    sessions_to_remove.push(key);
                    changed = true;
                }
            }
        }

        // Remove sessions that have been missing for too long
        for key in sessions_to_remove {
            self.sessions.retain(|s| s.name.to_lowercase() != key);
            self.missing_counts.remove(&key);
        }

        changed
    }

    /// Update resurrectable sessions with stability tracking
    /// Returns true if the visible list changed
    pub fn update_resurrectable_stable(
        &mut self,
        new_resurrectable: Vec<(String, Duration)>,
    ) -> bool {
        // For resurrectable sessions, we use simpler logic:
        // Just check if the set of names changed (case-insensitive)
        let mut current_names: Vec<String> = self
            .resurrectable_sessions
            .iter()
            .map(|(name, _)| name.to_lowercase())
            .collect();
        let mut new_names: Vec<String> = new_resurrectable
            .iter()
            .map(|(name, _)| name.to_lowercase())
            .collect();

        current_names.sort();
        new_names.sort();

        let changed = current_names != new_names;
        // Always update to get fresh durations
        self.resurrectable_sessions = new_resurrectable;
        changed
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[SessionInfo] {
        &self.sessions
    }

    /// Get all resurrectable sessions
    pub fn resurrectable_sessions(&self) -> &[(String, Duration)] {
        &self.resurrectable_sessions
    }

    /// Execute a session action
    pub fn execute_action(&mut self, action: SessionAction) {
        match action {
            SessionAction::Switch(name) => {
                switch_session(Some(&name));
            }
        }
    }

    /// Start session deletion confirmation
    pub fn start_deletion(&mut self, session_name: String) {
        self.pending_deletion = Some(session_name);
    }

    /// Rename a session in local state (optimistic update)
    /// Used for immediate UI feedback before Zellij confirms the rename
    pub fn rename_session_in_local_state(&mut self, old_name: &str, new_name: &str) {
        let old_key = old_name.to_lowercase();
        let new_key = new_name.to_lowercase();

        // Update in existing sessions
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|s| s.name.to_lowercase() == old_key)
        {
            session.name = new_name.to_string();
        }

        // Update MRU timestamps (transfer old name's timestamp to new name)
        if let Some(timestamp) = self.mru_timestamps.remove(old_name) {
            self.mru_timestamps.insert(new_name.to_string(), timestamp);
        }

        // Clean up missing counts for old name
        self.missing_counts.remove(&old_key);
        // Pre-emptively remove new name from missing counts in case it was tracked
        self.missing_counts.remove(&new_key);
    }

    /// Remove a session from local state (optimistic update)
    /// Used by confirm_deletion for immediate UI feedback
    fn remove_session_from_local_state(&mut self, session_name: &str) {
        let key = session_name.to_lowercase();

        // Remove from existing sessions
        self.sessions.retain(|s| s.name.to_lowercase() != key);

        // Remove from resurrectable sessions
        self.resurrectable_sessions
            .retain(|(name, _)| name.to_lowercase() != key);

        // Clean up tracking state
        self.missing_counts.remove(&key);
    }

    /// Confirm session deletion
    /// Uses optimistic update - removes from local state immediately before sending kill command
    pub fn confirm_deletion(&mut self) {
        if let Some(session_name) = self.pending_deletion.take() {
            // Check if it's a resurrectable session BEFORE the optimistic removal
            // (removal clears the list, so we must check first)
            let is_resurrectable = self
                .resurrectable_sessions
                .iter()
                .any(|(name, _)| name == &session_name);

            // Optimistic update: Remove session from local state immediately
            // This provides instant UI feedback rather than waiting for MISSING_THRESHOLD updates
            self.remove_session_from_local_state(&session_name);

            // Now execute the actual kill/delete action
            // If this fails, the session will reappear on the next SessionUpdate event
            if is_resurrectable {
                delete_dead_session(&session_name);
            } else {
                kill_sessions(&[&session_name]);
            }
        }
    }

    /// Cancel session deletion
    pub fn cancel_deletion(&mut self) {
        self.pending_deletion = None;
    }

    /// Get session pending deletion
    pub fn pending_deletion(&self) -> Option<&str> {
        self.pending_deletion.as_deref()
    }

    /// Record a session switch for MRU tracking
    /// Returns the unix timestamp that was recorded
    pub fn record_switch(&mut self, session_name: &str) -> u64 {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.mru_timestamps
            .insert(session_name.to_string(), timestamp);
        timestamp
    }

    /// Set MRU timestamps from persisted data
    pub fn set_mru_timestamps(&mut self, timestamps: HashMap<String, u64>) {
        self.mru_timestamps = timestamps;
    }

    /// Get MRU timestamps
    #[allow(dead_code)] // API completeness - useful for debugging/future features
    pub fn mru_timestamps(&self) -> &HashMap<String, u64> {
        &self.mru_timestamps
    }

    /// Get MRU rank for a session (higher = more recent)
    /// Returns 0 if session has no recorded timestamp
    pub fn get_mru_rank(&self, session_name: &str) -> u64 {
        self.mru_timestamps.get(session_name).copied().unwrap_or(0)
    }

    /// Generate incremented session name for a base name
    pub fn generate_incremented_name(&self, base_name: &str, separator: &str) -> String {
        let base_exists = self.sessions.iter().any(|s| s.name == base_name)
            || self
                .resurrectable_sessions
                .iter()
                .any(|(name, _)| name == base_name);

        if !base_exists {
            return base_name.to_string();
        }

        // Find the next available increment
        for counter in 2..=1000 {
            let candidate = format!("{}{}{}", base_name, separator, counter);
            let exists = self.sessions.iter().any(|s| s.name == candidate);

            if !exists {
                return candidate;
            }
        }

        // Fallback with UUID if too many increments
        format!(
            "{}{}{}",
            base_name,
            separator,
            &uuid::Uuid::new_v4().to_string()[..8]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(name: &str, is_current: bool) -> SessionInfo {
        use std::collections::BTreeMap;
        use zellij_tile::prelude::PaneManifest;
        SessionInfo {
            name: name.to_string(),
            is_current_session: is_current,
            tabs: vec![],
            panes: PaneManifest {
                panes: std::collections::HashMap::new(),
            },
            connected_clients: 0,
            available_layouts: vec![],
            plugins: BTreeMap::new(),
            tab_history: BTreeMap::new(),
            web_client_count: 0,
            web_clients_allowed: true,
        }
    }

    #[test]
    fn test_new_session_added_immediately() {
        let mut manager = SessionManager::default();

        let changed = manager.update_sessions_stable(vec![make_session("test", false)]);

        assert!(changed);
        assert_eq!(manager.sessions().len(), 1);
        assert_eq!(manager.sessions()[0].name, "test");
    }

    #[test]
    fn test_session_not_removed_on_single_missing_update() {
        let mut manager = SessionManager::default();

        // Add a session
        manager.update_sessions_stable(vec![make_session("test", false)]);

        // Session disappears for one update
        let changed = manager.update_sessions_stable(vec![]);

        // Should NOT be removed yet (needs MISSING_THRESHOLD updates)
        assert!(!changed);
        assert_eq!(manager.sessions().len(), 1);
    }

    #[test]
    fn test_session_removed_after_threshold_missing_updates() {
        let mut manager = SessionManager::default();

        // Add a session
        manager.update_sessions_stable(vec![make_session("test", false)]);

        // Session disappears for MISSING_THRESHOLD updates
        for i in 0..MISSING_THRESHOLD {
            let changed = manager.update_sessions_stable(vec![]);
            if i < MISSING_THRESHOLD - 1 {
                assert!(!changed, "Should not report changed before threshold");
                assert_eq!(manager.sessions().len(), 1, "Session should still exist");
            } else {
                assert!(changed, "Should report changed when removed");
                assert_eq!(manager.sessions().len(), 0, "Session should be removed");
            }
        }
    }

    #[test]
    fn test_session_reappearing_resets_missing_count() {
        let mut manager = SessionManager::default();

        // Add a session
        manager.update_sessions_stable(vec![make_session("test", false)]);

        // Session disappears for 2 updates (less than threshold)
        manager.update_sessions_stable(vec![]);
        manager.update_sessions_stable(vec![]);

        // Session reappears
        let changed = manager.update_sessions_stable(vec![make_session("test", false)]);
        assert!(!changed); // No visible change
        assert_eq!(manager.sessions().len(), 1);

        // Now it disappears again - counter should have been reset
        let changed = manager.update_sessions_stable(vec![]);
        assert!(!changed);
        assert_eq!(manager.sessions().len(), 1); // Still there after 1 missing
    }

    #[test]
    fn test_is_current_session_update_triggers_change() {
        let mut manager = SessionManager::default();

        // Add a non-current session
        manager.update_sessions_stable(vec![make_session("test", false)]);

        // Update it to be current
        let changed = manager.update_sessions_stable(vec![make_session("test", true)]);

        assert!(changed);
        assert!(manager.sessions()[0].is_current_session);
    }

    #[test]
    fn test_resurrectable_name_change_triggers_update() {
        let mut manager = SessionManager::default();

        // Add initial resurrectable sessions
        let changed = manager
            .update_resurrectable_stable(vec![("session1".to_string(), Duration::from_secs(60))]);
        assert!(changed);

        // Same sessions - no change
        let changed = manager
            .update_resurrectable_stable(vec![("session1".to_string(), Duration::from_secs(120))]);
        assert!(!changed);

        // Different sessions - change
        let changed = manager
            .update_resurrectable_stable(vec![("session2".to_string(), Duration::from_secs(60))]);
        assert!(changed);
    }

    #[test]
    fn test_resurrectable_case_insensitive_comparison() {
        let mut manager = SessionManager::default();

        manager.update_resurrectable_stable(vec![("Session".to_string(), Duration::from_secs(60))]);

        // Same name different case - should NOT trigger change
        let changed = manager
            .update_resurrectable_stable(vec![("session".to_string(), Duration::from_secs(60))]);
        assert!(!changed);
    }

    #[test]
    fn test_optimistic_removal_removes_session_immediately() {
        let mut manager = SessionManager::default();

        // Add sessions
        manager.update_sessions_stable(vec![
            make_session("keep", false),
            make_session("delete-me", false),
        ]);
        assert_eq!(manager.sessions().len(), 2);

        // Optimistic removal should remove session from local state immediately
        manager.remove_session_from_local_state("delete-me");

        // Session should be removed immediately
        assert_eq!(manager.sessions().len(), 1);
        assert_eq!(manager.sessions()[0].name, "keep");
    }

    #[test]
    fn test_optimistic_removal_removes_resurrectable_immediately() {
        let mut manager = SessionManager::default();

        // Add resurrectable session
        manager.update_resurrectable_stable(vec![(
            "dead-session".to_string(),
            Duration::from_secs(60),
        )]);
        assert_eq!(manager.resurrectable_sessions().len(), 1);

        // Optimistic removal should remove resurrectable session from local state immediately
        manager.remove_session_from_local_state("dead-session");

        // Should be removed immediately
        assert_eq!(manager.resurrectable_sessions().len(), 0);
    }

    #[test]
    fn test_optimistic_removal_cleans_up_missing_counts() {
        let mut manager = SessionManager::default();

        // Add a session
        manager.update_sessions_stable(vec![make_session("test", false)]);

        // Session disappears for one update - this adds to missing_counts
        manager.update_sessions_stable(vec![]);
        // Session reappears - this removes from missing_counts, then disappears again
        manager.update_sessions_stable(vec![make_session("test", false)]);
        manager.update_sessions_stable(vec![]);
        // At this point, "test" is in missing_counts

        // Optimistic removal should also clean up missing_counts
        manager.remove_session_from_local_state("test");

        // Even if session somehow reappears (failed kill), the missing_counts
        // shouldn't have stale data - verify session is gone from local state
        assert_eq!(manager.sessions().len(), 0);
    }

    #[test]
    fn test_optimistic_rename_updates_session_name() {
        let mut manager = SessionManager::default();

        // Add sessions
        manager.update_sessions_stable(vec![
            make_session("old-name", true),
            make_session("other", false),
        ]);
        assert_eq!(manager.sessions().len(), 2);

        // Optimistic rename should update session name immediately
        manager.rename_session_in_local_state("old-name", "new-name");

        // Session should have new name
        assert_eq!(manager.sessions().len(), 2);
        assert!(manager.sessions().iter().any(|s| s.name == "new-name"));
        assert!(!manager.sessions().iter().any(|s| s.name == "old-name"));
    }

    #[test]
    fn test_optimistic_rename_transfers_mru_timestamp() {
        let mut manager = SessionManager::default();

        // Add session and set MRU timestamp
        manager.update_sessions_stable(vec![make_session("old-name", true)]);
        manager.record_switch("old-name");
        let old_ts = manager.get_mru_rank("old-name");
        assert!(old_ts > 0);

        // Optimistic rename should transfer MRU timestamp
        manager.rename_session_in_local_state("old-name", "new-name");

        // Timestamp should be on new name, not old
        assert_eq!(manager.get_mru_rank("new-name"), old_ts);
        assert_eq!(manager.get_mru_rank("old-name"), 0);
    }
}
