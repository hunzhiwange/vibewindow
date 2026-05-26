//! 消息文本主体相关工具。
//!
//! 该模块负责纯文本消息、只读编辑器和思考文本内容的呈现，
//! 并集中维护分段显示与粗略高度估算逻辑。

use iced::widget::{column, container, text, text_editor};
use iced::{Border, Color, Element, Length, Theme};

use crate::app::{message, App, Message};

use super::styles::{
    message_body_text_color, message_text_line_height, think_block_text_color,
    MESSAGE_TEXT_SIZE,
};
use super::super::tool_text_support::{
    chat_text_font, chat_text_line_height, read_only_text_style,
};
use super::super::utils::normalize_display_text;

pub(super) const MAX_EDITOR_CHARS: usize = 20_000;

const SEGMENT_MAX_LINES: usize = 96;
const SEGMENT_MAX_CHARS: usize = 4_000;

pub(super) fn estimate_text_height(text: &str) -> f32 {
    const CHARS_PER_LINE: usize = 72;
    const LINE_HEIGHT: f32 = 24.0;

    let explicit_lines = text.lines().count().max(1);
    let wrapped_lines = text.len().div_ceil(CHARS_PER_LINE).max(1);
    explicit_lines.max(wrapped_lines) as f32 * LINE_HEIGHT
}

fn split_text_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_lines = 0usize;

    for line in text.lines() {
        let line_len = line.len() + usize::from(!current.is_empty());
        let next_lines = current_lines.saturating_add(1);
        let next_len = current.len().saturating_add(line_len);
        if !current.is_empty() && (next_lines > SEGMENT_MAX_LINES || next_len > SEGMENT_MAX_CHARS)
        {
            segments.push(current);
            current = String::new();
            current_lines = 0;
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        current_lines += 1;
    }

    if current.is_empty() {
        if !text.is_empty() {
            segments.push(text.to_string());
        }
    } else {
        segments.push(current);
    }

    segments
}

pub(super) fn should_segment_text_block(text: &str) -> bool {
    text.len() > SEGMENT_MAX_CHARS || text.lines().count() > SEGMENT_MAX_LINES
}

pub(super) fn message_text_body<'a>(content: String, is_user: bool) -> Element<'a, Message> {
    if should_segment_text_block(&content) {
        let mut blocks = column![].spacing(10);
        for segment in split_text_segments(&content) {
            blocks = blocks.push(
                container(
                    text(segment)
                        .size(MESSAGE_TEXT_SIZE)
                        .font(chat_text_font())
                        .line_height(message_text_line_height())
                        .style(move |theme: &Theme| iced::widget::text::Style {
                            color: Some(message_body_text_color(theme, is_user)),
                        }),
                )
                .width(Length::Fill),
            );
        }
        blocks.into()
    } else {
        container(
            text(content)
                .size(MESSAGE_TEXT_SIZE)
                .font(chat_text_font())
                .line_height(message_text_line_height())
                .style(move |theme: &Theme| iced::widget::text::Style {
                    color: Some(message_body_text_color(theme, is_user)),
                }),
        )
        .width(Length::Fill)
        .into()
    }
}

pub(super) fn message_editor_body<'a>(
    app: &'a App,
    idx: usize,
    value: Color,
) -> Option<Element<'a, Message>> {
    let editor_content = app.chat_message_editors.get(idx)?;
    let on_action = move |action| Message::Chat(message::ChatMessage::MessageEditorAction(idx, action));
    let editor = text_editor(editor_content)
        .on_action(on_action)
        .size(MESSAGE_TEXT_SIZE)
        .line_height(message_text_line_height())
        .padding(0)
        .height(Length::Shrink)
        .font(chat_text_font())
        .style(move |theme: &Theme, _status: text_editor::Status| {
            read_only_text_style(theme, value)
        });

    Some(container(editor).width(Length::Fill).into())
}

pub(super) fn think_text_body<'a>(
    app: &'a App,
    msg_idx: usize,
    think_idx: usize,
    content: String,
    prefer_plain_text: bool,
) -> Element<'a, Message> {
    let scroll_key = ((msg_idx as u64) << 32) | (think_idx as u64);
    if !prefer_plain_text
        && let Some(editor_content) = app.chat_think_editors.get(&scroll_key)
    {
        let on_action = move |action| {
            Message::Chat(message::ChatMessage::ThinkEditorAction(msg_idx, think_idx, action))
        };
        let editor = text_editor(editor_content)
            .on_action(on_action)
            .size(14.0)
            .line_height(chat_text_line_height())
            .padding(0)
            .height(Length::Shrink)
            .font(chat_text_font())
            .style(move |theme: &Theme, _status: text_editor::Status| {
                let value = think_block_text_color(theme);
                read_only_text_style(theme, value)
            });
        return container(editor)
            .width(Length::Fill)
            .style(|_theme: &Theme| iced::widget::container::Style {
                background: None,
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .into();
    }

    if should_segment_text_block(&content) {
        let mut blocks = column![].spacing(10);
        for segment in split_text_segments(&content) {
            blocks = blocks.push(
                container(
                    text(segment)
                        .size(14)
                        .font(chat_text_font())
                        .line_height(chat_text_line_height())
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(think_block_text_color(theme)),
                        }),
                )
                .width(Length::Fill)
                .style(|_theme: &Theme| iced::widget::container::Style {
                    background: None,
                    border: Border {
                        width: 0.0,
                        color: Color::TRANSPARENT,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        blocks.into()
    } else {
        container(
            text(content)
                .size(14)
                .font(chat_text_font())
                .line_height(chat_text_line_height())
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(think_block_text_color(theme)),
                }),
        )
        .width(Length::Fill)
        .style(|_theme: &Theme| iced::widget::container::Style {
            background: None,
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

pub(crate) fn should_prefer_plain_think_body(is_streaming_msg: bool, is_thinking: bool) -> bool {
    is_streaming_msg || is_thinking
}

pub(crate) fn estimate_message_height_rough(raw: &str) -> f32 {
    let normalized = normalize_display_text(raw.trim());
    let normalized = normalized.trim();
    let text_len = normalized.len();
    let explicit_lines = normalized.lines().count().max(1) as f32;
    let wrapped_lines = text_len.div_ceil(96).max(1) as f32;
    let line_count = explicit_lines.max(wrapped_lines);
    let think_bonus = normalized.matches("<think").count() as f32 * 28.0;
    let tool_bonus = normalized.matches("tool ").count() as f32 * 36.0;
    (line_count * 22.0 + think_bonus + tool_bonus + 56.0).clamp(64.0, 720.0)
}