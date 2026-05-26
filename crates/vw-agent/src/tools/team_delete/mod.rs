//! TeamDelete 工具实现。
//!
//! 该工具删除团队对象，并在变更 IPC 状态前执行权限检查。删除是破坏性操作，因此
//! tool spec 也显式标记为 destructive。

use super::agents_ipc::IpcDb;
use super::traits::{Tool, ToolResult, ToolSpec};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct Args {
    /// 团队 ID。
    id: String,
}

/// 删除团队对象的工具。
pub struct TeamDeleteTool {
    ipc_db: Arc<IpcDb>,
    security: Arc<SecurityPolicy>,
}

impl TeamDeleteTool {
    /// 创建 TeamDelete 工具实例。
    ///
    /// 参数：
    /// - `ipc_db`：团队状态存储。
    /// - `security`：工具操作安全策略。
    ///
    /// 返回值：新的工具实例。
    /// 错误处理：构造不返回错误；权限错误在执行时以工具结果返回。
    pub(crate) fn new(ipc_db: Arc<IpcDb>, security: Arc<SecurityPolicy>) -> Self {
        Self { ipc_db, security }
    }
}

#[cfg(test)]
mod tests;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for TeamDeleteTool {
    fn name(&self) -> &str {
        "TeamDelete"
    }

    fn description(&self) -> &str {
        "删除一个团队对象。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "id": {
                    "type": "string",
                    "description": "团队 ID。"
                }
            },
            "required": ["id"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("TeamDelete")
            .with_aliases(vec!["team_delete".to_string()])
            .with_read_only(false)
            .with_destructive(true)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "state_set") {
            // 删除团队会改变协作状态，权限拒绝必须显式返回给调用方。
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }
        self.ipc_db.heartbeat();
        let deleted = self.ipc_db.delete_team(&args.id).map_err(anyhow::Error::msg)?;
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "team_id": args.id,
                "deleted": deleted,
            }))?,
            error: None,
        })
    }
}
