//! 设计器便签创建弹窗模块，负责新增便签时的输入与操作控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::StickyNoteKind;
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

/// 渲染对应界面。
///
/// # 参数
/// - `_app`: 当前视图构建所需的状态、配置或消息。
/// - `state`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_sticky_note_create_dialog<'a>(
    _app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    if !state.sticky_note_dialog_open {
        return Space::new().into();
    }

    let header = row![
        column![
            text("创建便签")
                .size(18)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            text("创建前先选择类型，行为与导入图片弹窗一致。")
                .size(13)
                .style(secondary_text_style),
        ]
        .spacing(6),
        Space::new().width(Length::Fill),
        button(svg(assets::get_icon(Icon::X)).width(14).height(14))
            .on_press(Message::Design(DesignMessage::CloseStickyNoteDialog))
            .style(dialog_icon_button_style)
            .padding(6)
    ]
    .align_y(iced::Alignment::Center);

    let kind_cards = StickyNoteKind::ALL.into_iter().fold(column![], |column, kind| {
        column.push(render_kind_card(kind, state.sticky_note_dialog_default_kind == kind))
    });

    let panel = container(
        column![
            header,
            text("便签类型")
                .size(13)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            kind_cards.spacing(10),
            text("选择后会直接在当前可视区域内创建便签。")
                .size(12)
                .style(secondary_hint_style),
            row![
                Space::new().width(Length::Fill),
                button(text("取消").size(13))
                    .on_press(Message::Design(DesignMessage::CloseStickyNoteDialog))
                    .style(dialog_secondary_button_style)
                    .padding([10, 16]),
            ]
            .align_y(iced::Alignment::Center)
        ]
        .spacing(14),
    )
    .padding(20)
    .width(Length::Fixed(560.0))
    .style(dialog_panel_style);

    container(container(panel).center_x(Length::Fill).center_y(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(dialog_backdrop_style)
        .into()
}

fn render_kind_card<'a>(kind: StickyNoteKind, is_default: bool) -> Element<'a, Message> {
    let description = match kind {
        StickyNoteKind::Note => "普通笔记，适合记录页面说明和临时想法。",
        StickyNoteKind::Context => "上下文便签，适合约束、背景和补充条件。",
        StickyNoteKind::Prompt => "提示词便签，仅保存提示内容，不触发运行。",
    };
    let title_color = parse_hex_color(kind.text_color());
    let fill_color = parse_hex_color(kind.fill_color());
    let stroke_color = parse_hex_color(kind.stroke_color());

    let badge_label = if is_default { "默认" } else { "创建" };
    let badge = container(
        text(badge_label).size(12).style(move |_theme: &Theme| iced::widget::text::Style {
            color: Some(title_color),
        }),
    )
    .padding([6, 10])
    .style(move |_theme: &Theme| container::Style {
        background: Some(fill_color.into()),
        border: Border { width: 1.0, color: stroke_color, radius: 999.0.into() },
        ..Default::default()
    });

    button(
        container(
            row![
                column![
                    text(kind.bilingual_label())
                        .size(16)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .style(move |_theme: &Theme| iced::widget::text::Style {
                            color: Some(title_color),
                        }),
                    text(description).size(12).style(secondary_text_style),
                ]
                .spacing(6)
                .width(Length::Fill),
                badge,
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        )
        .padding(16)
        .style(move |_theme: &Theme| container::Style {
            background: Some(fill_color.scale_alpha(0.86).into()),
            border: Border {
                width: if is_default { 1.5 } else { 1.0 },
                color: stroke_color.scale_alpha(if is_default { 1.0 } else { 0.72 }),
                radius: 14.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.06),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 18.0,
            },
            ..Default::default()
        }),
    )
    .on_press(Message::Design(DesignMessage::CreateStickyNote(kind)))
    .style(dialog_card_button_style)
    .width(Length::Fill)
    .into()
}

fn parse_hex_color(hex: &str) -> Color {
    let value = hex.trim().trim_start_matches('#');
    match value.len() {
        6 => {
            let r = u8::from_str_radix(&value[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&value[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&value[4..6], 16).unwrap_or(0);
            Color::from_rgb8(r, g, b)
        }
        8 => {
            let r = u8::from_str_radix(&value[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&value[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&value[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&value[6..8], 16).unwrap_or(255);
            Color::from_rgba8(r, g, b, (a as f32) / 255.0)
        }
        _ => Color::BLACK,
    }
}

fn secondary_text_style(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    iced::widget::text::Style {
        color: Some(if is_dark {
            Color::from_rgba8(190, 194, 201, 1.0)
        } else {
            Color::from_rgba8(108, 112, 120, 1.0)
        }),
    }
}

fn secondary_hint_style(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    iced::widget::text::Style {
        color: Some(if is_dark {
            Color::from_rgba8(156, 160, 168, 1.0)
        } else {
            Color::from_rgba8(123, 126, 133, 1.0)
        }),
    }
}

fn dialog_backdrop_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    container::Style {
        background: Some(
            if is_dark {
                Color::from_rgba8(0, 0, 0, 0.42)
            } else {
                Color::from_rgba8(17, 24, 39, 0.18)
            }
            .into(),
        ),
        ..Default::default()
    }
}

fn dialog_panel_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    container::Style {
        background: Some(
            if is_dark {
                Color::from_rgba8(36, 38, 42, 0.985)
            } else {
                Color::from_rgba8(248, 248, 249, 0.995)
            }
            .into(),
        ),
        text_color: Some(if is_dark {
            Color::from_rgba8(246, 247, 249, 1.0)
        } else {
            Color::from_rgba8(28, 29, 31, 1.0)
        }),
        border: Border {
            width: 1.0,
            color: if is_dark {
                Color::from_rgba8(255, 255, 255, 0.10)
            } else {
                Color::from_rgba8(0, 0, 0, 0.08)
            },
            radius: 14.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(if is_dark { 0.28 } else { 0.15 }),
            offset: iced::Vector::new(0.0, 16.0),
            blur_radius: 42.0,
        },
        ..Default::default()
    }
}

fn dialog_icon_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered => Some(Color::from_rgba8(127, 127, 132, 0.12).into()),
            button::Status::Pressed => Some(Color::from_rgba8(127, 127, 132, 0.18).into()),
            _ => None,
        },
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
        ..Default::default()
    }
}

fn dialog_secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    button::Style {
        background: Some(
            match status {
                button::Status::Hovered => {
                    if is_dark {
                        Color::from_rgba8(71, 74, 80, 1.0)
                    } else {
                        Color::from_rgba8(240, 241, 243, 1.0)
                    }
                }
                button::Status::Pressed => {
                    if is_dark {
                        Color::from_rgba8(78, 82, 88, 1.0)
                    } else {
                        Color::from_rgba8(232, 234, 237, 1.0)
                    }
                }
                _ => {
                    if is_dark {
                        Color::from_rgba8(60, 63, 68, 1.0)
                    } else {
                        Color::from_rgba8(244, 245, 246, 1.0)
                    }
                }
            }
            .into(),
        ),
        text_color: palette.text,
        border: Border {
            width: 1.0,
            color: if is_dark {
                Color::from_rgba8(255, 255, 255, 0.08)
            } else {
                Color::from_rgba8(0, 0, 0, 0.06)
            },
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

fn dialog_card_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
    button::Style {
        background: Some(
            match status {
                button::Status::Hovered => {
                    if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.06)
                    } else {
                        Color::from_rgba8(255, 255, 255, 0.76)
                    }
                }
                button::Status::Pressed => {
                    if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.09)
                    } else {
                        Color::from_rgba8(255, 255, 255, 0.90)
                    }
                }
                _ => Color::TRANSPARENT,
            }
            .into(),
        ),
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 16.0.into() },
        ..Default::default()
    }
}
#[cfg(test)]
#[path = "sticky_note_create_tests.rs"]
mod sticky_note_create_tests;
