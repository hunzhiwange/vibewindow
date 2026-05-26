//! 思维导图布局辅助函数，收敛节点遍历、尺寸估算和路径处理等局部逻辑。

use iced::Size;
use std::collections::HashMap;

/// 构建或更新 node size 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_size(text: &str, has_priority: bool, has_url: bool, is_root: bool) -> Size {
    let mut max_line_len = 1usize;
    let mut line_count = 0usize;
    for line in text.split('\n') {
        line_count += 1;
        max_line_len = max_line_len.max(line.chars().count());
    }
    let line_count = line_count.max(1) as f32;
    let max_line_len = max_line_len.clamp(1, 50) as f32;

    let (base_w, char_w, base_h, line_h) =
        if is_root { (40.0, 16.0, 50.0, 22.0) } else { (26.0, 12.0, 34.0, 18.0) };

    let mut w = base_w + max_line_len * char_w;
    let deco_w = 32.0;
    if has_priority {
        w += deco_w;
    }
    if has_url {
        w += deco_w;
    }

    let h = base_h + (line_count - 1.0) * line_h;
    Size::new(w.clamp(80.0, 600.0), h)
}

/// 构建或更新 has priority 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn has_priority(node_priorities: &HashMap<Vec<usize>, u8>, path: &[usize]) -> bool {
    node_priorities.get(path).copied().filter(|p| (1..=10).contains(p)).is_some()
}

/// 构建或更新 has url 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn has_url(node_urls: &HashMap<Vec<usize>, String>, path: &[usize]) -> bool {
    node_urls.get(path).is_some_and(|u| !u.trim().is_empty())
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
