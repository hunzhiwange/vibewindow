use super::*;

#[test]
fn immediate_failure_preserves_tool_identity_and_marks_failure() {
    let (tool_name, call_id, outcome) = immediate_failure(
        "shell".to_string(),
        Some("call-1".to_string()),
        "blocked".to_string(),
    );

    assert_eq!(tool_name, "shell");
    assert_eq!(call_id.as_deref(), Some("call-1"));
    assert_eq!(outcome.tool_name, "shell");
    assert_eq!(outcome.error_reason.as_deref(), Some("blocked"));
    assert!(!outcome.success);
}
