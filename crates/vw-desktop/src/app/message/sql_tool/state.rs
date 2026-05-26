//! 承载 SQL 工具的格式化、持久化与临时状态逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::SqlToolMessage;
use crate::app::{App, Message};
use iced::Task;

/// 执行 close_context_menu 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn close_context_menu(app: &mut App) {
    app.sql_tool_context_menu_open = false;
    app.sql_tool_context_menu_pos = None;
}

/// 执行 notify_success 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn notify_success(app: &mut App, message: &str) {
    app.sql_tool_notification = Some(message.to_string());
}

/// 执行 clear_notification_task 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::SqlTool(SqlToolMessage::ClearNotification),
    )
}

/// 执行 max_scroll_top_line 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.sql_tool_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

/// 执行 apply_scroll_lines 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = max_scroll_top_line(app);
    app.sql_tool_scroll_top_line =
        (app.sql_tool_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.sql_tool_viewport_height / line_height).floor().max(1.0)
}
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
