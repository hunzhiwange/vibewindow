use serde_json::json;

use super::*;

#[test]
fn extract_acp_error_reads_direct_and_nested_payloads() {
    let direct = json!({
        "code": -32001,
        "message": "  missing session  ",
        "data": { "sessionId": "abc" }
    });
    let nested = json!({ "error": { "cause": direct.clone() } });

    let payload = extract_acp_error(&nested).expect("nested ACP error should parse");

    assert_eq!(payload.code, -32001);
    assert_eq!(payload.message, "missing session");
    assert_eq!(payload.data, Some(json!({ "sessionId": "abc" })));
}

#[test]
fn extract_acp_error_rejects_missing_or_empty_fields() {
    for value in [
        json!({ "code": -32001 }),
        json!({ "code": "bad", "message": "missing" }),
        json!({ "code": -32001, "message": "   " }),
    ] {
        assert!(extract_acp_error(&value).is_none());
    }
}

#[test]
fn format_unknown_error_message_prefers_non_empty_message() {
    assert_eq!(format_unknown_error_message(&json!("plain")), "plain");
    assert_eq!(format_unknown_error_message(&json!({ "message": "from object" })), "from object");
    assert_eq!(format_unknown_error_message(&json!({ "message": "" })), r#"{"message":""}"#);
}

#[test]
fn resource_not_found_detects_codes_messages_and_nested_hints() {
    assert!(is_acp_resource_not_found_error(&json!({ "code": -32002, "message": "missing" })));
    assert!(is_acp_resource_not_found_error(&json!({
        "code": 1,
        "message": "Session abc_123 not found"
    })));
    assert!(is_acp_resource_not_found_error(&json!({
        "error": {
            "code": 1,
            "message": "runtime",
            "data": { "details": ["unknown session"] }
        }
    })));
    assert!(!is_acp_resource_not_found_error(&json!({ "code": 1, "message": "other" })));
}
