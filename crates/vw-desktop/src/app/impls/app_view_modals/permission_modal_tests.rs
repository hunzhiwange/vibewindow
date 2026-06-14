use super::*;
use serde_json::{Map, json};

fn request(id: &str) -> vw_gateway_client::PendingPermissionRequestDto {
    vw_gateway_client::PendingPermissionRequestDto {
        id: id.to_string(),
        session_id: "session-1".to_string(),
        permission: "shell".to_string(),
        patterns: Vec::new(),
        metadata: Map::new(),
        always: Vec::new(),
        tool: None,
    }
}

fn root() -> iced::Element<'static, Message> {
    iced::widget::container(iced::widget::text("root")).into()
}

#[test]
fn permission_modal_title_prefers_trimmed_metadata_title() {
    let mut req = request("perm-1");
    req.metadata.insert("title".to_string(), json!("  Approve shell  "));

    assert_eq!(permission_modal_title(&req), "Approve shell");
}

#[test]
fn permission_modal_title_falls_back_to_argument_summary_or_permission() {
    let mut req = request("perm-1");
    req.metadata.insert("arguments".to_string(), json!("echo hello"));

    assert!(permission_modal_title(&req).contains("echo hello"));

    req.metadata.clear();
    assert_eq!(permission_modal_title(&req), "需要批准 shell 操作");
}

#[test]
fn permission_metadata_text_trims_empty_values() {
    let mut req = request("perm-1");
    req.metadata.insert("requested_by".to_string(), json!("  codex  "));
    req.metadata.insert("requested_channel".to_string(), json!("   "));

    assert_eq!(permission_metadata_text(&req, "requested_by").as_deref(), Some("codex"));
    assert_eq!(permission_metadata_text(&req, "requested_channel"), None);
    assert_eq!(permission_metadata_text(&req, "missing"), None);
}

#[test]
fn permission_argument_summary_handles_strings_objects_and_nulls() {
    let mut req = request("perm-1");
    req.metadata.insert("arguments".to_string(), json!("  long raw argument  "));
    assert_eq!(permission_argument_summary(&req).as_deref(), Some("long raw argument"));

    req.metadata.insert("arguments".to_string(), json!({"cmd": "pwd"}));
    assert!(permission_argument_summary(&req).is_some());

    req.metadata.insert("arguments".to_string(), json!(null));
    assert_eq!(permission_argument_summary(&req), None);
}

#[test]
fn permission_arguments_preview_skips_empty_values_and_truncates_large_json() {
    let mut req = request("perm-1");
    req.metadata.insert("arguments".to_string(), json!({}));
    assert_eq!(permission_arguments_preview(&req), None);

    req.metadata.insert("arguments".to_string(), json!({"cmd": "pwd"}));
    assert!(permission_arguments_preview(&req).expect("preview").contains("\"cmd\""));

    req.metadata.insert("arguments".to_string(), json!({"text": "x".repeat(500)}));
    assert!(permission_arguments_preview(&req).expect("truncated preview").len() <= 323);
}

#[test]
fn permission_request_selector_label_uses_permission_and_summary() {
    let mut req = request("perm-1");
    req.metadata.insert("arguments".to_string(), json!("echo hello"));

    let label = permission_request_selector_label(&req);

    assert!(label.contains("echo hello"));
    assert!(label.contains("·"));
}

#[test]
fn permission_detail_helpers_build_rows_and_blocks() {
    let _line = permission_meta_line("权限", "shell");
    let _block = permission_detail_block("参数", "{\"cmd\":\"pwd\"}".to_string());
}

#[test]
fn with_permission_modal_returns_root_when_request_is_absent() {
    let app = App::new().0;

    let _element: iced::Element<'_, Message> = with_permission_modal(&app, root());
}

#[test]
fn with_permission_modal_builds_single_and_multi_request_dialogs() {
    let mut app = App::new().0;
    let mut first = request("perm-1");
    first.patterns = (0..10).map(|idx| format!("pattern-{idx}")).collect();
    first.always = vec!["pattern-0".to_string(), "pattern-1".to_string()];
    first.metadata.insert("reason".to_string(), json!("Need command access"));
    first.metadata.insert("arguments".to_string(), json!({"cmd": "pwd"}));
    first.metadata.insert("requested_by".to_string(), json!("agent"));
    first.metadata.insert("requested_channel".to_string(), json!("chat"));
    first.metadata.insert("expires_at".to_string(), json!("2030-01-01T00:00:00Z"));
    first.tool = Some(vw_gateway_client::PendingPermissionToolDto {
        message_id: "msg-1".to_string(),
        call_id: "call-1".to_string(),
    });
    let second = request("perm-2");
    app.permission_modal_request = Some(first.clone());
    app.permission_modal_requests = vec![first, second];

    let _element: iced::Element<'_, Message> = with_permission_modal(&app, root());
}
