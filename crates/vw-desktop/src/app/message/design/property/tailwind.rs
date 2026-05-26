//! 处理 Tailwind 属性输入和解析，把类名变化同步到设计元素。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::utils::apply_tailwind_classes;
use crate::app::views::design::models::DesignElement;
use crate::app::views::design::properties::ActiveTailwindClassPicker;
use crate::app::{App, Message};
use iced::Task;

fn find_mut<'a>(elements: &'a mut Vec<DesignElement>, id: &str) -> Option<&'a mut DesignElement> {
    for el in elements {
        if el.id == id {
            return Some(el);
        }
        if let Some(found) = find_mut(&mut el.children, id) {
            return Some(found);
        }
    }
    None
}

fn apply_classes_for_element(state: &mut crate::app::views::design::state::DesignState, element_id: &str) {
    if let Some(el) = find_mut(&mut state.doc.children, element_id) {
        apply_tailwind_classes(el);
    }
    state.canvas_cache.clear();
}

/// set_filter 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_filter(app: &mut App, query: String) -> Task<Message> {
    app.tailwind_filter_query = query;
    Task::none()
}

/// open_class_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_class_picker(
    app: &mut App,
    element_id: String,
    position_opt: Option<iced::Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    app.active_color_picker = None;
    app.active_fill_picker = None;
    app.active_effect_picker = None;
    app.active_font_picker = None;
    app.active_icon_picker = None;
    app.design_help_text = None;
    app.tailwind_filter_query.clear();
    app.active_tailwind_class_picker = Some(ActiveTailwindClassPicker { element_id, position });
    Task::none()
}

/// close_class_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_class_picker(app: &mut App) -> Task<Message> {
    app.active_tailwind_class_picker = None;
    Task::none()
}

/// set_inspector_hover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_inspector_hover(app: &mut App, hovered: bool) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.tailwind_inspector_hovered = hovered;
    }
    Task::none()
}

/// class_input_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn class_input_changed(app: &mut App, element_id: String, input: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if state.selected_element_id.as_deref() != Some(element_id.as_str()) {
            return Task::none();
        }
        state.tailwind_class_input = input.replace('\n', " ");
    }
    Task::none()
}

/// class_input_submit 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn class_input_submit(app: &mut App, element_id: String) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    if state.selected_element_id.as_deref() != Some(element_id.as_str()) {
        return Task::none();
    }

    let raw = state.tailwind_class_input.replace('\n', " ");
    let to_add = raw
        .split_whitespace()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if to_add.is_empty() {
        return Task::none();
    }

    let mut did_update = false;
    if let Some(el) = state.doc.find_element(&element_id) {
        let current = el.class.as_deref().unwrap_or("");
        let mut tokens = crate::app::views::design::properties::split_class_tokens(current);
        for token in to_add {
            if !tokens.iter().any(|existing| existing == token) {
                tokens.push(token.to_string());
                did_update = true;
            }
        }

        if did_update {
            state.doc.update_property(
                &element_id,
                "class",
                serde_json::Value::String(tokens.join(" ")),
            );
            apply_classes_for_element(state, &element_id);
        }
    }

    if did_update {
        state.tailwind_class_input.clear();
        Task::done(Message::Design(DesignMessage::Snapshot))
    } else {
        Task::none()
    }
}

/// node_class_input_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn node_class_input_changed(
    app: &mut App,
    element_id: String,
    path: Vec<usize>,
    input: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if !state.doc.tailwind_selection.as_ref().is_some_and(|(sel_id, sel_path)| {
            sel_id == &element_id && sel_path.as_slice() == path.as_slice()
        }) {
            return Task::none();
        }
        state.tailwind_node_class_input = input.replace('\n', " ");
        state.tailwind_node_class_dropdown_open = !state.tailwind_node_class_input.trim().is_empty();
    }
    Task::none()
}

/// close_node_class_dropdown 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn close_node_class_dropdown(
    app: &mut App,
    element_id: String,
    path: Vec<usize>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if !state.doc.tailwind_selection.as_ref().is_some_and(|(sel_id, sel_path)| {
            sel_id == &element_id && sel_path.as_slice() == path.as_slice()
        }) {
            return Task::none();
        }
        state.tailwind_node_class_dropdown_open = false;
    }
    Task::none()
}

/// node_class_input_submit 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn node_class_input_submit(
    app: &mut App,
    element_id: String,
    path: Vec<usize>,
) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    if !state.doc.tailwind_selection.as_ref().is_some_and(|(sel_id, sel_path)| {
        sel_id == &element_id && sel_path.as_slice() == path.as_slice()
    }) {
        return Task::none();
    }

    let raw = state.tailwind_node_class_input.replace('\n', " ");
    let to_add = raw
        .split_whitespace()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if to_add.is_empty() {
        return Task::none();
    }

    let mut tokens = state
        .tailwind_node_class_editor
        .text()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut did_update = false;
    for token in to_add {
        if !tokens.iter().any(|existing| existing == token) {
            tokens.push(token.to_string());
            did_update = true;
        }
    }
    if !did_update {
        return Task::none();
    }

    state.tailwind_node_class_input.clear();
    state.tailwind_node_class_dropdown_open = false;
    Task::done(Message::Design(DesignMessage::UpdateTailwindNodeClass(
        element_id,
        path,
        tokens.join(" "),
    )))
}

/// add_class_token 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn add_class_token(app: &mut App, element_id: String, token: String) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let token = token.trim();
    if token.is_empty() {
        return Task::none();
    }

    if let Some(el) = state.doc.find_element(&element_id) {
        let current = el.class.as_deref().unwrap_or("");
        let mut tokens = crate::app::views::design::properties::split_class_tokens(current);
        if !tokens.iter().any(|existing| existing == token) {
            tokens.push(token.to_string());
            state.doc.update_property(
                &element_id,
                "class",
                serde_json::Value::String(tokens.join(" ")),
            );
            apply_classes_for_element(state, &element_id);
        }
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}

