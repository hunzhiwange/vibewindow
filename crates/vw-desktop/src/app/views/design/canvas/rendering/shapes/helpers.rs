//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

use iced::{
    Color, Point, Rectangle, Size,
    widget::canvas::{Frame, Path, Stroke},
};

use crate::app::views::design::canvas::creation::BRUSH_STROKE_CLASS;
use crate::app::views::design::models::DesignElement;

/// 模块内部可见的 is_brush_path_class 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn is_brush_path_class(class_name: Option<&str>) -> bool {
    class_name.is_some_and(|class_name| {
        class_name.split_whitespace().any(|token| token == BRUSH_STROKE_CLASS)
    })
}

/// 模块内部可见的 draw_slot_hatch 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_slot_hatch(frame: &mut Frame, bounds: Rectangle, zoom: f32, color: Color) {
    let w = bounds.width.max(0.0);
    let h = bounds.height.max(0.0);
    if w <= f32::EPSILON || h <= f32::EPSILON {
        return;
    }

    let spacing = (8.0 * zoom).clamp(4.0, 18.0);
    let stroke_width = (1.0 * zoom).clamp(0.8, 2.0);
    let stroke = Stroke::default().with_color(color).with_width(stroke_width);

    let mut s = -h;
    while s <= w {
        let (sx, sy) = if s < 0.0 { (bounds.x, bounds.y - s) } else { (bounds.x + s, bounds.y) };
        let (ex, ey) = if s + h > w {
            (bounds.x + w, bounds.y + w - s)
        } else {
            (bounds.x + s + h, bounds.y + h)
        };
        frame.stroke(&Path::line(Point::new(sx, sy), Point::new(ex, ey)), stroke);
        s += spacing;
    }
}

/// 模块内部可见的 expand_slot_children 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn expand_slot_children(element: &DesignElement) -> Option<Vec<DesignElement>> {
    let Some(serde_json::Value::Array(arr)) = element.slot.as_ref() else {
        return None;
    };
    if arr.is_empty() {
        return None;
    }
    let ids: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
    if ids.is_empty() {
        return None;
    }

    Some(
        ids.into_iter()
            .enumerate()
            .map(|(i, slot_id)| DesignElement {
                kind: "ref".to_string(),
                id: format!("{}__slot__{i}", element.id),
                reference: Some(slot_id.to_string()),
                ..Default::default()
            })
            .collect(),
    )
}

/// 模块内部可见的 clamp_child_size_to_content 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn clamp_child_size_to_content(
    content_size: Size,
    child_x: f32,
    child_y: f32,
    child_size: Size,
) -> Size {
    let clipped_w = (content_size.width - child_x).max(0.0).min(child_size.width);
    let clipped_h = (content_size.height - child_y).max(0.0).min(child_size.height);
    Size::new(clipped_w, clipped_h)
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
