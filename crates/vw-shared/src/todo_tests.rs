#[test]
fn todo_defaults_are_applied_when_fields_are_missing() {
    let todo: super::Todo = serde_json::from_str(r#"{"content":"ship","id":7}"#).unwrap();

    assert_eq!(todo.content, "ship");
    assert_eq!(todo.id, "7");
    assert_eq!(todo.status, "pending");
    assert_eq!(todo.priority, "medium");
}

#[test]
fn todo_id_accepts_string_or_number_only() {
    let string_id: super::Todo =
        serde_json::from_str(r#"{"content":"ship","id":"task-1"}"#).unwrap();
    let number_id: super::Todo = serde_json::from_str(r#"{"content":"ship","id":42}"#).unwrap();
    let invalid = serde_json::from_str::<super::Todo>(r#"{"content":"ship","id":true}"#);

    assert_eq!(string_id.id, "task-1");
    assert_eq!(number_id.id, "42");
    assert!(invalid.is_err());
}
