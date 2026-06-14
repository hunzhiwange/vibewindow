#[test]
fn get_task_dir_keeps_storage_inside_project_metadata_dir() {
    let dir = super::get_task_dir("/tmp/project");
    assert!(dir.ends_with(".vibewindow/tasks"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn index_db_and_lock_paths_use_task_dir() {
    assert!(super::get_index_db_path("/tmp/project").ends_with(".vibewindow/tasks/_index.sqlite3"));
    assert!(super::get_task_log_dir("/tmp/project").ends_with(".vibewindow/tasks/logs"));
}

#[test]
fn ensure_task_dir_creates_nested_metadata_directory() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().join("project").join("nested");
    let project = project.to_string_lossy().to_string();

    super::ensure_task_dir(&project).expect("task dir should be created");

    assert!(super::get_task_dir(&project).is_dir());
}

#[test]
fn with_index_lock_runs_closure_and_creates_lock_file() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let project = temp.path().to_string_lossy().to_string();

    let value = super::with_index_lock(&project, || 42);

    assert_eq!(value, 42);
    assert!(super::get_task_dir(&project).join("_index.lock").exists());
}
