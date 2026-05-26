//! ACP JSON-RPC 消息识别与辅助解析测试。
//!
//! 用例覆盖通知、请求、响应和会话更新字段，确保协议边界只接受
//! 结构完整的 JSON-RPC 消息。

use serde_json::json;
use vw_acp::{
    extract_session_update_notification, is_acp_json_rpc_message, is_json_rpc_notification,
    is_session_update_notification, parse_json_rpc_error_message, parse_prompt_stop_reason,
};

/// 验证通知、请求和响应三类 JSON-RPC 消息都能被识别。
#[test]
fn is_acp_json_rpc_message_accepts_notifications_requests_and_responses() {
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": {
                    "type": "text",
                    "text": "hello"
                }
            }
        }
    });
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "session/send",
        "params": {}
    });
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "stopReason": "end_turn"
        }
    });
    let invalid = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {},
        "error": {
            "code": -32603,
            "message": "boom"
        }
    });

    assert!(is_acp_json_rpc_message(&notification));
    assert!(is_acp_json_rpc_message(&request));
    assert!(is_acp_json_rpc_message(&response));
    assert!(!is_acp_json_rpc_message(&invalid));
}

/// 验证会话更新辅助函数能从 SDK 通知中提取结构化内容。
#[test]
fn session_update_helpers_extract_sdk_notification() {
    let message = json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "plan",
                "entries": []
            }
        }
    });

    assert!(is_json_rpc_notification(&message));
    assert!(is_session_update_notification(&message));

    let notification = extract_session_update_notification(&message).unwrap();
    let serialized = serde_json::to_value(notification).unwrap();

    assert_eq!(serialized["sessionId"], json!("session-1"));
    assert_eq!(serialized["update"]["sessionUpdate"], json!("plan"));
}

/// 验证提示停止原因和错误文案能从 ACP 响应中稳定解析。
#[test]
fn prompt_stop_reason_and_error_message_are_parsed() {
    let result_message = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "result": {
            "stopReason": "end_turn"
        }
    });
    let error_message = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "error": {
            "code": -32603,
            "message": "internal error"
        }
    });

    assert_eq!(parse_prompt_stop_reason(&result_message), Some("end_turn".to_string()));
    assert_eq!(parse_json_rpc_error_message(&error_message), Some("internal error".to_string()));
}
