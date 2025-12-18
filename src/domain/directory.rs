use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct Directory {
    pub path: String,
    pub ranking: f64,
    pub session_name: String,
}

impl Eq for Directory {}

impl PartialOrd for Directory {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Directory {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .ranking
            .partial_cmp(&self.ranking)
            .unwrap_or(Ordering::Equal)
    }
}
