//! 渲染 Brief 工具结果。
//! 视图展示发送状态、附件和正文摘要，保持聊天面板内的结果紧凑可读。

use chrono::{DateTime, Local};
use iced::widget::{Space, button, column, container, mouse_area, row, text};
use iced::{Alignment, Element, Length, Theme};
use serde_json::{Map, Value};
use std::path::Path;

use crate::app::components::chat_panel::utils::{
    chat_secondary_muted_text_color, chat_secondary_subtle_text_color, chat_secondary_text_color,
    eye_icon_button_style, eye_icon_svg_style, icon_svg, truncate_chars,
};
use crate::app::{App, Message, message};
use crate::app::{assets::Icon};

use super::tool_parse::{tool_result_data, tool_status};
use super::{ToolTextTarget, canonical_tool_name, tool_text_editor};

#[derive(Debug, Clone)]
struct BriefAttachment {
    path: String,
    size: u64,
    is_image: bool,
}

fn brief_data(value: &Value) -> Option<&Map<String, Value>> {
    tool_result_data(value)?.as_object()
}

pub(super) fn compact_attachment_path(path: &str) -> String {
    let path_ref = Path::new(path);
    let file_name = path_ref
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(path);
    let parent = path_ref
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty());

    match parent {
        Some(parent) => truncate_chars(&format!("{parent}/{file_name}"), 72).to_string(),
        None => truncate_chars(file_name, 72).to_string(),
    }
}

pub(super) fn format_attachment_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    let size = size as f64;
    if size >= MB {
        format!("{:.1} MB", size / MB)
    } else if size >= KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{} B", size as u64)
    }
}

pub(super) fn format_sent_at(value: &str) -> Option<String> {
    let parsed = DateTime::parse_from_rfc3339(value).ok()?;
    Some(parsed.with_timezone(&Local).format("%H:%M").to_string())
}

fn parse_attachments(data: &Map<String, Value>) -> Vec<BriefAttachment> {
    data.get("attachments")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let object = item.as_object()?;
            let path = object.get("path").and_then(Value::as_str)?.trim();
            if path.is_empty() {
                return None;
            }
            Some(BriefAttachment {
                path: path.to_string(),
                size: object.get("size").and_then(Value::as_u64).unwrap_or(0),
                is_image: object.get("isImage").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect()
}

/// 执行 tool_brief_view 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub fn tool_brief_view<'a>(
    app: &'a App,
    msg_idx: usize,
    tool_idx: usize,
    visible: &str,
) -> Option<Element<'a, Message>> {
    let (first, rest) = visible.split_once('\n')?;
    let tool_name = canonical_tool_name(first.trim().strip_prefix("tool ")?.trim());
    if tool_name != "brief" {
        return None;
    }

    let value = serde_json::from_str::<Value>(rest.trim()).ok()?;
    let status = tool_status(&value);
    if matches!(status, "error" | "denied") {
        return None;
    }

    let data = brief_data(&value)?;
    let message = data
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let attachments = parse_attachments(data);
    if message.is_empty() && attachments.is_empty() {
        return None;
    }

    let intent = data
        .get("status")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("normal")
        .to_string();
    let sent_at = data
        .get("sentAt")
        .and_then(Value::as_str)
        .and_then(format_sent_at);

    let key = ((msg_idx as u64) << 32) | (tool_idx as u64);
    let is_hovered = app.chat_tool_hovered_idx == Some(key);
    let detail_btn = button(
        icon_svg(Icon::Eye)
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .style(eye_icon_svg_style),
    )
    .padding([2, 4])
    .style(|theme: &Theme, status| eye_icon_button_style(theme, status))
    .on_press(Message::Chat(message::ChatMessage::OpenToolDetail(
        msg_idx,
        tool_idx,
        visible.to_string(),
    )));
    let detail_slot: Element<'a, Message> =
        if is_hovered { detail_btn.into() } else { Space::new().width(Length::Fixed(22.0)).into() };

    let mut meta_parts = Vec::new();
    if is_hovered || intent == "proactive" {
        meta_parts.push("Brief".to_string());
    }
    if intent == "proactive" {
        meta_parts.push("主动更新".to_string());
    }
    if let Some(sent_at) = sent_at {
        meta_parts.push(sent_at);
    }
    let meta_label = meta_parts.join(" · ");

    let head_row = row![
        text(meta_label).size(12).style(|theme: &Theme| iced::widget::text::Style {
            color: Some(chat_secondary_subtle_text_color(theme)),
        }),
        container(Space::new()).width(Length::Fill),
        detail_slot,
    ]
    .align_y(Alignment::Center);

    let mut body = column![].spacing(6);
    if !message.is_empty() {
        let message_view = tool_text_editor(
            app,
            ToolTextTarget::ToolCardText { msg_idx, tool_idx, text_idx: 0 },
            crate::app::components::chat_panel::tool_text_support::chat_text_font_name(),
            14.0,
            true,
            false,
        )
        .unwrap_or_else(|| {
            text(message)
                .size(15)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.palette().text),
                })
                .into()
        });
        body = body.push(container(message_view).width(Length::Fill));
    }

    if !attachments.is_empty() {
        let mut attachment_column = column![].spacing(4);
        for attachment in attachments {
            let label = if attachment.is_image { "图像" } else { "文件" };
            let display_path = compact_attachment_path(&attachment.path);
            let size_text = format_attachment_size(attachment.size);
            attachment_column = attachment_column.push(
                row![
                    text(label).size(12).style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(chat_secondary_muted_text_color(theme)),
                    }),
                    text(display_path).size(13).style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(chat_secondary_text_color(theme)),
                    }),
                    text(size_text).size(12).style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(chat_secondary_subtle_text_color(theme)),
                    }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        body = body.push(container(attachment_column).padding([2, 2]).width(Length::Fill));
    }

    let card = mouse_area(container(column![head_row, body].spacing(4)).width(Length::Fill))
    .on_enter(Message::Chat(message::ChatMessage::ToolHover(msg_idx, tool_idx)))
    .on_exit(Message::Chat(message::ChatMessage::ToolHoverLeave));

    Some(card.into())
}
