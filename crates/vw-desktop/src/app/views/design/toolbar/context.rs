//! 选中元素后的上下文工具栏。

use iced::widget::{Space, button, container, svg};
use iced::{Color, Element, Length, Theme};

use super::super::models::DesignTool;
use super::super::state::{ContextPopoverType, DesignState};
use super::context_shape::render_shape_popover;
use super::context_style::{
    extract_hex_token, parse_hex_color, render_border_popover, render_fill_popover,
};
use super::context_text::render_text_context_toolbar;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;

pub fn render_context_toolbar(state: &DesignState) -> Option<Element<'static, Message>> {
    let sel_id = state.selected_element_id.as_deref()?;
    if state.active_tool != DesignTool::Move {
        return None;
    }

    let el = state.doc.find_element(sel_id)?;
    if matches!(el.kind.as_str(), "text" | "Typography")
        || el.kind.eq_ignore_ascii_case("sticky_note")
    {
        return render_text_context_toolbar(state, sel_id, el);
    }

    let current_kind = el.kind.to_ascii_lowercase();
    let fill_json = el.fill.as_ref().map(ToString::to_string).unwrap_or_default();
    let stroke_fill = el
        .stroke
        .as_ref()
        .and_then(|stroke| stroke.fill.as_deref())
        .unwrap_or_default()
        .to_string();
    let border_mode = if el.stroke.is_none() || stroke_fill.is_empty() {
        "none"
    } else if stroke_fill.contains("dashArray") {
        "dashed"
    } else {
        "solid"
    };
    let latest_fill_hex = extract_hex_token(&fill_json)
        .or_else(|| extract_hex_token(&stroke_fill))
        .unwrap_or_else(|| "#40E2D0".to_string());
    let latest_fill_color =
        parse_hex_color(&latest_fill_hex).unwrap_or(Color::from_rgba8(64, 226, 208, 1.0));
    let latest_stroke_hex =
        extract_hex_token(&stroke_fill).unwrap_or_else(|| latest_fill_hex.clone());
    let latest_stroke_color = parse_hex_color(&latest_stroke_hex).unwrap_or(latest_fill_color);

    let icon_size = 14;
    let text_color = Color::from_rgba8(237, 239, 244, 1.0);
    let panel_bg = Color::from_rgba8(18, 19, 23, 0.98);
    let panel_border = Color::from_rgba8(255, 255, 255, 0.12);
    let popover_bg = Color::from_rgba8(24, 26, 31, 0.99);
    let popover_border = Color::from_rgba8(255, 255, 255, 0.14);
    let hover_bg = Color::from_rgba8(255, 255, 255, 0.10);
    let pressed_bg = Color::from_rgba8(255, 255, 255, 0.16);
    let active_bg = Color::from_rgba8(117, 94, 255, 0.26);
    let active_border = Color::from_rgba8(142, 122, 255, 0.75);

    let button_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let is_hovered = status == button::Status::Hovered;
            let is_pressed = status == button::Status::Pressed;

            let background = if is_pressed {
                pressed_bg
            } else if is_hovered {
                hover_bg
            } else if active {
                active_bg
            } else {
                Color::TRANSPARENT
            };
            let border = if active {
                iced::Border { radius: 8.0.into(), width: 1.0, color: active_border }
            } else {
                iced::Border { radius: 8.0.into(), width: 1.0, color: Color::TRANSPARENT }
            };

            button::Style {
                background: Some(background.into()),
                text_color,
                border,
                ..Default::default()
            }
        }
    };

    let shape_active = state.context_popover == Some(ContextPopoverType::Shape);
    let fill_active = state.context_popover == Some(ContextPopoverType::Fill);
    let border_active = state.context_popover == Some(ContextPopoverType::Border);
    let chevron_color = text_color.scale_alpha(0.72);

    let shape_btn = button(
        iced::widget::row![
            svg(assets::get_icon(Icon::Copy))
                .width(icon_size)
                .height(icon_size)
                .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) }),
            svg(assets::get_icon(Icon::ChevronDown))
                .width(10)
                .height(10)
                .style(move |_theme: &Theme, _status| svg::Style { color: Some(chevron_color) })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([5, 8])
    .style(button_style(shape_active))
    .on_press(Message::Design(DesignMessage::ToggleContextPopover(Some(
        ContextPopoverType::Shape,
    ))));

    let fill_btn = button(
        iced::widget::row![
            container(Space::new().width(Length::Fixed(12.0)).height(Length::Fixed(12.0))).style(
                move |_theme: &Theme| container::Style {
                    background: Some(latest_fill_color.into()),
                    border: iced::Border {
                        radius: 999.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }
            ),
            svg(assets::get_icon(Icon::ChevronDown))
                .width(10)
                .height(10)
                .style(move |_theme: &Theme, _status| svg::Style { color: Some(chevron_color) })
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center),
    )
    .padding([5, 8])
    .style(button_style(fill_active))
    .on_press(Message::Design(DesignMessage::ToggleContextPopover(Some(ContextPopoverType::Fill))));

    let border_btn =
        button(
            iced::widget::row![
                container(Space::new().width(Length::Fixed(12.0)).height(Length::Fixed(12.0)))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(latest_stroke_color.into()),
                        border: iced::Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }),
                svg(assets::get_icon(Icon::ChevronDown)).width(10).height(10).style(
                    move |_theme: &Theme, _status| svg::Style { color: Some(chevron_color) }
                )
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center),
        )
        .padding([5, 8])
        .style(button_style(border_active))
        .on_press(Message::Design(DesignMessage::ToggleContextPopover(Some(
            ContextPopoverType::Border,
        ))));

    let separator = || {
        container(Space::new().width(Length::Fixed(1.0)).height(Length::Fixed(14.0))).style(
            |_theme: &Theme| container::Style {
                background: Some(Color::from_rgba8(255, 255, 255, 0.20).into()),
                ..Default::default()
            },
        )
    };

    let row = iced::widget::row![shape_btn, separator(), fill_btn, separator(), border_btn]
        .spacing(1)
        .align_y(iced::Alignment::Center);
    let toolbar_row =
        container(row).width(Length::Fill).align_x(iced::alignment::Horizontal::Center);

    let mut col = iced::widget::column![].spacing(4);
    if let Some(popover) = state.context_popover {
        let popover_content: Element<'static, Message> = match popover {
            ContextPopoverType::ToolbarBrush
            | ContextPopoverType::ToolbarShape
            | ContextPopoverType::ToolbarIcon => {
                return None;
            }
            ContextPopoverType::Shape => render_shape_popover(state, &current_kind),
            ContextPopoverType::Fill => render_fill_popover(sel_id, &fill_json, latest_fill_color),
            ContextPopoverType::Border => {
                render_border_popover(sel_id, &stroke_fill, border_mode, latest_stroke_color)
            }
            ContextPopoverType::TextColor => iced::widget::column![].into(),
        };

        let popover_container =
            container(popover_content).padding(3).style(move |_theme: &Theme| container::Style {
                background: Some(popover_bg.into()),
                border: iced::Border { color: popover_border, width: 1.0, radius: 10.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.42),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            });
        col = col.push(popover_container);
    }

    let panel_width = match state.context_popover {
        Some(ContextPopoverType::Shape) => 256.0,
        Some(ContextPopoverType::Fill) => 332.0,
        Some(ContextPopoverType::Border) => 332.0,
        Some(ContextPopoverType::TextColor) => 164.0,
        Some(ContextPopoverType::ToolbarBrush) => 164.0,
        Some(ContextPopoverType::ToolbarShape) => 164.0,
        Some(ContextPopoverType::ToolbarIcon) => 164.0,
        None => 164.0,
    };

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
#[path = "context_tests.rs"]
mod context_tests;
