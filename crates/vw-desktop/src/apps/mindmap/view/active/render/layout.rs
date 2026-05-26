//! 思维导图活动视图布局渲染逻辑，组织工具区、画布和侧边面板的位置。

use crate::apps::mindmap::state::{MindMapColorTarget, MindMapTab};
use iced::{Point, Rectangle, Size};

/// RenderLayout 数据结构，承载当前模块对外传递的显式状态。
pub(super) struct RenderLayout {
    pub(super) default_side_anchor: Point,
    pub(super) priority_picker_anchor: Point,
    pub(super) url_editor_anchor: Point,
    pub(super) zoom_menu_anchor: Point,
    pub(super) ui_blocked_rects: Vec<Rectangle>,
}

/// LayoutInputs 数据结构，承载当前模块对外传递的显式状态。
pub(super) struct LayoutInputs {
    pub(super) action_menu_x: f32,
    pub(super) action_menu_y: f32,
    pub(super) action_bar_w: f32,
    pub(super) action_bar_h: f32,
    pub(super) action_menu_gap: f32,
    pub(super) action_menu_w: f32,
    pub(super) action_menu_h: f32,
    pub(super) picker_left_gap: f32,
    pub(super) picker_top_gap: f32,
    pub(super) zoom_panel_margin: f32,
    pub(super) zoom_control_w: f32,
    pub(super) zoom_control_h: f32,
    pub(super) zoom_menu_gap: f32,
    pub(super) zoom_menu_padding: f32,
    pub(super) zoom_menu_item_h: f32,
    pub(super) zoom_menu_spacing: f32,
    pub(super) zoom_menu_item_count: usize,
    pub(super) bg_panel_w: f32,
    pub(super) bg_panel_h: f32,
    pub(super) bg_panel_margin: f32,
    pub(super) diagram_panel_w: f32,
    pub(super) diagram_panel_h: f32,
    pub(super) diagram_panel_margin: f32,
    pub(super) bg_color_panel_w: f32,
    pub(super) bg_color_panel_h: f32,
    pub(super) bg_color_panel_margin: f32,
}

/// 构建或更新 build layout 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn build_layout(
    tab: &MindMapTab,
    inputs: &LayoutInputs,
    node_toolbar_rect: Option<Rectangle>,
    active_picker_target: Option<MindMapColorTarget>,
    color_picker_anchor: impl Fn(MindMapColorTarget) -> Point,
    node_action_btn_bottom_left_anchor: impl Fn(usize, f32) -> Option<Point>,
    node_action_btn_top_anchor: impl Fn(usize) -> Option<Point>,
) -> RenderLayout {
    let default_side_anchor = Point::new(
        inputs.action_menu_x + inputs.action_menu_w + 8.0,
        inputs.action_menu_y + inputs.action_bar_h,
    );
    let priority_picker_anchor = node_action_btn_top_anchor(0).unwrap_or(default_side_anchor);
    let url_editor_anchor = node_action_btn_top_anchor(5).unwrap_or(default_side_anchor);

    let zoom_menu_h = inputs.zoom_menu_padding * 2.0
        + inputs.zoom_menu_item_h * inputs.zoom_menu_item_count as f32
        + inputs.zoom_menu_spacing * (inputs.zoom_menu_item_count.saturating_sub(1)) as f32;
    let zoom_menu_anchor = Point::new(100_000.0, inputs.zoom_panel_margin + inputs.zoom_control_h);

    let mut ui_blocked_rects = vec![Rectangle::new(
        Point::new(inputs.action_menu_x, inputs.action_menu_y),
        Size::new(inputs.action_bar_w, inputs.action_bar_h),
    )];

    if let Some(rect) = node_toolbar_rect {
        ui_blocked_rects.push(rect);
    }

    if tab.show_action_menu {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(
                inputs.action_menu_x,
                inputs.action_menu_y + inputs.action_bar_h + inputs.action_menu_gap,
            ),
            Size::new(inputs.action_menu_w, inputs.action_menu_h),
        ));
    }

    if tab.show_priority_picker {
        push_picker_rect(
            &mut ui_blocked_rects,
            node_toolbar_rect,
            inputs.picker_top_gap,
            priority_picker_anchor,
            node_action_btn_bottom_left_anchor(0, 140.0),
            140.0,
            150.0,
        );
    } else if tab.show_url_editor {
        push_picker_rect(
            &mut ui_blocked_rects,
            node_toolbar_rect,
            inputs.picker_top_gap,
            url_editor_anchor,
            node_action_btn_bottom_left_anchor(5, 350.0),
            350.0,
            84.0,
        );
    } else if tab.show_text_editor {
        ui_blocked_rects.push(fullscreen_block_rect());
    } else if let Some(target) = active_picker_target {
        let height = match target {
            MindMapColorTarget::EdgeStroke | MindMapColorTarget::NodeBorder => 400.0,
            _ => 360.0,
        };
        let anchor = color_picker_anchor(target);
        ui_blocked_rects.push(Rectangle::new(
            Point::new(anchor.x - inputs.picker_left_gap - 220.0, anchor.y - height / 2.0),
            Size::new(220.0, height),
        ));
    }

    if let Some(anchor) = tab.context_menu_anchor {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(anchor.x, anchor.y + 6.0),
            Size::new(260.0, 44.0),
        ));
    }

    ui_blocked_rects.push(Rectangle::new(
        Point::new(100_000.0, inputs.zoom_panel_margin),
        Size::new(inputs.zoom_control_w, inputs.zoom_control_h),
    ));

    if tab.show_zoom_menu {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(100_000.0, zoom_menu_anchor.y + inputs.zoom_menu_gap),
            Size::new(inputs.zoom_control_w, zoom_menu_h),
        ));
    }

    if tab.show_markdown_import {
        ui_blocked_rects.push(fullscreen_block_rect());
    }

    ui_blocked_rects.push(Rectangle::new(
        Point::new(inputs.bg_color_panel_margin, 100_000.0),
        Size::new(inputs.bg_color_panel_w, inputs.bg_color_panel_h),
    ));

    if tab.show_theme_panel {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(100_000.0, 100_000.0),
            Size::new(
                inputs.bg_panel_w + inputs.bg_panel_margin,
                inputs.bg_panel_h + inputs.bg_panel_margin,
            ),
        ));
    }

    if tab.show_diagram_type_picker {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(100_000.0, 100_000.0),
            Size::new(
                inputs.diagram_panel_w + inputs.diagram_panel_margin,
                inputs.diagram_panel_h + inputs.diagram_panel_margin,
            ),
        ));
    }

    RenderLayout {
        default_side_anchor,
        priority_picker_anchor,
        url_editor_anchor,
        zoom_menu_anchor,
        ui_blocked_rects,
    }
}

fn push_picker_rect(
    ui_blocked_rects: &mut Vec<Rectangle>,
    node_toolbar_rect: Option<Rectangle>,
    picker_top_gap: f32,
    top_anchor: Point,
    bottom_left_anchor: Option<Point>,
    width: f32,
    height: f32,
) {
    if node_toolbar_rect.is_some_and(|rect| rect.y >= picker_top_gap + height) {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(top_anchor.x - width / 2.0, top_anchor.y - picker_top_gap - height),
            Size::new(width, height),
        ));
    } else if let Some(anchor) = bottom_left_anchor {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(anchor.x, anchor.y + picker_top_gap),
            Size::new(width, height),
        ));
    } else {
        ui_blocked_rects.push(Rectangle::new(
            Point::new(top_anchor.x - width / 2.0, top_anchor.y - picker_top_gap - height),
            Size::new(width, height),
        ));
    }
}

fn fullscreen_block_rect() -> Rectangle {
    Rectangle::new(Point::new(0.0, 0.0), Size::new(100_000.0, 100_000.0))
}
