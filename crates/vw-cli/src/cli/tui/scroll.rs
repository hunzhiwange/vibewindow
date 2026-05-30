//! 滚动条渲染模块
//!
//! 本模块提供了在终端用户界面（TUI）中渲染自定义滚动条的功能。
//! 滚动条用于可视化显示当前会话内容在可滚动区域中的位置，
//! 帮助用户了解当前视图在整体内容中的相对位置。
//!
//! # 主要功能
//!
//! - 计算滚动条位置和大小
//! - 渲染滚动条轨道和滑块
//! - 根据内容长度自适应调整滑块比例
//!
//! # 滚动条设计
//!
//! 滚动条采用简化的单列字符设计：
//! - 轨道使用 `│` 字符（暗灰色）
//! - 滑块使用 `█` 字符（青色表示活跃，暗灰色表示无滚动）
//!
//! 滑块大小与内容总长度成正比，遵循标准滚动条的视觉比例。

use crate::app::agent::agent::loop_::cli::theme::{SCROLLBAR_THUMB, SCROLLBAR_TRACK};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

fn u16_from_usize_saturating(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn u32_from_usize_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn usize_from_u64_saturating(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// 在指定区域渲染自定义滚动条
///
/// 根据会话内容行数和当前滚动位置，在终端界面的右侧渲染一个垂直滚动条。
/// 滚动条由轨道和滑块组成，滑块的位置和大小反映当前视图在整体内容中的位置。
///
/// # 参数
///
/// * `f` - ratatui 框实例，用于渲染组件
/// * `area` - 滚动条所在区域的矩形边界
/// * `conversation_lines` - 会话内容的所有行，用于计算滚动比例
/// * `effective_scroll` - 当前的有效滚动偏移量（以行为单位）
///
/// # 滚动条行为
///
/// - **无滚动情况**：当内容可完全显示在区域内时，渲染填充整个轨道的暗灰色滑块
/// - **有滚动情况**：渲染可移动的青色滑块，滑块大小与内容比例对应
///
/// # 算法说明
///
/// 滑块高度计算公式：
/// ```text
/// thumb_height = (inner_height² / total_lines)
/// ```
///
/// 滑块位置计算公式：
/// ```text
/// thumb_top = (effective_scroll * max_position) / base_scroll_for_bar
/// ```
///
/// # 示例
///
/// ```rust,ignore
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
///
/// // 假设有一个包含 100 行会话内容的终端
/// let conversation = vec![
///     "Line 1".to_string(),
///     "Line 2".to_string(),
///     // ... 共 100 行
/// ];
///
/// // 在区域 [x:10, y:0, width:50, height:20] 渲染滚动条
/// // 当前滚动位置为第 30 行
/// render_scrollbar(&mut frame, area, &conversation, 30);
/// ```
///
/// # 注意事项
///
/// - 区域高度至少需要 2 才能渲染（扣除边框后的内部高度至少为 1）
/// - 滑块最小高度为 1 行
/// - 滚动条宽度固定为 1 字符，位于区域右侧第二列
pub(crate) fn render_scrollbar(
    f: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    conversation_lines: &[String],
    effective_scroll: u16,
) {
    let inner_h = usize::from(area.height);

    // 获取会话内容的总行数
    let total_lines = conversation_lines.len();

    // 计算最大可滚动距离
    // 当内容行数超过内部高度时，差值即为需要滚动的总行数
    let base_scroll_for_bar = u16_from_usize_saturating(total_lines.saturating_sub(inner_h));

    // 如果内部高度为 0，无法渲染滚动条，直接返回
    if inner_h == 0 {
        return;
    }

    let bar_x = area.x.saturating_add(area.width.saturating_sub(1));

    let bar_y = area.y;

    // 初始化滚动条轨道
    // 创建一个由暗灰色 `│` 字符组成的向量，长度等于内部高度
    let mut bar =
        vec![
            Line::from(Span::styled("│", ratatui::style::Style::default().fg(SCROLLBAR_TRACK)));
            inner_h
        ];

    // 根据是否有滚动需求，采用不同的渲染策略
    if base_scroll_for_bar > 0 {
        // === 有滚动情况：计算并渲染可移动滑块 ===

        // 计算滑块高度
        // 使用平方公式：thumb_height = (inner_height² / total_lines)
        // 这确保滑块大小与内容在可视区域中的比例相对应
        // max(1) 确保滑块至少有 1 行高
        // min(inner_h) 确保滑块不会超过轨道高度
        let inner_h_u32 = u32_from_usize_saturating(inner_h);
        let total_lines_u32 = u32_from_usize_saturating(total_lines.max(1));
        let thumb_h = ((inner_h_u32 * inner_h_u32) / total_lines_u32).max(1).min(inner_h_u32);
        let thumb_h = usize::try_from(thumb_h).unwrap_or(inner_h);

        // 计算滑块可移动的最大位置
        // 即轨道高度减去滑块高度
        let max_pos = inner_h.saturating_sub(thumb_h);

        // 计算滑块顶部位置
        // 根据当前滚动偏移量按比例计算滑块在轨道中的位置
        // 使用 u64 避免中间计算溢出
        let thumb_top = if max_pos == 0 {
            // 如果滑块高度等于轨道高度，固定在顶部
            0
        } else {
            // 按比例计算位置：(当前滚动 / 总可滚动) * 最大位置
            let max_pos_u64 = u64::try_from(max_pos).unwrap_or(u64::MAX);
            usize_from_u64_saturating(
                u64::from(effective_scroll) * max_pos_u64 / u64::from(base_scroll_for_bar),
            )
        };

        // 将滑块范围内的轨道字符替换为青色 `█` 字符
        // saturating_add 和 min 确保不会越界
        for line in
            bar.iter_mut().take(thumb_top.saturating_add(thumb_h).min(inner_h)).skip(thumb_top)
        {
            *line =
                Line::from(Span::styled("█", ratatui::style::Style::default().fg(SCROLLBAR_THUMB)));
        }
    } else {
        // === 无滚动情况：渲染填充整个轨道的暗灰色滑块 ===
        // 当内容可完全显示时，滑块填充整个轨道，使用暗灰色表示非活跃状态
        for line in &mut bar {
            *line =
                Line::from(Span::styled("█", ratatui::style::Style::default().fg(SCROLLBAR_TRACK)));
        }
    }

    // 创建滚动条的渲染区域
    // 宽度固定为 1，高度等于内部高度
    let bar_area = ratatui::layout::Rect {
        x: bar_x,
        y: bar_y,
        width: 1,
        height: u16_from_usize_saturating(inner_h),
    };

    // 将滚动条渲染到帧上
    // 使用 Paragraph 组件将多行文本渲染到指定区域
    f.render_widget(Paragraph::new(Text::from(bar)), bar_area);
}
#[cfg(test)]
#[path = "scroll_tests.rs"]
mod scroll_tests;
