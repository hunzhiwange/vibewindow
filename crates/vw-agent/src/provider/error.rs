//! Provider 错误解析与归一化。
//!
//! 上游模型服务的错误格式不稳定，本模块把 API 调用错误和流式错误转换成前端更容易
//! 消费的结构：区分上下文溢出、普通 API 错误、是否可重试，并保留必要的响应信息。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 原始 API 调用错误。
///
/// 该结构保留 provider 边界返回的主要字段，供后续解析函数判断展示文案和重试语义。
pub struct ApiCallError {
    /// 原始错误消息。
    pub message: String,
    /// HTTP 状态码。
    #[serde(default)]
    pub status_code: Option<u16>,
    /// 上游或传输层给出的可重试标记。
    #[serde(default)]
    pub is_retryable: bool,
    /// 触发错误的请求 URL；解析后只作为诊断 metadata 暴露。
    #[serde(default)]
    pub url: Option<String>,
    /// 响应头快照。
    #[serde(default)]
    pub response_headers: Option<HashMap<String, String>>,
    /// 响应体文本。
    #[serde(default)]
    pub response_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
/// 从流式响应事件中解析出的错误。
pub enum ParsedStreamError {
    /// 输入超过模型上下文窗口。
    ContextOverflow { message: String, response_body: String },
    /// 普通 API 错误。
    ApiError { message: String, is_retryable: bool, response_body: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
/// 从 API 调用失败中解析出的错误。
pub enum ParsedApiCallError {
    /// 输入超过模型上下文窗口。
    ContextOverflow {
        /// 面向用户的错误消息。
        message: String,
        /// 原始响应体，供诊断使用。
        response_body: Option<String>,
    },
    /// 普通 API 错误。
    ApiError {
        /// 面向用户的错误消息。
        message: String,
        #[serde(default)]
        /// HTTP 状态码。
        status_code: Option<u16>,
        /// 是否建议调用方重试。
        is_retryable: bool,
        #[serde(default)]
        /// 响应头快照。
        response_headers: Option<HashMap<String, String>>,
        #[serde(default)]
        /// 原始响应体。
        response_body: Option<String>,
        #[serde(default)]
        /// 额外诊断字段。
        metadata: Option<HashMap<String, String>>,
    },
}

fn overflow_patterns() -> &'static [&'static str] {
    &[
        "prompt is too long",
        "input is too long for requested model",
        "exceeds the context window",
        "input token count",
        "maximum prompt length is",
        "reduce the length of the messages",
        "maximum context length is",
        "exceeds the available context size",
        "greater than the context length",
        "context window exceeds limit",
        "exceeded model token limit",
        "context_length_exceeded",
    ]
}

fn is_overflow(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    if overflow_patterns().iter().any(|p| msg.contains(p)) {
        return true;
    }
    // 有些网关在 body 缺失时只留下状态码摘要，仍需把 400/413 的空体情况归为上下文溢出。
    if msg.starts_with("400") && msg.contains("(no body)") {
        return true;
    }
    msg.starts_with("413") && msg.contains("(no body)")
}

fn is_openai_error_retryable(e: &ApiCallError) -> bool {
    let Some(status) = e.status_code else {
        return e.is_retryable;
    };
    // OpenAI 兼容网关中 404 常见于模型路由短暂不可用，保留重试机会。
    status == 404 || e.is_retryable
}

fn transform_message(provider_id: &str, message: &str, status_code: Option<u16>) -> String {
    if provider_id.contains("github-copilot") && status_code == Some(403) {
        return "请重新对 copilot provider 进行认证，以确保凭据可用。".to_string();
    }
    message.to_string()
}

fn json(input: &str) -> Option<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(input)
        .ok()
        .and_then(|v| if v.is_object() { Some(v) } else { None })
}

/// 解析流式响应中的错误事件。
///
/// 只接受 `{"type":"error"}` 形态的 JSON 对象；不能识别的事件返回 `None`，由调用方
/// 继续按普通流事件处理。
///
/// # 返回值
///
/// 识别成功时返回归一化错误，否则返回 `None`。
pub fn parse_stream_error(input: &str) -> Option<ParsedStreamError> {
    let body = json(input)?;
    if body.get("type")?.as_str()? != "error" {
        return None;
    }
    let response_body = serde_json::to_string(&body).ok()?;
    let code = body.get("error").and_then(|e| e.get("code")).and_then(|v| v.as_str());
    match code {
        Some("context_length_exceeded") => Some(ParsedStreamError::ContextOverflow {
            message: "输入超出该模型的上下文窗口".to_string(),
            response_body,
        }),
        Some("insufficient_quota") => Some(ParsedStreamError::ApiError {
            message: "配额已用尽，请检查套餐与计费信息。".to_string(),
            is_retryable: false,
            response_body,
        }),
        Some("usage_not_included") => Some(ParsedStreamError::ApiError {
            message:
                "要使用 Codex 与 ChatGPT 套餐，请升级到 Plus：https://chatgpt.com/explore/plus。"
                    .to_string(),
            is_retryable: false,
            response_body,
        }),
        Some("invalid_prompt") => Some(ParsedStreamError::ApiError {
            message: body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("无效的 prompt。")
                .to_string(),
            is_retryable: false,
            response_body,
        }),
        _ => None,
    }
}

/// 解析并归一化 API 调用错误。
///
/// 会补齐空消息、转换特定 provider 的认证提示、识别上下文溢出，并按 provider 策略
/// 计算可重试标记。
///
/// # 返回值
///
/// 返回可序列化给前端的归一化错误。
pub fn parse_api_call_error(provider_id: &str, error: ApiCallError) -> ParsedApiCallError {
    let msg0 = if error.message.trim().is_empty() {
        error
            .response_body
            .clone()
            .or_else(|| error.status_code.map(|s| s.to_string()))
            .unwrap_or_else(|| "Unknown error".to_string())
    } else {
        error.message.clone()
    };
    let msg = transform_message(provider_id, &msg0, error.status_code);
    if is_overflow(&msg) {
        return ParsedApiCallError::ContextOverflow {
            message: msg,
            response_body: error.response_body,
        };
    }
    let metadata = error.url.clone().map(|u| HashMap::from([("url".to_string(), u)]));
    ParsedApiCallError::ApiError {
        message: msg,
        status_code: error.status_code,
        is_retryable: if provider_id.starts_with("openai") {
            is_openai_error_retryable(&error)
        } else {
            error.is_retryable
        },
        response_headers: error.response_headers,
        response_body: error.response_body,
        metadata,
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
