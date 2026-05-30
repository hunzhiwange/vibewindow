//! Agent 间 IPC 消息发送工具。
//!
//! 本模块实现把一段文本消息发送给单个 agent、团队成员或显式广播目标的工具入口。
//! 发送前会通过安全策略检查操作权限，并复用 `IpcDb` 作为唯一的消息路由边界。

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
    #[serde(default)]
    to_agent: Option<String>,
    #[serde(default)]
    team_id: Option<String>,
    payload: String,
}

/// 向其他 agent 或团队发送 IPC 消息的工具。
///
/// 工具持有 IPC 数据库和安全策略；调用时只接受非空载荷，并要求提供明确路由。
pub struct SendMessageTool {
    ipc_db: Arc<IpcDb>,
    security: Arc<SecurityPolicy>,
}

impl SendMessageTool {
    /// 创建消息发送工具。
    ///
    /// # 参数
    ///
    /// - `ipc_db`: 当前 agent 使用的 IPC 数据库。
    /// - `security`: 当前会话安全策略。
    ///
    /// # 返回值
    ///
    /// 返回绑定 IPC 数据库和安全策略的工具实例。
    pub(crate) fn new(ipc_db: Arc<IpcDb>, security: Arc<SecurityPolicy>) -> Self {
        Self { ipc_db, security }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SendMessageTool {
    fn name(&self) -> &str {
        "SendMessage"
    }

    fn description(&self) -> &str {
        "向单个 agent、团队成员或全部在线 agent 发送 IPC 消息。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "to_agent": {
                    "type": "string",
                    "description": "目标 agent ID，或 '*' 表示广播。"
                },
                "team_id": {
                    "type": "string",
                    "description": "目标团队 ID。提供后会向团队成员逐个发送。"
                },
                "payload": {
                    "type": "string",
                    "description": "消息内容。"
                }
            },
            "required": ["payload"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("SendMessage")
            .with_aliases(vec!["send_message".to_string()])
            .with_read_only(false)
            .with_destructive(false)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)?;
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "agents_send")
        {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }
        // 发送消息也代表当前 agent 仍在线，先刷新心跳能让团队路由按最新活跃状态工作。
        self.ipc_db.heartbeat();

        let payload = args.payload.trim();
        if payload.is_empty() {
            anyhow::bail!("payload must not be empty");
        }

        let targets = if let Some(team_id) = args.team_id.as_deref() {
            self.ipc_db.read_team_members(team_id).map_err(anyhow::Error::msg)?
        } else {
            let target = args
                .to_agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!("either 'to_agent' or 'team_id' is required"))?;
            vec![target.to_string()]
        };

        for target in &targets {
            self.ipc_db.send_message(target, payload).map_err(anyhow::Error::msg)?;
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "targets": targets,
                "count": targets.len(),
            }))?,
            error: None,
        })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
