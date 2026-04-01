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
    pub dev_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: TargetMode::default(),
            default_layout: None,
            session_separator: ".".to_string(),
            show_resurrectable_sessions: false,
            base_paths: Vec::new(),
            dev_mode: false,
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
            dev_mode: config
                .get("dev_mode")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.mode, TargetMode::Session);
        assert_eq!(config.session_separator, ".");
        assert!(!config.show_resurrectable_sessions);
        assert!(config.base_paths.is_empty());
        assert!(config.default_layout.is_none());
        assert!(!config.dev_mode);
    }

    #[test]
    fn target_mode_from_str_tab() {
        assert_eq!(TargetMode::from_str("tab"), TargetMode::Tab);
        assert_eq!(TargetMode::from_str("TAB"), TargetMode::Tab);
        assert_eq!(TargetMode::from_str("Tab"), TargetMode::Tab);
    }

    #[test]
    fn target_mode_from_str_session() {
        assert_eq!(TargetMode::from_str("session"), TargetMode::Session);
        assert_eq!(TargetMode::from_str("unknown"), TargetMode::Session);
        assert_eq!(TargetMode::from_str(""), TargetMode::Session);
    }

    #[test]
    fn from_zellij_config_parses_mode() {
        let mut map = BTreeMap::new();
        map.insert("mode".to_string(), "tab".to_string());
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.mode, TargetMode::Tab);
    }

    #[test]
    fn from_zellij_config_parses_separator() {
        let mut map = BTreeMap::new();
        map.insert("session_separator".to_string(), "-".to_string());
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.session_separator, "-");
    }

    #[test]
    fn from_zellij_config_parses_default_layout() {
        let mut map = BTreeMap::new();
        map.insert("default_layout".to_string(), "compact".to_string());
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.default_layout, Some("compact".to_string()));
    }

    #[test]
    fn from_zellij_config_parses_resurrectable_true() {
        let mut map = BTreeMap::new();
        map.insert("show_resurrectable_sessions".to_string(), "true".to_string());
        let config = Config::from_zellij_config(&map);
        assert!(config.show_resurrectable_sessions);
    }

    #[test]
    fn from_zellij_config_parses_resurrectable_false() {
        let mut map = BTreeMap::new();
        map.insert("show_resurrectable_sessions".to_string(), "false".to_string());
        let config = Config::from_zellij_config(&map);
        assert!(!config.show_resurrectable_sessions);
    }

    #[test]
    fn from_zellij_config_parses_base_paths() {
        let mut map = BTreeMap::new();
        map.insert("base_paths".to_string(), "/home/user|/work".to_string());
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.base_paths, vec!["/home/user", "/work"]);
    }

    #[test]
    fn from_zellij_config_handles_empty_base_paths() {
        let mut map = BTreeMap::new();
        map.insert("base_paths".to_string(), "".to_string());
        let config = Config::from_zellij_config(&map);
        assert!(config.base_paths.is_empty());
    }

    #[test]
    fn from_zellij_config_trims_base_paths() {
        let mut map = BTreeMap::new();
        map.insert("base_paths".to_string(), " /home/user | /work ".to_string());
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.base_paths, vec!["/home/user", "/work"]);
    }

    #[test]
    fn from_zellij_config_empty_map() {
        let map = BTreeMap::new();
        let config = Config::from_zellij_config(&map);
        assert_eq!(config.mode, TargetMode::Session);
        assert_eq!(config.session_separator, ".");
        assert!(!config.show_resurrectable_sessions);
        assert!(config.base_paths.is_empty());
        assert!(config.default_layout.is_none());
        assert!(!config.dev_mode);
    }

    #[test]
    fn from_zellij_config_parses_dev_mode_true() {
        let mut map = BTreeMap::new();
        map.insert("dev_mode".to_string(), "true".to_string());
        let config = Config::from_zellij_config(&map);
        assert!(config.dev_mode);
    }

    #[test]
    fn from_zellij_config_parses_dev_mode_false() {
        let mut map = BTreeMap::new();
        map.insert("dev_mode".to_string(), "false".to_string());
        let config = Config::from_zellij_config(&map);
        assert!(!config.dev_mode);
    }
}
