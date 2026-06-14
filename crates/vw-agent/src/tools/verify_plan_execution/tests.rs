//! VerifyPlanExecution 工具测试。
//!
//! 覆盖缺少计划条件时的阻断结果，以及计划模式和 todo 准备好后的成功路径。

use super::VerifyPlanExecutionTool;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::tools::context::{ToolUseContext, scope_tool_use_context};
use crate::app::agent::tools::todo::TodoWriteTool;
use crate::app::agent::tools::traits::ToolResult;
use crate::app::agent::tools::{
    Tool, VERIFY_PLAN_EXECUTION_TOOL_ALIASES, VERIFY_PLAN_EXECUTION_TOOL_ID,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn verify_plan_execution_exposes_expected_metadata() {
    let tool = VerifyPlanExecutionTool::new();

    assert_eq!(tool.name(), VERIFY_PLAN_EXECUTION_TOOL_ID);
    assert!(tool.description().contains("计划模式"));
    assert_eq!(
        tool.parameters_schema(),
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {}
        })
    );

    let spec = tool.spec();
    assert_eq!(spec.id, VERIFY_PLAN_EXECUTION_TOOL_ID);
    assert_eq!(
        spec.aliases,
        VERIFY_PLAN_EXECUTION_TOOL_ALIASES
            .iter()
            .map(|alias| alias.to_string())
            .collect::<Vec<_>>()
    );
    assert!(spec.read_only);
    assert!(!spec.destructive);
    assert!(spec.concurrency_safe);
    assert!(!spec.requires_user_interaction);
    assert!(spec.strict);
}

#[tokio::test]
async fn verify_plan_execution_requires_active_tool_context() {
    let tool = VerifyPlanExecutionTool::new();
    let error = tool.call(json!({})).await.expect_err("missing context should fail");

    assert!(error.to_string().contains("missing active tool context"));
}

#[tokio::test]
async fn verify_plan_execution_requires_active_plan_mode_and_todos() {
    let tool = VerifyPlanExecutionTool::new();
    let context = Arc::new(ToolUseContext::new("verify-plan-blocked", None));

    let result = scope_tool_use_context(context, tool.call(json!({})))
        .await
        .expect("tool should return structured result");

    assert!(!result.is_success());
    assert_eq!(result.data["ready"].as_bool(), Some(false));
    assert_eq!(result.model_result.as_str(), Some("Plan execution is blocked"));
    assert_eq!(result.data["todo_count"].as_u64(), Some(0));
    assert_eq!(result.data["pending_count"].as_u64(), Some(0));
    assert_eq!(result.data["in_progress_count"].as_u64(), Some(0));
    let blockers = result.data["blockers"].as_array().expect("blockers should be present");
    assert!(blockers.len() >= 2);
    assert!(blockers.iter().any(|value| value.as_str() == Some("plan mode is not active")));
    assert!(
        blockers.iter().any(|value| {
            value.as_str() == Some("at least one todo is required before execution")
        })
    );
    let hint = result.render_hint.expect("render hint should be present");
    assert_eq!(hint.title.as_deref(), Some("VerifyPlanExecution"));
    assert_eq!(hint.kind.as_deref(), Some("plan_mode"));
    assert_eq!(hint.summary.as_deref(), Some("Blocked by 2 issue(s)"));
    assert_eq!(hint.metadata["ready"].as_bool(), Some(false));
    assert_eq!(result.content_blocks.len(), 1);
    assert_eq!(result.telemetry.expect("telemetry should be present").success, false);
}

#[tokio::test]
async fn verify_plan_execution_blocks_when_multiple_todos_are_in_progress() {
    let security = Arc::new(SecurityPolicy::default());
    let todo_tool = TodoWriteTool::new("verify-plan-multiple".to_string(), security);
    let verify_tool = VerifyPlanExecutionTool::new();
    let context = Arc::new(ToolUseContext::new("verify-plan-multiple", None));
    context.enter_plan_mode(
        Some("finish workflow control migration".to_string()),
        Some("keep focus on one task".to_string()),
    );

    scope_tool_use_context(
        context.clone(),
        todo_tool.call(json!({
            "todos": [
                {
                    "id": "todo-1",
                    "content": "Implement plan mode tools",
                    "status": "in_progress",
                    "priority": "high"
                },
                {
                    "id": "todo-2",
                    "content": "Write verification tests",
                    "status": "in_progress",
                    "priority": "medium"
                },
                {
                    "id": "todo-3",
                    "content": "Queue follow-up cleanup",
                    "status": "completed",
                    "priority": "low"
                }
            ],
            "merge": false
        })),
    )
    .await
    .expect("todo write should succeed");

    let result = scope_tool_use_context(context, verify_tool.call(json!({})))
        .await
        .expect("verify should succeed");

    assert!(!result.is_success());
    assert_eq!(result.data["todo_count"].as_u64(), Some(3));
    assert_eq!(result.data["pending_count"].as_u64(), Some(2));
    assert_eq!(result.data["in_progress_count"].as_u64(), Some(2));
    assert_eq!(result.data["goal"].as_str(), Some("finish workflow control migration"));
    assert_eq!(result.data["note"].as_str(), Some("keep focus on one task"));
    let blockers = result.data["blockers"].as_array().expect("blockers should be present");
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].as_str(), Some("only one todo may be in_progress"));
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
    assert_eq!(result.model_result.as_str(), Some("Plan execution verified"));
    assert_eq!(result.data["todo_count"].as_u64(), Some(1));
    assert_eq!(result.data["pending_count"].as_u64(), Some(1));
    assert_eq!(result.data["in_progress_count"].as_u64(), Some(0));
    assert_eq!(result.data["goal"].as_str(), Some("finish workflow control migration"));
    assert!(result.data["blockers"].as_array().is_some_and(|items| items.is_empty()));
    let hint = result.render_hint.expect("render hint should be present");
    assert_eq!(hint.summary.as_deref(), Some("Ready to execute 1 todo(s)"));
    assert_eq!(hint.metadata["ready"].as_bool(), Some(true));
    assert_eq!(result.telemetry.expect("telemetry should be present").success, true);
}

#[tokio::test]
async fn verify_plan_execution_execute_returns_success_result() {
    let tool = VerifyPlanExecutionTool::new();

    let result: ToolResult = tool.execute(json!({})).await.expect("execute should succeed");

    assert!(result.success);
    assert_eq!(result.output, "Verified plan execution");
    assert_eq!(result.error, None);
}
