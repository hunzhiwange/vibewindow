use super::todo_panel::{
    TodoPanelSurface, compute_todo_data, read_todos_for_panel, todo_id_display, todo_panel,
};
use crate::app::{App, Message, TodoPanelPlacement};
use vw_shared::todo::Todo;

fn test_app() -> App {
    App::new().0
}

fn todo(id: &str, status: &str, content: &str) -> Todo {
    Todo {
        id: id.to_string(),
        status: status.to_string(),
        content: content.to_string(),
        priority: "medium".to_string(),
    }
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn task_740_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("todo_panel_tests.rs"));
}

#[test]
fn todo_panel_placement_labels_match_ui_options() {
    assert_eq!(TodoPanelPlacement::ChatTopRight.label(), "右上角");
    assert_eq!(TodoPanelPlacement::InputBottom.label(), "输入底部");
}

#[test]
fn compute_todo_data_counts_completed_and_prefers_running_todo() {
    let app = test_app();
    let items = vec![
        todo("2", "completed", "done"),
        todo("1", "in_progress", "running task"),
        todo("3", "pending", "later"),
    ];

    let data = compute_todo_data(&app, &items);

    assert_eq!(data.total, 3);
    assert_eq!(data.completed, 1);
    assert_eq!(data.running_task, "running task");
}

#[test]
fn compute_todo_data_uses_default_text_without_running_work() {
    let app = test_app();
    let items = vec![todo("1", "pending", "later"), todo("2", "completed", "done")];

    let data = compute_todo_data(&app, &items);

    assert_eq!(data.total, 2);
    assert_eq!(data.completed, 1);
    assert_eq!(data.running_task, "无执行中任务");
}

#[test]
fn read_todos_for_panel_returns_empty_when_session_does_not_match() {
    let mut app = test_app();
    app.active_session_id = Some("active".to_string());
    app.chat_todo_session_id = Some("other".to_string());
    app.chat_todo_items = vec![todo("1", "pending", "hidden")];

    let (items, error) = read_todos_for_panel(&app).expect("panel data");

    assert!(items.is_empty());
    assert_eq!(error, None);
}

#[test]
fn read_todos_for_panel_sorts_numeric_ids_before_text_ids() {
    let mut app = test_app();
    app.active_session_id = Some("session".to_string());
    app.chat_todo_session_id = Some("session".to_string());
    app.chat_todo_items = vec![
        todo("b", "pending", "b"),
        todo("10", "pending", "ten"),
        todo("2", "pending", "two"),
        todo("a", "pending", "a"),
    ];

    let (items, error) = read_todos_for_panel(&app).expect("panel data");

    assert_eq!(error, None);
    assert_eq!(
        items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        ["2", "10", "a", "b"]
    );
}

#[test]
fn todo_id_display_trims_numeric_suffix_and_preserves_non_numeric_values() {
    assert_eq!(todo_id_display(" task_42 "), "42");
    assert_eq!(todo_id_display("42"), "42");
    assert_eq!(todo_id_display("task_alpha"), "task_alpha");
    assert_eq!(todo_id_display(""), "");
}

#[test]
fn todo_panel_builds_expanded_and_collapsed_surfaces() {
    let items = vec![
        todo("1", "completed", "done"),
        todo("2", "in_progress", "running"),
        todo("3", "pending", "later"),
    ];
    let mut app = test_app();
    app.git_changed_files = vec!["src/main.rs".to_string(), "README.md".to_string()];
    app.chat_todo_placement = TodoPanelPlacement::ChatTopRight;

    app.chat_todo_expanded = true;
    keep(todo_panel(&app, &items, 0, TodoPanelSurface::ChatTopRight));
    keep(todo_panel(&app, &items, 1, TodoPanelSurface::InputBottom));
    keep(todo_panel(&app, &items, 2, TodoPanelSurface::ChatTopRight));
    keep(todo_panel(&app, &items, 3, TodoPanelSurface::InputBottom));

    app.chat_todo_expanded = false;
    keep(todo_panel(&app, &items, 0, TodoPanelSurface::ChatTopRight));
    keep(todo_panel(&app, &items, 0, TodoPanelSurface::InputBottom));
}

#[test]
fn todo_panel_builds_empty_expanded_list() {
    let mut app = test_app();
    app.chat_todo_expanded = true;

    keep(todo_panel(&app, &[], 0, TodoPanelSurface::InputBottom));
}
