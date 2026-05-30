//! JSON 工具模块
//!
//! 本模块提供 JSON 数据处理工具的消息处理功能，包括格式化、压缩、转义、
//! Unicode 转换等操作。所有操作均通过消息驱动的异步方式执行，避免阻塞 UI。
//!
//! # 主要功能
//!
//! - **格式化**：将 JSON 字符串转换为美观的缩进格式
//! - **压缩**：移除 JSON 中多余的空白字符
//! - **转义/反转义**：处理 JSON 字符串中的特殊字符
//! - **Unicode 转换**：中文与 Unicode 编码之间的相互转换
//! - **URL 参数转换**：将 JSON 对象转换为 GET 请求参数格式
//! - **内容持久化**：可选地保存编辑器内容到配置文件

use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::config::save_json_tool_content;
#[cfg(target_arch = "wasm32")]
use crate::app::config::save_json_tool_content_async;
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;
use std::fmt::Write;

/// JSON 工具消息枚举
///
/// 定义了 JSON 工具支持的所有用户操作和内部事件。
/// 这些消息用于驱动 JSON 工具的状态更新和异步操作。
#[derive(Debug, Clone)]
pub enum JsonToolMessage {
    /// 编辑器操作事件
    ///
    /// 由文本编辑器组件产生的操作，如光标移动、文本输入等
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

    /// 格式化 JSON
    ///
    /// 将当前编辑器中的 JSON 字符串转换为带缩进的美观格式
    Format,

    /// 清空编辑器
    ///
    /// 清除编辑器中的所有内容，并可选地清除持久化存储
    Clear,

    /// 压缩 JSON
    ///
    /// 移除 JSON 字符串中的所有空白字符，生成紧凑格式
    Compress,

    /// 转义字符串
    ///
    /// 对文本进行 JSON 字符串转义处理，添加必要的转义字符
    Escape,

    /// 反转义字符串
    ///
    /// 将已转义的 JSON 字符串还原为原始文本
    Unescape,

    /// Unicode 转中文
    ///
    /// 将文本中的 Unicode 编码（如 `\u4e2d`）转换为对应的中文字符
    UnicodeToCn,

    /// 中文转 Unicode
    ///
    /// 将文本中的中文字符转换为 Unicode 编码格式
    CnToUnicode,

    /// 转换为 GET 参数
    ///
    /// 将 JSON 对象转换为 URL 查询字符串格式（key=value&...）
    ToGet,

    /// 复制到剪贴板
    ///
    /// 将当前编辑器内容复制到系统剪贴板
    Copy,

    /// 切换记忆开关
    ///
    /// 控制是否自动保存编辑器内容到持久化存储
    ToggleRemember(bool),

    /// 内容更新完成
    ///
    /// 异步操作完成后的回调，携带处理结果
    /// - `Some(content)`：操作成功，包含新内容
    /// - `None`：操作失败或格式错误
    ContentUpdated(Option<String>),

    /// 清除通知消息
    ///
    /// 移除当前显示的操作结果通知
    ClearNotification,
}

/// URL 编码函数
///
/// 将字符串按照 URL 编码规范进行编码，保留安全字符，
/// 其他字符转换为 `%XX` 格式。
///
/// # 参数
///
/// - `s`：待编码的字符串
///
/// # 返回值
///
/// 返回 URL 编码后的字符串
///
/// # 安全字符
///
/// 以下字符不进行编码：
/// - 字母：`a-z`、`A-Z`
/// - 数字：`0-9`
/// - 特殊字符：`-`、`_`、`.`、`~`
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match *b as char {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(*b as char),
            _ => write!(result, "%{:02X}", b).unwrap(),
        }
    }
    result
}

/// 中文转 Unicode 编码
///
/// 将字符串中的非 ASCII 字符（主要是中文）转换为 Unicode 转义序列格式。
/// ASCII 字符保持不变。
///
/// # 参数
///
/// - `text`：待转换的文本
///
/// # 返回值
///
/// 返回转换后的字符串，其中非 ASCII 字符被替换为 `\uXXXX` 格式
///
/// # 示例
///
/// ```ignore
/// let result = cn_to_unicode("你好");
/// // 返回: "\\u4f60\\u597d"
/// ```
fn cn_to_unicode(text: &str) -> String {
    let mut output = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if c.is_ascii() {
            output.push(c);
        } else {
            // 非 ASCII 字符转换为 \uXXXX 格式
            write!(output, "\\u{:04x}", c as u32).unwrap();
        }
    }
    output
}

/// Unicode 转义序列转中文
///
/// 将字符串中的 Unicode 转义序列（`\uXXXX`）转换为对应的字符。
/// 对于无效的转义序列，保留原始文本。
///
/// # 参数
///
/// - `text`：包含 Unicode 转义序列的文本
///
/// # 返回值
///
/// 返回转换后的字符串，其中有效的 Unicode 转义序列被替换为对应字符
///
/// # 示例
///
/// ```ignore
/// let result = unicode_to_cn("\\u4f60\\u597d");
/// // 返回: "你好"
/// ```
///
/// # 错误处理
///
/// - 如果 `\u` 后不足 4 个十六进制数字，保留原始字符序列
/// - 如果十六进制数字不是有效的 Unicode 码点，保留转义序列
fn unicode_to_cn(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'u') {
            // 跳过 'u' 字符
            chars.next();

            // 尝试读取 4 个十六进制数字
            let mut hex_digits = String::new();
            for _ in 0..4 {
                if let Some(hex_char) = chars.next() {
                    if hex_char.is_ascii_hexdigit() {
                        hex_digits.push(hex_char);
                    } else {
                        // 无效的十六进制数字，按原样输出
                        output.push('\\');
                        output.push('u');
                        output.push_str(&hex_digits);
                        output.push(hex_char);
                        break;
                    }
                } else {
                    // 十六进制数字不足，按原样输出
                    output.push('\\');
                    output.push('u');
                    output.push_str(&hex_digits);
                    break;
                }
            }

            // 成功读取 4 个十六进制数字，尝试转换为字符
            if hex_digits.len() == 4 {
                if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        output.push(unicode_char);
                    } else {
                        // 无效的 Unicode 码点，保留转义序列
                        output.push_str(&format!("\\u{}", hex_digits));
                    }
                } else {
                    // 无效的十六进制数字，保留转义序列
                    output.push_str(&format!("\\u{}", hex_digits));
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

/// JSON 工具消息处理函数
///
/// 根据接收到的消息类型，执行相应的操作并更新应用状态。
/// 所有耗时操作均在异步任务中执行，避免阻塞 UI。
///
/// # 参数
///
/// - `app`：应用状态的可变引用
/// - `message`：待处理的消息
///
/// # 返回值
///
/// 返回一个或多个 `Task`，用于执行异步操作或更新 UI
///
/// # 消息处理
///
/// - `ClearNotification`：立即清除通知，不产生额外任务
/// - `EditorAction`：直接操作编辑器，不产生异步任务
/// - `ContentUpdated`：更新编辑器内容和状态，设置自动清除通知的定时器
/// - `Format`/`Compress`：启动异步 JSON 处理任务
/// - `Escape`/`Unescape`：启动异步字符串转义/反转义任务
/// - `UnicodeToCn`/`CnToUnicode`：启动异步编码转换任务
/// - `ToGet`：启动异步 JSON 到 URL 参数转换任务
/// - `Copy`：复制内容到剪贴板并显示通知
/// - `ToggleRemember`：切换持久化开关并保存配置
pub fn update(app: &mut App, message: JsonToolMessage) -> Task<Message> {
    match message {
        JsonToolMessage::ClearNotification => {
            app.json_tool_notification = None;
            Task::none()
        }
        JsonToolMessage::OpenContextMenu { x, y } => {
            app.json_tool_context_menu_open = true;
            app.json_tool_context_menu_pos = Some((x, y));
            Task::none()
        }
        JsonToolMessage::CloseContextMenu => {
            close_context_menu(app);
            focus_editor_task(&app.json_tool_editor_id)
        }
        JsonToolMessage::ContextMenuCopy => {
            close_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.json_tool_editor, &app.json_tool_editor_id);

            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonToolMessage::ContextMenuCut => {
            close_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.json_tool_editor, &app.json_tool_editor_id);

            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        JsonToolMessage::ContextMenuPaste => {
            close_context_menu(app);
            paste_task(&app.json_tool_editor_id, |content| {
                Message::JsonTool(JsonToolMessage::EditorAction(paste_action(content)))
            })
        }
        JsonToolMessage::ContextMenuDelete => {
            close_context_menu(app);
            let (_outcome, task) =
                selection_delete_task(&mut app.json_tool_editor, &app.json_tool_editor_id);
            task
        }
        JsonToolMessage::EditorAction(action) => {
            close_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            app.json_tool_editor.perform(action);
            Task::none()
        }
        JsonToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            close_context_menu(app);
            app.json_tool_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.json_tool_scroll_remainder += delta_lines;

            let whole_lines = if app.json_tool_scroll_remainder >= 0.0 {
                app.json_tool_scroll_remainder.floor() as i32
            } else {
                app.json_tool_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.json_tool_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.json_tool_editor.perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        JsonToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            close_context_menu(app);
            app.json_tool_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.json_tool_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.json_tool_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        JsonToolMessage::ContentUpdated(Some(content)) => {
            app.json_tool_loading = false;
            notify_success(app, "操作成功");
            app.json_tool_editor = text_editor::Content::with_text(&content);
            app.json_tool_scroll_top_line = 0.0;
            app.json_tool_scroll_remainder = 0.0;
            close_context_menu(app);

            let save_task = if app.json_tool_remember {
                save_json_tool_content_task(content.clone())
            } else {
                Task::none()
            };

            Task::batch(vec![clear_notification_task(), save_task])
        }
        JsonToolMessage::ContentUpdated(None) => {
            app.json_tool_loading = false;
            app.json_tool_notification = Some("操作失败或格式错误".to_string());
            close_context_menu(app);
            clear_notification_task()
        }
        JsonToolMessage::Format => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            serde_json::to_string_pretty(&value).ok()
                        } else {
                            None
                        }
                    })
                    .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::Clear => {
            app.json_tool_editor = text_editor::Content::new();
            app.json_tool_scroll_top_line = 0.0;
            app.json_tool_scroll_remainder = 0.0;
            close_context_menu(app);

            let save_task = if app.json_tool_remember {
                save_json_tool_content_task(String::new())
            } else {
                Task::none()
            };
            notify_success(app, "已清空");
            Task::batch(vec![clear_notification_task(), save_task])
        }
        JsonToolMessage::Compress => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            serde_json::to_string(&value).ok()
                        } else {
                            None
                        }
                    })
                    .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::Escape => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        if let Ok(escaped) = serde_json::to_string(&text) {
                            if escaped.len() >= 2 {
                                Some(escaped[1..escaped.len() - 1].to_string())
                            } else {
                                Some(escaped)
                            }
                        } else {
                            None
                        }
                    })
                    .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::Unescape => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        let to_parse = format!("\"{}\"", text);
                        serde_json::from_str::<String>(&to_parse).ok()
                    })
                    .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::UnicodeToCn => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || Some(unicode_to_cn(&text)))
                        .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::CnToUnicode => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || Some(cn_to_unicode(&text)))
                        .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::ToGet => {
            app.json_tool_loading = true;
            let text = app.json_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
                            && let Some(obj) = value.as_object()
                        {
                            let params: Vec<String> = obj
                                .iter()
                                .map(|(k, v)| {
                                    let v_str = match v {
                                        serde_json::Value::String(s) => s.clone(),
                                        _ => v.to_string(),
                                    };
                                    format!("{}={}", url_encode(k), url_encode(&v_str))
                                })
                                .collect();
                            return Some(params.join("&"));
                        }
                        None
                    })
                    .await
                },
                |res| Message::JsonTool(JsonToolMessage::ContentUpdated(res)),
            )
        }
        JsonToolMessage::Copy => {
            let text = app.json_tool_editor.text();
            notify_success(app, "已复制");
            close_context_menu(app);
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        JsonToolMessage::ToggleRemember(val) => {
            app.json_tool_remember = val;
            close_context_menu(app);
            crate::app::set_config_field("json_tool_remember", serde_json::Value::Bool(val));
            if val {
                return save_json_tool_content_task(app.json_tool_editor.text());
            }
            Task::none()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn save_json_tool_content_task(content: String) -> Task<Message> {
    Task::perform(async move { save_json_tool_content_async(&content).await }, |result| {
        if let Err(error) = result {
            tracing::warn!(target: "vw_desktop", error = %error, "failed to save json tool content");
        }
        Message::None
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn save_json_tool_content_task(content: String) -> Task<Message> {
    save_json_tool_content(&content);
    Task::none()
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.json_tool_viewport_height / line_height).floor().max(1.0)
}

fn close_context_menu(app: &mut App) {
    app.json_tool_context_menu_open = false;
    app.json_tool_context_menu_pos = None;
}

fn notify_success(app: &mut App, message: &str) {
    app.json_tool_notification = Some(message.to_string());
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::JsonTool(JsonToolMessage::ClearNotification),
    )
}

fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.json_tool_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = max_scroll_top_line(app);
    app.json_tool_scroll_top_line =
        (app.json_tool_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}
