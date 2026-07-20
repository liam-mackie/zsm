pub struct SmartTruncator {
    separator: String,
    max_length: usize,
}

impl SmartTruncator {
    pub fn new(separator: String, max_length: usize) -> Self {
        Self {
            separator,
            max_length,
        }
    }

    /// Fits the trailing `min_segments` path segments into `max_length` bytes.
    /// The basename is the session's identity, so it is kept whole for as long
    /// as possible: context segments are abbreviated first, then dropped from
    /// the left, and the basename itself is only cut once it alone is too long.
    pub fn truncate(&self, segments: &[&str], min_segments: usize) -> String {
        let mut parts: Vec<String> = segments[segments.len().saturating_sub(min_segments)..]
            .iter()
            .map(|s| s.to_string())
            .collect();

        if self.joined_length(&parts) > self.max_length && parts.len() > 1 {
            let last = parts.len() - 1;
            for part in &mut parts[..last] {
                *part = abbreviate(part);
            }
        }

        while self.joined_length(&parts) > self.max_length && parts.len() > 1 {
            parts.remove(0);
        }

        let mut result = parts.join(&self.separator);
        truncate_to_char_boundary(&mut result, self.max_length);
        result
    }

    fn joined_length(&self, segments: &[String]) -> usize {
        if segments.is_empty() {
            0
        } else {
            segments.iter().map(|s| s.len()).sum::<usize>()
                + (segments.len() - 1) * self.separator.len()
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
        let truncator = SmartTruncator::new(".".to_string(), 29);
        let segments = vec!["very", "long", "path", "segments", "here"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= 29);
    }

    #[test]
    fn basename_kept_whole_while_context_abbreviates() {
        let truncator = SmartTruncator::new(".".to_string(), 21);
        let segments = vec!["Users", "liammackie", "g", "OctopusDeploy"];
        let result = truncator.truncate(&segments, 3);
        assert_eq!(result, "lia.g.OctopusDeploy");
    }

    #[test]
    fn context_dropped_before_basename_is_cut() {
        let truncator = SmartTruncator::new(".".to_string(), 8);
        let segments = vec!["verylongcontext", "project"];
        let result = truncator.truncate(&segments, 2);
        assert_eq!(result, "project");
    }

    #[test]
    fn basename_cut_as_last_resort() {
        let truncator = SmartTruncator::new(".".to_string(), 5);
        let segments = vec!["superduperproject"];
        let result = truncator.truncate(&segments, 1);
        assert_eq!(result, "super");
    }

    #[test]
    fn truncator_does_not_panic_on_multibyte_single_segment() {
        let truncator = SmartTruncator::new(".".to_string(), 29);
        let segments = vec!["あ_あ_あ_あ_あ_あ_あ_あ"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= 29);
    }

    #[test]
    fn truncator_does_not_panic_on_multibyte_joined_result() {
        let truncator = SmartTruncator::new(".".to_string(), 29);
        let segments = vec!["ホーム", "プロジェクト", "アプリケーション"];
        let result = truncator.truncate(&segments, 2);
        assert!(result.len() <= 29);
    }
}
