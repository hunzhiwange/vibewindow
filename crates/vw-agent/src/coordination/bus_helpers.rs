//! 协调总线的轻量计数辅助函数。
//!
//! 这些函数只维护 correlation 维度的消息计数，供 inbox 等模块在入队和出队时复用。
//! 逻辑保持局部化，避免把计数细节散落到多个状态更新路径。

use std::collections::HashMap;

use crate::app::agent::coordination::envelope::CoordinationEnvelope;
use crate::app::agent::coordination::util::normalized_non_empty;

/// 增加 envelope 所属 correlation 的计数。
///
/// 参数 `counts` 是待更新的 correlation 计数表，`envelope` 提供可选 correlation
/// id。空白或缺失的 correlation 会被忽略，函数不返回值。
pub(crate) fn increment_correlation_count(
    counts: &mut HashMap<String, usize>,
    envelope: &CoordinationEnvelope,
) {
    if let Some(correlation_id) = normalized_non_empty(envelope.correlation_id.as_deref()) {
        *counts.entry(correlation_id.to_string()).or_insert(0) += 1;
    }
}

/// 减少 envelope 所属 correlation 的计数。
///
/// 参数 `counts` 是待更新的计数表，`envelope` 提供 correlation id。计数降到零时会
/// 删除对应键，保持状态快照紧凑；缺失 correlation 时直接返回。
pub(crate) fn decrement_correlation_count(
    counts: &mut HashMap<String, usize>,
    envelope: &CoordinationEnvelope,
) {
    let Some(correlation_id) = normalized_non_empty(envelope.correlation_id.as_deref()) else {
        return;
    };

    let mut remove_key = false;
    if let Some(count) = counts.get_mut(correlation_id) {
        if *count <= 1 {
            remove_key = true;
        } else {
            *count -= 1;
        }
    }
    if remove_key {
        counts.remove(correlation_id);
    }
}
