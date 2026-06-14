use super::text_view::{copy_content_hash, tool_text_view, workflow_preview_message};
use crate::app::{App, Message};

fn app() -> App {
    App::new().0
}

#[test]
fn text_view_test_module_is_linked() {
    assert_eq!("text_view", "text_view");
}

#[test]
fn copy_hash_is_stable_and_content_sensitive() {
    assert_eq!(copy_content_hash("same"), copy_content_hash("same"));
    assert_ne!(copy_content_hash("same"), copy_content_hash("different"));
}

#[test]
fn workflow_preview_message_requires_workflow_metadata() {
    let value = serde_json::json!({
        "metadata": {
            "workflow_yaml": "nodes: []",
            "node_id": "node-1"
        }
    });

    assert!(matches!(
        workflow_preview_message("workflow_node", &value),
        Some(Message::WorkflowTool(_))
    ));

    assert!(workflow_preview_message("bash", &value).is_none());
    assert!(
        workflow_preview_message("workflow_node", &serde_json::json!({"metadata": {}})).is_none()
    );
}

#[test]
fn text_view_filters_special_tools_and_bad_blocks() {
    let app = app();

    assert!(tool_text_view(&app, 0, 0, "tool bash\n{}").is_none());
    assert!(tool_text_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(tool_text_view(&app, 0, 0, "tool custom\nnot-json").is_none());
    assert!(tool_text_view(&app, 0, 0, "custom\n{}").is_none());
}

#[test]
fn text_view_builds_success_error_permission_and_summary_only_cards() {
    let mut app = app();
    app.chat_tool_hovered_idx = Some((1_u64 << 32) | 2);

    let success = r#"tool grep
{"status":"completed","input":"{\"pattern\":\"App\"}","output":"match line"}"#;
    let error = r#"tool grep
{"status":"error","input":"{}","error":"failed hard"}"#;
    let denied = r#"tool file_write
{"status":"denied","input":"{\"path\":\"src/lib.rs\"}","error":"permission denied"}"#;
    let summary_only = r#"tool question
{"status":"completed","input":"What next?","summary":"Need answer"}"#;
    let empty = r#"tool question
{"status":"completed","input":"","output":""}"#;

    assert!(tool_text_view(&app, 1, 2, success).is_some());
    assert!(tool_text_view(&app, 1, 3, error).is_some());
    assert!(tool_text_view(&app, 1, 4, denied).is_some());
    assert!(tool_text_view(&app, 1, 5, summary_only).is_some());
    assert!(tool_text_view(&app, 1, 6, empty).is_none());
}
