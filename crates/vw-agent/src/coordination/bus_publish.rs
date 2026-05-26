//! # 协调总线消息发布模块
//!
//! 本模块提供协调总线的消息发布功能，是 VibeWindow 代理间通信的核心组件之一。
//!
//! ## 主要功能
//!
//! - **消息验证**：验证信封格式的合法性，确保必填字段存在且有效
//! - **去重处理**：基于消息 ID 的幂等性检查，防止重复消息污染系统
//! - **上下文补丁应用**：处理上下文更新类型的消息，维护共享状态一致性
//! - **投递路由**：支持直接投递（Direct）和广播投递（Broadcast）两种模式
//! - **死信处理**：将无效或无法投递的消息转移到死信队列，便于审计和恢复
//! - **统计追踪**：记录发布尝试、投递次数、溢出驱逐等运行时指标
//!
//! ## 投递模式
//!
//! - `Direct`：单播模式，将消息投递给指定的目标代理
//! - `Broadcast`：广播模式，将消息投递给所有已注册的代理
//!
//! ## 溢出处理策略
//!
//! 当收件箱达到容量上限时，采用 FIFO（先进先出）策略驱逐最旧的消息，
//! 并将被驱逐的消息转移到死信队列以保留审计轨迹。

use crate::app::agent::coordination::bus_context::apply_context_patch_locked;
use crate::app::agent::coordination::bus_dead_letters::push_dead_letter_locked;
use crate::app::agent::coordination::bus_inbox::push_inbox_entry_locked;
use crate::app::agent::coordination::envelope::{
    CoordinationEnvelope, CoordinationPayload, DeliveryScope,
};
use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::state::BusState;
use crate::app::agent::coordination::types::{PublishReceipt, SequencedEnvelope};

/// 将协调信封发布到总线并进行投递
///
/// 该函数是消息发布的主入口点，负责完成消息的完整生命周期处理：
/// 验证、去重、序列化、路由和投递。
///
/// # 参数
///
/// - `state`：总线的可变状态引用，包含收件箱、死信队列、去重集合等
/// - `envelope`：待发布的协调信封，包含消息元数据和载荷
///
/// # 返回值
///
/// - `Ok(PublishReceipt)`：发布成功，包含分配的序列号和实际投递的目标数量
/// - `Err(CoordinationError)`：发布失败，具体错误类型包括：
///   - `ValidationError`：信封格式验证失败
///   - `DuplicateMessageId`：消息 ID 重复
///   - `ContextPatchConflict`：上下文补丁版本冲突
///   - `UnknownTarget`：直接投递模式下目标代理不存在
///
/// # 处理流程
///
/// 1. **验证阶段**：检查信封格式的合法性
/// 2. **去重阶段**：基于消息 ID 检查是否已处理过
/// 3. **LRU 维护**：必要时驱逐最旧的已知消息 ID
/// 4. **上下文处理**：若为上下文补丁载荷，应用版本化更新
/// 5. **序列分配**：为消息分配全局递增序列号
/// 6. **投递路由**：根据投递作用域执行直接或广播投递
/// 7. **统计更新**：累加相关的运行时统计指标
///
/// # 失败处理
///
/// 任何阶段的失败都会导致消息被转移到死信队列，并返回相应的错误。
/// 这确保了系统在任何异常情况下都能保留完整的审计轨迹。
///
/// # 示例
///
/// ```rust,ignore
/// use crate::app::agent::coordination::bus_publish::publish_envelope;
/// use crate::app::agent::coordination::envelope::{CoordinationEnvelope, DeliveryScope};
///
/// let envelope = CoordinationEnvelope {
///     id: "msg-001".to_string(),
///     from: Some("agent-a".to_string()),
///     to: Some("agent-b".to_string()),
///     scope: DeliveryScope::Direct,
///     // ... 其他字段
/// };
///
/// match publish_envelope(&mut state, envelope) {
///     Ok(receipt) => {
///         println!("消息已发布，序列号: {}, 投递数: {}",
///             receipt.sequence, receipt.delivered_to);
///     }
///     Err(e) => {
///         eprintln!("发布失败: {}", e);
///     }
/// }
/// ```
pub(crate) fn publish_envelope(
    state: &mut BusState,
    envelope: CoordinationEnvelope,
) -> Result<PublishReceipt, CoordinationError> {
    // === 第一阶段：信封格式验证 ===
    // 验证信封的必填字段和格式约束，确保消息的基本合法性
    if let Err(error) = envelope.validate() {
        // 验证失败的消息转移到死信队列，保留审计轨迹
        push_dead_letter_locked(state, envelope, error.to_string());
        return Err(error);
    }

    // === 第二阶段：消息去重检查 ===
    // 增加发布尝试计数器，用于监控系统负载
    state.stats.publish_attempts_total += 1;

    // 检查消息 ID 是否已被处理过，实现幂等性保证
    if state.seen_message_ids.contains(&envelope.id) {
        let error = CoordinationError::DuplicateMessageId { message_id: envelope.id.clone() };
        // 重复消息转移到死信队列，避免重复处理
        push_dead_letter_locked(state, envelope, error.to_string());
        return Err(error);
    }

    // === 第三阶段：LRU 缓存维护 ===
    // 当已知消息 ID 集合达到上限时，执行 LRU 驱逐策略
    if state.seen_message_ids.len() >= state.limits.max_seen_message_ids {
        // 从有序队列前端移除最旧的消息 ID
        if let Some(evicted_id) = state.seen_message_order.pop_front() {
            // 从哈希集合中删除对应的 ID
            if state.seen_message_ids.remove(&evicted_id) {
                // 记录驱逐事件，用于监控缓存淘汰频率
                state.stats.seen_message_id_evictions_total += 1;
            }
        }
    }

    // 将当前消息 ID 添加到去重集合和有序队列
    state.seen_message_ids.insert(envelope.id.clone());
    state.seen_message_order.push_back(envelope.id.clone());

    // === 第四阶段：上下文补丁处理 ===
    // 若消息载荷为上下文补丁类型，执行版本化的状态更新
    if let CoordinationPayload::ContextPatch { key, expected_version, value } = &envelope.payload {
        // 应用上下文补丁，会验证版本号并执行乐观锁检查
        if let Err(error) =
            apply_context_patch_locked(state, &envelope, key, *expected_version, value)
        {
            // 上下文更新失败（如版本冲突）时转移至死信队列
            push_dead_letter_locked(state, envelope, error.to_string());
            return Err(error);
        }
    }

    // === 第五阶段：序列号分配 ===
    // 分配全局序列号后再递增，保持 `next_sequence` 表示下一个可用序号。
    let sequence = state.next_sequence;
    state.next_sequence += 1;

    // 创建带序列号的信封，用于后续的投递追踪
    let sequenced = SequencedEnvelope { sequence, envelope: envelope.clone() };

    // === 第六阶段：投递路由与执行 ===
    // 根据投递作用域选择对应的投递策略
    let delivered_to = match envelope.scope {
        // 直接投递模式：单播到指定目标代理
        DeliveryScope::Direct => {
            // 获取目标代理标识（验证阶段已确保存在）
            let target = envelope.to.as_deref().expect("validated direct target");

            // 检查目标代理是否已注册
            if !state.inboxes.contains_key(target) {
                let error = CoordinationError::UnknownTarget {
                    agent: target.to_string(),
                    message_id: envelope.id.clone(),
                };
                // 目标不存在时转移至死信队列
                push_dead_letter_locked(state, envelope, error.to_string());
                return Err(error);
            }

            // 将消息推送到目标代理的收件箱
            // 返回值表示是否因溢出而驱逐了旧消息
            let dropped = push_inbox_entry_locked(state, target, sequenced);

            // 处理收件箱溢出情况
            if let Some(dropped) = dropped {
                // 被驱逐的消息转移到死信队列，保留审计轨迹
                push_dead_letter_locked(
                    state,
                    dropped,
                    format!("inbox overflow: dropped oldest message for agent '{target}'"),
                );
            }

            // 直接投递模式下，成功投递的目标数为 1
            1
        }

        // 广播投递模式：投递给所有已注册的代理
        DeliveryScope::Broadcast => {
            // 若无已注册代理，则无需投递
            if state.inboxes.is_empty() {
                0
            } else {
                // 记录扇出数量（目标代理数）
                let fanout = state.inboxes.len();

                // 收集因溢出而被驱逐的消息，批量处理以提高效率
                let mut dropped_items: Vec<(String, CoordinationEnvelope)> = Vec::new();

                // 收集所有代理标识，避免在迭代时持有不可变借用
                let agents = state.inboxes.keys().cloned().collect::<Vec<_>>();

                // 遍历所有代理，将消息推送到各自的收件箱
                for agent in &agents {
                    if let Some(dropped) = push_inbox_entry_locked(state, agent, sequenced.clone())
                    {
                        // 记录被驱逐的消息及其所属代理
                        dropped_items.push((agent.clone(), dropped));
                    }
                }

                // 批量处理溢出驱逐的消息
                for (agent, dropped) in dropped_items {
                    // 被驱逐的消息转移到死信队列
                    push_dead_letter_locked(
                        state,
                        dropped,
                        format!("inbox overflow: dropped oldest message for agent '{agent}'"),
                    );
                }

                // 返回实际投递的目标数量
                fanout
            }
        }
    };

    // === 第七阶段：统计更新 ===
    // 累加成功投递的总数
    state.stats.deliveries_total += delivered_to as u64;

    // 返回发布回执，包含序列号和投递统计
    Ok(PublishReceipt { sequence, delivered_to })
}
