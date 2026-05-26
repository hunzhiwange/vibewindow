use serde_json::json;

use super::*;

#[test]
fn json_rpc_message_accepts_requests_notifications_and_responses() {
    assert!(is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "method": "session/new",
        "id": "1"
    })));
    assert!(is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "method": "session/update"
    })));
    assert!(is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {}
    })));
    assert!(is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -1, "message": "failed" }
    })));
}

#[test]
fn json_rpc_message_rejects_invalid_shapes() {
    assert!(!is_acp_json_rpc_message(&json!({ "jsonrpc": "1.0", "method": "x" })));
    assert!(!is_acp_json_rpc_message(&json!({ "jsonrpc": "2.0", "id": true, "result": {} })));
    assert!(!is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {},
        "error": { "code": -1, "message": "failed" }
    })));
    assert!(!is_acp_json_rpc_message(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": "bad", "message": "failed" }
    })));
}

#[test]
fn notification_helpers_require_session_update_payload() {
    let valid = json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": { "sessionUpdate": "plan", "entries": [] }
        }
    });

    assert!(is_json_rpc_notification(&valid));
    assert!(is_session_update_notification(&valid));
    assert!(extract_session_update_notification(&valid).is_some());
    assert!(
        extract_session_update_notification(&json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": { "sessionId": "session-1", "update": {} }
        }))
        .is_none()
    );
}

#[test]
fn parse_prompt_stop_reason_and_error_message() {
    assert_eq!(
        parse_prompt_stop_reason(&json!({ "result": { "stopReason": "end_turn" } })),
        Some("end_turn".to_string())
    );
    assert_eq!(
        parse_json_rpc_error_message(&json!({ "error": { "message": "boom" } })),
        Some("boom".to_string())
    );
    assert!(parse_prompt_stop_reason(&json!({ "result": {} })).is_none());
}
