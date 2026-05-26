//! 委托协调状态工具
//!
//! 只读的运行时可观测性工具，用于查看委托代理之间的协调事件和消息。
//!
//! # 主要功能
//!
//! - 查看智能体收件箱积压状态
//! - 检查上下文状态转换
//! - 查看死信事件
//!
//! # 使用场景
//!
//! 该工具主要用于调试和监控委托协调系统的运行状态，帮助开发者了解：
//! - 消息传递是否正常
//! - 是否存在积压的消息
//! - 死信事件的详细信息

use super::traits::{Tool, ToolResult};
use crate::app::agent::coordination::{CoordinationPayload, InMemoryMessageBus, SequencedEnvelope};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// 死信条目的默认限制数量
const DEFAULT_DEAD_LETTER_LIMIT: usize = 10;

/// 死信条目的最大限制数量
const MAX_DEAD_LETTER_LIMIT: usize = 100;

/// 死信条目的最大偏移量
const MAX_DEAD_LETTER_OFFSET: usize = 10_000;

/// 消息预览的默认限制数量
const DEFAULT_MESSAGE_LIMIT: usize = 5;

/// 消息预览的最大限制数量
const MAX_MESSAGE_LIMIT: usize = 50;

/// 消息预览的最大偏移量
const MAX_MESSAGE_OFFSET: usize = 10_000;

/// 上下文条目的默认限制数量
const DEFAULT_CONTEXT_LIMIT: usize = 25;

/// 上下文条目的最大限制数量
const MAX_CONTEXT_LIMIT: usize = 200;

/// 上下文条目的最大偏移量
const MAX_CONTEXT_OFFSET: usize = 10_000;

/// 委托协调状态工具
///
/// 这是一个只读的运行时可观测性工具，用于查看委托代理之间的协调事件和消息。
/// 它提供了对消息总线状态的深度可见性，包括智能体收件箱、上下文状态和死信事件。
///
/// # 功能特性
///
/// - **收件箱状态**: 查看每个智能体的待处理消息数量和预览
/// - **上下文状态**: 检查委托协调上下文的状态转换
/// - **死信事件**: 查看无法投递的消息及其原因
///
/// # 安全性
///
/// 该工具在执行前会检查安全策略，确保只有授权的操作才能访问协调状态。
///
/// # 示例
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use vibe_window::app::agent::tools::delegate_coordination_status::DelegateCoordinationStatusTool;
/// use vibe_window::app::agent::coordination::InMemoryMessageBus;
/// use vibe_window::app::agent::security::SecurityPolicy;
///
/// let bus = InMemoryMessageBus::new(Default::default());
/// let security = Arc::new(SecurityPolicy::default());
/// let tool = DelegateCoordinationStatusTool::new(bus, security);
/// ```
pub struct DelegateCoordinationStatusTool {
    /// 内存消息总线引用
    bus: InMemoryMessageBus,
    /// 安全策略引用
    security: Arc<SecurityPolicy>,
}

impl DelegateCoordinationStatusTool {
    /// 创建新的委托协调状态工具实例
    ///
    /// # 参数
    ///
    /// - `bus`: 内存消息总线实例，用于获取协调状态
    /// - `security`: 安全策略引用，用于权限检查
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `DelegateCoordinationStatusTool` 实例
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use vibe_window::app::agent::tools::delegate_coordination_status::DelegateCoordinationStatusTool;
    /// use vibe_window::app::agent::coordination::InMemoryMessageBus;
    /// use vibe_window::app::agent::security::SecurityPolicy;
    ///
    /// let bus = InMemoryMessageBus::new(Default::default());
    /// let security = Arc::new(SecurityPolicy::default());
    /// let tool = DelegateCoordinationStatusTool::new(bus, security);
    /// ```
    pub fn new(bus: InMemoryMessageBus, security: Arc<SecurityPolicy>) -> Self {
        Self { bus, security }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for DelegateCoordinationStatusTool {
    /// 获取工具名称
    ///
    /// # 返回值
    ///
    /// 返回工具的标识符 `"delegate_coordination_status"`
    fn name(&self) -> &str {
        "delegate_coordination_status"
    }

    /// 获取工具描述
    ///
    /// # 返回值
    ///
    /// 返回工具的中文描述信息，说明工具的用途
    fn description(&self) -> &str {
        "检查委托协调运行时状态（智能体收件箱积压、上下文状态转换和死信事件）。"
    }

    /// 获取工具参数的 JSON Schema 定义
    ///
    /// # 返回值
    ///
    /// 返回一个 JSON Schema 对象，定义了工具支持的所有参数及其约束条件。
    ///
    /// # 参数说明
    ///
    /// - `agent`: 可选的智能体名称，如果设置则只报告该智能体的收件箱
    /// - `correlation_id`: 可选的委托关联 ID，用于过滤上下文和死信输出
    /// - `include_messages`: 是否包含收件箱的消息预览，默认为 false
    /// - `message_limit`: 当 include_messages=true 时，每个收件箱的最大预览消息数
    /// - `message_offset`: 预览消息的偏移量（按最旧优先排序）
    /// - `include_dead_letters`: 是否包含死信预览条目，默认为 true
    /// - `dead_letter_limit`: 最大死信条目数
    /// - `dead_letter_offset`: 死信条目的偏移量（按最新优先排序）
    /// - `context_limit`: 最大上下文条目数（按最新更新优先排序）
    /// - `context_offset`: 上下文条目的偏移量（按最新更新优先排序）
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "agent": {
                    "type": "string",
                    "description": "可选的智能体名称。如果设置，则只报告该智能体的收件箱。"
                },
                "correlation_id": {
                    "type": "string",
                    "description": "可选的委托关联 ID。用于过滤上下文和死信输出。"
                },
                "include_messages": {
                    "type": "boolean",
                    "description": "是否包含收件箱的消息预览",
                    "default": false
                },
                "message_limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_MESSAGE_LIMIT,
                    "description": "当 include_messages=true 时，每个收件箱的最大预览消息数",
                    "default": DEFAULT_MESSAGE_LIMIT
                },
                "message_offset": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": MAX_MESSAGE_OFFSET,
                    "description": "预览消息的偏移量（按最旧优先排序，或当设置 correlation_id 时按匹配的最旧优先）",
                    "default": 0
                },
                "include_dead_letters": {
                    "type": "boolean",
                    "description": "是否包含死信预览条目",
                    "default": true
                },
                "dead_letter_limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_DEAD_LETTER_LIMIT,
                    "description": "最大死信条目数",
                    "default": DEFAULT_DEAD_LETTER_LIMIT
                },
                "dead_letter_offset": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": MAX_DEAD_LETTER_OFFSET,
                    "description": "死信条目的偏移量（按最新优先排序）",
                    "default": 0
                },
                "context_limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_CONTEXT_LIMIT,
                    "description": "最大上下文条目数（按最新更新优先排序）",
                    "default": DEFAULT_CONTEXT_LIMIT
                },
                "context_offset": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": MAX_CONTEXT_OFFSET,
                    "description": "上下文条目的偏移量（按最新更新优先排序）",
                    "default": 0
                }
            },
            "required": []
        })
    }

    /// 执行工具操作
    ///
    /// 根据提供的参数查询委托协调状态，包括智能体收件箱、上下文状态和死信事件。
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象，支持以下字段：
    ///   - `agent`: 可选的智能体名称过滤器
    ///   - `correlation_id`: 可选的关联 ID 过滤器
    ///   - `include_messages`: 是否包含消息预览
    ///   - `message_limit`: 消息预览限制
    ///   - `message_offset`: 消息偏移量
    ///   - `include_dead_letters`: 是否包含死信
    ///   - `dead_letter_limit`: 死信限制
    ///   - `dead_letter_offset`: 死信偏移量
    ///   - `context_limit`: 上下文限制
    ///   - `context_offset`: 上下文偏移量
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，包含：
    /// - `success`: 操作是否成功
    /// - `output`: JSON 格式的状态信息（如果成功）
    /// - `error`: 错误信息（如果失败）
    ///
    /// # 安全检查
    ///
    /// 执行前会通过安全策略检查 `ToolOperation::Read` 权限。
    ///
    /// # 返回数据结构
    ///
    /// 成功时返回的 JSON 包含：
    /// - `subscriber_count`: 订阅者数量
    /// - `context_count`: 上下文数量
    /// - `delegate_context_count`: 委托上下文数量
    /// - `dead_letter_count`: 死信数量
    /// - `limits`: 总线限制配置
    /// - `stats`: 统计信息
    /// - `filter`: 使用的过滤条件
    /// - `inboxes`: 智能体收件箱信息数组
    /// - `contexts`: 上下文条目数组
    /// - `dead_letters`: 死信条目数组
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 安全策略检查：验证是否有读取权限
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Read, self.name()) {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }

        // 解析并清理智能体名称过滤参数
        let filter_agent = args
            .get("agent")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        // 解析并清理关联 ID 过滤参数
        let filter_correlation = args
            .get("correlation_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        // 解析是否包含消息预览的标志
        let include_messages =
            args.get("include_messages").and_then(serde_json::Value::as_bool).unwrap_or(false);

        // 解析并限制消息预览数量
        let message_limit = clamp_usize(
            args.get("message_limit")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            DEFAULT_MESSAGE_LIMIT,
            MAX_MESSAGE_LIMIT,
        );

        // 解析并限制消息偏移量
        let message_offset = clamp_offset(
            args.get("message_offset")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            MAX_MESSAGE_OFFSET,
        );

        // 解析是否包含死信的标志
        let include_dead_letters =
            args.get("include_dead_letters").and_then(serde_json::Value::as_bool).unwrap_or(true);

        // 解析并限制死信条目数量
        let dead_letter_limit = clamp_usize(
            args.get("dead_letter_limit")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            DEFAULT_DEAD_LETTER_LIMIT,
            MAX_DEAD_LETTER_LIMIT,
        );

        // 解析并限制死信偏移量
        let dead_letter_offset = clamp_offset(
            args.get("dead_letter_offset")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            MAX_DEAD_LETTER_OFFSET,
        );

        // 解析并限制上下文条目数量
        let context_limit = clamp_usize(
            args.get("context_limit")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            DEFAULT_CONTEXT_LIMIT,
            MAX_CONTEXT_LIMIT,
        );

        // 解析并限制上下文偏移量
        let context_offset = clamp_offset(
            args.get("context_offset")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| usize::try_from(value).ok()),
            MAX_CONTEXT_OFFSET,
        );

        // 确定要查询的智能体列表：如果指定了过滤则只查该智能体，否则查所有注册的智能体
        let agents = if let Some(agent) = filter_agent.clone() {
            vec![agent]
        } else {
            self.bus.registered_agents()
        };

        // 收集所有智能体的收件箱信息
        let mut inboxes = Vec::new();
        for agent in agents {
            // 获取该智能体的待处理消息数量
            let pending = match self.bus.pending_for_agent(&agent) {
                Ok(value) => value,
                Err(_) => continue, // 如果获取失败，跳过该智能体
            };

            // 如果设置了关联 ID 过滤，获取过滤后的待处理数量
            let pending_filtered = filter_correlation.as_deref().and_then(|correlation_id| {
                self.bus.pending_for_agent_correlation(&agent, correlation_id).ok()
            });

            let mut message_total = 0usize;
            let mut message_preview = Vec::new();

            // 如果请求包含消息预览，则获取消息详情
            if include_messages {
                // 根据是否设置了关联 ID 过滤选择不同的查询方式
                let matched_messages = if let Some(correlation_id) = filter_correlation.as_deref() {
                    message_total = pending_filtered.unwrap_or(0);
                    self.bus
                        .peek_for_agent_correlation_with_offset(
                            &agent,
                            correlation_id,
                            message_offset,
                            message_limit,
                        )
                        .unwrap_or_default()
                } else {
                    message_total = pending;
                    self.bus
                        .peek_for_agent_with_offset(&agent, message_offset, message_limit)
                        .unwrap_or_default()
                };

                // 将消息封装转换为摘要格式
                message_preview =
                    matched_messages.into_iter().map(summarize_envelope).collect::<Vec<_>>();
            }

            // 计算分页信息
            let messages_returned = message_preview.len();
            let messages_truncated = include_messages
                && message_offset.saturating_add(messages_returned) < message_total;
            let message_next_offset = (include_messages && messages_truncated)
                .then_some(message_offset + messages_returned);

            // 构建收件箱信息对象
            inboxes.push(json!({
                "agent": agent,
                "pending": pending,
                "pending_filtered": pending_filtered,
                "message_total": message_total,
                "message_offset": message_offset,
                "messages_returned": messages_returned,
                "messages_truncated": messages_truncated,
                "message_next_offset": message_next_offset,
                "messages": message_preview
            }));
        }

        // 获取上下文条目信息
        let (contexts_total, context_entries) = if let Some(correlation_id) =
            filter_correlation.as_deref()
        {
            // 如果设置了关联 ID 过滤，则获取该关联 ID 的上下文
            (
                self.bus.delegate_context_count_for_correlation(correlation_id),
                self.bus.delegate_context_entries_recent_for_correlation_with_offset(
                    correlation_id,
                    context_offset,
                    context_limit,
                ),
            )
        } else {
            // 否则获取所有上下文
            (
                self.bus.delegate_context_count(),
                self.bus.delegate_context_entries_recent_with_offset(context_offset, context_limit),
            )
        };

        // 将上下文条目转换为 JSON 格式
        let contexts = context_entries
            .into_iter()
            .map(|(key, entry)| {
                json!({
                    "key": key,
                    "version": entry.version,
                    "updated_by": entry.updated_by,
                    "last_message_id": entry.last_message_id,
                    "value": entry.value
                })
            })
            .collect::<Vec<_>>();

        // 计算上下文分页信息
        let contexts_returned = contexts.len();
        let contexts_truncated = context_offset.saturating_add(contexts_returned) < contexts_total;
        let context_next_offset = contexts_truncated.then_some(context_offset + contexts_returned);

        // 获取死信信息
        let mut dead_letter_preview = Vec::new();
        let mut dead_letters_total = 0usize;
        if include_dead_letters {
            // 根据是否设置了关联 ID 过滤选择不同的查询方式
            let matching = if let Some(correlation_id) = filter_correlation.as_deref() {
                dead_letters_total = self.bus.dead_letter_count_for_correlation(correlation_id);
                self.bus.dead_letters_recent_for_correlation(
                    correlation_id,
                    dead_letter_offset,
                    dead_letter_limit,
                )
            } else {
                dead_letters_total = self.bus.dead_letter_count();
                self.bus.dead_letters_recent(dead_letter_offset, dead_letter_limit)
            };

            // 将死信条目转换为 JSON 格式（反转顺序以获取最新的在前）
            dead_letter_preview = matching
                .into_iter()
                .rev()
                .map(|entry| {
                    json!({
                        "message_id": entry.envelope.id,
                        "topic": entry.envelope.topic,
                        "from": entry.envelope.from,
                        "to": entry.envelope.to,
                        "correlation_id": entry.envelope.correlation_id,
                        "payload_kind": payload_kind(&entry.envelope.payload),
                        "reason": entry.reason
                    })
                })
                .collect::<Vec<_>>();
        }

        // 计算死信分页信息
        let dead_letters_returned = dead_letter_preview.len();
        let dead_letters_truncated =
            dead_letter_offset.saturating_add(dead_letters_returned) < dead_letters_total;
        let dead_letter_next_offset =
            dead_letters_truncated.then_some(dead_letter_offset + dead_letters_returned);

        // 计算过滤后的委托上下文数量
        let delegate_context_count_filtered = filter_correlation
            .as_deref()
            .map(|correlation_id| self.bus.delegate_context_count_for_correlation(correlation_id))
            .unwrap_or_else(|| self.bus.delegate_context_count());

        // 构建最终输出结果
        let output = json!({
            "subscriber_count": self.bus.subscriber_count(),
            "context_count": self.bus.context_count(),
            "delegate_context_count": self.bus.delegate_context_count(),
            "delegate_context_count_filtered": delegate_context_count_filtered,
            "dead_letter_count": self.bus.dead_letter_count(),
            "limits": self.bus.limits(),
            "stats": self.bus.stats(),
            "filter": {
                "agent": filter_agent,
                "correlation_id": filter_correlation
            },
            "contexts_total": contexts_total,
            "contexts_offset": context_offset,
            "contexts_returned": contexts_returned,
            "contexts_truncated": contexts_truncated,
            "context_next_offset": context_next_offset,
            "dead_letters_total": dead_letters_total,
            "dead_letter_offset": dead_letter_offset,
            "dead_letters_returned": dead_letters_returned,
            "dead_letters_truncated": dead_letters_truncated,
            "dead_letter_next_offset": dead_letter_next_offset,
            "inboxes": inboxes,
            "contexts": contexts,
            "dead_letters": dead_letter_preview
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&output).unwrap_or_default(),
            error: None,
        })
    }
}

/// 限制 usize 值在有效范围内
///
/// 如果提供的值为 None 或 0，则使用默认值；否则将值限制在最大值以内。
///
/// # 参数
///
/// - `value`: 可选的 usize 值
/// - `default_value`: 默认值（当 value 为 None 或 0 时使用）
/// - `max_value`: 最大值限制
///
/// # 返回值
///
/// 返回限制后的 usize 值
///
/// # 示例
///
/// ```rust
/// // 返回默认值
/// assert_eq!(clamp_usize(None, 10, 100), 10);
/// assert_eq!(clamp_usize(Some(0), 10, 100), 10);
///
/// // 返回限制后的值
/// assert_eq!(clamp_usize(Some(50), 10, 100), 50);
/// assert_eq!(clamp_usize(Some(150), 10, 100), 100); // 超过最大值，返回最大值
/// ```
fn clamp_usize(value: Option<usize>, default_value: usize, max_value: usize) -> usize {
    match value {
        Some(value) if value > 0 => value.min(max_value),
        _ => default_value,
    }
}

/// 限制偏移量在有效范围内
///
/// 如果提供的值为 None，则使用 0；否则将值限制在最大值以内。
///
/// # 参数
///
/// - `value`: 可选的偏移量值
/// - `max_value`: 最大偏移量限制
///
/// # 返回值
///
/// 返回限制后的偏移量值
///
/// # 示例
///
/// ```rust
/// // 返回默认值 0
/// assert_eq!(clamp_offset(None, 1000), 0);
///
/// // 返回限制后的值
/// assert_eq!(clamp_offset(Some(50), 1000), 50);
/// assert_eq!(clamp_offset(Some(1500), 1000), 1000); // 超过最大值，返回最大值
/// ```
fn clamp_offset(value: Option<usize>, max_value: usize) -> usize {
    value.unwrap_or(0).min(max_value)
}

/// 将消息封装转换为摘要格式
///
/// 提取消息封装中的关键信息，生成用于预览的 JSON 对象。
///
/// # 参数
///
/// - `entry`: 带序号的消息封装
///
/// # 返回值
///
/// 返回包含以下字段的 JSON 对象：
/// - `sequence`: 消息序号
/// - `message_id`: 消息 ID
/// - `topic`: 消息主题
/// - `from`: 发送者
/// - `to`: 接收者
/// - `correlation_id`: 关联 ID
/// - `causation_id`: 因果 ID
/// - `payload_kind`: 载荷类型
fn summarize_envelope(entry: SequencedEnvelope) -> serde_json::Value {
    json!({
        "sequence": entry.sequence,
        "message_id": entry.envelope.id,
        "topic": entry.envelope.topic,
        "from": entry.envelope.from,
        "to": entry.envelope.to,
        "correlation_id": entry.envelope.correlation_id,
        "causation_id": entry.envelope.causation_id,
        "payload_kind": payload_kind(&entry.envelope.payload)
    })
}

/// 获取协调载荷的类型名称
///
/// 根据载荷的变体返回对应的类型字符串。
///
/// # 参数
///
/// - `payload`: 协调载荷引用
///
/// # 返回值
///
/// 返回载荷类型的字符串标识：
/// - `"delegate_task"`: 委托任务
/// - `"context_patch"`: 上下文补丁
/// - `"task_result"`: 任务结果
/// - `"ack"`: 确认消息
/// - `"control"`: 控制消息
fn payload_kind(payload: &CoordinationPayload) -> &'static str {
    match payload {
        CoordinationPayload::DelegateTask { .. } => "delegate_task",
        CoordinationPayload::ContextPatch { .. } => "context_patch",
        CoordinationPayload::TaskResult { .. } => "task_result",
        CoordinationPayload::Ack { .. } => "ack",
        CoordinationPayload::Control { .. } => "control",
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
