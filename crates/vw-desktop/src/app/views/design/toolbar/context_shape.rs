//! 图形上下文切换弹层。

use iced::widget::{button, container, svg, text};
use iced::{Color, Element, Length, Theme};

use super::super::state::DesignState;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;

pub(super) fn render_shape_popover(
    state: &DesignState,
    current_kind: &str,
) -> Element<'static, Message> {
    let text_color = Color::from_rgba8(237, 239, 244, 1.0);
    let popover_bg = Color::from_rgba8(24, 26, 31, 0.99);
    let popover_border = Color::from_rgba8(255, 255, 255, 0.14);
    let hover_bg = Color::from_rgba8(255, 255, 255, 0.10);
    let active_bg = Color::from_rgba8(117, 94, 255, 0.26);
    let active_border = Color::from_rgba8(142, 122, 255, 0.75);
    let shape_cell_bg = Color::from_rgba8(255, 255, 255, 0.06);
    let shape_cell_bg_hover = Color::from_rgba8(255, 255, 255, 0.12);
    let shape_cell_bg_active = Color::from_rgba8(117, 94, 255, 0.32);
    let shape_cell_border = Color::from_rgba8(255, 255, 255, 0.14);
    let shape_cell_border_active = Color::from_rgba8(142, 122, 255, 0.95);
    let tab_bg = Color::from_rgba8(255, 255, 255, 0.05);
    let tab_border = Color::from_rgba8(255, 255, 255, 0.14);
    let chevron_color = text_color.scale_alpha(0.72);

    let shape_cell_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let is_hovered = status == button::Status::Hovered;
            let is_pressed = status == button::Status::Pressed;
            let background = if active {
                shape_cell_bg_active
            } else if is_pressed || is_hovered {
                shape_cell_bg_hover
            } else {
                shape_cell_bg
            };
            button::Style {
                background: Some(background.into()),
                text_color,
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if active { shape_cell_border_active } else { shape_cell_border },
                },
                ..Default::default()
            }
        }
    };

    let shape_icon = |kind: &'static str| {
        let data = match kind {
            "line" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 18 20 6" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#
            }
            "square" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "plus" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M10 4h4v6h6v4h-6v6h-4v-6H4v-4h6V4Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "rounded" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="5" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "chevron" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h10l4 6-4 6H5l4-6-4-6Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/></svg>"#
            }
            "arrow_left" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 12 10 6v4h10v4H10v4l-6-6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "arrow_right" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20 12 14 6v4H4v4h10v4l6-6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "circle" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><ellipse cx="12" cy="12" rx="8" ry="7" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "diamond" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 4 20 12 12 20 4 12 12 4Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "triangle_up" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 5 20 19H4L12 5Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "triangle_down" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 6h16L12 19 4 6Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "split_rect" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="4" y="6" width="16" height="12" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/><path d="M12 6v12" stroke="white" stroke-width="1.4"/></svg>"#
            }
            "chat_left" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h14v10H10l-4 3v-3H5V6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "chat_right" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h14v10h-1v3l-4-3H5V6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "parallelogram" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M7 6h13l-3 12H4L7 6Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "hexagon" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M8 4h8l4 8-4 8H8l-4-8 4-8Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "octagon" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M9 4h6l5 5v6l-5 5H9l-5-5V9l5-5Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "slanted_r" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M6 6h13l-3 12H3L6 6Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "slanted_l" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h13l3 12H8L5 6Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "cylinder" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><ellipse cx="12" cy="7" rx="6.5" ry="2.8" fill="none" stroke="white" stroke-width="1.6"/><path d="M5.5 7v10c0 1.6 2.9 2.8 6.5 2.8s6.5-1.2 6.5-2.8V7" fill="none" stroke="white" stroke-width="1.6"/><ellipse cx="12" cy="17" rx="6.5" ry="2.8" fill="none" stroke="white" stroke-width="1.6"/></svg>"#
            }
            "file" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M6 4h9l3 3v13H6V4Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/><path d="M15 4v4h4" fill="none" stroke="white" stroke-width="1.5" stroke-linejoin="round"/></svg>"#
            }
            "folder" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 7h6l2 2h8v10H4V7Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "wave_doc" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h14v10c-2.3-1.5-3.7 1.5-6 0-2.3-1.5-3.7 1.5-6 0V6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "stacked_doc" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M7 5h12v11c-1.9-1.2-3.1 1.2-5 0-1.9-1.2-3.1 1.2-5 0V5Z" fill="none" stroke="white" stroke-width="1.5" stroke-linejoin="round"/><path d="M5 8v9c1.9 1.2 3.1-1.2 5 0" fill="none" stroke="white" stroke-width="1.5"/></svg>"#
            }
            "capsule_h" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="4" y="8" width="16" height="8" rx="4" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "capsule_v" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="8" y="4" width="8" height="16" rx="4" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "trapezoid" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M7 6h10l3 12H4L7 6Z" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
            "offpage" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 6h14v8l-7 5-7-5V6Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "manual_input" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M6 7h14l-2 10H4L6 7Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "ring_x" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="7" fill="none" stroke="white" stroke-width="1.7"/><path d="m8.5 8.5 7 7m0-7-7 7" stroke="white" stroke-width="1.5"/></svg>"#
            }
            "crosshair" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="7" fill="none" stroke="white" stroke-width="1.6"/><path d="M12 4v16M4 12h16" stroke="white" stroke-width="1.3"/></svg>"#
            }
            "notch_tl" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 8V4h4m-4 4v12h16V8H8V4" fill="none" stroke="white" stroke-width="1.7"/></svg>"#
            }
            "notch_tr" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20 8V4h-4m4 4v12H4V8h12V4" fill="none" stroke="white" stroke-width="1.7"/></svg>"#
            }
            "notch_bl" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 16v4h4m-4-4V4h16v12H8v4" fill="none" stroke="white" stroke-width="1.7"/></svg>"#
            }
            "notch_br" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20 16v4h-4m4-4V4H4v12h12v4" fill="none" stroke="white" stroke-width="1.7"/></svg>"#
            }
            "pentagon" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="m12 4 7 5-2.7 10H7.7L5 9l7-5Z" fill="none" stroke="white" stroke-width="1.7" stroke-linejoin="round"/></svg>"#
            }
            "star" => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="m12 4 2.3 4.8 5.2.8-3.7 3.8.9 5.2-4.7-2.5-4.7 2.5.9-5.2-3.7-3.8 5.2-.8L12 4Z" fill="none" stroke="white" stroke-width="1.6" stroke-linejoin="round"/></svg>"#
            }
            _ => {
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/></svg>"#
            }
        };

        svg(iced::widget::svg::Handle::from_memory(data.as_bytes().to_vec()))
            .width(18)
            .height(18)
            .style(move |_theme: &Theme, _status| svg::Style { color: Some(text_color) })
    };

    let normalize_shape_kind = |kind: &str| match kind {
        "rect" | "Rectangle" => "rectangle".to_string(),
        "circle" => "ellipse".to_string(),
        "chevron_right" => "chevron".to_string(),
        _ => kind.to_ascii_lowercase(),
    };

    let shape_groups = vec![
        (
            "connector",
            "连接线",
            vec![
                ("line", "line"),
                ("arrow_left", "arrow_left"),
                ("arrow_right", "arrow_right"),
                ("chevron", "chevron"),
            ],
        ),
        (
            "basic",
            "基本图形",
            vec![
                ("square", "rectangle"),
                ("rounded", "rounded"),
                ("circle", "ellipse"),
                ("diamond", "diamond"),
                ("triangle_up", "triangle"),
                ("triangle_down", "triangle_down"),
                ("pentagon", "pentagon"),
                ("hexagon", "hexagon"),
                ("octagon", "octagon"),
                ("capsule_h", "capsule"),
                ("plus", "plus"),
                ("star", "star"),
                ("chat_left", "chat_left"),
                ("chat_right", "chat_right"),
            ],
        ),
        (
            "flow",
            "流程图",
            vec![
                ("parallelogram", "parallelogram"),
                ("slanted_r", "slanted_r"),
                ("slanted_l", "slanted_l"),
                ("cylinder", "cylinder"),
                ("capsule_v", "capsule_v"),
                ("file", "file"),
                ("folder", "folder"),
                ("wave_doc", "wave_doc"),
                ("stacked_doc", "stacked_doc"),
                ("split_rect", "split_rect"),
                ("offpage", "offpage"),
                ("trapezoid", "trapezoid"),
                ("manual_input", "manual_input"),
                ("notch_tl", "notch_tl"),
                ("crosshair", "crosshair"),
                ("ring_x", "ring_x"),
            ],
        ),
    ];

    let group_btn_style = |active: bool| {
        move |_theme: &Theme, status: button::Status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if active {
                        active_bg
                    } else if hovered {
                        hover_bg
                    } else {
                        tab_bg
                    }
                    .into(),
                ),
                text_color,
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if active { active_border } else { tab_border },
                },
                ..Default::default()
            }
        }
    };

    let normalized_current_kind = normalize_shape_kind(current_kind);
    let active_group_key = shape_groups.iter().find_map(|(group_key, _, entries)| {
        entries
            .iter()
            .any(|(_, kind)| normalized_current_kind == normalize_shape_kind(kind))
            .then_some(*group_key)
    });
    let selected_group_key = state
        .context_shape_group_hover
        .as_deref()
        .or(active_group_key)
        .unwrap_or("basic");

    let mut group_list = iced::widget::column![].spacing(6);
    for (group_key, group_name, _) in &shape_groups {
        let group_active = *group_key == selected_group_key;
        let group_btn = button(
            iced::widget::row![
                text(*group_name).size(12),
                svg(assets::get_icon(Icon::ChevronRight)).width(10).height(10).style(
                    move |_theme: &Theme, _status| svg::Style { color: Some(chevron_color) }
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fixed(96.0))
        .padding([6, 8])
        .style(group_btn_style(group_active));

        group_list = group_list.push(
            iced::widget::MouseArea::new(group_btn)
                .on_enter(Message::Design(DesignMessage::ContextShapeGroupHover(Some(
                    (*group_key).to_string(),
                ))))
                .on_press(Message::None),
        );
    }

    let selected_entries = shape_groups
        .iter()
        .find(|(group_key, _, _)| *group_key == selected_group_key)
        .map(|(_, _, entries)| entries.as_slice())
        .unwrap_or(&[]);

    let mut grid = iced::widget::column![].spacing(6);
    for chunk in selected_entries.chunks(4) {
        let mut row = iced::widget::row![].spacing(6);
        for (icon_key, apply_kind) in chunk {
            let active = normalized_current_kind == normalize_shape_kind(apply_kind);
            let kind = (*apply_kind).to_string();
            row = row.push(
                button(container(shape_icon(icon_key)).center_x(Length::Fill))
                    .width(Length::Fixed(38.0))
                    .height(Length::Fixed(30.0))
                    .padding([3, 5])
                    .style(shape_cell_style(active))
                    .on_press(Message::Design(DesignMessage::UpdateContextShape(kind))),
            );
        }
        grid = grid.push(row);
    }

    let right_panel = container(grid).padding(8).style(move |_theme: &Theme| container::Style {
        background: Some(popover_bg.into()),
        border: iced::Border {
            color: popover_border,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(0.35),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    });

    iced::widget::row![group_list, right_panel]
        .spacing(6)
        .align_y(iced::Alignment::Start)
        .into()
}
#[cfg(test)]
#[path = "context_shape_tests.rs"]
mod context_shape_tests;
