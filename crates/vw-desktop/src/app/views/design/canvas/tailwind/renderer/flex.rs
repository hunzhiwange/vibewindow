//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::Rectangle;

use super::super::parser::ParsedStyle;
use super::frame::ResolvedNodeFrame;

#[derive(Debug, Clone, Copy, Default)]
struct FlexMainAxisDistribution {
    leading_space: f32,
    between_gap: f32,
}

#[derive(Debug, Clone, Copy, Default)]
/// FlexItemConstraints 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct FlexItemConstraints {
    pub(super) grow: f32,
    pub(super) shrink: f32,
    pub(super) basis: Option<f32>,
}

/// 执行 is_reverse_flex_direction 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn is_reverse_flex_direction(style: &ParsedStyle) -> bool {
    matches!(style.flex_direction.as_deref(), Some("row-reverse") | Some("column-reverse"))
}

/// 执行 is_column_flex_direction 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn is_column_flex_direction(style: &ParsedStyle) -> bool {
    matches!(style.flex_direction.as_deref(), Some("column") | Some("column-reverse"))
}

/// 执行 is_row_flex_direction 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn is_row_flex_direction(style: &ParsedStyle) -> bool {
    matches!(style.flex_direction.as_deref(), Some("row") | Some("row-reverse"))
}

/// 执行 is_row_layout 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn is_row_layout(style: &ParsedStyle) -> bool {
    let is_flex = style.display.as_deref().map(|s| s.contains("flex")).unwrap_or(false);
    is_row_flex_direction(style) || (is_flex && !is_column_flex_direction(style))
}

/// 执行 effective_divide_x_reverse 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn effective_divide_x_reverse(style: &ParsedStyle) -> bool {
    style.divide_x_reverse ^ matches!(style.flex_direction.as_deref(), Some("row-reverse"))
}

/// 执行 effective_divide_y_reverse 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn effective_divide_y_reverse(style: &ParsedStyle) -> bool {
    style.divide_y_reverse ^ matches!(style.flex_direction.as_deref(), Some("column-reverse"))
}

/// 执行 apply_flex_item_constraints 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn apply_flex_item_constraints(
    children: &mut [(usize, Rectangle)],
    flow_child_indices: &[usize],
    constraints: &[FlexItemConstraints],
    is_row: bool,
    available_main: f32,
    base_gap: f32,
) -> f32 {
    if flow_child_indices.is_empty() {
        return 0.0;
    }

    for (out_idx, constraint) in flow_child_indices.iter().zip(constraints.iter()) {
        if let Some(basis) = constraint.basis {
            let rect = &mut children[*out_idx].1;
            set_rect_main_size(rect, is_row, basis);
        }
    }

    let gap_total = base_gap * flow_child_indices.len().saturating_sub(1) as f32;
    let mut used_main = gap_total;
    for out_idx in flow_child_indices {
        used_main += rect_main_size(&children[*out_idx].1, is_row);
    }

    if used_main < available_main {
        let total_grow: f32 = constraints.iter().map(|constraint| constraint.grow.max(0.0)).sum();
        if total_grow > 0.0 {
            let remaining = available_main - used_main;
            for (out_idx, constraint) in flow_child_indices.iter().zip(constraints.iter()) {
                if constraint.grow <= 0.0 {
                    continue;
                }

                let rect = &mut children[*out_idx].1;
                let size = rect_main_size(rect, is_row);
                let extra = remaining * (constraint.grow / total_grow);
                set_rect_main_size(rect, is_row, size + extra);
            }
        }
    } else if used_main > available_main {
        let deficit = used_main - available_main;
        let total_shrink_weight: f32 = flow_child_indices
            .iter()
            .zip(constraints.iter())
            .map(|(out_idx, constraint)| {
                rect_main_size(&children[*out_idx].1, is_row) * constraint.shrink.max(0.0)
            })
            .sum();

        if total_shrink_weight > 0.0 {
            for (out_idx, constraint) in flow_child_indices.iter().zip(constraints.iter()) {
                if constraint.shrink <= 0.0 {
                    continue;
                }

                let rect = &mut children[*out_idx].1;
                let size = rect_main_size(rect, is_row);
                let shrink_weight = size * constraint.shrink;
                let reduction = deficit * (shrink_weight / total_shrink_weight);
                set_rect_main_size(rect, is_row, (size - reduction).max(0.0));
            }
        }
    }

    gap_total
        + flow_child_indices
            .iter()
            .map(|out_idx| rect_main_size(&children[*out_idx].1, is_row))
            .sum::<f32>()
}

#[allow(clippy::too_many_arguments)]
/// 执行 apply_flex_alignment 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn apply_flex_alignment(
    children: &mut [(usize, Rectangle)],
    flow_child_indices: &[usize],
    flow_child_auto_cross_sizes: &[bool],
    draw_bounds: Rectangle,
    frame: &ResolvedNodeFrame,
    is_row: bool,
    is_reverse: bool,
    align_items: &str,
    justify_content: &str,
    content_main: f32,
) {
    if flow_child_indices.is_empty() {
        return;
    }

    let available_main = if is_row {
        (draw_bounds.width - frame.pl - frame.pr).max(0.0)
    } else {
        (draw_bounds.height - frame.pt - frame.pb).max(0.0)
    };
    let base_gap = if is_row { frame.gap_x } else { frame.gap_y };
    let distribution = resolve_flex_main_axis_distribution(
        justify_content,
        flow_child_indices.len(),
        available_main,
        content_main,
        base_gap,
    );

    let available_cross = if is_row {
        (draw_bounds.height - frame.pt - frame.pb).max(0.0)
    } else {
        (draw_bounds.width - frame.pl - frame.pr).max(0.0)
    };
    let mut cursor = if is_reverse {
        if is_row {
            draw_bounds.x + draw_bounds.width - frame.pr - distribution.leading_space
        } else {
            draw_bounds.y + draw_bounds.height - frame.pb - distribution.leading_space
        }
    } else if is_row {
        draw_bounds.x + frame.pl + distribution.leading_space
    } else {
        draw_bounds.y + frame.pt + distribution.leading_space
    };

    for (order, (out_idx, auto_cross_size)) in
        flow_child_indices.iter().zip(flow_child_auto_cross_sizes.iter()).enumerate()
    {
        let rect = &mut children[*out_idx].1;
        let main_size = if is_row { rect.width } else { rect.height };

        if is_reverse {
            cursor -= main_size;
            if is_row {
                rect.x = cursor;
            } else {
                rect.y = cursor;
            }
        } else if is_row {
            rect.x = cursor;
        } else {
            rect.y = cursor;
        }

        if align_items == "stretch" && *auto_cross_size {
            if is_row {
                rect.height = available_cross;
            } else {
                rect.width = available_cross;
            }
        } else if align_items == "center" {
            if is_row {
                rect.y = draw_bounds.y + frame.pt + (available_cross - rect.height) / 2.0;
            } else {
                rect.x = draw_bounds.x + frame.pl + (available_cross - rect.width) / 2.0;
            }
        } else if align_items == "flex-end" {
            if is_row {
                rect.y = draw_bounds.y + frame.pt + available_cross - rect.height;
            } else {
                rect.x = draw_bounds.x + frame.pl + available_cross - rect.width;
            }
        }

        if !is_reverse {
            cursor += main_size;
        }
        if order + 1 < flow_child_indices.len() {
            if is_reverse {
                cursor -= distribution.between_gap;
            } else {
                cursor += distribution.between_gap;
            }
        }
    }
}

fn rect_main_size(rect: &Rectangle, is_row: bool) -> f32 {
    if is_row { rect.width } else { rect.height }
}

fn set_rect_main_size(rect: &mut Rectangle, is_row: bool, value: f32) {
    if is_row {
        rect.width = value;
    } else {
        rect.height = value;
    }
}

fn resolve_flex_main_axis_distribution(
    justify_content: &str,
    item_count: usize,
    available_main: f32,
    content_main: f32,
    base_gap: f32,
) -> FlexMainAxisDistribution {
    let remaining_space = (available_main - content_main).max(0.0);

    match justify_content {
        "center" => {
            FlexMainAxisDistribution { leading_space: remaining_space / 2.0, between_gap: base_gap }
        }
        "flex-end" => {
            FlexMainAxisDistribution { leading_space: remaining_space, between_gap: base_gap }
        }
        "space-between" if item_count > 1 && remaining_space > 0.0 => FlexMainAxisDistribution {
            leading_space: 0.0,
            between_gap: base_gap + remaining_space / (item_count.saturating_sub(1) as f32),
        },
        _ => FlexMainAxisDistribution { leading_space: 0.0, between_gap: base_gap },
    }
}

#[cfg(test)]
#[path = "flex_tests.rs"]
mod flex_tests;
