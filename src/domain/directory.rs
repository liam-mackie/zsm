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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dir(path: &str, ranking: f64) -> Directory {
        Directory {
            path: path.to_string(),
            ranking,
            session_name: path.to_string(),
        }
    }

    #[test]
    fn directories_sorted_by_ranking_descending() {
        let mut dirs = [make_dir("a", 10.0),
            make_dir("b", 100.0),
            make_dir("c", 50.0)];
        dirs.sort();
        assert_eq!(dirs[0].path, "b");
        assert_eq!(dirs[1].path, "c");
        assert_eq!(dirs[2].path, "a");
    }

    #[test]
    fn equal_rankings_are_equal() {
        let a = make_dir("a", 50.0);
        let b = make_dir("b", 50.0);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn higher_ranking_comes_first() {
        let high = make_dir("high", 100.0);
        let low = make_dir("low", 10.0);
        assert_eq!(high.cmp(&low), Ordering::Less);
        assert_eq!(low.cmp(&high), Ordering::Greater);
    }
}
