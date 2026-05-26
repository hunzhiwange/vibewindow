//! 密码生成工具消息处理模块
//!
//! 本模块提供密码生成器的核心逻辑和消息处理功能，包括：
//! - 安全随机密码生成
//! - 字符集配置（数字、小写、大写、特殊字符）
//! - 批量生成密码
//! - 剪贴板复制
//! - 右键菜单（复制选择、剪切、粘贴、删除）
//! - 编辑器滚动同步

use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;
use rand::{RngCore, rngs::OsRng, seq::SliceRandom};

type Charset = &'static str;

pub(crate) const DIGITS_CHARSET: Charset = "0123456789";
pub(crate) const LOWERCASE_CHARSET: Charset = "abcdefghijklmnopqrstuvwxyz";
pub(crate) const UPPERCASE_CHARSET: Charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub(crate) const SPECIAL_CHARSET: Charset = "~!@#$%^&*()[]{}-_+=:;\"',.<>/?`|\\";

/// 密码生成工具的消息类型
#[derive(Debug, Clone)]
pub enum PasswordToolMessage {
    EditorAction(text_editor::Action),
    OpenContextMenu { x: f32, y: f32 },
    CloseContextMenu,
    ContextMenuCopy,
    ContextMenuCut,
    ContextMenuPaste,
    ContextMenuDelete,
    EditorWheelScrolled { delta: mouse::ScrollDelta, viewport_height: f32 },
    ScrollbarChanged { top_line: f32, viewport_height: f32 },
    ToggleDigits(bool),
    ToggleLowercase(bool),
    ToggleUppercase(bool),
    ToggleSpecial(bool),
    LengthChanged(String),
    CountChanged(String),
    Generate,
    Copy,
    Clear,
    ClearNotification,
}

/// 处理密码工具消息并更新应用状态
pub fn update(app: &mut App, message: PasswordToolMessage) -> Task<Message> {
    match message {
        PasswordToolMessage::ClearNotification => {
            app.pwd_notification = None;
            app.pwd_notification_is_error = false;
            Task::none()
        }
        PasswordToolMessage::OpenContextMenu { x, y } => {
            app.pwd_context_menu_open = true;
            app.pwd_context_menu_pos = Some((x, y));
            Task::none()
        }
        PasswordToolMessage::CloseContextMenu => {
            close_context_menu(app);
            focus_editor_task(&app.pwd_editor_id)
        }
        PasswordToolMessage::ContextMenuCopy => {
            close_context_menu(app);
            let (outcome, task) = selection_copy_task(&app.pwd_output_editor, &app.pwd_editor_id);
            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        PasswordToolMessage::ContextMenuCut => {
            close_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.pwd_output_editor, &app.pwd_editor_id);
            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        PasswordToolMessage::ContextMenuPaste => {
            close_context_menu(app);
            paste_task(&app.pwd_editor_id, |content| {
                Message::PasswordTool(PasswordToolMessage::EditorAction(paste_action(content)))
            })
        }
        PasswordToolMessage::ContextMenuDelete => {
            close_context_menu(app);
            let (_outcome, task) =
                selection_delete_task(&mut app.pwd_output_editor, &app.pwd_editor_id);
            task
        }
        PasswordToolMessage::EditorAction(action) => {
            close_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            app.pwd_output_editor.perform(action);
            Task::none()
        }
        PasswordToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            close_context_menu(app);
            app.pwd_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.pwd_scroll_remainder += delta_lines;

            let whole_lines = if app.pwd_scroll_remainder >= 0.0 {
                app.pwd_scroll_remainder.floor() as i32
            } else {
                app.pwd_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.pwd_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.pwd_output_editor.perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        PasswordToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            close_context_menu(app);
            app.pwd_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.pwd_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.pwd_output_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        PasswordToolMessage::ToggleDigits(enabled) => {
            app.pwd_digits = enabled;
            Task::none()
        }
        PasswordToolMessage::ToggleLowercase(enabled) => {
            app.pwd_lowercase = enabled;
            Task::none()
        }
        PasswordToolMessage::ToggleUppercase(enabled) => {
            app.pwd_uppercase = enabled;
            Task::none()
        }
        PasswordToolMessage::ToggleSpecial(enabled) => {
            app.pwd_special = enabled;
            Task::none()
        }
        PasswordToolMessage::LengthChanged(value) => {
            app.pwd_length_input = value;
            Task::none()
        }
        PasswordToolMessage::CountChanged(value) => {
            app.pwd_count_input = value;
            Task::none()
        }
        PasswordToolMessage::Generate => {
            close_context_menu(app);

            let length = parse_length_input(&app.pwd_length_input);
            let count = parse_count_input(&app.pwd_count_input);
            let charsets = selected_charsets(
                app.pwd_digits,
                app.pwd_lowercase,
                app.pwd_uppercase,
                app.pwd_special,
            );

            if charsets.is_empty() {
                notify_error(app, "至少选择一种字符集");
                return clear_notification_task();
            }

            if length < charsets.len() {
                notify_error(app, &format!("密码长度至少为 {} 位", charsets.len()));
                return clear_notification_task();
            }

            let pool = build_pool(&charsets);
            let mut rng = OsRng;
            let mut lines = Vec::with_capacity(count);

            for _ in 0..count {
                let password = match generate_one(length, &pool, &charsets, &mut rng) {
                    Ok(password) => password,
                    Err(_) => {
                        notify_error(app, "生成失败：安全随机源不可用");
                        return clear_notification_task();
                    }
                };
                lines.push(password);
            }

            app.pwd_output_editor = text_editor::Content::with_text(&lines.join("\n"));
            app.pwd_scroll_top_line = 0.0;
            app.pwd_scroll_remainder = 0.0;
            notify_success(app, "生成成功");
            clear_notification_task()
        }
        PasswordToolMessage::Copy => {
            let text = app.pwd_output_editor.text();
            close_context_menu(app);

            if text.trim().is_empty() {
                notify_error(app, "没有可复制的密码");
                return clear_notification_task();
            }

            notify_success(app, "已复制");
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        PasswordToolMessage::Clear => {
            app.pwd_output_editor = text_editor::Content::new();
            app.pwd_scroll_top_line = 0.0;
            app.pwd_scroll_remainder = 0.0;
            close_context_menu(app);
            notify_success(app, "已清空");
            clear_notification_task()
        }
    }
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.pwd_viewport_height / line_height).floor().max(1.0)
}

fn close_context_menu(app: &mut App) {
    app.pwd_context_menu_open = false;
    app.pwd_context_menu_pos = None;
}

fn notify_success(app: &mut App, message: &str) {
    app.pwd_notification = Some(message.to_string());
    app.pwd_notification_is_error = false;
}

fn notify_error(app: &mut App, message: &str) {
    app.pwd_notification = Some(message.to_string());
    app.pwd_notification_is_error = true;
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::PasswordTool(PasswordToolMessage::ClearNotification),
    )
}

fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.pwd_output_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = max_scroll_top_line(app);
    app.pwd_scroll_top_line = (app.pwd_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

fn selected_charsets(digits: bool, lower: bool, upper: bool, special: bool) -> Vec<Charset> {
    let mut charsets = Vec::new();

    if digits {
        charsets.push(DIGITS_CHARSET);
    }
    if lower {
        charsets.push(LOWERCASE_CHARSET);
    }
    if upper {
        charsets.push(UPPERCASE_CHARSET);
    }
    if special {
        charsets.push(SPECIAL_CHARSET);
    }

    charsets
}

fn build_pool(charsets: &[Charset]) -> Vec<u8> {
    let total_len = charsets.iter().map(|charset| charset.len()).sum();
    let mut pool = Vec::with_capacity(total_len);

    for charset in charsets {
        pool.extend_from_slice(charset.as_bytes());
    }

    pool
}

fn generate_one(
    length: usize,
    pool: &[u8],
    required_charsets: &[Charset],
    rng: &mut impl RngCore,
) -> Result<String, rand::Error> {
    let mut password = Vec::with_capacity(length);

    for charset in required_charsets {
        let index = random_index(rng, charset.len())?;
        password.push(charset.as_bytes()[index]);
    }

    while password.len() < length {
        let index = random_index(rng, pool.len())?;
        password.push(pool[index]);
    }

    password.shuffle(rng);

    Ok(String::from_utf8(password).expect("ASCII charsets must remain valid UTF-8"))
}

fn random_index(rng: &mut impl RngCore, upper_bound: usize) -> Result<usize, rand::Error> {
    if upper_bound <= 1 {
        return Ok(0);
    }

    let upper_bound = u32::try_from(upper_bound).expect("charset length must fit into u32");
    let zone = u32::MAX - (u32::MAX % upper_bound);

    loop {
        let value = next_u32(rng)?;
        if value < zone {
            return Ok((value % upper_bound) as usize);
        }
    }
}

fn next_u32(rng: &mut impl RngCore) -> Result<u32, rand::Error> {
    let mut bytes = [0_u8; 4];
    rng.try_fill_bytes(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn parse_length_input(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(12).max(1)
}

fn parse_count_input(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(1).clamp(1, 500)
}

#[cfg(test)]
#[path = "password_tool_tests.rs"]
mod tests;
