//! JSON/YAML 互转工具消息处理模块
//!
//! 本模块提供 JSON 与 YAML 格式之间相互转换的消息处理逻辑。
//! 主要功能包括：
//!
//! - JSON 到 YAML 的格式转换
//! - YAML 到 JSON 的格式转换
//! - 双编辑器的内容交换、清空和复制操作
//! - 左右编辑器的右键菜单（复制、剪切、粘贴、删除）
//! - 自定义滚轮滚动与独立滚动条
//! - 异步转换任务的调度与结果处理
//!
//! 该模块与 UI 层的文本编辑器紧密配合，通过 Iced 框架的消息机制驱动界面更新。

use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum JsonYamlToolMessage {
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

    YamlToJson,
    JsonToYaml,
    Swap,
    ClearLeft,
    ClearRight,
    CopyLeft,
    CopyRight,
    OutputUpdated(Option<String>),
    ClearNotification,
}

pub fn update(app: &mut App, message: JsonYamlToolMessage) -> Task<Message> {
    match message {
        JsonYamlToolMessage::LeftEditorAction(action) => {
            close_left_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_left_scroll_lines(app, *lines);
            }
            app.json_yaml_left_editor.perform(action);
            Task::none()
        }
        JsonYamlToolMessage::RightEditorAction(action) => {
            close_right_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_right_scroll_lines(app, *lines);
            }
            app.json_yaml_right_editor.perform(action);
            Task::none()
        }

        JsonYamlToolMessage::LeftOpenContextMenu { x, y } => {
            app.json_yaml_left_context_menu_open = true;
            app.json_yaml_left_context_menu_pos = Some((x, y));
            Task::none()
        }
        JsonYamlToolMessage::LeftCloseContextMenu => {
            close_left_context_menu(app);
            focus_editor_task(&app.json_yaml_left_editor_id)
        }
        JsonYamlToolMessage::LeftContextMenuCopy => {
            close_left_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.json_yaml_left_editor, &app.json_yaml_left_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonYamlToolMessage::LeftContextMenuCut => {
            close_left_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.json_yaml_left_editor, &app.json_yaml_left_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonYamlToolMessage::LeftContextMenuPaste => {
            close_left_context_menu(app);
            paste_task(&app.json_yaml_left_editor_id, |content| {
                Message::JsonYamlTool(JsonYamlToolMessage::LeftEditorAction(paste_action(content)))
            })
        }
        JsonYamlToolMessage::LeftContextMenuDelete => {
            close_left_context_menu(app);
            let (_, task) = selection_delete_task(
                &mut app.json_yaml_left_editor,
                &app.json_yaml_left_editor_id,
            );
            task
        }
        JsonYamlToolMessage::LeftEditorWheelScrolled { delta, viewport_height } => {
            close_left_context_menu(app);
            app.json_yaml_left_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.json_yaml_left_scroll_remainder += delta_lines;

            let whole_lines = if app.json_yaml_left_scroll_remainder >= 0.0 {
                app.json_yaml_left_scroll_remainder.floor() as i32
            } else {
                app.json_yaml_left_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.json_yaml_left_scroll_remainder -= whole_lines as f32;
                apply_left_scroll_lines(app, whole_lines);
                app.json_yaml_left_editor
                    .perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        JsonYamlToolMessage::LeftScrollbarChanged { top_line, viewport_height } => {
            close_left_context_menu(app);
            app.json_yaml_left_viewport_height = viewport_height.max(0.0);

            let max_scroll = left_max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.json_yaml_left_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_left_scroll_lines(app, delta);
                app.json_yaml_left_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }

        JsonYamlToolMessage::RightOpenContextMenu { x, y } => {
            app.json_yaml_right_context_menu_open = true;
            app.json_yaml_right_context_menu_pos = Some((x, y));
            Task::none()
        }
        JsonYamlToolMessage::RightCloseContextMenu => {
            close_right_context_menu(app);
            focus_editor_task(&app.json_yaml_right_editor_id)
        }
        JsonYamlToolMessage::RightContextMenuCopy => {
            close_right_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.json_yaml_right_editor, &app.json_yaml_right_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonYamlToolMessage::RightContextMenuCut => {
            close_right_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.json_yaml_right_editor, &app.json_yaml_right_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonYamlToolMessage::RightContextMenuPaste => {
            close_right_context_menu(app);
            paste_task(&app.json_yaml_right_editor_id, |content| {
                Message::JsonYamlTool(JsonYamlToolMessage::RightEditorAction(paste_action(content)))
            })
        }
        JsonYamlToolMessage::RightContextMenuDelete => {
            close_right_context_menu(app);
            let (_, task) = selection_delete_task(
                &mut app.json_yaml_right_editor,
                &app.json_yaml_right_editor_id,
            );
            task
        }
        JsonYamlToolMessage::RightEditorWheelScrolled { delta, viewport_height } => {
            close_right_context_menu(app);
            app.json_yaml_right_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.json_yaml_right_scroll_remainder += delta_lines;

            let whole_lines = if app.json_yaml_right_scroll_remainder >= 0.0 {
                app.json_yaml_right_scroll_remainder.floor() as i32
            } else {
                app.json_yaml_right_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.json_yaml_right_scroll_remainder -= whole_lines as f32;
                apply_right_scroll_lines(app, whole_lines);
                app.json_yaml_right_editor
                    .perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        JsonYamlToolMessage::RightScrollbarChanged { top_line, viewport_height } => {
            close_right_context_menu(app);
            app.json_yaml_right_viewport_height = viewport_height.max(0.0);

            let max_scroll = right_max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.json_yaml_right_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_right_scroll_lines(app, delta);
                app.json_yaml_right_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }

        JsonYamlToolMessage::Swap => {
            let left = app.json_yaml_left_editor.text();
            let right = app.json_yaml_right_editor.text();
            app.json_yaml_left_editor = text_editor::Content::with_text(&right);
            app.json_yaml_right_editor = text_editor::Content::with_text(&left);
            app.json_yaml_left_scroll_top_line = 0.0;
            app.json_yaml_left_scroll_remainder = 0.0;
            app.json_yaml_right_scroll_top_line = 0.0;
            app.json_yaml_right_scroll_remainder = 0.0;
            close_left_context_menu(app);
            close_right_context_menu(app);
            Task::none()
        }
        JsonYamlToolMessage::ClearLeft => {
            app.json_yaml_left_editor = text_editor::Content::new();
            app.json_yaml_left_scroll_top_line = 0.0;
            app.json_yaml_left_scroll_remainder = 0.0;
            close_left_context_menu(app);
            Task::none()
        }
        JsonYamlToolMessage::ClearRight => {
            app.json_yaml_right_editor = text_editor::Content::new();
            app.json_yaml_right_scroll_top_line = 0.0;
            app.json_yaml_right_scroll_remainder = 0.0;
            close_right_context_menu(app);
            Task::none()
        }
        JsonYamlToolMessage::CopyLeft => {
            let text = app.json_yaml_left_editor.text();
            notify_success(app, "已复制左侧");
            close_left_context_menu(app);
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        JsonYamlToolMessage::CopyRight => {
            let text = app.json_yaml_right_editor.text();
            notify_success(app, "已复制右侧");
            close_right_context_menu(app);
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        JsonYamlToolMessage::ClearNotification => {
            app.json_yaml_notification = None;
            Task::none()
        }
        JsonYamlToolMessage::OutputUpdated(Some(content)) => {
            app.json_yaml_loading = false;
            app.json_yaml_right_editor = text_editor::Content::with_text(&content);
            app.json_yaml_right_scroll_top_line = 0.0;
            app.json_yaml_right_scroll_remainder = 0.0;
            close_right_context_menu(app);
            notify_success(app, "转换成功");
            clear_notification_task()
        }
        JsonYamlToolMessage::OutputUpdated(None) => {
            app.json_yaml_loading = false;
            close_right_context_menu(app);
            notify_success(app, "转换失败或格式错误");
            clear_notification_task()
        }
        JsonYamlToolMessage::YamlToJson => {
            app.json_yaml_loading = true;
            close_left_context_menu(app);
            close_right_context_menu(app);
            let input = app.json_yaml_left_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || yaml_to_json_pretty(&input))
                        .await
                },
                |res| Message::JsonYamlTool(JsonYamlToolMessage::OutputUpdated(res)),
            )
        }
        JsonYamlToolMessage::JsonToYaml => {
            app.json_yaml_loading = true;
            close_left_context_menu(app);
            close_right_context_menu(app);
            let input = app.json_yaml_left_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || json_to_yaml(&input)).await
                },
                |res| Message::JsonYamlTool(JsonYamlToolMessage::OutputUpdated(res)),
            )
        }
    }
}

fn left_visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.json_yaml_left_viewport_height / line_height).floor().max(1.0)
}

fn right_visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.json_yaml_right_viewport_height / line_height).floor().max(1.0)
}

fn close_left_context_menu(app: &mut App) {
    app.json_yaml_left_context_menu_open = false;
    app.json_yaml_left_context_menu_pos = None;
}

fn close_right_context_menu(app: &mut App) {
    app.json_yaml_right_context_menu_open = false;
    app.json_yaml_right_context_menu_pos = None;
}

fn notify_success(app: &mut App, message: &str) {
    app.json_yaml_notification = Some(message.to_string());
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::JsonYamlTool(JsonYamlToolMessage::ClearNotification),
    )
}

fn left_max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.json_yaml_left_editor.line_count().max(1) as f32;
    (total_lines - left_visible_line_count(app)).max(0.0)
}

fn right_max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.json_yaml_right_editor.line_count().max(1) as f32;
    (total_lines - right_visible_line_count(app)).max(0.0)
}

fn apply_left_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }
    let max_scroll = left_max_scroll_top_line(app);
    app.json_yaml_left_scroll_top_line =
        (app.json_yaml_left_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn apply_right_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }
    let max_scroll = right_max_scroll_top_line(app);
    app.json_yaml_right_scroll_top_line =
        (app.json_yaml_right_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn json_to_yaml(input: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(input).ok()?;
    let yaml = serde_yaml::to_string(&value).ok()?;
    Some(strip_yaml_doc_marker(&yaml))
}

fn yaml_to_json_pretty(input: &str) -> Option<String> {
    let value = serde_yaml::from_str::<serde_yaml::Value>(input).ok()?;
    let json = yaml_value_to_json(&value)?;
    serde_json::to_string_pretty(&json).ok()
}

fn strip_yaml_doc_marker(s: &str) -> String {
    let without_doc = s.strip_prefix("---\n").or_else(|| s.strip_prefix("---\r\n")).unwrap_or(s);
    without_doc.trim_end().to_string()
}

fn yaml_value_to_json(v: &serde_yaml::Value) -> Option<serde_json::Value> {
    match v {
        serde_yaml::Value::Null => Some(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Some(serde_json::Value::Bool(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(serde_json::Value::Number(i.into()))
            } else if let Some(u) = n.as_u64() {
                Some(serde_json::Value::Number(u.into()))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f).map(serde_json::Value::Number)
            } else {
                None
            }
        }
        serde_yaml::Value::String(s) => Some(serde_json::Value::String(s.clone())),
        serde_yaml::Value::Sequence(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(yaml_value_to_json(it)?);
            }
            Some(serde_json::Value::Array(out))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map.iter() {
                let key = yaml_key_to_string(k);
                out.insert(key, yaml_value_to_json(v)?);
            }
            Some(serde_json::Value::Object(out))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_json(&tagged.value),
    }
}

fn yaml_key_to_string(k: &serde_yaml::Value) -> String {
    match k {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(_)
        | serde_yaml::Value::Mapping(_)
        | serde_yaml::Value::Tagged(_) => {
            let s = serde_yaml::to_string(k).unwrap_or_default();
            strip_yaml_doc_marker(&s)
        }
    }
}
