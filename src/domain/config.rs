use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetMode {
    #[default]
    Session,
    Tab,
}

impl TargetMode {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tab" => Self::Tab,
            _ => Self::Session,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: TargetMode,
    pub default_layout: Option<String>,
    pub session_separator: String,
    pub show_resurrectable_sessions: bool,
    pub base_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: TargetMode::default(),
            default_layout: None,
            session_separator: ".".to_string(),
            show_resurrectable_sessions: false,
            base_paths: Vec::new(),
        }
    }
}

impl Config {
    pub fn from_zellij_config(config: &BTreeMap<String, String>) -> Self {
        Self {
            mode: config
                .get("mode")
                .map(|m| TargetMode::from_str(m))
                .unwrap_or_default(),
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
        }
    }
}
