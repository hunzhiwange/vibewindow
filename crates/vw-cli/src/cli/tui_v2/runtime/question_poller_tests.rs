use vw_gateway_client::vw_api_types::todo::{TodoPriority, TodoStatus};
use vw_shared::question;
use vw_shared::todo::Todo;

use super::question_poller::{
    filter_questions_for_session, todo_priority_from_str, todo_put_body, todo_status_from_str,
};

#[test]
fn filter_questions_for_session_returns_all_when_session_is_blank_or_missing() {
    let requests = vec![request("r1", "s1"), request("r2", "s2")];

    assert_eq!(filter_questions_for_session(requests.clone(), None).len(), 2);
    assert_eq!(filter_questions_for_session(requests.clone(), Some("   ")).len(), 2);
}

#[test]
fn filter_questions_for_session_trims_requested_session_and_matches_exact_request_session() {
    let requests = vec![request("r1", "s1"), request("r2", " s1 "), request("r3", "s2")];

    let filtered = filter_questions_for_session(requests, Some(" s1 "));
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "r1");
}

#[test]
fn todo_status_from_str_accepts_aliases_and_rejects_blank_or_unknown_values() {
    assert_eq!(todo_status_from_str("pending"), Ok(TodoStatus::Pending));
    assert_eq!(todo_status_from_str("IN_PROGRESS"), Ok(TodoStatus::InProgress));
    assert_eq!(todo_status_from_str("in-progress"), Ok(TodoStatus::InProgress));
    assert_eq!(todo_status_from_str("inprogress"), Ok(TodoStatus::InProgress));
    assert_eq!(todo_status_from_str("complete"), Ok(TodoStatus::Completed));
    assert_eq!(todo_status_from_str("done"), Ok(TodoStatus::Completed));
    assert_eq!(todo_status_from_str("canceled"), Ok(TodoStatus::Cancelled));
    assert_eq!(todo_status_from_str("  "), Err("todo status is required".to_string()));
    assert_eq!(
        todo_status_from_str("blocked"),
        Err("unsupported todo status: blocked".to_string())
    );
}

#[test]
fn todo_priority_from_str_accepts_known_values_and_rejects_blank_or_unknown_values() {
    assert_eq!(todo_priority_from_str("low"), Ok(TodoPriority::Low));
    assert_eq!(todo_priority_from_str("MEDIUM"), Ok(TodoPriority::Medium));
    assert_eq!(todo_priority_from_str("high"), Ok(TodoPriority::High));
    assert_eq!(todo_priority_from_str("  "), Err("todo priority is required".to_string()));
    assert_eq!(
        todo_priority_from_str("urgent"),
        Err("unsupported todo priority: urgent".to_string())
    );
}

#[test]
fn todo_put_body_converts_items_and_reports_first_invalid_field() {
    let body = todo_put_body(&[
        Todo {
            id: "  1  ".to_string(),
            content: "write".to_string(),
            status: "in-progress".to_string(),
            priority: "high".to_string(),
        },
        Todo {
            id: "2".to_string(),
            content: "ship".to_string(),
            status: "done".to_string(),
            priority: "low".to_string(),
        },
    ])
    .expect("valid todos should convert");

    assert_eq!(body.todos.len(), 2);
    assert_eq!(body.todos[0].id, "1");
    assert_eq!(body.todos[0].status, TodoStatus::InProgress);
    assert_eq!(body.todos[0].priority, TodoPriority::High);
    assert_eq!(body.todos[1].status, TodoStatus::Completed);

    assert_eq!(
        todo_put_body(&[Todo {
            id: " ".to_string(),
            content: "missing id".to_string(),
            status: "pending".to_string(),
            priority: "medium".to_string(),
        }])
        .unwrap_err(),
        "todo id is required"
    );
    assert_eq!(
        todo_put_body(&[Todo {
            id: "1".to_string(),
            content: "bad status".to_string(),
            status: "blocked".to_string(),
            priority: "medium".to_string(),
        }])
        .unwrap_err(),
        "unsupported todo status: blocked"
    );
}

fn request(id: &str, session_id: &str) -> question::Request {
    question::Request {
        id: id.to_string(),
        session_id: session_id.to_string(),
        questions: Vec::new(),
        tool: None,
    }
}
