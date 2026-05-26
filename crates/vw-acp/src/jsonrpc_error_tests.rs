use serde_json::{Value, json};

use super::jsonrpc_error::*;
use super::types::{OutputErrorAcpPayload, OutputErrorCode, OutputErrorOrigin};

#[test]
fn fallback_error_response_preserves_metadata() {
    let response = build_json_rpc_error_response(BuildJsonRpcErrorParams {
        id: Some(json!("request-1")),
        output_code: OutputErrorCode::Timeout,
        detail_code: Some("SLOW".to_string()),
        origin: Some(OutputErrorOrigin::Runtime),
        message: "timed out".to_string(),
        retryable: Some(false),
        timestamp: Some("2026-05-24T00:00:00Z".to_string()),
        session_id: Some("session-1".to_string()),
        acp: None,
    });

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, json!("request-1"));
    assert_eq!(response.error.code, -32070);
    assert_eq!(response.error.message, "timed out");
    assert_eq!(
        response.error.data,
        Some(json!({
            "vwacpCode": "TIMEOUT",
            "detailCode": "SLOW",
            "origin": "runtime",
            "retryable": false,
            "timestamp": "2026-05-24T00:00:00Z",
            "sessionId": "session-1"
        }))
    );
}

#[test]
fn valid_acp_error_takes_precedence_over_fallback_fields() {
    let response = build_json_rpc_error_response(BuildJsonRpcErrorParams {
        id: None,
        output_code: OutputErrorCode::Runtime,
        detail_code: Some("IGNORED".to_string()),
        origin: Some(OutputErrorOrigin::Acp),
        message: "fallback".to_string(),
        retryable: Some(true),
        timestamp: None,
        session_id: None,
        acp: Some(OutputErrorAcpPayload {
            code: -32000,
            message: "agent message".to_string(),
            data: Some(json!({ "methodId": "token" })),
        }),
    });

    assert_eq!(response.id, Value::Null);
    assert_eq!(response.error.code, -32000);
    assert_eq!(response.error.message, "agent message");
    assert_eq!(response.error.data, Some(json!({ "methodId": "token" })));
}
