//! 消息视图渲染模块
//!
//! 本模块负责将聊天消息渲染为 Iced UI 元素。主要功能包括：
//! - 解析消息内容中的特殊区块（思考块、工具调用块、文本块）
//! - 渲染不同角色（用户、助手、系统）的消息气泡
//! - 处理消息的交互功能（复制、展开/折叠思考块、编辑器集成）
//!
//! ## 主要组件
//!
//! - [`message_view`]: 消息视图的主入口函数，负责完整消息气泡的渲染
//! - [`think_block_view`]: 渲染可展开/折叠的思考块
//! - [`assistant_api_error_view`]: 渲染 API 错误信息的特殊视图
//!
//! ## 内容解析
//!
//! 消息内容可能包含以下特殊区块：
//! - `<think ...>...</think`>: 思考块，显示 AI 的推理过程
//! - `tool ...`: 工具调用块，记录 AI 执行的工具操作

mod assistant_body;
mod assistant_error;
mod parse;
mod render_cache;
mod styles;
mod text;
mod think_block;
mod tool_summaries;

#[cfg(test)]
mod assistant_error_tests;
#[cfg(test)]
mod parse_tests;
#[cfg(test)]
mod render_cache_tests;
#[cfg(test)]
mod styles_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod think_block_tests;
#[cfg(test)]
mod tool_summaries_tests;

use iced::widget::svg::{self};
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{Space, button, column, container, row, text as iced_text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};
use std::borrow::Cow;

use crate::app::assets::Icon;
use crate::app::components::input_panel::attachment::{
    attachment_preview_strip, parse_attachment_markers,
};
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::components::widgets::RightClickArea;
use crate::app::models::{self, ChatRole};
use crate::app::{message, App, Message};

use self::assistant_body::render_assistant_body;
use self::render_cache::resolve_visible_text_and_copy_hash;
use self::styles::{
    is_dark_theme, message_meta_text_color, neutral_card_surface, subtle_card_shadow,
    user_bubble_surface, COMPACT_ACTION_BUTTON_RADIUS, COMPACT_ACTION_BUTTON_SIZE,
    MESSAGE_META_TEXT_SIZE,
};
use self::text::{message_editor_body, message_text_body, MAX_EDITOR_CHARS};

use super::tool_text_support::{
    chat_text_font, is_safe_for_text_editor, selected_chat_text_for_message,
};
use super::tools::{
    pending_permission_badge_label, pending_permission_request_badge_label,
    pending_permission_request_for_message, pending_permission_targets_message,
};
use super::utils::{
    chat_context_menu, chat_context_target_key, chat_secondary_muted_text_color,
    copy_tooltip_content, icon_svg, is_recent_copy, normalize_display_text,
};

pub(crate) use self::parse::hash_chat_content;
pub(crate) use self::render_cache::{
    assistant_render_blocks, build_render_cache_entry, effective_assistant_render_cache,
};
pub(crate) use self::assistant_body::deduped_tool_last_indices;
#[cfg(test)]
pub(crate) use self::assistant_body::should_highlight_pending_permission_tool;
pub(crate) use self::text::estimate_message_height_rough;
#[cfg(test)]
pub(crate) use self::text::should_prefer_plain_think_body;
#[cfg(test)]
pub(crate) use self::think_block::{
    should_render_think_block, think_block_default_expanded, think_block_resolved_expanded,
};
#[cfg(test)]
pub(crate) use self::tool_summaries::summarize_explore_items;
#[cfg(test)]
pub(crate) use self::tool_summaries::trailing_tool_tail_text_source_block_idx;
#[cfg(test)]
pub(crate) use self::tool_summaries::tool_card_text_blocks;

/// 渲染消息视图（主入口函数）
///
/// 创建完整的消息气泡 UI，包括消息内容、元信息（模型名称、时间）和操作按钮（复制）。
/// 根据消息角色（用户/助手/系统）采用不同的布局和样式。
pub fn message_view<'a>(
    app: &'a App,
    idx: usize,
    role: ChatRole,
    content: &'a str,
    think_timing: &'a [models::ThinkTiming],
    message_meta: Option<Cow<'a, str>>,
    render_cache: &'a models::ChatRenderCacheEntry,
    _enable_heavy_tool_content: bool,
) -> Element<'a, Message> {
    let runtime = app.current_session_runtime();
    let is_user = role == ChatRole::User;
    let assistant_like = matches!(role, ChatRole::Assistant | ChatRole::Tool);
    let is_streaming_assistant =
        role == ChatRole::Assistant && runtime.is_requesting && idx + 1 == app.chat.len();
    let (visible_for_copy, visible_content_hash) =
        resolve_visible_text_and_copy_hash(app, idx, content, render_cache);
    let render_cache = effective_assistant_render_cache(
        content,
        render_cache,
        visible_for_copy.as_ref(),
        visible_content_hash,
        is_streaming_assistant,
    );
    let (assistant_blocks, has_special_assistant_blocks) =
        assistant_render_blocks(content, render_cache.as_ref(), false);
    let assistant_blocks = assistant_blocks.into_owned();
    let assistant_has_tool_blocks = assistant_blocks
        .iter()
        .any(|block| matches!(block, models::ParsedChatBlock::Tool { .. }));
    let (cleaned_attachment_text, attachment_items) = parse_attachment_markers(content);
    let prefer_attachment_card_render = !attachment_items.is_empty() && !has_special_assistant_blocks;

    let recently_copied = is_recent_copy(app, visible_content_hash);
    let copy_icon = if recently_copied { Icon::Check } else { Icon::Copy };
    let copy_tip = if recently_copied { "已复制" } else { "复制消息" };
    let permission_badge_text = pending_permission_badge_label(
        &app.permission_modal_requests,
        app.permission_modal_request_id.as_deref(),
    );
    let message_id = app.chat_message_ids.get(idx).and_then(|message_id| message_id.as_deref());
    let supports_permission_badge =
        role == ChatRole::Tool || (role == ChatRole::Assistant && assistant_has_tool_blocks);
    let is_permission_target = pending_permission_targets_message(
        app.permission_modal_request.as_ref(),
        message_id,
    ) && supports_permission_badge;
    let matched_permission_request = supports_permission_badge
        .then(|| pending_permission_request_for_message(&app.permission_modal_requests, message_id))
        .flatten();
    let permission_badge: Option<(String, Option<Message>)> = if is_permission_target {
        Some((
            permission_badge_text.clone(),
            app.permission_modal_request_id.clone().map(|request_id| {
                Message::Chat(message::ChatMessage::PermissionSelectRequest(request_id))
            }),
        ))
    } else {
        matched_permission_request.map(|request| {
            (
                pending_permission_request_badge_label(
                    &app.permission_modal_requests,
                    request.id.as_str(),
                ),
                Some(Message::Chat(message::ChatMessage::PermissionSelectRequest(
                    request.id.clone(),
                ))),
            )
        })
    };
    let has_message_id =
        app.chat_message_ids.get(idx).and_then(|message_id| message_id.as_ref()).is_some();
    let can_branch_or_reset = is_user && app.active_session_id.is_some() && has_message_id;
    let show_reset_actions =
        can_branch_or_reset && has_message_id && app.chat_reset_menu_idx == Some(idx);

    let action_btn = |label: &'static str, on_press: Message| -> Element<'a, Message> {
        button(iced_text(label).size(13))
            .padding([4, 10])
            .style(|theme: &Theme, status| {
                let (idle_bg, idle_border) = neutral_card_surface(theme);
                let (bg, border_color) = match status {
                    iced::widget::button::Status::Pressed => {
                        if is_dark_theme(theme) {
                            (
                                Color::from_rgba8(36, 38, 44, 0.98),
                                Color::from_rgba8(63, 67, 75, 0.94),
                            )
                        } else {
                            (
                                Color::from_rgba8(232, 236, 241, 1.0),
                                Color::from_rgba8(210, 216, 224, 1.0),
                            )
                        }
                    }
                    iced::widget::button::Status::Hovered => {
                        if is_dark_theme(theme) {
                            (
                                Color::from_rgba8(31, 33, 38, 0.96),
                                Color::from_rgba8(52, 56, 63, 0.92),
                            )
                        } else {
                            (
                                Color::from_rgba8(241, 243, 246, 1.0),
                                Color::from_rgba8(218, 223, 230, 1.0),
                            )
                        }
                    }
                    _ => (idle_bg, idle_border),
                };
                iced::widget::button::Style {
                    background: Some(Background::Color(bg)),
                    border: Border { width: 1.0, color: border_color, radius: 999.0.into() },
                    text_color: chat_secondary_muted_text_color(theme),
                    ..Default::default()
                }
            })
            .on_press(on_press)
            .into()
    };

    let icon_action_btn = |icon: Icon,
                           label: &'static str,
                           on_press: Message,
                           highlighted: bool,
                           compact_square: bool|
     -> Element<'a, Message> {
        let icon_size = if compact_square { 7.5 } else { 11.0 };
        let btn = button(
            icon_svg(icon)
                .width(Length::Fixed(icon_size))
                .height(Length::Fixed(icon_size))
                .style(move |theme: &Theme, _status| svg::Style {
                    color: Some(if highlighted {
                        theme.extended_palette().success.base.color
                    } else {
                        chat_secondary_muted_text_color(theme)
                    }),
                }),
        )
        .width(if compact_square {
            Length::Fixed(COMPACT_ACTION_BUTTON_SIZE)
        } else {
            Length::Shrink
        })
        .height(if compact_square {
            Length::Fixed(COMPACT_ACTION_BUTTON_SIZE)
        } else {
            Length::Shrink
        })
        .padding(if compact_square { 0 } else { 2 })
        .style(move |theme: &Theme, status| {
            let (bg, show_bg) = match status {
                iced::widget::button::Status::Hovered => {
                    if is_dark_theme(theme) {
                        (Color::from_rgba8(31, 33, 38, 0.94), true)
                    } else {
                        (Color::from_rgb8(0xF1, 0xF3, 0xF6), true)
                    }
                }
                iced::widget::button::Status::Pressed => {
                    if is_dark_theme(theme) {
                        (Color::from_rgba8(36, 38, 44, 0.98), true)
                    } else {
                        (Color::from_rgb8(0xE8, 0xEC, 0xF1), true)
                    }
                }
                _ => (Color::TRANSPARENT, false),
            };
            iced::widget::button::Style {
                background: if show_bg { Some(Background::Color(bg)) } else { None },
                border: Border {
                    width: if show_bg { 1.0 } else { 0.0 },
                    color: if is_dark_theme(theme) {
                        Color::from_rgba8(52, 56, 63, 0.9)
                    } else {
                        Color::from_rgba8(218, 223, 230, 1.0)
                    },
                    radius: if compact_square {
                        COMPACT_ACTION_BUTTON_RADIUS.into()
                    } else {
                        999.0.into()
                    },
                },
                text_color: chat_secondary_muted_text_color(theme),
                ..Default::default()
            }
        })
        .on_press(on_press);

        Tooltip::new(btn, copy_tooltip_content(label), TooltipPosition::Top).gap(6).into()
    };

    let approval_badge_view = |label: String, on_press: Option<Message>| -> Element<'a, Message> {
        let badge = container(
            iced_text(label)
                .size(11)
                .font(chat_text_font())
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(if is_dark_theme(theme) {
                        Color::from_rgba8(255, 235, 177, 0.96)
                    } else {
                        Color::from_rgba8(142, 103, 16, 1.0)
                    }),
                }),
        )
        .padding([2, 8])
        .style(|theme: &Theme| {
            let is_dark = is_dark_theme(theme);
            iced::widget::container::Style {
                background: Some(Background::Color(if is_dark {
                    Color::from_rgba8(79, 62, 21, 0.72)
                } else {
                    Color::from_rgba8(255, 245, 213, 1.0)
                })),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        Color::from_rgba8(168, 132, 45, 0.72)
                    } else {
                        Color::from_rgba8(222, 187, 102, 0.92)
                    },
                    radius: 999.0.into(),
                },
                ..Default::default()
            }
        });

        if let Some(on_press) = on_press {
            button(badge)
                .padding(0)
                .style(|_theme: &Theme, _status| iced::widget::button::Style {
                    background: None,
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .on_press(on_press)
                .into()
        } else {
            badge.into()
        }
    };

    let copy_message = Message::CopyCode(visible_for_copy.to_string());

    let fork_press = Message::Chat(message::ChatMessage::ForkSessionAt(idx));
    let reset_press = Message::Chat(message::ChatMessage::ToggleResetMenu(idx));
    let reset_revert_press = Message::Chat(message::ChatMessage::ResetSessionToMessage {
        msg_idx: idx,
        revert_code: true,
    });
    let reset_keep_press = Message::Chat(message::ChatMessage::ResetSessionToMessage {
        msg_idx: idx,
        revert_code: false,
    });

    let body = if assistant_like && !prefer_attachment_card_render {
        render_assistant_body(
            app,
            idx,
            think_timing,
            render_cache.special_text_blocks.clone(),
            render_cache.display_text.clone(),
            assistant_blocks,
            has_special_assistant_blocks,
            is_streaming_assistant,
            visible_for_copy.as_ref(),
        )
    } else {
        let mut body = column![].spacing(8);
        let use_editor = attachment_items.is_empty()
            && content.len() <= MAX_EDITOR_CHARS
            && is_safe_for_text_editor(content)
            && app.chat_message_editors.get(idx).is_some();
        if use_editor {
            if let Some(view) = message_editor_body(app, idx, app.theme().palette().text) {
                body = body.push(view);
            }
        } else {
            let text = normalize_display_text(cleaned_attachment_text.trim()).into_owned();
            if !text.is_empty() {
                body = body.push(message_text_body(text, true));
            }
        }
        body
    };
    let body = if prefer_attachment_card_render {
        let mut attachment_body = column![].spacing(8);
        if !attachment_items.is_empty() {
            attachment_body = attachment_body.push(attachment_preview_strip(attachment_items.clone()));
        }
        let text = normalize_display_text(cleaned_attachment_text.trim()).into_owned();
        if !text.is_empty() {
            attachment_body = attachment_body.push(message_text_body(text, true));
        }
        attachment_body
    } else if !attachment_items.is_empty() && !assistant_like {
        let mut attachment_body = column![attachment_preview_strip(attachment_items.clone())].spacing(8);
        let text = normalize_display_text(cleaned_attachment_text.trim()).into_owned();
        if !text.is_empty() {
            attachment_body = attachment_body.push(message_text_body(text, true));
        }
        attachment_body
    } else {
        body
    };

    let context_key = chat_context_target_key(idx, None);
    let context_menu_open = app.chat_context_menu_target == Some(context_key);
    let context_menu_anchor = app.chat_context_menu_pos.unwrap_or((12.0, 26.0));

    let bubble_inner = container(body)
        .padding(if is_user {
            iced::Padding { top: 12.0, right: 14.0, bottom: 12.0, left: 14.0 }
        } else {
            iced::Padding { top: 2.0, right: 2.0, bottom: 2.0, left: 0.0 }
        })
        .width(if is_user { Length::FillPortion(7) } else { Length::Fill })
        .style(move |theme: &Theme| {
            if !is_user {
                if is_permission_target {
                    let is_dark = is_dark_theme(theme);
                    return iced::widget::container::Style {
                        background: Some(Background::Color(if is_dark {
                            Color::from_rgba8(61, 49, 19, 0.34)
                        } else {
                            Color::from_rgba8(255, 248, 229, 0.96)
                        })),
                        border: Border {
                            width: 1.0,
                            color: if is_dark {
                                Color::from_rgba8(168, 132, 45, 0.82)
                            } else {
                                Color::from_rgba8(214, 169, 68, 0.96)
                            },
                            radius: 14.0.into(),
                        },
                        ..Default::default()
                    };
                }
                return iced::widget::container::Style {
                    background: None,
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                };
            }

            let (bubble_bg, bubble_border) = user_bubble_surface(theme);
            iced::widget::container::Style {
                background: Some(Background::Color(bubble_bg)),
                border: Border { width: 1.0, color: bubble_border, radius: 18.0.into() },
                shadow: subtle_card_shadow(theme),
                ..Default::default()
            }
        });

    let context_text = selected_chat_text_for_message(app, idx).unwrap_or_else(|| {
        if !visible_for_copy.trim().is_empty() {
            visible_for_copy.trim().to_string()
        } else {
            content.trim().to_string()
        }
    });

    let bubble_inner: Element<'a, Message> = RightClickArea::new(
        bubble_inner.into(),
        Box::new(move |point| {
            Message::Chat(message::ChatMessage::OpenMessageContextMenu {
                target: context_key,
                x: point.x,
                y: point.y,
                text: context_text.clone(),
            })
        }),
    )
    .preserve_on_right_click()
    .into();

    let bubble_inner: Element<'a, Message> = if let Some(menu) = chat_context_menu(context_menu_open)
    {
        PointBelowOverlay::new(bubble_inner, menu)
            .show(true)
            .anchor(iced::Point::new(context_menu_anchor.0, context_menu_anchor.1))
            .gap(0.0)
            .on_close(Message::Chat(message::ChatMessage::CloseMessageContextMenu))
            .into()
    } else {
        bubble_inner
    };

    let footer_element: Element<'a, Message> = {
        let mut footer_col = column![].spacing(6);

        if show_reset_actions {
            let reset_confirm_row: Element<'a, Message> = if is_user {
                row![
                    container(Space::new()).width(Length::Fill),
                    row![
                        action_btn("回滚代码", reset_revert_press),
                        action_btn("不回滚代码", reset_keep_press),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
            } else {
                row![
                    action_btn("回滚代码", reset_revert_press),
                    action_btn("不回滚代码", reset_keep_press),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
            };
            footer_col = footer_col.push(reset_confirm_row);
        }

        let meta_row: Element<'a, Message> = if let Some(meta_text) = message_meta {
            let meta_view = iced_text(meta_text)
                .size(MESSAGE_META_TEXT_SIZE)
                .font(chat_text_font())
                .style(move |theme: &Theme| iced::widget::text::Style {
                    color: Some(message_meta_text_color(theme, is_user))
                });
            let approval_badge: Option<Element<'a, Message>> = permission_badge
                .clone()
                .map(|(label, on_press)| approval_badge_view(label, on_press));

            if is_user {
                let mut row = row![container(Space::new()).width(Length::Fill), meta_view]
                    .spacing(8)
                    .align_y(Alignment::Center);
                if let Some(approval_badge) = approval_badge {
                    row = row.push(approval_badge);
                }
                if can_branch_or_reset {
                    row = row.push(icon_action_btn(
                        Icon::GitBranch,
                        "分叉到新会话",
                        fork_press.clone(),
                        false,
                        false,
                    ));
                    row = row.push(icon_action_btn(
                        Icon::ArrowCounterClockwise,
                        "重置到此点",
                        reset_press.clone(),
                        false,
                        false,
                    ));
                }
                row.push(icon_action_btn(
                    copy_icon,
                    copy_tip,
                    copy_message.clone(),
                    recently_copied,
                    true,
                ))
                .into()
            } else {
                let mut row = row![meta_view].spacing(8).align_y(Alignment::Center);
                if let Some(approval_badge) = approval_badge {
                    row = row.push(approval_badge);
                }
                if can_branch_or_reset {
                    row = row.push(icon_action_btn(
                        Icon::GitBranch,
                        "分叉到新会话",
                        fork_press.clone(),
                        false,
                        false,
                    ));
                    row = row.push(icon_action_btn(
                        Icon::ArrowCounterClockwise,
                        "重置到此点",
                        reset_press.clone(),
                        false,
                        false,
                    ));
                }
                row.push(icon_action_btn(
                    copy_icon,
                    copy_tip,
                    copy_message.clone(),
                    recently_copied,
                    true,
                ))
                .into()
            }
        } else if is_user {
            let mut row = row![container(Space::new()).width(Length::Fill)]
                .spacing(8)
                .align_y(Alignment::Center);
            if let Some((label, on_press)) = permission_badge.clone() {
                row = row.push(approval_badge_view(label, on_press));
            }
            if can_branch_or_reset {
                row = row.push(icon_action_btn(
                    Icon::GitBranch,
                    "分叉到新会话",
                    fork_press.clone(),
                    false,
                    false,
                ));
                row = row.push(icon_action_btn(
                    Icon::ArrowCounterClockwise,
                    "重置到此点",
                    reset_press.clone(),
                    false,
                    false,
                ));
            }
            row.push(icon_action_btn(
                copy_icon,
                copy_tip,
                copy_message.clone(),
                recently_copied,
                true,
            ))
            .into()
        } else {
            let mut row = row![].spacing(8).align_y(Alignment::Center);
            if let Some((label, on_press)) = permission_badge.clone() {
                row = row.push(approval_badge_view(label, on_press));
            }
            if can_branch_or_reset {
                row = row.push(icon_action_btn(
                    Icon::GitBranch,
                    "分叉到新会话",
                    fork_press.clone(),
                    false,
                    false,
                ));
                row = row.push(icon_action_btn(
                    Icon::ArrowCounterClockwise,
                    "重置到此点",
                    reset_press.clone(),
                    false,
                    false,
                ));
            }
            row.push(icon_action_btn(
                copy_icon,
                copy_tip,
                copy_message.clone(),
                recently_copied,
                true,
            ))
            .into()
        };

        footer_col = footer_col.push(meta_row);
        footer_col.into()
    };

    let footer_container = container(footer_element)
        .padding([2, 8])
        .width(if is_user { Length::FillPortion(7) } else { Length::Fill })
        .align_x(if is_user {
            iced::alignment::Horizontal::Right
        } else {
            iced::alignment::Horizontal::Left
        });

    if is_user {
        column![
            row![container(Space::new()).width(Length::FillPortion(3)), bubble_inner]
                .align_y(Alignment::End),
            row![container(Space::new()).width(Length::FillPortion(3)), footer_container]
                .align_y(Alignment::Center)
        ]
        .spacing(4)
        .into()
    } else {
        column![
            row![bubble_inner].align_y(Alignment::Start),
            row![footer_container].align_y(Alignment::Center)
        ]
        .spacing(4)
        .into()
    }
}
