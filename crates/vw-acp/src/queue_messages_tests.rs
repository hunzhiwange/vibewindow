use super::*;
use crate::types::{OutputErrorCode, OutputErrorOrigin, PermissionMode};
use serde_json::json;

#[test]
fn parse_queue_request_defaults_prompt_from_message() {
    let request = parse_queue_request(&json!({
        "type": "submit_prompt",
        "requestId": "req-1",
        "message": "hello",
        "permissionMode": "approve-reads",
        "waitForCompletion": true,
        "timeoutMs": 10.4
    }))
    .expect("valid request");

    match request {
        QueueRequest::SubmitPrompt { prompt, permission_mode, timeout_ms, .. } => {
            assert_eq!(permission_mode, PermissionMode::ApproveReads);
            assert_eq!(timeout_ms, Some(10));
            assert_eq!(prompt.len(), 1);
        }
        _ => panic!("unexpected request variant"),
    }
}

#[test]
fn parse_queue_request_rejects_invalid_generation_and_empty_mode() {
    assert!(
        parse_queue_request(&json!({
            "type": "cancel_prompt",
            "requestId": "req-1",
            "ownerGeneration": 0
        }))
        .is_none()
    );

    assert!(
        parse_queue_request(&json!({
            "type": "set_mode",
            "requestId": "req-1",
            "modeId": " "
        }))
        .is_none()
    );
}

#[test]
fn parse_queue_owner_error_keeps_optional_acp_payload() {
    let message = parse_queue_owner_message(&json!({
        "type": "error",
        "requestId": "req-1",
        "code": "RUNTIME",
        "origin": "queue",
        "message": "failed",
        "detailCode": " DETAIL ",
        "retryable": true,
        "acp": {"code": -1, "message": "bad", "data": {"x": 1}}
    }))
    .expect("valid error message");

    match message {
        QueueOwnerMessage::Error { code, origin, detail_code, retryable, acp, .. } => {
            assert_eq!(code, OutputErrorCode::Runtime);
            assert_eq!(origin, OutputErrorOrigin::Queue);
            assert_eq!(detail_code.as_deref(), Some("DETAIL"));
            assert_eq!(retryable, Some(true));
            assert_eq!(acp.expect("acp payload").message, "bad");
        }
        _ => panic!("unexpected message variant"),
    }
}
