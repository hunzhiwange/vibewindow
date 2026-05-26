//! 统一的 agent 调用工具。
//!
//! `AgentTool` 兼容同步委派和后台子 agent 会话管理。它集中处理 action 推断、
//! 参数别名、状态查询和终止请求的安全检查，让上层只暴露一个稳定工具入口。

use super::delegate::DelegateTool;
use super::subagent_registry::{SubAgentRegistry, SubAgentSessionInfo};
use super::subagent_spawn::SubAgentSpawnTool;
use super::traits::{Tool, ToolResult, ToolSpec};
use crate::app::agent::config::DelegateAgentConfig;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

/// 统一 agent 工具的运行时状态。
pub struct AgentTool {
    agents: Arc<HashMap<String, DelegateAgentConfig>>,
    delegate_tool: Arc<DelegateTool>,
    background_tool: Arc<SubAgentSpawnTool>,
    registry: Arc<SubAgentRegistry>,
    security: Arc<SecurityPolicy>,
}

enum AgentAction {
    Launch,
    List,
    Get,
    Stop,
}

impl AgentTool {
    /// 创建新的 `AgentTool`。
    ///
    /// # 参数
    ///
    /// - `agents`: 可调用 agent 配置表。
    /// - `delegate_tool`: 同步委派工具。
    /// - `background_tool`: 后台子 agent 启动工具。
    /// - `registry`: 后台会话注册表。
    /// - `security`: 安全策略，用于终止等动作型操作。
    pub fn new(
        agents: HashMap<String, DelegateAgentConfig>,
        delegate_tool: Arc<DelegateTool>,
        background_tool: Arc<SubAgentSpawnTool>,
        registry: Arc<SubAgentRegistry>,
        security: Arc<SecurityPolicy>,
    ) -> Self {
        Self { agents: Arc::new(agents), delegate_tool, background_tool, registry, security }
    }

    fn available_agents_description(&self) -> String {
        let mut names = self.agents.keys().cloned().collect::<Vec<_>>();
        names.sort();
        if names.is_empty() { "（未配置）".to_string() } else { names.join(", ") }
    }

    fn resolve_action(&self, args: &Value) -> anyhow::Result<AgentAction> {
        let explicit = args
            .get("action")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase());

        let inferred = match explicit.as_deref() {
            Some("launch") | Some("run") | Some("spawn") => AgentAction::Launch,
            Some("list") => AgentAction::List,
            Some("get") | Some("status") => AgentAction::Get,
            Some("stop") | Some("kill") => AgentAction::Stop,
            Some("message") => {
                anyhow::bail!("AgentTool action 'message' is not implemented yet in vw-agent")
            }
            Some(other) => anyhow::bail!(
                "Unknown AgentTool action '{other}'. Supported actions: launch, list, get, stop"
            ),
            None => {
                // 兼容旧调用：只有 status 时视为列表查询，有 session_id 时视为
                // 查询单个会话，其余情况默认启动 agent。
                if args.get("status").is_some()
                    && args.get("agent").is_none()
                    && args.get("subagent_type").is_none()
                    && args.get("session_id").is_none()
                {
                    AgentAction::List
                } else if args.get("session_id").is_some() {
                    AgentAction::Get
                } else {
                    AgentAction::Launch
                }
            }
        };
        Ok(inferred)
    }

    fn agent_name_from_args<'a>(&self, args: &'a Value) -> Option<&'a str> {
        args.get("agent")
            .and_then(Value::as_str)
            .or_else(|| args.get("subagent_type").and_then(Value::as_str))
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    fn prompt_from_args<'a>(&self, args: &'a Value) -> Option<&'a str> {
        args.get("prompt")
            .and_then(Value::as_str)
            .or_else(|| args.get("task").and_then(Value::as_str))
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    fn session_id_from_args<'a>(&self, args: &'a Value) -> anyhow::Result<&'a str> {
        args.get("session_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'session_id' parameter"))
    }

    fn run_in_background(&self, args: &Value) -> bool {
        if let Some(value) = args.get("run_in_background").and_then(Value::as_bool) {
            return value;
        }
        args.get("task").and_then(Value::as_str).is_some()
    }

    fn normalized_status_filter<'a>(&self, args: &'a Value) -> anyhow::Result<Option<&'a str>> {
        let filter = args.get("status").and_then(Value::as_str).map(str::trim);
        if let Some(filter) = filter {
            if filter.is_empty() || filter == "all" {
                return Ok(None);
            }
            if matches!(filter, "running" | "completed" | "failed" | "killed") {
                return Ok(Some(filter));
            }
            anyhow::bail!(
                "Invalid status filter '{filter}'. Must be one of: running, completed, failed, killed, all"
            );
        }
        Ok(None)
    }

    async fn handle_launch(&self, args: &Value) -> anyhow::Result<ToolResult> {
        let agent = self
            .agent_name_from_args(args)
            .ok_or_else(|| anyhow::anyhow!("Missing 'agent' parameter"))?;
        let prompt = self
            .prompt_from_args(args)
            .ok_or_else(|| anyhow::anyhow!("Missing 'prompt' parameter"))?;
        let context = args.get("context").and_then(Value::as_str).map(str::trim).unwrap_or("");

        if self.run_in_background(args) {
            // 后台路径通过 SubAgentSpawnTool 复用注册表和生命周期管理，避免
            // AgentTool 自己复制一套会话调度逻辑。
            let result = self
                .background_tool
                .execute(json!({
                    "agent": agent,
                    "task": prompt,
                    "context": context,
                    "_via_agent_tool": true,
                }))
                .await?;
            if !result.success {
                return Ok(result);
            }

            let mut output = serde_json::from_str::<Value>(&result.output)?;
            if let Some(object) = output.as_object_mut() {
                object.insert(
                    "message".to_string(),
                    Value::String(
                        "AgentTool launched in background. Use AgentTool with action='get' or action='list' to inspect progress."
                            .to_string(),
                    ),
                );
            }
            return Ok(ToolResult {
                success: true,
                output: serde_json::to_string(&output)?,
                error: None,
            });
        }

        self.delegate_tool
            .execute(json!({
                "agent": agent,
                "prompt": prompt,
                "context": context,
                "_via_agent_tool": true,
            }))
            .await
    }

    fn handle_list(&self, args: &Value) -> anyhow::Result<ToolResult> {
        let sessions: Vec<SubAgentSessionInfo> =
            self.registry.list(self.normalized_status_filter(args)?);
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&sessions)?,
            error: None,
        })
    }

    fn handle_get(&self, args: &Value) -> anyhow::Result<ToolResult> {
        let session_id = self.session_id_from_args(args)?;
        let Some(snap) = self.registry.get_status(session_id) else {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown session '{session_id}'")),
            });
        };

        let duration_ms = snap.completed_at.map(|end| {
            u64::try_from((end - snap.started_at).num_milliseconds()).unwrap_or_default()
        });

        let mut output = json!({
            "session_id": session_id,
            "agent": snap.agent_name,
            "title": snap.title,
            "task": snap.task,
            "metadata": snap.metadata,
            "status": snap.status.as_str(),
            "started_at": snap.started_at.to_rfc3339(),
            "updated_at": snap.updated_at.to_rfc3339(),
            "duration_ms": duration_ms,
        });

        if let Some(end) = snap.completed_at {
            output["completed_at"] = json!(end.to_rfc3339());
        }

        if let Some(ref result) = snap.result {
            output["result"] = json!({
                "success": result.success,
                "output": truncate_output(&result.output, 500),
                "error": result.error,
            });
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&output)?,
            error: None,
        })
    }

    fn handle_stop(&self, args: &Value) -> anyhow::Result<ToolResult> {
        let session_id = self.session_id_from_args(args)?;
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "AgentTool") {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }

        if !self.registry.exists(session_id) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown session '{session_id}'")),
            });
        }

        if !self.registry.kill(session_id) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Session '{session_id}' is not running (may have already completed or been killed)"
                )),
            });
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string(&json!({
                "session_id": session_id,
                "status": "killed",
                "message": "Agent session cancelled successfully",
                "updated_at": Utc::now().to_rfc3339(),
            }))?,
            error: None,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for AgentTool {
    fn name(&self) -> &str {
        "AgentTool"
    }

    fn description(&self) -> &str {
        "统一的 AgentTool。可同步运行专门 agent，也可在后台启动 agent 会话，并支持列出、查看、终止后台 agent。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["launch", "run", "spawn", "list", "get", "stop", "kill"],
                    "description": "AgentTool 操作类型。省略时会根据参数自动推断。"
                },
                "agent": {
                    "type": "string",
                    "description": format!("要调用的 agent 名称。可用：{}", self.available_agents_description())
                },
                "subagent_type": {
                    "type": "string",
                    "description": "Claude Code 兼容别名，等价于 agent。"
                },
                "prompt": {
                    "type": "string",
                    "description": "发送给 agent 的任务描述。"
                },
                "task": {
                    "type": "string",
                    "description": "后台启动时的兼容字段，等价于 prompt。"
                },
                "context": {
                    "type": "string",
                    "description": "可选的补充上下文。"
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "为 true 时以后台会话方式启动 agent。"
                },
                "session_id": {
                    "type": "string",
                    "description": "后台 agent 会话 ID，用于 get / stop。"
                },
                "status": {
                    "type": "string",
                    "enum": ["running", "completed", "failed", "killed", "all"],
                    "description": "list 时按会话状态筛选。"
                }
            }
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("AgentTool")
            .with_aliases(vec!["Agent".to_string(), "agent_tool".to_string(), "agent".to_string()])
            .with_read_only(false)
            .with_destructive(false)
            .with_concurrency_safe(false)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        match self.resolve_action(&args)? {
            AgentAction::Launch => self.handle_launch(&args).await,
            AgentAction::List => self.handle_list(&args),
            AgentAction::Get => self.handle_get(&args),
            AgentAction::Stop => self.handle_stop(&args),
        }
    }
}

/// 按字符数截断 agent 输出。
///
/// 使用字符边界而非字节下标，避免截断 UTF-8 文本时产生无效字符串。
fn truncate_output(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let trunc_idx = text.char_indices().nth(max_chars).map(|(idx, _)| idx).unwrap_or(text.len());
    format!("{}... (truncated)", &text[..trunc_idx])
}
#[cfg(test)]
mod tests;
