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
