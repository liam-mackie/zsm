use super::*;
use crate::domain::Directory;
use crate::naming::DEFAULT_MAX_NAME_LENGTH;

fn make_dir(path: &str) -> Directory {
    Directory {
        path: path.to_string(),
        ranking: 1.0,
        session_name: String::new(),
    }
}

#[test]
fn nested_directories_get_context() {
    let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
    let mut dirs = vec![
        make_dir("/home/user/project"),
        make_dir("/home/user/project/frontend"),
    ];
    gen.generate_names(&mut dirs);

    assert_eq!(dirs[0].session_name, "project");
    assert!(
        dirs[1].session_name.contains("frontend"),
        "nested dir should include frontend: {}",
        dirs[1].session_name
    );
}

#[test]
fn triple_conflict_resolution() {
    let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
    let mut dirs = vec![
        make_dir("/home/user/work/client/app"),
        make_dir("/home/user/personal/client/app"),
        make_dir("/home/user/other/app"),
    ];
    gen.generate_names(&mut dirs);

    let names: Vec<&str> = dirs.iter().map(|d| d.session_name.as_str()).collect();
    let unique_names: std::collections::HashSet<&str> = names.iter().copied().collect();

    assert_eq!(
        names.len(),
        unique_names.len(),
        "all names should be unique: {:?}",
        names
    );
}

#[test]
fn custom_separator() {
    let gen = SessionNameGenerator::new("-".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
    let mut dirs = vec![
        make_dir("/home/user/work/project"),
        make_dir("/home/user/personal/project"),
    ];
    gen.generate_names(&mut dirs);

    assert!(dirs[0].session_name.contains('-'));
    assert!(dirs[1].session_name.contains('-'));
}

#[test]
fn base_paths_are_stripped() {
    let gen = SessionNameGenerator::new(
        ".".to_string(),
        vec!["/home/user/projects".to_string()],
        DEFAULT_MAX_NAME_LENGTH,
    );
    let mut dirs = vec![make_dir("/home/user/projects/myapp")];
    gen.generate_names(&mut dirs);

    assert_eq!(dirs[0].session_name, "myapp");
}

#[test]
fn long_names_are_truncated() {
    let gen = SessionNameGenerator::new(".".to_string(), vec![], DEFAULT_MAX_NAME_LENGTH);
    let mut dirs = vec![
        make_dir("/home/user/really-long-directory-name/another-very-long-name/project"),
        make_dir("/home/user/different-long-path/another-very-long-name/project"),
    ];
    gen.generate_names(&mut dirs);

    for dir in &dirs {
        assert!(
            dir.session_name.len() <= DEFAULT_MAX_NAME_LENGTH,
            "name too long: {} ({})",
            dir.session_name,
            dir.session_name.len()
        );
    }
}
