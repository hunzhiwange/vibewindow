//! LSP 工具输入校验和缺失 server 路径测试。
//!
//! 当前测试避免依赖本机安装语言服务器，只覆盖 schema、必需位置参数和不支持扩展名。

use super::super::*;
use crate::app::agent::security::SecurityPolicy;
use serde_json::json;
use std::sync::Arc;

fn test_tool(workspace: &std::path::Path) -> LspTool {
    let mut security = SecurityPolicy::default();
    security.workspace_dir = workspace.to_path_buf();
    LspTool::new(Arc::new(security))
}

#[test]
fn lsp_schema_is_object() {
    let schema = LspTool::schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["operation"].is_object());
    assert!(schema["properties"]["filePath"].is_object());
    assert!(schema["properties"]["line"].is_object());
    assert!(schema["properties"]["character"].is_object());
    assert!(schema["properties"]["uri"].is_object());
}

#[tokio::test]
async fn lsp_rejects_missing_required_position() {
    let workspace = tempfile::tempdir().unwrap();
    let tool = test_tool(workspace.path());

    let error = tool
        .validate_input(json!({
            "operation": "hover",
            "filePath": "demo.rs"
        }))
        .unwrap_err();

    assert!(error.to_string().contains("Missing line or character"));
}

#[tokio::test]
async fn lsp_returns_missing_server_error_for_unsupported_extension() {
    let workspace = tempfile::tempdir().unwrap();
    let file_path = workspace.path().join("demo.txt");
    tokio::fs::write(&file_path, "hello").await.unwrap();

    let tool = test_tool(workspace.path());
    let result = tool
        .call(json!({
            "operation": "hover",
            "filePath": file_path.to_string_lossy(),
            "line": 1,
            "character": 1
        }))
        .await
        .unwrap();

    assert!(!result.is_success());
    assert!(result.model_text().contains("No LSP server available for file type: .txt"));
}
