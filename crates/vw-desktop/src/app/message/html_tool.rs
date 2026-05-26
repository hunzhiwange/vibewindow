//! HTML 工具消息处理模块
//!
//! 本模块提供 HTML 代码的格式化和处理功能，包括：
//! - HTML 美化（Beautify）：将 HTML 代码格式化为带缩进的可读形式
//! - HTML 压缩（Compress）：移除多余空白，生成紧凑的 HTML 代码
//! - 内容持久化：支持记忆功能，保存编辑器内容
//!
//! 模块通过 iced 的消息机制与 UI 交互，处理用户的编辑操作和各种工具命令。

use crate::app::components::text_editor_context_menu::{
    SelectionActionOutcome, focus_editor_task, paste_action, paste_task, selection_copy_task,
    selection_cut_task, selection_delete_task,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::config::save_html_tool_content;
#[cfg(target_arch = "wasm32")]
use crate::app::config::save_html_tool_content_async;
use crate::app::{App, Message};
use iced::Task;
use iced::mouse;
use iced::widget::text_editor;
use std::collections::HashSet;
use std::sync::OnceLock;

/// HTML 工具消息枚举
#[derive(Debug, Clone)]
pub enum HtmlToolMessage {
    EditorAction(text_editor::Action),
    OpenContextMenu { x: f32, y: f32 },
    CloseContextMenu,
    ContextMenuCopy,
    ContextMenuCut,
    ContextMenuPaste,
    ContextMenuDelete,
    EditorWheelScrolled { delta: mouse::ScrollDelta, viewport_height: f32 },
    ScrollbarChanged { top_line: f32, viewport_height: f32 },
    Beautify,
    Compress,
    Clear,
    Copy,
    ToggleRemember(bool),
    ContentUpdated(Option<String>),
    ClearNotification,
}

/// 处理 HTML 工具消息
pub fn update(app: &mut App, message: HtmlToolMessage) -> Task<Message> {
    match message {
        HtmlToolMessage::ClearNotification => {
            app.html_tool_notification = None;
            Task::none()
        }
        HtmlToolMessage::OpenContextMenu { x, y } => {
            app.html_tool_context_menu_open = true;
            app.html_tool_context_menu_pos = Some((x, y));
            Task::none()
        }
        HtmlToolMessage::CloseContextMenu => {
            close_context_menu(app);
            focus_editor_task(&app.html_tool_editor_id)
        }
        HtmlToolMessage::ContextMenuCopy => {
            close_context_menu(app);
            let (outcome, task) =
                selection_copy_task(&app.html_tool_editor, &app.html_tool_editor_id);

            if outcome == SelectionActionOutcome::Copied {
                notify_success(app, "已复制");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        HtmlToolMessage::ContextMenuCut => {
            close_context_menu(app);
            let (outcome, task) =
                selection_cut_task(&mut app.html_tool_editor, &app.html_tool_editor_id);

            if outcome == SelectionActionOutcome::Cut {
                notify_success(app, "已剪切");
                Task::batch(vec![task, clear_notification_task()])
            } else {
                task
            }
        }
        HtmlToolMessage::ContextMenuPaste => {
            close_context_menu(app);
            paste_task(&app.html_tool_editor_id, |content| {
                Message::HtmlTool(HtmlToolMessage::EditorAction(paste_action(content)))
            })
        }
        HtmlToolMessage::ContextMenuDelete => {
            close_context_menu(app);
            let (_outcome, task) =
                selection_delete_task(&mut app.html_tool_editor, &app.html_tool_editor_id);
            task
        }
        HtmlToolMessage::EditorAction(action) => {
            close_context_menu(app);
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            app.html_tool_editor.perform(action);
            Task::none()
        }
        HtmlToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            close_context_menu(app);
            app.html_tool_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.html_tool_scroll_remainder += delta_lines;

            let whole_lines = if app.html_tool_scroll_remainder >= 0.0 {
                app.html_tool_scroll_remainder.floor() as i32
            } else {
                app.html_tool_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.html_tool_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.html_tool_editor.perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        HtmlToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            close_context_menu(app);
            app.html_tool_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.html_tool_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.html_tool_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        HtmlToolMessage::ContentUpdated(Some(content)) => {
            app.html_tool_loading = false;
            notify_success(app, "操作成功");
            app.html_tool_editor = text_editor::Content::with_text(&content);
            app.html_tool_scroll_top_line = 0.0;
            app.html_tool_scroll_remainder = 0.0;
            close_context_menu(app);
            let save_task = if app.html_tool_remember {
                save_html_tool_content_task(content.clone())
            } else {
                Task::none()
            };
            Task::batch(vec![clear_notification_task(), save_task])
        }
        HtmlToolMessage::ContentUpdated(None) => {
            app.html_tool_loading = false;
            app.html_tool_notification = Some("操作失败或格式错误".to_string());
            close_context_menu(app);
            clear_notification_task()
        }
        HtmlToolMessage::Beautify => {
            app.html_tool_loading = true;
            let text = app.html_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || beautify_html(&text)).await
                },
                |res| Message::HtmlTool(HtmlToolMessage::ContentUpdated(res)),
            )
        }
        HtmlToolMessage::Compress => {
            app.html_tool_loading = true;
            let text = app.html_tool_editor.text();
            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || compress_html(&text)).await
                },
                |res| Message::HtmlTool(HtmlToolMessage::ContentUpdated(res)),
            )
        }
        HtmlToolMessage::Clear => {
            app.html_tool_editor = text_editor::Content::new();
            app.html_tool_scroll_top_line = 0.0;
            app.html_tool_scroll_remainder = 0.0;
            close_context_menu(app);
            let save_task = if app.html_tool_remember {
                save_html_tool_content_task(String::new())
            } else {
                Task::none()
            };
            notify_success(app, "已清空");
            Task::batch(vec![clear_notification_task(), save_task])
        }
        HtmlToolMessage::Copy => {
            let text = app.html_tool_editor.text();
            notify_success(app, "已复制");
            close_context_menu(app);
            Task::batch(vec![iced::clipboard::write(text), clear_notification_task()])
        }
        HtmlToolMessage::ToggleRemember(val) => {
            app.html_tool_remember = val;
            close_context_menu(app);
            crate::app::set_config_field("html_tool_remember", serde_json::Value::Bool(val));
            if val {
                return save_html_tool_content_task(app.html_tool_editor.text());
            }
            Task::none()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn save_html_tool_content_task(content: String) -> Task<Message> {
    Task::perform(async move { save_html_tool_content_async(&content).await }, |result| {
        if let Err(error) = result {
            tracing::warn!(target: "vw_desktop", error = %error, "failed to save html tool content");
        }
        Message::None
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn save_html_tool_content_task(content: String) -> Task<Message> {
    save_html_tool_content(&content);
    Task::none()
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.html_tool_viewport_height / line_height).floor().max(1.0)
}

fn close_context_menu(app: &mut App) {
    app.html_tool_context_menu_open = false;
    app.html_tool_context_menu_pos = None;
}

fn notify_success(app: &mut App, message: &str) {
    app.html_tool_notification = Some(message.to_string());
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::HtmlTool(HtmlToolMessage::ClearNotification),
    )
}

fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.html_tool_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    if delta_lines == 0 {
        return;
    }

    let max_scroll = max_scroll_top_line(app);
    app.html_tool_scroll_top_line =
        (app.html_tool_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

/// HTML Token 类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    StartTag { name: String, raw: String, self_closing: bool },
    EndTag { name: String, raw: String },
    Comment(String),
    Doctype(String),
    RawText { content: String, mode: RawTextMode },
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawTextMode {
    Indented,
    PreserveExact,
}

pub(super) fn beautify_html(input: &str) -> Option<String> {
    let tokens = tokenize_html(input);
    let mut out = String::new();
    let mut depth: usize = 0;

    for token in tokens {
        match token {
            Token::Doctype(raw) | Token::Comment(raw) => {
                let s = raw.trim();
                if s.is_empty() {
                    continue;
                }
                out.push_str(&indent(depth));
                out.push_str(s);
                out.push('\n');
            }

            Token::StartTag { raw, self_closing, .. } => {
                let s = raw.trim();
                if s.is_empty() {
                    continue;
                }
                out.push_str(&indent(depth));
                out.push_str(s);
                out.push('\n');
                if !self_closing {
                    depth = depth.saturating_add(1);
                }
            }

            Token::EndTag { raw, .. } => {
                depth = depth.saturating_sub(1);
                let s = raw.trim();
                if s.is_empty() {
                    continue;
                }
                out.push_str(&indent(depth));
                out.push_str(s);
                out.push('\n');
            }

            Token::RawText { content, mode } => match mode {
                RawTextMode::Indented => {
                    if content.is_empty() {
                        continue;
                    }

                    let mut any = false;
                    for line in content.lines() {
                        let l = line.trim_end_matches(['\n', '\r']);
                        if l.trim().is_empty() {
                            continue;
                        }
                        any = true;
                        out.push_str(&indent(depth));
                        out.push_str(l);
                        out.push('\n');
                    }

                    if !any {
                        continue;
                    }
                }
                RawTextMode::PreserveExact => {
                    if content.is_empty() {
                        continue;
                    }

                    out.push_str(&content);
                    if !content.ends_with('\n') {
                        out.push('\n');
                    }
                }
            },

            Token::Text(text) => {
                let collapsed = collapse_ws(&text);
                let s = collapsed.trim();
                if s.is_empty() {
                    continue;
                }
                out.push_str(&indent(depth));
                out.push_str(s);
                out.push('\n');
            }
        }
    }

    let s = out.trim().to_string();
    if s.is_empty() { None } else { Some(s + "\n") }
}

pub(super) fn compress_html(input: &str) -> Option<String> {
    let tokens = tokenize_html(input);
    let mut out = String::new();
    let mut prev_was_tag = false;

    for token in tokens {
        match token {
            Token::StartTag { raw, .. }
            | Token::EndTag { raw, .. }
            | Token::Doctype(raw)
            | Token::Comment(raw) => {
                let s = raw.trim();
                if s.is_empty() {
                    continue;
                }
                out.push_str(s);
                prev_was_tag = s.ends_with('>');
            }

            Token::RawText { content, .. } => {
                if content.is_empty() {
                    continue;
                }
                out.push_str(&content);
                prev_was_tag = false;
            }

            Token::Text(text) => {
                let collapsed = collapse_ws(&text);
                let t = collapsed.trim();
                if t.is_empty() {
                    continue;
                }
                if prev_was_tag && out.ends_with('>') {
                    out.push_str(t);
                } else {
                    if !out.is_empty() && !out.ends_with(' ') && !out.ends_with('>') {
                        out.push(' ');
                    }
                    out.push_str(t);
                }
                prev_was_tag = false;
            }
        }
    }

    let s = out.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[inline]
fn indent(depth: usize) -> String {
    "    ".repeat(depth)
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn tokenize_html(input: &str) -> Vec<Token> {
    let void = void_tag_set();
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = &input[start..i];
            tokens.push(Token::Text(text.to_string()));
            continue;
        }

        if input[i..].starts_with("<!--") {
            let start = i;
            if let Some(end_rel) = input[i + 4..].find("-->") {
                i = i + 4 + end_rel + 3;
            } else {
                i = bytes.len();
            }
            tokens.push(Token::Comment(input[start..i].to_string()));
            continue;
        }

        if input[i..].starts_with("</") {
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let name_start = i;
            while i < bytes.len() && is_name_char(bytes[i]) {
                i += 1;
            }
            let name = input[name_start..i].to_lowercase();
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            tokens.push(Token::EndTag { name, raw: input[start..i].to_string() });
            continue;
        }

        if input[i..].starts_with("<!") {
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            let raw = input[start..i].to_string();
            if raw[0..raw.len().min(9)].to_ascii_lowercase().starts_with("<!doctype") {
                tokens.push(Token::Doctype(raw));
            } else {
                tokens.push(Token::Comment(raw));
            }
            continue;
        }

        if input[i..].starts_with("<?") {
            let start = i;
            if let Some(end_rel) = input[i + 2..].find("?>") {
                i = i + 2 + end_rel + 2;
            } else {
                while i < bytes.len() && bytes[i] != b'>' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            tokens.push(Token::Comment(input[start..i].to_string()));
            continue;
        }

        let start = i;
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let name_start = i;
        while i < bytes.len() && is_name_char(bytes[i]) {
            i += 1;
        }
        let name = input[name_start..i].to_lowercase();

        let mut in_single = false;
        let mut in_double = false;
        while i < bytes.len() {
            match bytes[i] {
                b'\'' if !in_double => in_single = !in_single,
                b'"' if !in_single => in_double = !in_double,
                b'>' if !in_single && !in_double => {
                    i += 1;
                    break;
                }
                _ => {}
            }
            i += 1;
        }

        let raw = input[start..i].to_string();
        let self_closing = is_self_closing_tag(&raw) || void.contains(&name);
        tokens.push(Token::StartTag { name: name.clone(), raw, self_closing });

        if !self_closing
            && i < bytes.len()
            && let Some(mode) = raw_text_mode(&name)
            && let Some((raw_text, end_tag_raw, end_tag_end)) =
                extract_raw_text_section(input, i, &name)
        {
            if !raw_text.is_empty() {
                tokens.push(Token::RawText { content: raw_text, mode });
            }
            if !end_tag_raw.is_empty() {
                tokens.push(Token::EndTag { name: name.clone(), raw: end_tag_raw });
            }
            i = end_tag_end;
        }
    }
    tokens
}

fn extract_raw_text_section(
    input: &str,
    from: usize,
    tag: &str,
) -> Option<(String, String, usize)> {
    let needle = format!("</{}", tag);
    let close_start = find_case_insensitive(input, from, &needle)?;
    let raw_text = input[from..close_start].to_string();
    let end_tag_end = input[close_start..].find('>').map(|p| close_start + p + 1)?;
    let end_tag_raw = input[close_start..end_tag_end].to_string();
    Some((raw_text, end_tag_raw, end_tag_end))
}

fn raw_text_mode(tag: &str) -> Option<RawTextMode> {
    match tag {
        "script" | "style" => Some(RawTextMode::Indented),
        "textarea" | "pre" => Some(RawTextMode::PreserveExact),
        _ => None,
    }
}

fn find_case_insensitive(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();

    if n.is_empty() || from >= h.len() {
        return None;
    }

    let needle_lower: Vec<u8> = n.iter().map(|b| b.to_ascii_lowercase()).collect();
    let mut i = from;

    while i + needle_lower.len() <= h.len() {
        let mut ok = true;
        for (j, nb) in needle_lower.iter().enumerate() {
            if h[i + j].to_ascii_lowercase() != *nb {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[inline]
fn is_self_closing_tag(raw: &str) -> bool {
    let s = raw.trim_end();
    s.ends_with("/>")
}

#[inline]
fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b':' || b == b'-' || b == b'_'
}

fn void_tag_set() -> &'static HashSet<String> {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        for t in [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
            "source", "track", "wbr",
        ] {
            s.insert(t.to_string());
        }
        s
    })
}
