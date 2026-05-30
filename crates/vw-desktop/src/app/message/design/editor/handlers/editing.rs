//! 处理设计编辑器中的基础编辑动作，协调元素状态与界面反馈。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::{App, Message};
use iced::Task;

/// edit_start 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn edit_start(app: &mut App, id: String, content: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.editing_id = Some(id);
        state.editing_content = content;
        state.editing_editor =
            iced::widget::text_editor::Content::with_text(&state.editing_content);
        state.canvas_cache.clear();
        if let Some(edit_id) = &state.editing_id {
            state.selected_element_id = Some(edit_id.clone());
            state.selected_element_ids.clear();
            state.selected_element_ids.insert(edit_id.clone());
        }
    }
    Task::none()
}

/// edit_content_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn edit_content_changed(app: &mut App, content: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.editing_content = content;
        state.editing_editor =
            iced::widget::text_editor::Content::with_text(&state.editing_content);
    }
    Task::none()
}

/// edit_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn edit_editor_action(
    app: &mut App,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.editing_editor.perform(action);
        state.editing_content = state.editing_editor.text().to_string();
    }
    Task::none()
}

/// edit_submit 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn edit_submit(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if let Some(id) = &state.editing_id {
            state.editing_content = state.editing_editor.text().to_string();
            state.doc.update_property(
                id,
                "content",
                serde_json::Value::String(state.editing_content.clone()),
            );
            state.canvas_cache.clear();
            state.selected_element_id = Some(id.clone());
            state.selected_element_ids.clear();
            state.selected_element_ids.insert(id.clone());
        }
        state.editing_id = None;
        state.editing_content.clear();
        state.editing_editor = iced::widget::text_editor::Content::new();
    }
    Task::none()
}

/// edit_cancel 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn edit_cancel(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if let Some(id) = &state.editing_id {
            state.selected_element_id = Some(id.clone());
            state.selected_element_ids.clear();
            state.selected_element_ids.insert(id.clone());
        }
        state.canvas_cache.clear();
        state.editing_id = None;
        state.editing_content.clear();
        state.editing_editor = iced::widget::text_editor::Content::new();
    }
    Task::none()
}
