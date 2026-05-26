use super::*;

#[test]
fn autosave_memory_key_uses_prefix_and_unique_suffix() {
    let first = autosave_memory_key("chat");
    let second = autosave_memory_key("chat");

    assert!(first.starts_with("chat_"));
    assert!(second.starts_with("chat_"));
    assert_ne!(first, second);
}

#[test]
fn parsed_tool_calls_build_native_assistant_history() {
    let calls = vec![ParsedToolCall {
        name: "shell".to_string(),
        arguments: serde_json::json!({"command": "pwd"}),
        tool_call_id: Some("call-1".to_string()),
    }];

    let history = build_native_assistant_history_from_parsed_calls("text", &calls, Some("think"))
        .expect("parsed calls should build history");

    assert!(history.contains("call-1"));
    assert!(history.contains("shell"));
    assert!(history.contains("think"));
}
