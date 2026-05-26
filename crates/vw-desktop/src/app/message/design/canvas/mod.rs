//! # 画布消息处理模块
//!
//! 本模块负责处理设计视图中画布相关的所有消息和交互操作。
//!
//! ## 主要功能
//!
//! - 视图变换：处理画布的平移与缩放操作
//! - 工具管理：处理设计工具的选择和切换
//! - 视图适配：提供画布自适应、缩放到指定元素等布局功能
//! - 元素层级管理：处理元素的重新父级化操作

mod brush;
mod element_tree;
mod handlers;

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::creation::DEFAULT_BRUSH_COLOR_HEX;
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::{App, Message};
use iced::Task;

/// 处理画布相关的消息更新。
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    let window_size = app.window_size;

    if let Some(state) = app.active_design_state_mut() {
        match message {
            DesignMessage::Pan(new_pan) => {
                state.pan = new_pan;
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::Zoom(factor, center_opt) => {
                let old_zoom = state.zoom;
                let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);

                if let Some(point) = center_opt {
                    let screen_point = iced::Vector::new(point.x, point.y);
                    state.pan = screen_point - (screen_point - state.pan) * (new_zoom / old_zoom);
                }

                state.zoom = new_zoom;
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::ToolSelected(tool) => handlers::handle_tool_selected(state, tool),
            DesignMessage::SetBrushColor(color) => {
                state.brush_color_hex = brush::extract_hex_token(&color)
                    .unwrap_or_else(|| DEFAULT_BRUSH_COLOR_HEX.to_string());
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::SetBrushWidth(width) => {
                state.brush_width_px = width.clamp(1.0, 18.0);
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::EraseBrushAt(center_world, radius_world) => {
                if radius_world <= 0.0 {
                    return Task::none();
                }

                let changed = brush::erase_brush_nodes(
                    &mut state.doc.children,
                    iced::Point::new(0.0, 0.0),
                    center_world,
                    radius_world,
                );

                if changed {
                    state.selected_element_ids.retain(|id| state.doc.find_element(id).is_some());
                    state.selected_element_id = state
                        .selected_element_id
                        .clone()
                        .filter(|id| state.doc.find_element(id).is_some());
                    state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                    state.canvas_cache.clear();
                }

                Task::none()
            }
            DesignMessage::CreateElement {
                element,
                parent_id,
                start_editing,
            } => handlers::handle_create_element(state, element, parent_id, start_editing),
            DesignMessage::ZoomIn => {
                state.zoom = (state.zoom * 1.2).clamp(0.1, 10.0);
                state.show_zoom_menu = false;
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::ZoomOut => {
                state.zoom = (state.zoom / 1.2).clamp(0.1, 10.0);
                state.show_zoom_menu = false;
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::ZoomFit => handlers::handle_zoom_fit(state, window_size),
            DesignMessage::ZoomSet(value) => {
                state.zoom = value.clamp(0.1, 10.0);
                state.show_zoom_menu = false;
                state.canvas_cache.clear();
                Task::none()
            }
            DesignMessage::ZoomPresetSelected(label) => match label.as_str() {
                "Fit" => update(app, DesignMessage::ZoomFit),
                "20%" => update(app, DesignMessage::ZoomSet(0.2)),
                "30%" => update(app, DesignMessage::ZoomSet(0.3)),
                "50%" => update(app, DesignMessage::ZoomSet(0.5)),
                "80%" => update(app, DesignMessage::ZoomSet(0.8)),
                "100%" => update(app, DesignMessage::ZoomSet(1.0)),
                "200%" => update(app, DesignMessage::ZoomSet(2.0)),
                "300%" => update(app, DesignMessage::ZoomSet(3.0)),
                _ => {
                    if let Some(number) = label.strip_suffix('%').and_then(|raw| raw.parse::<f32>().ok()) {
                        update(app, DesignMessage::ZoomSet(number / 100.0))
                    } else {
                        Task::none()
                    }
                }
            },
            DesignMessage::FitToElement(id) => handlers::handle_fit_to_element(state, &id, window_size),
            DesignMessage::ToggleZoomMenu => {
                state.show_zoom_menu = !state.show_zoom_menu;
                Task::none()
            }
            DesignMessage::CanvasContextMenuOpen(anchor, hit_id) => {
                handlers::handle_canvas_context_menu_open(state, anchor, hit_id)
            }
            DesignMessage::CanvasContextMenuClose => handlers::handle_canvas_context_menu_close(state),
            DesignMessage::CanvasContextMenuAction(action) => {
                handlers::handle_canvas_context_menu_action(state, action)
            }
            DesignMessage::ToggleContextPopover(popover) => {
                if state.context_popover == popover {
                    state.context_popover = None;
                } else {
                    state.context_popover = popover;
                }
                state.context_shape_group_hover = None;
                Task::none()
            }
            DesignMessage::ContextShapeGroupHover(group) => {
                state.context_shape_group_hover = group;
                Task::none()
            }
            DesignMessage::SetIconFilter(query) => {
                state.icon_filter_query = query;
                Task::none()
            }
            DesignMessage::SetToolbarIconFamilyTab(family) => {
                state.toolbar_icon_family_tab = family;
                Task::none()
            }
            DesignMessage::SelectToolbarIcon { family, name } => {
                handlers::handle_select_toolbar_icon(state, family, name)
            }
            DesignMessage::UpdateContextShape(kind) => handlers::handle_update_context_shape(state, kind),
            DesignMessage::UpdateContextFill(fill_type) => {
                handlers::handle_update_context_fill(state, fill_type)
            }
            DesignMessage::UpdateContextBorder(border_type) => {
                handlers::handle_update_context_border(state, border_type)
            }
            DesignMessage::ReparentElements(ids, parent_opt) => {
                element_tree::reparent_elements(state, ids, parent_opt)
            }
            _ => Task::none(),
        }
    } else {
        Task::none()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "element_tree_tests.rs"]
mod element_tree_tests;

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod handlers_tests;
