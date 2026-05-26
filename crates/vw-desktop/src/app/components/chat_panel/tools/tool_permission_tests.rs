use super::tool_permission::{ToolPermissionState, pending_permission_badge_label, pending_permission_request_badge_label, tool_permission_state, tool_permission_summary};
use serde_json::json;

#[test]
fn permission_badge_labels_handle_empty_request_lists() {
    assert_eq!(pending_permission_badge_label(&[], None), "当前审批");
    assert_eq!(pending_permission_request_badge_label(&[], "missing"), "当前审批");
}

#[test]
fn permission_state_detects_approval_required() {
    let value = json!({"status":"error","error":"approval required before running command"});

    assert_eq!(tool_permission_state("bash", &value), Some(ToolPermissionState::ApprovalRequired));
    assert_eq!(tool_permission_summary("bash", &value), Some("需要权限批准"));
}
