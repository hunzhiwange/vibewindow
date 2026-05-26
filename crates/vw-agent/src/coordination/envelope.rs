//! 协调信封模块
//!
//! 本模块定义了代理间协调通信的核心数据结构，包括消息信封和负载类型。
//!
//! # 主要组件
//!
//! - [`DeliveryScope`]: 定义消息投递范围（点对点或广播）
//! - [`CoordinationPayload`]: 定义协调协议支持的所有负载类型
//! - [`CoordinationEnvelope`]: 包装消息元数据和负载的信封结构
//!
//! # 设计原则
//!
//! - 使用类型系统确保消息结构完整性
//! - 支持消息溯源和因果关系追踪
//! - 内置验证逻辑确保协议合规性

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::util::require_non_empty;

/// 消息投递范围枚举
///
/// 定义协调消息的投递模式，用于控制消息的分发范围。
///
/// # 变体说明
///
/// - `Direct`: 点对点投递，消息发送给指定的单个目标代理
/// - `Broadcast`: 广播投递，消息发送给所有已注册的代理
///
/// # 序列化
///
/// 使用 `snake_case` 格式进行序列化，例如：
/// - `Direct` 序列化为 `"direct"`
/// - `Broadcast` 序列化为 `"broadcast"`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryScope {
    /// 点对点投递模式
    ///
    /// 消息仅发送给 `CoordinationEnvelope::to` 字段指定的单个目标代理。
    /// 适用于需要精确控制消息接收者的场景，如任务委派、结果回复等。
    Direct,

    /// 广播投递模式
    ///
    /// 消息发送给所有已注册的代理。`CoordinationEnvelope::to` 字段应为 `None`。
    /// 适用于需要通知所有代理的场景，如状态同步、控制指令等。
    Broadcast,
}

/// 协调负载枚举
///
/// 定义代理间协调通信支持的所有负载类型。每种负载类型代表一种特定的协调操作。
///
/// # 变体说明
///
/// - `DelegateTask`: 委派任务给其他代理
/// - `ContextPatch`: 更新共享上下文数据
/// - `TaskResult`: 返回任务执行结果
/// - `Ack`: 确认消息接收
/// - `Control`: 控制指令
///
/// # 序列化
///
/// 使用内部标签（internally tagged）方式序列化，`kind` 字段标识负载类型：
/// ```json
/// {
///   "kind": "delegate_task",
///   "task_id": "...",
///   "summary": "...",
///   "metadata": {}
/// }
/// ```
///
/// # 示例
///
/// ```
/// use vibe_window::app::agent::coordination::envelope::CoordinationPayload;
/// use serde_json::json;
///
/// // 创建任务委派负载
/// let payload = CoordinationPayload::DelegateTask {
///     task_id: "task-001".to_string(),
///     summary: "处理用户请求".to_string(),
///     metadata: json!({"priority": "high"}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CoordinationPayload {
    /// 任务委派负载
    ///
    /// 用于将任务委派给目标代理执行。
    ///
    /// # 字段说明
    ///
    /// - `task_id`: 任务的唯一标识符
    /// - `summary`: 任务摘要描述
    /// - `metadata`: 任务元数据，可包含优先级、截止时间等额外信息
    ///
    /// # 投递约束
    ///
    /// 必须使用 `DeliveryScope::Direct` 模式投递。
    DelegateTask { task_id: String, summary: String, metadata: Value },

    /// 上下文补丁负载
    ///
    /// 用于更新代理间的共享上下文数据，支持乐观并发控制。
    ///
    /// # 字段说明
    ///
    /// - `key`: 上下文键名
    /// - `expected_version`: 预期的当前版本号，用于并发控制
    /// - `value`: 新的值
    ///
    /// # 并发控制
    ///
    /// 使用版本号机制防止并发更新冲突。如果实际版本与预期不匹配，更新将被拒绝。
    ContextPatch { key: String, expected_version: u64, value: Value },

    /// 任务结果负载
    ///
    /// 用于返回已委派任务的执行结果。
    ///
    /// # 字段说明
    ///
    /// - `task_id`: 对应任务的唯一标识符
    /// - `success`: 任务是否执行成功
    /// - `output`: 任务输出或错误信息
    ///
    /// # 关联约束
    ///
    /// 必须设置 `CoordinationEnvelope::correlation_id` 以关联原始委派消息。
    TaskResult { task_id: String, success: bool, output: String },

    /// 确认负载
    ///
    /// 用于确认已接收到特定消息。
    ///
    /// # 字段说明
    ///
    /// - `acked_message_id`: 被确认消息的唯一标识符
    Ack { acked_message_id: String },

    /// 控制指令负载
    ///
    /// 用于发送控制指令，如暂停、恢复、停止等。
    ///
    /// # 字段说明
    ///
    /// - `action`: 控制动作类型
    /// - `note`: 可选的备注说明
    Control { action: String, note: Option<String> },
}

/// 协调信封结构
///
/// 包装协调消息的完整元数据和负载，是代理间通信的标准信封格式。
///
/// # 字段说明
///
/// - `id`: 消息的唯一标识符（UUID）
/// - `conversation_id`: 会话标识符，用于关联同一会话的多条消息
/// - `correlation_id`: 关联标识符，用于关联请求-响应对
/// - `causation_id`: 因果标识符，用于追踪消息的因果关系链
/// - `from`: 发送者代理标识符
/// - `to`: 目标代理标识符（广播模式下为 `None`）
/// - `topic`: 消息主题，用于消息路由和过滤
/// - `scope`: 投递范围（点对点或广播）
/// - `payload`: 消息负载内容
///
/// # 消息溯源
///
/// 通过 `id`、`correlation_id` 和 `causation_id` 字段支持完整的消息溯源：
/// - `id`: 标识当前消息
/// - `correlation_id`: 关联到触发此消息的原始请求
/// - `causation_id`: 标识导致此消息产生的事件
///
/// # 示例
///
/// ```
/// use vibe_window::app::agent::coordination::envelope::{
///     CoordinationEnvelope, CoordinationPayload, DeliveryScope,
/// };
/// use serde_json::json;
///
/// // 创建点对点任务委派消息
/// let envelope = CoordinationEnvelope::new_direct(
///     "agent-001",
///     "agent-002",
///     "conv-123",
///     "task.delegation",
///     CoordinationPayload::DelegateTask {
///         task_id: "task-001".to_string(),
///         summary: "处理数据".to_string(),
///         metadata: json!({}),
///     },
/// );
///
/// // 验证消息
/// assert!(envelope.validate().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinationEnvelope {
    /// 消息唯一标识符
    ///
    /// 由 UUID v4 生成，确保全局唯一性。
    pub id: String,

    /// 会话标识符
    ///
    /// 用于关联属于同一会话或工作流的所有消息。
    pub conversation_id: String,

    /// 关联标识符（可选）
    ///
    /// 用于关联请求-响应模式中的消息对。
    /// 例如，任务结果的 `correlation_id` 应设置为原始任务委派的 `id`。
    pub correlation_id: Option<String>,

    /// 因果标识符（可选）
    ///
    /// 用于追踪消息的因果关系链，标识导致当前消息产生的事件。
    pub causation_id: Option<String>,

    /// 发送者代理标识符
    ///
    /// 消息的来源代理，不能为空。
    pub from: String,

    /// 目标代理标识符（可选）
    ///
    /// 点对点模式下为目标代理标识符，广播模式下必须为 `None`。
    pub to: Option<String>,

    /// 消息主题
    ///
    /// 用于消息路由和过滤的主题标识符，通常采用点分命名格式。
    /// 例如：`task.delegation`、`context.update`、`control.pause`
    pub topic: String,

    /// 投递范围
    ///
    /// 定义消息的分发模式，详见 [`DeliveryScope`]。
    pub scope: DeliveryScope,

    /// 消息负载
    ///
    /// 包含实际的消息内容，详见 [`CoordinationPayload`]。
    pub payload: CoordinationPayload,
}

impl CoordinationEnvelope {
    /// 创建点对点消息信封
    ///
    /// 构造一个投递范围为 `Direct` 的协调信封，用于发送给特定目标代理。
    ///
    /// # 参数说明
    ///
    /// - `from`: 发送者代理标识符，实现 `Into<String>` trait 的类型
    /// - `to`: 目标代理标识符，实现 `Into<String>` trait 的类型
    /// - `conversation_id`: 会话标识符，实现 `Into<String>` trait 的类型
    /// - `topic`: 消息主题，实现 `Into<String>` trait 的类型
    /// - `payload`: 消息负载内容
    ///
    /// # 返回值
    ///
    /// 返回新创建的 `CoordinationEnvelope` 实例，具有以下特征：
    /// - `id`: 自动生成的 UUID
    /// - `scope`: 设置为 `DeliveryScope::Direct`
    /// - `to`: 设置为 `Some(to)`
    /// - `correlation_id` 和 `causation_id`: 初始为 `None`，可后续设置
    ///
    /// # 示例
    ///
    /// ```
    /// use vibe_window::app::agent::coordination::envelope::{
    ///     CoordinationEnvelope, CoordinationPayload,
    /// };
    /// use serde_json::json;
    ///
    /// let envelope = CoordinationEnvelope::new_direct(
    ///     "agent-sender",
    ///     "agent-receiver",
    ///     "conversation-123",
    ///     "task.execute",
    ///     CoordinationPayload::DelegateTask {
    ///         task_id: "task-001".to_string(),
    ///         summary: "执行任务".to_string(),
    ///         metadata: json!({}),
    ///     },
    /// );
    ///
    /// assert!(envelope.to.is_some());
    /// assert_eq!(envelope.scope, DeliveryScope::Direct);
    /// ```
    pub fn new_direct(
        from: impl Into<String>,
        to: impl Into<String>,
        conversation_id: impl Into<String>,
        topic: impl Into<String>,
        payload: CoordinationPayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.into(),
            correlation_id: None,
            causation_id: None,
            from: from.into(),
            to: Some(to.into()),
            topic: topic.into(),
            scope: DeliveryScope::Direct,
            payload,
        }
    }

    /// 创建广播消息信封
    ///
    /// 构造一个投递范围为 `Broadcast` 的协调信封，用于发送给所有已注册代理。
    ///
    /// # 参数说明
    ///
    /// - `from`: 发送者代理标识符，实现 `Into<String>` trait 的类型
    /// - `conversation_id`: 会话标识符，实现 `Into<String>` trait 的类型
    /// - `topic`: 消息主题，实现 `Into<String>` trait 的类型
    /// - `payload`: 消息负载内容
    ///
    /// # 返回值
    ///
    /// 返回新创建的 `CoordinationEnvelope` 实例，具有以下特征：
    /// - `id`: 自动生成的 UUID
    /// - `scope`: 设置为 `DeliveryScope::Broadcast`
    /// - `to`: 设置为 `None`（广播无需指定目标）
    /// - `correlation_id` 和 `causation_id`: 初始为 `None`，可后续设置
    ///
    /// # 示例
    ///
    /// ```
    /// use vibe_window::app::agent::coordination::envelope::{
    ///     CoordinationEnvelope, CoordinationPayload,
    /// };
    ///
    /// let envelope = CoordinationEnvelope::new_broadcast(
    ///     "agent-controller",
    ///     "conversation-456",
    ///     "control.pause",
    ///     CoordinationPayload::Control {
    ///         action: "pause".to_string(),
    ///         note: Some("系统维护".to_string()),
    ///     },
    /// );
    ///
    /// assert!(envelope.to.is_none());
    /// assert_eq!(envelope.scope, DeliveryScope::Broadcast);
    /// ```
    pub fn new_broadcast(
        from: impl Into<String>,
        conversation_id: impl Into<String>,
        topic: impl Into<String>,
        payload: CoordinationPayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.into(),
            correlation_id: None,
            causation_id: None,
            from: from.into(),
            to: None,
            topic: topic.into(),
            scope: DeliveryScope::Broadcast,
            payload,
        }
    }

    /// 验证信封的有效性
    ///
    /// 在发布前验证传输层和负载层的协议约束，确保消息结构完整且符合规范。
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 验证通过，消息符合所有协议约束
    /// - `Err(CoordinationError)`: 验证失败，返回具体的错误信息
    ///
    /// # 验证规则
    ///
    /// ## 基础字段验证
    /// - `id`: 不能为空
    /// - `conversation_id`: 不能为空
    /// - `from`: 不能为空
    /// - `topic`: 不能为空
    ///
    /// ## 投递范围约束
    /// - `Direct` 模式：`to` 必须存在且非空
    /// - `Broadcast` 模式：`to` 必须为 `None`
    ///
    /// ## 可选字段约束
    /// - 如果设置了 `correlation_id`，则不能为空
    /// - 如果设置了 `causation_id`，则不能为空
    ///
    /// ## 负载特定约束
    /// - `DelegateTask`: 必须使用 `Direct` 模式，`task_id` 和 `summary` 不能为空
    /// - `ContextPatch`: `key` 不能为空
    /// - `TaskResult`: `task_id` 和 `output` 不能为空，必须设置 `correlation_id`
    /// - `Ack`: `acked_message_id` 不能为空
    /// - `Control`: `action` 不能为空
    ///
    /// # 错误类型
    ///
    /// - `CoordinationError::MissingTarget`: 点对点消息缺少目标
    /// - `CoordinationError::BroadcastHasTarget`: 广播消息不应指定目标
    /// - `CoordinationError::MissingCorrelationId`: 任务结果缺少关联 ID
    /// - `CoordinationError::InvalidDeliveryScope`: 投递范围与负载类型不匹配
    /// - `CoordinationError::EmptyField`: 必填字段为空
    ///
    /// # 示例
    ///
    /// ```
    /// use vibe_window::app::agent::coordination::envelope::{
    ///     CoordinationEnvelope, CoordinationPayload, DeliveryScope,
    /// };
    ///
    /// // 有效的消息
    /// let valid = CoordinationEnvelope::new_direct(
    ///     "agent-001",
    ///     "agent-002",
    ///     "conv-123",
    ///     "task.do",
    ///     CoordinationPayload::DelegateTask {
    ///         task_id: "t-001".to_string(),
    ///         summary: "任务描述".to_string(),
    ///         metadata: serde_json::json!({}),
    ///     },
    /// );
    /// assert!(valid.validate().is_ok());
    ///
    /// // 无效的消息（广播模式下设置了目标）
    /// let mut invalid = valid.clone();
    /// invalid.scope = DeliveryScope::Broadcast;
    /// invalid.to = Some("should-be-none".to_string());
    /// assert!(invalid.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), CoordinationError> {
        // 验证基础字段：确保核心标识符不为空
        require_non_empty(&self.id, "id")?;
        require_non_empty(&self.conversation_id, "conversation_id")?;
        require_non_empty(&self.from, "from")?;
        require_non_empty(&self.topic, "topic")?;

        // 根据投递范围验证目标字段
        match self.scope {
            // 点对点模式：必须指定有效的目标代理
            DeliveryScope::Direct => {
                // 去除空白字符后检查是否为空
                let target = self.to.as_deref().map(str::trim).filter(|value| !value.is_empty());
                if target.is_none() {
                    return Err(CoordinationError::MissingTarget { message_id: self.id.clone() });
                }
            }
            // 广播模式：不应指定目标代理
            DeliveryScope::Broadcast => {
                if self.to.is_some() {
                    return Err(CoordinationError::BroadcastHasTarget {
                        message_id: self.id.clone(),
                    });
                }
            }
        }

        // 验证可选字段：如果设置了，则不能为空字符串
        if let Some(correlation_id) = &self.correlation_id {
            require_non_empty(correlation_id, "correlation_id")?;
        }
        if let Some(causation_id) = &self.causation_id {
            require_non_empty(causation_id, "causation_id")?;
        }

        // 根据负载类型执行特定验证
        match &self.payload {
            // 任务委派：验证必填字段和投递范围约束
            CoordinationPayload::DelegateTask { task_id, summary, .. } => {
                require_non_empty(task_id, "task_id")?;
                require_non_empty(summary, "summary")?;
                // 任务委派必须使用点对点模式
                if self.scope != DeliveryScope::Direct {
                    return Err(CoordinationError::InvalidDeliveryScope {
                        message_id: self.id.clone(),
                        expected: DeliveryScope::Direct,
                        actual: self.scope,
                        payload: "delegate_task".to_string(),
                    });
                }
            }
            // 上下文补丁：验证键名
            CoordinationPayload::ContextPatch { key, .. } => {
                require_non_empty(key, "key")?;
            }
            // 任务结果：验证必填字段和关联约束
            CoordinationPayload::TaskResult { task_id, output, .. } => {
                require_non_empty(task_id, "task_id")?;
                require_non_empty(output, "output")?;
                // 任务结果必须关联到原始委派消息
                if self
                    .correlation_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .is_none()
                {
                    return Err(CoordinationError::MissingCorrelationId {
                        message_id: self.id.clone(),
                    });
                }
            }
            // 确认消息：验证被确认的消息 ID
            CoordinationPayload::Ack { acked_message_id } => {
                require_non_empty(acked_message_id, "acked_message_id")?;
            }
            // 控制指令：验证动作类型
            CoordinationPayload::Control { action, .. } => {
                require_non_empty(action, "action")?;
            }
        }

        Ok(())
    }
}
