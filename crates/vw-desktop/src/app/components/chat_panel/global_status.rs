//! 聊天面板底部全局状态栏。

use iced::widget::text::{LineHeight, Wrapping};
use iced::widget::{Space, column, container, text};
use iced::{Background, Border, Color, Element, Length, Theme};
use serde_json::Value;

use crate::app::components::chat_panel::tools::{
    tool_inline_summary, tool_input, tool_name_from_raw, tool_status, tool_verb,
};
use crate::app::models::{ChatMessage, ChatRole};
use crate::app::{App, Message};

use super::utils::{bold_font, chat_secondary_subtle_text_color, truncate_chars};

const STATUS_CARD_HEIGHT: f32 = 52.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChatGlobalStatus {
    pub action: String,
    pub detail: String,
}

pub(crate) fn chat_global_status(
    chat: &[ChatMessage],
    is_requesting: bool,
) -> Option<ChatGlobalStatus> {
    let Some(last_assistant) =
        chat.iter().rev().find(|message| message.role == ChatRole::Assistant)
    else {
        return None;
    };

    if let Some(raw_tool) = last_tool_block(&last_assistant.content)
        && let Some(status) = status_from_tool_raw(&raw_tool, is_requesting)
    {
        return Some(status);
    }

    let (thinks, _, _thinking_open) = crate::app::ui::chat::split_think(&last_assistant.content);
    if is_requesting && !thinks.is_empty() {
        return Some(ChatGlobalStatus {
            action: "正在思考".to_string(),
            detail: latest_think_detail(&thinks),
        });
    }

    None
}

pub(crate) fn chat_global_status_view(app: &App) -> Element<'_, Message> {
    let Some(status) = chat_global_status(&app.chat, app.current_session_runtime().is_requesting)
    else {
        return Space::new().height(Length::Fixed(0.0)).into();
    };
    let action = status.action;
    let detail = status.detail;
    let detail_text =
        if detail.trim().is_empty() { " ".to_string() } else { detail.trim().to_string() };

    let content = column![
        text(action)
            .size(12)
            .font(bold_font())
            .style(|theme: &Theme| iced::widget::text::Style { color: Some(theme.palette().text) }),
        container(
            text(detail_text)
                .size(11)
                .width(Length::Fill)
                .line_height(LineHeight::Relative(1.15))
                .wrapping(Wrapping::Word)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(chat_secondary_subtle_text_color(theme)),
                })
        )
        .width(Length::Fill)
        .height(Length::Fixed(24.0))
        .clip(true),
    ]
    .spacing(1);

    let card = container(content)
        .width(Length::Fill)
        .height(Length::Fixed(STATUS_CARD_HEIGHT))
        .padding(iced::Padding { top: 7.0, right: 28.0, bottom: 5.0, left: 28.0 })
        .style(|theme: &Theme| {
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;
            iced::widget::container::Style {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgba8(24, 26, 30, 0.98)
                } else {
                    Color::from_rgba8(252, 252, 253, 1.0)
                })),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        Color::from_rgba8(24, 26, 30, 0.98)
                    } else {
                        Color::from_rgba8(252, 252, 253, 1.0)
                    },
                    radius: iced::border::Radius {
                        top_left: 28.0,
                        top_right: 28.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(if is_dark { 0.22 } else { 0.08 }),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            }
        });

    container(card)
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(iced::Padding { top: 8.0, right: 28.0, bottom: 0.0, left: 28.0 })
        .into()
}

fn status_from_tool_raw(raw: &str, is_requesting: bool) -> Option<ChatGlobalStatus> {
    let tool_name = tool_name_from_raw(raw)?;
    let (_, rest) = raw.split_once('\n')?;
    let value = serde_json::from_str::<Value>(rest.trim()).ok()?;
    let status = tool_status(&value);
    let input = tool_input(&value);
    let summary = tool_inline_summary(&tool_name, input).unwrap_or_default();
    let verb = tool_verb(&tool_name);
    let detail = if summary.trim().is_empty() { tool_name.clone() } else { summary };

    let action = match status {
        "completed" => return None,
        "error" => format!("{}失败", verb),
        "running" => format!("正在{}", verb),
        _ if is_requesting && status.is_empty() => format!("正在{}", verb),
        _ => return None,
    };

    Some(ChatGlobalStatus { action, detail })
}

fn latest_think_detail(thinks: &[String]) -> String {
    let text = thinks.last().map(String::as_str).unwrap_or_default();
    let mut lines = text
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        return String::new();
    }

    if lines.len() > 2 {
        lines = lines.split_off(lines.len() - 2);
    }

    lines.into_iter().map(|line| truncate_chars(line.trim(), 92)).collect::<Vec<_>>().join("\n")
}

fn last_tool_block(raw: &str) -> Option<String> {
    let mut rest = raw;
    let mut last = None;
    while let Some(pos) = find_tool_start(rest) {
        let tool = &rest[pos..];
        let Some(consumed) = parse_tool_block(tool) else {
            break;
        };
        last = Some(tool[..consumed].to_string());
        rest = &tool[consumed..];
    }
    last
}

fn find_tool_start(s: &str) -> Option<usize> {
    if s.starts_with("tool ") && parse_tool_block(s).is_some() {
        return Some(0);
    }
    let mut search_from = 0usize;
    while let Some(pos_rel) = s[search_from..].find("\ntool ") {
        let pos = search_from + pos_rel + 1;
        if parse_tool_block(&s[pos..]).is_some() {
            return Some(pos);
        }
        search_from = pos + 1;
    }
    None
}

fn parse_tool_block(s: &str) -> Option<usize> {
    if !s.starts_with("tool ") {
        return None;
    }

    let line_end = s.find('\n')?;
    let mut idx = line_end + 1;
    let mut buf = String::new();

    for _ in 0..64 {
        if idx >= s.len() {
            break;
        }

        let next_end = s[idx..].find('\n').map(|offset| idx + offset).unwrap_or(s.len());
        let line = &s[idx..next_end];
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(line);

        if serde_json::from_str::<Value>(buf.trim()).is_ok() {
            return Some(if next_end < s.len() { next_end + 1 } else { next_end });
        }
        if next_end >= s.len() {
            break;
        }
        idx = next_end + 1;
    }

    None
}
