use crate::session::types::SessionAction;
use std::time::Duration;
use zellij_tile::prelude::{delete_dead_session, kill_sessions, switch_session, SessionInfo};

/// Manages session operations and state
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Currently known sessions from Zellij
    sessions: Vec<SessionInfo>,
    /// Session name pending deletion confirmation
    pending_deletion: Option<String>,
    /// Resurrectable sessions
    resurrectable_sessions: Vec<(String, Duration)>,
}

impl SessionManager {
    /// Update the session list with new session information
    pub fn update_sessions(&mut self, sessions: Vec<SessionInfo>) {
        self.sessions = sessions;
    }

    /// Update the resurrectable sessions
    pub fn update_resurrectable_sessions(
        &mut self,
        resurrectable_sessions: Vec<(String, Duration)>,
    ) {
        self.resurrectable_sessions = resurrectable_sessions;
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
            SessionAction::Kill(name) => {
                if self
                    .resurrectable_sessions
                    .iter()
                    .any(|(session_name, _)| session_name == &name)
                {
                    // If the session is resurrectable, we should delete it
                    delete_dead_session(&name);
                } else {
                    // Otherwise, we need to kill the session
                    kill_sessions(&[&name]);
                }
            }
        }
    }

    /// Start session deletion confirmation
    pub fn start_deletion(&mut self, session_name: String) {
        self.pending_deletion = Some(session_name);
    }

    /// Confirm session deletion
    pub fn confirm_deletion(&mut self) {
        if let Some(session_name) = self.pending_deletion.take() {
            self.execute_action(SessionAction::Kill(session_name));
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
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        )
    }
}
