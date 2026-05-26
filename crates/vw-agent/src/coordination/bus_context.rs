//! 协调总线共享上下文的写入与索引维护。
//!
//! 该模块只处理已经持有总线状态锁时的上下文 patch 操作。它同时维护全局顺序、
//! correlation 维度顺序以及 delegate 专用顺序，确保读取侧可以按不同视角稳定分页。

use serde_json::Value;

use crate::app::agent::coordination::envelope::CoordinationEnvelope;
use crate::app::agent::coordination::errors::CoordinationError;
use crate::app::agent::coordination::state::BusState;
use crate::app::agent::coordination::types::SharedContextEntry;
use crate::app::agent::coordination::util::{
    normalized_non_empty, parse_delegate_context_correlation_from_key,
};

/// 在持有状态锁时应用共享上下文 patch。
///
/// 参数 `state` 是待更新的总线状态，`envelope` 提供写入者和 correlation 信息，
/// `key` 标识上下文条目，`expected_version` 用于乐观并发控制，`value` 是新内容。
/// 返回 `Ok(())` 表示写入成功；版本不匹配、delegate key 非法或 correlation 不一致
/// 时返回 `CoordinationError`。
pub(crate) fn apply_context_patch_locked(
    state: &mut BusState,
    envelope: &CoordinationEnvelope,
    key: &str,
    expected_version: u64,
    value: &Value,
) -> Result<(), CoordinationError> {
    // delegate 上下文会被下游 agent 直接消费，因此 key 中的 correlation 必须与
    // envelope 一致，防止不同任务之间的上下文串线。
    let key_delegate_correlation = if key.starts_with("delegate/") {
        let parsed = parse_delegate_context_correlation_from_key(key).ok_or_else(|| {
            CoordinationError::InvalidDelegateContextKey {
                key: key.to_string(),
                message_id: envelope.id.clone(),
            }
        })?;
        let envelope_correlation = normalized_non_empty(envelope.correlation_id.as_deref())
            .ok_or_else(|| CoordinationError::MissingDelegateContextCorrelation {
                key: key.to_string(),
                message_id: envelope.id.clone(),
            })?;
        if parsed != envelope_correlation {
            return Err(CoordinationError::DelegateContextCorrelationMismatch {
                key: key.to_string(),
                message_id: envelope.id.clone(),
                key_correlation_id: parsed.to_string(),
                envelope_correlation_id: envelope_correlation.to_string(),
            });
        }
        Some(parsed)
    } else {
        None
    };

    let current_version = state.context.get(key).map_or(0, |entry| entry.version);
    if current_version != expected_version {
        return Err(CoordinationError::ContextVersionMismatch {
            key: key.to_string(),
            expected: expected_version,
            actual: current_version,
        });
    }

    let key_owned = key.to_string();
    let key_is_delegate = key_delegate_correlation.is_some();
    let previous_correlation = state.context_correlation_by_key.get(key).cloned();
    let is_new_key = !state.context.contains_key(key);
    if is_new_key && state.context.len() >= state.limits.max_context_entries {
        // 上下文容量达到上限时淘汰最旧条目，并同步清理所有辅助索引，避免分页读取到
        // 已不存在的 key。
        if let Some(evicted_key) = state.context_order.pop_front() {
            if state.context.remove(&evicted_key).is_some() {
                state.stats.context_evictions_total += 1;
            }
            let evicted_correlation = state.context_correlation_by_key.remove(&evicted_key);
            if let Some(correlation_id) = evicted_correlation.as_deref() {
                remove_key_from_context_correlation_order(state, correlation_id, &evicted_key);
            }
            if evicted_key.starts_with("delegate/") {
                remove_key_from_delegate_context_order(
                    state,
                    &evicted_key,
                    evicted_correlation.as_deref(),
                );
            }
        }
    }

    if !is_new_key {
        if let Some(position) = state.context_order.iter().position(|existing| existing == key) {
            let _ = state.context_order.remove(position);
        }
    }
    state.context_order.push_back(key_owned.clone());

    if let Some(correlation_id) = previous_correlation.as_deref() {
        remove_key_from_context_correlation_order(state, correlation_id, key);
    }
    if key_is_delegate {
        // delegate key 更新时需要从旧位置移除再追加到末尾，保证 recent 语义以最新
        // 写入时间为准。
        remove_key_from_delegate_context_order(state, key, previous_correlation.as_deref());
        state.delegate_context_order.push_back(key_owned.clone());
    }
    if let Some(correlation_id) = normalized_non_empty(envelope.correlation_id.as_deref()) {
        state
            .context_order_by_correlation
            .entry(correlation_id.to_string())
            .or_default()
            .push_back(key_owned.clone());
        if key_is_delegate {
            state
                .delegate_context_order_by_correlation
                .entry(correlation_id.to_string())
                .or_default()
                .push_back(key_owned.clone());
        }
        state.context_correlation_by_key.insert(key_owned.clone(), correlation_id.to_string());
    } else {
        state.context_correlation_by_key.remove(&key_owned);
    }

    state.context.insert(
        key_owned.clone(),
        SharedContextEntry {
            key: key_owned,
            value: value.clone(),
            version: current_version + 1,
            updated_by: envelope.from.clone(),
            last_message_id: envelope.id.clone(),
        },
    );

    Ok(())
}

/// 从指定 correlation 的上下文顺序索引中移除一个 key。
///
/// 参数 `state` 是已加锁的总线状态，`correlation_id` 是索引分组，`key` 是待移除
/// 的上下文 key。函数无返回值；当分组变空时会删除分组入口。
fn remove_key_from_context_correlation_order(
    state: &mut BusState,
    correlation_id: &str,
    key: &str,
) {
    let mut remove_correlation_key = false;
    if let Some(order) = state.context_order_by_correlation.get_mut(correlation_id) {
        if let Some(position) = order.iter().position(|existing| existing == key) {
            let _ = order.remove(position);
        }
        remove_correlation_key = order.is_empty();
    }
    if remove_correlation_key {
        state.context_order_by_correlation.remove(correlation_id);
    }
}

/// 从 delegate 上下文的全局和 correlation 顺序索引中移除一个 key。
///
/// 参数 `state` 是已加锁的总线状态，`key` 是待移除的 delegate 上下文 key，
/// `correlation_id` 为可选分组。函数无返回值；缺少 correlation 时只清理全局索引。
fn remove_key_from_delegate_context_order(
    state: &mut BusState,
    key: &str,
    correlation_id: Option<&str>,
) {
    if let Some(position) = state.delegate_context_order.iter().position(|existing| existing == key)
    {
        let _ = state.delegate_context_order.remove(position);
    }

    let Some(correlation_id) = correlation_id else {
        return;
    };

    let mut remove_correlation_key = false;
    if let Some(order) = state.delegate_context_order_by_correlation.get_mut(correlation_id) {
        if let Some(position) = order.iter().position(|existing| existing == key) {
            let _ = order.remove(position);
        }
        remove_correlation_key = order.is_empty();
    }
    if remove_correlation_key {
        state.delegate_context_order_by_correlation.remove(correlation_id);
    }
}
