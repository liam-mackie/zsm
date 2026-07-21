use super::normalizer::PathNormalizer;
use std::collections::HashMap;

pub struct ConflictResolver {
    separator: String,
    normalizer: PathNormalizer,
}

impl ConflictResolver {
    pub fn new(separator: String, normalizer: PathNormalizer) -> Self {
        Self {
            separator,
            normalizer,
        }
    }

    pub fn find_basename_groups(&self, paths: &[String]) -> HashMap<String, Vec<usize>> {
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, path) in paths.iter().enumerate() {
            let basename = self.extract_basename(path);
            groups.entry(basename).or_default().push(i);
        }

        groups
    }

    pub fn resolve_name(
        &self,
        path: &str,
        all_paths: &[String],
        conflict_indices: &[usize],
        is_nested: bool,
    ) -> String {
        let normalized = self.normalizer.normalize(path);
        let segments: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

        if segments.is_empty() {
            return "root".to_string();
        }

        let initial_segments = if is_nested { 3.min(segments.len()) } else { 1 };

        let required = self.find_minimal_unique_context(
            &segments,
            initial_segments,
            all_paths,
            conflict_indices,
            path,
        );

        let final_required = if is_nested && required < 2 {
            2.min(segments.len())
        } else {
            required
        };

        let final_segments = &segments[segments.len().saturating_sub(final_required)..];
        final_segments.join(&self.separator)
    }

    pub fn is_nested(&self, path: &str, all_paths: &[String]) -> bool {
        let normalized = self.normalizer.normalize(path);
        let path_obj = std::path::Path::new(&normalized);

        for other_path in all_paths {
            if other_path == path {
                continue;
            }

            let other_normalized = self.normalizer.normalize(other_path);
            let other_path_obj = std::path::Path::new(&other_normalized);

            if path_obj.starts_with(other_path_obj) {
                return true;
            }
        }

        false
    }

    fn extract_basename(&self, path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn find_minimal_unique_context(
        &self,
        segments: &[&str],
        initial_segments: usize,
        all_paths: &[String],
        conflict_indices: &[usize],
        current_path: &str,
    ) -> usize {
        for context_len in initial_segments..=segments.len() {
            let candidate_segments = &segments[segments.len().saturating_sub(context_len)..];
            let candidate_name = candidate_segments.join(&self.separator);

            if self.is_unique_among_conflicts(
                &candidate_name,
                context_len,
                all_paths,
                conflict_indices,
                current_path,
            ) {
                return context_len;
            }
        }

        segments.len()
    }

    fn is_unique_among_conflicts(
        &self,
        candidate_name: &str,
        context_len: usize,
        all_paths: &[String],
        conflict_indices: &[usize],
        current_path: &str,
    ) -> bool {
        for &idx in conflict_indices {
            let conflict_path = &all_paths[idx];
            if conflict_path == current_path {
                continue;
            }

            let conflict_normalized = self.normalizer.normalize(conflict_path);
            let conflict_segments: Vec<&str> = conflict_normalized
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();

            if conflict_segments.len() >= context_len {
                let conflict_candidate = &conflict_segments
                    [conflict_segments.len().saturating_sub(context_len)..];
                let conflict_name = conflict_candidate.join(&self.separator);

                if candidate_name == conflict_name {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver() -> ConflictResolver {
        ConflictResolver::new(".".to_string(), PathNormalizer::new(vec![]))
    }

    #[test]
    fn find_basename_groups_single() {
        let r = resolver();
        let paths = vec!["/home/user/project".to_string()];
        let groups = r.find_basename_groups(&paths);
        assert_eq!(groups.get("project"), Some(&vec![0]));
    }

    #[test]
    fn find_basename_groups_conflicts() {
        let r = resolver();
        let paths = vec![
            "/home/user/work/project".to_string(),
            "/home/user/personal/project".to_string(),
        ];
        let groups = r.find_basename_groups(&paths);
        assert_eq!(groups.get("project"), Some(&vec![0, 1]));
    }

    #[test]
    fn resolve_name_adds_context_for_conflicts() {
        let r = resolver();
        let paths = vec![
            "/home/user/work/project".to_string(),
            "/home/user/personal/project".to_string(),
        ];
        let name = r.resolve_name(&paths[0], &paths, &[0, 1], false);
        assert_eq!(name, "work.project");
    }
}
