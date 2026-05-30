//! 子代理生成工具
//!
//! 在后台异步启动委托代理，立即返回会话 ID。
//! 子代理在后台运行，结果存储在共享的 [`SubAgentRegistry`] 中。
//!
//! ## 功能说明
//!
//! 本模块实现了 `subagent_spawn` 工具，通过 `tokio::spawn` 异步启动委托代理，
//! 并立即返回会话 ID。该工具支持：
//!
//! - 后台异步执行子代理任务
//! - 两种执行模式：简单模式和代理模式（agentic）
//! - 并发子代理数量限制
//! - 超时控制和会话管理
//!
//! ## 使用场景
//!
//! 当主代理需要委托子任务给专用代理时使用此工具，例如：
//! - 代码审查
//! - 文档生成
//! - 数据分析
//!
//! 相关工具变更手册请参见 `AGENTS.md` §7.3。

use super::subagent_registry::{SubAgentRegistry, SubAgentSession, SubAgentStatus};
use super::traits::{Tool, ToolResult};
use crate::app::agent::approval::ApprovalManager;
use crate::app::agent::config::DelegateAgentConfig;
use crate::app::agent::hooks::HookRunner;
use crate::app::agent::observability::traits::{Observer, ObserverEvent, ObserverMetric};
use crate::app::agent::providers::{ChatMessage, Provider};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 后台子代理提供者调用的默认超时时间（秒）。
///
/// 当子代理执行时间超过此限制时，将被强制终止并返回超时错误。
/// 默认值为 300 秒（5 分钟），适用于大多数复杂任务的执行时间。
const SPAWN_TIMEOUT_SECS: u64 = 300;

/// 最大并发子代理数量。
///
/// 限制同时运行的后台子代理数量，防止资源耗尽。
/// 默认值为 10，可根据系统资源进行调整。
const MAX_CONCURRENT_SUBAGENTS: usize = 10;

/// 子代理生成工具
///
/// 在后台启动委托代理，立即返回会话 ID。子代理异步运行，
/// 并将结果存储在共享的 [`SubAgentRegistry`] 中。
///
/// ## 字段说明
///
/// - `agents`: 委托代理配置映射，键为代理名称
/// - `security`: 安全策略，用于权限检查
/// - `fallback_credential`: 备用 API 凭证，当代理配置中未指定时使用
/// - `provider_runtime_options`: 提供者运行时选项
/// - `registry`: 子代理会话注册表，用于跟踪所有子代理状态
/// - `parent_tools`: 父代理的工具集，可被子代理继承使用
/// - `multimodal_config`: 多模态配置，支持文本、图像等多种输入类型
pub struct SubAgentSpawnTool {
    agents: Arc<HashMap<String, DelegateAgentConfig>>,
    security: Arc<SecurityPolicy>,
    fallback_credential: Option<String>,
    provider_runtime_options: crate::app::agent::providers::ProviderRuntimeOptions,
    registry: Arc<SubAgentRegistry>,
    parent_tools: Arc<Vec<Arc<dyn Tool>>>,
    multimodal_config: crate::app::agent::config::MultimodalConfig,
    workspace_identity_context: String,
    skill_contexts: HashMap<String, String>,
}

impl SubAgentSpawnTool {
    /// 创建新的子代理生成工具实例。
    ///
    /// # 参数
    ///
    /// - `agents`: 委托代理配置映射表，键为代理名称
    /// - `fallback_credential`: 备用 API 凭证，当代理配置未指定时使用
    /// - `security`: 安全策略实例，用于工具操作权限验证
    /// - `provider_runtime_options`: 提供者运行时配置选项
    /// - `registry`: 子代理会话注册表，用于状态追踪
    /// - `parent_tools`: 父代理工具集，可被代理模式的子代理使用
    /// - `multimodal_config`: 多模态配置，支持多种输入类型
    ///
    /// # 返回值
    ///
    /// 返回配置完成的 `SubAgentSpawnTool` 实例。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let tool = SubAgentSpawnTool::new(
    ///     agents_config,
    ///     Some("api_key".to_string()),
    ///     security_policy,
    ///     runtime_options,
    ///     registry,
    ///     tools,
    ///     multimodal_config,
    /// );
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agents: HashMap<String, DelegateAgentConfig>,
        fallback_credential: Option<String>,
        security: Arc<SecurityPolicy>,
        provider_runtime_options: crate::app::agent::providers::ProviderRuntimeOptions,
        registry: Arc<SubAgentRegistry>,
        parent_tools: Arc<Vec<Arc<dyn Tool>>>,
        multimodal_config: crate::app::agent::config::MultimodalConfig,
    ) -> Self {
        Self {
            agents: Arc::new(agents),
            security,
            fallback_credential,
            provider_runtime_options,
            registry,
            parent_tools,
            multimodal_config,
            workspace_identity_context: String::new(),
            skill_contexts: HashMap::new(),
        }
    }

    pub fn with_workspace_identity_context(mut self, workspace_identity_context: String) -> Self {
        self.workspace_identity_context = workspace_identity_context;
        self
    }

    pub fn with_skill_contexts(mut self, skill_contexts: HashMap<String, String>) -> Self {
        self.skill_contexts = skill_contexts;
        self
    }

    fn merged_system_prompt(
        &self,
        agent_name: &str,
        agent_system_prompt: Option<&str>,
    ) -> Option<String> {
        let mut sections = Vec::new();
        if !self.workspace_identity_context.trim().is_empty() {
            sections.push(self.workspace_identity_context.clone());
        }
        if let Some(skill_context) =
            self.skill_contexts.get(agent_name).filter(|context| !context.trim().is_empty())
        {
            sections.push(skill_context.clone());
        }
        if let Some(agent_prompt) =
            agent_system_prompt.map(str::trim).filter(|prompt| !prompt.is_empty())
        {
            sections.push(agent_prompt.to_string());
        }

        (!sections.is_empty()).then(|| sections.join("\n\n"))
    }
}

/// `Tool` trait 实现
///
/// 为 `SubAgentSpawnTool` 实现 `Tool` trait，使其可作为工具被代理调用。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SubAgentSpawnTool {
    /// 返回工具名称。
    ///
    /// # 返回值
    ///
    /// 固定返回 `"subagent_spawn"`。
    fn name(&self) -> &str {
        "subagent_spawn"
    }

    /// 返回工具描述。
    ///
    /// # 返回值
    ///
    /// 返回中文描述，说明工具的功能和使用方式。
    fn description(&self) -> &str {
        "在后台启动委托智能体。立即返回 session_id。公开入口由 AgentTool 负责后续 list/get/stop。"
    }

    /// 返回工具参数的 JSON Schema。
    ///
    /// # 返回值
    ///
    /// 返回包含以下字段的 JSON Schema：
    /// - `agent`: 必填，要启动的代理名称
    /// - `task`: 必填，发送给子代理的任务/提示
    /// - `context`: 可选，前置上下文信息
    fn parameters_schema(&self) -> serde_json::Value {
        let agent_names: Vec<&str> = self.agents.keys().map(|s: &String| s.as_str()).collect();
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "agent": {
                    "type": "string",
                    "minLength": 1,
                    "description": format!(
                        "要启动的智能体名称。可用：{}",
                        if agent_names.is_empty() {
                            "（未配置）".to_string()
                        } else {
                            agent_names.join(", ")
                        }
                    )
                },
                "task": {
                    "type": "string",
                    "minLength": 1,
                    "description": "发送给子智能体的任务/提示"
                },
                "context": {
                    "type": "string",
                    "description": "可选的前置上下文（例如相关代码、先前发现）"
                }
            },
            "required": ["agent", "task"]
        })
    }

    /// 执行子代理生成工具。
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象，必须包含 `agent` 和 `task` 字段
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，其中：
    /// - 成功时：`output` 包含会话 ID、代理名称和状态信息
    /// - 失败时：`error` 包含错误描述
    ///
    /// # 执行流程
    ///
    /// 1. 解析并验证参数（agent、task、context）
    /// 2. 检查安全策略权限
    /// 3. 查找代理配置
    /// 4. 创建提供者实例
    /// 5. 构建完整提示（合并上下文和任务）
    /// 6. 原子性检查并发限制并注册会话
    /// 7. 异步启动后台任务
    /// 8. 立即返回会话 ID
    ///
    /// # 错误情况
    ///
    /// - 缺少必填参数
    /// - 参数值为空
    /// - 安全策略拒绝操作
    /// - 代理名称不存在
    /// - 提供者创建失败
    /// - 超过最大并发限制
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let called_via_agent_tool =
            args.get("_via_agent_tool").and_then(|v| v.as_bool()).unwrap_or(false);
        let agent_name = args
            .get("agent")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .ok_or_else(|| anyhow::anyhow!("Missing 'agent' parameter"))?;

        if agent_name.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'agent' parameter must not be empty".into()),
            });
        }

        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .ok_or_else(|| anyhow::anyhow!("Missing 'task' parameter"))?;

        if task.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'task' parameter must not be empty".into()),
            });
        }

        let context = args.get("context").and_then(|v| v.as_str()).map(str::trim).unwrap_or("");

        // 安全策略强制执行：spawn 是写操作
        if !called_via_agent_tool
            && let Err(error) =
                self.security.enforce_tool_operation(ToolOperation::Act, "subagent_spawn")
        {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }

        // 查找代理配置
        let agent_config = match self.agents.get(agent_name) {
            Some(cfg) => cfg.clone(),
            None => {
                let available: Vec<&str> =
                    self.agents.keys().map(|s: &String| s.as_str()).collect();
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Unknown agent '{agent_name}'. Available agents: {}",
                        if available.is_empty() {
                            "(none configured)".to_string()
                        } else {
                            available.join(", ")
                        }
                    )),
                });
            }
        };

        // 为此代理创建提供者
        let provider_credential_owned =
            agent_config.api_key.clone().or_else(|| self.fallback_credential.clone());
        #[allow(clippy::option_as_ref_deref)]
        let provider_credential = provider_credential_owned.as_ref().map(String::as_str);

        let provider: Box<dyn Provider> =
            match crate::app::agent::providers::create_provider_with_options(
                &agent_config.provider,
                provider_credential,
                &self.provider_runtime_options,
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "Failed to create provider '{}' for agent '{agent_name}': {e}",
                            agent_config.provider
                        )),
                    });
                }
            };

        // 构建消息
        let full_prompt = if context.is_empty() {
            task.to_string()
        } else {
            format!("[Context]\n{context}\n\n[Task]\n{task}")
        };

        let session_id = uuid::Uuid::new_v4().to_string();
        let agent_name_owned = agent_name.to_string();
        let task_owned = task.to_string();
        let merged_system_prompt =
            self.merged_system_prompt(agent_name, agent_config.system_prompt.as_deref());

        // 判断是否为代理模式
        let is_agentic = agent_config.agentic;
        let parent_tools = self.parent_tools.clone();
        let multimodal_config = self.multimodal_config.clone();
        let security = self.security.clone();

        // 原子性检查并发限制并注册会话，防止竞争条件
        let session = SubAgentSession {
            id: session_id.clone(),
            agent_name: agent_name_owned.clone(),
            title: None,
            task: task_owned,
            metadata: Value::Object(Default::default()),
            status: SubAgentStatus::Running,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            result: None,
            #[cfg(not(target_arch = "wasm32"))]
            handle: None,
        };
        if let Err(_running) = self.registry.try_insert(session, MAX_CONCURRENT_SUBAGENTS) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Maximum concurrent sub-agents reached ({MAX_CONCURRENT_SUBAGENTS}). \
                     Wait for running agents to complete or kill some."
                )),
            });
        }

        // 克隆后台任务所需的引用
        let registry = self.registry.clone();
        let sid = session_id.clone();

        let task_future = async move {
            // 根据代理模式选择执行路径
            let result = if is_agentic {
                run_agentic_background(
                    &agent_name_owned,
                    &agent_config,
                    &*provider,
                    merged_system_prompt.as_deref(),
                    &full_prompt,
                    &parent_tools,
                    &multimodal_config,
                    security,
                )
                .await
            } else {
                run_simple_background(
                    &agent_name_owned,
                    &agent_config,
                    &*provider,
                    merged_system_prompt.as_deref(),
                    &full_prompt,
                )
                .await
            };

            // 处理执行结果并更新注册表
            match result {
                Ok(tool_result) => {
                    if tool_result.success {
                        registry.complete(&sid, tool_result);
                    } else {
                        registry.fail(
                            &sid,
                            tool_result.error.unwrap_or_else(|| "Unknown error".to_string()),
                        );
                    }
                }
                Err(e) => {
                    registry.fail(&sid, format!("Agent '{agent_name_owned}' error: {e}"));
                }
            }
        };

        // 在非 WASM 环境使用 tokio::spawn
        #[cfg(not(target_arch = "wasm32"))]
        {
            let handle = tokio::spawn(task_future);
            self.registry.set_handle(&session_id, handle);
        }

        // 在 WASM 环境使用 wasm_bindgen_futures::spawn_local
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(task_future);
        }

        Ok(ToolResult {
            success: true,
            output: json!({
                "session_id": session_id,
                "agent": agent_name,
                "status": "running",
                "message": "Agent session spawned in background. Use AgentTool to inspect progress."
            })
            .to_string(),
            error: None,
        })
    }
}

/// 在后台运行简单模式的子代理。
///
/// 简单模式下，子代理只进行单次对话，不使用工具。
/// 适用于简单的问答或分析任务。
///
/// # 参数
///
/// - `agent_name`: 代理名称，用于日志和错误信息
/// - `agent_config`: 代理配置，包含模型、温度等参数
/// - `provider`: 提供者实例，用于调用 LLM API
/// - `full_prompt`: 完整提示文本，已包含上下文和任务
///
/// # 返回值
///
/// 返回 `ToolResult`，其中：
/// - 成功时：`output` 包含代理响应
/// - 失败时：`error` 包含超时或 API 错误信息
///
/// # 超时处理
///
/// 执行时间超过 `SPAWN_TIMEOUT_SECS` 时将返回超时错误。
async fn run_simple_background(
    agent_name: &str,
    agent_config: &DelegateAgentConfig,
    provider: &dyn Provider,
    system_prompt: Option<&str>,
    full_prompt: &str,
) -> anyhow::Result<ToolResult> {
    let temperature = agent_config.temperature.unwrap_or(0.7);

    // 使用超时包装器执行提供者调用
    let result = tokio::time::timeout(
        Duration::from_secs(SPAWN_TIMEOUT_SECS),
        provider.chat_with_system(system_prompt, full_prompt, &agent_config.model, temperature),
    )
    .await;

    // 处理超时情况
    let result = match result {
        Ok(inner) => inner,
        Err(_elapsed) => {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Agent '{agent_name}' timed out after {SPAWN_TIMEOUT_SECS}s")),
            });
        }
    };

    // 处理提供者响应
    match result {
        Ok(response) => {
            // 处理空响应
            let rendered =
                if response.trim().is_empty() { "[Empty response]".to_string() } else { response };

            Ok(ToolResult {
                success: true,
                output: format!(
                    "[Agent '{agent_name}' ({provider}/{model})]\n{rendered}",
                    provider = agent_config.provider,
                    model = agent_config.model
                ),
                error: None,
            })
        }
        Err(e) => Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Agent '{agent_name}' failed: {e}")),
        }),
    }
}

/// 空操作观察者。
///
/// 不记录任何事件和指标的观察者实现。
/// 用于子代理的代理模式循环中，当不需要观察功能时使用。
///
/// ## 设计原因
///
/// 子代理运行在独立的后台任务中，通常不需要向父代理的
/// 观察者报告事件。使用空观察者可以避免不必要的开销。
struct NoopObserver;

/// `Observer` trait 实现
///
/// 所有方法均为空操作，不记录任何数据。
impl Observer for NoopObserver {
    /// 不记录事件。
    fn record_event(&self, _event: &ObserverEvent) {}

    /// 不记录指标。
    fn record_metric(&self, _metric: &ObserverMetric) {}

    /// 返回观察者名称。
    fn name(&self) -> &str {
        "noop"
    }

    /// 返回 `Any` 引用，用于类型转换。
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 在后台运行代理模式的子代理。
///
/// 代理模式下，子代理可以使用工具并执行多轮对话循环。
/// 适用于需要调用工具完成复杂任务的场景。
///
/// # 参数
///
/// - `agent_name`: 代理名称，用于日志和错误信息
/// - `agent_config`: 代理配置，包含模型、温度、允许的工具列表等
/// - `provider`: 提供者实例，用于调用 LLM API
/// - `full_prompt`: 完整提示文本，已包含上下文和任务
/// - `parent_tools`: 父代理的工具集，子代理可从中选择使用
/// - `multimodal_config`: 多模态配置
///
/// # 返回值
///
/// 返回 `ToolResult`，其中：
/// - 成功时：`output` 包含代理的最终响应
/// - 失败时：`error` 包含错误信息（如无可用工具、超时等）
///
/// # 工具过滤规则
///
/// 子代理只能使用 `allowed_tools` 列表中的工具，且排除以下工具：
/// - `delegate`: 防止递归委托
/// - `subagent_spawn`: 防止子代理再生成子代理
/// - `subagent_manage`: 防止子代理管理其他子代理
///
/// # 超时处理
///
/// 执行时间超过 `SPAWN_TIMEOUT_SECS` 时将返回超时错误。
async fn run_agentic_background(
    agent_name: &str,
    agent_config: &DelegateAgentConfig,
    provider: &dyn Provider,
    system_prompt: Option<&str>,
    full_prompt: &str,
    parent_tools: &[Arc<dyn Tool>],
    multimodal_config: &crate::app::agent::config::MultimodalConfig,
    security: Arc<SecurityPolicy>,
) -> anyhow::Result<ToolResult> {
    // 检查允许的工具列表是否为空
    if agent_config.allowed_tools.is_empty() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!(
                "Agent '{agent_name}' has agentic=true but allowed_tools is empty"
            )),
        });
    }

    let sub_tools = crate::app::agent::tools::delegated_tools::build_agentic_tools(
        parent_tools,
        &agent_config.allowed_tools,
        &agent_config.allowed_skills,
    );

    // 检查过滤后是否还有可用工具
    if sub_tools.is_empty() {
        return Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!(
                "Agent '{agent_name}' has no executable tools after filtering allowlist ({})",
                agent_config.allowed_tools.join(", ")
            )),
        });
    }

    // 构建对话历史
    let temperature = agent_config.temperature.unwrap_or(0.7);
    let mut history = Vec::new();

    // 添加系统提示（如果有）
    if let Some(system_prompt) = system_prompt {
        history.push(ChatMessage::system(system_prompt.to_string()));
    }

    // 添加用户消息
    history.push(ChatMessage::user(full_prompt.to_string()));

    // 创建空观察者
    let noop_observer = NoopObserver;

    // 使用超时包装器运行工具调用循环
    let result = tokio::time::timeout(
        Duration::from_secs(SPAWN_TIMEOUT_SECS),
        crate::app::agent::agent::loop_::run_tool_call_loop(
            provider,
            &mut history,
            &sub_tools,
            &noop_observer,
            &agent_config.provider,
            &agent_config.model,
            temperature,
            true, // 启用工具调用
            Option::<Arc<ApprovalManager>>::None,
            "subagent_spawn",
            multimodal_config,
            agent_config.max_iterations,
            None,
            None,
            Option::<Arc<HookRunner>>::None,
            Some(security),
            &[],
        ),
    )
    .await;

    // 处理执行结果
    match result {
        Ok(Ok(response)) => {
            // 处理空响应
            let rendered =
                if response.trim().is_empty() { "[Empty response]".to_string() } else { response };

            Ok(ToolResult {
                success: true,
                output: format!(
                    "[Agent '{agent_name}' ({provider}/{model}, agentic)]\n{rendered}",
                    provider = agent_config.provider,
                    model = agent_config.model
                ),
                error: None,
            })
        }
        Ok(Err(e)) => Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Agent '{agent_name}' failed: {e}")),
        }),
        Err(_) => Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("Agent '{agent_name}' timed out after {SPAWN_TIMEOUT_SECS}s")),
        }),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
