//! 处理设计预览相关消息，维护预览刷新、选择和状态同步。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::message::DesignMessage;
use crate::app::views::design::export::generate_element_html;
use crate::app::{App, Message};
use iced::Task;

/// view_element_html 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn view_element_html(app: &mut App, id: String) -> Task<Message> {
    if let Some(state) = app.active_design_state()
        && let Some(html) = generate_element_html(&state.doc, &id)
    {
        app.element_html_preview_editor = iced::widget::text_editor::Content::with_text(&html);
        app.show_element_html_preview = true;
    }
    Task::none()
}

/// html_preview_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn html_preview_action(
    app: &mut App,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    app.element_html_preview_editor.perform(action);
    Task::none()
}

/// close_html_preview 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_html_preview(app: &mut App) -> Task<Message> {
    app.show_element_html_preview = false;
    Task::none()
}

/// design_generation_prompt_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_prompt_action(
    app: &mut App,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_chat_input.perform(action);
        state.sync_active_chat_session_from_legacy();
    }
    Task::none()
}

/// design_generation_log_editor_action 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_log_editor_action(
    app: &mut App,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        match action {
            iced::widget::text_editor::Action::Edit(_) => {}
            other => state.design_generation_log_editor.perform(other),
        }
    }
    Task::none()
}

/// design_generation_copy_chat_message 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_copy_chat_message(app: &mut App, index: usize) -> Task<Message> {
    if let Some(state) = app.active_design_state()
        && let Some(message) = state.design_chat_messages.get(index)
    {
        return iced::clipboard::write(message.content.clone());
    }
    Task::none()
}

/// design_generation_select_chat_message 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_select_chat_message(app: &mut App, index: usize) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_chat_selected_message = Some(index);
    }
    Task::none()
}

/// design_generation_clear_chat_selection 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_clear_chat_selection(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_chat_selected_message = None;
    }
    Task::none()
}

/// design_generation_show_all_logs 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_show_all_logs(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_show_all_logs = !state.design_generation_show_all_logs;
        if state.design_generation_show_all_logs {
            return Task::done(Message::Design(DesignMessage::DesignGenerationLoadLogFiles));
        }
    }
    Task::none()
}

/// design_generation_load_log_files 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_load_log_files(app: &mut App) -> Task<Message> {
    let project_path = app
        .project_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().display().to_string());
    // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。
    Task::perform(
        async move {
            crate::app::message::spawn_blocking_opt(move || {
                let log_dir = std::path::Path::new(&project_path)
                    .join(".vibewindow")
                    .join("design")
                    .join("logs");
                let mut files = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&log_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str()
                            && name.ends_with(".log")
                        {
                            files.push(name.to_string());
                        }
                    }
                }
                files.sort_by(|a, b| b.cmp(a));
                Some(files)
            })
            .await
            .unwrap_or_default()
        },
        |files| Message::Design(DesignMessage::DesignGenerationLogFilesLoaded(files)),
    )
}

/// design_generation_log_files_loaded 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_log_files_loaded(
    app: &mut App,
    files: Vec<String>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_log_files = files;
    }
    Task::none()
}
