use super::tool_permission::{
    ToolPermissionState, pending_permission_badge_label, pending_permission_request_badge_label,
    pending_permission_request_for_message, pending_permission_request_for_tool_call,
    pending_permission_targets_message, pending_permission_targets_tool_call,
    tool_permission_detail_text, tool_permission_error_text, tool_permission_state,
    tool_permission_summary, tool_permission_target_summary, tool_permission_title,
};
use serde_json::Map;
use serde_json::json;

fn request(
    id: &str,
    message_id: &str,
    call_id: &str,
) -> vw_gateway_client::PendingPermissionRequestDto {
    vw_gateway_client::PendingPermissionRequestDto {
        id: id.to_string(),
        session_id: "session-1".to_string(),
        permission: "tool".to_string(),
        patterns: vec![],
        metadata: Map::new(),
        always: vec![],
        tool: Some(vw_gateway_client::PendingPermissionToolDto {
            message_id: message_id.to_string(),
            call_id: call_id.to_string(),
        }),
    }
}

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

#[test]
fn pending_permission_request_matching_prefers_call_id_then_message() {
    let requests = vec![request("req-1", "msg-1", "call-1"), request("req-2", "msg-2", "call-2")];
    let raw = "tool bash\n{\"tool_call_id\":\"call-1\",\"input\":\"ls\"}";

    assert!(pending_permission_targets_message(Some(&requests[0]), Some("msg-1")));
    assert!(pending_permission_targets_tool_call(Some(&requests[0]), Some("msg-1"), raw));
    assert_eq!(
        pending_permission_request_for_tool_call(&requests, Some("msg-1"), raw)
            .map(|item| item.id.as_str()),
        Some("req-1")
    );
    assert_eq!(
        pending_permission_request_for_message(&requests, Some("msg-2"))
            .map(|item| item.id.as_str()),
        Some("req-2")
    );
}

#[test]
fn permission_state_detects_rejection_and_formats_details() {
    let value = json!({
        "status": "denied",
        "error": "permission denied",
        "permission_request": {
            "reason": "Need approval for write",
            "warning": "Modifies tracked files",
            "updated_input": {"path": "src/main.rs"}
        }
    });

    assert_eq!(tool_permission_state("file_write", &value), Some(ToolPermissionState::Rejected));
    assert_eq!(tool_permission_summary("file_write", &value), Some("权限已拒绝"));
    assert_eq!(
        tool_permission_target_summary("file_write", &value).as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(tool_permission_title("写入", ToolPermissionState::Rejected), "写入已拒绝");

    let detail = tool_permission_detail_text("file_write", &value).expect("detail");
    assert!(detail.contains("原因：Need approval for write"));
    assert!(detail.contains("提示：Modifies tracked files"));
    assert!(detail.contains("目标：src/main.rs"));
    assert!(
        tool_permission_error_text("file_write", &value)
            .expect("error text")
            .contains("permission denied")
    );
}
