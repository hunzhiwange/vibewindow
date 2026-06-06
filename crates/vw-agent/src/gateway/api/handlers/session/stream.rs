//! 桌面聊天流式接口。
//!
//! 本模块把 gateway chat 请求转换为 session processor 请求，并把处理器事件编码为
//! SSE 帧返回桌面端。它还负责合并 agent/model/options 覆盖项、把 tool 消息转换为
//! session 可理解的文本，以及在流结束后补写桌面会话消息。

use std::convert::Infallible;

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use futures_util::StreamExt;
use serde_json::{Map, Value};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vw_api_types::chat::GatewayChatStreamRequest;

use crate::app::agent::agent;
use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::approval_state;
use crate::app::agent::gateway::instance::InstanceQuery;
use crate::app::agent::gateway::instance::resolve_directory;
use crate::app::agent::gateway::instance::with_instance;
use crate::app::agent::project::instance;
use crate::app::agent::provider::provider;
use crate::app::agent::session as agent_session;
use crate::id;
use crate::session::ui_types as ui_models;

/// 单轮流式对话预分配的消息 id。
///
/// 预分配 id 让前端可以在流完成事件中得到稳定的 assistant/user 消息关联。
#[derive(Debug, Clone)]
pub(super) struct StreamTurnMessageIds {
    assistant_id: String,
    user_id: String,
}

impl StreamTurnMessageIds {
    /// 创建一组流式对话消息 id。
    ///
    /// # 参数
    ///
    /// * `assistant_id` - assistant 消息 id。
    /// * `user_id` - user 消息 id。
    ///
    /// # 返回值
    ///
    /// 返回用于持久化流式轮次的 id 容器。
    pub(super) fn new(assistant_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self { assistant_id: assistant_id.into(), user_id: user_id.into() }
    }
}

fn tool_content_to_session_text(content: &str) -> String {
    let parsed = serde_json::from_str::<Value>(content).ok();
    let tool_call_id =
        parsed.as_ref().and_then(|value| value.get("tool_call_id")).and_then(Value::as_str);
    let tool_output = parsed
        .as_ref()
        .and_then(|value| value.get("content"))
        .and_then(Value::as_str)
        .unwrap_or(content);

    match tool_call_id {
        Some(tool_call_id) => {
            format!(
                "[Tool results]\n<tool_result id=\"{tool_call_id}\">\n{tool_output}\n</tool_result>"
            )
        }
        None => format!("[Tool results]\n<tool_result>\n{tool_output}\n</tool_result>"),
    }
}

fn merge_gateway_request_options(
    options: Option<Value>,
    agent: Option<String>,
    allowed_tools: Option<Vec<String>>,
    chat_system_prompt: Option<String>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    acp_agent: Option<String>,
    acp_allowed_tools: Option<Vec<String>>,
) -> Value {
    let mut merged = match options {
        Some(Value::Object(map)) => map,
        Some(_) | None => Map::new(),
    };

    if let Some(agent) = agent {
        merged.insert("agent".to_string(), Value::String(agent));
    }

    let allowed_tools_for_acp = allowed_tools.clone();
    if let Some(allowed_tools) = allowed_tools {
        merged.insert(
            "allowed_tools".to_string(),
            Value::Array(allowed_tools.into_iter().map(Value::String).collect()),
        );
    }

    if let Some(chat_system_prompt) = chat_system_prompt
        && !merged.contains_key("chat_system_prompt")
    {
        // 用户显式 options 优先；agent 默认值只填补缺失项，避免覆盖调用方意图。
        merged.insert("chat_system_prompt".to_string(), Value::String(chat_system_prompt));
    }

    if let Some(temperature) = temperature
        && !merged.contains_key("temperature")
    {
        merged.insert("temperature".to_string(), Value::from(temperature));
    }

    if let Some(top_p) = top_p
        && !merged.contains_key("top_p")
    {
        merged.insert("top_p".to_string(), Value::from(top_p));
    }

    if let Some(acp_agent) = acp_agent {
        merged.insert("acp_test".to_string(), Value::Bool(true));
        merged.insert("acp_agent".to_string(), Value::String(acp_agent));
    }

    let acp_requested = merged.get("acp_test").and_then(Value::as_bool).unwrap_or(false)
        || merged
            .get("acp_agent")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
    let acp_allowed_tools =
        acp_allowed_tools.or_else(|| if acp_requested { allowed_tools_for_acp } else { None });
    if let Some(acp_allowed_tools) = acp_allowed_tools {
        merged.insert("acp_test".to_string(), Value::Bool(true));
        merged.insert(
            "acp_allowed_tools".to_string(),
            Value::Array(acp_allowed_tools.into_iter().map(Value::String).collect()),
        );
    }

    Value::Object(merged)
}

#[derive(Debug, Clone, Default)]
struct DelegateRequestOverrides {
    agent: Option<String>,
    model: Option<String>,
    allowed_tools: Option<Vec<String>>,
    chat_system_prompt: Option<String>,
    temperature: Option<f64>,
    top_p: Option<f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayChatStreamCollected {
    pub(crate) output: String,
    pub(crate) message_id: Option<String>,
    pub(crate) parent_message_id: Option<String>,
}

fn normalize_tool_ids(tools: Vec<String>) -> Option<Vec<String>> {
    let mut normalized = Vec::new();
    for tool in tools {
        let trimmed = tool.trim();
        if trimmed.is_empty() || normalized.iter().any(|existing| existing == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    (!normalized.is_empty()).then_some(normalized)
}

async fn resolve_delegate_request_overrides(
    agent_name: Option<String>,
    allowed_tools: Option<Vec<String>>,
    explicit_model: Option<String>,
) -> Result<DelegateRequestOverrides, ApiError> {
    let agent_name = agent_name.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    });
    let explicit_allowed_tools = allowed_tools.and_then(normalize_tool_ids);

    let Some(agent_name) = agent_name else {
        return Ok(DelegateRequestOverrides {
            model: explicit_model,
            allowed_tools: explicit_allowed_tools,
            ..DelegateRequestOverrides::default()
        });
    };

    let Some(config) = agent::get(&agent_name).await else {
        return Err(ApiError::bad_request(format!("unknown agent: {}", agent_name)));
    };

    // 显式 model 优先于 agent 绑定模型；agent 只提供默认补全。
    let model = if explicit_model.is_some() {
        explicit_model
    } else {
        agent::resolve_model_ref(&config)
            .map(|model_ref| format!("{}/{}", model_ref.provider_id, model_ref.model_id))
    };

    Ok(DelegateRequestOverrides {
        agent: Some(agent_name),
        model,
        allowed_tools: explicit_allowed_tools.or_else(|| normalize_tool_ids(config.allowed_tools)),
        chat_system_prompt: config
            .system_prompt
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        temperature: config.temperature,
        top_p: config.top_p,
    })
}

async fn start_gateway_chat_stream(
    dir: String,
    body: GatewayChatStreamRequest,
) -> Result<UnboundedReceiver<Value>, ApiError> {
    let GatewayChatStreamRequest {
        session_id,
        messages,
        system: _,
        model,
        agent,
        allowed_tools,
        acp_agent,
        acp_allowed_tools,
        options,
    } = body;

    let delegate_overrides =
        resolve_delegate_request_overrides(agent, allowed_tools, model).await?;
    let model = delegate_overrides.model.clone();
    let options = merge_gateway_request_options(
        options,
        delegate_overrides.agent,
        delegate_overrides.allowed_tools,
        delegate_overrides.chat_system_prompt,
        delegate_overrides.temperature,
        delegate_overrides.top_p,
        acp_agent,
        acp_allowed_tools,
    );
    let resume_history_only =
        options.get("desktop_resume_history_only").and_then(Value::as_bool).unwrap_or(false);
    let (tx, rx) = mpsc::unbounded_channel::<Value>();
    let approval_manager = approval_state::approval_manager_for_current_instance().await;

    let history_messages = messages;
    let root_dir = dir;
    let stream_session_id = session_id.as_ref().map(|value| value.0.clone()).unwrap_or_default();
    let stream_message_ids = if stream_session_id.trim().is_empty() {
        None
    } else {
        preallocate_stream_turn_message_ids().ok()
    };

    std::thread::spawn(move || {
        let total_messages = history_messages.len();
        let mut history = Vec::new();
        let mut query = String::new();
        let mut assistant_text = String::new();
        let mut assistant_finish_reason: Option<String> = None;
        let mut assistant_model = model.clone();

        for (idx, message) in history_messages.into_iter().enumerate() {
            let role = message.get("role").and_then(Value::as_str).unwrap_or("user");
            let content =
                message.get("content").and_then(Value::as_str).unwrap_or_default().to_string();

            let normalized_content =
                if role == "tool" { tool_content_to_session_text(&content) } else { content };

            if idx + 1 == total_messages && role == "user" {
                query = normalized_content;
                continue;
            }

            let mapped_role = match role {
                "assistant" => ui_models::ChatRole::Assistant,
                "system" => ui_models::ChatRole::System,
                "tool" => ui_models::ChatRole::Tool,
                _ => ui_models::ChatRole::User,
            };

            history.push(ui_models::ChatMessage {
                role: mapped_role,
                content: normalized_content,
                think_timing: Vec::new(),
            });
        }

        if !resume_history_only
            && query.is_empty()
            && let Some(last_user_idx) = history
                .iter()
                .rposition(|message| matches!(message.role, ui_models::ChatRole::User))
        {
            query = history.remove(last_user_idx).content;
        }

        let approval_reply_target = if stream_session_id.trim().is_empty() {
            "desktop".to_string()
        } else {
            stream_session_id.clone()
        };
        let (approval_prompt_tx, _approval_prompt_rx) = tokio::sync::mpsc::unbounded_channel();

        agent_session::processor::run(
            agent_session::processor::Request {
                stream: 0,
                session: stream_session_id.clone(),
                query: query.clone(),
                root: Some(root_dir),
                model: model.clone(),
                options: options.clone(),
                approval: Some(approval_manager.clone()),
                channel_name: Some("desktop".to_string()),
                non_cli_approval_context: Some(crate::agent::loop_::NonCliApprovalContext {
                    sender: "desktop".to_string(),
                    reply_target: approval_reply_target,
                    prompt_tx: approval_prompt_tx,
                }),
                assistant_message_id: stream_message_ids
                    .as_ref()
                    .map(|ids| ids.assistant_id.clone()),
                history,
                persist_app_session_artifacts: false,
            },
            move |ev| {
                let payload = match ev {
                    agent_session::processor::StreamEvent::Delta(delta) => {
                        assistant_text.push_str(&delta);
                        serde_json::json!({ "type": "chat.delta", "delta": delta })
                    }
                    agent_session::processor::StreamEvent::StepStart {
                        step_index,
                        created_ms,
                        model,
                    } => {
                        serde_json::json!({
                            "type": "chat.step_start",
                            "step_index": step_index,
                            "created_ms": created_ms,
                            "model": model
                        })
                    }
                    agent_session::processor::StreamEvent::StepFinish {
                        step_index,
                        finished_ms,
                        usage,
                        finish_reason,
                        model,
                    } => {
                        if finish_reason.is_some() {
                            assistant_finish_reason = finish_reason.clone();
                        }
                        if model.is_some() {
                            assistant_model = model.clone();
                        }
                        serde_json::json!({
                            "type": "chat.step_finish",
                            "step_index": step_index,
                            "finished_ms": finished_ms,
                            "usage": usage,
                            "finish_reason": finish_reason,
                            "model": model
                        })
                    }
                    agent_session::processor::StreamEvent::PostToolRound { step_index } => {
                        serde_json::json!({
                            "type": "chat.post_tool_round",
                            "step_index": step_index
                        })
                    }
                    agent_session::processor::StreamEvent::Done(usage) => {
                        let (message_id, parent_message_id) = if !stream_session_id.is_empty()
                            && !(resume_history_only && query.trim().is_empty())
                        {
                            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                let stream_message_ids = stream_message_ids.clone();
                                match tokio::task::block_in_place(|| {
                                    handle.block_on(persist_stream_chat_turn(
                                        &stream_session_id,
                                        &query,
                                        &assistant_text,
                                        assistant_model.as_deref(),
                                        &usage,
                                        assistant_finish_reason.as_deref(),
                                        stream_message_ids.as_ref(),
                                    ))
                                }) {
                                    Ok((assistant_id, user_id)) => {
                                        (Some(assistant_id), Some(user_id))
                                    }
                                    Err(_) => (None, None),
                                }
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };
                        serde_json::json!({
                            "type": "chat.done",
                            "usage": usage,
                            "message_id": message_id,
                            "parent_message_id": parent_message_id
                        })
                    }
                    agent_session::processor::StreamEvent::Error(err) => {
                        serde_json::json!({ "type": "chat.error", "error": err })
                    }
                };
                tx.send(payload).is_ok()
            },
        );
    });

    Ok(rx)
}

pub(crate) async fn collect_gateway_chat_stream(
    dir: String,
    body: GatewayChatStreamRequest,
) -> Result<GatewayChatStreamCollected, ApiError> {
    let mut rx = with_instance(dir.clone(), move || {
        Box::pin(async move { start_gateway_chat_stream(dir, body).await })
    })
    .await?;
    let mut output = String::new();
    let mut message_id = None;
    let mut parent_message_id = None;

    while let Some(payload) = rx.recv().await {
        match payload.get("type").and_then(Value::as_str) {
            Some("chat.delta") => {
                if let Some(delta) = payload.get("delta").and_then(Value::as_str) {
                    output.push_str(delta);
                }
            }
            Some("chat.done") => {
                message_id = payload.get("message_id").and_then(Value::as_str).map(str::to_string);
                parent_message_id =
                    payload.get("parent_message_id").and_then(Value::as_str).map(str::to_string);
                return Ok(GatewayChatStreamCollected { output, message_id, parent_message_id });
            }
            Some("chat.error") => {
                let error = payload
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("gateway chat stream failed");
                return Err(ApiError::internal(error.to_string()));
            }
            _ => {}
        }
    }

    Ok(GatewayChatStreamCollected { output, message_id, parent_message_id })
}

/// 处理桌面聊天流式请求。
///
/// # 参数
///
/// * `query` - 实例目录查询。
/// * `headers` - 可携带实例目录 header。
/// * `body` - 聊天历史、模型、agent 与 options。
///
/// # 返回值
///
/// 返回 `text/event-stream` 响应，事件类型包括 delta、step_start、step_finish、
/// post_tool_round、done 和 error。
///
/// # 错误处理
///
/// 实例解析、agent 解析或 processor 初始化失败时返回 [`ApiError`]；流内错误会编码为
/// `chat.error` 事件发送给前端。
pub(super) async fn chat_stream(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
    Json(body): Json<GatewayChatStreamRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let dir = resolve_directory(&query, &headers);
    let sse = with_instance(dir.clone(), move || {
        Box::pin(async move {
            let rx = start_gateway_chat_stream(dir, body).await?;
            let stream = UnboundedReceiverStream::new(rx).map(|payload| {
                let frame = format!("data: {}\n\n", payload);
                Ok::<Bytes, Infallible>(Bytes::from(frame))
            });
            let response = axum::response::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/event-stream")
                .header(header::CACHE_CONTROL, "no-cache")
                .header(header::CONNECTION, "keep-alive")
                .body(Body::from_stream(stream))
                .expect("failed to build chat stream response");

            Ok(response)
        })
    })
    .await?;

    Ok(sse)
}

/// 拆分流式请求中的模型引用。
///
/// # 参数
///
/// * `model` - 可选模型引用，通常形如 `provider/model`。
///
/// # 返回值
///
/// 返回 session 消息可保存的 provider/model 结构；空值返回空引用。
pub(super) fn split_stream_model_ref(model: Option<&str>) -> agent_session::message::ModelRef {
    let Some(model) = model.map(str::trim).filter(|value| !value.is_empty()) else {
        return agent_session::message::ModelRef {
            provider_id: String::new(),
            model_id: String::new(),
        };
    };

    let parsed = provider::parse_model(model);
    if parsed.model_id.is_empty() {
        agent_session::message::ModelRef {
            provider_id: String::new(),
            model_id: parsed.provider_id,
        }
    } else {
        agent_session::message::ModelRef {
            provider_id: parsed.provider_id,
            model_id: parsed.model_id,
        }
    }
}

fn token_info_from_usage(usage: &ui_models::TokenUsage) -> agent_session::message::TokenInfo {
    agent_session::message::TokenInfo {
        total: Some(usage.input_tokens + usage.output_tokens + usage.cached_tokens),
        input: usage.input_tokens,
        output: usage.output_tokens,
        reasoning: usage.reasoning_tokens,
        cache: agent_session::message::TokenCacheInfo { read: usage.cached_tokens, write: 0 },
    }
}

/// 持久化一轮流式聊天的 user/assistant 消息。
///
/// # 参数
///
/// * `session_id` - 目标会话 id。
/// * `query` - 本轮用户输入。
/// * `assistant_text` - assistant 最终文本。
/// * `model` - 可选模型引用。
/// * `usage` - token 使用量。
/// * `finish_reason` - 可选完成原因。
/// * `preallocated_ids` - 可选预分配消息 id。
///
/// # 返回值
///
/// 返回 `(assistant_id, user_id)`，用于 done 事件回传。
///
/// # 错误处理
///
/// id 生成或消息/片段写入失败时返回 [`ApiError`]。
pub(super) async fn persist_stream_chat_turn(
    session_id: &str,
    query: &str,
    assistant_text: &str,
    model: Option<&str>,
    usage: &ui_models::TokenUsage,
    finish_reason: Option<&str>,
    preallocated_ids: Option<&StreamTurnMessageIds>,
) -> Result<(String, String), ApiError> {
    let model_ref = split_stream_model_ref(model);
    let now = agent_session::session::now_ms();

    let user_id = match preallocated_ids {
        Some(ids) => ids.user_id.clone(),
        None => id::ascending(id::Prefix::Message, None)
            .map_err(|err| ApiError::internal(err.to_string()))?,
    };
    let user_part_id =
        id::ascending(id::Prefix::Part, None).map_err(|err| ApiError::internal(err.to_string()))?;
    let user_info =
        agent_session::message::Info::User(Box::new(agent_session::message::UserInfo {
            id: user_id.clone(),
            session_id: session_id.to_string(),
            time: agent_session::message::UserTime { created: now },
            summary: None,
            agent: "gateway".to_string(),
            model: model_ref.clone(),
            system: None,
            tools: None,
            variant: None,
        }));
    let user_part = agent_session::message::Part::Text(agent_session::message::TextPart {
        base: agent_session::message::PartBase {
            id: user_part_id,
            session_id: session_id.to_string(),
            message_id: user_id.clone(),
        },
        text: query.to_string(),
        synthetic: None,
        ignored: None,
        time: Some(agent_session::message::PartTime { start: now, end: Some(now) }),
        metadata: None,
    });

    let assistant_id = match preallocated_ids {
        Some(ids) => ids.assistant_id.clone(),
        None => id::ascending(id::Prefix::Message, None)
            .map_err(|err| ApiError::internal(err.to_string()))?,
    };
    let assistant_part_id =
        id::ascending(id::Prefix::Part, None).map_err(|err| ApiError::internal(err.to_string()))?;
    let assistant_info =
        agent_session::message::Info::Assistant(Box::new(agent_session::message::AssistantInfo {
            id: assistant_id.clone(),
            session_id: session_id.to_string(),
            time: agent_session::message::AssistantTime { created: now, completed: Some(now) },
            error: None,
            parent_id: user_id.clone(),
            model_id: model_ref.model_id,
            provider_id: model_ref.provider_id,
            mode: "chat".to_string(),
            agent: "gateway".to_string(),
            path: agent_session::message::PathInfo {
                cwd: instance::directory(),
                root: instance::worktree(),
            },
            summary: None,
            cost: 0.0,
            tokens: token_info_from_usage(usage),
            variant: None,
            finish: finish_reason.map(str::to_string),
        }));
    let assistant_part = agent_session::message::Part::Text(agent_session::message::TextPart {
        base: agent_session::message::PartBase {
            id: assistant_part_id,
            session_id: session_id.to_string(),
            message_id: assistant_id.clone(),
        },
        text: assistant_text.to_string(),
        synthetic: None,
        ignored: None,
        time: Some(agent_session::message::PartTime { start: now, end: Some(now) }),
        metadata: None,
    });

    agent_session::message::update_message(&user_info)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    agent_session::message::update_part(&user_part)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    agent_session::message::update_message(&assistant_info)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    agent_session::message::update_part(&assistant_part)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;

    Ok((assistant_id, user_id))
}

fn preallocate_stream_turn_message_ids() -> Result<StreamTurnMessageIds, ApiError> {
    let assistant_id = id::ascending(id::Prefix::Message, None)
        .map_err(|err| ApiError::internal(err.to_string()))?;
    let user_id = id::ascending(id::Prefix::Message, None)
        .map_err(|err| ApiError::internal(err.to_string()))?;
    Ok(StreamTurnMessageIds { assistant_id, user_id })
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod stream_tests;
