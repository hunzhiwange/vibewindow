//! 文本类元素的上下文工具栏。

use iced::widget::{Space, button, container, svg, text, text_input};
use iced::{Color, Element, Length, Theme};

use super::context_style::{extract_hex_token, parse_hex_color};
use super::super::models::{ColorPickerTarget, DesignElement};
use super::super::state::{ContextPopoverType, DesignState};
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;

fn parse_font_size_value(value: &Option<serde_json::Value>) -> f32 {
    value
        .as_ref()
        .and_then(|v| {
            v.as_f64()
                .map(|n| n as f32)
                .or_else(|| v.as_i64().map(|n| n as f32))
                .or_else(|| v.as_u64().map(|n| n as f32))
                .or_else(|| v.as_str().and_then(|s| s.parse::<f32>().ok()))
        })
        .unwrap_or(16.0)
}

fn toggle_text_decoration(current: Option<&str>, decoration: &str) -> String {
    let mut parts: Vec<&str> = current.unwrap_or("none").split_whitespace().collect();
    if parts.contains(&decoration) {
        parts.retain(|value| *value != decoration);
    } else {
        parts.push(decoration);
    }
    parts.retain(|value| *value != "none");
    if parts.is_empty() { "none".to_string() } else { parts.join(" ") }
}

pub fn text_context_panel_width(current_font: &str, color_active: bool) -> f32 {
    let font_chars = current_font.chars().count().min(40) as f32;
    let estimated_font_button_width = (font_chars * 7.2) + 34.0;
    let base = if color_active { 362.0 } else { 342.0 };
    let min_width = if color_active { 532.0 } else { 512.0 };
    (base + estimated_font_button_width).max(min_width).min(760.0)
}

pub(super) fn render_text_context_toolbar(
    state: &DesignState,
    sel_id: &str,
    el: &DesignElement,
) -> Option<Element<'static, Message>> {
    let text_color = Color::from_rgba8(237, 239, 244, 1.0);
    let panel_bg = Color::from_rgba8(18, 19, 23, 0.98);
    let panel_border = Color::from_rgba8(255, 255, 255, 0.12);
    let popover_bg = Color::from_rgba8(24, 26, 31, 0.99);
    let popover_border = Color::from_rgba8(255, 255, 255, 0.14);
    let hover_bg = Color::from_rgba8(255, 255, 255, 0.10);
    let pressed_bg = Color::from_rgba8(255, 255, 255, 0.16);
    let active_bg = Color::from_rgba8(117, 94, 255, 0.26);
    let active_border = Color::from_rgba8(142, 122, 255, 0.75);

    let fill_json = el.fill.as_ref().map(ToString::to_string).unwrap_or_default();
    let latest_text_hex = el
        .color
        .clone()
        .or_else(|| extract_hex_token(&fill_json))
        .unwrap_or_else(|| "#111111".to_string());
    let latest_text_color =
        parse_hex_color(&latest_text_hex).unwrap_or(Color::from_rgba8(17, 17, 17, 1.0));
    let current_font = el.font_family.clone().unwrap_or_else(|| "Inter".to_string());
    let current_size = parse_font_size_value(&el.font_size).round();
    let current_weight = el.font_weight.as_ref().and_then(|value| value.as_str()).unwrap_or("400");
    let bold_active = matches!(current_weight, "600" | "700" | "800" | "bold" | "Bold");
    let strike_active = el.text_decoration.as_deref().is_some_and(|value| value.contains("line-through"));
    let underline_active = el.text_decoration.as_deref().is_some_and(|value| value.contains("underline"));
    let align_mode = el.text_align.as_deref().unwrap_or("left");
    let color_active = state.context_popover == Some(ContextPopoverType::TextColor);

    let button_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let is_hovered = status == button::Status::Hovered;
            let is_pressed = status == button::Status::Pressed;
            button::Style {
                background: Some(
                    if is_pressed {
                        pressed_bg
                    } else if is_hovered {
                        hover_bg
                    } else if active {
                        active_bg
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                text_color,
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if active {
                        active_border
                    } else {
                        Color::from_rgba8(255, 255, 255, 0.08)
                    },
                },
                ..Default::default()
            }
        }
    };

    let divider = || {
        container(Space::new().width(Length::Fixed(1.0)).height(Length::Fixed(18.0))).style(
            |_theme: &Theme| container::Style {
                background: Some(Color::from_rgba8(255, 255, 255, 0.20).into()),
                ..Default::default()
            },
        )
    };

    let color_btn = button(
        iced::widget::row![
            container(Space::new().width(Length::Fixed(12.0)).height(Length::Fixed(12.0))).style(
                move |_theme: &Theme| container::Style {
                    background: Some(latest_text_color.into()),
                    border: iced::Border {
                        radius: 999.0.into(),
                        width: 1.0,
                        color: Color::from_rgba8(255, 255, 255, 0.22),
                    },
                    ..Default::default()
                }
            ),
            svg(assets::get_icon(Icon::ChevronDown)).width(10).height(10).style(
                move |_theme: &Theme, _status| {
                    svg::Style { color: Some(text_color.scale_alpha(0.72)) }
                }
            )
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([5, 8])
    .style(button_style(color_active))
    .on_press(Message::Design(DesignMessage::ToggleContextPopover(Some(
        ContextPopoverType::TextColor,
    ))));

    let font_btn = button(
        iced::widget::row![
            text(current_font.clone()).size(13),
            svg(assets::get_icon(Icon::ChevronDown)).width(10).height(10).style(
                move |_theme: &Theme, _status| {
                    svg::Style { color: Some(text_color.scale_alpha(0.72)) }
                }
            )
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([5, 10])
    .style(button_style(false))
    .on_press(Message::Design(DesignMessage::OpenFontPicker(sel_id.to_string(), None)));

    let size_down = button(text("−").size(14))
        .width(Length::Fixed(20.0))
        .padding([1, 2])
        .style(button_style(false))
        .on_press(Message::Design(DesignMessage::PropertyUpdate(
            sel_id.to_string(),
            "fontSize".to_string(),
            serde_json::json!(current_size - 1.0),
        )));
    let size_input_value = format!("{current_size:.0}");
    let size_input = text_input("", &size_input_value)
        .on_input({
            let id = sel_id.to_string();
            move |raw| {
                let value = raw.trim().parse::<f32>().ok();
                if let Some(value) = value {
                    Message::Design(DesignMessage::PropertyUpdate(
                        id.clone(),
                        "fontSize".to_string(),
                        serde_json::json!(value),
                    ))
                } else {
                    Message::None
                }
            }
        })
        .padding([2, 4])
        .size(12)
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fixed(46.0))
        .style(move |_theme: &Theme, _status| iced::widget::text_input::Style {
            background: Color::from_rgba8(255, 255, 255, 0.10).into(),
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.18),
            },
            icon: text_color.scale_alpha(0.75),
            placeholder: text_color.scale_alpha(0.55),
            value: text_color,
            selection: Color::from_rgba8(142, 122, 255, 0.35),
        });
    let size_up = button(text("+").size(13))
        .width(Length::Fixed(20.0))
        .padding([1, 2])
        .style(button_style(false))
        .on_press(Message::Design(DesignMessage::PropertyUpdate(
            sel_id.to_string(),
            "fontSize".to_string(),
            serde_json::json!(current_size + 1.0),
        )));

    let bold_btn = button(
        svg(assets::get_icon(Icon::TypeBold))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(bold_active))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "fontWeight".to_string(),
        serde_json::Value::String(if bold_active { "400" } else { "700" }.to_string()),
    )));

    let strike_btn = button(
        svg(assets::get_icon(Icon::TypeStrikethrough))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(strike_active))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "textDecoration".to_string(),
        serde_json::Value::String(toggle_text_decoration(
            el.text_decoration.as_deref(),
            "line-through",
        )),
    )));

    let underline_btn = button(
        svg(assets::get_icon(Icon::TypeUnderline))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(underline_active))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "textDecoration".to_string(),
        serde_json::Value::String(toggle_text_decoration(
            el.text_decoration.as_deref(),
            "underline",
        )),
    )));

    let align_left_btn = button(
        svg(assets::get_icon(Icon::TextLeft))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(align_mode == "left"))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "textAlign".to_string(),
        serde_json::Value::String("left".to_string()),
    )));
    let align_center_btn = button(
        svg(assets::get_icon(Icon::TextCenter))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(align_mode == "center"))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "textAlign".to_string(),
        serde_json::Value::String("center".to_string()),
    )));
    let align_right_btn = button(
        svg(assets::get_icon(Icon::TextRight))
            .width(14)
            .height(14)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
    )
    .padding([6, 8])
    .style(button_style(align_mode == "right"))
    .on_press(Message::Design(DesignMessage::PropertyUpdate(
        sel_id.to_string(),
        "textAlign".to_string(),
        serde_json::Value::String("right".to_string()),
    )));

    let toolbar_row = container(
        iced::widget::row![
            color_btn,
            divider(),
            font_btn,
            divider(),
            size_down,
            size_input,
            size_up,
            divider(),
            bold_btn,
            strike_btn,
            underline_btn,
            divider(),
            align_left_btn,
            align_center_btn,
            align_right_btn
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Shrink)
    .align_x(iced::alignment::Horizontal::Center);

    let mut col = iced::widget::column![].spacing(4).align_x(iced::alignment::Horizontal::Center);
    if color_active {
        let swatches = vec![
            "#20252B", "#4B5159", "#F84D16", "#FFA640", "#FFD247", "#67D768", "#59D8F5", "#3CA0FF",
            "#7F5BFF", "#F44BD0", "#FFFFFF", "#D0D2D5", "#F3F3F3", "#FFD9D5", "#FFE8C9", "#FFF2D0",
            "#D7FCDD", "#D8FBFF", "#CAE9FF", "#E1D6FF", "#FFD8F7", "#C9B4FF",
        ];
        let mut top_row = iced::widget::row![].spacing(8);
        for hex in swatches.iter().take(11) {
            let color = parse_hex_color(hex).unwrap_or(Color::WHITE);
            let selected = latest_text_hex.eq_ignore_ascii_case(hex);
            top_row = top_row.push(
                button(
                    container(Space::new().width(Length::Fixed(18.0)).height(Length::Fixed(18.0)))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(color.into()),
                            border: iced::Border {
                                radius: 999.0.into(),
                                width: if selected { 2.0 } else { 1.0 },
                                color: if selected {
                                    Color::from_rgba8(117, 94, 255, 1.0)
                                } else {
                                    Color::from_rgba8(255, 255, 255, 0.20)
                                },
                            },
                            ..Default::default()
                        }),
                )
                .width(Length::Fixed(22.0))
                .height(Length::Fixed(22.0))
                .padding(0)
                .style(button::text)
                .on_press(Message::Design(DesignMessage::PropertyUpdate(
                    sel_id.to_string(),
                    "color".to_string(),
                    serde_json::Value::String((*hex).to_string()),
                ))),
            );
        }
        let mut bottom_row = iced::widget::row![].spacing(8);
        for hex in swatches.iter().skip(11) {
            let color = parse_hex_color(hex).unwrap_or(Color::WHITE);
            let selected = latest_text_hex.eq_ignore_ascii_case(hex);
            bottom_row = bottom_row.push(
                button(
                    container(Space::new().width(Length::Fixed(18.0)).height(Length::Fixed(18.0)))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(color.into()),
                            border: iced::Border {
                                radius: 999.0.into(),
                                width: if selected { 2.0 } else { 1.0 },
                                color: if selected {
                                    Color::from_rgba8(117, 94, 255, 1.0)
                                } else {
                                    Color::from_rgba8(255, 255, 255, 0.20)
                                },
                            },
                            ..Default::default()
                        }),
                )
                .width(Length::Fixed(22.0))
                .height(Length::Fixed(22.0))
                .padding(0)
                .style(button::text)
                .on_press(Message::Design(DesignMessage::PropertyUpdate(
                    sel_id.to_string(),
                    "color".to_string(),
                    serde_json::Value::String((*hex).to_string()),
                ))),
            );
        }
        bottom_row = bottom_row.push(
            button(
                container(svg(assets::get_icon(Icon::Palette)).width(14).height(14).style(
                    move |_theme: &Theme, _status| svg::Style { color: Some(Color::WHITE) },
                ))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
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
                latest_text_color,
                ColorPickerTarget::ContextText { element_id: sel_id.to_string() },
                None,
            ))),
        );
        col = col.push(
            container(
                iced::widget::column![
                    top_row.align_y(iced::Alignment::Center),
                    bottom_row.align_y(iced::Alignment::Center)
                ]
                .spacing(8),
            )
            .padding([8, 10])
            .style(move |_theme: &Theme| container::Style {
                background: Some(popover_bg.into()),
                border: iced::Border { color: popover_border, width: 1.0, radius: 10.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.42),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }),
        );
    }

    let panel_width = text_context_panel_width(&current_font, color_active);
    col = col.push(toolbar_row);
    Some(
        container(col)
            .padding(4)
            .width(Length::Fixed(panel_width))
            .style(move |_theme: &Theme| container::Style {
                background: Some(panel_bg.into()),
                border: iced::Border { color: panel_border, width: 1.0, radius: 12.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.48),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            })
            .into(),
    )
}
#[cfg(test)]
#[path = "context_text_tests.rs"]
mod context_text_tests;
