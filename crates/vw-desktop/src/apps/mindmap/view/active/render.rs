//! 思维导图活跃视图渲染入口。
//!
//! 本文件保留状态派生与总装配逻辑，具体几何计算、遮挡区域计算、
//! 文本编辑器覆盖层和 Markdown 导入弹层分别拆到私有子模块中。

use crate::app::Message;
use crate::app::components::overlays::{PointAboveOverlay, PointBelowOverlay, PointLeftOverlay};
use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::canvas::MindMapCanvas;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapColorTarget, MindMapTab};
use iced::widget::{container, stack};
use iced::{Color, Element, Length, Point};

mod bars;
mod context_menu;
mod geometry;
mod layout;
mod markdown_import;
mod menus;
mod panels;
mod pickers;
mod text_editor;

#[cfg(test)]
mod bars_tests;
#[cfg(test)]
mod context_menu_tests;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod markdown_import_tests;
#[cfg(test)]
mod menus_tests;
#[cfg(test)]
mod pickers_tests;
#[cfg(test)]
mod text_editor_tests;

use super::super::common::base_style;
use geometry::{
    node_action_btn_anchor, node_action_btn_bottom_left_anchor, node_action_btn_top_anchor,
    node_toolbar_layout,
};
use layout::{LayoutInputs, build_layout};

/// 渲染思维导图的完整活跃编辑界面
///
/// 该函数构建思维导图编辑器的完整UI层次结构，包括多个层级和覆盖层。
/// 它是活跃视图的核心渲染函数，负责协调所有UI组件的布局和渲染。
///
/// # 参数
///
/// * `tab` - 思维导图标签页的状态，包含：
///   - 文档数据和节点位置
///   - 选择状态和编辑状态
///   - UI状态（各种面板的显示/隐藏状态）
///   - 样式设置（颜色、边框样式等）
///   - 缩放和平移状态
///
/// # 返回值
///
/// 返回一个 Iced `Element`，包含完整的活跃视图UI组件树
///
/// # UI层次结构
///
/// 渲染的UI包含以下层次（从底到顶）：
/// 1. 画布层 - 渲染思维导图的节点和连线
/// 2. 背景色面板 - 位于左下角
/// 3. 主题面板 - 位于右下角（当显示时）
/// 4. 图表类型面板 - 位于右下角（当显示时）
/// 5. 工具工具栏 - 位于顶部中央
/// 6. 画笔面板 - 位于工具工具栏下方
/// 7. 操作栏 - 位于左上角
/// 8. 缩放控制 - 位于右上角
/// 9. 各种覆盖层（菜单、选择器、编辑器等）
///
/// # 示例
///
/// ```ignore
/// let tab = MindMapTab::load_from_file("mindmap.json")?;
/// let ui = render(&tab);
/// // 将 ui 添加到 Iced 应用程序中
/// ```
pub(super) fn render(tab: &MindMapTab) -> Element<'_, Message> {
    let (active_picker_target, active_picker_color, active_picker_format, active_picker_picking) =
        tab.active_color_picker
            .as_ref()
            .map(|p| (Some(p.target), p.color, p.format, p.picking))
            .unwrap_or((None, Color::BLACK, ColorFormat::Hex, false));
    let active_edge_style = tab
        .selected_path
        .as_ref()
        .and_then(|p| tab.edge_styles.get(p))
        .copied()
        .unwrap_or(tab.edge_style);

    // 获取当前选中节点的节点边框样式（如果有的话），否则使用默认节点边框样式
    let active_node_border_style = tab
        .selected_path
        .as_ref()
        .and_then(|p| tab.node_border_styles.get(p))
        .copied()
        .unwrap_or(tab.node_border_style);
    let selected_path_is_root = tab.selected_path.as_deref().map(|p| p.is_empty()).unwrap_or(true);
    let current_priority =
        tab.selected_path.as_ref().and_then(|p| tab.node_priorities.get(p)).copied();
    let current_url_present =
        tab.selected_path.as_ref().and_then(|p| tab.node_urls.get(p)).is_some();
    let can_undo = !tab.undo_stack.is_empty();
    let can_redo = !tab.redo_stack.is_empty();
    let can_cut = tab.selected_path.as_deref().is_some_and(|p| !p.is_empty());
    let can_copy = tab.selected_path.is_some();
    let can_paste = tab.selected_path.is_some() && tab.clipboard_node.is_some();
    let can_delete = tab.selected_path.as_deref().is_some_and(|p| !p.is_empty());
    let can_style = tab.selected_path.is_some();

    let action_menu_x = 14.0;
    let action_menu_y = 14.0;
    let action_btn_size = 30.0;
    let action_bar_padding = 6.0;
    let action_bar_spacing = 6.0;
    let action_bar_btn_count = 10.0;
    let action_bar_w = action_bar_padding * 2.0
        + action_btn_size * action_bar_btn_count
        + action_bar_spacing * (action_bar_btn_count - 1.0);
    let action_bar_h = action_bar_padding * 2.0 + action_btn_size;
    let action_menu_gap = 6.0;
    let action_menu_w = 260.0;
    let action_menu_h = 156.0;

    let node_toolbar_btn_w = 30.0;
    let node_toolbar_btn_spacing = 6.0;
    let node_toolbar_padding = 6.0;
    let node_toolbar_group_gap = 10.0;
    let node_toolbar_divider_w = 1.0;
    let node_toolbar_btn_h = 24.0;
    let node_toolbar_divider_h = node_toolbar_btn_h;

    let edit_actions_w = node_toolbar_btn_w * 4.0 + node_toolbar_btn_spacing * 3.0;
    let edge_btn_w = if selected_path_is_root { 0.0 } else { node_toolbar_btn_w };
    let node_btn_widths = [
        node_toolbar_btn_w,
        node_toolbar_btn_w,
        node_toolbar_btn_w,
        node_toolbar_btn_w,
        edge_btn_w,
        node_toolbar_btn_w,
        node_toolbar_btn_w,
    ];
    let node_actions_w = node_btn_widths.iter().copied().sum::<f32>()
        + node_toolbar_btn_spacing * (node_btn_widths.len() as f32 - 1.0);
    let node_toolbar_w = node_toolbar_padding * 2.0
        + edit_actions_w
        + node_toolbar_group_gap
        + node_toolbar_divider_w
        + node_toolbar_group_gap
        + node_actions_w;
    let node_toolbar_h = node_toolbar_padding * 2.0 + node_toolbar_btn_h;
    let node_toolbar_gap = 10.0;

    let node_toolbar_layout = node_toolbar_layout(
        tab,
        action_menu_y,
        action_bar_h,
        action_menu_gap,
        node_toolbar_w,
        node_toolbar_h,
        node_toolbar_gap,
    );
    let node_toolbar_rect = node_toolbar_layout.as_ref().map(|layout| layout.rect);

    let node_action_btn_anchor = |btn_index| {
        node_toolbar_rect.and_then(|toolbar_rect| {
            node_action_btn_anchor(
                toolbar_rect,
                &node_btn_widths,
                node_toolbar_padding,
                edit_actions_w,
                node_toolbar_group_gap,
                node_toolbar_divider_w,
                node_toolbar_btn_spacing,
                btn_index,
            )
        })
    };
    let node_action_btn_top_anchor = |btn_index| {
        node_toolbar_rect.and_then(|toolbar_rect| {
            node_action_btn_top_anchor(
                toolbar_rect,
                &node_btn_widths,
                node_toolbar_padding,
                edit_actions_w,
                node_toolbar_group_gap,
                node_toolbar_divider_w,
                node_toolbar_btn_spacing,
                btn_index,
            )
        })
    };
    let node_action_btn_bottom_left_anchor = |btn_index, overlay_w| {
        node_toolbar_rect.and_then(|toolbar_rect| {
            node_action_btn_bottom_left_anchor(
                toolbar_rect,
                &node_btn_widths,
                node_toolbar_padding,
                edit_actions_w,
                node_toolbar_group_gap,
                node_toolbar_divider_w,
                node_toolbar_btn_spacing,
                btn_index,
                overlay_w,
            )
        })
    };

    let picker_left_gap = 18.0;
    let picker_top_gap = 10.0;

    let color_picker_anchor = |target: MindMapColorTarget, default_side_anchor: Point| {
        match target {
            MindMapColorTarget::NodeText => node_action_btn_anchor(1),
            MindMapColorTarget::NodeFill => node_action_btn_anchor(2),
            MindMapColorTarget::NodeBorder => node_action_btn_anchor(3),
            MindMapColorTarget::EdgeStroke => node_action_btn_anchor(4),
            _ => None,
        }
        .unwrap_or(default_side_anchor)
    };

    let zoom_preset_percents = [10u32, 20, 40, 70, 80, 100, 125, 150, 175, 200, 250, 300, 400];
    let zoom_panel_margin = 14.0;
    let zoom_control_w = 156.0;
    let zoom_control_h = 32.0;
    let zoom_menu_gap = 8.0;
    let zoom_menu_padding = 6.0;
    let zoom_menu_item_h = 28.0;
    let zoom_menu_spacing = 2.0;
    let zoom_menu_item_count = 1usize + zoom_preset_percents.len();

    let bg_panel_w = 360.0;
    let bg_panel_h = 250.0;
    let bg_panel_margin = 14.0;
    let diagram_panel_w = 620.0;
    let diagram_panel_h = 260.0;
    let diagram_panel_margin = 14.0;
    let bg_color_panel_w = 280.0;
    let bg_color_panel_h = 32.0;
    let bg_color_panel_margin = 14.0;

    let computed_layout = build_layout(
        tab,
        &LayoutInputs {
            action_menu_x,
            action_menu_y,
            action_bar_w,
            action_bar_h,
            action_menu_gap,
            action_menu_w,
            action_menu_h,
            picker_left_gap,
            picker_top_gap,
            zoom_panel_margin,
            zoom_control_w,
            zoom_control_h,
            zoom_menu_gap,
            zoom_menu_padding,
            zoom_menu_item_h,
            zoom_menu_spacing,
            zoom_menu_item_count,
            bg_panel_w,
            bg_panel_h,
            bg_panel_margin,
            diagram_panel_w,
            diagram_panel_h,
            diagram_panel_margin,
            bg_color_panel_w,
            bg_color_panel_h,
            bg_color_panel_margin,
        },
        node_toolbar_rect,
        active_picker_target,
        |target| {
            color_picker_anchor(
                target,
                Point::new(action_menu_x + action_menu_w + 8.0, action_menu_y + action_bar_h),
            )
        },
        node_action_btn_bottom_left_anchor,
        node_action_btn_top_anchor,
    );

    let canvas = MindMapCanvas {
        doc: &tab.doc,
        cache: &tab.canvas_cache,
        pan: tab.pan,
        zoom: tab.zoom,
        selected_path: tab.selected_path.as_deref(),
        node_positions: &tab.node_positions,
        diagram_type: tab.diagram_type,
        layout_format: tab.layout_format,
        org_chart_layout_format: tab.org_chart_layout_format,
        fishbone_layout_format: tab.fishbone_layout_format,
        timeline_layout_format: tab.timeline_layout_format,
        bracket_layout_format: tab.bracket_layout_format,
        tree_layout_format: tab.tree_layout_format,
        node_fills: &tab.node_fills,
        node_text_colors: &tab.node_text_colors,
        node_border_colors: &tab.node_border_colors,
        node_border_style: tab.node_border_style,
        node_border_styles: &tab.node_border_styles,
        node_priorities: &tab.node_priorities,
        node_urls: &tab.node_urls,
        collapsed_paths: &tab.collapsed_paths,
        background: tab.background,
        follow_theme_background: tab.follow_theme_background,
        edge_style: tab.edge_style,
        edge_styles: &tab.edge_styles,
        edge_colors: &tab.edge_colors,
        canvas_tool: tab.canvas_tool,
        doodle_rgba: tab.doodle_rgba,
        doodle_width_px: tab.doodle_width_px,
        doodles: &tab.doodles,
        ui_blocked_rects: computed_layout.ui_blocked_rects,
        theme_group: &tab.theme_group,
        theme_variant: tab.theme_variant,
        custom_themes: &tab.custom_themes,
        theme_panel_open: tab.show_theme_panel,
    };
    let canvas_el: Element<'_, Message> =
        container(iced::widget::canvas(canvas).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(base_style)
            .into();

    let tool_toolbar_w = 168.0;
    let tool_toolbar_h = 40.0;
    let tool_toolbar_margin = 14.0;
    let pen_panel_w = 460.0;
    let pen_panel_h = 40.0;
    let pen_panel_gap = 8.0;

    let bg_color_panel = panels::background_panel(tab, bg_color_panel_w, bg_color_panel_h);
    let tool_toolbar = panels::tool_toolbar(tab, tool_toolbar_w, tool_toolbar_h);
    let pen_panel = panels::pen_panel(tab, pen_panel_w, pen_panel_h);
    let action_bar = bars::action_bar(
        tab,
        action_btn_size,
        action_bar_padding,
        action_bar_spacing,
        can_undo,
        can_redo,
        can_cut,
        can_copy,
        can_paste,
        can_delete,
    );
    let action_menu_overlay = menus::action_menu_overlay(tab, action_menu_w);
    let node_toolbar_overlay = bars::node_toolbar_overlay(
        tab,
        node_toolbar_btn_w,
        node_toolbar_btn_h,
        node_toolbar_padding,
        node_toolbar_divider_h,
        selected_path_is_root,
        can_cut,
        can_copy,
        can_paste,
        can_delete,
        can_style,
        current_priority,
        current_url_present,
    );
    let zoom_control = menus::zoom_control(tab, zoom_control_w, zoom_control_h);
    let zoom_menu_overlay = menus::zoom_menu_overlay(
        tab,
        zoom_control_w,
        zoom_menu_item_h,
        zoom_menu_spacing,
        zoom_menu_padding,
        &zoom_preset_percents,
    );
    let mut layers: Vec<Element<'_, Message>> = vec![canvas_el];
    layers.push(
        container(bg_color_panel)
            .padding(bg_color_panel_margin)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Bottom)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    );
    if tab.show_theme_panel {
        layers.push(
            container(panels::theme_panel(tab, bg_panel_w, bg_panel_h))
                .padding(bg_panel_margin)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        );
    }
    if tab.show_diagram_type_picker {
        layers.push(
            container(panels::diagram_type_panel(tab, diagram_panel_w, diagram_panel_h))
                .padding(diagram_panel_margin)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        );
    }
    layers.push(
        container(tool_toolbar)
            .padding(iced::Padding { top: tool_toolbar_margin, right: 0.0, bottom: 0.0, left: 0.0 })
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    );
    layers.push(
        container(pen_panel)
            .padding(iced::Padding {
                top: tool_toolbar_margin + tool_toolbar_h + pen_panel_gap,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    );
    layers.push(
        container(action_bar)
            .padding(iced::Padding {
                top: action_menu_y,
                right: 0.0,
                bottom: 0.0,
                left: action_menu_x,
            })
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    );
    layers.push(
        container(zoom_control)
            .padding(iced::Padding {
                top: zoom_panel_margin,
                right: zoom_panel_margin,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Top)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    );
    let canvas_with_ui: Element<'_, Message> = stack(layers).into();
    let action_anchor = Point::new(action_menu_x, action_menu_y + action_bar_h);
    let mut canvas_with_overlays: Element<'_, Message> = if tab.show_action_menu {
        PointBelowOverlay::new(canvas_with_ui, action_menu_overlay)
            .show(true)
            .anchor(action_anchor)
            .gap(action_menu_gap)
            .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
            .into()
    } else {
        canvas_with_ui
    };
    if tab.show_zoom_menu {
        canvas_with_overlays = PointBelowOverlay::new(canvas_with_overlays, zoom_menu_overlay)
            .show(true)
            .anchor(computed_layout.zoom_menu_anchor)
            .gap(zoom_menu_gap)
            .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
            .into();
    }
    if can_style && let Some(node_toolbar_layout) = node_toolbar_layout {
        canvas_with_overlays = if node_toolbar_layout.place_above {
            PointAboveOverlay::new(canvas_with_overlays, node_toolbar_overlay)
                .show(true)
                .anchor(node_toolbar_layout.anchor)
                .gap(node_toolbar_gap)
                .into()
        } else {
            PointBelowOverlay::new(canvas_with_overlays, node_toolbar_overlay)
                .show(true)
                .anchor(node_toolbar_layout.anchor)
                .gap(node_toolbar_gap)
                .into()
        };
    }
    if tab.show_priority_picker {
        canvas_with_overlays = if node_toolbar_rect.is_some_and(|r| r.y >= picker_top_gap + 150.0) {
            PointAboveOverlay::new(
                canvas_with_overlays,
                pickers::priority_picker_overlay(current_priority),
            )
            .show(true)
            .anchor(computed_layout.priority_picker_anchor)
            .gap(picker_top_gap)
            .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
            .into()
        } else {
            let anchor = node_action_btn_bottom_left_anchor(0, 140.0)
                .unwrap_or(computed_layout.default_side_anchor);
            PointBelowOverlay::new(
                canvas_with_overlays,
                pickers::priority_picker_overlay(current_priority),
            )
            .show(true)
            .anchor(anchor)
            .gap(picker_top_gap)
            .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
            .into()
        };
    } else if tab.show_url_editor {
        canvas_with_overlays = if node_toolbar_rect.is_some_and(|r| r.y >= picker_top_gap + 84.0) {
            PointAboveOverlay::new(canvas_with_overlays, pickers::url_editor_overlay(tab))
                .show(true)
                .anchor(computed_layout.url_editor_anchor)
                .gap(picker_top_gap)
                .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
                .into()
        } else {
            let anchor = node_action_btn_bottom_left_anchor(5, 350.0)
                .unwrap_or(computed_layout.default_side_anchor);
            PointBelowOverlay::new(canvas_with_overlays, pickers::url_editor_overlay(tab))
                .show(true)
                .anchor(anchor)
                .gap(picker_top_gap)
                .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
                .into()
        };
    } else if let Some(target) = active_picker_target {
        if let Some(title) = color_picker_title(target) {
            canvas_with_overlays = PointLeftOverlay::new(
                canvas_with_overlays,
                pickers::color_picker_overlay(
                    title,
                    target,
                    active_edge_style,
                    active_node_border_style,
                    active_picker_color,
                    active_picker_format,
                    active_picker_picking,
                ),
            )
            .show(true)
            .anchor(color_picker_anchor(target, computed_layout.default_side_anchor))
            .gap(picker_left_gap)
            .on_close(Message::MindMapTool(MindMapMessage::ClosePickers))
            .into();
        }
    }
    if tab.show_context_menu {
        let menu_anchor =
            tab.context_menu_anchor.unwrap_or(Point::new(action_menu_x, action_menu_y));
        canvas_with_overlays = PointBelowOverlay::new(
            canvas_with_overlays,
            context_menu::context_menu_overlay(can_cut, can_copy, can_paste, can_delete),
        )
        .show(true)
        .anchor(menu_anchor)
        .gap(6.0)
        .on_close(Message::MindMapTool(MindMapMessage::CloseContextMenu))
        .into();
    }
    if tab.show_text_editor {
        canvas_with_overlays = text_editor::with_text_editor_overlay(tab, canvas_with_overlays);
    }
    if tab.show_markdown_import {
        markdown_import::with_markdown_import_overlay(tab, canvas_with_overlays)
    } else {
        canvas_with_overlays
    }
}

pub(super) fn color_picker_title(target: MindMapColorTarget) -> Option<&'static str> {
    match target {
        MindMapColorTarget::NodeText => Some("文字颜色"),
        MindMapColorTarget::NodeFill => Some("节点填充"),
        MindMapColorTarget::NodeBorder => Some("边框颜色"),
        MindMapColorTarget::EdgeStroke => Some("连线颜色"),
        _ => None,
    }
}
