use super::*;

#[test]
fn fallback_tool_call_ids_are_stable_per_iteration() {
    let mut calls = vec![
        ParsedToolCall {
            name: "shell".to_string(),
            arguments: serde_json::json!({}),
            tool_call_id: None,
        },
        ParsedToolCall {
            name: "grep".to_string(),
            arguments: serde_json::json!({}),
            tool_call_id: Some("existing".to_string()),
        },
    ];

    assign_fallback_tool_call_ids(&mut calls, 2);

    assert_eq!(calls[0].tool_call_id.as_deref(), Some("fallback_3_1"));
    assert_eq!(calls[1].tool_call_id.as_deref(), Some("existing"));
}

#[test]
fn assistant_history_keeps_plain_text_without_native_tools() {
    let history = build_assistant_history("plain response", &[], &[], None, false);

    assert_eq!(history, "plain response");
}
