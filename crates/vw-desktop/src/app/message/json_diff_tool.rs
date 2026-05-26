//! JSON 差异对比工具消息处理模块
//!
//! 本模块负责 JSON 对比工具的状态更新与异步任务调度，核心能力包括：
//! - 左右双编辑器的内容管理
//! - 左右独立滚动与右键菜单
//! - JSON 格式化快捷操作
//! - 结构化 JSON 深度对比
//! - 结果与错误消息回传

use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonDiffEntry {
    pub path: String,
    pub left: Option<String>,
    pub right: Option<String>,
}

impl JsonDiffEntry {
    fn new(path: String, left: Option<String>, right: Option<String>) -> Self {
        Self { path, left, right }
    }
}

#[derive(Debug, Clone)]
pub enum JsonDiffToolMessage {
    LeftEditorAction(text_editor::Action),
    RightEditorAction(text_editor::Action),

    LeftOpenContextMenu { x: f32, y: f32 },
    LeftCloseContextMenu,
    LeftContextMenuCopy,
    LeftContextMenuCut,
    LeftContextMenuPaste,
    LeftContextMenuDelete,
    LeftEditorWheelScrolled { delta: mouse::ScrollDelta, viewport_height: f32 },
    LeftScrollbarChanged { top_line: f32, viewport_height: f32 },

    RightOpenContextMenu { x: f32, y: f32 },
    RightCloseContextMenu,
    RightContextMenuCopy,
    RightContextMenuCut,
    RightContextMenuPaste,
    RightContextMenuDelete,
    RightEditorWheelScrolled { delta: mouse::ScrollDelta, viewport_height: f32 },
    RightScrollbarChanged { top_line: f32, viewport_height: f32 },

    Compare,
    CompareFinished(Result<Vec<JsonDiffEntry>, String>),
    FormatLeft,
    FormatLeftFinished(Result<String, String>),
    FormatRight,
    FormatRightFinished(Result<String, String>),
    FormatBoth,
    FormatBothFinished(Result<(String, String), String>),
    Swap,
    ClearLeft,
    ClearRight,
    CopyLeft,
    CopyRight,
    ClearNotification,
    InsertLeft(String),
    InsertRight(String),
    InsertPair(String, String),
}

pub fn update(app: &mut App, message: JsonDiffToolMessage) -> Task<Message> {
    match message {
        JsonDiffToolMessage::LeftEditorAction(action) => {
            close_left_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_left_scroll_lines(app, *lines);
            }
            app.json_diff_left_editor.perform(action);
            Task::none()
        }
        JsonDiffToolMessage::RightEditorAction(action) => {
            close_right_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_right_scroll_lines(app, *lines);
            }
            app.json_diff_right_editor.perform(action);
            Task::none()
        }

        JsonDiffToolMessage::LeftOpenContextMenu { x, y } => {
            close_all_context_menus(app);
            app.json_diff_left_context_menu_open = true;
            app.json_diff_left_context_menu_pos = Some((x, y));
            Task::none()
        }
        JsonDiffToolMessage::LeftCloseContextMenu => {
            close_all_context_menus(app);
            focus_editor_task(&app.json_diff_left_editor_id)
        }
        JsonDiffToolMessage::LeftContextMenuCopy => {
            close_all_context_menus(app);
            let (outcome, task) =
                selection_copy_task(&app.json_diff_left_editor, &app.json_diff_left_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制左侧选中内容");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonDiffToolMessage::LeftContextMenuCut => {
            close_all_context_menus(app);
            let (outcome, task) =
                selection_cut_task(&mut app.json_diff_left_editor, &app.json_diff_left_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切左侧选中内容");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonDiffToolMessage::LeftContextMenuPaste => {
            close_all_context_menus(app);
            paste_task(&app.json_diff_left_editor_id, |content| {
                Message::JsonDiffTool(JsonDiffToolMessage::LeftEditorAction(paste_action(content)))
            })
        }
        JsonDiffToolMessage::LeftContextMenuDelete => {
            close_all_context_menus(app);
            let (_, task) = selection_delete_task(
                &mut app.json_diff_left_editor,
                &app.json_diff_left_editor_id,
            );
            task
        }
        JsonDiffToolMessage::LeftEditorWheelScrolled { delta, viewport_height } => {
            close_left_context_menu(app);
            app.json_diff_left_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.json_diff_left_scroll_remainder += delta_lines;

            let whole_lines = if app.json_diff_left_scroll_remainder >= 0.0 {
                app.json_diff_left_scroll_remainder.floor() as i32
            } else {
                app.json_diff_left_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.json_diff_left_scroll_remainder -= whole_lines as f32;
                apply_left_scroll_lines(app, whole_lines);
                app.json_diff_left_editor
                    .perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        JsonDiffToolMessage::LeftScrollbarChanged { top_line, viewport_height } => {
            close_left_context_menu(app);
            app.json_diff_left_viewport_height = viewport_height.max(0.0);

            let max_scroll = left_max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.json_diff_left_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_left_scroll_lines(app, delta);
                app.json_diff_left_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }

        JsonDiffToolMessage::RightOpenContextMenu { x, y } => {
            close_all_context_menus(app);
            app.json_diff_right_context_menu_open = true;
            app.json_diff_right_context_menu_pos = Some((x, y));
            Task::none()
        }
        JsonDiffToolMessage::RightCloseContextMenu => {
            close_all_context_menus(app);
            focus_editor_task(&app.json_diff_right_editor_id)
        }
        JsonDiffToolMessage::RightContextMenuCopy => {
            close_all_context_menus(app);
            let (outcome, task) =
                selection_copy_task(&app.json_diff_right_editor, &app.json_diff_right_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制右侧选中内容");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonDiffToolMessage::RightContextMenuCut => {
            close_all_context_menus(app);
            let (outcome, task) =
                selection_cut_task(&mut app.json_diff_right_editor, &app.json_diff_right_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切右侧选中内容");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonDiffToolMessage::RightContextMenuPaste => {
            close_all_context_menus(app);
            paste_task(&app.json_diff_right_editor_id, |content| {
                Message::JsonDiffTool(JsonDiffToolMessage::RightEditorAction(paste_action(content)))
            })
        }
        JsonDiffToolMessage::RightContextMenuDelete => {
            close_all_context_menus(app);
            let (_, task) = selection_delete_task(
                &mut app.json_diff_right_editor,
                &app.json_diff_right_editor_id,
            );
            task
        }
        JsonDiffToolMessage::RightEditorWheelScrolled { delta, viewport_height } => {
            close_right_context_menu(app);
            app.json_diff_right_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.json_diff_right_scroll_remainder += delta_lines;

            let whole_lines = if app.json_diff_right_scroll_remainder >= 0.0 {
                app.json_diff_right_scroll_remainder.floor() as i32
            } else {
                app.json_diff_right_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.json_diff_right_scroll_remainder -= whole_lines as f32;
                apply_right_scroll_lines(app, whole_lines);
                app.json_diff_right_editor
                    .perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        JsonDiffToolMessage::RightScrollbarChanged { top_line, viewport_height } => {
            close_right_context_menu(app);
            app.json_diff_right_viewport_height = viewport_height.max(0.0);

            let max_scroll = right_max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.json_diff_right_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_right_scroll_lines(app, delta);
                app.json_diff_right_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }

        JsonDiffToolMessage::Compare => {
            app.json_diff_loading = true;
            close_all_context_menus(app);

            let left = app.json_diff_left_editor.text();
            let right = app.json_diff_right_editor.text();

            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        Some(compare_json_documents(&left, &right))
                    })
                    .await
                    .unwrap_or_else(|| Err("JSON 对比任务执行失败".to_string()))
                },
                |result| Message::JsonDiffTool(JsonDiffToolMessage::CompareFinished(result)),
            )
        }
        JsonDiffToolMessage::CompareFinished(result) => {
            app.json_diff_loading = false;
            close_all_context_menus(app);

            match result {
                Ok(diffs) => {
                    let count = diffs.len();
                    app.json_diff_results = diffs;
                    let message = if count == 0 {
                        "结构一致，未发现差异".to_string()
                    } else {
                        format!("共发现 {count} 处差异")
                    };
                    notify_success(app, &message);
                }
                Err(error) => {
                    app.json_diff_results.clear();
                    notify_error(app, &error);
                }
            }

            clear_notification_task()
        }

        JsonDiffToolMessage::FormatLeft => {
            app.json_diff_loading = true;
            close_all_context_menus(app);
            let text = app.json_diff_left_editor.text();

            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || Some(prettify_json_text(&text)))
                        .await
                        .unwrap_or_else(|| Err("左侧 JSON 格式化失败".to_string()))
                },
                |result| Message::JsonDiffTool(JsonDiffToolMessage::FormatLeftFinished(result)),
            )
        }
        JsonDiffToolMessage::FormatLeftFinished(result) => {
            app.json_diff_loading = false;
            close_left_context_menu(app);

            match result {
                Ok(content) => {
                    app.json_diff_left_editor = text_editor::Content::with_text(&content);
                    reset_left_scroll(app);
                    notify_success(app, "左侧已格式化");
                }
                Err(error) => notify_error(app, &error),
            }

            clear_notification_task()
        }

        JsonDiffToolMessage::FormatRight => {
            app.json_diff_loading = true;
            close_all_context_menus(app);
            let text = app.json_diff_right_editor.text();

            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || Some(prettify_json_text(&text)))
                        .await
                        .unwrap_or_else(|| Err("右侧 JSON 格式化失败".to_string()))
                },
                |result| Message::JsonDiffTool(JsonDiffToolMessage::FormatRightFinished(result)),
            )
        }
        JsonDiffToolMessage::FormatRightFinished(result) => {
            app.json_diff_loading = false;
            close_right_context_menu(app);

            match result {
                Ok(content) => {
                    app.json_diff_right_editor = text_editor::Content::with_text(&content);
                    reset_right_scroll(app);
                    notify_success(app, "右侧已格式化");
                }
                Err(error) => notify_error(app, &error),
            }

            clear_notification_task()
        }

        JsonDiffToolMessage::FormatBoth => {
            app.json_diff_loading = true;
            close_all_context_menus(app);
            let left = app.json_diff_left_editor.text();
            let right = app.json_diff_right_editor.text();

            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        let left = match prettify_json_text(&left) {
                            Ok(content) => content,
                            Err(error) => return Some(Err(format!("左侧 {error}"))),
                        };
                        let right = match prettify_json_text(&right) {
                            Ok(content) => content,
                            Err(error) => return Some(Err(format!("右侧 {error}"))),
                        };
                        Some(Ok((left, right)))
                    })
                    .await
                    .unwrap_or_else(|| Err("双侧 JSON 格式化失败".to_string()))
                },
                |result| Message::JsonDiffTool(JsonDiffToolMessage::FormatBothFinished(result)),
            )
        }
        JsonDiffToolMessage::FormatBothFinished(result) => {
            app.json_diff_loading = false;
            close_all_context_menus(app);

            match result {
                Ok((left, right)) => {
                    app.json_diff_left_editor = text_editor::Content::with_text(&left);
                    app.json_diff_right_editor = text_editor::Content::with_text(&right);
                    reset_left_scroll(app);
                    reset_right_scroll(app);
                    notify_success(app, "左右两侧已格式化");
                }
                Err(error) => notify_error(app, &error),
            }

            clear_notification_task()
        }

        JsonDiffToolMessage::Swap => {
            let left = app.json_diff_left_editor.text();
            let right = app.json_diff_right_editor.text();
            app.json_diff_left_editor = text_editor::Content::with_text(&right);
            app.json_diff_right_editor = text_editor::Content::with_text(&left);
            reset_left_scroll(app);
            reset_right_scroll(app);
            close_all_context_menus(app);
            notify_success(app, "已交换左右内容");
            clear_notification_task()
        }
        JsonDiffToolMessage::ClearLeft => {
            app.json_diff_left_editor = text_editor::Content::new();
            reset_left_scroll(app);
            close_left_context_menu(app);
            notify_success(app, "已清空左侧");
            clear_notification_task()
        }
        JsonDiffToolMessage::ClearRight => {
            app.json_diff_right_editor = text_editor::Content::new();
            reset_right_scroll(app);
            close_right_context_menu(app);
            notify_success(app, "已清空右侧");
            clear_notification_task()
        }
        JsonDiffToolMessage::CopyLeft => {
            let text = app.json_diff_left_editor.text();
            close_left_context_menu(app);
            notify_success(app, "已复制左侧");
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        JsonDiffToolMessage::CopyRight => {
            let text = app.json_diff_right_editor.text();
            close_right_context_menu(app);
            notify_success(app, "已复制右侧");
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        JsonDiffToolMessage::ClearNotification => {
            app.json_diff_notification = None;
            app.json_diff_notification_is_error = false;
            Task::none()
        }
        JsonDiffToolMessage::InsertLeft(content) => {
            app.json_diff_left_editor = text_editor::Content::with_text(&content);
            reset_left_scroll(app);
            notify_success(app, "已填充左侧示例");
            clear_notification_task()
        }
        JsonDiffToolMessage::InsertRight(content) => {
            app.json_diff_right_editor = text_editor::Content::with_text(&content);
            reset_right_scroll(app);
            notify_success(app, "已填充右侧示例");
            clear_notification_task()
        }
        JsonDiffToolMessage::InsertPair(left, right) => {
            app.json_diff_left_editor = text_editor::Content::with_text(&left);
            app.json_diff_right_editor = text_editor::Content::with_text(&right);
            reset_left_scroll(app);
            reset_right_scroll(app);
            notify_success(app, "已填充左右示例");
            clear_notification_task()
        }
    }
}

fn compare_json_documents(left: &str, right: &str) -> Result<Vec<JsonDiffEntry>, String> {
    let left_value = serde_json::from_str::<serde_json::Value>(left)
        .map_err(|error| format_json_error("左侧 JSON 解析失败", &error))?;
    let right_value = serde_json::from_str::<serde_json::Value>(right)
        .map_err(|error| format_json_error("右侧 JSON 解析失败", &error))?;

    let mut diffs = Vec::new();
    diff_values(&left_value, &right_value, "", &mut diffs);
    Ok(diffs)
}

fn prettify_json_text(input: &str) -> Result<String, String> {
    let value = serde_json::from_str::<serde_json::Value>(input)
        .map_err(|error| format_json_error("JSON 解析失败", &error))?;
    serde_json::to_string_pretty(&value).map_err(|_| "JSON 格式化失败".to_string())
}

fn format_json_error(prefix: &str, error: &serde_json::Error) -> String {
    format!("{prefix}（第 {} 行，第 {} 列）", error.line(), error.column())
}

fn stringify(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
        }
        _ => v.to_string(),
    }
}

fn diff_values(
    left: &serde_json::Value,
    right: &serde_json::Value,
    path: &str,
    out: &mut Vec<JsonDiffEntry>,
) {
    match (left, right) {
        (serde_json::Value::Object(lo), serde_json::Value::Object(ro)) => {
            use std::collections::BTreeSet;

            let mut keys = BTreeSet::new();
            for key in lo.keys() {
                keys.insert(key.clone());
            }
            for key in ro.keys() {
                keys.insert(key.clone());
            }

            for key in keys {
                let left_value = lo.get(&key);
                let right_value = ro.get(&key);
                let next_path = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };

                match (left_value, right_value) {
                    (Some(left_value), Some(right_value)) => {
                        diff_values(left_value, right_value, &next_path, out);
                    }
                    (Some(left_value), None) => {
                        out.push(JsonDiffEntry::new(next_path, Some(stringify(left_value)), None))
                    }
                    (None, Some(right_value)) => {
                        out.push(JsonDiffEntry::new(next_path, None, Some(stringify(right_value))))
                    }
                    (None, None) => {}
                }
            }
        }
        (serde_json::Value::Array(left_items), serde_json::Value::Array(right_items)) => {
            let max_len = left_items.len().max(right_items.len());
            for index in 0..max_len {
                let next_path =
                    if path.is_empty() { format!("[{index}]") } else { format!("{path}[{index}]") };

                match (left_items.get(index), right_items.get(index)) {
                    (Some(left_value), Some(right_value)) => {
                        diff_values(left_value, right_value, &next_path, out);
                    }
                    (Some(left_value), None) => {
                        out.push(JsonDiffEntry::new(next_path, Some(stringify(left_value)), None))
                    }
                    (None, Some(right_value)) => {
                        out.push(JsonDiffEntry::new(next_path, None, Some(stringify(right_value))))
                    }
                    (None, None) => {}
                }
            }
        }
        _ => {
            if left != right {
                out.push(JsonDiffEntry::new(
                    path.to_string(),
                    Some(stringify(left)),
                    Some(stringify(right)),
                ));
            }
        }
    }
}

fn left_visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.json_diff_left_viewport_height / line_height).floor().max(1.0)
}

fn right_visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.json_diff_right_viewport_height / line_height).floor().max(1.0)
}

fn left_max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.json_diff_left_editor.line_count().max(1) as f32;
    (total_lines - left_visible_line_count(app)).max(0.0)
}

fn right_max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.json_diff_right_editor.line_count().max(1) as f32;
    (total_lines - right_visible_line_count(app)).max(0.0)
}

fn apply_left_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = left_max_scroll_top_line(app);
    app.json_diff_left_scroll_top_line =
        (app.json_diff_left_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn apply_right_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = right_max_scroll_top_line(app);
    app.json_diff_right_scroll_top_line =
        (app.json_diff_right_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn reset_left_scroll(app: &mut App) {
    app.json_diff_left_scroll_top_line = 0.0;
    app.json_diff_left_scroll_remainder = 0.0;
}

fn reset_right_scroll(app: &mut App) {
    app.json_diff_right_scroll_top_line = 0.0;
    app.json_diff_right_scroll_remainder = 0.0;
}

fn close_left_context_menu(app: &mut App) {
    app.json_diff_left_context_menu_open = false;
    app.json_diff_left_context_menu_pos = None;
}

fn close_right_context_menu(app: &mut App) {
    app.json_diff_right_context_menu_open = false;
    app.json_diff_right_context_menu_pos = None;
}

fn close_all_context_menus(app: &mut App) {
    close_left_context_menu(app);
    close_right_context_menu(app);
}

fn notify_success(app: &mut App, message: &str) {
    app.json_diff_notification = Some(message.to_string());
    app.json_diff_notification_is_error = false;
}

fn notify_error(app: &mut App, message: &str) {
    app.json_diff_notification = Some(message.to_string());
    app.json_diff_notification_is_error = true;
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::JsonDiffTool(JsonDiffToolMessage::ClearNotification),
    )
}

#[cfg(test)]
#[path = "json_diff_tool_tests.rs"]
mod tests;
