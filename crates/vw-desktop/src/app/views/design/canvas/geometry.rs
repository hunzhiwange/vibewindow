//! 设计画布核心模块。
//!
//! 该模块承载节点创建、几何计算、命中测试与画布入口组织逻辑，是设计视图交互和渲染之间的核心边界。

use super::super::models::{DesignDoc, DesignElement};
use super::layout::resolve_element_size;
use super::utils::theme_mode_for_element;
use iced::{Point, Rectangle, Size, Vector};

/// 公开的 rotate_point 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn rotate_point(x: f32, y: f32, cx: f32, cy: f32, angle_rad: f32) -> (f32, f32) {
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    let dx = x - cx;
    let dy = y - cy;
    (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
}

/// 公开的 get_element_screen_bounds 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn get_element_screen_bounds(
    doc: &DesignDoc,
    target_id: &str,
    pan: Vector,
    zoom: f32,
) -> Option<Rectangle> {
    for child in &doc.children {
        if let Some(rect) =
            get_element_screen_bounds_recursive(child, doc, target_id, pan.x, pan.y, zoom, None)
        {
            return Some(rect);
        }
    }
    None
}

fn get_element_screen_bounds_recursive(
    element: &DesignElement,
    doc: &DesignDoc,
    target_id: &str,
    parent_x: f32,
    parent_y: f32,
    zoom: f32,
    inherited_theme_mode: Option<&str>,
) -> Option<Rectangle> {
    let x = parent_x + (element.x * zoom);
    let y = parent_y + (element.y * zoom);
    let resolved = resolve_element_size(element, None, doc, inherited_theme_mode);
    let w = resolved.width * zoom;
    let h = resolved.height * zoom;

    if element.id == target_id {
        return Some(Rectangle::new(Point::new(x, y), Size::new(w, h)));
    }

    let theme_mode = theme_mode_for_element(doc, element, inherited_theme_mode);
    for child in &element.children {
        if let Some(rect) =
            get_element_screen_bounds_recursive(child, doc, target_id, x, y, zoom, theme_mode)
        {
            return Some(rect);
        }
    }
    None
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod geometry_tests;
