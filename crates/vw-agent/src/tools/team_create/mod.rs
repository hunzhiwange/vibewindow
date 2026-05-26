//! TeamCreate 工具实现。
//!
//! 该工具通过 IPC 数据库创建或覆盖团队对象，并在写入前执行工具操作权限检查。
//! 团队状态属于代理协作状态，不能在未授权上下文中静默修改。

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
    /// 团队成员 agent ID 列表。
    members: Vec<String>,
}

/// 创建或覆盖团队对象的工具。
pub struct TeamCreateTool {
    ipc_db: Arc<IpcDb>,
    security: Arc<SecurityPolicy>,
}

impl TeamCreateTool {
    /// 创建 TeamCreate 工具实例。
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
impl Tool for TeamCreateTool {
    fn name(&self) -> &str {
        "TeamCreate"
    }

    fn description(&self) -> &str {
        "创建或覆盖一个团队对象，并定义成员边界。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "id": {
                    "type": "string",
                    "description": "团队 ID。"
                },
                "members": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "团队成员 agent ID 列表。"
                }
            },
            "required": ["id", "members"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("TeamCreate")
            .with_aliases(vec!["team_create".to_string()])
            .with_read_only(false)
            .with_destructive(false)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "state_set") {
            // 权限失败作为工具失败返回，避免把策略拒绝伪装成运行时异常。
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }
        self.ipc_db.heartbeat();
        let data = self.ipc_db.create_team(&args.id, &args.members).map_err(anyhow::Error::msg)?;
        Ok(ToolResult { success: true, output: serde_json::to_string_pretty(&data)?, error: None })
    }
}
