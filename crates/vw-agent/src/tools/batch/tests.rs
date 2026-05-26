//! Batch 工具聚合执行测试。
//!
//! 使用一个最小 echo 工具验证 batch 能顺序分发子调用，同时拒绝递归调用自身，
//! 防止工具编排形成无限嵌套。

use super::super::*;
use async_trait::async_trait;
use serde_json::json;

struct EchoTool;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo input"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({"type": "object"})
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let value = args.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        Ok(ToolResult { success: true, output: value, error: None })
    }
}

fn build_tool() -> BatchTool {
    let tools: Arc<Vec<Arc<dyn Tool>>> = Arc::new(vec![Arc::new(EchoTool)]);
    BatchTool::new(tools)
}

#[tokio::test]
async fn batch_executes_calls() {
    let tool = build_tool();
    let result = tool
        .execute(json!({
            "tool_calls": [
                {"tool": "echo", "args": {"value": "a"}},
                {"tool": "echo", "parameters": {"value": "b"}}
            ]
        }))
        .await
        .unwrap();

    assert!(result.success);
    assert!(result.output.contains("全部 2 个工具均执行成功"));
    assert!(result.output.contains("a"));
    assert!(result.output.contains("b"));
}

#[tokio::test]
async fn batch_rejects_recursive_call() {
    let tool = build_tool();
    // batch 调用 batch 会形成不可控递归，必须在调度前拒绝。
    let result = tool
        .execute(json!({
            "tool_calls": [
                {"tool": "batch", "args": {}}
            ]
        }))
        .await
        .unwrap();

    assert!(result.success);
    assert!(result.output.contains("不允许在 batch 中调用"));
}
