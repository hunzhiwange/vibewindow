//! ACP 客户端辅助函数。
//!
//! 该模块集中放置进程清理、环境变量认证、会话选项 meta 构造、错误摘要
//! 格式化和用量映射等局部工具，避免主客户端和 actor 循环承担过多细节。

use super::*;

/// 清理代理进程组。
///
/// 会先发送温和终止信号并短暂等待，再发送强制结束信号；非 Unix 平台下信号
/// 函数为空实现，因此该函数主要依赖子进程自身退出。
pub(super) async fn cleanup_process_group(process_group_id: Option<u32>) {
    send_terminate_signal_to_process_group(process_group_id);
    tokio::time::sleep(Duration::from_millis(150)).await;
    send_kill_signal_to_process_group(process_group_id);
}

/// 向 Unix 进程组发送 `SIGTERM`。
///
/// `process_group_id` 为 `None` 时不会执行任何操作；发送失败会被忽略，因为
/// 进程可能已经退出。
#[cfg(unix)]
pub(super) fn send_terminate_signal_to_process_group(process_group_id: Option<u32>) {
    let _ = send_signal_to_process_group(process_group_id, libc::SIGTERM);
}

/// 非 Unix 平台的 `SIGTERM` 占位实现。
#[cfg(not(unix))]
pub(super) fn send_terminate_signal_to_process_group(_process_group_id: Option<u32>) {}

/// 向 Unix 进程组发送 `SIGKILL`。
///
/// 用于温和终止超时后的兜底清理，确保代理子进程不会继续持有工作区资源。
#[cfg(unix)]
pub(super) fn send_kill_signal_to_process_group(process_group_id: Option<u32>) {
    let _ = send_signal_to_process_group(process_group_id, libc::SIGKILL);
}

/// 非 Unix 平台的 `SIGKILL` 占位实现。
#[cfg(not(unix))]
pub(super) fn send_kill_signal_to_process_group(_process_group_id: Option<u32>) {}

#[cfg(unix)]
fn send_signal_to_process_group(process_group_id: Option<u32>, signal: i32) -> bool {
    let Some(process_group_id) = process_group_id else {
        return false;
    };
    let result = unsafe { libc::kill(-(process_group_id as i32), signal) };
    if result == 0 {
        return true;
    }
    matches!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH))
}

/// 从子进程退出状态构造生命周期摘要。
///
/// 返回值包含退出码和 Unix 信号名；没有退出状态时字段为空。
pub(super) fn child_exit_summary(status: Option<&ExitStatus>) -> ChildExitSummary {
    ChildExitSummary {
        exit_code: status.and_then(ExitStatus::code),
        signal: exit_signal_name(status),
    }
}

#[cfg(unix)]
fn exit_signal_name(status: Option<&ExitStatus>) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;

    status.and_then(|value| value.signal()).map(|signal| format!("SIG{signal}"))
}

#[cfg(not(unix))]
fn exit_signal_name(_status: Option<&ExitStatus>) -> Option<String> {
    None
}

/// 将会话选项转换为 ACP `meta` 扩展字段。
///
/// 当前只在存在有效选项时返回 `Some`，避免向代理发送空扩展对象。空字符串工具
/// 名或模型名会被过滤。
pub(super) fn build_session_options_meta(options: Option<&AcpSessionOptions>) -> Option<acp::Meta> {
    let options = options?;
    let mut claude_code_options = Map::new();
    if let Some(model) = options.model.as_ref().filter(|value| !value.trim().is_empty()) {
        claude_code_options.insert("model".to_string(), Value::String(model.clone()));
    }
    if let Some(allowed_tools) = &options.allowed_tools {
        let allowed_tools = allowed_tools
            .iter()
            .filter_map(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then(|| Value::String(trimmed.to_string()))
            })
            .collect::<Vec<_>>();
        if !allowed_tools.is_empty() {
            claude_code_options.insert("allowedTools".to_string(), Value::Array(allowed_tools));
        }
    }
    if let Some(max_turns) = options.max_turns {
        claude_code_options.insert("maxTurns".to_string(), json!(max_turns));
    }
    if claude_code_options.is_empty() {
        return None;
    }

    let mut meta = Map::new();
    meta.insert(
        "claudeCode".to_string(),
        json!({
            "options": Value::Object(claude_code_options),
        }),
    );
    Some(meta)
}

/// 将客户端侧错误映射为 ACP 错误并维护权限统计。
///
/// 权限拒绝和权限提示不可用会分别计入 denied/cancelled，其余错误作为内部错误
/// 返回给代理。
pub(super) fn map_client_error(
    err: Box<dyn StdError + Send + Sync + 'static>,
    permission_stats: &Arc<Mutex<PermissionStats>>,
) -> acp::Error {
    if err.is::<crate::errors::PermissionDeniedError>() {
        permission_stats.lock().denied += 1;
    } else if err.is::<crate::errors::PermissionPromptUnavailableError>() {
        permission_stats.lock().cancelled += 1;
    }
    acp::Error::internal_error().data(err.to_string())
}

/// 根据认证凭据构造注入代理进程的环境变量。
///
/// 空凭据会被忽略；方法 ID 会以原始 key、规范化 key 和 `VWACP_AUTH_` 前缀 key
/// 尝试注入。包含 `=` 或 NUL 的原始 key 不会直接作为环境变量名使用，避免生成
/// 非法或含义不明的环境项。
pub(super) fn build_agent_environment(
    auth_credentials: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut env = HashMap::new();
    for (method_id, credential) in auth_credentials {
        if credential.trim().is_empty() {
            continue;
        }

        if !method_id.contains('=') && !method_id.contains('\0') {
            env.entry(method_id.clone()).or_insert_with(|| credential.clone());
        }

        if let Some(normalized) = to_env_token(method_id) {
            env.entry(format!("VWACP_AUTH_{normalized}")).or_insert_with(|| credential.clone());
            env.entry(normalized).or_insert_with(|| credential.clone());
        }
    }
    env
}

/// 构造 ACP 权限请求的取消响应。
///
/// 该响应用于本地取消中的会话，避免代理继续等待权限确认。
pub(super) fn cancelled_permission_response() -> acp::RequestPermissionResponse {
    serde_json::from_value(serde_json::json!({
        "outcome": {
            "outcome": "cancelled"
        }
    }))
    .expect("valid ACP permission cancellation response")
}

/// 从环境变量读取指定认证方法的凭据。
///
/// 会尝试原始方法 ID、规范化 token 和 `VWACP_AUTH_` 前缀形式。返回 `None`
/// 表示没有非空凭据。
pub(super) fn read_env_credential(method_id: &str) -> Option<String> {
    auth_env_keys(method_id)
        .into_iter()
        .find_map(|key| std::env::var(&key).ok().filter(|value| !value.trim().is_empty()))
}

fn auth_env_keys(method_id: &str) -> Vec<String> {
    let mut keys = vec![method_id.to_string()];
    if let Some(token) = to_env_token(method_id) {
        keys.push(token.clone());
        keys.push(format!("VWACP_AUTH_{token}"));
    }
    keys
}

/// 将任意认证方法 ID 转换为可用的环境变量 token。
///
/// 非 ASCII 字母数字字符会被替换为下划线，首尾下划线会被裁剪，最终转为大写。
/// 如果结果为空则返回 `None`。
pub(super) fn to_env_token(value: &str) -> Option<String> {
    let token = value
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_ascii_uppercase();

    if token.is_empty() { None } else { Some(token) }
}

/// 提取命令 basename 并转为小写 token。
pub(super) fn basename_token(command: &str) -> String {
    Path::new(command)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(command)
        .to_ascii_lowercase()
}

/// 解析 Gemini ACP 初始化超时时间。
///
/// 环境变量缺失、非数字或小于等于零时使用默认值。
pub(super) fn resolve_gemini_acp_startup_timeout() -> Duration {
    resolve_timeout_from_env("VWACP_GEMINI_ACP_STARTUP_TIMEOUT_MS", 15_000)
}

/// 解析 Claude ACP 会话创建超时时间。
///
/// 环境变量缺失、非数字或小于等于零时使用默认值。
pub(super) fn resolve_claude_acp_session_create_timeout() -> Duration {
    resolve_timeout_from_env("VWACP_CLAUDE_ACP_SESSION_CREATE_TIMEOUT_MS", 60_000)
}

fn resolve_timeout_from_env(key: &str, default_ms: u64) -> Duration {
    let timeout_ms = std::env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_ms);
    Duration::from_millis(timeout_ms)
}

/// 构造 Gemini ACP 初始化超时的诊断信息。
///
/// 如果没有检测到 API key，会在返回文案中提示非交互认证缺失。
pub(super) fn build_gemini_acp_startup_timeout_message(command: &str) -> String {
    let mut parts = vec![
        "Gemini CLI ACP startup timed out before initialize completed.".to_string(),
        "This usually means the local Gemini CLI is waiting on interactive OAuth or has incompatible ACP subprocess behavior.".to_string(),
    ];

    if std::env::var("GEMINI_API_KEY").ok().filter(|value| !value.trim().is_empty()).is_none()
        && std::env::var("GOOGLE_API_KEY").ok().filter(|value| !value.trim().is_empty()).is_none()
    {
        parts.push(
            "No GEMINI_API_KEY or GOOGLE_API_KEY was set for non-interactive auth.".to_string(),
        );
    }

    parts.push(format!(
        "Try upgrading Gemini CLI and using API-key-based auth for non-interactive ACP runs. Command: {command}"
    ));
    parts.join(" ")
}

/// 构造 Claude ACP 会话创建超时的诊断信息。
pub(super) fn build_claude_acp_session_create_timeout_message() -> String {
    [
        "Claude ACP session creation timed out before session/new completed.",
        "This matches the known persistent-session stall seen with some Claude Code and @agentclientprotocol/claude-agent-acp combinations.",
        "In harnessed or non-interactive runs, prefer --approve-all with nonInteractivePermissions=deny, upgrade Claude Code and the Claude ACP adapter, or use vwacp claude exec as a one-shot fallback.",
    ]
    .join(" ")
}

/// 将代理进程退出上下文补充到 ACP 错误中。
///
/// 仅对初始化和新建会话错误追加上下文，避免改变其他错误类型的语义。
pub(super) fn enrich_acp_error_with_process_context(
    err: AcpError,
    finalized: &FinalizedChild,
) -> AcpError {
    let Some(context) = format_process_context(&finalized.summary, &finalized.stderr_output) else {
        return err;
    };

    match err {
        AcpError::Initialize(message) => AcpError::Initialize(format!("{message} {context}")),
        AcpError::NewSession(message) => AcpError::NewSession(format!("{message} {context}")),
        other => other,
    }
}

fn format_process_context(summary: &ChildExitSummary, stderr_output: &str) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(code) = summary.exit_code {
        parts.push(format!("exit code {code}"));
    }
    if let Some(signal) = summary.signal.as_deref() {
        parts.push(format!("signal {signal}"));
    }

    let stderr = stderr_output.trim();
    if !stderr.is_empty() {
        let preview = stderr.chars().take(400).collect::<String>().replace('\n', " | ");
        parts.push(format!("stderr: {preview}"));
    }

    if parts.is_empty() {
        return None;
    }

    Some(format!("ACP agent process exited early ({})", parts.join(", ")))
}

/// 包装会话控制类 ACP 错误。
///
/// 该函数会识别常见 JSON-RPC 错误码，给不支持的 `session/set_*` 能力提供更清晰
/// 的提示；无法解析时保留原始错误文本。
pub(super) fn wrap_session_control_error<E>(
    method: &'static str,
    context: Option<String>,
    error: E,
    constructor: fn(String) -> AcpError,
) -> AcpError
where
    E: StdError + Send + Sync + 'static,
{
    let raw = error.to_string();
    if let Some(acp) = parse_session_control_acp_summary(&raw) {
        let summary = format_session_control_acp_summary(&acp);
        if is_likely_session_control_unsupported_error(&acp) {
            let context_suffix =
                context.as_deref().map(|value| format!(" {value}")).unwrap_or_default();
            return constructor(format!(
                "Agent rejected {method}{context_suffix}: {summary}. The adapter may not implement {method}, or the requested value is not supported."
            ));
        }
        return constructor(format!(
            "Failed {method}{}: {summary}",
            context.as_deref().map(|value| format!(" {value}")).unwrap_or_default()
        ));
    }

    constructor(format!(
        "Failed {method}{}: {raw}",
        context.as_deref().map(|value| format!(" {value}")).unwrap_or_default()
    ))
}

#[derive(Debug)]
struct SessionControlAcpSummary {
    code: i64,
    message: String,
    details: Option<String>,
}

fn parse_session_control_acp_summary(raw: &str) -> Option<SessionControlAcpSummary> {
    let code = [-32601_i64, -32602, -32603]
        .into_iter()
        .find(|candidate| raw.contains(&candidate.to_string()))?;

    let lower = raw.to_ascii_lowercase();
    let message = if lower.contains("method not found") {
        "method not found".to_string()
    } else if lower.contains("invalid params") {
        "invalid params".to_string()
    } else if lower.contains("internal error") {
        "internal error".to_string()
    } else {
        raw.trim().to_string()
    };

    Some(SessionControlAcpSummary { code, message, details: extract_details_field(raw) })
}

fn is_likely_session_control_unsupported_error(acp: &SessionControlAcpSummary) -> bool {
    if matches!(acp.code, -32601 | -32602) {
        return true;
    }
    acp.code == -32603
        && acp
            .details
            .as_deref()
            .is_some_and(|details| details.to_ascii_lowercase().contains("invalid params"))
}

fn format_session_control_acp_summary(acp: &SessionControlAcpSummary) -> String {
    if let Some(details) = acp.details.as_deref().filter(|details| !details.trim().is_empty()) {
        return format!(
            "{} (ACP {}, adapter reported \"{}\")",
            details.trim(),
            acp.code,
            acp.message
        );
    }
    format!("{} (ACP {})", acp.message, acp.code)
}

fn extract_details_field(raw: &str) -> Option<String> {
    for needle in ["details: ", "\"details\":\"", "\"details\": \""] {
        if let Some(index) = raw.find(needle) {
            let tail = &raw[index + needle.len()..];
            if let Some(extracted) = extract_quoted_or_delimited_text(tail) {
                return Some(extracted);
            }
        }
    }
    None
}

fn extract_quoted_or_delimited_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim_start();
    let mut chars = trimmed.chars();
    let first = chars.next()?;
    if matches!(first, '"' | '\'') {
        let rest = chars.as_str();
        let end = rest.find(first)?;
        return Some(rest[..end].to_string());
    }

    let end = trimmed.find([',', '}', '\n']).unwrap_or(trimmed.len());
    let candidate = trimmed[..end].trim().trim_matches(|ch| matches!(ch, '"' | '\'')).to_string();
    if candidate.is_empty() {
        return None;
    }
    Some(candidate)
}

/// 将 ACP 结束原因映射为 VibeWindow 内部提示词结束原因。
pub(super) fn acp_finish_reason(reason: acp::StopReason) -> String {
    match reason {
        acp::StopReason::EndTurn => "stop".to_string(),
        acp::StopReason::MaxTokens => "length".to_string(),
        acp::StopReason::MaxTurnRequests => "max_turn_requests".to_string(),
        acp::StopReason::Refusal => "refusal".to_string(),
        acp::StopReason::Cancelled => "cancelled".to_string(),
        _ => "stop".to_string(),
    }
}

/// 将 ACP 用量结构映射为内部用量结构。
pub(super) fn map_usage(usage: &acp::Usage) -> PromptUsage {
    PromptUsage {
        input_tokens: usage.input_tokens as i64,
        output_tokens: usage.output_tokens as i64,
        cached_tokens: usage.cached_read_tokens.unwrap_or_default() as i64,
        reasoning_tokens: usage.thought_tokens.unwrap_or_default() as i64,
    }
}
