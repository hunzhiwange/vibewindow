use super::*;

#[test]
fn todo_status_and_priority_parse_with_safe_defaults() {
    assert_eq!(todo_status_to_str(&TodoStatus::InProgress), "in_progress");
    assert_eq!(todo_priority_to_str(&TodoPriority::High), "high");
    assert!(matches!(todo_status_from_str("completed"), TodoStatus::Completed));
    assert!(matches!(todo_status_from_str("unknown"), TodoStatus::Pending));
    assert!(matches!(todo_priority_from_str("low"), TodoPriority::Low));
    assert!(matches!(todo_priority_from_str("unknown"), TodoPriority::Medium));
}

#[test]
fn todo_status_and_priority_cover_all_variants() {
    assert_eq!(todo_status_to_str(&TodoStatus::Pending), "pending");
    assert_eq!(todo_status_to_str(&TodoStatus::InProgress), "in_progress");
    assert_eq!(todo_status_to_str(&TodoStatus::Completed), "completed");
    assert_eq!(todo_status_to_str(&TodoStatus::Cancelled), "cancelled");

    assert_eq!(todo_priority_to_str(&TodoPriority::Low), "low");
    assert_eq!(todo_priority_to_str(&TodoPriority::Medium), "medium");
    assert_eq!(todo_priority_to_str(&TodoPriority::High), "high");

    assert!(matches!(todo_status_from_str("in_progress"), TodoStatus::InProgress));
    assert!(matches!(todo_status_from_str("cancelled"), TodoStatus::Cancelled));
    assert!(matches!(todo_priority_from_str("high"), TodoPriority::High));
}

#[tokio::test]
async fn update_persists_ui_todos_and_get_round_trips_gateway_shape() {
    crate::session::ui_store::set_session_scope(None);
    let session_id = format!("todo-roundtrip-{}", std::process::id());
    let todos = vec![
        Info {
            id: "one".to_string(),
            content: "write tests".to_string(),
            status: TodoStatus::InProgress,
            priority: TodoPriority::High,
        },
        Info {
            id: "two".to_string(),
            content: "ship".to_string(),
            status: TodoStatus::Completed,
            priority: TodoPriority::Low,
        },
    ];

    update(UpdateInput { session_id: session_id.clone(), todos }).await.unwrap();
    let loaded = get(&session_id).await;

    assert_eq!(loaded.len(), 2);
    assert!(matches!(loaded[0].status, TodoStatus::InProgress));
    assert!(matches!(loaded[0].priority, TodoPriority::High));
    assert!(matches!(loaded[1].status, TodoStatus::Completed));
    assert!(matches!(loaded[1].priority, TodoPriority::Low));
}
