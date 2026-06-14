use std::fs;

use vw_shared::task::Task;

use super::{
    first_non_empty_line, parse_temperature, print_task_json, print_tasks_json, resolve_project_dir,
};

#[test]
fn parse_temperature_accepts_inclusive_bounds_and_rejects_invalid_values() {
    assert_eq!(parse_temperature("0").unwrap(), 0.0);
    assert_eq!(parse_temperature("2.0").unwrap(), 2.0);
    assert_eq!(parse_temperature("2.01").unwrap_err(), "temperature must be between 0.0 and 2.0");
    assert!(parse_temperature("not-a-number").is_err());
}

#[test]
fn first_non_empty_line_trims_and_returns_empty_when_content_is_blank() {
    assert_eq!(first_non_empty_line("\n \t\nfirst\nsecond"), "first");
    assert_eq!(first_non_empty_line("  leading  \nsecond"), "leading");
    assert_eq!(first_non_empty_line("\n \t"), "");
}

#[test]
fn resolve_project_dir_rejects_empty_missing_and_file_paths() {
    assert!(
        resolve_project_dir("   ")
            .unwrap_err()
            .to_string()
            .contains("--project-dir cannot be empty")
    );
    assert!(resolve_project_dir("/tmp/vibewindow-missing-project-dir-for-test").is_err());

    let file_path =
        std::env::temp_dir().join(format!("vw-cli-resolve-project-file-{}", std::process::id()));
    fs::write(&file_path, "not a directory").unwrap();
    let error = resolve_project_dir(file_path.to_str().unwrap()).unwrap_err().to_string();
    fs::remove_file(&file_path).unwrap();

    assert!(error.contains("project directory is not a folder"));
}

#[test]
fn resolve_project_dir_returns_canonical_directory() {
    let dir_path =
        std::env::temp_dir().join(format!("vw-cli-resolve-project-dir-{}", std::process::id()));
    fs::create_dir_all(&dir_path).unwrap();

    let resolved = resolve_project_dir(dir_path.to_str().unwrap()).unwrap();
    let expected = fs::canonicalize(&dir_path).unwrap().to_string_lossy().to_string();
    fs::remove_dir(&dir_path).unwrap();

    assert_eq!(resolved, expected);
}

#[test]
fn print_task_json_helpers_serialize_tasks_without_error() {
    let task = Task::new(3);
    let tasks = vec![task.clone(), Task::new(5)];

    print_task_json(&task).unwrap();
    print_tasks_json(&tasks).unwrap();
}
