use super::handle;
use crate::app::App;
use crate::app::message::project::ProjectMessage;

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("session_tests"));
}

#[test]
fn handle_returns_none_for_non_session_message() {
    let (mut app, _task) = App::new();

    let task = handle(&mut app, ProjectMessage::ProjectPathChanged("/tmp/project".to_string()));

    assert!(task.is_none());
}

#[test]
fn handle_dispatches_project_create_session_picker_close_to_session_modules() {
    let (mut app, _task) = App::new();
    app.new_session_picker_project = Some("/tmp/project".to_string());

    let _task = handle(&mut app, ProjectMessage::ProjectCreateSessionPickerClose)
        .expect("close picker should be handled");

    assert!(app.new_session_picker_project.is_none());
}
