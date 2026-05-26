//! ToolSearch 工具测试。
//!
//! 验证搜索结果包含命中原因，保证工具发现结果对模型可解释。

use super::ToolSearchTool;
use crate::app::agent::tools::Tool;
use crate::app::agent::tools::context::{ToolUseContext, scope_tool_use_context};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn tool_search_returns_match_reason() {
    let tool = ToolSearchTool::new();
    let context = Arc::new(ToolUseContext::new("tool-search", None));

    let result = scope_tool_use_context(
        context,
        tool.call(json!({
            "query": "read"
        })),
    )
    .await
    .expect("tool search should succeed");

    assert!(result.is_success());
    let items = result.data["items"].as_array().expect("items should be present");
    assert!(!items.is_empty());
    assert!(items[0].get("reason").is_some());
}
