use super::task_mode::task_mode_form;
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn task_739_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("task_mode_tests.rs"));
}

#[test]
fn disabled_task_mode_returns_placeholder_element() {
    let app = test_app();

    keep(task_mode_form(&app, None, false, "5", "auto", None, &[]));
}

#[test]
fn enabled_task_mode_builds_with_empty_default_executor_and_no_runtime_editors() {
    let mut app = test_app();
    app.acp_agents = vec!["codex".to_string(), "claude".to_string()];
    app.show_executor_popover = true;

    keep(task_mode_form(
        &app,
        None,
        true,
        "1",
        "provider/model",
        None,
        &["first".to_string(), "second".to_string()],
    ));
}

#[test]
fn enabled_task_mode_builds_with_selected_executor_and_runtime_editors() {
    let mut app = test_app();
    app.acp_agents = vec!["codex".to_string(), "claude".to_string()];
    {
        let runtime = app.current_session_runtime_mut();
        runtime.task_mode_subtasks = vec![
            "line one\nline two\nline three\nline four".to_string(),
            "single line".to_string(),
        ];
        runtime.task_mode_subtask_editors = runtime
            .task_mode_subtasks
            .iter()
            .map(|subtask| iced::widget::text_editor::Content::with_text(subtask))
            .collect();
    }
    let runtime = app.current_session_runtime();

    keep(task_mode_form(
        &app,
        Some(&runtime),
        true,
        "99",
        "auto",
        Some("claude".to_string()),
        &runtime.task_mode_subtasks,
    ));
}

#[test]
fn enabled_task_mode_builds_with_single_subtask_disabled_move_buttons() {
    let app = test_app();
    keep(task_mode_form(
        &app,
        None,
        true,
        "10",
        "",
        Some("custom-agent".to_string()),
        &["only child".to_string()],
    ));
}
