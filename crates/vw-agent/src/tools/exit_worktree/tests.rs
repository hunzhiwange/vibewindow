//! 退出 worktree 工具的基础错误路径测试。
//!
//! 确认没有会话绑定时不会尝试移除任何目录。

use super::ExitWorktreeTool;
use crate::app::agent::tools::Tool;
use crate::app::agent::tools::context::{ToolUseContext, scope_tool_use_context};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn exit_worktree_fails_without_binding() {
    let tool = ExitWorktreeTool::new();
    let context = Arc::new(ToolUseContext::new("exit-worktree", None));

    let result = scope_tool_use_context(context, tool.call(json!({}))).await;

    assert!(result.is_err());
}
