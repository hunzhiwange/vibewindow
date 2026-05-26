//! VerifyPlanExecution 工具测试。
//!
//! 覆盖缺少计划条件时的阻断结果，以及计划模式和 todo 准备好后的成功路径。

use super::VerifyPlanExecutionTool;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use crate::app::agent::tools::context::{ToolUseContext, scope_tool_use_context};
use crate::app::agent::tools::todo::TodoWriteTool;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn verify_plan_execution_requires_active_plan_mode_and_todos() {
    let tool = VerifyPlanExecutionTool::new();
    let context = Arc::new(ToolUseContext::new("verify-plan-blocked", None));

    let result = scope_tool_use_context(context, tool.call(json!({})))
        .await
        .expect("tool should return structured result");

    assert!(!result.is_success());
    let blockers = result.data["blockers"].as_array().expect("blockers should be present");
    assert!(blockers.len() >= 2);
}

#[tokio::test]
async fn verify_plan_execution_succeeds_with_active_plan_and_todos() {
    let security = Arc::new(SecurityPolicy::default());
    let todo_tool = TodoWriteTool::new("verify-plan-ready".to_string(), security);
    let verify_tool = VerifyPlanExecutionTool::new();
    let context = Arc::new(ToolUseContext::new("verify-plan-ready", None));
    context.enter_plan_mode(Some("finish workflow control migration".to_string()), None);

    scope_tool_use_context(
        context.clone(),
        todo_tool.call(json!({
            "todos": [{
                "id": "todo-1",
                "content": "Implement plan mode tools",
                "status": "pending",
                "priority": "high"
            }],
            "merge": false
        })),
    )
    .await
    .expect("todo write should succeed");

    let result = scope_tool_use_context(context, verify_tool.call(json!({})))
        .await
        .expect("verify should succeed");

    assert!(result.is_success());
    assert_eq!(result.data["ready"].as_bool(), Some(true));
}
