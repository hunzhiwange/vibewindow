//! 图形上下文样式弹层与颜色辅助工具。

use iced::widget::{Space, button, container, svg, text};
use iced::{Color, Element, Length, Theme};

use super::super::models::ColorPickerTarget;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;

pub(super) fn parse_hex_color(hex: &str) -> Option<Color> {
    let raw = hex.trim().trim_start_matches('#');
    let (r, g, b, a) = match raw.len() {
        6 => {
            let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
            let a = u8::from_str_radix(&raw[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    Some(Color::from_rgba8(r, g, b, (a as f32) / 255.0))
}

pub(super) fn extract_hex_token(input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut token = String::new();
        token.push('#');
        while let Some(next) = chars.peek().copied() {
            if next.is_ascii_hexdigit() && token.len() < 9 {
                token.push(next);
                chars.next();
            } else {
                break;
            }
        }
        if token.len() == 7 || token.len() == 9 {
            return Some(token);
        }
    }
    None
}

pub(super) fn contains_translucent_hex(input: &str) -> bool {
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut token = String::new();
        token.push('#');
        while let Some(next) = chars.peek().copied() {
            if next.is_ascii_hexdigit() && token.len() < 9 {
                token.push(next);
                chars.next();
            } else {
                break;
            }
        }
        if token.len() == 9
            && u8::from_str_radix(&token[7..9], 16).map(|alpha| alpha < 255).unwrap_or(false)
        {
            return true;
        }
    }
    false
}

pub(super) fn render_fill_popover(
    sel_id: &str,
    fill_json: &str,
    latest_fill_color: Color,
) -> Element<'static, Message> {
    let text_color = Color::from_rgba8(237, 239, 244, 1.0);
    let tab_bg = Color::from_rgba8(255, 255, 255, 0.05);
    let tab_bg_active = Color::from_rgba8(117, 94, 255, 0.28);
    let tab_border = Color::from_rgba8(255, 255, 255, 0.14);
    let tab_border_active = Color::from_rgba8(142, 122, 255, 0.96);

    let fill_is_none = fill_json == "null" || fill_json == "[]" || fill_json.is_empty();
    let fill_is_transparent = contains_translucent_hex(fill_json);
    let fill_mode = if fill_is_none {
        "none"
    } else if fill_is_transparent {
        "transparent"
    } else {
        "fill"
    };

    let tab_button_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if active {
                        tab_bg_active
                    } else if hovered {
                        Color::from_rgba8(255, 255, 255, 0.11)
                    } else {
                        tab_bg
                    }
                    .into(),
                ),
                text_color,
                border: iced::Border {
                    radius: 7.0.into(),
                    width: 1.0,
                    color: if active { tab_border_active } else { tab_border },
                },
                ..Default::default()
            }
        }
    };

    let mode_row =
        iced::widget::row![
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::Square)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("填充").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(fill_mode == "fill"))
            .on_press(Message::Design(DesignMessage::UpdateContextFill("填充".to_string()))),
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::Square)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("透明").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(fill_mode == "transparent"))
            .on_press(Message::Design(DesignMessage::UpdateContextFill("透明".to_string()))),
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::Square)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("无填充").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(fill_mode == "none"))
            .on_press(Message::Design(DesignMessage::UpdateContextFill("无填充".to_string())))
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    let fill_colors = vec![
        "#20252B", "#4B5159", "#F84D16", "#FFA640", "#FFD247", "#67D768", "#59D8F5", "#3CA0FF",
        "#7F5BFF", "#F44BD0", "#FFFFFF", "#D0D2D5", "#F3F3F3", "#FFD9D5", "#FFE8C9", "#FFF2D0",
        "#D7FCDD", "#D8FBFF", "#CAE9FF", "#E1D6FF", "#FFD8F7",
    ];

    let mut top_swatches = iced::widget::row![].spacing(6);
    for hex in fill_colors.iter().take(11) {
        let c = parse_hex_color(hex).unwrap_or(Color::WHITE);
        let selected = fill_json.contains(hex);
        top_swatches = top_swatches.push(
            button(
                container(Space::new().width(Length::Fixed(16.0)).height(Length::Fixed(16.0)))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(c.into()),
                        border: iced::Border {
                            radius: 999.0.into(),
                            width: if selected { 2.0 } else { 1.0 },
                            color: if selected {
                                Color::from_rgba8(117, 94, 255, 1.0)
                            } else {
                                Color::from_rgba8(255, 255, 255, 0.18)
                            },
                        },
                        ..Default::default()
                    }),
            )
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .padding(0)
            .style(button::text)
            .on_press(Message::Design(DesignMessage::UpdateContextFill((*hex).to_string()))),
        );
    }

    let wheel_btn = button(
        container(svg(assets::get_icon(Icon::Palette)).width(13).height(13).style(
            move |theme: &Theme, _status| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                svg::Style {
                    color: Some(if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.92)
                    } else {
                        Color::from_rgba8(43, 48, 56, 0.92)
                    }),
                }
            },
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(20.0))
    .height(Length::Fixed(20.0))
    .padding(0)
    .style(move |_theme: &Theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                (if hovered {
                    Color::from_rgba8(255, 255, 255, 0.18)
                } else {
                    Color::from_rgba8(255, 255, 255, 0.10)
                })
                .into(),
            ),
            border: iced::Border {
                radius: 999.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.22),
            },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::OpenColorPicker(
        latest_fill_color,
        ColorPickerTarget::ContextFill { element_id: sel_id.to_string() },
        None,
    )));

    let mut bottom_swatches = iced::widget::row![].spacing(6);
    for hex in fill_colors.iter().skip(11) {
        let c = parse_hex_color(hex).unwrap_or(Color::WHITE);
        let selected = fill_json.contains(hex);
        bottom_swatches = bottom_swatches.push(
            button(
                container(Space::new().width(Length::Fixed(16.0)).height(Length::Fixed(16.0)))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(c.into()),
                        border: iced::Border {
                            radius: 999.0.into(),
                            width: if selected { 2.0 } else { 1.0 },
                            color: if selected {
                                Color::from_rgba8(117, 94, 255, 1.0)
                            } else {
                                Color::from_rgba8(255, 255, 255, 0.18)
                            },
                        },
                        ..Default::default()
                    }),
            )
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .padding(0)
            .style(button::text)
            .on_press(Message::Design(DesignMessage::UpdateContextFill((*hex).to_string()))),
        );
    }
    bottom_swatches = bottom_swatches.push(wheel_btn);

    let swatch_grid = iced::widget::column![
        top_swatches.align_y(iced::Alignment::Center),
        bottom_swatches.align_y(iced::Alignment::Center)
    ]
    .spacing(6);

    iced::widget::column![
        mode_row,
        container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
            |_theme: &Theme| container::Style {
                background: Some(Color::from_rgba8(255, 255, 255, 0.12).into()),
                ..Default::default()
            }
        ),
        swatch_grid
    ]
    .spacing(9)
    .into()
}

pub(super) fn render_border_popover(
    sel_id: &str,
    stroke_fill: &str,
    border_mode: &str,
    latest_stroke_color: Color,
) -> Element<'static, Message> {
    let text_color = Color::from_rgba8(237, 239, 244, 1.0);
    let tab_bg = Color::from_rgba8(255, 255, 255, 0.05);
    let tab_bg_active = Color::from_rgba8(117, 94, 255, 0.28);
    let tab_border = Color::from_rgba8(255, 255, 255, 0.14);
    let tab_border_active = Color::from_rgba8(142, 122, 255, 0.96);

    let tab_button_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if active {
                        tab_bg_active
                    } else if hovered {
                        Color::from_rgba8(255, 255, 255, 0.11)
                    } else {
                        tab_bg
                    }
                    .into(),
                ),
                text_color,
                border: iced::Border {
                    radius: 7.0.into(),
                    width: 1.0,
                    color: if active { tab_border_active } else { tab_border },
                },
                ..Default::default()
            }
        }
    };

    let mode_row =
        iced::widget::row![
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::Square)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("实线").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(border_mode == "solid"))
            .on_press(Message::Design(DesignMessage::UpdateContextBorder("solid".to_string()))),
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::ListUl)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("虚线").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(border_mode == "dashed"))
            .on_press(Message::Design(DesignMessage::UpdateContextBorder("dashed".to_string()))),
            button(
                iced::widget::row![
                    svg(assets::get_icon(Icon::EyeSlash)).width(11).height(11).style(
                        move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }
                    ),
                    text("无边框").size(12)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .padding([4, 9])
            .style(tab_button_style(border_mode == "none"))
            .on_press(Message::Design(DesignMessage::UpdateContextBorder("none".to_string())))
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    let border_colors = vec![
        "#20252B", "#4B5159", "#F84D16", "#FFA640", "#FFD247", "#67D768", "#59D8F5", "#3CA0FF",
        "#7F5BFF", "#F44BD0", "#FFFFFF", "#D0D2D5", "#F3F3F3", "#FFD9D5", "#FFE8C9", "#FFF2D0",
        "#D7FCDD", "#D8FBFF", "#CAE9FF", "#E1D6FF", "#FFD8F7",
    ];

    let mut top_swatches = iced::widget::row![].spacing(6);
    for hex in border_colors.iter().take(11) {
        let c = parse_hex_color(hex).unwrap_or(Color::WHITE);
        let selected = stroke_fill.contains(hex);
        let mode = border_mode.to_string();
        top_swatches = top_swatches.push(
            button(
                container(Space::new().width(Length::Fixed(16.0)).height(Length::Fixed(16.0)))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(c.into()),
                        border: iced::Border {
                            radius: 999.0.into(),
                            width: if selected { 2.0 } else { 1.0 },
                            color: if selected {
                                Color::from_rgba8(117, 94, 255, 1.0)
                            } else {
                                Color::from_rgba8(255, 255, 255, 0.18)
                            },
                        },
                        ..Default::default()
                    }),
            )
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .padding(0)
            .style(button::text)
            .on_press(Message::Design(DesignMessage::UpdateContextBorder(format!(
                "{}|{}",
                mode, hex
            )))),
        );
    }

    let wheel_btn = button(
        container(svg(assets::get_icon(Icon::Palette)).width(13).height(13).style(
            move |theme: &Theme, _status| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                svg::Style {
                    color: Some(if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.92)
                    } else {
                        Color::from_rgba8(43, 48, 56, 0.92)
                    }),
                }
            },
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(20.0))
    .height(Length::Fixed(20.0))
    .padding(0)
    .style(move |_theme: &Theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(
                (if hovered {
                    Color::from_rgba8(255, 255, 255, 0.18)
                } else {
                    Color::from_rgba8(255, 255, 255, 0.10)
                })
                .into(),
            ),
            border: iced::Border {
                radius: 999.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.22),
            },
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::OpenColorPicker(
        latest_stroke_color,
        ColorPickerTarget::ContextBorder { element_id: sel_id.to_string() },
        None,
    )));

    let mut bottom_swatches = iced::widget::row![].spacing(6);
    for hex in border_colors.iter().skip(11) {
        let c = parse_hex_color(hex).unwrap_or(Color::WHITE);
        let selected = stroke_fill.contains(hex);
        let mode = border_mode.to_string();
        bottom_swatches = bottom_swatches.push(
            button(
                container(Space::new().width(Length::Fixed(16.0)).height(Length::Fixed(16.0)))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(c.into()),
                        border: iced::Border {
                            radius: 999.0.into(),
                            width: if selected { 2.0 } else { 1.0 },
                            color: if selected {
                                Color::from_rgba8(117, 94, 255, 1.0)
                            } else {
                                Color::from_rgba8(255, 255, 255, 0.18)
                            },
                        },
                        ..Default::default()
                    }),
            )
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .padding(0)
            .style(button::text)
            .on_press(Message::Design(DesignMessage::UpdateContextBorder(format!(
                "{}|{}",
                mode, hex
            )))),
        );
    }
    bottom_swatches = bottom_swatches.push(wheel_btn);

    let swatch_grid = iced::widget::column![
        top_swatches.align_y(iced::Alignment::Center),
        bottom_swatches.align_y(iced::Alignment::Center)
    ]
    .spacing(6);

    iced::widget::column![
        mode_row,
        container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
            |_theme: &Theme| container::Style {
                background: Some(Color::from_rgba8(255, 255, 255, 0.12).into()),
                ..Default::default()
            }
        ),
        swatch_grid
    ]
    .spacing(8)
    .into()
}
#[cfg(test)]
#[path = "context_style_tests.rs"]
mod context_style_tests;
