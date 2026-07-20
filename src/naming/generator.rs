use super::conflict::ConflictResolver;
use super::normalizer::PathNormalizer;
use super::truncator::SmartTruncator;
use crate::domain::Directory;

pub struct SessionNameGenerator {
    resolver: ConflictResolver,
    truncator: SmartTruncator,
    normalizer: PathNormalizer,
    separator: String,
    max_name_length: usize,
}

impl SessionNameGenerator {
    pub fn new(separator: String, base_paths: Vec<String>, max_name_length: usize) -> Self {
        let normalizer = PathNormalizer::new(base_paths.clone());
        let resolver = ConflictResolver::new(separator.clone(), PathNormalizer::new(base_paths));
        let truncator = SmartTruncator::new(separator.clone(), max_name_length);

        Self {
            resolver,
            truncator,
            normalizer,
            separator,
            max_name_length,
        }
    }

    pub fn generate_names(&self, directories: &mut [Directory]) {
        let paths: Vec<String> = directories.iter().map(|d| d.path.clone()).collect();
        let groups = self.resolver.find_basename_groups(&paths);

        for (basename, indices) in groups {
            if indices.len() == 1 {
                let idx = indices[0];
                let path = &paths[idx];
                let is_nested = self.resolver.is_nested(path, &paths);

                if is_nested {
                    directories[idx].session_name =
                        self.generate_name_with_context(path, &paths, &indices, true);
                } else {
                    directories[idx].session_name = basename;
                }
            } else {
                for &idx in &indices {
                    let path = &paths[idx];
                    directories[idx].session_name =
                        self.generate_name_with_context(path, &paths, &indices, false);
                }
            }
        }

        self.apply_length_limits(directories);
    }

    fn generate_name_with_context(
        &self,
        path: &str,
        all_paths: &[String],
        conflict_indices: &[usize],
        is_nested: bool,
    ) -> String {
        let is_actually_nested = is_nested || self.resolver.is_nested(path, all_paths);
        self.resolver
            .resolve_name(path, all_paths, conflict_indices, is_actually_nested)
    }

    fn apply_length_limits(&self, directories: &mut [Directory]) {
        for dir in directories.iter_mut() {
            if dir.session_name.len() > self.max_name_length {
                let normalized = self.normalizer.normalize(&dir.path);
                let segments: Vec<&str> =
                    normalized.split('/').filter(|s| !s.is_empty()).collect();

                let min_segments = dir
                    .session_name
                    .split(self.separator.as_str())
                    .count()
                    .min(segments.len());

                dir.session_name = self.truncator.truncate(&segments, min_segments);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naming::DEFAULT_MAX_NAME_LENGTH;

    fn make_dir(path: &str) -> Directory {
        Directory {
            path: path.to_string(),
            ranking: 1.0,
            session_name: String::new(),
        }
    }

    #[test]
    fn generate_simple_basename() {
        let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
        let mut dirs = vec![make_dir("/home/user/project")];
        gen.generate_names(&mut dirs);
        assert_eq!(dirs[0].session_name, "project");
    }

    #[test]
    fn generate_resolves_conflicts() {
        let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
        let mut dirs = vec![
            make_dir("/home/user/work/project"),
            make_dir("/home/user/personal/project"),
        ];
        gen.generate_names(&mut dirs);
        assert_eq!(dirs[0].session_name, "work.project");
        assert_eq!(dirs[1].session_name, "personal.project");
    }

    #[test]
    fn generate_uses_base_paths() {
        let gen = SessionNameGenerator::new(
            ".".to_string(),
            vec!["/home/user".to_string()],
            DEFAULT_MAX_NAME_LENGTH,
        );
        let mut dirs = vec![
            make_dir("/home/user/work/project"),
            make_dir("/home/user/personal/project"),
        ];
        gen.generate_names(&mut dirs);
        assert_eq!(dirs[0].session_name, "work.project");
        assert_eq!(dirs[1].session_name, "personal.project");
    }

    #[test]
    fn generate_respects_max_length() {
        let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
        let mut dirs = vec![make_dir(
            "/home/user/very/long/path/that/exceeds/the/maximum/allowed/length/for/session/names",
        )];
        gen.generate_names(&mut dirs);
        assert!(dirs[0].session_name.len() <= DEFAULT_MAX_NAME_LENGTH);
    }

    #[test]
    fn generate_respects_tight_socket_budget() {
        let gen = SessionNameGenerator::new(".".to_string(), vec![], 21);
        let mut dirs = vec![
            make_dir("/Users/liammackie/g/OctopusDeploy"),
            make_dir("/Users/liammackie/worktrees/OctopusDeploy"),
        ];
        gen.generate_names(&mut dirs);
        for dir in &dirs {
            assert!(
                dir.session_name.len() <= 21,
                "name too long: {} ({})",
                dir.session_name,
                dir.session_name.len()
            );
        }
        assert!(dirs[0].session_name.ends_with("OctopusDeploy"));
    }
}
