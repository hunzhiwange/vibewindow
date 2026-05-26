//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::{Point, Rectangle, Size};

use super::super::dom::TailwindNode;
use super::tree::ComputedNodeLayout;

/// 执行 hit_test_node 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn hit_test_node(
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    point: Point,
    path_prefix: Vec<usize>,
) -> Option<(Vec<usize>, Size)> {
    if !layout.visual_rect.contains(point) && node.text.is_none() {
        return None;
    }

    for (idx, child_layout) in layout.children.iter().rev() {
        if child_layout.visual_rect.contains(point) {
            let mut child_path = path_prefix.clone();
            child_path.push(*idx);

            let child = &node.children[*idx];
            if let Some(found) = hit_test_node(child, child_layout, point, child_path.clone()) {
                return Some(found);
            }

            return Some((child_path, layout.size));
        }
    }

    Some((path_prefix, layout.size))
}

/// 执行 bounds_for_node_path 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn bounds_for_node_path(
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    remaining_path: &[usize],
) -> Option<(Rectangle, Size)> {
    if remaining_path.is_empty() {
        return Some((layout.visual_rect, layout.size));
    }

    let (idx, rest) = remaining_path.split_first()?;
    let (child_index, child_layout) =
        layout.children.iter().find(|(child_index, _)| child_index == idx)?;
    bounds_for_node_path(&node.children[*child_index], child_layout, rest)
}
