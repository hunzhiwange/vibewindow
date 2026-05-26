//! 思维导图活动视图几何辅助逻辑，计算画布尺寸和屏幕坐标。

use crate::apps::mindmap::canvas::selected_node_rect_screen;
use crate::apps::mindmap::state::MindMapTab;
use iced::{Point, Rectangle, Size};

/// NodeToolbarLayout 数据结构，承载当前模块对外传递的显式状态。
pub(super) struct NodeToolbarLayout {
    pub(super) place_above: bool,
    pub(super) anchor: Point,
    pub(super) rect: Rectangle,
}

/// 构建或更新 selected node rect 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn selected_node_rect(tab: &MindMapTab) -> Option<Rectangle> {
    tab.selected_path.as_deref().and_then(|path| {
        selected_node_rect_screen(
            &tab.doc,
            &tab.node_positions,
            &tab.collapsed_paths,
            tab.pan,
            tab.zoom,
            path,
            tab.diagram_type,
            tab.layout_format,
            tab.org_chart_layout_format,
            tab.fishbone_layout_format,
            tab.timeline_layout_format,
            tab.bracket_layout_format,
            tab.tree_layout_format,
        )
    })
}

/// 构建或更新 node toolbar layout 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_toolbar_layout(
    tab: &MindMapTab,
    action_menu_y: f32,
    action_bar_h: f32,
    action_menu_gap: f32,
    node_toolbar_w: f32,
    node_toolbar_h: f32,
    node_toolbar_gap: f32,
) -> Option<NodeToolbarLayout> {
    selected_node_rect(tab).map(|node_rect| {
        let center_x = node_rect.x + node_rect.width / 2.0;
        let top_center = Point::new(center_x, node_rect.y);
        let bottom_center = Point::new(center_x, node_rect.y + node_rect.height);

        let safe_top_y = action_menu_y + action_bar_h + action_menu_gap;
        let toolbar_top_y = top_center.y - node_toolbar_gap - node_toolbar_h;
        let place_above = toolbar_top_y >= safe_top_y;

        let (anchor, rect) = if place_above {
            (
                top_center,
                Rectangle::new(
                    Point::new(
                        center_x - node_toolbar_w / 2.0,
                        top_center.y - node_toolbar_gap - node_toolbar_h,
                    ),
                    Size::new(node_toolbar_w, node_toolbar_h),
                ),
            )
        } else {
            (
                bottom_center,
                Rectangle::new(
                    Point::new(
                        center_x - node_toolbar_w / 2.0,
                        bottom_center.y + node_toolbar_gap,
                    ),
                    Size::new(node_toolbar_w, node_toolbar_h),
                ),
            )
        };

        NodeToolbarLayout { place_above, anchor, rect }
    })
}

/// 构建或更新 node action btn anchor 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_action_btn_anchor(
    toolbar_rect: Rectangle,
    node_btn_widths: &[f32],
    node_toolbar_padding: f32,
    edit_actions_w: f32,
    node_toolbar_group_gap: f32,
    node_toolbar_divider_w: f32,
    node_toolbar_btn_spacing: f32,
    btn_index: usize,
) -> Option<Point> {
    if btn_index >= node_btn_widths.len() {
        return None;
    }

    let actions_left_x = toolbar_rect.x
        + node_toolbar_padding
        + edit_actions_w
        + node_toolbar_group_gap
        + node_toolbar_divider_w
        + node_toolbar_group_gap;
    let toolbar_center_y = toolbar_rect.y + toolbar_rect.height / 2.0;

    let mut x_in_actions = 0.0;
    for width in node_btn_widths.iter().take(btn_index) {
        x_in_actions += width + node_toolbar_btn_spacing;
    }
    x_in_actions += node_btn_widths[btn_index] / 2.0;

    Some(Point::new(actions_left_x + x_in_actions, toolbar_center_y))
}

/// 构建或更新 node action btn top anchor 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_action_btn_top_anchor(
    toolbar_rect: Rectangle,
    node_btn_widths: &[f32],
    node_toolbar_padding: f32,
    edit_actions_w: f32,
    node_toolbar_group_gap: f32,
    node_toolbar_divider_w: f32,
    node_toolbar_btn_spacing: f32,
    btn_index: usize,
) -> Option<Point> {
    node_action_btn_anchor(
        toolbar_rect,
        node_btn_widths,
        node_toolbar_padding,
        edit_actions_w,
        node_toolbar_group_gap,
        node_toolbar_divider_w,
        node_toolbar_btn_spacing,
        btn_index,
    )
    .map(|point| Point::new(point.x, toolbar_rect.y))
}

/// 构建或更新 node action btn bottom left anchor 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_action_btn_bottom_left_anchor(
    toolbar_rect: Rectangle,
    node_btn_widths: &[f32],
    node_toolbar_padding: f32,
    edit_actions_w: f32,
    node_toolbar_group_gap: f32,
    node_toolbar_divider_w: f32,
    node_toolbar_btn_spacing: f32,
    btn_index: usize,
    overlay_w: f32,
) -> Option<Point> {
    node_action_btn_anchor(
        toolbar_rect,
        node_btn_widths,
        node_toolbar_padding,
        edit_actions_w,
        node_toolbar_group_gap,
        node_toolbar_divider_w,
        node_toolbar_btn_spacing,
        btn_index,
    )
    .map(|point| Point::new(point.x - overlay_w / 2.0, toolbar_rect.y + toolbar_rect.height))
}
