use serde::{Deserialize, Serialize};

/// Represents a directory from zoxide with its ranking and generated session name
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct ZoxideDirectory {
    /// Zoxide ranking score (higher = more frequently used)
    pub ranking: f64,
    /// Full directory path
    pub directory: String,
    /// Generated session name for this directory
    pub session_name: String,
}

impl Ord for ZoxideDirectory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by ranking in descending order (higher scores first)
        other.ranking
            .partial_cmp(&self.ranking)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(self.directory.cmp(&other.directory))
    }
}

impl PartialOrd for ZoxideDirectory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ZoxideDirectory {}

impl ZoxideDirectory {
}