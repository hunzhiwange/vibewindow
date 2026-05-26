//! 会话工件持久化模块。
//!
//! 本模块集中处理会话处理器在运行过程中产出的持久化载荷，
//! 包括 AI 调用摘要与 LLM 原始步骤数据，避免主循环重复拼装大块 JSON。

use super::llm_runner::LlmStep;
use super::prompting;
use super::types::Request;
use super::utils;
use crate::session::session::{Role, Session};
use crate::session::ui_types as models;
use serde_json::{Value, json};

fn session_messages_payload(session: &Session) -> Vec<Value> {
    session
        .messages
        .iter()
        .map(|message| {
            json!({
                "role": match message.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "system",
                    Role::Tool => "tool",
                },
                "content": message.content,
            })
        })
        .collect()
}

/// 持久化单步 AI 调用摘要载荷。
pub(crate) fn persist_step_ai_call_payload(
    req: &Request,
    session: &Session,
    total_usage: &models::TokenUsage,
    step: &LlmStep,
    step_index: u32,
    app_session_scope: Option<&str>,
) {
    if !req.persist_app_session_artifacts || req.session.is_empty() {
        return;
    }

    let usage_now = models::TokenUsage {
        input_tokens: total_usage.input_tokens + step.usage.input_tokens,
        output_tokens: total_usage.output_tokens + step.usage.output_tokens,
        cached_tokens: total_usage.cached_tokens + step.usage.cached_tokens,
        reasoning_tokens: total_usage.reasoning_tokens + step.usage.reasoning_tokens,
    };
    let payload = json!({
        "time": { "created_ms": utils::now_ms() },
        "session_id": req.session,
        "stream_id": req.stream,
        "step_index": step_index,
        "model": req.model,
        "root": req.root,
        "usage": {
            "input_tokens": usage_now.input_tokens,
            "output_tokens": usage_now.output_tokens,
            "cached_tokens": usage_now.cached_tokens,
            "reasoning_tokens": usage_now.reasoning_tokens,
        },
        "prompt": prompting::build_prompt(
            session,
            req.model.as_deref(),
            req.root.as_deref(),
            None,
        ),
        "answer": step.text,
        "messages": session_messages_payload(session),
    });
    let _ = crate::session::ui_store::persist_ai_call_payload(
        &req.session,
        req.stream,
        &payload,
        app_session_scope,
    );
}

/// 持久化原始 LLM 步骤载荷。
pub(crate) fn persist_llm_raw_step_payload(
    req: &Request,
    session_id: &str,
    step_index: u32,
    llm_messages: &[Value],
    step: &LlmStep,
    app_session_scope: Option<&str>,
) {
    if !req.persist_app_session_artifacts || session_id.is_empty() {
        return;
    }

    let tool_calls = step
        .tool_calls
        .iter()
        .map(|call| {
            json!({
                "id": call.id,
                "type": "function",
                "function": { "name": call.name, "arguments": call.arguments }
            })
        })
        .collect::<Vec<_>>();
    let system_messages = step
        .full_messages
        .iter()
        .filter(|message| message.get("role").and_then(Value::as_str) == Some("system"))
        .filter_map(|message| {
            message
                .get("content")
                .and_then(Value::as_str)
                .map(|content| content.to_string())
        })
        .collect::<Vec<_>>();
    let payload = json!({
        "session_id": session_id,
        "step_index": step_index,
        "model": req.model.clone(),
        "system": system_messages,
        "messages": llm_messages,
        "output": {
            "text": step.text,
            "reasoning_content": step.reasoning_content,
            "finish_reason": step.finish_reason,
            "tool_calls": tool_calls,
            "usage": step.usage,
        }
    });
    let _ = crate::session::ui_store::persist_llm_raw_step(
        session_id,
        step_index,
        &payload,
        app_session_scope,
    );
}

/// 持久化最终 AI 调用载荷。
pub(crate) fn persist_final_ai_call_payload(
    req: &Request,
    session: &Session,
    total_usage: &models::TokenUsage,
    answer: &str,
    app_session_scope: Option<&str>,
    should_persist: bool,
) {
    if !should_persist {
        return;
    }

    let created_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let payload = json!({
        "time": { "created_ms": created_ms },
        "session_id": req.session,
        "stream_id": req.stream,
        "model": req.model,
        "root": req.root,
        "usage": {
            "input_tokens": total_usage.input_tokens,
            "output_tokens": total_usage.output_tokens,
            "cached_tokens": total_usage.cached_tokens,
            "reasoning_tokens": total_usage.reasoning_tokens,
        },
        "prompt": prompting::build_prompt(
            session,
            req.model.as_deref(),
            req.root.as_deref(),
            None,
        ),
        "answer": answer,
        "messages": session_messages_payload(session),
    });
    let _ = crate::session::ui_store::persist_ai_call_payload(
        &req.session,
        req.stream,
        &payload,
        app_session_scope,
    );
}
#[cfg(test)]
#[path = "artifacts_tests.rs"]
mod artifacts_tests;
