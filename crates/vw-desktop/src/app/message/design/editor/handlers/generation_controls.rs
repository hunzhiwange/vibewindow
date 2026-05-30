//! 处理设计生成控制项的输入变化，保持生成参数与应用状态同步。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::super::canvas::{
    find_generation_page_mut, normalize_target_frame_id, sync_module_placeholder_status,
};
use super::super::logging::{
    format_design_log_stream, push_design_generation_log, push_design_stream_line_to_chat,
};
use super::super::prompts::design_model_input_placeholder;
use crate::app::message::DesignMessage;
use crate::app::task::{TASK_MODEL_AUTO, TaskExecutorBackend, normalize_task_model_input};
use crate::app::views::design::state::{
    DesignChatMessage, DesignChatRole, DesignGenerationStatus, DesignPlannerTab,
    sanitize_design_generation_parallel_pages,
};
use crate::app::{App, Message};
use iced::Task;

/// toggle_design_generation_executor_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_generation_executor_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_executor_popover = !state.design_generation_executor_popover;
        if state.design_generation_executor_popover {
            state.design_generation_model_popover = false;
            state.design_generation_theme_popover = false;
            state.design_generation_device_popover = false;
            state.design_generation_style_popover = false;
        }
    }
    Task::none()
}

/// close_design_generation_executor_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_generation_executor_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_executor_popover = false;
    }
    Task::none()
}

/// design_generation_acp_agent_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_acp_agent_selected(
    app: &mut App,
    agent: Option<String>,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_executor = TaskExecutorBackend::Internal;
        state.design_generation_executor_popover = false;
        if state.design_generation_model.trim().is_empty() {
            state.design_generation_model = TASK_MODEL_AUTO.to_string();
        }
        state.design_generation_summary = Some(format!(
            "ACP 智能体已切换为 {}，模型输入建议：{}",
            agent.as_deref().unwrap_or("ACP 智能体"),
            design_model_input_placeholder()
        ));
    }
    app.current_session_runtime_mut().acp_agent = agent.clone();
    app.acp_agent = agent;
    Task::none()
}

/// toggle_design_generation_model_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_generation_model_popover(app: &mut App) -> Task<Message> {
    let mut should_refresh_models = false;
    if let Some(state) = app.active_design_state_mut() {
        let new = !state.design_generation_model_popover;
        state.design_generation_model_popover = new;
        if new {
            state.design_generation_executor_popover = false;
            state.design_generation_theme_popover = false;
            state.design_generation_device_popover = false;
            state.design_generation_style_popover = false;
            if !app.model_settings.loading && app.model_settings.providers.is_empty() {
                should_refresh_models = true;
            }
        }
    }
    if should_refresh_models {
        return Task::done(Message::Settings(crate::app::message::SettingsMessage::ModelsRefresh));
    }
    Task::none()
}

/// close_design_generation_model_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_generation_model_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_model_popover = false;
    }
    Task::none()
}

/// design_generation_model_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_model_selected(app: &mut App, model: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_model = normalize_task_model_input(&model);
        state.design_generation_model_popover = false;
    }
    Task::none()
}

/// design_generation_style_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_style_selected(
    app: &mut App,
    style: crate::app::views::design::state::DesignStyle,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_style = style;
        state.design_generation_style_popover = false;
    }
    Task::none()
}

/// design_generation_device_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_device_selected(
    app: &mut App,
    device: crate::app::views::design::state::DesignGenerationDevice,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_device = device;
        state.design_generation_device_popover = false;
        state.design_generation_summary =
            Some(format!("端类型已切换为 {}，后续按该宽度策略生成。", device.label()));
    }
    Task::none()
}

/// design_generation_model_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_model_changed(app: &mut App, model: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_model = if model.trim().is_empty() {
            TASK_MODEL_AUTO.to_string()
        } else {
            normalize_task_model_input(&model)
        };
    }
    Task::none()
}

/// design_generation_parallel_pages_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_parallel_pages_changed(
    app: &mut App,
    value: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let filtered = value.chars().filter(|ch| ch.is_ascii_digit()).collect::<String>();
        state.design_generation_parallel_pages_input = filtered.clone();
        let parsed = filtered.parse::<usize>().ok().filter(|value| *value > 0);
        if let Some(parsed) = parsed {
            let sanitized = sanitize_design_generation_parallel_pages(parsed);
            state.design_generation_parallel_pages = sanitized;
            if sanitized.to_string() != filtered {
                state.design_generation_parallel_pages_input = sanitized.to_string();
            }
            crate::app::set_config_field(
                "design_generation_parallel_pages",
                serde_json::Value::Number(serde_json::Number::from(sanitized as u64)),
            );
            state.design_generation_summary =
                Some(format!("页面并行数已设置为 {}，后续将按该并发度生成。", sanitized));
        } else if filtered.is_empty() {
            state.design_generation_summary = Some("请输入页面并行数，最小为 1。".to_string());
        }
    }
    Task::none()
}

/// design_generation_theme_selected 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_generation_theme_selected(
    app: &mut App,
    theme: crate::app::views::design::state::DesignGenerationTheme,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_theme = theme;
        state.design_generation_theme_popover = false;
        state.design_generation_summary = Some(format!(
            "风格已切换为 {}，将基于该主题的 tokens、组件和提示词生成。",
            theme.label()
        ));
    }
    Task::none()
}

/// toggle_design_generation_theme_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_generation_theme_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let new = !state.design_generation_theme_popover;
        state.design_generation_theme_popover = new;
        if new {
            state.design_generation_executor_popover = false;
            state.design_generation_model_popover = false;
            state.design_generation_device_popover = false;
            state.design_generation_style_popover = false;
        }
    }
    Task::none()
}

/// close_design_generation_theme_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_generation_theme_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_theme_popover = false;
    }
    Task::none()
}

/// toggle_design_generation_device_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_generation_device_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let new = !state.design_generation_device_popover;
        state.design_generation_device_popover = new;
        if new {
            state.design_generation_executor_popover = false;
            state.design_generation_model_popover = false;
            state.design_generation_theme_popover = false;
            state.design_generation_style_popover = false;
        }
    }
    Task::none()
}

/// close_design_generation_device_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_generation_device_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_device_popover = false;
    }
    Task::none()
}

/// toggle_design_generation_style_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_generation_style_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let new = !state.design_generation_style_popover;
        state.design_generation_style_popover = new;
        if new {
            state.design_generation_executor_popover = false;
            state.design_generation_model_popover = false;
            state.design_generation_theme_popover = false;
            state.design_generation_device_popover = false;
        }
    }
    Task::none()
}

/// close_design_generation_style_popover 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_generation_style_popover(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_style_popover = false;
    }
    Task::none()
}

/// design_generation_stream_tick 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_stream_tick(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_generation_anim_frame = state.design_generation_anim_frame.wrapping_add(1);
        if let Some(receiver) = state.design_generation_stream_rx.take() {
            let mut active_receiver = Some(receiver);
            for _ in 0..24 {
                let Some(current_receiver) = active_receiver.as_ref() else {
                    break;
                };
                match current_receiver.try_recv() {
                    Ok(log) => {
                        if let Some(line) = format_design_log_stream(&log) {
                            let scoped_line = format!("[plan] {}", line);
                            push_design_generation_log(state, scoped_line.clone());
                            push_design_stream_line_to_chat(state, &scoped_line);
                            state.design_generation_stream_cursor =
                                state.design_generation_stream_cursor.saturating_add(1);
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        active_receiver = None;
                        break;
                    }
                }
            }
            state.design_generation_stream_rx = active_receiver;
        }
        for page in state.design_generation_pages.clone() {
            for module in page.modules {
                if module.status == DesignGenerationStatus::Running || module.is_generating {
                    sync_module_placeholder_status(
                        state,
                        &module.target_frame_id,
                        DesignGenerationStatus::Running,
                    );
                }
            }
        }
    }
    Task::none()
}

/// design_generation_cancel 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_cancel(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        if !state.design_generation_loading {
            return Task::done(Message::Design(DesignMessage::Snapshot));
        }
        state.design_generation_loading = false;
        state.design_generation_stream_rx = None;
        state.design_generation_anim_frame = 0;
        for page in &mut state.design_generation_pages {
            for module in &mut page.modules {
                if module.is_generating || module.status == DesignGenerationStatus::Running {
                    module.is_generating = false;
                    module.status = DesignGenerationStatus::Queued;
                }
            }
        }
        for page in state.design_generation_pages.clone() {
            for module in page.modules {
                if module.status == DesignGenerationStatus::Queued {
                    sync_module_placeholder_status(
                        state,
                        &module.target_frame_id,
                        DesignGenerationStatus::Queued,
                    );
                }
            }
        }
        state.design_generation_summary = Some("已取消当前生成任务。".to_string());
        state.design_chat_messages.push(DesignChatMessage {
            role: DesignChatRole::Assistant,
            content: "已停止本轮生成，你可以继续修改需求后重新生成。".to_string(),
        });
        state.design_chat_selected_message = None;
        state.sync_active_chat_session_from_legacy();
    }
    Task::done(Message::Design(DesignMessage::Snapshot))
}

/// toggle_design_planner_panel_collapsed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_design_planner_panel_collapsed(app: &mut App) -> Task<Message> {
    app.show_design_planner_panel = !app.show_design_planner_panel;
    crate::app::set_config_field(
        "show_design_planner_panel",
        serde_json::Value::Bool(app.show_design_planner_panel),
    );
    Task::none()
}

/// design_planner_select_tab 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_planner_select_tab(app: &mut App, tab: DesignPlannerTab) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_planner_active_tab = tab;
    }
    Task::none()
}

/// open_design_planner_quick_menu 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn open_design_planner_quick_menu(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_planner_quick_menu_open = true;
    }
    Task::none()
}

/// close_design_planner_quick_menu 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_design_planner_quick_menu(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.design_planner_quick_menu_open = false;
    }
    Task::none()
}

/// design_planner_set_corner 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn design_planner_set_corner(
    app: &mut App,
    corner: crate::app::views::design::state::DesignPlannerCorner,
) -> Task<Message> {
    app.design_planner_corner = corner;
    if let Some(state) = app.active_design_state_mut() {
        state.design_planner_quick_menu_open = false;
    }
    crate::app::set_config_field(
        "design_planner_corner",
        serde_json::Value::String(corner.config_key().to_string()),
    );
    Task::none()
}

/// design_planner_new_chat_session 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_planner_new_chat_session(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.create_design_chat_session();
        state.design_planner_active_tab = DesignPlannerTab::Chat;
        state.design_planner_quick_menu_open = false;
    }
    app.show_design_planner_panel = true;
    crate::app::set_config_field(
        "show_design_planner_panel",
        serde_json::Value::Bool(app.show_design_planner_panel),
    );
    Task::none()
}

/// design_planner_select_chat_session 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_planner_select_chat_session(app: &mut App, index: usize) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.select_design_chat_session(index);
        state.design_planner_active_tab = DesignPlannerTab::Chat;
    }
    Task::none()
}

/// design_generation_apply_partial_regenerate 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn design_generation_apply_partial_regenerate(app: &mut App) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    if state.design_generation_loading {
        state.design_generation_summary = Some("当前正在生成中，请稍后重试。".to_string());
        return Task::none();
    }
    let mut targets = Vec::new();
    for page in &state.design_generation_pages {
        let failed = page
            .modules
            .iter()
            .find(|module| matches!(module.status, DesignGenerationStatus::Failed))
            .or_else(|| page.modules.first());
        if let Some(module) = failed {
            targets.push((page.frame_id.clone(), module.module_id.clone()));
        }
    }
    if targets.is_empty() {
        state.design_generation_summary = Some("暂无可重新生成的模块。".to_string());
        return Task::none();
    }
    state.design_generation_summary =
        Some(format!("已触发重新生成：{} 个页面任务。", targets.len()));
    state.design_generation_loading = true;
    let mut tasks = vec![Task::done(Message::Design(DesignMessage::Snapshot))];
    tasks.extend(targets.into_iter().map(|(page_id, module_id)| {
        Task::done(Message::Design(DesignMessage::RegenerateDesignPage(page_id, module_id)))
    }));
    Task::batch(tasks)
}

/// set_design_page_target_frame 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn set_design_page_target_frame(
    app: &mut App,
    page_frame_id: String,
    module_id: String,
    target_frame_id: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut()
        && let Some(page) =
            find_generation_page_mut(&mut state.design_generation_pages, &page_frame_id)
        && let Some(module) = page.modules.iter_mut().find(|module| module.module_id == module_id)
    {
        let normalized = normalize_target_frame_id(&target_frame_id);
        module.target_frame_id = normalized.clone();
        if !module.target_frame_options.iter().any(|option| option == &normalized) {
            module.target_frame_options.push(normalized.clone());
        }
        state.design_generation_summary =
            Some(format!("模块“{}”的汇总目标已切换为 {}", module.title, normalized));
    }
    Task::none()
}
