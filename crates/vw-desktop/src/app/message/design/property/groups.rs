//! 处理设计属性面板中的分组级操作，维持属性组状态一致。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::common::clone_page_elements;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::{App, Message};
use iced::Task;

/// set_active_group 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_active_group(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    state.ensure_valid_group();
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    if state.renaming_page_id != Some(group_id) {
        state.renaming_page_id = None;
        state.renaming_page_name.clear();
    }
    if state.active_group_id == group_id {
        return Task::none();
    }
    if !state.doc.groups.iter().any(|group| group.id == group_id) {
        return Task::none();
    }
    state.active_group_id = group_id;
    let focus_id = state.focus_first_element_in_active_group();
    if let Some(id) = focus_id {
        Task::done(Message::Design(DesignMessage::FitToElement(id)))
    } else {
        Task::none()
    }
}

/// new_group_name_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn new_group_name_changed(app: &mut App, value: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.new_group_name = value;
    }
    Task::none()
}

/// create_group 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn create_group(app: &mut App) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    state.ensure_valid_group();
    let group_id = state.doc.next_group_id();
    let group_name = {
        let trimmed = state.new_group_name.trim();
        if trimmed.is_empty() {
            crate::app::views::design::models::DesignDoc::default_group_name(group_id)
        } else {
            trimmed.to_string()
        }
    };
    state.doc.groups.push(crate::app::views::design::models::DesignGroup {
        id: group_id,
        name: group_name,
    });
    state.doc.normalize_groups();
    state.active_group_id = group_id;
    state.new_group_name.clear();
    state.selected_element_id = None;
    state.selected_element_ids.clear();
    state.selected_fill_index = None;
    state.selected_effect_index = None;
    state.canvas_cache.clear();
    Task::none()
}

/// toggle_page_menu 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_page_menu(app: &mut App, group_id: u32, x: f32, y: f32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    if state.active_page_menu == Some(group_id) {
        state.active_page_menu = None;
        state.page_menu_anchor = None;
    } else {
        state.active_page_menu = Some(group_id);
        state.page_menu_anchor = Some(iced::Point::new(x, y));
    }
    Task::none()
}

/// close_page_menu 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_page_menu(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.active_page_menu = None;
        state.page_menu_anchor = None;
    }
    Task::none()
}

/// rename_page_requested 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn rename_page_requested(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    if let Some(group) = state.doc.groups.iter().find(|group| group.id == group_id) {
        state.renaming_page_id = Some(group_id);
        state.renaming_page_name = group.name.clone();
    }
    Task::none()
}

/// duplicate_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn duplicate_page(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let Some(group_index) = state.doc.groups.iter().position(|group| group.id == group_id) else {
        return Task::none();
    };
    let new_group_id = state.doc.next_group_id();
    let source_name = state
        .doc
        .groups
        .get(group_index)
        .map(|group| group.name.clone())
        .unwrap_or_else(|| "页面".to_string());
    let source_elements = state
        .doc
        .children
        .iter()
        .filter(|child| child.group_id == group_id)
        .cloned()
        .collect::<Vec<_>>();
    let cloned_elements = clone_page_elements(&source_elements, new_group_id);
    state.doc.groups.insert(
        group_index + 1,
        crate::app::views::design::models::DesignGroup {
            id: new_group_id,
            name: format!("{source_name} 副本"),
        },
    );
    state.doc.children.extend(cloned_elements);
    state.active_group_id = new_group_id;
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    state.renaming_page_id = None;
    state.renaming_page_name.clear();
    state.layer_tree_metrics = compute_tree_metrics(&state.doc);
    let focus_id = state.focus_first_element_in_active_group();
    if let Some(id) = focus_id {
        Task::done(Message::Design(DesignMessage::FitToElement(id)))
    } else {
        Task::none()
    }
}

/// delete_page 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn delete_page(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let Some(group_index) = state.doc.groups.iter().position(|group| group.id == group_id) else {
        return Task::none();
    };
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    state.renaming_page_id = None;
    state.renaming_page_name.clear();
    state.doc.children.retain(|child| child.group_id != group_id);
    if state.doc.groups.len() > 1 {
        state.doc.groups.remove(group_index);
        if state.active_group_id == group_id {
            let next_index = group_index.min(state.doc.groups.len().saturating_sub(1));
            if let Some(group) = state.doc.groups.get(next_index) {
                state.active_group_id = group.id;
            }
        }
    } else if let Some(group) = state.doc.groups.first_mut() {
        group.name = crate::app::views::design::models::DesignDoc::default_group_name(group.id);
        state.active_group_id = group.id;
    }
    state.layer_tree_metrics = compute_tree_metrics(&state.doc);
    let focus_id = state.focus_first_element_in_active_group();
    if let Some(id) = focus_id {
        Task::done(Message::Design(DesignMessage::FitToElement(id)))
    } else {
        Task::none()
    }
}

/// move_page_up 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn move_page_up(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    if let Some(group_index) = state.doc.groups.iter().position(|group| group.id == group_id)
        && group_index > 0
    {
        state.doc.groups.swap(group_index - 1, group_index);
    }
    Task::none()
}

/// move_page_down 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn move_page_down(app: &mut App, group_id: u32) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    state.active_page_menu = None;
    state.page_menu_anchor = None;
    if let Some(group_index) = state.doc.groups.iter().position(|group| group.id == group_id)
        && group_index + 1 < state.doc.groups.len()
    {
        state.doc.groups.swap(group_index, group_index + 1);
    }
    Task::none()
}

/// page_rename_changed 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn page_rename_changed(app: &mut App, value: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.renaming_page_name = value;
    }
    Task::none()
}

/// submit_page_rename 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn submit_page_rename(app: &mut App) -> Task<Message> {
    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };
    let Some(group_id) = state.renaming_page_id.take() else {
        return Task::none();
    };
    let new_name = state.renaming_page_name.trim().to_string();
    state.renaming_page_name.clear();
    if new_name.is_empty() {
        return Task::none();
    }
    if let Some(group) = state.doc.groups.iter_mut().find(|group| group.id == group_id) {
        group.name = new_name;
    }
    Task::none()
}

/// cancel_page_rename 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn cancel_page_rename(app: &mut App) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.renaming_page_id = None;
        state.renaming_page_name.clear();
    }
    Task::none()
}

