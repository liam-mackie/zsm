use zellij_tile::prelude::{SessionInfo, kill_sessions, switch_session};
use crate::session::types::SessionAction;

/// Manages session operations and state
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Currently known sessions from Zellij
    sessions: Vec<SessionInfo>,
    /// Session name pending deletion confirmation
    pending_deletion: Option<String>,
}

impl SessionManager {
    /// Update the session list with new session information
    pub fn update_sessions(&mut self, sessions: Vec<SessionInfo>) {
        self.sessions = sessions;
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[SessionInfo] {
        &self.sessions
    }

    /// Execute a session action
    pub fn execute_action(&mut self, action: SessionAction) {
        match action {
            SessionAction::Switch(name) => {
                switch_session(Some(&name));
            }
            SessionAction::Kill(name) => {
                kill_sessions(&[&name]);
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
        let base_exists = self.sessions.iter().any(|s| s.name == base_name);
        
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
        format!("{}{}{}", base_name, separator, uuid::Uuid::new_v4().to_string()[..8].to_string())
    }
}