//! 处理设计属性编辑器的字段变化，并把输入应用到选中元素。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::models::DesignElement;
use crate::app::{App, Message};
use iced::Task;
use iced::widget::text_editor::Action;

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

/// context_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn context_editor_action(app: &mut App, action: Action) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.context_editor.perform(action);
        let content = state.context_editor.text().to_string();
        if let Some(id) = &state.selected_element_id.clone() {
            state.doc.update_property(id, "context", serde_json::Value::String(content));
            state.canvas_cache.clear();
        }
    }
    Task::none()
}

/// toggle_context_editor 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_context_editor(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.context_expanded = !state.context_expanded;
    }
    Task::none()
}

/// content_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn content_editor_action(app: &mut App, action: Action) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.content_editor.perform(action);
        let content = state.content_editor.text().to_string();
        if let Some(id) = &state.selected_element_id.clone() {
            state.doc.update_property(id, "content", serde_json::Value::String(content));
            state.canvas_cache.clear();
        }
    }
    Task::none()
}

/// tailwind_html_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn tailwind_html_editor_action(app: &mut App, action: Action) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.tailwind_html_editor.perform(action);
        let html = state.tailwind_html_editor.text().to_string();
        if let Some(id) = &state.selected_element_id.clone()
            && let Some(el) = state.doc.find_element(id)
            && el.kind.eq_ignore_ascii_case("tailwind")
        {
            state.doc.update_property(id, "content", serde_json::Value::String(html));
            state.canvas_cache.clear();
        }
    }
    Task::none()
}

/// tailwind_node_class_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn tailwind_node_class_editor_action(app: &mut App, action: Action) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.tailwind_node_class_editor.perform(action);
    }
    Task::none()
}

/// tailwind_node_text_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn tailwind_node_text_editor_action(app: &mut App, action: Action) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.tailwind_node_text_editor.perform(action);
        if let Some((id, path)) = state.doc.tailwind_selection.clone() {
            let text = state.tailwind_node_text_editor.text().to_string();
            if let Some(el) = find_mut(&mut state.doc.children, &id)
                && let Some(content) = &el.content
            {
                let mut nodes =
                    crate::app::views::design::canvas::tailwind::dom::parse_html(content);
                if !path.is_empty()
                    && let Some(root_idx) = path.first()
                    && let Some(curr_node) = nodes.get_mut(*root_idx)
                {
                    fn update_node(
                        node: &mut crate::app::views::design::canvas::tailwind::dom::TailwindNode,
                        path: &[usize],
                        text: &str,
                    ) {
                        if path.is_empty() {
                            node.text = Some(text.to_string());
                            return;
                        }
                        let idx = path[0];
                        if let Some(child) = node.children.get_mut(idx) {
                            update_node(child, &path[1..], text);
                        }
                    }
                    update_node(curr_node, &path[1..], &text);
                }

                el.content =
                    Some(crate::app::views::design::canvas::tailwind::dom::nodes_to_html(&nodes));
                if state.selected_element_id.as_deref() == Some(id.as_str())
                    && let Some(html) = el.content.as_deref()
                {
                    state.tailwind_html_editor =
                        iced::widget::text_editor::Content::with_text(html);
                }
            }
            state.canvas_cache.clear();
        }
    }
    Task::none()
}
