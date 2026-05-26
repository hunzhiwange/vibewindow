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
