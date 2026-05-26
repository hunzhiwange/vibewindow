use chrono::{Duration, Utc};

use crate::app::agent::config::AutonomyConfig;

use super::{
    ApprovalManager, ApprovalResponse, PendingApprovalError, PendingNonCliApprovalRequest,
};

#[test]
fn pending_request_deduplicates_and_resolves_by_requester_context() {
    let manager = ApprovalManager::from_config(&AutonomyConfig::default());
    let args = serde_json::json!({"path": "README.md"});
    let first = manager.create_non_cli_pending_request(
        "file_read",
        "alice",
        "discord",
        "chan-1",
        Some("need approval".to_string()),
        args.clone(),
        Some("msg-1".to_string()),
        Some("call-1".to_string()),
    );
    let second = manager.create_non_cli_pending_request(
        "file_read",
        "alice",
        "discord",
        "chan-1",
        Some("need approval".to_string()),
        args,
        Some("msg-1".to_string()),
        Some("call-1".to_string()),
    );

    assert_eq!(first.request_id, second.request_id);
    assert!(manager.has_non_cli_pending_request(&first.request_id));
    assert_eq!(
        manager
            .confirm_non_cli_pending_request(&first.request_id, "bob", "discord", "chan-1")
            .unwrap_err(),
        PendingApprovalError::RequesterMismatch
    );
    assert!(
        manager
            .confirm_non_cli_pending_request(&first.request_id, "alice", "discord", "chan-1")
            .is_ok()
    );
    assert!(!manager.has_non_cli_pending_request(&first.request_id));
}

#[test]
fn pending_resolution_only_records_yes_or_no() {
    let manager = ApprovalManager::from_config(&AutonomyConfig::default());

    manager.record_non_cli_pending_resolution("apr-1", ApprovalResponse::Always);
    assert_eq!(manager.take_non_cli_pending_resolution("apr-1"), None);

    manager.record_non_cli_pending_resolution("apr-1", ApprovalResponse::No);
    assert_eq!(manager.take_non_cli_pending_resolution("apr-1"), Some(ApprovalResponse::No));
}

#[test]
fn expired_pending_request_is_detected() {
    let request = PendingNonCliApprovalRequest {
        request_id: "apr-old".to_string(),
        tool_name: "shell".to_string(),
        arguments: serde_json::json!({}),
        message_id: None,
        call_id: None,
        requested_by: "alice".to_string(),
        requested_channel: "discord".to_string(),
        requested_reply_target: "chan".to_string(),
        reason: None,
        created_at: (Utc::now() - Duration::minutes(40)).to_rfc3339(),
        expires_at: (Utc::now() - Duration::minutes(1)).to_rfc3339(),
    };

    assert!(super::pending::is_pending_request_expired(&request));
}
