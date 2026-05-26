//! 代理委派工具的协调消息追踪。
//!
//! 本模块只负责把一次委派请求映射为轻量的内存消息总线事件，便于上层观察
//! “已排队 / 已完成 / 失败”等状态。它不参与实际执行，也不改变委派结果。

use super::{COORDINATION_PREVIEW_MAX_CHARS, DelegateTool};
use crate::app::agent::config::DelegateAgentConfig;
use crate::app::agent::coordination::{
    CoordinationEnvelope, CoordinationPayload, InMemoryMessageBus,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// 单次委派调用在协调总线中的关联信息。
///
/// `correlation_id` 用于把请求、状态补丁与结果串起来；`conversation_id`
/// 为该委派调用提供独立对话范围；`request_message_id` 在请求成功发布后记录。
#[derive(Debug, Clone)]
pub(super) struct CoordinationTrace {
    pub(super) correlation_id: String,
    pub(super) conversation_id: String,
    pub(super) request_message_id: Option<String>,
}

pub(super) fn build_coordination_bus(
    agents: &HashMap<String, DelegateAgentConfig>,
    lead_agent: &str,
) -> Option<InMemoryMessageBus> {
    if agents.is_empty() {
        return None;
    }

    let bus = InMemoryMessageBus::new();

    // lead agent 也注册进总线，便于后续把状态补丁发回同一个观察入口。
    if let Err(error) = bus.register_agent(lead_agent.to_string()) {
        tracing::warn!(
            "delegate coordination: failed to register default lead agent '{lead_agent}': {error}"
        );
        return None;
    }

    for name in agents.keys() {
        if let Err(error) = bus.register_agent(name.clone()) {
            tracing::warn!(
                "delegate coordination: failed to register delegate agent '{name}': {error}"
            );
            return None;
        }
    }

    Some(bus)
}

pub(super) fn start_coordination_trace(
    tool: &DelegateTool,
    agent_name: &str,
    prompt: &str,
    context: &str,
    agent_config: &DelegateAgentConfig,
) -> CoordinationTrace {
    let correlation_id = Uuid::new_v4().to_string();
    let conversation_id = format!("delegate:{correlation_id}");
    let mut trace = CoordinationTrace {
        correlation_id: correlation_id.clone(),
        conversation_id: conversation_id.clone(),
        request_message_id: None,
    };

    let Some(bus) = &tool.coordination_bus else {
        return trace;
    };

    let mut request = CoordinationEnvelope::new_direct(
        tool.coordination_lead_agent.clone(),
        agent_name.to_string(),
        conversation_id.clone(),
        "delegate.request",
        CoordinationPayload::DelegateTask {
            task_id: correlation_id.clone(),
            summary: text_preview(prompt, COORDINATION_PREVIEW_MAX_CHARS),
            metadata: json!({
                "provider": agent_config.provider,
                "model": agent_config.model,
                "agentic": agent_config.agentic,
                "max_depth": agent_config.max_depth,
                "max_iterations": agent_config.max_iterations,
                "context_present": !context.is_empty()
            }),
        },
    );
    request.correlation_id = Some(correlation_id.clone());
    let request_message_id = request.id.clone();

    if let Err(error) = bus.publish(request) {
        tracing::warn!(
            "delegate coordination: failed to publish delegate request for '{agent_name}': {error}"
        );
    } else {
        trace.request_message_id = Some(request_message_id);
    }

    // 状态补丁写回 lead agent 名下，避免子代理未启动或失败时丢失可观测状态。
    let mut queued_state = CoordinationEnvelope::new_direct(
        tool.coordination_lead_agent.clone(),
        tool.coordination_lead_agent.clone(),
        conversation_id,
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: format!("delegate/{correlation_id}/state"),
            expected_version: 0,
            value: json!({
                "phase": "queued",
                "agent": agent_name,
                "context_present": !context.is_empty()
            }),
        },
    );
    queued_state.correlation_id = Some(correlation_id);
    queued_state.causation_id = trace.request_message_id.clone();

    if let Err(error) = bus.publish(queued_state) {
        tracing::warn!(
            "delegate coordination: failed to publish queued-state patch for '{agent_name}': {error}"
        );
    }

    trace
}

pub(super) fn finish_coordination_trace(
    tool: &DelegateTool,
    agent_name: &str,
    trace: &CoordinationTrace,
    success: bool,
    detail: &str,
) {
    let Some(bus) = &tool.coordination_bus else {
        return;
    };

    let detail_preview = text_preview(detail, COORDINATION_PREVIEW_MAX_CHARS);
    let mut result = CoordinationEnvelope::new_direct(
        agent_name.to_string(),
        tool.coordination_lead_agent.clone(),
        trace.conversation_id.clone(),
        "delegate.result",
        CoordinationPayload::TaskResult {
            task_id: trace.correlation_id.clone(),
            success,
            output: detail_preview.clone(),
        },
    );
    result.correlation_id = Some(trace.correlation_id.clone());
    result.causation_id = trace.request_message_id.clone();

    if let Err(error) = bus.publish(result) {
        tracing::warn!(
            "delegate coordination: failed to publish delegate result for '{agent_name}': {error}"
        );
    }

    let phase = if success { "completed" } else { "failed" };
    let mut completed_state = CoordinationEnvelope::new_direct(
        tool.coordination_lead_agent.clone(),
        tool.coordination_lead_agent.clone(),
        trace.conversation_id.clone(),
        "delegate.state",
        CoordinationPayload::ContextPatch {
            key: format!("delegate/{}/state", trace.correlation_id),
            expected_version: 1,
            value: json!({
                "phase": phase,
                "agent": agent_name,
                "success": success,
                "detail": detail_preview
            }),
        },
    );
    completed_state.correlation_id = Some(trace.correlation_id.clone());
    completed_state.causation_id = trace.request_message_id.clone();

    if let Err(error) = bus.publish(completed_state) {
        tracing::warn!(
            "delegate coordination: failed to publish completion-state patch for '{agent_name}': {error}"
        );
    }
}

fn text_preview(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "[empty]".to_string();
    }

    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let mut preview = trimmed.chars().take(max_chars).collect::<String>();
    preview.push_str("...");
    preview
}
#[cfg(test)]
#[path = "coordination_tests.rs"]
mod coordination_tests;
