#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("basic_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

#[test]
fn project_path_changed_updates_input() {
    let mut app = app();

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectPathChanged(
            "/tmp/project".to_string(),
        ),
    );

    assert!(task.is_some());
    assert_eq!(app.project_path_input, "/tmp/project");
}

#[test]
fn file_url_changed_and_add_file_pressed_append_file_once() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileUrlChanged(
            "https://example.test/a.rs".to_string(),
        ),
    );
    let _ = super::handle(&mut app, crate::app::message::project::ProjectMessage::AddFilePressed);

    assert_eq!(app.files, vec!["https://example.test/a.rs".to_string()]);
    assert!(app.file_url_input.is_empty());
}

#[test]
fn add_file_pressed_ignores_empty_input() {
    let mut app = app();
    app.file_url_input.clear();

    let _ = super::handle(&mut app, crate::app::message::project::ProjectMessage::AddFilePressed);

    assert!(app.files.is_empty());
}

#[test]
fn attachment_files_picked_appends_paths() {
    let mut app = app();
    let first = tempfile::NamedTempFile::new().expect("temp attachment should be created");
    let second = tempfile::NamedTempFile::new().expect("temp attachment should be created");
    let first_path = first.path().canonicalize().unwrap().to_string_lossy().to_string();
    let second_path = second.path().canonicalize().unwrap().to_string_lossy().to_string();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::AttachmentFilesPicked(Some(vec![
            first.path().to_string_lossy().to_string(),
            second.path().to_string_lossy().to_string(),
        ])),
    );

    assert!(app.files.contains(&first_path));
    assert!(app.files.contains(&second_path));
}

#[test]
fn remove_attached_file_removes_matching_path() {
    let mut app = app();
    app.files = vec!["/tmp/a.txt".to_string(), "/tmp/b.txt".to_string()];

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::RemoveAttachedFile("/tmp/a.txt".to_string()),
    );

    assert_eq!(app.files, vec!["/tmp/b.txt".to_string()]);
}

#[test]
fn file_index_ready_sets_index_and_stops_refreshing() {
    let mut app = app();
    app.project_path = Some("/tmp/project".to_string());
    app.file_manager_file_tree_refreshing = true;

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileIndexReady(vec![
            "/tmp/project/src/main.rs".to_string(),
        ]),
    );

    assert!(task.is_some());
    assert_eq!(app.current_file_index(), &["/tmp/project/src/main.rs".to_string()]);
    assert!(!app.file_manager_file_tree_refreshing);
}

#[test]
fn file_manager_show_changes_true_clears_active_preview_and_expands_changed_dirs() {
    let mut app = app();
    app.active_preview_path = Some("/tmp/project/src/main.rs".to_string());
    app.show_file_manager = false;
    app.git_changed_files = vec!["src/main.rs".to_string(), "README.md".to_string()];

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileManagerShowChanges(true),
    );

    assert!(app.file_manager_show_changes);
    assert!(app.active_preview_path.is_none());
    assert!(app.show_file_manager);
    assert!(app.file_tree_expanded.contains(&"src".to_string()));
}

#[test]
fn open_changed_file_without_project_returns_none_task_only() {
    let mut app = app();
    app.project_path = None;

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::OpenChangedFile("src/main.rs".to_string()),
    );

    assert!(task.is_some());
    assert!(app.project_path.is_none());
}
