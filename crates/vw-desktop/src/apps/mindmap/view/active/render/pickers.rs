//! 思维导图拾取器渲染逻辑，负责颜色、主题和节点样式选择控件。

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::views::design::models::ColorFormat;
use crate::app::views::design::properties::color_picker::render_mini_color_picker;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{EdgeStyle, MindMapColorTarget, MindMapTab};
use iced::widget::svg;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Renderer, Theme};

use super::super::super::common::{ideal_text_color, priority_color};
use super::super::previews::{BorderStylePreview, LineStylePreview};

/// 构建或更新 color picker overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn color_picker_overlay(
    title: &'static str,
    target: MindMapColorTarget,
    active_edge_style: EdgeStyle,
    active_node_border_style: EdgeStyle,
    active_picker_color: Color,
    active_picker_format: ColorFormat,
    active_picker_picking: bool,
) -> Element<'static, Message> {
    fn style_btn<'a>(
        preview: Element<'a, Message, Theme, Renderer>,
        active: bool,
        on: Message,
    ) -> Element<'a, Message, Theme, Renderer> {
        button(
            container(preview)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .style(move |theme: &Theme, _status| {
            let palette = theme.extended_palette();
            let (bg, txt) = if active {
                (palette.primary.base.color, palette.primary.strong.text)
            } else {
                (iced::color!(0xf0, 0xf0, 0xf0), palette.background.strong.text)
            };

            button::Style {
                background: Some(bg.into()),
                text_color: txt,
                border: Border { radius: 20.0.into(), width: 0.0, color: Color::TRANSPARENT },
                ..button::Style::default()
            }
        })
        .padding([6, 0])
        .width(Length::FillPortion(1))
        .on_press(on)
        .into()
    }

    let preview_color = |active: bool| {
        if active {
            Color::from_rgba8(255, 255, 255, 0.92)
        } else {
            Color::from_rgba8(0, 0, 0, 0.68)
        }
    };

    let style_row: Option<Element<'_, Message>> = match target {
        MindMapColorTarget::EdgeStroke => Some(
            container(
                row![
                    style_btn(
                        iced::widget::canvas(LineStylePreview {
                            style: EdgeStyle::Solid,
                            color: preview_color(active_edge_style == EdgeStyle::Solid),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_edge_style == EdgeStyle::Solid,
                        Message::MindMapTool(MindMapMessage::SetEdgeStyle(EdgeStyle::Solid)),
                    ),
                    style_btn(
                        iced::widget::canvas(LineStylePreview {
                            style: EdgeStyle::Dashed,
                            color: preview_color(active_edge_style == EdgeStyle::Dashed),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_edge_style == EdgeStyle::Dashed,
                        Message::MindMapTool(MindMapMessage::SetEdgeStyle(EdgeStyle::Dashed)),
                    ),
                    style_btn(
                        iced::widget::canvas(LineStylePreview {
                            style: EdgeStyle::Dotted,
                            color: preview_color(active_edge_style == EdgeStyle::Dotted),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_edge_style == EdgeStyle::Dotted,
                        Message::MindMapTool(MindMapMessage::SetEdgeStyle(EdgeStyle::Dotted)),
                    ),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into(),
        ),
        MindMapColorTarget::NodeBorder => Some(
            container(
                row![
                    style_btn(
                        iced::widget::canvas(BorderStylePreview {
                            style: EdgeStyle::Solid,
                            color: preview_color(active_node_border_style == EdgeStyle::Solid),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_node_border_style == EdgeStyle::Solid,
                        Message::MindMapTool(MindMapMessage::SetNodeBorderStyle(EdgeStyle::Solid)),
                    ),
                    style_btn(
                        iced::widget::canvas(BorderStylePreview {
                            style: EdgeStyle::Dashed,
                            color: preview_color(active_node_border_style == EdgeStyle::Dashed),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_node_border_style == EdgeStyle::Dashed,
                        Message::MindMapTool(MindMapMessage::SetNodeBorderStyle(EdgeStyle::Dashed)),
                    ),
                    style_btn(
                        iced::widget::canvas(BorderStylePreview {
                            style: EdgeStyle::Dotted,
                            color: preview_color(active_node_border_style == EdgeStyle::Dotted),
                        })
                        .width(Length::Fill)
                        .height(Length::Fixed(16.0))
                        .into(),
                        active_node_border_style == EdgeStyle::Dotted,
                        Message::MindMapTool(MindMapMessage::SetNodeBorderStyle(EdgeStyle::Dotted)),
                    ),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into(),
        ),
        _ => None,
    };

    container(
        {
            let mut content = column![
                row![
                    text(title).size(13),
                    Space::new().width(Length::Fill),
                    button(
                        svg(assets::get_icon(Icon::ArrowCounterClockwise))
                            .width(Length::Fixed(12.0))
                            .height(Length::Fixed(12.0))
                            .content_fit(iced::ContentFit::Contain)
                    )
                    .style(button::text)
                    .on_press(Message::MindMapTool(MindMapMessage::ResetColorTarget(target)))
                    .padding(4)
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
            ];
            if let Some(row) = style_row {
                content = content.push(row);
            }
            content.push(render_mini_color_picker(
                active_picker_color,
                active_picker_format,
                active_picker_picking,
                move |c| Message::MindMapTool(MindMapMessage::ColorPickerChanged(c)),
                move |f| Message::MindMapTool(MindMapMessage::ColorPickerFormatChanged(f)),
                || Message::None,
            ))
        }
        .spacing(10)
        .padding(12),
    )
    .width(Length::Fixed(220.0))
    .style(|theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(theme.extended_palette().background.base.color)),
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.weak.color,
            radius: 10.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// 构建或更新 priority picker overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn priority_picker_overlay(current_priority: Option<u8>) -> Element<'static, Message> {
    let circle_btn = |level: u8| -> Element<'static, Message> {
        let bg = priority_color(level);
        let txt = ideal_text_color(bg);
        let active = current_priority == Some(level);

        let circle = container(text(level.to_string()).size(12).color(txt))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    width: if active { 2.0 } else { 1.0 },
                    color: Color::from_rgba8(0, 0, 0, if active { 0.35 } else { 0.15 }),
                    radius: 999.0.into(),
                },
                ..Default::default()
            });

        button(circle)
            .on_press(Message::MindMapTool(MindMapMessage::SetNodePriority(level)))
            .padding(2)
            .style(button::text)
            .into()
    };

    let clear_priority_btn: Element<'static, Message> = {
        let icon = svg(assets::get_icon(Icon::ArrowCounterClockwise))
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(12.0))
            .content_fit(iced::ContentFit::Contain);

        let base = button(icon).style(button::text).padding(4);

        if current_priority.is_some() {
            base.on_press(Message::MindMapTool(MindMapMessage::ClearNodePriority)).into()
        } else {
            base.into()
        }
    };

    let completed_btn: Element<'static, Message> = {
        let icon = svg(assets::get_icon(Icon::Check))
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .content_fit(iced::ContentFit::Contain)
            .style(|_theme: &Theme, _| iced::widget::svg::Style { color: Some(Color::WHITE) });

        let bg = priority_color(10);
        let active = current_priority == Some(10);

        let circle = container(icon)
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    width: if active { 2.0 } else { 1.0 },
                    color: Color::from_rgba8(0, 0, 0, if active { 0.35 } else { 0.15 }),
                    radius: 999.0.into(),
                },
                ..Default::default()
            });

        button(circle)
            .on_press(Message::MindMapTool(MindMapMessage::SetNodePriority(10)))
            .padding(2)
            .style(button::text)
            .into()
    };

    container(
        column![
            row![text("优先级").size(13), completed_btn, clear_priority_btn]
                .align_y(Alignment::Center)
                .spacing(8),
            row![circle_btn(1), circle_btn(2), circle_btn(3)].spacing(6).align_y(Alignment::Center),
            row![circle_btn(4), circle_btn(5), circle_btn(6)].spacing(6).align_y(Alignment::Center),
            row![circle_btn(7), circle_btn(8), circle_btn(9)].spacing(6).align_y(Alignment::Center),
        ]
        .spacing(6)
        .padding(12),
    )
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        let bg = theme.palette().background;
        let bg = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);
        iced::widget::container::Style {
            background: Some(Background::Color(bg)),
            border: Border { width: 1.0, color: p.background.weak.color, radius: 10.0.into() },
            ..Default::default()
        }
    })
    .into()
}

/// 构建或更新 url editor overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn url_editor_overlay(tab: &MindMapTab) -> Element<'_, Message> {
    let delete_button_style = |theme: &Theme, status: iced::widget::button::Status| {
        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        let pressed = matches!(status, iced::widget::button::Status::Pressed);
        let bg = if pressed {
            Some(Color::from_rgba8(0, 0, 0, 0.05))
        } else if hovered {
            Some(Color::from_rgba8(0, 0, 0, 0.03))
        } else {
            None
        };
        iced::widget::button::Style {
            background: bg.map(Background::Color),
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    };

    let url_input_style = |theme: &Theme, _status: iced::widget::text_input::Status| {
        let palette = theme.palette();
        iced::widget::text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
            icon: Color::TRANSPARENT,
            placeholder: palette.text.scale_alpha(0.55),
            value: palette.text,
            selection: palette.primary.scale_alpha(0.30),
        }
    };

    let input_row = container(
        row![
            text_input("输入链接…", &tab.url_editor_value)
                .on_input(|s| Message::MindMapTool(MindMapMessage::NodeUrlChanged(s)))
                .on_submit(Message::MindMapTool(MindMapMessage::SaveNodeUrl))
                .style(url_input_style)
                .padding([7, 10])
                .size(13)
                .width(Length::Fill),
            button(
                svg(assets::get_icon(Icon::Trash))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(iced::ContentFit::Contain),
            )
            .style(delete_button_style)
            .on_press(Message::MindMapTool(MindMapMessage::ClearNodeUrl))
            .padding(8)
        ]
        .align_y(Alignment::Center)
        .spacing(4),
    )
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        let bg = p.background.base.color;
        let bg = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);
        iced::widget::container::Style {
            background: Some(Background::Color(bg)),
            border: Border { width: 1.0, color: p.background.strong.color, radius: 999.0.into() },
            ..Default::default()
        }
    })
    .width(Length::Fill);

    container(input_row)
        .padding(10)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            let bg = p.background.base.color;
            let bg = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);
            iced::widget::container::Style {
                background: Some(Background::Color(bg)),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 16.0.into() },
                ..Default::default()
            }
        })
        .width(Length::Fixed(350.0))
        .into()
}

/// 构建或更新 text editor overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
#[allow(dead_code)]
pub(super) fn text_editor_overlay(tab: &MindMapTab) -> Element<'_, Message> {
    let icon_button_style = |theme: &Theme, status: iced::widget::button::Status| {
        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        let pressed = matches!(status, iced::widget::button::Status::Pressed);
        let bg = if pressed {
            Some(Color::from_rgba8(0, 0, 0, 0.05))
        } else if hovered {
            Some(Color::from_rgba8(0, 0, 0, 0.03))
        } else {
            None
        };
        iced::widget::button::Style {
            background: bg.map(Background::Color),
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 10.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    };

    let input_row = container(
        row![
            container(
                iced::widget::text_editor(&tab.node_text_editor)
                    .placeholder("输入文本…")
                    .on_action(|action| {
                        Message::MindMapTool(MindMapMessage::NodeTextEditorAction(action))
                    })
                    .key_binding(|kp| {
                        let key = kp.key.clone();
                        if matches!(
                            key,
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                        ) {
                            Some(iced::widget::text_editor::Binding::Custom(Message::MindMapTool(
                                MindMapMessage::NodeTextEditorEnter { shift: kp.modifiers.shift() },
                            )))
                        } else {
                            iced::widget::text_editor::Binding::from_key_press(kp)
                        }
                    })
                    .padding([10, 12])
                    .size(13),
            )
            .width(Length::Fill),
            button(
                svg(assets::get_icon(Icon::Check))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(iced::ContentFit::Contain),
            )
            .style(icon_button_style)
            .on_press(Message::MindMapTool(MindMapMessage::SaveNodeText))
            .padding(10)
        ]
        .align_y(Alignment::Center)
        .spacing(6),
    )
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        let bg = p.background.base.color;
        let bg = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);
        iced::widget::container::Style {
            background: Some(Background::Color(bg)),
            border: Border { width: 1.0, color: p.background.strong.color, radius: 14.0.into() },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.08),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        }
    })
    .width(Length::Fill);

    container(input_row)
        .padding(14)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            let bg = p.background.base.color;
            let bg = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);
            iced::widget::container::Style {
                background: Some(Background::Color(bg)),
                border: Border { width: 1.0, color: p.background.weak.color, radius: 16.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.18),
                    offset: iced::Vector::new(0.0, 12.0),
                    blur_radius: 26.0,
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(350.0))
        .into()
}
