#![allow(unused_must_use)]
use super::{
    handle_task_mode_add_subtask, handle_task_mode_executor_changed,
    handle_task_mode_model_changed, handle_task_mode_move_subtask_down,
    handle_task_mode_move_subtask_up, handle_task_mode_priority_changed,
    handle_task_mode_remove_subtask, handle_task_mode_subtask_changed,
    handle_task_mode_subtask_editor_action, handle_task_mode_toggled, handle_workflow_mode_toggled,
};
use crate::app::App;
use iced::widget::text_editor;

fn test_app() -> App {
    App::new().0
}

#[test]
fn task_mode_tests_module_is_wired() {
    assert!(module_path!().ends_with("task_mode_tests"));
}

#[test]
fn task_mode_scalar_settings_update_current_runtime() {
    let mut app = test_app();

    handle_task_mode_toggled(&mut app, true);
    handle_workflow_mode_toggled(&mut app, true);
    handle_task_mode_priority_changed(&mut app, "10".to_string());
    handle_task_mode_model_changed(&mut app, "  gpt-5  ".to_string());
    handle_task_mode_executor_changed(&mut app, Some("codex".to_string()));

    let runtime = app.current_session_runtime();
    assert!(runtime.task_mode_enabled);
    assert!(runtime.workflow_mode_enabled);
    assert_eq!(runtime.task_mode_priority, "10");
    assert_eq!(runtime.task_mode_model, "gpt-5");
    assert_eq!(runtime.task_mode_executor.as_deref(), Some("codex"));

    handle_task_mode_model_changed(&mut app, "   ".to_string());
    handle_task_mode_executor_changed(&mut app, None);

    let runtime = app.current_session_runtime();
    assert_eq!(runtime.task_mode_model, "auto");
    assert_eq!(runtime.task_mode_executor, None);
}

#[test]
fn task_mode_subtask_text_and_editor_stay_in_sync() {
    let mut app = test_app();

    handle_task_mode_subtask_changed(&mut app, 1, "write tests".to_string());

    let runtime = app.current_session_runtime();
    assert_eq!(runtime.task_mode_subtasks[1], "write tests");
    assert_eq!(runtime.task_mode_subtask_editors[1].text(), "write tests");

    handle_task_mode_subtask_changed(&mut app, 99, "ignored".to_string());
    assert_eq!(app.current_session_runtime().task_mode_subtasks.len(), 3);

    handle_task_mode_subtask_editor_action(
        &mut app,
        1,
        text_editor::Action::Edit(text_editor::Edit::Paste(std::sync::Arc::new(
            " now".to_string(),
        ))),
    );

    let runtime = app.current_session_runtime();
    assert_eq!(runtime.task_mode_subtask_editors[1].text(), "write tests now");
    assert_eq!(runtime.task_mode_subtasks[1], "write tests now");

    let _task =
        handle_task_mode_subtask_editor_action(&mut app, 99, text_editor::Action::SelectAll);
}

#[test]
fn task_mode_subtask_add_remove_and_move_preserve_editor_order() {
    let mut app = test_app();
    {
        let runtime = app.current_session_runtime_mut();
        runtime.task_mode_subtasks = vec!["one".into(), "two".into(), "three".into()];
        runtime.task_mode_subtask_editors = runtime
            .task_mode_subtasks
            .iter()
            .map(|value| text_editor::Content::with_text(value))
            .collect();
    }

    handle_task_mode_add_subtask(&mut app);
    assert_eq!(app.current_session_runtime().task_mode_subtasks, ["one", "two", "three", ""]);

    handle_task_mode_move_subtask_up(&mut app, 2);
    assert_eq!(app.current_session_runtime().task_mode_subtasks, ["one", "three", "two", ""]);
    assert_eq!(app.current_session_runtime().task_mode_subtask_editors[1].text(), "three");

    handle_task_mode_move_subtask_down(&mut app, 1);
    assert_eq!(app.current_session_runtime().task_mode_subtasks, ["one", "two", "three", ""]);
    assert_eq!(app.current_session_runtime().task_mode_subtask_editors[2].text(), "three");

    handle_task_mode_remove_subtask(&mut app, 1);
    assert_eq!(app.current_session_runtime().task_mode_subtasks, ["one", "three", ""]);

    handle_task_mode_remove_subtask(&mut app, 99);
    assert_eq!(app.current_session_runtime().task_mode_subtasks, ["one", "three", ""]);

    for _ in 0..3 {
        handle_task_mode_remove_subtask(&mut app, 0);
    }

    let runtime = app.current_session_runtime();
    assert_eq!(runtime.task_mode_subtasks, [""]);
    assert_eq!(runtime.task_mode_subtask_editors.len(), 1);
}
