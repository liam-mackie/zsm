use std::collections::BTreeMap;

/// Plugin configuration loaded from Zellij layout
#[derive(Debug, Clone)]
pub struct Config {
    /// Default layout for quick session creation with Ctrl+Enter
    pub default_layout: Option<String>,
    /// Separator used in session names (default: ".")
    pub session_separator: String,
    /// Whether you'd like resurrectable sessions to be shown in the session list
    pub show_resurrectable_sessions: bool,
    /// Base paths to strip from directory names when generating session names
    pub base_paths: Vec<String>,
    /// Whether to show all sessions, not just those matching zoxide directories
    pub show_all_sessions: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_layout: None,
            session_separator: ".".to_string(),
            show_resurrectable_sessions: false,
            base_paths: Vec::new(),
            show_all_sessions: false,
        }
    }
}

impl Config {
    /// Create configuration from Zellij plugin configuration
    pub fn from_zellij_config(config: &BTreeMap<String, String>) -> Self {
        Self {
            default_layout: config.get("default_layout").cloned(),
            session_separator: config
                .get("session_separator")
                .cloned()
                .unwrap_or_else(|| ".".to_string()),
            show_resurrectable_sessions: config
                .get("show_resurrectable_sessions")
                .map(|v| v == "true")
                .unwrap_or(false),
            base_paths: config
                .get("base_paths")
                .map(|paths| {
                    paths
                        .split('|')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            show_all_sessions: config
                .get("show_all_sessions")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}
