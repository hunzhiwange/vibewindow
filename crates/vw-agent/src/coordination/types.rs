//! 协调系统核心类型定义模块
//!
//! 本模块定义了多代理协调系统的核心数据结构，包括：
//! - 消息序列化包装器（`SequencedEnvelope`）
//! - 死信队列项（`DeadLetter`）
//! - 共享上下文条目（`SharedContextEntry`）
//! - 发布回执（`PublishReceipt`）
//! - 内存消息总线容量限制（`InMemoryMessageBusLimits`）
//! - 运行时统计信息（`InMemoryMessageBusStats`）
//!
//! # 主要功能
//!
//! 1. **消息序列化**：为总线发出的每条消息分配全局递增序列号
//! 2. **死信管理**：保留投递失败的消息用于审计与调试
//! 3. **共享状态**：维护代理间可共享的版本化上下文
//! 4. **容量控制**：限制各类资源的最大容量以防止资源耗尽
//! 5. **运行监控**：提供操作可见性的统计计数器

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app::agent::coordination::envelope::CoordinationEnvelope;

/// 序列化信封，表示消息总线发出的带序列号的消息
///
/// 每条通过总线分发的消息都会被分配一个全局递增的序列号，
/// 用于保证消息顺序性、支持断点续传和精确一次语义。
///
/// # 字段说明
///
/// - `sequence`: 全局递增的序列号，从 0 开始
/// - `envelope`: 实际的协调信封内容
#[derive(Debug, Clone)]
pub struct SequencedEnvelope {
    pub sequence: u64,
    pub envelope: CoordinationEnvelope,
}

/// 死信项，保留投递失败的消息用于审计和调试
///
/// 当消息无法被成功投递到目标代理时，会被记录为死信。
/// 死信队列保留失败消息的完整内容和失败原因，
/// 便于后续问题排查和消息重试。
///
/// # 字段说明
///
/// - `envelope`: 投递失败的消息信封
/// - `reason`: 失败原因描述
#[derive(Debug, Clone)]
pub struct DeadLetter {
    pub envelope: CoordinationEnvelope,
    pub reason: String,
}

/// 版本化共享上下文条目，通过 `ContextPatch` 写入
///
/// 共享上下文提供代理间的状态共享机制。每个条目都包含版本号，
/// 支持乐观并发控制。当多个代理尝试更新同一个键时，
/// 版本号用于检测和解决冲突。
///
/// # 字段说明
///
/// - `key`: 上下文键名，用于索引和查询
/// - `value`: 上下文值，采用 JSON 格式以支持复杂数据结构
/// - `version`: 版本号，每次更新时递增
/// - `updated_by`: 最后一次更新的代理标识
/// - `last_message_id`: 触发此次更新的消息 ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedContextEntry {
    pub key: String,
    pub value: Value,
    pub version: u64,
    pub updated_by: String,
    pub last_message_id: String,
}

/// 发布回执，包含消息发布后的元数据
///
/// 当消息成功发布到总线后，返回此结构体以提供发布结果信息，
/// 包括分配的序列号和实际投递的代理数量。
///
/// # 字段说明
///
/// - `sequence`: 分配给此消息的全局序列号
/// - `delivered_to`: 成功投递到的代理数量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishReceipt {
    pub sequence: u64,
    pub delivered_to: usize,
}

/// `InMemoryMessageBus` 容量限制配置
///
/// 定义内存消息总线中各类资源的最大容量限制，
/// 用于防止资源耗尽和控制系统内存占用。
///
/// # 限制说明
///
/// - `max_inbox_messages_per_agent`: 每个代理收件箱的最大消息数
/// - `max_dead_letters`: 死信队列的最大条目数
/// - `max_context_entries`: 共享上下文的最大条目数
/// - `max_seen_message_ids`: 幂等性去重窗口的最大消息 ID 数
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct InMemoryMessageBusLimits {
    pub max_inbox_messages_per_agent: usize,
    pub max_dead_letters: usize,
    pub max_context_entries: usize,
    pub max_seen_message_ids: usize,
}

impl InMemoryMessageBusLimits {
    /// 返回默认的容量限制配置
    ///
    /// # 默认值
    ///
    /// - 每代理收件箱：256 条消息
    /// - 死信队列：256 条记录
    /// - 共享上下文：512 个条目
    /// - 幂等性窗口：4096 个消息 ID
    ///
    /// # 返回值
    ///
    /// 使用推荐默认值初始化的 `InMemoryMessageBusLimits` 实例
    ///
    /// # 示例
    ///
    /// ```
    /// use vibe_window::app::agent::coordination::types::InMemoryMessageBusLimits;
    ///
    /// let limits = InMemoryMessageBusLimits::default();
    /// assert_eq!(limits.max_inbox_messages_per_agent, 256);
    /// assert_eq!(limits.max_dead_letters, 256);
    /// ```
    pub fn recommended() -> Self {
        Self {
            max_inbox_messages_per_agent: 256,
            max_dead_letters: 256,
            max_context_entries: 512,
            max_seen_message_ids: 4096,
        }
    }
}

impl Default for InMemoryMessageBusLimits {
    fn default() -> Self {
        Self::recommended()
    }
}

/// 运行时统计计数器，用于操作可见性监控
///
/// 跟踪消息总线的各种运行时指标，包括消息投递、
/// 资源淘汰等关键事件，便于监控和调试。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct InMemoryMessageBusStats {
    /// 通过信封验证的总发布尝试次数
    pub publish_attempts_total: u64,
    /// 总成功投递次数（广播时为扇出计数总和）
    pub deliveries_total: u64,
    /// 因收件箱容量限制而被驱逐的消息数
    pub inbox_overflow_evictions_total: u64,
    /// 有史以来记录的死信条目总数
    pub dead_letters_total: u64,
    /// 因死信容量上限而被驱逐的死信条目数
    pub dead_letter_evictions_total: u64,
    /// 因上下文容量限制而被驱逐的共享上下文条目数
    pub context_evictions_total: u64,
    /// 因去重窗口容量限制而被驱逐的幂等性 ID 数
    pub seen_message_id_evictions_total: u64,
}
