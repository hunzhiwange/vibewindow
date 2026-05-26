//! 协调总线死信队列管理模块
//!
//! 本模块提供死信队列的维护功能，用于存储和管理无法正常投递的协调消息。
//! 当消息因为各种原因（如目标不存在、路由失败等）无法投递时，会被转移到死信队列中。
//!
//! # 主要功能
//!
//! - 将无法投递的消息添加到死信队列
//! - 当队列满时自动淘汰最旧的消息
//! - 维护按关联ID（correlation_id）索引的死信记录，便于按事务追踪

use crate::app::agent::coordination::envelope::CoordinationEnvelope;
use crate::app::agent::coordination::state::BusState;
use crate::app::agent::coordination::types::DeadLetter;
use crate::app::agent::coordination::util::normalized_non_empty;

/// 将无法投递的消息添加到死信队列（需要在已持有锁的情况下调用）
///
/// 该函数将一个无法正常投递的协调消息及其失败原因添加到死信队列中。
/// 如果队列已达到容量上限，会先淘汰最旧的消息再添加新消息。
/// 同时会更新统计信息和按关联ID索引的辅助数据结构。
///
/// # 参数
///
/// - `state`: 总线状态的可变引用，包含死信队列和相关统计信息
/// - `envelope`: 无法投递的协调消息信封
/// - `reason`: 消息进入死信队列的原因描述
///
/// # 容量管理
///
/// 当死信队列已满时：
/// 1. 统计信息中的淘汰计数会增加
/// 2. 最旧的消息会从主队列中移除
/// 3. 如果被淘汰的消息有关联ID，也会从关联索引中移除对应的条目
///
/// # 索引维护
///
/// 如果新消息包含非空的关联ID：
/// - 消息会被添加到按关联ID索引的辅助映射中
/// - 支持按关联ID快速查找相关的死信记录，便于问题追踪和调试
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::coordination::bus_dead_letters::push_dead_letter_locked;
///
/// // 在持有状态锁的情况下调用
/// push_dead_letter_locked(
///     &mut bus_state,
///     envelope,
///     "目标处理器不存在".to_string(),
/// );
/// ```
pub(crate) fn push_dead_letter_locked(
    state: &mut BusState,
    envelope: CoordinationEnvelope,
    reason: String,
) {
    // 增加死信总数统计
    state.stats.dead_letters_total += 1;

    // 检查是否需要淘汰旧消息（队列已满）
    if state.dead_letters.len() >= state.limits.max_dead_letters {
        // 增加淘汰计数
        state.stats.dead_letter_evictions_total += 1;

        // 处理被淘汰消息的关联ID索引
        if let Some(evicted) = state.dead_letters.first() {
            // 如果被淘汰的消息有非空的关联ID，需要从关联索引中清理
            if let Some(correlation_id) =
                normalized_non_empty(evicted.envelope.correlation_id.as_deref())
            {
                let mut remove_correlation_key = false;

                // 从关联ID对应的死信列表中移除最早的一条
                if let Some(entries) = state.dead_letters_by_correlation.get_mut(correlation_id) {
                    let _ = entries.pop_front();
                    // 如果该关联ID下的死信列表已空，标记需要删除整个键
                    remove_correlation_key = entries.is_empty();
                }

                // 如果关联ID对应的列表已空，移除该键以节省空间
                if remove_correlation_key {
                    state.dead_letters_by_correlation.remove(correlation_id);
                }
            }
        }

        // 从主队列中移除最旧的消息（索引0）
        let _ = state.dead_letters.remove(0);
    }

    // 创建新的死信记录
    let dead_letter = DeadLetter { envelope, reason };

    // 如果消息有非空的关联ID，添加到关联索引中
    if let Some(correlation_id) =
        normalized_non_empty(dead_letter.envelope.correlation_id.as_deref())
    {
        state
            .dead_letters_by_correlation
            .entry(correlation_id.to_string())
            .or_default()
            .push_back(dead_letter.clone());
    }

    // 将死信添加到主队列
    state.dead_letters.push(dead_letter);
}
