use std::collections::BTreeMap;

/// Session list sort order
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SortOrder {
    /// Most recently used sessions first (default)
    #[default]
    Mru,
    /// Alphabetical order by session name
    Alphabetical,
}

impl SortOrder {
    /// Parse sort order from config string (case-insensitive)
    fn from_config_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "alphabetical" => SortOrder::Alphabetical,
            _ => SortOrder::Mru, // "mru" or any other value defaults to MRU
        }
    }
}

/// Plugin configuration loaded from Zellij layout
#[derive(Debug, Clone)]
pub struct Config {
    /// Default layout for quick session creation with Ctrl+Enter
    pub default_layout: Option<String>,
    /// Separator used in session names (default: ".")
    pub session_separator: String,
    /// Base paths to strip from directory names when generating session names
    pub base_paths: Vec<String>,
    /// Whether to show all sessions, not just those matching zoxide directories
    pub show_all_sessions: bool,
    /// Sort order for session list (default: MRU)
    pub sort_order: SortOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_layout: None,
            session_separator: ".".to_string(),
            base_paths: Vec::new(),
            show_all_sessions: false,
            sort_order: SortOrder::default(),
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
            sort_order: config
                .get("sort_order")
                .map(|v| SortOrder::from_config_str(v))
                .unwrap_or_default(),
        }
    }
}
