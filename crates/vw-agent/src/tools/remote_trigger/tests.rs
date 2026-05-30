//! 远端触发工具测试。
//!
//! 当前运行时没有稳定远端触发后端，因此测试锁定显式不支持的返回语义。

use super::RemoteTriggerTool;
use crate::app::agent::tools::Tool;
use serde_json::json;

#[tokio::test]
async fn remote_trigger_returns_explicit_unsupported_result() {
    let tool = RemoteTriggerTool::new();
    let result = tool
        .execute(json!({
            "target": "prod-eu"
        }))
        .await
        .expect("remote trigger should return structured result");

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("RemoteTrigger is not supported in this runtime"));
}
