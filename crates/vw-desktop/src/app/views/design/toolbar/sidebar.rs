//! 左侧主工具栏视图。

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    Space, button, column, container, image, scrollable, slider, svg, text, text_input, tooltip,
};
use iced::{Color, Element, Length, Theme};

use super::super::models::DesignTool;
use super::super::state::ContextPopoverType;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::SideOverlay;
use crate::app::message::DesignMessage;

const ICON_PICKER_RESULT_LIMIT: usize = 50;
const ICON_PICKER_REQUIRE_QUERY_THRESHOLD: usize = 200;

fn icon_display_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_toolbar_hex_color(input: &str) -> Option<Color> {
    let raw = input.trim().trim_start_matches('#');
    let parse = |start| u8::from_str_radix(&raw[start..start + 2], 16).ok();

    match raw.len() {
        6 => Some(Color::from_rgba8(parse(0)?, parse(2)?, parse(4)?, 1.0)),
        8 => Some(Color::from_rgba8(parse(0)?, parse(2)?, parse(4)?, f32::from(parse(6)?) / 255.0)),
        _ => None,
    }
}

/// 渲染设计工具栏
///
/// 创建一个包含所有设计工具和操作按钮的工具栏组件。工具栏采用垂直布局，
/// 显示在视图左侧，提供设计所需的各种工具入口。
pub fn render_toolbar(
    active_tool: DesignTool,
    show_layer_panel: bool,
    show_properties_panel: bool,
    show_variables_panel: bool,
    toolbar_brush_open: bool,
    toolbar_shape_open: bool,
    toolbar_icon_open: bool,
    brush_color_hex: &str,
    brush_width_px: f32,
    icon_filter_query: &str,
    toolbar_icon_family: &str,
    toolbar_icon_name: &str,
    toolbar_icon_family_tab: &str,
    layer_panel_width: f32,
) -> Element<'static, Message> {
    let figma_active_bg = Color::from_rgba8(24, 160, 251, 1.0);
    let figma_active_bg_hover = Color::from_rgba8(13, 145, 237, 1.0);

    let tools = vec![
        (DesignTool::Move, "移动工具 (V)"),
        (DesignTool::Hand, "抓手 (H)"),
        (DesignTool::Rectangle, "形状组"),
        (DesignTool::Icon, "图标"),
        (DesignTool::ImportImage, "导入图片"),
        (DesignTool::ImportFigma, "导入 Figma"),
        (DesignTool::Pen, "画笔 (P)"),
        (DesignTool::Eraser, "橡皮擦"),
        (DesignTool::Text, "文本 (T)"),
        (DesignTool::Frame, "画板 (F)"),
        (DesignTool::StickyNote, "便签"),
    ];

    let shape_tools = vec![
        DesignTool::Line,
        DesignTool::Chevron,
        DesignTool::Rectangle,
        DesignTool::Ellipse,
        DesignTool::Triangle,
        DesignTool::Diamond,
        DesignTool::Pentagon,
        DesignTool::Hexagon,
        DesignTool::Capsule,
        DesignTool::Star,
        DesignTool::Parallelogram,
        DesignTool::Trapezoid,
    ];

    let shape_groups = vec![
        ("连接线", vec![DesignTool::Line, DesignTool::Chevron]),
        (
            "基本图形",
            vec![
                DesignTool::Rectangle,
                DesignTool::Ellipse,
                DesignTool::Triangle,
                DesignTool::Diamond,
                DesignTool::Pentagon,
                DesignTool::Hexagon,
                DesignTool::Capsule,
                DesignTool::Star,
            ],
        ),
        ("流程图", vec![DesignTool::Parallelogram, DesignTool::Trapezoid]),
    ];
    let brush_palette = [
        "#FFFFFF", "#111827", "#EF4444", "#F97316", "#F59E0B", "#22C55E", "#06B6D4", "#3B82F6",
        "#A855F7",
    ];

    let tool_icon = |tool: DesignTool| match tool {
        DesignTool::Move => Icon::Cursor,
        DesignTool::Hand => Icon::HandIndex,
        DesignTool::Line => Icon::Bezier,
        DesignTool::Rectangle => Icon::Square,
        DesignTool::Ellipse => Icon::Circle,
        DesignTool::Triangle => Icon::Triangle,
        DesignTool::Diamond => Icon::Diamond,
        DesignTool::Star => Icon::Star,
        DesignTool::Pentagon => Icon::Pentagon,
        DesignTool::Hexagon => Icon::Hexagon,
        DesignTool::Parallelogram => Icon::Parallelogram,
        DesignTool::Trapezoid => Icon::Trapezoid,
        DesignTool::Chevron => Icon::ChevronRight,
        DesignTool::Capsule => Icon::Capsule,
        DesignTool::Icon => Icon::Star,
        DesignTool::ImportImage => Icon::Image,
        DesignTool::ImportFigma => Icon::Figma,
        DesignTool::Pen => Icon::Pen,
        DesignTool::Eraser => Icon::Eraser,
        DesignTool::Text => Icon::Type,
        DesignTool::Frame => Icon::LayoutTextWindow,
        DesignTool::StickyNote => Icon::FileText,
    };

    let mut col = column![].spacing(4).padding(4);

    let make_tooltip = |content, label: &'static str| {
        tooltip::Tooltip::new(
            content,
            container(text(label).size(12)).padding([6, 8]).style(move |theme: &Theme| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                let tooltip_bg = if is_dark {
                    Color::from_rgba8(48, 48, 52, 0.98)
                } else {
                    Color::from_rgba8(31, 31, 31, 0.95)
                };
                let tooltip_border = if is_dark {
                    Color::from_rgba8(255, 255, 255, 0.14)
                } else {
                    Color::TRANSPARENT
                };
                container::Style {
                    background: Some(tooltip_bg.into()),
                    text_color: Some(Color::WHITE),
                    border: iced::Border {
                        color: tooltip_border,
                        width: if is_dark { 1.0 } else { 0.0 },
                        radius: 6.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::BLACK.scale_alpha(if is_dark { 0.42 } else { 0.35 }),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: if is_dark { 16.0 } else { 12.0 },
                    },
                    ..Default::default()
                }
            }),
            tooltip::Position::Right,
        )
        .gap(8.0)
    };

    for (tool, label) in tools {
        let is_shape_group = tool == DesignTool::Rectangle;
        let is_icon_group = tool == DesignTool::Icon;
        let is_brush_group = tool == DesignTool::Pen;
        let is_active = if is_shape_group {
            shape_tools.contains(&active_tool) || toolbar_shape_open
        } else if is_icon_group {
            tool == active_tool || toolbar_icon_open
        } else if is_brush_group {
            tool == active_tool || toolbar_brush_open
        } else {
            tool == active_tool
        };
        let icon = tool_icon(tool);
        let icon_size = if is_shape_group { 14 } else { 16 };

        let btn_icon: Element<'static, Message> = if is_icon_group {
            let preview_color =
                if is_active { Color::WHITE } else { Color::from_rgba8(43, 48, 56, 1.0) };
            if let Some(handle) =
                assets::get_named_icon_image(toolbar_icon_family, toolbar_icon_name, preview_color)
            {
                container(
                    image(handle)
                        .width(Length::Fixed(icon_size as f32))
                        .height(Length::Fixed(icon_size as f32)),
                )
                .padding(6)
                .into()
            } else {
                container(svg(assets::get_icon(icon)).width(icon_size).height(icon_size).style(
                    move |theme: &Theme, _status| {
                        let palette = theme.palette();
                        svg::Style {
                            color: if is_active { Some(Color::WHITE) } else { Some(palette.text) },
                        }
                    },
                ))
                .padding(6)
                .into()
            }
        } else {
            container(svg(assets::get_icon(icon)).width(icon_size).height(icon_size).style(
                move |theme: &Theme, _status| {
                    let palette = theme.palette();
                    svg::Style {
                        color: if is_active { Some(Color::WHITE) } else { Some(palette.text) },
                    }
                },
            ))
            .padding(6)
            .into()
        };

        let btn = button(btn_icon)
            .style(move |theme: &Theme, status| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                let is_hovered = status == button::Status::Hovered;
                let is_pressed = status == button::Status::Pressed;
                let figma_hover_bg = if is_dark {
                    Color::from_rgba8(255, 255, 255, 0.10)
                } else {
                    Color::from_rgba8(242, 243, 245, 1.0)
                };
                let figma_pressed_bg = if is_dark {
                    Color::from_rgba8(255, 255, 255, 0.16)
                } else {
                    Color::from_rgba8(232, 234, 237, 1.0)
                };
                button::Style {
                    background: if is_active {
                        Some(
                            (if is_hovered || is_pressed {
                                figma_active_bg_hover
                            } else {
                                figma_active_bg
                            })
                            .into(),
                        )
                    } else if is_pressed {
                        Some(figma_pressed_bg.into())
                    } else if is_hovered {
                        Some(figma_hover_bg.into())
                    } else {
                        None
                    },
                    text_color: if is_active { Color::WHITE } else { palette.text },
                    border: iced::Border {
                        radius: 7.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }
            })
            .on_press(if is_shape_group {
                Message::Design(DesignMessage::ToggleContextPopover(Some(
                    ContextPopoverType::ToolbarShape,
                )))
            } else if is_icon_group {
                Message::Design(DesignMessage::ToggleContextPopover(Some(
                    ContextPopoverType::ToolbarIcon,
                )))
            } else if tool == DesignTool::StickyNote {
                Message::Design(DesignMessage::OpenStickyNoteDialog)
            } else {
                Message::Design(DesignMessage::ToolSelected(tool))
            });

        if is_shape_group {
            let btn_with_tooltip = make_tooltip(btn, label);
            let mut shape_panel = column![].spacing(8).padding(10).width(Length::Fixed(220.0));

            for (group_name, items) in &shape_groups {
                let mut row_buttons =
                    iced::widget::row![].spacing(6).align_y(iced::Alignment::Center);
                for shape_tool in items {
                    let shape_active = *shape_tool == active_tool;
                    let shape_icon = tool_icon(*shape_tool);
                    let item_btn = button(
                        container(svg(assets::get_icon(shape_icon)).width(13).height(13).style(
                            move |theme: &Theme, _status| {
                                let palette = theme.palette();
                                svg::Style {
                                    color: if shape_active {
                                        Some(Color::WHITE)
                                    } else {
                                        Some(palette.text)
                                    },
                                }
                            },
                        ))
                        .padding(5),
                    )
                    .style(move |theme: &Theme, status| {
                        let palette = theme.palette();
                        let is_dark =
                            palette.background.r + palette.background.g + palette.background.b
                                < 1.5;
                        let is_hovered = status == button::Status::Hovered;
                        let is_pressed = status == button::Status::Pressed;
                        let hover_bg = if is_dark {
                            Color::from_rgba8(255, 255, 255, 0.10)
                        } else {
                            Color::from_rgba8(242, 243, 245, 1.0)
                        };
                        let pressed_bg = if is_dark {
                            Color::from_rgba8(255, 255, 255, 0.16)
                        } else {
                            Color::from_rgba8(232, 234, 237, 1.0)
                        };
                        button::Style {
                            background: if shape_active {
                                Some(
                                    (if is_hovered || is_pressed {
                                        figma_active_bg_hover
                                    } else {
                                        figma_active_bg
                                    })
                                    .into(),
                                )
                            } else if is_pressed {
                                Some(pressed_bg.into())
                            } else if is_hovered {
                                Some(hover_bg.into())
                            } else {
                                None
                            },
                            text_color: if shape_active { Color::WHITE } else { palette.text },
                            border: iced::Border {
                                radius: 6.0.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            ..Default::default()
                        }
                    })
                    .on_press(Message::Design(DesignMessage::ToolSelected(*shape_tool)));
                    row_buttons = row_buttons.push(item_btn);
                }

                shape_panel = shape_panel.push(
                    column![
                        text(*group_name).size(11).style(move |_theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(Color::from_rgba8(127, 127, 132, 1.0)),
                            }
                        }),
                        row_buttons
                    ]
                    .spacing(6),
                );
            }

            let shape_min_x = if show_layer_panel { 44.0 + layer_panel_width } else { 44.0 };
            let overlay_btn = SideOverlay::new(
                btn_with_tooltip,
                container(shape_panel).style(move |theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    let panel_bg = if is_dark {
                        Color::from_rgba8(32, 35, 39, 0.98)
                    } else {
                        Color::from_rgba8(255, 255, 255, 0.98)
                    };
                    let border_color = if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.12)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.08)
                    };
                    container::Style {
                        background: Some(panel_bg.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: border_color,
                        },
                        shadow: iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.28),
                            offset: iced::Vector::new(0.0, 4.0),
                            blur_radius: 14.0,
                        },
                        ..Default::default()
                    }
                }),
            )
            .show(toolbar_shape_open)
            .gap(0.0)
            .min_x(shape_min_x)
            .snap_within_viewport(true)
            .on_close(Message::Design(DesignMessage::ToggleContextPopover(None)));
            col = col.push(overlay_btn);
        } else if is_brush_group {
            let btn_with_tooltip = make_tooltip(btn, label);
            let swatch_button = |hex: &'static str| {
                let active = brush_color_hex.eq_ignore_ascii_case(hex);
                let swatch_color = parse_toolbar_hex_color(hex).unwrap_or(Color::WHITE);

                button(
                    container(Space::new().width(Length::Fixed(18.0)).height(Length::Fixed(18.0)))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(swatch_color.into()),
                            border: iced::Border {
                                radius: 999.0.into(),
                                width: if active { 2.0 } else { 1.0 },
                                color: if active {
                                    figma_active_bg
                                } else if hex == "#FFFFFF" {
                                    Color::from_rgba8(209, 213, 219, 1.0)
                                } else {
                                    Color::from_rgba8(255, 255, 255, 0.12)
                                },
                            },
                            ..Default::default()
                        }),
                )
                .padding(0)
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .style(move |_theme: &Theme, status| button::Style {
                    background: Some(
                        (if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                            Color::from_rgba8(59, 130, 246, 0.10)
                        } else {
                            Color::TRANSPARENT
                        })
                        .into(),
                    ),
                    border: iced::Border {
                        radius: 999.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                })
                .on_press(Message::Design(DesignMessage::SetBrushColor(hex.to_string())))
            };

            let mut swatches = iced::widget::row![].spacing(8).align_y(iced::Alignment::Center);
            for hex in brush_palette {
                swatches = swatches.push(swatch_button(hex));
            }

            let panel = container(
                iced::widget::row![
                    swatches,
                    Space::new().width(Length::Fixed(10.0)).height(Length::Fixed(1.0)),
                    slider(1.0..=18.0, brush_width_px.clamp(1.0, 18.0), |value| {
                        Message::Design(DesignMessage::SetBrushWidth(value))
                    })
                    .width(Length::Fixed(150.0)),
                    text(format!("{:.0}px", brush_width_px.clamp(1.0, 18.0))).size(12)
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            )
            .padding([10, 12])
            .style(|theme: &Theme| {
                let palette = theme.palette();
                let is_dark =
                    palette.background.r + palette.background.g + palette.background.b < 1.5;
                container::Style {
                    background: Some(
                        if is_dark {
                            Color::from_rgba8(32, 35, 39, 0.98)
                        } else {
                            Color::from_rgba8(255, 255, 255, 0.98)
                        }
                        .into(),
                    ),
                    border: iced::Border {
                        radius: 16.0.into(),
                        width: 1.0,
                        color: if is_dark {
                            Color::from_rgba8(255, 255, 255, 0.12)
                        } else {
                            Color::from_rgba8(0, 0, 0, 0.08)
                        },
                    },
                    shadow: iced::Shadow {
                        color: Color::BLACK.scale_alpha(0.18),
                        offset: iced::Vector::new(0.0, 10.0),
                        blur_radius: 24.0,
                    },
                    ..Default::default()
                }
            });

            let brush_min_x = if show_layer_panel { 44.0 + layer_panel_width } else { 44.0 };
            let overlay_btn = SideOverlay::new(btn_with_tooltip, panel)
                .show(toolbar_brush_open)
                .gap(0.0)
                .min_x(brush_min_x)
                .snap_within_viewport(true)
                .on_close(Message::Design(DesignMessage::ToggleContextPopover(None)));
            col = col.push(overlay_btn);
        } else if is_icon_group {
            let btn_with_tooltip = make_tooltip(btn, label);
            let icon_panel: Element<'static, Message> = if toolbar_icon_open {
                let catalog = assets::named_icon_catalog();
                let active_family = catalog
                    .iter()
                    .find(|entry| entry.family == toolbar_icon_family_tab)
                    .map(|entry| entry.family.as_str())
                    .or_else(|| catalog.first().map(|entry| entry.family.as_str()))
                    .unwrap_or("lucide");
                let query = icon_filter_query.trim().to_ascii_lowercase();

                let input_style = |theme: &Theme, status: text_input::Status| {
                    let palette = theme.palette();
                    let extended = theme.extended_palette();
                    let focused = matches!(status, text_input::Status::Focused { .. });
                    let border_color =
                        if focused { palette.primary } else { extended.background.strong.color };
                    let bg = if focused {
                        extended.background.weak.color
                    } else {
                        extended.background.base.color
                    };
                    iced::widget::text_input::Style {
                        background: iced::Background::Color(bg),
                        border: iced::Border {
                            width: 1.0,
                            color: border_color,
                            radius: 8.0.into(),
                        },
                        icon: palette.text.scale_alpha(0.5),
                        placeholder: palette.text.scale_alpha(0.55),
                        value: palette.text,
                        selection: palette.primary.scale_alpha(0.30),
                    }
                };

                let family_button_style = |active: bool| {
                    move |theme: &Theme, status: button::Status| {
                        let ext = theme.extended_palette();
                        let hovered =
                            matches!(status, button::Status::Hovered | button::Status::Pressed);
                        let background = if active {
                            theme.palette().primary
                        } else if hovered {
                            ext.background.weak.color
                        } else {
                            ext.background.base.color
                        };
                        button::Style {
                            background: Some(background.into()),
                            text_color: if active { Color::WHITE } else { theme.palette().text },
                            border: iced::Border {
                                color: if active {
                                    theme.palette().primary
                                } else {
                                    ext.background.strong.color
                                },
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            ..Default::default()
                        }
                    }
                };

                let icon_button_style = |active: bool| {
                    move |theme: &Theme, status: button::Status| {
                        let ext = theme.extended_palette();
                        let hovered =
                            matches!(status, button::Status::Hovered | button::Status::Pressed);
                        button::Style {
                            background: Some(
                                if active {
                                    theme.palette().primary.scale_alpha(0.18)
                                } else if hovered {
                                    ext.background.weak.color
                                } else {
                                    ext.background.base.color
                                }
                                .into(),
                            ),
                            text_color: theme.palette().text,
                            border: iced::Border {
                                color: if active {
                                    theme.palette().primary
                                } else {
                                    ext.background.strong.color
                                },
                                width: 1.0,
                                radius: 10.0.into(),
                            },
                            ..Default::default()
                        }
                    }
                };

                let mut family_list = column![].spacing(6);
                for entry in catalog {
                    let family_active = entry.family == active_family;
                    family_list = family_list.push(
                        button(text(entry.family.clone()).size(12))
                            .padding([6, 10])
                            .width(Length::Fill)
                            .style(family_button_style(family_active))
                            .on_press(Message::Design(DesignMessage::SetToolbarIconFamilyTab(
                                entry.family.clone(),
                            ))),
                    );
                }

                let active_icons = catalog
                    .iter()
                    .find(|entry| entry.family == active_family)
                    .map(|entry| entry.icons.as_slice())
                    .unwrap_or(&[]);
                let filtered_icons = active_icons
                    .iter()
                    .filter(|name| query.is_empty() || name.contains(&query))
                    .cloned()
                    .collect::<Vec<_>>();
                let require_query =
                    query.is_empty() && active_icons.len() > ICON_PICKER_REQUIRE_QUERY_THRESHOLD;
                let visible_icons = if require_query {
                    Vec::new()
                } else {
                    filtered_icons
                        .iter()
                        .take(ICON_PICKER_RESULT_LIMIT)
                        .cloned()
                        .collect::<Vec<_>>()
                };
                let result_summary = if require_query {
                    format!("{} 个图标，请先输入名称搜索", active_icons.len())
                } else if filtered_icons.len() > visible_icons.len() {
                    format!(
                        "显示前 {} / {} 个结果，请继续输入缩小范围",
                        visible_icons.len(),
                        filtered_icons.len()
                    )
                } else {
                    format!("{} 个图标", filtered_icons.len())
                };

                let tooltip_style = |theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    container::Style {
                        background: Some(
                            if is_dark {
                                Color::from_rgba8(32, 35, 39, 0.98)
                            } else {
                                Color::from_rgba8(255, 255, 255, 0.98)
                            }
                            .into(),
                        ),
                        text_color: Some(theme.palette().text),
                        border: iced::Border {
                            color: if is_dark {
                                Color::from_rgba8(255, 255, 255, 0.12)
                            } else {
                                Color::from_rgba8(0, 0, 0, 0.08)
                            },
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        shadow: iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.16),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 16.0,
                        },
                        ..Default::default()
                    }
                };

                let mut grid = column![].spacing(8);
                for chunk in visible_icons.chunks(4) {
                    let mut card_row = iced::widget::row![].spacing(8);
                    for name in chunk {
                        let selected =
                            active_family == toolbar_icon_family && name == toolbar_icon_name;
                        let icon_color = if selected {
                            figma_active_bg
                        } else {
                            Color::from_rgba8(43, 48, 56, 1.0)
                        };
                        let preview: Element<'static, Message> = if let Some(handle) =
                            assets::get_named_icon_image(active_family, name, icon_color)
                        {
                            image(handle)
                                .width(Length::Fixed(20.0))
                                .height(Length::Fixed(20.0))
                                .into()
                        } else {
                            svg(assets::get_icon(Icon::Star))
                                .width(Length::Fixed(20.0))
                                .height(Length::Fixed(20.0))
                                .style(move |_theme: &Theme, _status| svg::Style {
                                    color: Some(icon_color),
                                })
                                .into()
                        };
                        let label = icon_display_name(name);
                        let item = button(
                            container(
                                text(label.clone())
                                    .size(10)
                                    .width(Length::Fill)
                                    .align_x(iced::alignment::Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .height(Length::Fixed(46.0))
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center),
                        )
                        .width(Length::Fill)
                        .padding([8, 6])
                        .style(icon_button_style(selected))
                        .on_press(Message::Design(
                            DesignMessage::SelectToolbarIcon {
                                family: active_family.to_string(),
                                name: name.clone(),
                            },
                        ));
                        let hover_preview = tooltip::Tooltip::new(
                            item,
                            container(
                                column![
                                    container(preview)
                                        .width(Length::Fixed(36.0))
                                        .height(Length::Fixed(36.0))
                                        .align_x(iced::alignment::Horizontal::Center)
                                        .align_y(iced::alignment::Vertical::Center),
                                    text(label).size(11)
                                ]
                                .spacing(6)
                                .align_x(iced::Alignment::Center),
                            )
                            .padding([8, 10])
                            .style(tooltip_style),
                            tooltip::Position::Right,
                        )
                        .gap(8.0);
                        card_row =
                            card_row.push(container(hover_preview).width(Length::FillPortion(1)));
                    }
                    grid = grid.push(card_row);
                }

                if require_query {
                    grid = grid.push(
                        container(text("图标数量过多，请先输入图标名称搜索").size(12))
                            .width(Length::Fill)
                            .padding([20, 0])
                            .align_x(iced::alignment::Horizontal::Center),
                    );
                } else if filtered_icons.is_empty() {
                    grid = grid.push(
                        container(text("没有匹配图标").size(12))
                            .width(Length::Fill)
                            .padding([20, 0])
                            .align_x(iced::alignment::Horizontal::Center),
                    );
                }

                container(
                    iced::widget::row![
                        container(
                            scrollable(family_list)
                                .direction(Direction::Vertical(
                                    Scrollbar::new().width(4).scroller_width(4)
                                ))
                                .height(Length::Fill)
                        )
                        .width(Length::Fixed(112.0)),
                        column![
                            text_input("搜索图标名...", icon_filter_query)
                                .on_input(|value| Message::Design(DesignMessage::SetIconFilter(
                                    value
                                )))
                                .style(input_style)
                                .padding([6, 8])
                                .size(12),
                            text(result_summary).size(11).style(iced::widget::text::secondary),
                            scrollable(grid)
                                .direction(Direction::Vertical(
                                    Scrollbar::new().width(4).scroller_width(4)
                                ))
                                .height(Length::Fill)
                        ]
                        .spacing(10)
                        .width(Length::Fill)
                    ]
                    .spacing(10),
                )
                .padding(10)
                .width(Length::Fixed(430.0))
                .height(Length::Fixed(320.0))
                .style(move |theme: &Theme| {
                    let palette = theme.palette();
                    let is_dark =
                        palette.background.r + palette.background.g + palette.background.b < 1.5;
                    let panel_bg = if is_dark {
                        Color::from_rgba8(32, 35, 39, 0.98)
                    } else {
                        Color::from_rgba8(255, 255, 255, 0.98)
                    };
                    let border_color = if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.12)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.08)
                    };
                    container::Style {
                        background: Some(panel_bg.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: border_color,
                        },
                        shadow: iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.28),
                            offset: iced::Vector::new(0.0, 4.0),
                            blur_radius: 14.0,
                        },
                        ..Default::default()
                    }
                })
                .into()
            } else {
                Space::new().into()
            };

            let icon_min_x = if show_layer_panel { 44.0 + layer_panel_width } else { 44.0 };
            let overlay_btn = SideOverlay::new(btn_with_tooltip, icon_panel)
                .show(toolbar_icon_open)
                .gap(0.0)
                .min_x(icon_min_x)
                .snap_within_viewport(true)
                .on_close(Message::Design(DesignMessage::ToggleContextPopover(None)));
            col = col.push(overlay_btn);
        } else {
            col = col.push(make_tooltip(btn, label));
        }
    }

    col = col.push(Space::new().height(Length::Fixed(10.0)));

    let actions = vec![
        (
            Message::Design(DesignMessage::ToggleVariables),
            Icon::Sliders,
            "变量",
            show_variables_panel,
        ),
        (Message::Design(DesignMessage::ToggleShortcuts), Icon::Keyboard, "快捷键", false),
        (
            Message::Design(DesignMessage::ToggleLayerPanel),
            if show_layer_panel { Icon::ChevronLeft } else { Icon::LayoutSidebar },
            "图层",
            show_layer_panel,
        ),
        (
            Message::Design(DesignMessage::TogglePropertiesPanel),
            if show_properties_panel { Icon::ChevronRight } else { Icon::LayoutSidebarReverse },
            "属性",
            show_properties_panel,
        ),
        (Message::Design(DesignMessage::ToggleSettings), Icon::Gear, "设置", false),
    ];

    for (msg, icon, label, is_active) in actions {
        let btn = button(
            container(svg(assets::get_icon(icon)).width(14).height(14).style(
                move |theme: &Theme, _status| {
                    let palette = theme.palette();
                    svg::Style { color: Some(if is_active { Color::WHITE } else { palette.text }) }
                },
            ))
            .padding(6),
        )
        .style(move |theme: &Theme, status| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            let is_hovered = status == button::Status::Hovered;
            let is_pressed = status == button::Status::Pressed;
            let figma_hover_bg = if is_dark {
                Color::from_rgba8(255, 255, 255, 0.10)
            } else {
                Color::from_rgba8(242, 243, 245, 1.0)
            };
            let figma_pressed_bg = if is_dark {
                Color::from_rgba8(255, 255, 255, 0.16)
            } else {
                Color::from_rgba8(232, 234, 237, 1.0)
            };
            let active_bg = if is_dark {
                Color::from_rgba8(24, 160, 251, 0.88)
            } else {
                Color::from_rgba8(24, 160, 251, 0.96)
            };
            let active_hover_bg = if is_dark {
                Color::from_rgba8(13, 145, 237, 0.94)
            } else {
                Color::from_rgba8(13, 145, 237, 1.0)
            };
            let active_border = if is_dark {
                Color::from_rgba8(148, 210, 255, 0.70)
            } else {
                Color::from_rgba8(8, 112, 184, 0.72)
            };
            button::Style {
                background: if is_active {
                    Some(
                        (if is_hovered || is_pressed { active_hover_bg } else { active_bg }).into(),
                    )
                } else if is_pressed {
                    Some(figma_pressed_bg.into())
                } else if is_hovered {
                    Some(figma_hover_bg.into())
                } else {
                    None
                },
                text_color: if is_active { Color::WHITE } else { palette.text },
                border: iced::Border {
                    radius: 7.0.into(),
                    width: if is_active { 1.0 } else { 0.0 },
                    color: if is_active { active_border } else { Color::TRANSPARENT },
                },
                shadow: iced::Shadow {
                    color: if is_active {
                        Color::from_rgba8(24, 160, 251, if is_dark { 0.30 } else { 0.18 })
                    } else {
                        Color::TRANSPARENT
                    },
                    offset: iced::Vector::new(0.0, if is_active { 3.0 } else { 0.0 }),
                    blur_radius: if is_active { 10.0 } else { 0.0 },
                },
                ..Default::default()
            }
        })
        .on_press(msg);

        col = col.push(make_tooltip(btn, label));
    }

    container(col)
        .style(move |theme: &Theme| {
            let palette = theme.palette();
            let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.5;
            let figma_panel_bg = if is_dark {
                Color::from_rgba8(30, 32, 36, 0.96)
            } else {
                Color::from_rgba8(255, 255, 255, 0.96)
            };
            let figma_panel_border = if is_dark {
                Color::from_rgba8(255, 255, 255, 0.14)
            } else {
                Color::from_rgba8(224, 224, 224, 1.0)
            };
            container::Style {
                background: Some(figma_panel_bg.into()),
                border: iced::Border { color: figma_panel_border, width: 1.0, radius: 10.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(if is_dark { 0.24 } else { 0.08 }),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 18.0,
                },
                ..Default::default()
            }
        })
        .into()
}
#[cfg(test)]
#[path = "sidebar_tests.rs"]
mod sidebar_tests;
