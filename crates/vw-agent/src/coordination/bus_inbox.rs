//! 总线收件箱管理模块
//!
//! 本模块提供代理收件箱的核心操作功能，负责管理消息的推送、存储和淘汰。
//! 主要职责包括：
//! - 向指定代理的收件箱推送消息条目
//! - 维护收件箱容量限制（FIFO淘汰策略）
//! - 管理消息相关性计数以确保准确的消息跟踪
//!
//! 该模块是协调系统（coordination）的关键组件，确保消息在各代理间可靠传递。

use crate::app::agent::coordination::bus_helpers::{
    decrement_correlation_count, increment_correlation_count,
};
use crate::app::agent::coordination::envelope::CoordinationEnvelope;
use crate::app::agent::coordination::state::BusState;
use crate::app::agent::coordination::types::SequencedEnvelope;

/// 向指定代理的收件箱推送已加锁的消息条目
///
/// 该函数在已持有状态锁的情况下执行，将新的消息条目添加到目标代理的收件箱中。
/// 如果收件箱已达到容量上限，将采用先进先出（FIFO）策略淘汰最旧的消息。
///
/// # 参数
///
/// * `state` - 总线状态的可变引用，包含所有代理的收件箱和相关计数器
/// * `agent` - 目标代理的标识符字符串
/// * `entry` - 待推送的已序列化消息信封
///
/// # 返回值
///
/// 返回 `Option<CoordinationEnvelope>`：
/// - `Some(envelope)` - 如果收件箱已满，返回被淘汰的消息信封
/// - `None` - 如果收件箱未满，无需淘汰任何消息
///
/// # 前置条件
///
/// 调用此函数前必须确保：
/// - 目标代理已存在于系统中（通过 `state.inboxes` 验证）
/// - 调用方已持有状态锁
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::coordination::bus_inbox::push_inbox_entry_locked;
///
/// let mut state = get_bus_state();
/// let agent_id = "agent_001";
/// let entry = create_sequenced_envelope();
///
/// // 推送消息，可能返回被淘汰的旧消息
/// let dropped = push_inbox_entry_locked(&mut state, agent_id, entry);
/// if let Some(old_envelope) = dropped {
///     println!("淘汰旧消息: {:?}", old_envelope);
/// }
/// ```
///
/// # 并发安全
///
/// 此函数本身不提供锁机制，调用方需要确保在调用期间持有适当的锁。
pub(crate) fn push_inbox_entry_locked(
    state: &mut BusState,
    agent: &str,
    entry: SequencedEnvelope,
) -> Option<CoordinationEnvelope> {
    // 获取每个代理的最大收件箱消息数量限制
    let max_inbox_messages_per_agent = state.limits.max_inbox_messages_per_agent;

    // 分别获取收件箱映射表和相关性计数映射表的可变引用
    let (inboxes, correlation_counts_by_agent) =
        (&mut state.inboxes, &mut state.inbox_correlation_counts);

    // 获取目标代理的收件箱（调用前应已验证代理存在）
    let inbox = inboxes
        .get_mut(agent)
        .expect("agent existence should be validated before pushing inbox entry");

    // 获取或创建该代理的相关性计数器
    let correlation_counts = correlation_counts_by_agent.entry(agent.to_string()).or_default();

    // 检查收件箱是否已达到容量上限
    // 如果已满，从队首移除最旧的消息（FIFO策略）
    let dropped =
        if inbox.len() >= max_inbox_messages_per_agent { inbox.pop_front() } else { None };

    // 如果有消息被淘汰，减少其相关性计数
    if let Some(dropped_entry) = dropped.as_ref() {
        decrement_correlation_count(correlation_counts, &dropped_entry.envelope);
        state.stats.inbox_overflow_evictions_total += 1;
    }

    // 增加新消息的相关性计数
    increment_correlation_count(correlation_counts, &entry.envelope);

    // 将新消息添加到收件箱队尾
    inbox.push_back(entry);

    // 返回被淘汰的消息信封（如果有）
    dropped.map(|value| value.envelope)
}
