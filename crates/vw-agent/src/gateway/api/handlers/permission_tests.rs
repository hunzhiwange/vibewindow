use super::*;
use crate::app::agent::config::AutonomyConfig;
use axum::http::StatusCode;

fn pending_request(
    reason: Option<&str>,
    arguments: serde_json::Value,
    message_id: Option<&str>,
    call_id: Option<&str>,
) -> PendingNonCliApprovalRequest {
    PendingNonCliApprovalRequest {
        request_id: "apr-1".to_string(),
        tool_name: "shell.exec".to_string(),
        arguments,
        message_id: message_id.map(str::to_string),
        call_id: call_id.map(str::to_string),
        requested_by: "alice".to_string(),
        requested_channel: "telegram".to_string(),
        requested_reply_target: "chat-1".to_string(),
        reason: reason.map(str::to_string),
        created_at: "2026-01-02T03:04:05Z".to_string(),
        expires_at: "2026-01-02T03:34:05Z".to_string(),
    }
}

fn manager_with_pending_request() -> (crate::app::agent::approval::ApprovalManager, String) {
    let manager =
        crate::app::agent::approval::ApprovalManager::from_config(&AutonomyConfig::default());
    let request = manager.create_non_cli_pending_request(
        "shell.exec",
        "alice",
        "telegram",
        "chat-1",
        Some("needs approval".to_string()),
        serde_json::json!({"command": "pwd"}),
        Some("msg-1".to_string()),
        Some("call-1".to_string()),
    );

    (manager, request.request_id)
}

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn permission_reply_request_deserializes_reply_field() {
    let body: PermissionReplyRequest = serde_json::from_value(serde_json::json!({"reply": "once"}))
        .expect("valid permission reply");

    assert_eq!(body.reply, permission_next::Reply::Once);
    assert!(body.message.is_none());
}

#[test]
fn permission_reply_request_deserializes_all_reply_variants_and_message() {
    let always: PermissionReplyRequest = serde_json::from_value(serde_json::json!({
        "reply": "always",
        "message": "remember this"
    }))
    .expect("valid always reply");
    let reject: PermissionReplyRequest =
        serde_json::from_value(serde_json::json!({"reply": "reject"})).expect("valid reject reply");

    assert_eq!(always.reply, permission_next::Reply::Always);
    assert_eq!(always.message.as_deref(), Some("remember this"));
    assert_eq!(reject.reply, permission_next::Reply::Reject);
    assert!(reject.message.is_none());
}

#[test]
fn permission_request_from_pending_maps_metadata_and_tool_info() {
    let request = permission_request_from_pending(pending_request(
        Some(" needs approval "),
        serde_json::json!({"command": "pwd"}),
        Some("msg-1"),
        Some("call-1"),
    ));

    assert_eq!(request.id, "apr-1");
    assert_eq!(request.session_id, "chat-1");
    assert_eq!(request.permission, "shell.exec");
    assert!(request.patterns.is_empty());
    assert_eq!(request.always, vec!["shell.exec".to_string()]);
    assert_eq!(request.metadata["reason"], " needs approval ");
    assert_eq!(request.metadata["arguments"], serde_json::json!({"command": "pwd"}));
    assert_eq!(request.metadata["requested_by"], "alice");
    assert_eq!(request.metadata["requested_channel"], "telegram");
    assert_eq!(request.metadata["requested_reply_target"], "chat-1");
    assert_eq!(request.metadata["created_at"], "2026-01-02T03:04:05Z");
    assert_eq!(request.metadata["expires_at"], "2026-01-02T03:34:05Z");

    let tool = request.tool.expect("tool info");
    assert_eq!(tool.message_id, "msg-1");
    assert_eq!(tool.call_id, "call-1");
}

#[test]
fn permission_request_from_pending_skips_blank_optional_fields() {
    let request = permission_request_from_pending(pending_request(
        Some("   "),
        serde_json::Value::Null,
        Some("msg-1"),
        Some("  "),
    ));

    assert!(!request.metadata.contains_key("reason"));
    assert!(!request.metadata.contains_key("arguments"));
    assert!(request.tool.is_none());
}

#[test]
fn map_pending_approval_error_preserves_public_status_and_message() {
    let cases = [
        (PendingApprovalError::NotFound, StatusCode::NOT_FOUND, "request not found"),
        (PendingApprovalError::Expired, StatusCode::BAD_REQUEST, "request expired"),
        (
            PendingApprovalError::RequesterMismatch,
            StatusCode::BAD_REQUEST,
            "request actor mismatch",
        ),
    ];

    for (input, status, message) in cases {
        let error = map_pending_approval_error(input);
        assert_eq!(error.status, status);
        assert_eq!(error.to_string(), message);
    }
}

#[tokio::test]
async fn reply_pending_request_returns_false_for_missing_request() {
    let manager =
        crate::app::agent::approval::ApprovalManager::from_config(&AutonomyConfig::default());

    let applied =
        reply_pending_request(&manager, "apr-missing", permission_next::Reply::Once, None)
            .await
            .expect("missing request is not an api error");

    assert!(!applied);
}

#[tokio::test]
async fn reply_pending_request_once_confirms_and_records_yes_resolution() {
    let (manager, request_id) = manager_with_pending_request();

    let applied = reply_pending_request(
        &manager,
        &request_id,
        permission_next::Reply::Once,
        Some("approved".to_string()),
    )
    .await
    .expect("reply should apply");

    assert!(applied);
    assert!(!manager.has_non_cli_pending_request(&request_id));
    assert_eq!(manager.take_non_cli_pending_resolution(&request_id), Some(ApprovalResponse::Yes));
    assert!(!manager.is_non_cli_session_granted("shell.exec"));
}

#[tokio::test]
async fn reply_pending_request_reject_removes_request_and_records_no_resolution() {
    let (manager, request_id) = manager_with_pending_request();

    let applied =
        reply_pending_request(&manager, &request_id, permission_next::Reply::Reject, None)
            .await
            .expect("reply should apply");

    assert!(applied);
    assert!(!manager.has_non_cli_pending_request(&request_id));
    assert_eq!(manager.take_non_cli_pending_resolution(&request_id), Some(ApprovalResponse::No));
}
