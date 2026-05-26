//! 管理聊天面板的分块加载状态。
//! 该模块把大消息流的可见窗口控制在组件内部，降低渲染压力。

use std::collections::HashSet;

use crate::app::session::{CHAT_UI_CHUNK_SIZE, chat_ui_chunk_bounds, chat_ui_chunk_start_idx};

const CHAT_CHUNK_RETAIN_NEIGHBOR_RADIUS: usize = 1;
const CHAT_PROTECTED_CHUNK_RETAIN_RADIUS: usize = 1;

fn normalized_window(chat_len: usize, start_idx: usize, end_idx: usize) -> Option<(usize, usize)> {
    if chat_len == 0 {
        return None;
    }

    let visible_start_idx = start_idx.min(chat_len.saturating_sub(1));
    let visible_end_idx = end_idx.max(visible_start_idx.saturating_add(1)).min(chat_len);
    Some((visible_start_idx, visible_end_idx))
}

fn chunk_starts_around(chat_len: usize, chunk_start_idx: usize, radius: usize) -> Vec<usize> {
    if chat_len == 0 {
        return Vec::new();
    }

    let normalized_chunk_start =
        chat_ui_chunk_start_idx(chunk_start_idx.min(chat_len.saturating_sub(1)));
    let center_chunk_idx = normalized_chunk_start / CHAT_UI_CHUNK_SIZE;
    let max_chunk_idx = chat_len.saturating_sub(1) / CHAT_UI_CHUNK_SIZE;
    let start_chunk_idx = center_chunk_idx.saturating_sub(radius);
    let end_chunk_idx = center_chunk_idx.saturating_add(radius).min(max_chunk_idx);

    (start_chunk_idx..=end_chunk_idx).map(|chunk_idx| chunk_idx * CHAT_UI_CHUNK_SIZE).collect()
}

/// 执行 retained_chunk_starts 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) fn retained_chunk_starts(
    chat_len: usize,
    start_idx: usize,
    end_idx: usize,
    protected_chunk_starts: &[usize],
) -> HashSet<usize> {
    let mut retained = HashSet::new();
    let Some((visible_start_idx, visible_end_idx)) =
        normalized_window(chat_len, start_idx, end_idx)
    else {
        return retained;
    };

    let mut chunk_start_idx = chat_ui_chunk_start_idx(visible_start_idx);
    while chunk_start_idx < visible_end_idx {
        retained.insert(chunk_start_idx);
        chunk_start_idx = chunk_start_idx.saturating_add(CHAT_UI_CHUNK_SIZE);
    }

    let first_visible_chunk_start = chat_ui_chunk_start_idx(visible_start_idx);
    let last_visible_chunk_start = chat_ui_chunk_start_idx(visible_end_idx.saturating_sub(1));
    for chunk_start_idx in
        chunk_starts_around(chat_len, first_visible_chunk_start, CHAT_CHUNK_RETAIN_NEIGHBOR_RADIUS)
    {
        retained.insert(chunk_start_idx);
    }
    for chunk_start_idx in
        chunk_starts_around(chat_len, last_visible_chunk_start, CHAT_CHUNK_RETAIN_NEIGHBOR_RADIUS)
    {
        retained.insert(chunk_start_idx);
    }

    for &chunk_start_idx in protected_chunk_starts {
        for retained_chunk_start in
            chunk_starts_around(chat_len, chunk_start_idx, CHAT_PROTECTED_CHUNK_RETAIN_RADIUS)
        {
            retained.insert(retained_chunk_start);
        }
    }

    retained
}

/// 执行 eviction_chunk_starts 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) fn eviction_chunk_starts(
    prepared_chunk_starts: &HashSet<usize>,
    retained_chunk_starts: &HashSet<usize>,
) -> Vec<usize> {
    let mut eviction_chunk_starts: Vec<_> = prepared_chunk_starts
        .iter()
        .copied()
        .filter(|chunk_start_idx| !retained_chunk_starts.contains(chunk_start_idx))
        .collect();
    eviction_chunk_starts.sort_unstable();
    eviction_chunk_starts
}

/// 执行 heavy_cache_keep_bounds 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) fn heavy_cache_keep_bounds(
    chat_len: usize,
    start_idx: usize,
    end_idx: usize,
    margin: usize,
    protected_chunk_starts: &[usize],
) -> (usize, usize) {
    let Some((visible_start_idx, visible_end_idx)) =
        normalized_window(chat_len, start_idx, end_idx)
    else {
        return (0, 0);
    };

    let mut keep_start = visible_start_idx.saturating_sub(margin);
    let mut keep_end = visible_end_idx.saturating_add(margin).min(chat_len);

    for &chunk_start_idx in protected_chunk_starts {
        let (chunk_start_idx, chunk_end_idx) = chat_ui_chunk_bounds(chat_len, chunk_start_idx);
        keep_start = keep_start.min(chunk_start_idx.saturating_sub(margin));
        keep_end = keep_end.max(chunk_end_idx.saturating_add(margin).min(chat_len));
    }

    (keep_start, keep_end)
}
