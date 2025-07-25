use std::collections::BTreeMap;

/// Plugin configuration loaded from Zellij layout
#[derive(Debug, Clone)]
pub struct Config {
    /// Default layout for quick session creation with Ctrl+Enter
    pub default_layout: Option<String>,
    /// Separator used in session names (default: ".")
    pub session_separator: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_layout: None,
            session_separator: ".".to_string(),
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
        }
    }
}