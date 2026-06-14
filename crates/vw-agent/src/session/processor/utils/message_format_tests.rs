use crate::app::agent::session::llm;

#[test]
fn now_ms_is_close_to_system_time() {
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_millis() as u64;
    let now = super::now_ms();
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_millis() as u64;

    assert!(now >= before);
    assert!(now <= after.saturating_add(1));
}

#[test]
fn tool_calls_to_assistant_message_preserves_call_shape_and_reasoning() {
    let calls = vec![
        llm::ToolCall {
            id: "call-1".to_string(),
            name: "file_read".to_string(),
            arguments: r#"{"path":"src/lib.rs"}"#.to_string(),
        },
        llm::ToolCall {
            id: "call-2".to_string(),
            name: "bash".to_string(),
            arguments: r#"{"command":"echo ok"}"#.to_string(),
        },
    ];

    let message = super::tool_calls_to_assistant_message(&calls, "thinking");
    let tool_calls =
        message.get("tool_calls").and_then(serde_json::Value::as_array).expect("tool calls");

    assert_eq!(message.get("role").and_then(serde_json::Value::as_str), Some("assistant"));
    assert_eq!(
        message.get("reasoning_content").and_then(serde_json::Value::as_str),
        Some("thinking")
    );
    assert_eq!(tool_calls.len(), 2);
    assert_eq!(tool_calls[0].get("id").and_then(serde_json::Value::as_str), Some("call-1"));
    assert_eq!(
        tool_calls[0]
            .get("function")
            .and_then(|function| function.get("name"))
            .and_then(serde_json::Value::as_str),
        Some("file_read")
    );
    assert_eq!(
        tool_calls[1]
            .get("function")
            .and_then(|function| function.get("arguments"))
            .and_then(serde_json::Value::as_str),
        Some(r#"{"command":"echo ok"}"#)
    );
}

#[test]
fn tool_calls_to_assistant_message_handles_empty_calls() {
    let message = super::tool_calls_to_assistant_message(&[], "");

    assert_eq!(
        message.get("tool_calls").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(message.get("reasoning_content").and_then(serde_json::Value::as_str), Some(""));
}

#[test]
fn tool_result_to_message_uses_tool_role_and_call_id() {
    let message = super::tool_result_to_message("call-9", "done");

    assert_eq!(message.get("role").and_then(serde_json::Value::as_str), Some("tool"));
    assert_eq!(message.get("tool_call_id").and_then(serde_json::Value::as_str), Some("call-9"));
    assert_eq!(message.get("content").and_then(serde_json::Value::as_str), Some("done"));
}

#[test]
fn assistant_message_with_reasoning_includes_content_and_reasoning() {
    let message = super::assistant_message_with_reasoning("answer", "chain");

    assert_eq!(message.get("role").and_then(serde_json::Value::as_str), Some("assistant"));
    assert_eq!(message.get("content").and_then(serde_json::Value::as_str), Some("answer"));
    assert_eq!(message.get("reasoning_content").and_then(serde_json::Value::as_str), Some("chain"));
}
