//! `SessionLlmProvider` 的消息投影与用量解析测试。
//!
//! 这些测试聚焦 provider 和 gateway 边界的数据转换，避免直接依赖真实 gateway 进程。

use super::SessionLlmProvider;
use crate::app::agent::providers::ChatMessage;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_usage_maps_gateway_done_usage() {
    let payload = serde_json::json!({
        "input_tokens": 120,
        "output_tokens": 45,
        "cached_tokens": 30,
        "reasoning_tokens": 12,
    });

    let usage = SessionLlmProvider::parse_usage(Some(&payload)).expect("usage should parse");

    assert_eq!(usage.input_tokens, Some(120));
    assert_eq!(usage.output_tokens, Some(45));
    assert_eq!(usage.cached_tokens, Some(30));
    assert_eq!(usage.reasoning_tokens, Some(12));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parse_usage_returns_none_without_payload() {
    assert!(SessionLlmProvider::parse_usage(None).is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_history_messages_preserves_supported_roles() {
    let messages = vec![
        ChatMessage::system("sys"),
        ChatMessage::user("u1"),
        ChatMessage::assistant("a1"),
        ChatMessage::user("u2"),
    ];

    let projected = SessionLlmProvider::build_history_messages(&messages);

    assert_eq!(projected.len(), 4);
    assert_eq!(projected[0].get("role").and_then(serde_json::Value::as_str), Some("system"));
    assert_eq!(projected[1].get("role").and_then(serde_json::Value::as_str), Some("user"));
    assert_eq!(projected[2].get("role").and_then(serde_json::Value::as_str), Some("assistant"));
    assert_eq!(projected[3].get("content").and_then(serde_json::Value::as_str), Some("u2"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn build_history_messages_preserves_tool_role_for_gateway_boundary() {
    // 工具消息内容通常包含结构化字段，投影时必须原样保留给 gateway 继续解释。
    let tool_message = ChatMessage::tool(
        serde_json::json!({
            "tool_call_id": "call_123",
            "content": "pwd\n/Users/demo"
        })
        .to_string(),
    );

    let projected = SessionLlmProvider::build_history_messages(&[tool_message]);

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].get("role").and_then(serde_json::Value::as_str), Some("tool"));
    let content = projected[0]
        .get("content")
        .and_then(serde_json::Value::as_str)
        .expect("tool projection should keep content");
    assert!(content.contains("tool_call_id"));
    assert!(content.contains("/Users/demo"));
}
