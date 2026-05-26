//! 画布元素选择模块
//!
//! 本模块提供画布上基于矩形区域的元素选择功能。通过给定的选择矩形，
//! 递归遍历设计文档中的所有元素，查找与选择区域相交的元素并返回其 ID 列表。
//!
//! # 主要功能
//!
//! - 基于屏幕坐标的矩形区域选择
//! - 递归遍历嵌套的元素层级结构
//! - 智能过滤：在根层级排除框架类型的元素
//!
//! # 使用场景
//!
//! - 用户框选多个元素进行批量操作
//! - 实现拖拽选择工具
//! - 确定哪些元素位于特定的屏幕区域内

use super::super::super::models::{DesignDoc, DesignElement};
use super::super::geometry::get_element_screen_bounds;
use iced::{Point, Rectangle, Size, Vector};

/// 查找与选择矩形相交的所有元素 ID
///
/// 该函数遍历给定的元素列表，计算每个元素在屏幕上的边界矩形，
/// 并检查是否与选择矩形相交。相交的元素 ID 会被收集并返回。
///
/// # 参数
///
/// * `elements` - 要检查的设计元素切片，通常为文档的顶层元素
/// * `doc` - 设计文档的引用，用于获取元素的布局信息
/// * `pan` - 画布的平移偏移量（视图变换）
/// * `zoom` - 画布的缩放比例（视图变换）
/// * `selection_rect` - 选择矩形的屏幕坐标区域
///
/// # 返回值
///
/// 返回所有与选择矩形相交的元素 ID 向量，顺序为遍历顺序。
///
/// # 特殊行为
///
/// - 根层级的框架（frame）类型元素不会被包含在结果中
/// - 递归检查所有嵌套的子元素
///
/// # 示例
///
/// ```ignore
/// use iced::{Rectangle, Vector};
///
/// let selection = Rectangle::new(
///     Point::new(100.0, 100.0),
///     Size::new(200.0, 150.0),
/// );
/// let selected_ids = find_intersecting_ids(
///     &doc.elements,
///     &doc,
///     Vector::new(0.0, 0.0),
///     1.0,
///     selection,
/// );
/// ```
pub(super) fn find_intersecting_ids(
    elements: &[DesignElement],
    doc: &DesignDoc,
    pan: Vector,
    zoom: f32,
    selection_rect: Rectangle,
) -> Vec<String> {
    /// 递归遍历元素树并收集相交元素 ID
    ///
    /// # 参数
    ///
    /// * `elements` - 当前层级要检查的元素列表
    /// * `doc` - 设计文档引用
    /// * `pan` - 画布平移偏移量
    /// * `zoom` - 画布缩放比例
    /// * `selection_rect` - 选择矩形区域
    /// * `acc` - 累积器，用于收集匹配的元素 ID
    /// * `is_root` - 标识当前是否为根层级（顶层元素）
    fn rec(
        elements: &[DesignElement],
        doc: &DesignDoc,
        pan: Vector,
        zoom: f32,
        selection_rect: Rectangle,
        acc: &mut Vec<String>,
        is_root: bool,
    ) {
        for el in elements {
            // 获取元素在屏幕坐标系中的边界框
            if let Some(bounds) = get_element_screen_bounds(doc, &el.id, pan, zoom) {
                // 将边界转换为 iced Rectangle 类型以便进行相交检测
                let el_rect = Rectangle::new(
                    Point::new(bounds.x, bounds.y),
                    Size::new(bounds.width, bounds.height),
                );

                // 检查元素是否与选择区域相交
                // 同时排除根层级的框架元素（框架作为容器通常不应被直接选中）
                if selection_rect.intersection(&el_rect).is_some()
                    && !(is_root && el.kind == "frame")
                {
                    acc.push(el.id.clone());
                }
            }

            // 递归处理子元素，子元素层级不再是根层级
            rec(&el.children, doc, pan, zoom, selection_rect, acc, false);
        }
    }

    // 初始化结果向量并启动递归遍历
    let mut out = Vec::new();
    rec(elements, doc, pan, zoom, selection_rect, &mut out, true);
    out
}

#[cfg(test)]
#[path = "selection_tests.rs"]
mod selection_tests;
