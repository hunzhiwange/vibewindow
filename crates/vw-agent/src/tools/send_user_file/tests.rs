//! 用户文件附件工具测试。
//!
//! 测试确保工作区相对路径会解析为真实文件，并返回模型可引用的附件标记。

use super::SendUserFileTool;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn send_user_file_returns_attachment_marker() {
    let tmp = TempDir::new().expect("tempdir should create");
    let file_path = tmp.path().join("report.txt");
    fs::write(&file_path, "hello").expect("test file should write");

    let mut policy = SecurityPolicy::default();
    policy.workspace_dir = tmp.path().to_path_buf();
    let tool = SendUserFileTool::new(Arc::new(policy));
    let result = tool
        .call(json!({
            "path": "report.txt"
        }))
        .await
        .expect("send user file should succeed");

    assert!(result.is_success());
    let expected = format!("[DOCUMENT:{}]", file_path.display());
    assert_eq!(result.model_result.as_str(), Some(expected.as_str()));
}
