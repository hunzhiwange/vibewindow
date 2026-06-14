//! AI SDK 适配层的消息、工具和流式事件辅助函数。
//!
//! 这些工具负责在 OpenAI 兼容消息结构、VibeWindow UI 模型和 `aisdk` 核心类型之间做
//! 局部转换。所有逻辑都限制在非 wasm 目标下，因为当前 AI SDK 后端依赖原生运行时能力。

#[cfg(not(target_arch = "wasm32"))]
use aisdk::core::ToolCallInfo as AiToolCallInfo;
#[cfg(not(target_arch = "wasm32"))]
use aisdk::core::ToolResultInfo as AiToolResultInfo;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::{Map, Value};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use crate::app::agent::provider::provider;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::agent::session::llm::types::Error;
#[cfg(not(target_arch = "wasm32"))]
use crate::session::ui_types as models;

#[cfg(not(target_arch = "wasm32"))]
/// 将 JSON schema 规范化为严格对象模式。
///
/// 参数会被原地修改：对象 schema 会设置 `additionalProperties=false`，并把所有
/// properties 键写入 `required`。该函数递归处理嵌套对象、数组和组合 schema。
pub(crate) fn normalize_strict_object_required(schema: &mut Value) {
    let Some(obj) = schema.as_object_mut() else { return };

    let is_object = obj.get("type").and_then(Value::as_str) == Some("object")
        || obj.get("properties").and_then(Value::as_object).is_some();

    if is_object {
        obj.insert("additionalProperties".to_string(), Value::Bool(false));
    }

    if let Some(props_obj) = obj.get("properties").and_then(Value::as_object) {
        let mut required: Vec<String> = props_obj.keys().cloned().collect();
        required.sort();
        obj.insert(
            "required".to_string(),
            Value::Array(required.into_iter().map(Value::String).collect()),
        );
    }

    // 组合 schema 中的子项同样需要严格化，否则模型仍可能从嵌套分支输出未声明字段。
    if let Some(props) = obj.get_mut("properties").and_then(Value::as_object_mut) {
        for (_, v) in props.iter_mut() {
            normalize_strict_object_required(v);
        }
    }

    if let Some(v) = obj.get_mut("items") {
        normalize_strict_object_required(v);
    }
    for key in ["anyOf", "allOf", "oneOf"] {
        if let Some(arr) = obj.get_mut(key).and_then(Value::as_array_mut) {
            for item in arr.iter_mut() {
                normalize_strict_object_required(item);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 合并 assistant 文本和 DeepSeek 风格 reasoning 字段。
///
/// 返回值为空表示两类内容都不存在；当 reasoning 存在时会用显式标签隔开，避免和普通
/// assistant 文本混淆。
pub(crate) fn assistant_text_with_reasoning(obj: &Map<String, Value>) -> String {
    let text = openai_message_content_to_text(obj.get("content"));
    let reasoning =
        obj.get("reasoning_content").and_then(Value::as_str).unwrap_or_default().trim().to_string();
    match (text.trim().is_empty(), reasoning.is_empty()) {
        (true, true) => String::new(),
        (false, true) => text,
        (true, false) => format!("[reasoning]\n{}", reasoning),
        (false, false) => format!("{}\n\n[reasoning]\n{}", text, reasoning),
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 将 OpenAI message content 转成纯文本。
///
/// 支持字符串和 `[{ type: "text", text: ... }]` 数组；其他内容块会被跳过。
pub(crate) fn openai_message_content_to_text(content: Option<&Value>) -> String {
    let Some(content) = content else { return String::new() };
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    let Some(arr) = content.as_array() else { return String::new() };
    let mut out = String::new();
    for part in arr {
        let Some(ty) = part.get("type").and_then(Value::as_str) else { continue };
        if ty != "text" {
            continue;
        }
        if let Some(t) = part.get("text").and_then(Value::as_str) {
            out.push_str(t);
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
/// 将 OpenAI 兼容消息数组转换为 AI SDK 消息列表。
///
/// `messages` 不是数组时返回空列表；无法识别的消息或空文本会被跳过。DeepSeek reasoner
/// 不支持标准工具消息链路时，会把工具结果降级为用户文本，从而保持上下文可读。
pub(crate) fn openai_messages_to_aisdk_messages(
    messages: &Value,
    model: &provider::Model,
) -> Result<Vec<aisdk::core::Message>, Error> {
    let Some(arr) = messages.as_array() else {
        return Ok(Vec::new());
    };
    let mut out: Vec<aisdk::core::Message> = Vec::with_capacity(arr.len());
    let mut tool_name_by_id: HashMap<String, String> = HashMap::new();
    let is_deepseek_reasoner = model.api.id.contains("deepseek-reasoner")
        || model.provider_id.to_ascii_lowercase().contains("deepseek");

    for msg in arr {
        let Some(obj) = msg.as_object() else { continue };
        let role = obj.get("role").and_then(Value::as_str).unwrap_or_default();

        if role == "assistant" && !is_deepseek_reasoner {
            if let Some(tool_calls) = obj.get("tool_calls").and_then(Value::as_array) {
                for tc in tool_calls {
                    let Some(fn_obj) = tc.get("function").and_then(Value::as_object) else {
                        continue;
                    };
                    let id = tc.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
                    let name =
                        fn_obj.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
                    let args_str = fn_obj
                        .get("arguments")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if !id.trim().is_empty() && !name.trim().is_empty() {
                        tool_name_by_id.insert(id.clone(), name.clone());
                    }
                    let mut tool_info = AiToolCallInfo::new(name);
                    tool_info.id(id);
                    let input = serde_json::from_str::<Value>(&args_str)
                        .unwrap_or_else(|_| Value::Object(Map::new()));
                    tool_info.input(input);
                    out.push(aisdk::core::Message::Assistant(aisdk::core::AssistantMessage {
                        content:
                            aisdk::core::language_model::LanguageModelResponseContentType::ToolCall(
                                tool_info,
                            ),
                        usage: None,
                    }));
                }
            }
        }

        match role {
            "system" => {
                let text = openai_message_content_to_text(obj.get("content"));
                if !text.trim().is_empty() {
                    out.push(aisdk::core::Message::System(text.into()));
                }
            }
            "user" => {
                let text = openai_message_content_to_text(obj.get("content"));
                if !text.trim().is_empty() {
                    out.push(aisdk::core::Message::User(text.into()));
                }
            }
            "assistant" => {
                let text = if is_deepseek_reasoner {
                    assistant_text_with_reasoning(obj)
                } else {
                    openai_message_content_to_text(obj.get("content"))
                };
                if !text.trim().is_empty() {
                    out.push(aisdk::core::Message::Assistant(text.into()));
                }
            }
            "tool" => {
                let id = obj.get("tool_call_id").and_then(Value::as_str).unwrap_or_default();
                let content = openai_message_content_to_text(obj.get("content"));
                if is_deepseek_reasoner {
                    // DeepSeek reasoner 会把 reasoning 和文本放在同一消息链路里；将工具结果
                    // 转为用户文本可避免构造它不理解的 Tool message，同时保留执行结果。
                    let text = if id.trim().is_empty() {
                        format!("[tool_result]\n{}", content)
                    } else {
                        format!("[tool_result:{}]\n{}", id, content)
                    };
                    out.push(aisdk::core::Message::User(text.into()));
                    continue;
                }
                let mut tool_result =
                    AiToolResultInfo::new(tool_name_by_id.get(id).cloned().unwrap_or_default());
                tool_result.id(id.to_string());
                let output = serde_json::from_str::<Value>(&content)
                    .unwrap_or_else(|_| Value::String(content));
                tool_result.output(output);
                out.push(aisdk::core::Message::Tool(tool_result));
            }
            "developer" => {
                let text = openai_message_content_to_text(obj.get("content"));
                if !text.trim().is_empty() {
                    out.push(aisdk::core::Message::Developer(text));
                }
            }
            _ => {}
        }
    }

    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
/// 从 JSON 值解析 stop sequences。
///
/// 字符串会转为单元素列表，字符串数组会过滤空白项；其他类型返回 `None`。
pub(crate) fn stop_sequences_from_value(v: &Value) -> Option<Vec<String>> {
    match v {
        Value::String(s) => {
            let s = s.trim();
            if s.is_empty() { None } else { Some(vec![s.to_string()]) }
        }
        Value::Array(arr) => {
            let out = arr
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            if out.is_empty() { None } else { Some(out) }
        }
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 从 OpenAI 兼容 usage 对象提取 UI token 用量。
///
/// 同时兼容传统 `prompt_tokens`/`completion_tokens` 和新式 `input_tokens`/`output_tokens`
/// 字段。缺失字段按 0 处理。
pub(crate) fn token_usage_from_openai_usage(v: &Value) -> models::TokenUsage {
    models::TokenUsage {
        input_tokens: v
            .get("prompt_tokens")
            .or_else(|| v.get("input_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        output_tokens: v
            .get("completion_tokens")
            .or_else(|| v.get("output_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        cached_tokens: v
            .pointer("/prompt_tokens_details/cached_tokens")
            .or_else(|| v.pointer("/input_tokens_details/cached_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        reasoning_tokens: v
            .pointer("/completion_tokens_details/reasoning_tokens")
            .or_else(|| v.get("reasoning_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 从 AI SDK usage 结构提取 UI token 用量。
pub(crate) fn token_usage_from_aisdk_usage(
    u: &aisdk::core::language_model::Usage,
) -> models::TokenUsage {
    models::TokenUsage {
        input_tokens: u.input_tokens.unwrap_or(0) as i64,
        output_tokens: u.output_tokens.unwrap_or(0) as i64,
        cached_tokens: u.cached_tokens.unwrap_or(0) as i64,
        reasoning_tokens: u.reasoning_tokens.unwrap_or(0) as i64,
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 从缓冲区中取出下一帧 SSE 事件。
///
/// 支持 `\r\n\r\n` 和 `\n\n` 两种帧分隔符；成功取出时会从缓冲区移除对应内容。
pub(crate) fn take_next_sse_event(buffer: &mut String) -> Option<String> {
    if let Some(pos) = buffer.find("\r\n\r\n") {
        let frame = buffer[..pos].to_string();
        buffer.drain(..pos + 4);
        return Some(frame);
    }
    if let Some(pos) = buffer.find("\n\n") {
        let frame = buffer[..pos].to_string();
        buffer.drain(..pos + 2);
        return Some(frame);
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
/// 流式工具调用的增量聚合状态。
#[derive(Default)]
pub(crate) struct StreamingToolCallState {
    /// 工具调用 id。
    pub(crate) id: String,
    /// 工具函数名。
    pub(crate) name: String,
    /// 流式拼接后的 JSON 参数文本。
    pub(crate) arguments: String,
}

#[cfg(not(target_arch = "wasm32"))]
/// 合并 OpenAI 流式工具调用 delta。
///
/// `states` 会按 `idx` 自动扩展；缺少 id 时使用 `fallback_idx` 生成稳定占位 id。
pub(crate) fn merge_tool_call_delta(
    states: &mut Vec<StreamingToolCallState>,
    idx: usize,
    call: &Value,
    fallback_idx: usize,
) {
    if states.len() <= idx {
        states.resize_with(idx + 1, StreamingToolCallState::default);
    }
    let state = &mut states[idx];
    if let Some(id) = call.get("id").and_then(Value::as_str)
        && !id.trim().is_empty()
    {
        state.id = id.to_string();
    }
    if let Some(function) = call.get("function").and_then(Value::as_object) {
        if let Some(name) = function.get("name").and_then(Value::as_str)
            && !name.trim().is_empty()
        {
            state.name = name.to_string();
        }
        if let Some(args) = function.get("arguments").and_then(Value::as_str)
            && !args.is_empty()
        {
            state.arguments.push_str(args);
        }
    }
    if state.id.trim().is_empty() {
        state.id = format!("call_{}", fallback_idx + 1);
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 检查请求取消信号是否已经触发。
pub(crate) fn should_abort(rx: Option<&tokio::sync::watch::Receiver<bool>>) -> bool {
    rx.is_some_and(|r| *r.borrow())
}
#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "util_tests.rs"]
mod util_tests;
