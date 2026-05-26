//! SQL 工具消息处理模块。

mod formatting;
mod persistence;
mod state;


use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;
use formatting::{beautify_sql, compress_sql, purify_sql};
use persistence::save_sql_tool_content_task;
use state::{
    apply_scroll_lines, clear_notification_task, close_context_menu, max_scroll_top_line,
    notify_success,
};

/// SQL 工具消息枚举
///
/// 定义了 SQL 工具支持的所有消息类型，用于处理用户交互和状态更新。
#[derive(Debug, Clone)]
pub enum SqlToolMessage {
    /// 文本编辑器操作，如光标移动、文本输入等
    EditorAction(text_editor::Action),
    OpenContextMenu {
        x: f32,
        y: f32,
    },
    CloseContextMenu,
    ContextMenuCopy,
    ContextMenuCut,
    ContextMenuPaste,
    ContextMenuDelete,
    /// 编辑器滚轮滚动
    EditorWheelScrolled {
        delta: mouse::ScrollDelta,
        viewport_height: f32,
    },
    /// 自定义滚动条位置变化
    ScrollbarChanged {
        top_line: f32,
        viewport_height: f32,
    },
    /// 美化 SQL 语句，格式化为易读的多行形式
    Beautify,
    /// 压缩 SQL 语句，移除多余空白压缩为单行
    Compress,
    /// 净化 SQL 语句，移除注释并保留基本格式
    Purify,
    /// 清空编辑器内容
    Clear,
    /// 复制编辑器内容到剪贴板
    Copy,
    /// 切换记忆功能开关
    ToggleRemember(bool),
    /// 内容更新完成通知，包含处理后的内容
    ContentUpdated(Option<String>),
    /// 清除通知消息
    ClearNotification,
}

/// 处理 SQL 工具消息
///
/// 根据不同的消息类型执行相应的操作，包括编辑器操作、格式化处理、
/// 剪贴板操作和配置持久化等。
///
/// # 参数
///
/// * `app` - 应用状态的可变引用
/// * `message` - 要处理的 SQL 工具消息
///
/// # 返回值
///
/// 返回 iced Task，用于执行异步操作或产生副作用
pub fn update(app: &mut App, message: SqlToolMessage) -> Task<Message> {
    match message {
        // 清除通知消息，重置通知状态
        SqlToolMessage::ClearNotification => {
            app.sql_tool_notification = None;
            Task::none()
        }
        SqlToolMessage::OpenContextMenu { x, y } => {
            app.sql_tool_context_menu_open = true;
            app.sql_tool_context_menu_pos = Some((x, y));
            Task::none()
        }
        SqlToolMessage::CloseContextMenu => {
            close_context_menu(app);
            focus_editor_task(&app.sql_tool_editor_id)
        }
        SqlToolMessage::ContextMenuCopy => {
            close_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.sql_tool_editor, &app.sql_tool_editor_id);

            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        SqlToolMessage::ContextMenuCut => {
            close_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.sql_tool_editor, &app.sql_tool_editor_id);

            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        SqlToolMessage::ContextMenuPaste => {
            close_context_menu(app);
            paste_task(&app.sql_tool_editor_id, |content| {
                Message::SqlTool(SqlToolMessage::EditorAction(paste_action(content)))
            })
        }
        SqlToolMessage::ContextMenuDelete => {
            close_context_menu(app);
            let (_outcome, task) =
                selection_delete_task(&mut app.sql_tool_editor, &app.sql_tool_editor_id);
            task
        }
        // 处理编辑器操作，如文本输入、光标移动等
        SqlToolMessage::EditorAction(action) => {
            close_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            app.sql_tool_editor.perform(action);
            Task::none()
        }
        SqlToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            close_context_menu(app);
            app.sql_tool_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.sql_tool_scroll_remainder += delta_lines;

            let whole_lines = if app.sql_tool_scroll_remainder >= 0.0 {
                app.sql_tool_scroll_remainder.floor() as i32
            } else {
                app.sql_tool_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.sql_tool_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.sql_tool_editor.perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        SqlToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            close_context_menu(app);
            app.sql_tool_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.sql_tool_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.sql_tool_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        // 内容更新成功，更新编辑器并保存（如果启用记忆功能）
        SqlToolMessage::ContentUpdated(Some(content)) => {
            app.sql_tool_loading = false;
            notify_success(app, "操作成功");
            app.sql_tool_editor = text_editor::Content::with_text(&content);
            app.sql_tool_scroll_top_line = 0.0;
            app.sql_tool_scroll_remainder = 0.0;
            close_context_menu(app);

            let save_task = if app.sql_tool_remember {
                save_sql_tool_content_task(content.clone())
            } else {
                Task::none()
            };

            Task::batch(vec![clear_notification_task(), save_task])
        }
        // 内容更新失败或内容为空
        SqlToolMessage::ContentUpdated(None) => {
            app.sql_tool_loading = false;
            app.sql_tool_notification = Some("操作失败或内容为空".to_string());
            close_context_menu(app);
            clear_notification_task()
        }
        // 执行 SQL 美化操作
        SqlToolMessage::Beautify => {
            app.sql_tool_loading = true;
            let text = app.sql_tool_editor.text();
            // 在异步任务中执行耗时的格式化操作
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || beautify_sql(&text)).await
                },
                |res| Message::SqlTool(SqlToolMessage::ContentUpdated(res)),
            )
        }
        // 执行 SQL 压缩操作
        SqlToolMessage::Compress => {
            app.sql_tool_loading = true;
            let text = app.sql_tool_editor.text();
            // 在异步任务中执行压缩操作
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || compress_sql(&text)).await
                },
                |res| Message::SqlTool(SqlToolMessage::ContentUpdated(res)),
            )
        }
        // 执行 SQL 净化操作
        SqlToolMessage::Purify => {
            app.sql_tool_loading = true;
            let text = app.sql_tool_editor.text();
            // 在异步任务中执行净化操作
            Task::perform(
                async move { crate::app::message::spawn_blocking_opt(move || purify_sql(&text)).await },
                |res| Message::SqlTool(SqlToolMessage::ContentUpdated(res)),
            )
        }
        // 清空编辑器内容
        SqlToolMessage::Clear => {
            app.sql_tool_editor = text_editor::Content::new();
            app.sql_tool_scroll_top_line = 0.0;
            app.sql_tool_scroll_remainder = 0.0;
            close_context_menu(app);

            let save_task = if app.sql_tool_remember {
                save_sql_tool_content_task(String::new())
            } else {
                Task::none()
            };

            notify_success(app, "已清空");
            Task::batch(vec![clear_notification_task(), save_task])
        }
        // 复制内容到剪贴板
        SqlToolMessage::Copy => {
            let text = app.sql_tool_editor.text();
            notify_success(app, "已复制");
            close_context_menu(app);
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        // 切换记忆功能开关
        SqlToolMessage::ToggleRemember(val) => {
            app.sql_tool_remember = val;
            close_context_menu(app);
            // 保存配置到持久化存储
            crate::app::set_config_field("sql_tool_remember", serde_json::Value::Bool(val));
            // 如果启用记忆功能，立即保存当前内容
            if val {
                return save_sql_tool_content_task(app.sql_tool_editor.text());
            }
            Task::none()
        }
    }
}
#[cfg(test)]
#[path = "sql_tool_tests.rs"]
mod sql_tool_tests;
