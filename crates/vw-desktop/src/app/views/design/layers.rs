use iced::advanced::{Clipboard, Layout, Shell, Widget, mouse, widget};
use iced::widget::{
    MouseArea, Space, button, column, container, row, scrollable, svg, text, text_input,
};
use iced::{Background, Border, Color, Element, Event, Length, Point, Rectangle, Theme, Vector};

use super::models::{DesignDoc, DesignElement};
use super::state::DesignState;
use crate::app::assets::{self, Icon};
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::message::design::{DesignMessage, LayerAction, PageAction};
use crate::app::{App, Message};

fn is_dark_theme(theme: &Theme) -> bool {
    let palette = theme.palette();
    palette.background.r + palette.background.g + palette.background.b < 1.5
}

fn layer_item_hover_bg(theme: &Theme) -> Color {
    let palette = theme.extended_palette();
    if is_dark_theme(theme) {
        iced::Color { a: 0.5, ..palette.background.weak.color }
    } else {
        Color::from_rgb8(240, 240, 240)
    }
}

fn can_switch_page(element: &DesignElement, depth: u16) -> bool {
    depth <= 1
        && !element.children.is_empty()
        && matches!(element.kind.as_str(), "frame" | "group" | "component" | "ref")
}

pub fn render_layers(app: &App) -> Element<'_, Message> {
    if !app.show_layer_panel {
        return Space::new().width(0).into();
    }

    let count = app
        .active_design_state()
        .map(|s| s.doc.top_level_children_count_in_group(s.active_group_id))
        .unwrap_or(0);
    let title = format!("图层 ({})", count);
    let header = row![
        text(title)
            .size(13)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        Space::new().width(Length::Fill)
    ]
    .align_y(iced::Alignment::Center);
    let group_controls = app
        .active_design_state()
        .map(render_group_controls)
        .unwrap_or_else(|| Space::new().height(0).into());

    let content_padding_left = 10.0f32;
    let content_padding_right = 8.0f32;
    let divider_width = 1.0f32;
    let viewport_width =
        (app.layer_panel_width - content_padding_left - content_padding_right - divider_width)
            .max(0.0);
    let (target_width, items): (f32, Vec<Element<'_, Message>>) =
        if let Some(state) = app.active_design_state() {
            let (max_len, max_depth) = state.layer_tree_metrics;
            let char_px = 7.0f32;
            let overlay = 30.0f32;
            let base_left = overlay + 6.0 + 14.0 + 6.0 + 18.0 + 6.0;
            let indent_px = (max_depth as f32) * 12.0;
            let text_px = (max_len as f32) * char_px;
            let actions_px = 64.0f32;
            let target_width =
                (base_left + indent_px + text_px + actions_px + 24.0).max(viewport_width);

            let v = state
                .doc
                .children
                .iter()
                .filter(|child| child.group_id == state.active_group_id)
                .map(|c| {
                    render_layer_item(
                        c,
                        &state.doc,
                        0,
                        state.selected_element_id.as_deref(),
                        &state.expanded_nodes,
                        app.dragging_layer.as_deref(),
                        app.drag_target_layer.as_deref(),
                        app.hovered_layer_id.as_deref(),
                        app.active_layer_menu.as_deref(),
                        app.layer_menu_anchor,
                        target_width,
                    )
                })
                .collect();
            (target_width, v)
        } else {
            (viewport_width, vec![])
        };
    let scroll_direction = if target_width > viewport_width + 0.5 {
        iced::widget::scrollable::Direction::Both {
            vertical: iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
            horizontal: iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
        }
    } else {
        iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
        )
    };

    let inner = if items.is_empty() {
        column(vec![container(text("暂无图层").size(12)).width(Length::Fill).padding(10).into()])
            .width(Length::Fixed(target_width))
    } else {
        column(items).spacing(2).width(Length::Fixed(target_width))
    };

    let base_content: Element<'_, Message> = MouseArea::new(
        scrollable(container(inner).width(Length::Fixed(target_width)).style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let background =
                if is_dark_theme(theme) { palette.background.weak.color } else { Color::WHITE };
            container::Style { background: Some(background.into()), ..Default::default() }
        }))
        .direction(scroll_direction)
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .on_exit(Message::Design(DesignMessage::LayerHoverLeave))
    .into();

    let content: Element<'_, Message> = base_content;

    container(row![
        column![header, group_controls, content]
            .spacing(10)
            .padding(iced::Padding { top: 10.0, right: 8.0, bottom: 10.0, left: 10.0 })
            .width(Length::Fill)
            .height(Length::Fill),
        container(Space::new()).width(Length::Fixed(1.0)).height(Length::Fill).style(
            |theme: &Theme| {
                let divider_color = if is_dark_theme(theme) {
                    Color::from_rgb8(60, 60, 60)
                } else {
                    Color::from_rgb8(224, 224, 224)
                };
                container::Style { background: Some(divider_color.into()), ..Default::default() }
            }
        )
    ])
    .width(Length::Fixed(app.layer_panel_width))
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = is_dark_theme(theme);
        let background = if is_dark { palette.background.base.color } else { Color::WHITE };
        container::Style {
            background: Some(background.into()),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() },
            shadow: iced::Shadow::default(),
            ..Default::default()
        }
    })
    .into()
}

fn render_group_controls<'a>(state: &'a DesignState) -> Element<'a, Message> {
    let page_cards =
        state.doc.groups.iter().fold(column![].spacing(8), |column, group| {
            column.push(render_page_card(state, group))
        });
    let add_page_button = button(
        container(svg(assets::get_icon(Icon::Plus)).width(10).height(10).style(
            |theme: &Theme, _| iced::widget::svg::Style {
                color: Some(theme.palette().text.scale_alpha(0.88)),
            },
        ))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(24.0))
    .height(Length::Fixed(24.0))
    .style(|theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        let is_dark = is_dark_theme(theme);
        let background = match status {
            iced::widget::button::Status::Hovered => {
                if is_dark {
                    palette.background.strong.color.scale_alpha(0.9)
                } else {
                    Color::from_rgb8(229, 231, 235)
                }
            }
            iced::widget::button::Status::Pressed => {
                if is_dark {
                    palette.background.strong.color
                } else {
                    Color::from_rgb8(220, 223, 228)
                }
            }
            _ => {
                if is_dark {
                    palette.background.weak.color
                } else {
                    Color::from_rgb8(243, 244, 246)
                }
            }
        };
        iced::widget::button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.5)
                } else {
                    Color::from_rgb8(229, 231, 235)
                },
                width: 1.0,
                radius: 7.0.into(),
            },
            text_color: theme.palette().text,
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::CreateGroup));

    container(
        column![
            row![
                text("页面").size(12).font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
                Space::new().width(Length::Fill),
                add_page_button
            ]
            .align_y(iced::Alignment::Center),
            container(
                scrollable(container(page_cards).padding([2, 0]).width(Length::Fill))
                    .direction(iced::widget::scrollable::Direction::Vertical(
                        iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4)
                    ))
                    .height(Length::Shrink),
            )
            .max_height(232.0)
            .width(Length::Fill),
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .padding(10)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = is_dark_theme(theme);
        container::Style {
            background: Some(
                if is_dark {
                    palette.background.weak.color
                } else {
                    Color::from_rgb8(248, 248, 248)
                }
                .into(),
            ),
            border: Border {
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.5)
                } else {
                    Color::from_rgb8(230, 230, 230)
                },
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn render_page_card<'a>(
    state: &'a DesignState,
    group: &'a crate::app::views::design::models::DesignGroup,
) -> Element<'a, Message> {
    let is_active = group.id == state.active_group_id;
    let is_menu_open = state.active_page_menu == Some(group.id);
    let is_renaming = state.renaming_page_id == Some(group.id);
    let page_count = state.doc.top_level_children_count_in_group(group.id);

    let card_content: Element<'a, Message> = if is_renaming {
        row![
            text_input("输入页面名称", &state.renaming_page_name)
                .on_input(|value| Message::Design(DesignMessage::PageRenameChanged(value)))
                .on_submit(Message::Design(DesignMessage::PageRenameSubmit))
                .padding([7, 8])
                .size(12)
                .width(Length::Fill),
            page_icon_button(Icon::Check, Message::Design(DesignMessage::PageRenameSubmit)),
            page_icon_button(Icon::X, Message::Design(DesignMessage::PageRenameCancel)),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![
            text(group.name.clone()).size(12).width(Length::Fill).style(move |theme: &Theme| {
                if is_active {
                    text::Style { color: Some(theme.palette().primary) }
                } else {
                    text::Style { color: Some(theme.palette().text) }
                }
            }),
            page_count_badge(page_count, is_active)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let card = container(card_content)
        .width(Length::Fill)
        .padding(if is_renaming {
            iced::Padding { top: 8.0, right: 8.0, bottom: 8.0, left: 8.0 }
        } else {
            iced::Padding { top: 10.0, right: 12.0, bottom: 10.0, left: 12.0 }
        })
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = is_dark_theme(theme);
            let background = if is_active {
                theme.palette().primary.scale_alpha(if is_dark { 0.18 } else { 0.12 })
            } else if is_dark {
                palette.background.base.color
            } else {
                Color::WHITE
            };
            let border_color = if is_active {
                theme.palette().primary.scale_alpha(0.7)
            } else if is_dark {
                palette.background.strong.color.scale_alpha(0.45)
            } else {
                Color::from_rgb8(229, 231, 235)
            };
            container::Style {
                background: Some(Background::Color(background)),
                border: Border { color: border_color, width: 1.0, radius: 12.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(if is_dark { 0.18 } else { 0.06 }),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 10.0,
                },
                ..Default::default()
            }
        });

    let interactive: Element<'a, Message> = if is_renaming {
        card.into()
    } else {
        MouseArea::new(card)
            .on_press(Message::Design(DesignMessage::SetActiveGroup(group.id)))
            .into()
    };

    let interactive: Element<'a, Message> = Element::new(RightClickArea::new(
        interactive,
        Box::new(move |pos: Point| {
            Message::Design(DesignMessage::PageMenuToggle(group.id, pos.x, pos.y))
        }),
    ));

    if is_menu_open {
        PointBelowOverlay::new(interactive, render_page_context_menu(group.id))
            .show(true)
            .anchor(state.page_menu_anchor.unwrap_or(Point::ORIGIN))
            .gap(6.0)
            .on_close(Message::Design(DesignMessage::PageMenuClose))
            .into()
    } else {
        interactive
    }
}

fn render_page_context_menu<'a>(group_id: u32) -> Element<'a, Message> {
    container(
        column![
            page_menu_item("重命名页面", Icon::Pencil, group_id, PageAction::Rename),
            page_menu_item("复制页面", Icon::Copy, group_id, PageAction::Duplicate),
            page_menu_divider(),
            page_menu_item("上移页面", Icon::ArrowUp, group_id, PageAction::MoveUp),
            page_menu_item("下移页面", Icon::ArrowDown, group_id, PageAction::MoveDown),
            page_menu_divider(),
            page_menu_item("删除页面", Icon::Trash, group_id, PageAction::Delete),
        ]
        .spacing(2),
    )
    .width(Length::Fixed(152.0))
    .padding([6, 6])
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(palette.background.base.color)),
            border: Border {
                color: palette.background.weak.color,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.12),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        }
    })
    .into()
}

fn page_count_badge<'a>(count: usize, is_active: bool) -> Element<'a, Message> {
    container(text(count.to_string()).size(11).style(move |theme: &Theme| text::Style {
        color: Some(if is_active {
            theme.palette().primary
        } else {
            theme.palette().text.scale_alpha(0.9)
        }),
    }))
    .padding(iced::Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 })
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let background = if is_active {
            theme.palette().primary.scale_alpha(0.12)
        } else if is_dark_theme(theme) {
            palette.background.strong.color.scale_alpha(0.7)
        } else {
            Color::from_rgb8(243, 244, 246)
        };
        container::Style {
            background: Some(Background::Color(background)),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 999.0.into() },
            ..Default::default()
        }
    })
    .into()
}

fn page_menu_item<'a>(
    label: &'static str,
    icon: Icon,
    group_id: u32,
    action: PageAction,
) -> Element<'a, Message> {
    button(
        row![
            svg(assets::get_icon(icon)).width(12).height(12).style(|theme: &Theme, _| {
                iced::widget::svg::Style { color: Some(theme.palette().text.scale_alpha(0.82)) }
            }),
            text(label).size(12)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([7, 10])
    .style(|theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        let background = match status {
            iced::widget::button::Status::Hovered => Some(palette.background.weak.color),
            iced::widget::button::Status::Pressed => Some(palette.background.strong.color),
            _ => None,
        };
        iced::widget::button::Style {
            background: background.map(Background::Color),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 8.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    })
    .on_press(Message::Design(DesignMessage::PageActionSelected(group_id, action)))
    .into()
}

fn page_menu_divider<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(
                    palette.background.weak.color.scale_alpha(0.32),
                )),
                ..Default::default()
            }
        })
        .padding([2, 10])
        .into()
}

fn page_icon_button<'a>(icon: Icon, message: Message) -> Element<'a, Message> {
    button(
        container(svg(assets::get_icon(icon)).width(12).height(12).style(|theme: &Theme, _| {
            iced::widget::svg::Style { color: Some(theme.palette().text.scale_alpha(0.82)) }
        }))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .width(Length::Fixed(24.0))
    .height(Length::Fixed(24.0))
    .style(|theme: &Theme, status: iced::widget::button::Status| {
        let palette = theme.extended_palette();
        let background = match status {
            iced::widget::button::Status::Hovered => Some(palette.background.weak.color),
            iced::widget::button::Status::Pressed => Some(palette.background.strong.color),
            _ => None,
        };
        iced::widget::button::Style {
            background: background.map(Background::Color),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 6.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    })
    .on_press(message)
    .into()
}

fn render_layer_item<'a>(
    element: &'a DesignElement,
    doc: &'a DesignDoc,
    depth: u16,
    selected_id: Option<&str>,
    expanded_nodes: &'a std::collections::HashSet<String>,
    dragging_id: Option<&str>,
    drag_target_id: Option<&str>,
    hovered_id: Option<&str>,
    active_menu_id: Option<&str>,
    menu_anchor: Option<Point>,
    target_width: f32,
) -> Element<'a, Message> {
    let is_selected = selected_id == Some(&element.id);
    let is_expanded = expanded_nodes.contains(&element.id);
    let is_visible = element.visible.unwrap_or(true);
    let is_dragging = dragging_id == Some(&element.id);
    let is_drag_target = drag_target_id == Some(&element.id);
    let is_hovered = hovered_id == Some(&element.id);
    let is_menu_open = active_menu_id == Some(element.id.as_str());
    let is_handle_hot = is_dragging;

    let mut children = &element.children;

    if element.kind == "ref"
        && let Some(ref_id) = &element.reference
            && let Some(ref_el) = doc.find_element(ref_id) {
                children = &ref_el.children;
            }

    let has_children = !children.is_empty();

    let mut content = row![].align_y(iced::Alignment::Center);

    // Drag handle
    let drag_icon_content =
        container(svg(assets::get_icon(Icon::ArrowsMove)).width(10).height(10).style(
            move |theme: &Theme, _| {
                let alpha = if is_handle_hot { 0.65 } else { 0.32 };
                let base = if is_dark_theme(theme) {
                    Color::from_rgb8(175, 175, 175)
                } else {
                    Color::from_rgb8(135, 135, 135)
                };
                iced::widget::svg::Style { color: Some(base.scale_alpha(alpha)) }
            },
        ))
        .padding(1)
        .style(move |_theme: &Theme| container::Style {
            border: iced::Border { radius: 4.0.into(), ..iced::Border::default() },
            ..Default::default()
        });

    let drag_interactive = MouseArea::new(drag_icon_content)
        .on_press(Message::Design(DesignMessage::LayerDragStart(element.id.clone())));

    let mid_h = 14.0;
    let handle_w = 14.0;
    let handle_center = container(drag_interactive)
        .width(Length::Fixed(handle_w))
        .height(Length::Fixed(mid_h))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    content = content.push(handle_center).push(Space::new().width(6));

    // Indentation
    if depth > 0 {
        content = content.push(Space::new().width(depth as f32 * 12.0));
    }

    // Icon based on type
    let type_icon_code = match element.kind.as_str() {
        "frame" => Icon::LayoutTextWindow,
        "text" | "Typography" => Icon::Type,
        "rect" => Icon::Square,
        "circle" => Icon::Circle,
        "icon_font" => Icon::Star,
        "vector" => Icon::Pen,
        "component" => Icon::Box,
        "ref" => Icon::FileText,
        _ => Icon::Box,
    };
    let type_icon: Element<'a, Message> = svg(assets::get_icon(type_icon_code))
        .width(12)
        .height(12)
        .style(|theme: &Theme, _| iced::widget::svg::Style {
            color: Some(theme.palette().text.scale_alpha(0.7)),
        })
        .into();

    let chevron: Element<'a, Message> = if has_children {
        let icon_code = if is_expanded { Icon::ChevronDown } else { Icon::ChevronRight };
        container(svg(assets::get_icon(icon_code)).width(10).height(10).style(
            |theme: &Theme, _| iced::widget::svg::Style {
                color: Some(theme.palette().text.scale_alpha(0.62)),
            },
        ))
        .width(14)
        .height(14)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|_theme: &Theme| container::Style { ..Default::default() })
        .into()
    } else {
        Space::new().width(14).height(14).into()
    };

    let indicator: Element<'a, Message> = container(type_icon)
        .width(18)
        .height(18)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();

    content = content.push(chevron).push(Space::new().width(6));
    content = content.push(indicator).push(Space::new().width(6));

    // Name
    let display_name =
        element.name.as_deref().filter(|s| !s.is_empty()).unwrap_or(element.kind.as_str());

    content = content.push(
        text(display_name)
            .size(12)
            .wrapping(iced::widget::text::Wrapping::None)
            .width(Length::Fill)
            .style(move |theme: &Theme| {
                if !is_visible {
                    text::Style { color: Some(theme.palette().text.scale_alpha(0.45)) }
                } else if is_selected {
                    text::Style { color: Some(theme.palette().primary) }
                } else {
                    text::Style { color: Some(theme.palette().text) }
                }
            }),
    );

    if can_switch_page(element, depth) {
        let switch_page_btn = button(
            container(svg(assets::get_icon(Icon::Fullscreen)).width(12).height(12).style(
                |theme: &Theme, _| iced::widget::svg::Style {
                    color: Some(theme.palette().text.scale_alpha(0.74)),
                },
            ))
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
        )
        .padding(0)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .style(|theme: &Theme, status: iced::widget::button::Status| {
            let p = theme.extended_palette();
            let bg = match status {
                iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                iced::widget::button::Status::Hovered => Some(p.background.weak.color),
                _ => None,
            };
            iced::widget::button::Style {
                background: bg.map(Background::Color),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .on_press(Message::Design(DesignMessage::FitToElement(element.id.clone())));

        content = content.push(Space::new().width(6)).push(switch_page_btn);
    }

    let row_content: Element<'a, Message> = container(content)
        .width(Length::Fixed(target_width))
        .height(Length::Fixed(30.0))
        .align_y(iced::alignment::Vertical::Center)
        .padding(iced::Padding { top: 4.0, right: 10.0, bottom: 4.0, left: 4.0 })
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let bg = if is_selected || is_hovered {
                layer_item_hover_bg(theme)
            } else {
                iced::Color::TRANSPARENT
            };
            container::Style {
                background: Some(bg.into()),
                border: iced::Border {
                    color: if is_drag_target {
                        palette.primary.base.color
                    } else {
                        iced::Color::TRANSPARENT
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        })
        .into();

    let row_item = MouseArea::new(row_content)
        .on_press(Message::Design(DesignMessage::LayerRowPressed(element.id.clone())))
        .on_enter(Message::Design(DesignMessage::LayerHover(element.id.clone())))
        .into();

    let menu_id = element.id.clone();
    let row_item: Element<'a, Message> = Element::new(RightClickArea::new(
        row_item,
        Box::new(move |pos: Point| {
            Message::Design(DesignMessage::LayerMenuToggle(menu_id.clone(), pos.x, pos.y))
        }),
    ));

    let row_item: Element<'a, Message> = if is_menu_open {
        let is_visible = element.visible.unwrap_or(true);
        let vis_icon = if is_visible { Icon::Eye } else { Icon::EyeSlash };

        let menu_icon_btn = |icon: Icon, msg: DesignMessage| -> Element<'a, Message> {
            let icon_el =
                svg(assets::get_icon(icon)).width(12).height(12).style(|theme: &Theme, _| {
                    iced::widget::svg::Style { color: Some(theme.palette().text.scale_alpha(0.8)) }
                });

            button(
                container(icon_el)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center),
            )
            .padding(0)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(|theme: &Theme, status: iced::widget::button::Status| {
                let p = theme.extended_palette();
                let bg = match status {
                    iced::widget::button::Status::Pressed => Some(p.background.strong.color),
                    iced::widget::button::Status::Hovered => Some(p.background.weak.color),
                    _ => None,
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .on_press(Message::Design(msg))
            .into()
        };

        let id = element.id.clone();
        let overlay = container(
            row![
                menu_icon_btn(
                    vis_icon,
                    DesignMessage::LayerActionSelected(id.clone(), LayerAction::ToggleVisible)
                ),
                menu_icon_btn(
                    Icon::ArrowUp,
                    DesignMessage::LayerActionSelected(id.clone(), LayerAction::MoveUp)
                ),
                menu_icon_btn(
                    Icon::ArrowDown,
                    DesignMessage::LayerActionSelected(id.clone(), LayerAction::MoveDown)
                ),
                menu_icon_btn(
                    Icon::Trash,
                    DesignMessage::LayerActionSelected(id, LayerAction::Delete)
                ),
            ]
            .spacing(3)
            .align_y(iced::Alignment::Center),
        )
        .padding([4, 6])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border { width: 1.0, color: p.background.weak.color, radius: 7.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.10),
                    offset: Vector::new(0.0, 6.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            }
        });

        PointBelowOverlay::new(row_item, overlay)
            .show(true)
            .anchor(menu_anchor.unwrap_or(Point::new(0.0, 0.0)))
            .gap(6.0)
            .on_close(Message::Design(DesignMessage::LayerMenuClose))
            .into()
    } else {
        row_item
    };

    // Recursive children
    let mut col = column![row_item].width(Length::Fixed(target_width));

    if is_expanded && has_children {
        for child in children {
            col = col.push(render_layer_item(
                child,
                doc,
                depth + 1,
                selected_id,
                expanded_nodes,
                dragging_id,
                drag_target_id,
                hovered_id,
                active_menu_id,
                menu_anchor,
                target_width,
            ));
        }
    }

    col.width(Length::Fixed(target_width)).into()
}

struct RightClickArea<'a, Message, ThemeT = Theme, RendererT = iced::Renderer> {
    content: Element<'a, Message, ThemeT, RendererT>,
    on_right_click: Box<dyn Fn(Point) -> Message + 'a>,
}

impl<'a, Message, ThemeT, RendererT> RightClickArea<'a, Message, ThemeT, RendererT> {
    fn new(
        content: Element<'a, Message, ThemeT, RendererT>,
        on_right_click: Box<dyn Fn(Point) -> Message + 'a>,
    ) -> Self {
        Self { content, on_right_click }
    }
}

impl<'a, Message, ThemeT, RendererT> Widget<Message, ThemeT, RendererT>
    for RightClickArea<'a, Message, ThemeT, RendererT>
where
    RendererT: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &RendererT,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut RendererT,
        theme: &ThemeT,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &RendererT,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::ButtonPressed(button)) = event
            && matches!(button, mouse::Button::Right)
            && let Some(pos) = cursor.position()
            && layout.bounds().contains(pos)
        {
            let bounds = layout.bounds();
            let local = Point::new(pos.x - bounds.x, pos.y - bounds.y);
            shell.publish((self.on_right_click)(local));
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &RendererT,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod layers_tests;
