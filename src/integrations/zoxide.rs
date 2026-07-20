use crate::domain::Directory;

pub fn parse_zoxide_output(output: &str) -> Vec<Directory> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
            if parts.len() == 2 {
                let score = parts[0].parse::<f64>().ok()?;
                let path = parts[1].to_string();
                Some(Directory {
                    path,
                    ranking: score,
                    session_name: String::new(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_output() {
        let output = "100.5 /home/user/project\n50.2 /home/user/other";
        let dirs = parse_zoxide_output(output);
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0].path, "/home/user/project");
        assert_eq!(dirs[0].ranking, 100.5);
        assert_eq!(dirs[1].path, "/home/user/other");
    }

    #[test]
    fn parse_empty_output() {
        let dirs = parse_zoxide_output("");
        assert!(dirs.is_empty());
    }

    #[test]
    fn parse_skips_invalid_lines() {
        let output = "invalid line\n100.5 /valid/path";
        let dirs = parse_zoxide_output(output);
        assert_eq!(dirs.len(), 1);
    }
}
