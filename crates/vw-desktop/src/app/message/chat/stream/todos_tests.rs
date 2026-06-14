#![allow(unused_must_use)]
use crate::app::App;

fn todo(id: &str, status: &str) -> vw_shared::todo::Todo {
    vw_shared::todo::Todo {
        id: id.to_string(),
        content: format!("todo {id}"),
        status: status.to_string(),
        priority: "medium".to_string(),
    }
}

#[test]
fn loaded_todos_for_inactive_session_are_ignored() {
    let (mut app, _task) = App::new();
    app.active_session_id = Some("active".to_string());
    app.chat_todo_session_id = Some("active".to_string());
    app.chat_todo_items = vec![todo("old", "pending")];

    super::handle_input_panel_todos_loaded(
        &mut app,
        "other".to_string(),
        Ok(vec![todo("new", "pending")]),
    );

    assert_eq!(app.chat_todo_session_id.as_deref(), Some("active"));
    assert_eq!(app.chat_todo_items[0].id, "old");
}

#[test]
fn loaded_pending_todos_expand_panel_for_new_session() {
    let (mut app, _task) = App::new();
    app.active_session_id = Some("active".to_string());
    app.chat_todo_expanded = false;
    app.chat_todo_anim = 0.0;

    super::handle_input_panel_todos_loaded(
        &mut app,
        "active".to_string(),
        Ok(vec![todo("1", "pending")]),
    );

    assert_eq!(app.chat_todo_session_id.as_deref(), Some("active"));
    assert!(app.chat_todo_expanded);
    assert_eq!(app.chat_todo_anim, 1.0);
    assert_eq!(app.chat_todo_items.len(), 1);
}

#[test]
fn loaded_completed_todos_auto_collapse_open_panel() {
    let (mut app, _task) = App::new();
    app.active_session_id = Some("active".to_string());
    app.chat_todo_session_id = Some("active".to_string());
    app.chat_todo_expanded = true;
    app.chat_todo_anim = 1.0;

    super::handle_input_panel_todos_loaded(
        &mut app,
        "active".to_string(),
        Ok(vec![todo("1", "completed"), todo("2", "completed")]),
    );

    assert!(!app.chat_todo_expanded);
    assert_eq!(app.chat_todo_items.len(), 2);
}

#[test]
fn load_todos_without_active_session_clears_cached_items() {
    let (mut app, _task) = App::new();
    app.chat_todo_session_id = Some("old".to_string());
    app.chat_todo_items = vec![todo("old", "pending")];

    super::handle_load_input_panel_todos(&mut app);

    assert_eq!(app.chat_todo_session_id, None);
    assert!(app.chat_todo_items.is_empty());
}

#[test]
fn loaded_todos_error_records_session_and_clears_items() {
    let (mut app, _task) = App::new();
    app.active_session_id = Some("active".to_string());
    app.chat_todo_items = vec![todo("old", "pending")];

    super::handle_input_panel_todos_loaded(
        &mut app,
        "active".to_string(),
        Err("gateway down".to_string()),
    );

    assert_eq!(app.chat_todo_session_id.as_deref(), Some("active"));
    assert!(app.chat_todo_items.is_empty());
}
