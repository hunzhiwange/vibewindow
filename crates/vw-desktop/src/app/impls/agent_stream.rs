//! 将桌面代理请求转换为网关流式消息。
//! 本模块隔离传输事件解析，让应用状态只接收明确的 Message。

use super::message;
use super::{AgentRequest, Message};

/// 模块内可见函数，执行 agent_stream 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn agent_stream(req: &AgentRequest) -> AgentBoxStream<Message> {
    gateway_agent_stream(req.clone())
}

#[cfg(not(target_arch = "wasm32"))]
type AgentBoxStream<T> = iced::futures::stream::BoxStream<'static, T>;
#[cfg(target_arch = "wasm32")]
type AgentBoxStream<T> = iced::futures::stream::LocalBoxStream<'static, T>;

fn gateway_agent_stream(req: AgentRequest) -> AgentBoxStream<Message> {
    use iced::futures::SinkExt;
    use iced::futures::StreamExt;
    use serde_json::{Value, json};
    use tracing::{debug, info};

    use crate::app::models;

    let s = iced::stream::channel(
        100,
        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            let endpoint = crate::app::config::gateway_client_endpoint();

            let mut messages = Vec::new();
            for message in &req.history {
                messages.push(json!({
                    "role": match message.role {
                        models::ChatRole::User => "user",
                        models::ChatRole::Assistant => "assistant",
                        models::ChatRole::System => "system",
                        models::ChatRole::Tool => "tool",
                    },
                    "content": message.content,
                }));
            }
            if !req.resume_history_only && !req.query.trim().is_empty() {
                messages.push(json!({"role": "user", "content": req.query}));
            }

            let body = json!({
                "messages": messages,
                "model": req.model,
            });
            let acp_cwd = req.root.clone();
            let mut options = serde_json::Map::new();
            if req.full_access_enabled {
                options.insert("full_access".to_string(), json!(true));
            }
            if let Some(agent) = &req.agent {
                options.insert("agent".to_string(), json!(agent));
            }
            if let Some(allowed_tools) = &req.allowed_tools {
                options.insert("allowed_tools".to_string(), json!(allowed_tools));
            }
            if req.acp_test {
                options.insert("acp_test".to_string(), json!(true));
                options.insert("acp_agent".to_string(), json!(req.acp_agent));
                if let Some(acp_allowed_tools) = &req.acp_allowed_tools {
                    options.insert("acp_allowed_tools".to_string(), json!(acp_allowed_tools));
                }
                options
                    .insert("acp_force_new_session".to_string(), json!(req.acp_force_new_session));
                options.insert(
                    "acp_history_strategy".to_string(),
                    json!(req.acp_history_mode.as_str()),
                );
                options.insert("acp_history_recent_count".to_string(), json!(req.acp_recent_count));
                options.insert("cwd".to_string(), json!(acp_cwd));
                if req.full_access_enabled {
                    options.insert("acp_permission_mode".to_string(), json!("approve-all"));
                }
            }
            if req.resume_history_only {
                options.insert("desktop_resume_history_only".to_string(), json!(true));
            }
            let options = (!options.is_empty()).then_some(Value::Object(options));
            let mut stream_done = false;
            let mut ended_by_post_tool_round_handoff = false;

            info!(
                target: "vw_desktop",
                request_id = req.id,
                session_id = %req.session,
                acp_test = req.acp_test,
                agent = ?req.agent,
                allowed_tools = ?req.allowed_tools,
                acp_agent = ?req.acp_agent,
                acp_allowed_tools = ?req.acp_allowed_tools,
                acp_force_new_session = req.acp_force_new_session,
                acp_history_mode = %req.acp_history_mode.as_str(),
                acp_recent_count = req.acp_recent_count,
                full_access_enabled = req.full_access_enabled,
                model = ?req.model,
                has_root = req.root.is_some(),
                history_len = req.history.len(),
                query_len = req.query.len(),
                options = ?options,
                "starting gateway agent stream"
            );

            let result = vw_gateway_client::GatewayClient::stream_chat(
                &endpoint,
                req.root.as_deref(),
                &vw_gateway_client::GatewayChatStreamRequest {
                    session_id: Some(req.session.clone().into()),
                    messages: body
                        .get("messages")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                    system: None,
                    model: body
                        .get("model")
                        .cloned()
                        .and_then(|value| serde_json::from_value(value).ok()),
                    agent: req.agent.clone(),
                    allowed_tools: req.allowed_tools.clone(),
                    acp_agent: req.acp_test.then(|| req.acp_agent.clone()).flatten(),
                    acp_allowed_tools: if req.acp_test {
                        req.acp_allowed_tools.clone()
                    } else {
                        None
                    },
                    options,
                },
                |event| {
                    let event_kind = match &event {
                        vw_gateway_client::GatewayChatStreamEvent::Delta(_) => "delta",
                        vw_gateway_client::GatewayChatStreamEvent::Done { .. } => "done",
                        vw_gateway_client::GatewayChatStreamEvent::Error(_) => "error",
                        vw_gateway_client::GatewayChatStreamEvent::Other(_) => "other",
                    };
                    let mut stop_after_send = false;

                    let next_message = match event {
                        vw_gateway_client::GatewayChatStreamEvent::Delta(delta) => {
                            let delta_len = delta.len();
                            let delta_preview: String = delta.chars().take(80).collect();
                            info!(
                                target: "vw_desktop",
                                request_id = req.id,
                                session_id = %req.session,
                                delta_len,
                                delta_preview = %delta_preview,
                                "desktop received gateway delta"
                            );
                            Some(Message::Chat(message::ChatMessage::AgentStreamDelta(
                                req.id, delta,
                            )))
                        }
                        vw_gateway_client::GatewayChatStreamEvent::Done {
                            usage,
                            message_id,
                            parent_message_id,
                            ..
                        } => {
                            info!(
                                target: "vw_desktop",
                                request_id = req.id,
                                session_id = %req.session,
                                has_usage = usage.is_some(),
                                message_id = ?message_id,
                                parent_message_id = ?parent_message_id,
                                "desktop received gateway done"
                            );
                            stream_done = true;
                            Some(Message::Chat(message::ChatMessage::AgentStreamDone(
                                req.id,
                                parse_usage(usage.as_ref()),
                                message_id,
                                parent_message_id,
                            )))
                        }
                        vw_gateway_client::GatewayChatStreamEvent::Error(error) => {
                            info!(
                                target: "vw_desktop",
                                request_id = req.id,
                                session_id = %req.session,
                                error = %error,
                                "desktop received gateway error"
                            );
                            stream_done = true;
                            Some(Message::Chat(message::ChatMessage::AgentStreamError(
                                req.id, error,
                            )))
                        }
                        vw_gateway_client::GatewayChatStreamEvent::Other(payload) => {
                            match payload
                                .get("type")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                            {
                                "chat.step_start" => {
                                    let step_index = payload
                                        .get("step_index")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or_default()
                                        as u32;
                                    let created_ms = payload
                                        .get("created_ms")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or_default();
                                    let model = payload
                                        .get("model")
                                        .and_then(serde_json::Value::as_str)
                                        .map(ToOwned::to_owned);
                                    Some(Message::Chat(message::ChatMessage::AgentStepStart(
                                        req.id,
                                        req.session.clone(),
                                        step_index,
                                        created_ms,
                                        model,
                                    )))
                                }
                                "chat.step_finish" => {
                                    let step_index = payload
                                        .get("step_index")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or_default()
                                        as u32;
                                    let finished_ms = payload
                                        .get("finished_ms")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or_default();
                                    let usage = parse_usage(payload.get("usage"));
                                    let finish_reason = payload
                                        .get("finish_reason")
                                        .and_then(serde_json::Value::as_str)
                                        .map(ToOwned::to_owned);
                                    let model = payload
                                        .get("model")
                                        .and_then(serde_json::Value::as_str)
                                        .map(ToOwned::to_owned);
                                    Some(Message::Chat(message::ChatMessage::AgentStepFinish(
                                        req.id,
                                        req.session.clone(),
                                        step_index,
                                        finished_ms,
                                        usage,
                                        finish_reason,
                                        model,
                                    )))
                                }
                                "chat.post_tool_round" => {
                                    if !crate::app::state::take_pending_guide_handoff(req.id) {
                                        None
                                    } else {
                                        ended_by_post_tool_round_handoff = true;
                                        stop_after_send = true;
                                        let step_index = payload
                                            .get("step_index")
                                            .and_then(serde_json::Value::as_u64)
                                            .unwrap_or_default()
                                            as u32;
                                        Some(Message::Chat(
                                            message::ChatMessage::AgentPostToolRound(
                                                req.id,
                                                req.session.clone(),
                                                step_index,
                                            ),
                                        ))
                                    }
                                }
                                _ => None,
                            }
                        }
                    };

                    let Some(next_message) = next_message else {
                        return true;
                    };

                    match output.try_send(next_message) {
                        Ok(()) => !stop_after_send,
                        Err(_) => {
                            debug!(
                                target: "vw_desktop",
                                request_id = req.id,
                                session_id = %req.session,
                                event_kind = event_kind,
                                "desktop agent stream forward failed"
                            );
                            false
                        }
                    }
                },
            )
            .await;

            if let Err(err) = result {
                debug!(
                    target: "vw_desktop",
                    request_id = req.id,
                    session_id = %req.session,
                    error = %err,
                    "gateway agent stream failed"
                );
                let _ = output
                    .send(Message::Chat(message::ChatMessage::AgentStreamError(req.id, err)))
                    .await;
                return;
            }

            if !stream_done {
                if ended_by_post_tool_round_handoff {
                    debug!(
                        target: "vw_desktop",
                        request_id = req.id,
                        session_id = %req.session,
                        "gateway agent stream stopped after post-tool-round handoff"
                    );
                    return;
                }
                debug!(
                    target: "vw_desktop",
                    request_id = req.id,
                    session_id = %req.session,
                    "gateway agent stream ended without done event"
                );
                let _ = output
                    .send(Message::Chat(message::ChatMessage::AgentStreamError(
                        req.id,
                        "任务异常终止：未收到网关完成信号".to_string(),
                    )))
                    .await;
            }
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    return s.boxed();
    #[cfg(target_arch = "wasm32")]
    return s.boxed_local();
}

fn parse_usage(value: Option<&serde_json::Value>) -> crate::app::models::TokenUsage {
    let Some(value) = value else {
        return crate::app::models::TokenUsage::default();
    };
    serde_json::from_value(value.clone()).unwrap_or_default()
}

#[cfg(test)]
#[path = "agent_stream_tests.rs"]
mod agent_stream_tests;
