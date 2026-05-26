//! ACP 错误形状解析的回归测试。
//!
//! 这些用例锁定嵌套错误、资源不存在识别和未知错误展示文案，
//! 避免上层输出协议在错误结构变化时静默退化。

use serde_json::json;
use vw_acp::{extract_acp_error, format_unknown_error_message, is_acp_resource_not_found_error};

/// 验证嵌套在 JSON-RPC 响应中的 ACP 错误载荷可以被提取。
#[test]
fn extract_acp_error_finds_nested_error_payload() {
    let error = json!({
        "error": {
            "cause": {
                "acp": {
                    "code": -32002,
                    "message": "Session \"abc-123\" not found",
                    "data": {
                        "sessionId": "abc-123"
                    }
                }
            }
        }
    });

    let payload = extract_acp_error(&error).unwrap();

    assert_eq!(payload.code, -32002);
    assert_eq!(payload.message, "Session \"abc-123\" not found");
    assert_eq!(payload.data, Some(json!({ "sessionId": "abc-123" })));
}

/// 验证资源不存在错误同时支持 code 和 message 线索匹配。
#[test]
fn is_acp_resource_not_found_error_matches_code_and_message_hints() {
    let coded_error = json!({
        "error": {
            "code": -32001,
            "message": "resource_not_found"
        }
    });
    let hinted_error = json!({
        "message": "wrapper",
        "data": {
            "cause": {
                "detail": "Unknown session"
            }
        }
    });
    let text_error = json!("Session abc-123 not found");

    assert!(is_acp_resource_not_found_error(&coded_error));
    assert!(is_acp_resource_not_found_error(&hinted_error));
    assert!(is_acp_resource_not_found_error(&text_error));
}

/// 验证未知错误优先使用明确 message 字段生成用户可读文案。
#[test]
fn format_unknown_error_message_prefers_message_field() {
    let with_message = json!({
        "message": "permission denied"
    });
    let without_message = json!({
        "status": "failed"
    });

    assert_eq!(format_unknown_error_message(&with_message), "permission denied");
    assert_eq!(format_unknown_error_message(&without_message), "{\"status\":\"failed\"}");
}
