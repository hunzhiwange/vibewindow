//! 集中处理设计画布交互消息，将指针、键盘和工具操作落到画布状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::brush::{extract_hex_token, with_alpha};
use super::element_tree::insert_into_parent;
use crate::app::message::design::{CanvasContextMenuAction, LayerAction};
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::geometry::get_element_screen_bounds;
use crate::app::views::design::models::{DesignElement, DesignTool, Stroke, compute_tree_metrics};
use crate::app::views::design::state::{ContextPopoverType, DesignState};
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
use crate::app::Message;
use iced::{Task, Vector, widget::text_editor};

/// handle_tool_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn handle_tool_selected(state: &mut DesignState, tool: DesignTool) -> Task<Message> {
    state.active_tool = tool;
    state.canvas_cache.clear();
    state.context_popover = match tool {
        DesignTool::Pen => Some(ContextPopoverType::ToolbarBrush),
        _ if matches!(
            state.context_popover,
            Some(ContextPopoverType::ToolbarBrush)
                | Some(ContextPopoverType::ToolbarShape)
                | Some(ContextPopoverType::ToolbarIcon)
        ) => None,
        _ => state.context_popover,
    };
    Task::none()
}

/// handle_create_element 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_create_element(
    state: &mut DesignState,
    mut element: DesignElement,
    parent_id: Option<String>,
    start_editing: bool,
) -> Task<Message> {
    let selected_id = element.id.clone();

    if let Some(parent_id) = parent_id {
        let group_id = state
            .doc
            .group_id_for_element(&parent_id)
            .unwrap_or(state.active_group_id);
        element.set_group_id_recursive(group_id);
        if let Err(element) = insert_into_parent(&mut state.doc.children, &parent_id, element) {
            state.doc.children.push(element);
        }
    } else {
        element.set_group_id_recursive(state.active_group_id);
        state.doc.children.push(element);
    }

    state.layer_tree_metrics = compute_tree_metrics(&state.doc);
    state.selected_element_id = Some(selected_id.clone());
    state.selected_element_ids.clear();
    state.selected_element_ids.insert(selected_id.clone());
    state.selected_fill_index = None;
    state.selected_effect_index = None;

    if start_editing {
        let content = state
            .doc
            .find_element(&selected_id)
            .and_then(|element| element.content.clone())
            .unwrap_or_default();
        state.editing_id = Some(selected_id);
        state.editing_content = content.clone();
        state.editing_editor = text_editor::Content::with_text(&content);
        if state.active_tool == DesignTool::Text {
            state.active_tool = DesignTool::Move;
        }
    } else {
        state.editing_id = None;
        state.editing_content.clear();
        state.editing_editor = text_editor::Content::new();
        if matches!(
            state.active_tool,
            DesignTool::Text
                | DesignTool::Icon
                | DesignTool::StickyNote
                | DesignTool::Rectangle
                | DesignTool::Ellipse
                | DesignTool::Frame
                | DesignTool::Line
                | DesignTool::Triangle
                | DesignTool::Diamond
                | DesignTool::Star
                | DesignTool::Pentagon
                | DesignTool::Hexagon
                | DesignTool::Parallelogram
                | DesignTool::Trapezoid
                | DesignTool::Chevron
                | DesignTool::Capsule
        ) {
            state.active_tool = DesignTool::Move;
        }
    }

    state.canvas_cache.clear();
    Task::none()
}

/// handle_zoom_fit 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_zoom_fit(
    state: &mut DesignState,
    window_size: (f32, f32),
) -> Task<Message> {
    if let Some((min_x, min_y, max_x, max_y)) = state.doc.get_bounds() {
        let width = max_x - min_x;
        let height = max_y - min_y;

        if width > 0.0 && height > 0.0 {
            let available_w = (window_size.0 - 500.0).max(100.0);
            let available_h = (window_size.1 - 100.0).max(100.0);
            let scale_x = available_w / width;
            let scale_y = available_h / height;
            let new_zoom = (scale_x.min(scale_y) * 0.9).clamp(0.1, 10.0);
            let content_cx = min_x + width / 2.0;
            let content_cy = min_y + height / 2.0;
            let view_cx = window_size.0 / 2.0;
            let view_cy = window_size.1 / 2.0;

            state.zoom = new_zoom;
            state.pan = Vector::new(view_cx - content_cx * new_zoom, view_cy - content_cy * new_zoom);
            state.show_zoom_menu = false;
            state.canvas_cache.clear();
        }
    }
    Task::none()
}

/// handle_fit_to_element 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_fit_to_element(
    state: &mut DesignState,
    id: &str,
    window_size: (f32, f32),
) -> Task<Message> {
    if let Some(bounds) = get_element_screen_bounds(&state.doc, id, Vector::new(0.0, 0.0), 1.0) {
        let padding = 50.0;
        let available_w = window_size.0 - padding * 2.0;
        let available_h = window_size.1 - padding * 2.0;
        let new_zoom = (available_w / bounds.width)
            .min(available_h / bounds.height)
            .min(2.0)
            .max(0.1);
        let center_viewport_x = window_size.0 / 2.0;
        let center_viewport_y = window_size.1 / 2.0;
        let center_el_x = bounds.x + bounds.width / 2.0;
        let center_el_y = bounds.y + bounds.height / 2.0;

        state.zoom = new_zoom;
        state.pan = Vector::new(
            center_viewport_x - center_el_x * new_zoom,
            center_viewport_y - center_el_y * new_zoom,
        );
        state.canvas_cache.clear();
    }
    Task::none()
}

/// handle_canvas_context_menu_open 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_canvas_context_menu_open(
    state: &mut DesignState,
    anchor: iced::Point,
    hit_id: Option<String>,
) -> Task<Message> {
    state.context_popover = None;
    state.show_zoom_menu = false;
    state.canvas_context_menu_anchor = Some(anchor);

    if let Some(id) = hit_id {
        if state.selected_element_id.as_deref() != Some(&id)
            || !state.selected_element_ids.contains(&id)
        {
            state.selected_element_id = Some(id.clone());
            state.selected_element_ids.clear();
            state.selected_element_ids.insert(id);
            state.canvas_cache.clear();
        }
    } else {
        state.selected_element_id = None;
        state.selected_element_ids.clear();
        state.canvas_cache.clear();
    }

    Task::none()
}

/// handle_canvas_context_menu_close 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn handle_canvas_context_menu_close(state: &mut DesignState) -> Task<Message> {
    state.canvas_context_menu_anchor = None;
    state.paste_anchor = None;
    Task::none()
}

/// handle_canvas_context_menu_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_canvas_context_menu_action(
    state: &mut DesignState,
    action: CanvasContextMenuAction,
) -> Task<Message> {
    let paste_anchor = state.canvas_context_menu_anchor;
    state.canvas_context_menu_anchor = None;
    state.paste_anchor = None;

    match action {
        CanvasContextMenuAction::Cut => Task::done(Message::Design(DesignMessage::Cut)),
        CanvasContextMenuAction::Copy => Task::done(Message::Design(DesignMessage::Copy)),
        CanvasContextMenuAction::Paste => {
            state.paste_anchor = paste_anchor;
            Task::done(Message::Design(DesignMessage::Paste))
        }
        CanvasContextMenuAction::Delete => state
            .selected_element_id
            .clone()
            .map(|id| {
                Task::done(Message::Design(DesignMessage::LayerActionSelected(
                    id,
                    LayerAction::Delete,
                )))
            })
            .unwrap_or_else(Task::none),
        CanvasContextMenuAction::MoveUp => state
            .selected_element_id
            .clone()
            .map(|id| Task::done(Message::Design(DesignMessage::MoveLayerItem(id, -1))))
            .unwrap_or_else(Task::none),
        CanvasContextMenuAction::MoveDown => state
            .selected_element_id
            .clone()
            .map(|id| Task::done(Message::Design(DesignMessage::MoveLayerItem(id, 1))))
            .unwrap_or_else(Task::none),
    }
}

/// handle_select_toolbar_icon 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_select_toolbar_icon(
    state: &mut DesignState,
    family: String,
    name: String,
) -> Task<Message> {
    state.toolbar_icon_family = family.clone();
    state.toolbar_icon_name = name;
    state.toolbar_icon_family_tab = family;
    state.icon_filter_query.clear();
    state.active_tool = DesignTool::Icon;
    state.context_popover = None;
    Task::none()
}

/// handle_update_context_shape 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_update_context_shape(
    state: &mut DesignState,
    kind: String,
) -> Task<Message> {
    state.context_popover = None;
    state.context_shape_group_hover = None;
    if let Some(id) = state.selected_element_id.clone() {
        return Task::done(Message::Design(DesignMessage::PropertyUpdate(
            id,
            "kind".to_string(),
            serde_json::json!(kind),
        )));
    }
    Task::none()
}

/// handle_update_context_fill 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_update_context_fill(
    state: &mut DesignState,
    fill_type: String,
) -> Task<Message> {
    if let Some(id) = state.selected_element_id.clone() {
        let fills = match fill_type.as_str() {
            "color" | "颜色" | "填充" | "fill" => {
                vec![FillItem::Object(FillObject::Solid {
                    color: "#000000".to_string(),
                    enabled: true,
                })]
            }
            "transparent" | "透明" => {
                let transparent_color = state
                    .doc
                    .find_element(&id)
                    .and_then(|element| element.fill.as_ref())
                    .map(ToString::to_string)
                    .and_then(|fill| extract_hex_token(&fill))
                    .map(|hex| with_alpha(&hex, 0x66))
                    .unwrap_or_else(|| "#00000066".to_string());
                vec![FillItem::Object(FillObject::Solid {
                    color: transparent_color,
                    enabled: true,
                })]
            }
            "none" | "不填充" => vec![],
            _ if fill_type.starts_with('#') => {
                vec![FillItem::Object(FillObject::Solid {
                    color: fill_type,
                    enabled: true,
                })]
            }
            _ => vec![],
        };
        let value = serde_json::to_value(fills).unwrap_or(serde_json::Value::Null);
        return Task::done(Message::Design(DesignMessage::PropertyUpdate(
            id,
            "fill".to_string(),
            value,
        )));
    }
    Task::none()
}

/// handle_update_context_border 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn handle_update_context_border(
    state: &mut DesignState,
    border_type: String,
) -> Task<Message> {
    if let Some(id) = state.selected_element_id.clone() {
        let (mode, color_override) = if let Some((mode, color)) = border_type.split_once('|') {
            (mode, Some(color.to_string()))
        } else {
            (border_type.as_str(), None)
        };
        let stroke = match mode {
            "solid" | "实线" => {
                let color = color_override.unwrap_or_else(|| "#000000".to_string());
                Some(Stroke {
                    align: Some("inside".to_string()),
                    thickness: Some(serde_json::json!(1.0)),
                    fill: Some(format!(
                        "[{{\"type\":\"solid\",\"color\":\"{}\",\"opacity\":1.0}}]",
                        color
                    )),
                })
            }
            "dashed" | "虚线" => {
                let color = color_override.unwrap_or_else(|| "#000000".to_string());
                Some(Stroke {
                    align: Some("inside".to_string()),
                    thickness: Some(serde_json::json!(1.0)),
                    fill: Some(format!(
                        "[{{\"type\":\"solid\",\"color\":\"{}\",\"opacity\":1.0,\"dashArray\":[4,4]}}]",
                        color
                    )),
                })
            }
            "none" | "无" => None,
            _ if mode.starts_with('#') => Some(Stroke {
                align: Some("inside".to_string()),
                thickness: Some(serde_json::json!(1.0)),
                fill: Some(format!(
                    "[{{\"type\":\"solid\",\"color\":\"{}\",\"opacity\":1.0}}]",
                    mode
                )),
            }),
            _ => None,
        };
        let value = serde_json::to_value(stroke).unwrap_or(serde_json::Value::Null);
        return Task::done(Message::Design(DesignMessage::PropertyUpdate(
            id,
            "stroke".to_string(),
            value,
        )));
    }
    Task::none()
}

