use super::*;
use crate::read_output_suppression::SUPPRESSED_READ_OUTPUT;
use crate::types::OutputFormatterContext;
use serde_json::{Value, json};

#[test]
fn json_rpc_id_key_keeps_number_and_string_namespaces_distinct() {
    assert_eq!(json_rpc_id_key(&json!("7")).as_deref(), Some("s:7"));
    assert_eq!(json_rpc_id_key(&json!(7)).as_deref(), Some("n:7"));
    assert_eq!(json_rpc_id_key(&Value::Null), None);
}

#[test]
fn sanitize_read_result_replaces_string_content_only() {
    let sanitized = sanitize_read_result(&json!({"content": "secret", "other": 1}));
    assert_eq!(sanitized["content"], SUPPRESSED_READ_OUTPUT);

    let unchanged = sanitize_read_result(&json!({"content": {"nested": true}}));
    assert_eq!(unchanged["content"], json!({"nested": true}));
}

#[test]
fn formatter_suppresses_tracked_read_response() {
    let mut formatter = JsonOutputFormatter::new(
        Vec::new(),
        true,
        Some(OutputFormatterContext { session_id: "s1".to_string() }),
    );

    let request = formatter.sanitize_message_value(
        json!({"jsonrpc": "2.0", "id": 1, "method": "fs/read_text_file", "params": {}}),
    );
    let response = formatter.sanitize_message_value(
        json!({"jsonrpc": "2.0", "id": 1, "result": {"content": "secret"}}),
    );

    assert_eq!(request["method"], "fs/read_text_file");
    assert_eq!(response["result"]["content"], SUPPRESSED_READ_OUTPUT);
}
