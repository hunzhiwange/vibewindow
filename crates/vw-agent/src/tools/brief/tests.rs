//! Brief 工具结构化结果测试。
//!
//! 验证发送给用户的消息、附件元数据、渲染提示和模型侧摘要保持一致。

use super::{Args, BriefStatus, BriefTool};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn brief_returns_user_message_payload() {
    let tmp = TempDir::new().expect("tempdir should create");
    let attachment_path = tmp.path().join("chart.png");
    fs::write(&attachment_path, b"png").expect("attachment should write");

    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = BriefTool::new(Arc::new(policy));

    let result = tool
        .call(json!({
            "message": "## 已完成\n请查看附件。",
            "attachments": ["chart.png"],
            "status": "proactive"
        }))
        .await
        .expect("brief should succeed");

    assert!(result.is_success());
    assert_eq!(
        result.model_result.as_str(),
        Some("Message delivered to user. (1 attachment included)")
    );

    let data = result.data.as_object().expect("brief data should be an object");
    assert_eq!(
        data.get("message").and_then(|value| value.as_str()),
        Some("## 已完成\n请查看附件。")
    );
    assert_eq!(data.get("status").and_then(|value| value.as_str()), Some("proactive"));
    assert!(data.get("sentAt").and_then(|value| value.as_str()).is_some());

    let attachments = data
        .get("attachments")
        .and_then(|value| value.as_array())
        .expect("attachments should be present");
    assert_eq!(attachments.len(), 1);
    assert_eq!(
        attachments[0].get("path").and_then(|value| value.as_str()),
        Some(attachment_path.to_string_lossy().as_ref())
    );
    assert_eq!(attachments[0].get("isImage").and_then(|value| value.as_bool()), Some(true));

    let render_hint = result.render_hint.expect("brief should include render hint");
    assert_eq!(render_hint.kind.as_deref(), Some("brief"));
    assert_eq!(render_hint.metadata["canonical_tool_id"], json!("brief"));
}

#[test]
fn status_summary_and_model_text_are_stable() {
    assert_eq!(BriefStatus::Normal.as_str(), "normal");
    assert_eq!(BriefStatus::Proactive.as_str(), "proactive");
    assert_eq!(BriefTool::summary_text("  hello\nworld  ", 0), "hello world");
    assert_eq!(BriefTool::summary_text("", 0), "用户消息");
    assert_eq!(BriefTool::summary_text("", 1), "1 个附件");
    assert_eq!(BriefTool::summary_text("", 3), "3 个附件");
    assert_eq!(BriefTool::model_delivery_text(0), "Message delivered to user.");
    assert_eq!(
        BriefTool::model_delivery_text(2),
        "Message delivered to user. (2 attachments included)"
    );

    let long = "a".repeat(97);
    assert_eq!(BriefTool::summary_text(&long, 0), format!("{}...", "a".repeat(96)));
}

#[test]
fn image_detection_is_case_insensitive() {
    assert!(BriefTool::is_image(Path::new("image.PNG")));
    assert!(BriefTool::is_image(Path::new("vector.svg")));
    assert!(!BriefTool::is_image(Path::new("notes.txt")));
    assert!(!BriefTool::is_image(Path::new("no-extension")));
}

#[tokio::test]
async fn brief_execute_returns_model_delivery_text() {
    let tmp = TempDir::new().expect("tempdir should create");
    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = BriefTool::new(Arc::new(policy));

    let result = tool
        .execute(json!({
            "message": "visible to user",
            "attachments": [],
            "status": "normal"
        }))
        .await
        .expect("brief execute should succeed");

    assert!(result.success);
    assert_eq!(result.output, "Message delivered to user.");
}

#[tokio::test]
async fn brief_rejects_empty_message_without_attachments() {
    let tmp = TempDir::new().expect("tempdir should create");
    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = BriefTool::new(Arc::new(policy));

    let err = tool
        .call(json!({"message": "   ", "attachments": []}))
        .await
        .expect_err("empty brief should fail");

    assert!(err.to_string().contains("either message or attachments is required"));
}

#[tokio::test]
async fn attachments_must_exist_and_be_regular_files() {
    let tmp = TempDir::new().expect("tempdir should create");
    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = BriefTool::new(Arc::new(policy));

    let missing = tool
        .resolve_attachments(&["missing.txt".to_string()])
        .await
        .expect_err("missing attachment should fail");
    assert!(missing.to_string().contains("failed to read attachment metadata"));

    fs::create_dir(tmp.path().join("folder")).expect("folder should create");
    let dir = tool
        .resolve_attachments(&["folder".to_string()])
        .await
        .expect_err("directory attachment should fail");
    assert!(dir.to_string().contains("regular file"));
}

#[tokio::test]
async fn attachments_can_be_only_payload() {
    let tmp = TempDir::new().expect("tempdir should create");
    let attachment_path = tmp.path().join("report.txt");
    fs::write(&attachment_path, b"hello").expect("attachment should write");

    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = BriefTool::new(Arc::new(policy));

    let result = tool
        .build_result(Args {
            message: String::new(),
            attachments: vec!["report.txt".to_string()],
            status: BriefStatus::Normal,
        })
        .await
        .expect("attachment-only brief should succeed");

    assert_eq!(result.render_hint.unwrap().summary.as_deref(), Some("1 个附件"));
    assert_eq!(result.data["attachments"][0]["size"], 5);
}
