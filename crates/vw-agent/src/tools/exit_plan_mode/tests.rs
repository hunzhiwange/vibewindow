//! 退出计划模式工具的行为测试。
//!
//! 覆盖缺少待办时的阻断，以及计划准备就绪后状态能够关闭。

use super::ExitPlanModeTool;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::Tool;
use crate::app::agent::tools::context::{ToolUseContext, scope_tool_use_context};
use crate::app::agent::tools::todo::TodoWriteTool;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn exit_plan_mode_blocks_without_todos() {
    let tool = ExitPlanModeTool::new();
    let context = Arc::new(ToolUseContext::new("exit-plan-blocked", None));
    context.enter_plan_mode(Some("ship workflow tools".to_string()), None);

    let result = scope_tool_use_context(context.clone(), tool.call(json!({})))
        .await
        .expect("tool should return structured result");

    assert!(!result.is_success());
    assert!(context.plan_mode_state().active);
}

#[tokio::test]
async fn exit_plan_mode_succeeds_after_plan_is_ready() {
    let security = Arc::new(SecurityPolicy::default());
    let todo_tool = TodoWriteTool::new("exit-plan-ready".to_string(), security);
    let exit_tool = ExitPlanModeTool::new();
    let context = Arc::new(ToolUseContext::new("exit-plan-ready", None));
    context.enter_plan_mode(Some("ship workflow tools".to_string()), None);

    scope_tool_use_context(
        context.clone(),
        todo_tool.call(json!({
            "todos": [{
                "id": "todo-1",
                "content": "Review plan",
                "status": "pending",
                "priority": "high"
            }],
            "merge": false
        })),
    )
    .await
    .expect("todo write should succeed");

    let result = scope_tool_use_context(context.clone(), exit_tool.call(json!({})))
        .await
        .expect("exit tool should succeed");

    assert!(result.is_success());
    assert!(!context.plan_mode_state().active);
}
