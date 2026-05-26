//! 网关 HTTP 辅助函数。
//!
//! 本模块聚焦于可复用的底层能力：
//! - 认证参数注入
//! - 查询参数规范化
//! - 成功/失败响应日志
//! - 统一错误文本拼装
//!
//! 业务 API 应尽量通过这些辅助函数保持日志格式与错误表现一致。

use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{debug, error, info, warn};

use crate::endpoint::GatewayEndpoint;

/// 为异步请求构造器附加 Basic Auth 与 x-skey 认证头。
///
/// 当认证字段为空或仅包含空白字符时，会自动跳过对应认证项。
pub fn apply_auth(
    builder: reqwest::RequestBuilder,
    endpoint: &GatewayEndpoint,
) -> reqwest::RequestBuilder {
    let Some(auth) = endpoint.auth.as_ref() else {
        return builder;
    };

    let builder = match auth.bearer_token.as_deref().filter(|value| !value.trim().is_empty()) {
        Some(token) => builder.bearer_auth(token),
        None => match auth.password.as_deref().filter(|value| !value.trim().is_empty()) {
            Some(password) => builder.basic_auth(
                auth.username
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("vibewindow"),
                Some(password),
            ),
            None => builder,
        },
    };

    match auth.skey.as_deref().filter(|value| !value.trim().is_empty()) {
        Some(skey) => builder.header("x-skey", skey),
        None => builder,
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 为阻塞请求构造器附加 Basic Auth 与 x-skey 认证头。
///
/// 行为与异步版本保持一致，只是作用于阻塞客户端。
pub fn apply_blocking_auth(
    builder: reqwest::blocking::RequestBuilder,
    endpoint: &GatewayEndpoint,
) -> reqwest::blocking::RequestBuilder {
    let Some(auth) = endpoint.auth.as_ref() else {
        return builder;
    };

    let builder = match auth.bearer_token.as_deref().filter(|value| !value.trim().is_empty()) {
        Some(token) => builder.bearer_auth(token),
        None => match auth.password.as_deref().filter(|value| !value.trim().is_empty()) {
            Some(password) => builder.basic_auth(
                auth.username
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("vibewindow"),
                Some(password),
            ),
            None => builder,
        },
    };

    match auth.skey.as_deref().filter(|value| !value.trim().is_empty()) {
        Some(skey) => builder.header("x-skey", skey),
        None => builder,
    }
}

#[allow(dead_code)]
/// 非流式请求默认超时时间，单位为秒。
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// 将可选目录参数转换为网关接口约定的查询参数。
///
/// 只有在目录非空时才会附加 `directory` 查询参数，避免把空字符串传给后端。
pub fn directory_query(directory: Option<&str>) -> Vec<(String, String)> {
    directory
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![("directory".to_string(), value.to_string())])
        .unwrap_or_default()
}

/// 解析 JSON 响应，并统一记录成功或解码失败日志。
///
/// 该函数假设调用方已经完成请求发送，仅负责对响应状态与 JSON 反序列化阶段做统一处理。
pub async fn parse_json_response<T: DeserializeOwned>(
    method: &str,
    endpoint: &GatewayEndpoint,
    path: &str,
    response: reqwest::Response,
) -> Result<T, String> {
    if !response.status().is_success() {
        return Err(response_error(method, endpoint, path, response).await);
    }
    info!(
        target: "vw_gateway_client",
        method = method,
        endpoint = %endpoint.describe(),
        path = path,
        "gateway request succeeded"
    );
    response.json().await.map_err(|err| {
        let msg = err.to_string();
        error!(
            target: "vw_gateway_client",
            method = method,
            endpoint = %endpoint.describe(),
            path = path,
            error = %msg,
            "gateway response JSON decode failed"
        );
        msg
    })
}

/// 将非成功响应转换为统一错误文本，便于上层直接展示。
///
/// 返回值不会携带结构化错误类型，目的是让上层无需额外转换即可直接显示给用户或记录到日志。
pub async fn response_error(
    method: &str,
    endpoint: &GatewayEndpoint,
    path: &str,
    response: reqwest::Response,
) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    warn!(
        target: "vw_gateway_client",
        method = method,
        endpoint = %endpoint.describe(),
        path = path,
        status = %status,
        response_body = %body.trim(),
        "gateway request returned non-success status"
    );
    if body.trim().is_empty() {
        format!("gateway request failed: {status}")
    } else {
        format!("gateway request failed: {status} {}", body.trim())
    }
}

/// 记录请求基础信息，便于排查网关交互问题。
///
/// 当前日志内容包含方法、端点、路径、查询参数与请求体 JSON 文本。
pub fn log_request<B: Serialize>(
    method: &str,
    endpoint: &GatewayEndpoint,
    path: &str,
    query: &[(String, String)],
    body: Option<&B>,
) {
    let body_json = body.and_then(|value| serde_json::to_string(value).ok()).unwrap_or_default();
    debug!(
        target: "vw_gateway_client",
        method = method,
        endpoint = %endpoint.describe(),
        path = path,
        query = %format_query(query),
        body = %body_json,
        "sending gateway request"
    );
}

/// 将非成功响应转换为统一错误文本（阻塞版本）。
#[cfg(not(target_arch = "wasm32"))]
pub fn response_error_blocking(
    method: &str,
    endpoint: &GatewayEndpoint,
    path: &str,
    response: reqwest::blocking::Response,
) -> String {
    let status = response.status();
    let body = response.text().unwrap_or_default();
    warn!(
        target: "vw_gateway_client",
        method = method,
        endpoint = %endpoint.describe(),
        path = path,
        status = %status,
        response_body = %body.trim(),
        "gateway request returned non-success status"
    );
    if body.trim().is_empty() {
        format!("gateway request failed: {status}")
    } else {
        format!("gateway request failed: {status} {}", body.trim())
    }
}

/// 将传输层错误转换为字符串，并补充统一日志。
///
/// 这里的“传输层错误”包括连接失败、超时、TLS/协议错误等 `reqwest` 发送阶段异常。
pub fn transport_error(
    method: &str,
    endpoint: &GatewayEndpoint,
    path: &str,
    err: reqwest::Error,
) -> String {
    let msg = err.to_string();
    error!(
        target: "vw_gateway_client",
        method = method,
        endpoint = %endpoint.describe(),
        path = path,
        error = %msg,
        "gateway transport error"
    );
    msg
}

fn format_query(query: &[(String, String)]) -> String {
    if query.is_empty() {
        String::new()
    } else {
        query.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join("&")
    }
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod http_tests;
