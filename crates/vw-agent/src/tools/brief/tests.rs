//! Brief 工具结构化结果测试。
//!
//! 验证发送给用户的消息、附件元数据、渲染提示和模型侧摘要保持一致。

use super::BriefTool;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use serde_json::json;
use std::fs;
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
