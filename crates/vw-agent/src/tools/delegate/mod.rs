//! 任务委托工具模块
//!
//! 本模块提供将子任务委托给具有不同提供方/模型配置的命名代理的能力。
//! 支持单次提示模式和带工具调用循环的智能体模式。

use super::traits::{Tool, ToolResult};
use crate::app::agent::config::DelegateAgentConfig;
use crate::app::agent::coordination::InMemoryMessageBus;
use crate::app::agent::providers::{Provider, ProviderRuntimeOptions};
use crate::app::agent::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

mod agentic;
mod coordination;
mod execution;
mod support;

/// 子智能体提供方调用的默认超时时间（秒）
const DELEGATE_TIMEOUT_SECS: u64 = 120;

/// 智能体模式子智能体运行的默认超时时间（秒）
const DELEGATE_AGENTIC_TIMEOUT_SECS: u64 = 300;

/// 协调事件追踪使用的默认合成主智能体名称
const DEFAULT_COORDINATION_LEAD_AGENT: &str = "delegate-lead";

/// 协调事件预览中保留的最大字符数
const COORDINATION_PREVIEW_MAX_CHARS: usize = 240;

/// 任务委托工具
pub struct DelegateTool {
    agents: Arc<HashMap<String, DelegateAgentConfig>>,
    security: Arc<SecurityPolicy>,
    fallback_credential: Option<String>,
    provider_runtime_options: ProviderRuntimeOptions,
    depth: u32,
    parent_tools: Arc<Vec<Arc<dyn Tool>>>,
    multimodal_config: crate::app::agent::config::MultimodalConfig,
    workspace_identity_context: String,
    coordination_bus: Option<InMemoryMessageBus>,
    coordination_lead_agent: String,
}

impl DelegateTool {
    pub fn new(
        agents: HashMap<String, DelegateAgentConfig>,
        fallback_credential: Option<String>,
        security: Arc<SecurityPolicy>,
    ) -> Self {
        Self::new_with_options(
            agents,
            fallback_credential,
            security,
            ProviderRuntimeOptions::default(),
        )
    }

    pub fn new_with_options(
        agents: HashMap<String, DelegateAgentConfig>,
        fallback_credential: Option<String>,
        security: Arc<SecurityPolicy>,
        provider_runtime_options: ProviderRuntimeOptions,
    ) -> Self {
        let coordination_bus =
            coordination::build_coordination_bus(&agents, DEFAULT_COORDINATION_LEAD_AGENT);
        Self {
            agents: Arc::new(agents),
            security,
            fallback_credential,
            provider_runtime_options,
            depth: 0,
            parent_tools: Arc::new(Vec::new()),
            multimodal_config: crate::app::agent::config::MultimodalConfig::default(),
            workspace_identity_context: String::new(),
            coordination_bus,
            coordination_lead_agent: DEFAULT_COORDINATION_LEAD_AGENT.to_string(),
        }
    }

    pub fn with_depth(
        agents: HashMap<String, DelegateAgentConfig>,
        fallback_credential: Option<String>,
        security: Arc<SecurityPolicy>,
        depth: u32,
    ) -> Self {
        Self::with_depth_and_options(
            agents,
            fallback_credential,
            security,
            depth,
            ProviderRuntimeOptions::default(),
        )
    }

    pub fn with_depth_and_options(
        agents: HashMap<String, DelegateAgentConfig>,
        fallback_credential: Option<String>,
        security: Arc<SecurityPolicy>,
        depth: u32,
        provider_runtime_options: ProviderRuntimeOptions,
    ) -> Self {
        let coordination_bus =
            coordination::build_coordination_bus(&agents, DEFAULT_COORDINATION_LEAD_AGENT);
        Self {
            agents: Arc::new(agents),
            security,
            fallback_credential,
            provider_runtime_options,
            depth,
            parent_tools: Arc::new(Vec::new()),
            multimodal_config: crate::app::agent::config::MultimodalConfig::default(),
            workspace_identity_context: String::new(),
            coordination_bus,
            coordination_lead_agent: DEFAULT_COORDINATION_LEAD_AGENT.to_string(),
        }
    }

    pub fn with_parent_tools(mut self, parent_tools: Arc<Vec<Arc<dyn Tool>>>) -> Self {
        self.parent_tools = parent_tools;
        self
    }

    pub fn with_multimodal_config(
        mut self,
        config: crate::app::agent::config::MultimodalConfig,
    ) -> Self {
        self.multimodal_config = config;
        self
    }

    pub fn with_workspace_identity_context(mut self, workspace_identity_context: String) -> Self {
        self.workspace_identity_context = workspace_identity_context;
        self
    }

    pub fn with_coordination_bus(
        mut self,
        bus: InMemoryMessageBus,
        lead_agent: impl Into<String>,
    ) -> Self {
        let lead_agent = {
            let lead = lead_agent.into();
            if lead.trim().is_empty() {
                DEFAULT_COORDINATION_LEAD_AGENT.to_string()
            } else {
                lead.trim().to_string()
            }
        };

        if let Err(error) = bus.register_agent(lead_agent.clone()) {
            tracing::warn!(
                "delegate coordination: failed to register lead agent '{lead_agent}': {error}"
            );
        }

        self.coordination_bus = Some(bus);
        self.coordination_lead_agent = lead_agent;
        self
    }

    pub fn with_coordination_disabled(mut self) -> Self {
        self.coordination_bus = None;
        self
    }

    #[cfg(test)]
    fn coordination_bus_snapshot(&self) -> Option<InMemoryMessageBus> {
        self.coordination_bus.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for DelegateTool {
    fn name(&self) -> &str {
        "delegate"
    }

    fn description(&self) -> &str {
        "将子任务委托给专门的智能体。适用场景：任务受益于不同的模型（例如快速摘要、深度推理、代码生成）。\
         子智能体默认运行单个提示；设置 agentic=true 时可进行带过滤工具调用循环的迭代。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        let agent_names: Vec<&str> = self.agents.keys().map(|name| name.as_str()).collect();
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "agent": {
                    "type": "string",
                    "minLength": 1,
                    "description": format!(
                        "要委托的智能体名称。可用：{}",
                        if agent_names.is_empty() {
                            "none configured".to_string()
                        } else {
                            agent_names.join(", ")
                        }
                    )
                },
                "prompt": {
                    "type": "string",
                    "minLength": 1,
                    "description": "发送给子智能体的任务/提示"
                },
                "context": {
                    "type": "string",
                    "description": "可选的前置上下文（例如相关代码、先前发现）"
                }
            },
            "required": ["agent", "prompt"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        execution::execute(self, args).await
    }
}

impl DelegateTool {
    fn merged_system_prompt(&self, agent_system_prompt: Option<&str>) -> Option<String> {
        match (
            self.workspace_identity_context.trim().is_empty(),
            agent_system_prompt.map(str::trim).filter(|prompt| !prompt.is_empty()),
        ) {
            (true, None) => None,
            (true, Some(agent_prompt)) => Some(agent_prompt.to_string()),
            (false, None) => Some(self.workspace_identity_context.clone()),
            (false, Some(agent_prompt)) => {
                Some(format!("{}\n\n{}", self.workspace_identity_context, agent_prompt))
            }
        }
    }

    async fn execute_agentic(
        &self,
        agent_name: &str,
        agent_config: &DelegateAgentConfig,
        provider: &dyn Provider,
        system_prompt: Option<&str>,
        full_prompt: &str,
        temperature: f64,
    ) -> anyhow::Result<ToolResult> {
        agentic::execute_agentic(
            self,
            agent_name,
            agent_config,
            provider,
            system_prompt,
            full_prompt,
            temperature,
        )
        .await
    }

    fn start_coordination_trace(
        &self,
        agent_name: &str,
        prompt: &str,
        context: &str,
        agent_config: &DelegateAgentConfig,
    ) -> coordination::CoordinationTrace {
        coordination::start_coordination_trace(self, agent_name, prompt, context, agent_config)
    }

    fn finish_coordination_trace(
        &self,
        agent_name: &str,
        trace: &coordination::CoordinationTrace,
        success: bool,
        detail: &str,
    ) {
        coordination::finish_coordination_trace(self, agent_name, trace, success, detail)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
