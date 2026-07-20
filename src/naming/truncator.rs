const MAX_NAME_LENGTH: usize = 29;

pub struct SmartTruncator {
    separator: String,
}

impl SmartTruncator {
    pub fn new(separator: String) -> Self {
        Self { separator }
    }

    pub fn truncate(&self, segments: &[&str], min_segments: usize) -> String {
        let mut result_segments = self.get_initial_segments(segments, min_segments);
        let mut current_length = self.joined_length(&result_segments);

        if current_length > MAX_NAME_LENGTH {
            result_segments = self.abbreviate_all(&result_segments);
            current_length = self.joined_length(&result_segments);

            while current_length > MAX_NAME_LENGTH && result_segments.len() > 1 {
                result_segments.remove(0);
                current_length = self.joined_length(&result_segments);
            }

            if current_length > MAX_NAME_LENGTH && result_segments.len() == 1 {
                truncate_to_char_boundary(&mut result_segments[0], MAX_NAME_LENGTH);
            }
        }

        self.try_add_context(segments, min_segments, &mut result_segments);

        let mut result = result_segments.join(&self.separator);
        truncate_to_char_boundary(&mut result, MAX_NAME_LENGTH);
        result
    }

    fn get_initial_segments(&self, segments: &[&str], min_segments: usize) -> Vec<String> {
        segments[segments.len().saturating_sub(min_segments)..]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn joined_length(&self, segments: &[String]) -> usize {
        if segments.is_empty() {
            0
        } else {
            segments.iter().map(|s| s.len()).sum::<usize>()
                + (segments.len() - 1) * self.separator.len()
        }
    }

    fn abbreviate_all(&self, segments: &[String]) -> Vec<String> {
        segments.iter().map(|s| abbreviate(s)).collect()
    }

    fn try_add_context(
        &self,
        segments: &[&str],
        min_segments: usize,
        result_segments: &mut Vec<String>,
    ) {
        let start_idx = segments.len().saturating_sub(min_segments);
        if start_idx == 0 {
            return;
        }

        let mut left_index = start_idx - 1;
        loop {
            let abbreviated = abbreviate(segments[left_index]);
            let mut test_segments = vec![abbreviated.clone()];
            test_segments.extend(result_segments.iter().cloned());

            if self.joined_length(&test_segments) <= MAX_NAME_LENGTH {
                result_segments.insert(0, abbreviated);
                if left_index == 0 {
                    break;
                }
                left_index -= 1;
            } else {
                break;
            }
        }
    }
}

// String::truncate panics if the byte offset splits a multibyte character
fn truncate_to_char_boundary(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
}

pub fn abbreviate(segment: &str) -> String {
    if segment.len() <= 3 {
        return segment.to_string();
    }

    if segment.contains('-') || segment.contains('_') {
        let parts: Vec<&str> = segment.split(&['-', '_'][..]).collect();
        if parts.len() > 1 {
            return parts
                .iter()
                .filter_map(|part| part.chars().next())
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("-");
        }
    }

    let chars: Vec<char> = segment.chars().collect();
    let mut abbreviated = String::new();
    abbreviated.push(chars[0]);

    for &c in &chars[1..] {
        if abbreviated.len() >= 3 {
            break;
        }
        if c.is_alphabetic() {
            abbreviated.push(c);
        }
    }

    if abbreviated.len() < 2 && chars.len() > 1 {
        abbreviated.push(chars[1]);
    }

    abbreviated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviate_short_segment() {
        assert_eq!(abbreviate("foo"), "foo");
        assert_eq!(abbreviate("ab"), "ab");
    }

    #[test]
    fn abbreviate_hyphenated_segment() {
        assert_eq!(abbreviate("lobster-watcher"), "l-w");
        assert_eq!(abbreviate("my-long-name"), "m-l-n");
    }

    #[test]
    fn abbreviate_underscored_segment() {
        assert_eq!(abbreviate("some_long_name"), "s-l-n");
    }

    #[test]
    fn abbreviate_regular_word() {
        assert_eq!(abbreviate("projects"), "pro");
        assert_eq!(abbreviate("documents"), "doc");
    }

    #[test]
    fn truncator_respects_max_length() {
        let truncator = SmartTruncator::new(".".to_string());
        let segments = vec!["very", "long", "path", "segments", "here"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= MAX_NAME_LENGTH);
    }

    #[test]
    fn truncator_does_not_panic_on_multibyte_single_segment() {
        let truncator = SmartTruncator::new(".".to_string());
        let segments = vec!["あ_あ_あ_あ_あ_あ_あ_あ"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= MAX_NAME_LENGTH);
    }

    #[test]
    fn truncator_does_not_panic_on_multibyte_joined_result() {
        let truncator = SmartTruncator::new(".".to_string());
        let segments = vec!["ホーム", "プロジェクト", "アプリケーション"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= MAX_NAME_LENGTH);
    }
}
