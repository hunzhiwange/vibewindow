//! 设计视图设置消息处理模块
//!
//! 本模块负责处理设计视图中与设置相关的消息更新逻辑。它提供了一个统一的
//! `update` 函数来处理各种设置切换操作，包括界面元素的显示/隐藏、功能开关等。
//!
//! # 主要功能
//!
//! - 切换变量面板的显示状态
//! - 切换快捷键提示的显示状态
//! - 切换设置面板的显示状态
//! - 切换设置面板中的标签页
//! - 控制鼠标滚轮缩放功能的启用/禁用
//! - 控制槽位内容显示功能的启用/禁用
//! - 控制槽位溢出显示功能的启用/禁用
//! - 切换属性面板的显示状态
//!
//! # 配置持久化
//!
//! 某些设置项会在变更时自动持久化到配置文件中，包括：
//! - `show_slot_content`: 槽位内容显示开关
//! - `show_slot_overflow`: 槽位溢出显示开关
//! - `show_properties_panel`: 属性面板显示开关
//!
//! # 画布缓存处理
//!
//! 当影响视觉呈现的设置项（如槽位内容/溢出显示）发生变更时，会清空画布缓存
//! 以确保界面正确重绘。

use crate::app::message::DesignMessage;
use crate::app::{App, Message};
use iced::Task;

/// 处理设计视图设置相关的消息更新
///
/// 该函数根据接收到的 `DesignMessage` 消息类型，更新应用程序状态并执行相应的操作。
/// 对于需要持久化的设置项，会自动保存到配置文件中。
///
/// # 参数
///
/// - `app`: 可变引用的应用程序状态，用于读取和更新各种设置项
/// - `message`: 设计消息枚举，指示需要执行的操作类型
///
/// # 返回值
///
/// 返回 `Task<Message>`，可能包含需要在事件循环中执行的后续任务。
/// 对于大多数设置切换操作，返回 `Task::none()` 表示无需额外任务。
///
/// # 示例
///
/// ```ignore
/// let task = update(&mut app, DesignMessage::ToggleShortcuts);
/// // 快捷键提示显示状态已切换，task 通常为 Task::none()
/// ```
///
/// # 处理的消息类型
///
/// - `ToggleVariables`: 切换变量面板显示状态
/// - `ToggleShortcuts`: 切换快捷键提示显示状态
/// - `ToggleSettings`: 切换设置面板显示状态
/// - `DesignSettingsSelectTab`: 切换设置面板中的标签页
/// - `ToggleMouseWheelZoom`: 启用/禁用鼠标滚轮缩放功能
/// - `ToggleSlotContent`: 启用/禁用槽位内容显示，并清空画布缓存
/// - `ToggleSlotOverflow`: 启用/禁用槽位溢出显示，并清空画布缓存
/// - `TogglePropertiesPanel`: 切换属性面板显示状态
/// - 其他消息: 返回空任务（默认处理）
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        // 切换变量面板显示状态
        DesignMessage::ToggleVariables => {
            app.show_design_variables = !app.show_design_variables;
            if let Some(state) = app.active_design_state_mut() {
                state.active_variable_theme_menu = None;
                state.renaming_variable_theme = None;
                state.variable_theme_rename_value.clear();
                state.confirm_delete_variable_theme = None;
                state.current_variable_collection = None;
                state.active_variable_collection_menu = None;
                state.renaming_variable_collection = None;
                state.variable_collection_rename_value.clear();
                state.confirm_delete_variable_collection = None;
                state.active_variable_menu = None;
                state.variable_move_target_picker = None;
                state.renaming_variable = None;
                state.variable_rename_value.clear();
                state.confirm_delete_variable = None;
                state.show_add_variable_menu = false;
            }
            Task::none()
        }

        // 切换快捷键提示的显示状态
        DesignMessage::ToggleShortcuts => {
            app.show_design_shortcuts = !app.show_design_shortcuts;
            Task::none()
        }

        // 切换设计设置面板的显示状态
        DesignMessage::ToggleSettings => {
            app.show_design_settings = !app.show_design_settings;
            Task::none()
        }

        DesignMessage::DesignSettingsSelectTab(tab) => {
            app.design_settings_active_tab = tab;
            Task::none()
        }

        // 启用或禁用鼠标滚轮缩放功能
        DesignMessage::ToggleMouseWheelZoom(enabled) => {
            app.mouse_wheel_zoom_enabled = enabled;
            Task::none()
        }

        // 启用或禁用槽位内容显示
        // 此设置会影响视觉呈现，需要持久化并清空画布缓存以触发重绘
        DesignMessage::ToggleSlotContent(enabled) => {
            app.show_slot_content = enabled;
            // 持久化设置到配置文件
            crate::app::set_config_field("show_slot_content", serde_json::Value::Bool(enabled));
            // 清空画布缓存以确保界面正确重绘
            if let Some(state) = app.active_design_state_mut() {
                state.canvas_cache.clear();
            }
            Task::none()
        }

        // 启用或禁用槽位溢出显示
        // 此设置会影响视觉呈现，需要持久化并清空画布缓存以触发重绘
        DesignMessage::ToggleSlotOverflow(enabled) => {
            app.show_slot_overflow = enabled;
            // 持久化设置到配置文件
            crate::app::set_config_field("show_slot_overflow", serde_json::Value::Bool(enabled));
            // 清空画布缓存以确保界面正确重绘
            if let Some(state) = app.active_design_state_mut() {
                state.canvas_cache.clear();
            }
            Task::none()
        }

        // 切换属性面板的显示状态
        // 此设置需要持久化到配置文件以在下次启动时恢复状态
        DesignMessage::TogglePropertiesPanel => {
            app.show_properties_panel = !app.show_properties_panel;
            // 持久化设置到配置文件
            crate::app::set_config_field(
                "show_properties_panel",
                serde_json::Value::Bool(app.show_properties_panel),
            );
            Task::none()
        }

        // 处理其他未明确处理的消息类型，返回空任务
        _ => Task::none(),
    }
}
