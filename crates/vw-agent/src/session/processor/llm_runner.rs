//! LLM 步骤运行器模块
//!
//! 本模块提供与大型语言模型（LLM）交互的核心功能，负责执行 LLM 调用并处理流式响应。
//! 主要职责包括：
//! - 管理 LLM 调用的生命周期，包括流式响应的收集和处理
//! - 处理推理内容（reasoning）和普通文本内容的分离与格式化
//! - 实现带重试机制的 LLM 调用策略，提高调用的可靠性
//! - 管理工具调用的收集和传递
//!
//! # 核心组件
//!
//! - [`LlmStep`]: 封装单次 LLM 调用的完整结果
//! - [`run_llm_step`]: 执行单次 LLM 调用
//! - [`run_llm_step_with_retry`]: 执行带重试机制的 LLM 调用
//!
//! # 流式处理策略
//!
//! 本模块实现了智能的流式内容刷新策略，基于两个维度：
//! - 时间间隔：至少间隔 33ms 刷新一次，避免过于频繁的事件发送
//! - 内容长度：累积至少 1024 字符后刷新，减少小包传输的开销
//!
//! # 推理内容处理
//!
//! 模块会自动检测并格式化推理内容（模型思考过程），使用标准化的标签包裹，
//! 确保推理内容与普通文本内容清晰分离。

use super::types::StreamEvent;
use crate::app::agent::session::llm;
use crate::session::ui_types as models;
use serde_json::Value;
use web_time::Instant;

/// LLM 步骤执行结果
///
/// 封装单次 LLM 调用的完整输出结果，包括令牌使用统计、完成原因、推理内容、
/// 文本输出、工具调用以及完整的消息历史。
///
/// # 字段说明
///
/// * `usage` - 令牌使用统计，包括输入和输出令牌数量
/// * `finish_reason` - 模型完成响应的原因（如 "stop"、"length" 等）
/// * `reasoning_content` - 模型的推理/思考过程内容
/// * `text` - 模型生成的文本内容
/// * `tool_calls` - 模型请求执行的工具调用列表
/// * `full_messages` - 完整的消息历史记录，用于上下文追溯
///
/// # 示例
///
/// ```ignore
/// let step = run_llm_step(&messages, &system, model, &tools, &mut on_event)?;
/// println!("生成的文本: {}", step.text);
/// println!("使用了 {} 个令牌", step.usage.total_tokens);
/// ```
#[derive(Debug, Clone)]
pub(crate) struct LlmStep {
    /// 令牌使用统计信息
    pub(crate) usage: models::TokenUsage,
    /// 响应完成的原因（可选）
    pub(crate) finish_reason: Option<String>,
    /// 模型的推理/思考内容
    pub(crate) reasoning_content: String,
    /// 模型生成的主要文本内容
    pub(crate) text: String,
    /// 模型请求执行的工具调用列表
    pub(crate) tool_calls: Vec<llm::ToolCall>,
    /// 完整的消息历史，包含所有上下文信息
    pub(crate) full_messages: Vec<serde_json::Value>,
}

fn is_acp_request(options: &Value) -> bool {
    options.get("acp_test").and_then(Value::as_bool).unwrap_or(false)
        || options
            .get("acp_agent")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
}

fn is_acp_retryable_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("模型响应超时")
        || normalized.contains("timed out")
        || normalized.contains("timeout")
        || normalized.contains("acp agent disconnected during request")
        || normalized.contains("queue owner disconnected before prompt completion")
}

/// 执行单次 LLM 调用并处理流式响应
///
/// 此函数是 LLM 交互的核心入口，负责：
/// 1. 准备工具规范并过滤允许的工具
/// 2. 在独立线程中启动 LLM 流式调用
/// 3. 接收并处理流式事件
/// 4. 管理内容的缓冲和刷新策略
/// 5. 处理推理内容和普通内容的分离
/// 6. 收集并返回完整的调用结果
///
/// # 参数
///
/// * `messages` - 对话消息历史，包含用户和助手的历史交互
/// * `system` - 系统提示词列表，用于设定模型行为
/// * `model` - 指定使用的模型名称（可选，使用默认模型）
/// * `allowed_tools` - 允许调用的工具 ID 集合
/// * `on_event` - 流式事件回调函数，返回 `false` 可中断处理
///
/// # 返回值
///
/// 成功时返回 [`LlmStep`] 包含完整的调用结果，失败时返回错误信息字符串。
///
/// # 错误情况
///
/// - 模型响应超时（默认 90 秒无响应）
/// - 模型调用失败（网络错误、API 错误等）
/// - 模型未返回任何内容（文本和工具调用都为空）
///
/// # 流式处理策略
///
/// 采用双重缓冲机制：
/// - **时间维度**：最小刷新间隔 33ms，避免事件过于频繁
/// - **内容维度**：最小刷新字符数 1024，减少小包传输
///
/// # 推理内容处理
///
/// 当模型返回推理内容时，会自动：
/// 1. 在推理内容前添加 `<thinking>` 开始标签
/// 2. 流式输出推理过程
/// 3. 在开始返回普通内容时添加 `</thinking>` 结束标签
/// 4. 如果推理内容未流式输出，则在最后统一输出
///
/// # 示例
///
/// ```ignore
/// let messages = vec![json!({"role": "user", "content": "你好"})];
/// let system = vec!["你是一个有帮助的助手".to_string()];
/// let tools = std::collections::HashSet::from(["search"]);
///
/// let result = run_llm_step(
///     &messages,
///     &system,
///     Some("gpt-4".to_string()),
///     &tools,
///     &mut |event| {
///         if let StreamEvent::Delta(text) = event {
///             print!("{}", text);
///         }
///         true
///     },
/// )?;
/// ```
pub(crate) fn run_llm_step(
    session_id: &str,
    messages: &[serde_json::Value],
    system: &[String],
    model: Option<String>,
    options: &serde_json::Value,
    allowed_tools: &std::collections::HashSet<String>,
    on_event: &mut impl FnMut(StreamEvent) -> bool,
) -> Result<LlmStep, String> {
    // 创建用于跨线程通信的通道
    let (tx, rx) = std::sync::mpsc::channel::<llm::StreamEvent>();

    // 克隆数据以便在独立线程中使用
    let session_id = session_id.to_string();
    let messages = messages.to_vec();
    let system = system.to_vec();
    let options = options.clone();

    // 获取工具规范并根据允许列表过滤
    let tools = crate::tools::registry::specs(model.as_deref());
    let tools = tools
        .into_iter()
        .filter(|s| allowed_tools.contains(&s.id)) // 仅保留允许的工具
        .map(|s| (s.id.clone(), s))
        .collect::<std::collections::HashMap<_, _>>();

    // 在独立线程中启动 LLM 流式调用，避免阻塞当前线程
    std::thread::spawn(move || {
        llm::stream_chat_with_tools_for_session(
            &session_id,
            messages,
            system,
            model,
            options,
            tools,
            move |ev| {
                let _ = tx.send(ev); // 将事件发送到主线程
            },
        );
    });

    // === 初始化结果收集变量 ===
    let mut reasoning_content = String::new(); // 推理内容缓冲区
    let mut text = String::new(); // 普通文本内容缓冲区
    let mut fallback_text = String::new(); // 回退文本（当没有普通文本时使用推理内容）
    let mut usage = models::TokenUsage::default(); // 令牌使用统计
    let mut finish_reason: Option<String> = None; // 完成原因
    let mut tool_calls: Vec<llm::ToolCall> = Vec::new(); // 工具调用列表
    let mut full_messages: Vec<serde_json::Value> = Vec::new(); // 完整消息历史

    // === 流式处理配置常量 ===
    use std::time::Duration;
    /// 流式内容刷新的最小时间间隔（约 30 FPS）
    const FLUSH_MIN_INTERVAL: Duration = Duration::from_millis(33);
    /// 流式内容刷新的最小字符数
    const FLUSH_MIN_CHARS: usize = 1024;
    /// 无响应超时时间（90 秒）
    const IDLE_TIMEOUT: Duration = Duration::from_secs(90);

    // === 状态追踪变量 ===
    let mut pending = String::new(); // 待刷新的普通文本缓冲区
    let mut pending_reasoning = String::new(); // 待刷新的推理文本缓冲区
    let mut last_flush = Instant::now(); // 上次普通文本刷新时间
    let mut last_reasoning_flush = Instant::now(); // 上次推理文本刷新时间
    let mut content_started = false; // 标记是否开始接收普通内容
    let mut thinking_open = false; // 标记 thinking 标签是否已打开
    let mut thinking_emitted = false; // 标记是否已输出推理内容

    // === 主事件处理循环 ===
    loop {
        // 尝试接收事件，设置超时以检测无响应情况
        let ev = match rx.recv_timeout(IDLE_TIMEOUT) {
            Ok(v) => v,
            // 超时处理：刷新所有待输出内容并返回错误
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // 刷新待输出的普通文本
                if !pending.is_empty() {
                    let chunk = std::mem::take(&mut pending);
                    on_event(StreamEvent::Delta(chunk));
                }
                // 刷新待输出的推理文本
                if !pending_reasoning.is_empty() {
                    let chunk = std::mem::take(&mut pending_reasoning);
                    on_event(StreamEvent::Delta(chunk));
                }
                // 如果 thinking 标签仍然打开，则关闭它
                if thinking_open {
                    on_event(StreamEvent::Delta("</think>\n\n".to_string()));
                }
                return Err("模型响应超时".to_string());
            }
            // 发送端断开，表示流结束
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };

        // 根据事件类型进行相应处理
        match ev {
            // 处理普通文本增量
            llm::StreamEvent::Delta(delta) => {
                // 检测是否首次接收到普通内容
                if !content_started {
                    content_started = true;
                    // 如果之前有推理内容，需要关闭 thinking 标签
                    if thinking_open {
                        if !pending_reasoning.is_empty() {
                            let chunk = std::mem::take(&mut pending_reasoning);
                            on_event(StreamEvent::Delta(chunk));
                        }
                        on_event(StreamEvent::Delta("</think>\n\n".to_string()));
                        thinking_open = false;
                    }
                }
                // 累积普通文本
                text.push_str(&delta);
                pending.push_str(&delta);
                // 检查是否满足刷新条件（内容长度或时间间隔）
                if pending.len() >= FLUSH_MIN_CHARS || last_flush.elapsed() >= FLUSH_MIN_INTERVAL {
                    let chunk = std::mem::take(&mut pending);
                    last_flush = Instant::now();
                    on_event(StreamEvent::Delta(chunk));
                }
            }
            // 处理推理内容增量
            llm::StreamEvent::ReasoningDelta(delta) => {
                // 累积推理内容
                reasoning_content.push_str(&delta);
                // 如果还未开始接收普通内容，则流式输出推理内容
                if !content_started {
                    // 同时保存到回退文本，以便在没有普通文本时使用
                    fallback_text.push_str(&delta);
                    // 如果 thinking 标签未打开，则打开它
                    if !thinking_open {
                        pending_reasoning.push_str("<think>");
                        thinking_open = true;
                        thinking_emitted = true;
                    }
                    pending_reasoning.push_str(&delta);
                    // 检查是否满足刷新条件
                    if pending_reasoning.len() >= FLUSH_MIN_CHARS
                        || last_reasoning_flush.elapsed() >= FLUSH_MIN_INTERVAL
                    {
                        let chunk = std::mem::take(&mut pending_reasoning);
                        last_reasoning_flush = Instant::now();
                        on_event(StreamEvent::Delta(chunk));
                    }
                }
            }
            // 处理工具调用
            llm::StreamEvent::ToolCalls(calls) => {
                tool_calls = calls;
            }
            // 处理完整消息历史
            llm::StreamEvent::FullMessages(msgs) => {
                full_messages = msgs;
            }
            // 处理完成事件
            llm::StreamEvent::Done { usage: u, finish_reason: f } => {
                usage = u;
                finish_reason = f;
                break;
            }
            // 处理错误事件
            llm::StreamEvent::Error(e) => {
                // 刷新所有待输出内容
                if !pending.is_empty() {
                    let chunk = std::mem::take(&mut pending);
                    on_event(StreamEvent::Delta(chunk));
                }
                if !pending_reasoning.is_empty() {
                    let chunk = std::mem::take(&mut pending_reasoning);
                    on_event(StreamEvent::Delta(chunk));
                }
                if thinking_open {
                    on_event(StreamEvent::Delta("</think>\n\n".to_string()));
                }
                // 将错误转换为字符串并返回
                return Err(
                    serde_json::to_string(&e).unwrap_or_else(|_| "模型调用失败".to_string())
                );
            }
        }
    }

    // === 流结束后的收尾处理 ===

    // 刷新剩余的待输出普通文本
    if !pending.is_empty() {
        on_event(StreamEvent::Delta(pending));
    }

    // 刷新剩余的待输出推理文本
    if !pending_reasoning.is_empty() {
        on_event(StreamEvent::Delta(pending_reasoning));
    }

    // 关闭仍打开的 thinking 标签
    if thinking_open {
        on_event(StreamEvent::Delta("</think>\n\n".to_string()));
    }

    // 如果推理内容未在流中输出，则在此处统一输出
    // 这种情况发生在推理内容在普通内容开始后才到达
    if !thinking_emitted && !reasoning_content.trim().is_empty() {
        on_event(StreamEvent::Delta(format!(
            "\n\n<think>{}</think>\n\n",
            reasoning_content.trim_end()
        )));
    }

    // 如果普通文本为空但推理内容不为空，使用推理内容作为回退文本
    if text.trim().is_empty() && !fallback_text.trim().is_empty() {
        text = fallback_text;
    }

    // 验证至少有文本内容或工具调用
    if text.trim().is_empty() && tool_calls.is_empty() {
        return Err("模型未返回内容".to_string());
    }

    // 构造并返回完整的 LLM 步骤结果
    Ok(LlmStep { usage, finish_reason, reasoning_content, text, tool_calls, full_messages })
}

/// 执行带重试机制的 LLM 调用
///
/// 此函数在 [`run_llm_step`] 的基础上添加了自动重试机制，当 LLM 调用失败时会自动重试，
/// 每次重试之间采用指数退避策略，以避免对服务器造成过大压力。
///
/// # 参数
///
/// * `messages` - 对话消息历史，包含用户和助手的历史交互
/// * `system` - 系统提示词列表，用于设定模型行为
/// * `model` - 指定使用的模型名称（可选，使用默认模型）
/// * `allowed_tools` - 允许调用的工具 ID 集合
/// * `on_event` - 流式事件回调函数，返回 `false` 可中断处理
/// * `max_attempts` - 最大尝试次数（包括首次调用）
///
/// # 返回值
///
/// 成功时返回 [`LlmStep`] 包含完整的调用结果，失败时返回最后一次的错误信息字符串。
///
/// # 重试策略
///
/// - 采用指数退避策略：首次重试等待 300ms，之后每次翻倍
/// - 最大退避时间：5000ms（5 秒）
/// - 退避公式：`min(5000, 300 * 2^(attempt-1))`
///
/// # 示例
///
/// ```ignore
/// let messages = vec![json!({"role": "user", "content": "你好"})];
/// let system = vec!["你是一个有帮助的助手".to_string()];
/// let tools = std::collections::HashSet::from(["search"]);
///
/// // 最多尝试 3 次
/// let result = run_llm_step_with_retry(
///     &messages,
///     &system,
///     Some("gpt-4".to_string()),
///     &tools,
///     &mut |event| {
///         if let StreamEvent::Delta(text) = event {
///             print!("{}", text);
///         }
///         true
///     },
///     3,
/// )?;
/// ```
///
/// # 注意事项
///
/// - 当 `max_attempts` 为 0 时，会被调整为 1（至少尝试一次）
/// - 所有尝试都会调用同一个 `on_event` 回调，请注意处理重复事件
/// - 退避期间会阻塞当前线程
pub(crate) fn run_llm_step_with_retry(
    session_id: &str,
    messages: &[serde_json::Value],
    system: &[String],
    model: Option<String>,
    options: &serde_json::Value,
    allowed_tools: &std::collections::HashSet<String>,
    on_event: &mut impl FnMut(StreamEvent) -> bool,
    max_attempts: usize,
) -> Result<LlmStep, String> {
    // 保存最后一次错误信息
    let mut last_err: Option<String> = None;
    let is_acp = is_acp_request(options);

    // 确保至少尝试一次
    let attempts = std::cmp::max(1, max_attempts);

    // 逐次尝试调用
    for attempt in 1..=attempts {
        match run_llm_step(
            session_id,
            messages,
            system,
            model.clone(),
            options,
            allowed_tools,
            on_event,
        ) {
            // 成功则立即返回结果
            Ok(v) => return Ok(v),
            // 失败则记录错误并考虑重试
            Err(e) => {
                last_err = Some(e);
                let error_preview = crate::app::agent::util::truncate_with_ellipsis(
                    &crate::agent::loop_::scrub_credentials(
                        last_err.as_deref().unwrap_or_default(),
                    ),
                    240,
                );
                tracing::warn!(
                    target: "vw_agent",
                    session_id,
                    attempt,
                    attempts,
                    is_acp,
                    acp_retry_allowed = last_err
                        .as_deref()
                        .is_some_and(is_acp_retryable_error),
                    error_preview = %error_preview,
                    "session processor retryable llm step attempt failed"
                );

                if is_acp && !last_err.as_deref().is_some_and(is_acp_retryable_error) {
                    break;
                }

                // 如果已达到最大尝试次数，不再重试
                if attempt == attempts {
                    break;
                }

                // 计算退避时间（指数退避，最大 5000ms）
                // 公式：min(5000, 300 * 2^(attempt-1))
                let backoff_ms =
                    std::cmp::min(5000, 300u64.saturating_mul(2u64.pow((attempt - 1) as u32)));

                // 等待后重试
                std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
            }
        }
    }

    // 所有尝试都失败，返回最后一次错误
    Err(last_err.unwrap_or_else(|| "模型调用失败".to_string()))
}
#[cfg(test)]
#[path = "llm_runner_tests.rs"]
mod llm_runner_tests;
