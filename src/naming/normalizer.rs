pub struct PathNormalizer {
    base_paths: Vec<String>,
}

impl PathNormalizer {
    pub fn new(base_paths: Vec<String>) -> Self {
        Self { base_paths }
    }

    pub fn normalize(&self, path: &str) -> String {
        if self.base_paths.is_empty() {
            return path.to_string();
        }

        let (longest_match, longest_match_len) = self.find_longest_matching_base(path);

        if let Some(base_path) = longest_match {
            self.strip_base_path(path, base_path, longest_match_len)
        } else {
            path.to_string()
        }
    }

    fn find_longest_matching_base(&self, path: &str) -> (Option<&String>, usize) {
        let mut longest_match: Option<&String> = None;
        let mut longest_match_len = 0;

        for base_path in &self.base_paths {
            let normalized_base = base_path.trim_end_matches('/');

            if path.starts_with(normalized_base) && self.is_directory_boundary(path, normalized_base)
                && normalized_base.len() > longest_match_len {
                    longest_match = Some(base_path);
                    longest_match_len = normalized_base.len();
                }
        }

        (longest_match, longest_match_len)
    }

    fn is_directory_boundary(&self, path: &str, normalized_base: &str) -> bool {
        path.len() == normalized_base.len()
            || path.chars().nth(normalized_base.len()) == Some('/')
    }

    fn strip_base_path(&self, path: &str, base_path: &str, _len: usize) -> String {
        let normalized_base = base_path.trim_end_matches('/');

        if path == normalized_base {
            return path.to_string();
        }

        if let Some(stripped) = path.strip_prefix(normalized_base) {
            let stripped = stripped.strip_prefix('/').unwrap_or(stripped);
            if !stripped.is_empty() {
                return stripped.to_string();
            }
        }

        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_matching_base_path() {
        let normalizer = PathNormalizer::new(vec!["/home/user/projects".to_string()]);
        assert_eq!(
            normalizer.normalize("/home/user/projects/myapp"),
            "myapp"
        );
    }

    #[test]
    fn normalize_strips_longest_matching_base() {
        let normalizer = PathNormalizer::new(vec![
            "/home/user".to_string(),
            "/home/user/projects".to_string(),
        ]);
        assert_eq!(
            normalizer.normalize("/home/user/projects/myapp"),
            "myapp"
        );
    }

    #[test]
    fn normalize_returns_original_when_no_match() {
        let normalizer = PathNormalizer::new(vec!["/other/path".to_string()]);
        assert_eq!(
            normalizer.normalize("/home/user/projects/myapp"),
            "/home/user/projects/myapp"
        );
    }

    #[test]
    fn normalize_handles_trailing_slash() {
        let normalizer = PathNormalizer::new(vec!["/home/user/projects/".to_string()]);
        assert_eq!(
            normalizer.normalize("/home/user/projects/myapp"),
            "myapp"
        );
    }

    #[test]
    fn normalize_respects_directory_boundary() {
        let normalizer = PathNormalizer::new(vec!["/home/user/pro".to_string()]);
        assert_eq!(
            normalizer.normalize("/home/user/projects/myapp"),
            "/home/user/projects/myapp"
        );
    }
}
