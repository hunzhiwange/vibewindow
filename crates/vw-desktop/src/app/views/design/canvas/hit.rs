//! 设计画布核心模块。
//!
//! 该模块承载节点创建、几何计算、命中测试与画布入口组织逻辑，是设计视图交互和渲染之间的核心边界。

use super::super::models::{DesignDoc, DesignElement};
use super::geometry::rotate_point;
use super::layout::resolve_element_size;
use super::types::Handle;
use super::utils::theme_mode_for_element;

/// 公开的 hit_test 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn hit_test(elements: &[DesignElement], doc: &DesignDoc, x: f32, y: f32) -> Option<String> {
    for element in elements.iter().rev() {
        if let Some(id) = hit_test_element(element, doc, x, y, 0.0, 0.0, true, None) {
            return Some(id);
        }
    }
    None
}

fn hit_test_element(
    element: &DesignElement,
    doc: &DesignDoc,
    target_x: f32,
    target_y: f32,
    parent_x: f32,
    parent_y: f32,
    _is_root: bool,
    inherited_theme_mode: Option<&str>,
) -> Option<String> {
    let el_x = parent_x + element.x;
    let el_y = parent_y + element.y;
    let resolved = resolve_element_size(element, None, doc, inherited_theme_mode);
    let w = resolved.width;
    let h = resolved.height;

    let rotation = element.rotation.unwrap_or(0.0);
    let (local_x, local_y) = if rotation != 0.0 {
        let center_x = el_x + w / 2.0;
        let center_y = el_y + h / 2.0;
        rotate_point(target_x, target_y, center_x, center_y, -rotation.to_radians())
    } else {
        (target_x, target_y)
    };

    let theme_mode = theme_mode_for_element(doc, element, inherited_theme_mode);
    for child in element.children.iter().rev() {
        if let Some(id) =
            hit_test_element(child, doc, local_x, local_y, el_x, el_y, false, theme_mode)
        {
            return Some(id);
        }
    }

    if local_x >= el_x && local_x <= el_x + w && local_y >= el_y && local_y <= el_y + h {
        return Some(element.id.clone());
    }

    None
}

/// 公开的 hit_test_handle 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn hit_test_handle(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    cursor_x: f32,
    cursor_y: f32,
    _zoom: f32,
) -> Option<Handle> {
    let handle_size = 10.0;
    let resize_hit_padding = 4.0;
    let rotate_inner = 12.0;
    let rotate_outer = 28.0;

    let check_rect = |left: f32, top: f32, size: f32| {
        cursor_x >= left - resize_hit_padding
            && cursor_x <= left + size + resize_hit_padding
            && cursor_y >= top - resize_hit_padding
            && cursor_y <= top + size + resize_hit_padding
    };

    let corner_left = x - handle_size / 2.0;
    let corner_top = y - handle_size / 2.0;
    let corner_right = x + w - handle_size / 2.0;
    let corner_bottom = y + h - handle_size / 2.0;
    let edge_mid_x = x + w / 2.0 - handle_size / 2.0;
    let edge_mid_y = y + h / 2.0 - handle_size / 2.0;

    if check_rect(corner_left, corner_top, handle_size) {
        return Some(Handle::TopLeft);
    }
    if check_rect(corner_right, corner_top, handle_size) {
        return Some(Handle::TopRight);
    }
    if check_rect(corner_left, corner_bottom, handle_size) {
        return Some(Handle::BottomLeft);
    }
    if check_rect(corner_right, corner_bottom, handle_size) {
        return Some(Handle::BottomRight);
    }

    if check_rect(edge_mid_x, y - handle_size / 2.0, handle_size) {
        return Some(Handle::Top);
    }
    if check_rect(edge_mid_x, y + h - handle_size / 2.0, handle_size) {
        return Some(Handle::Bottom);
    }
    if check_rect(x - handle_size / 2.0, edge_mid_y, handle_size) {
        return Some(Handle::Left);
    }
    if check_rect(x + w - handle_size / 2.0, edge_mid_y, handle_size) {
        return Some(Handle::Right);
    }

    let is_rotate_corner = |cx: f32, cy: f32| {
        let distance = (cursor_x - cx).hypot(cursor_y - cy);
        distance >= rotate_inner && distance <= rotate_outer
    };

    if is_rotate_corner(x, y) {
        return Some(Handle::RotateTopLeft);
    }
    if is_rotate_corner(x + w, y) {
        return Some(Handle::RotateTopRight);
    }
    if is_rotate_corner(x, y + h) {
        return Some(Handle::RotateBottomLeft);
    }
    if is_rotate_corner(x + w, y + h) {
        return Some(Handle::RotateBottomRight);
    }

    None
}

#[cfg(test)]
#[path = "hit_tests.rs"]
mod hit_tests;
