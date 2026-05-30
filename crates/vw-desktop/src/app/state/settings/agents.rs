//! 维护系统设置状态及其按领域拆分的派生逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;
use vw_config_types::agent::{
    BuiltinAgentKind, DelegateAgentConfig, builtin_agent_config, builtin_agent_keys,
    builtin_agent_spec,
};

#[derive(Debug, Clone)]
/// 表示 DelegateAgentSettingsEntry 相关的应用状态或派生数据。
pub(crate) struct DelegateAgentSettingsEntry {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) kind: AgentSettingsEntryKind,
    pub(crate) enabled: bool,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) system_prompt_editor: text_editor::Content,
    pub(crate) api_key_input: String,
    pub(crate) temperature: f32,
    pub(crate) compact_context: bool,
    pub(crate) max_tool_iterations: u32,
    pub(crate) max_history_messages: u32,
    pub(crate) parallel_tools: bool,
    pub(crate) tool_dispatcher: String,
    pub(crate) max_depth: u32,
    pub(crate) agentic: bool,
    pub(crate) allowed_tools: Vec<String>,
    pub(crate) allowed_skills: Vec<String>,
    pub(crate) max_iterations: u32,
}

impl DelegateAgentSettingsEntry {
    /// 执行 from_config 对应的领域操作。
    ///
    /// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
    /// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
    pub(crate) fn from_config(key: &str, config: Option<DelegateAgentConfig>) -> Self {
        let kind = if key == MAIN_AGENT_KEY {
            AgentSettingsEntryKind::Main
        } else if matches!(
            builtin_agent_spec(key).map(|spec| spec.kind),
            Some(BuiltinAgentKind::Worker)
        ) {
            AgentSettingsEntryKind::BuiltinWorker
        } else {
            AgentSettingsEntryKind::Custom
        };
        let config = config.unwrap_or_else(|| default_agent_config_for_key(key));

        Self {
            key: key.to_string(),
            label: default_agent_label(key, &config),
            kind,
            enabled: if key == MAIN_AGENT_KEY { true } else { config.enabled },
            provider: config.provider,
            model: config.model,
            system_prompt_editor: text_editor::Content::with_text(
                config
                    .system_prompt
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_default(),
            ),
            api_key_input: config.api_key.unwrap_or_default(),
            temperature: config
                .temperature
                .or_else(|| builtin_agent_spec(key).and_then(|spec| spec.default_temperature))
                .unwrap_or(0.7)
                .clamp(0.0, 2.0) as f32,
            compact_context: false,
            max_tool_iterations: 20,
            max_history_messages: 50,
            parallel_tools: false,
            tool_dispatcher: "auto".to_string(),
            max_depth: config.max_depth.clamp(1, 32),
            agentic: config.agentic,
            allowed_tools: config.allowed_tools,
            allowed_skills: config.allowed_skills,
            max_iterations: config.max_iterations.clamp(1, 100) as u32,
        }
    }
}

fn default_agent_config_for_key(key: &str) -> DelegateAgentConfig {
    builtin_agent_config(key).unwrap_or_default()
}

fn default_agent_label(key: &str, config: &DelegateAgentConfig) -> String {
    config
        .label
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| builtin_agent_spec(key).map(|spec| spec.label.to_string()))
        .unwrap_or_else(|| key.to_string())
}

/// 执行 ordered_agent_keys 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn ordered_agent_keys(
    configured_agents: &std::collections::HashMap<String, DelegateAgentConfig>,
) -> Vec<String> {
    let mut custom_agent_keys = configured_agents
        .keys()
        .filter(|key| builtin_agent_spec(key.as_str()).is_none())
        .cloned()
        .collect::<Vec<_>>();
    custom_agent_keys.sort();
    builtin_agent_keys().into_iter().map(str::to_string).chain(custom_agent_keys).collect()
}

#[derive(Debug, Clone)]
/// 表示 AgentsSettingsState 相关的应用状态或派生数据。
pub(crate) struct AgentsSettingsState {
    pub(crate) loading: bool,
    pub(crate) providers: Vec<ProviderSummary>,
    pub(crate) provider_models: Vec<ProviderModelsSummary>,
    pub(crate) entries: Vec<DelegateAgentSettingsEntry>,
    pub(crate) new_agent_key_input: String,
    pub(crate) selected_agent: String,
    pub(crate) active_detail_tab: String,
    pub(crate) active_prompt_tab: String,
    pub(crate) workspace_identity_files: Vec<WorkspaceIdentityFileState>,
    pub(crate) workspace_identity_root_path: Option<String>,
    pub(crate) available_tools: Vec<String>,
    pub(crate) save_error: Option<String>,
}

impl Default for AgentsSettingsState {
    fn default() -> Self {
        Self {
            loading: false,
            providers: Vec::new(),
            provider_models: Vec::new(),
            entries: builtin_agent_keys()
                .iter()
                .map(|key| {
                    DelegateAgentSettingsEntry::from_config(
                        key,
                        Some(default_agent_config_for_key(key)),
                    )
                })
                .collect(),
            new_agent_key_input: String::new(),
            selected_agent: MAIN_AGENT_KEY.to_string(),
            active_detail_tab: AGENT_DETAIL_BASIC_TAB.to_string(),
            active_prompt_tab: AGENT_PROMPT_SYSTEM_TAB.to_string(),
            workspace_identity_files: WORKSPACE_IDENTITY_FILES
                .iter()
                .map(|(file_name, label)| WorkspaceIdentityFileState {
                    file_name: (*file_name).to_string(),
                    label: (*label).to_string(),
                    editor: text_editor::Content::with_text(""),
                    size_bytes: None,
                    modified_at_ms: None,
                })
                .collect(),
            workspace_identity_root_path: None,
            available_tools: Vec::new(),
            save_error: None,
        }
    }
}

#[cfg(test)]
#[path = "agents_tests.rs"]
mod agents_tests;
