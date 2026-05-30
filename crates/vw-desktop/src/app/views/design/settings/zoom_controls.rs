//! 设计器设置视图模块，负责设置面板、快捷键说明与缩放控制的界面组合。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, button, column, container, row, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::message::DesignMessage;

/// 渲染对应界面。
///
/// # 参数
/// - `zoom`: 当前视图构建所需的状态、配置或消息。
/// - `show_menu`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render_zoom_controls(zoom: f32, show_menu: bool) -> Element<'static, Message> {
    let figma_control_bg = Color::from_rgba8(255, 255, 255, 0.96);
    let figma_border = Color::from_rgba8(224, 224, 224, 1.0);
    let figma_hover_bg = Color::from_rgba8(242, 243, 245, 1.0);
    let figma_pressed_bg = Color::from_rgba8(232, 234, 237, 1.0);
    let figma_text = Color::from_rgba8(34, 34, 34, 1.0);

    let figma_menu_bg = Color::from_rgba8(31, 31, 31, 0.97);
    let figma_menu_hover_bg = Color::from_rgba8(51, 51, 51, 1.0);
    let figma_menu_pressed_bg = Color::from_rgba8(61, 61, 61, 1.0);
    let figma_menu_text = Color::from_rgba8(245, 245, 245, 1.0);
    let figma_menu_separator = Color::from_rgba8(255, 255, 255, 0.10);

    let control_h = 30.0;
    let control_btn_w = 30.0;
    let control_label_w = 46.0;
    let divider_w = 1.0;
    let control_radius = 14.0;
    let control_w = control_btn_w * 3.0 + control_label_w + divider_w * 3.0;

    let base_row = row![
        button(
            container(text("-").size(14).line_height(1.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
        .on_press(Message::Design(DesignMessage::ZoomOut))
        .style(move |_theme: &Theme, status: iced::widget::button::Status| {
            iced::widget::button::Style {
                background: match status {
                    iced::widget::button::Status::Pressed => Some(figma_pressed_bg.into()),
                    iced::widget::button::Status::Hovered => Some(figma_hover_bg.into()),
                    _ => None,
                },
                text_color: figma_text,
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: (control_radius - 1.0).into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(control_btn_w))
        .height(Length::Fixed(control_h))
        .padding([0, 0]),
        container(Space::new().width(Length::Fixed(divider_w)).height(Length::Fixed(control_h)))
            .style(move |_theme: &Theme| container::Style {
                background: Some(figma_border.into()),
                ..Default::default()
            }),
        button(
            container(text(format!("{:.0}%", zoom * 100.0)).size(12).line_height(1.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
        .on_press(Message::Design(DesignMessage::ToggleZoomMenu))
        .style(move |_theme: &Theme, status: iced::widget::button::Status| {
            iced::widget::button::Style {
                background: match status {
                    iced::widget::button::Status::Pressed => Some(figma_pressed_bg.into()),
                    iced::widget::button::Status::Hovered => Some(figma_hover_bg.into()),
                    _ => None,
                },
                text_color: figma_text,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
                ..Default::default()
            }
        })
        .width(Length::Fixed(control_label_w))
        .height(Length::Fixed(control_h))
        .padding([0, 0]),
        container(Space::new().width(Length::Fixed(divider_w)).height(Length::Fixed(control_h)))
            .style(move |_theme: &Theme| container::Style {
                background: Some(figma_border.into()),
                ..Default::default()
            }),
        button(
            container(text("+").size(14).line_height(1.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
        .on_press(Message::Design(DesignMessage::ZoomIn))
        .style(move |_theme: &Theme, status: iced::widget::button::Status| {
            iced::widget::button::Style {
                background: match status {
                    iced::widget::button::Status::Pressed => Some(figma_pressed_bg.into()),
                    iced::widget::button::Status::Hovered => Some(figma_hover_bg.into()),
                    _ => None,
                },
                text_color: figma_text,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
                ..Default::default()
            }
        })
        .width(Length::Fixed(control_btn_w))
        .height(Length::Fixed(control_h))
        .padding([0, 0]),
        container(Space::new().width(Length::Fixed(divider_w)).height(Length::Fixed(control_h)))
            .style(move |_theme: &Theme| container::Style {
                background: Some(figma_border.into()),
                ..Default::default()
            }),
        button(
            container(text("?").size(14).line_height(1.0))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
        .on_press(Message::Design(DesignMessage::ToggleShortcuts))
        .style(move |_theme: &Theme, status: iced::widget::button::Status| {
            iced::widget::button::Style {
                background: match status {
                    iced::widget::button::Status::Pressed => Some(figma_pressed_bg.into()),
                    iced::widget::button::Status::Hovered => Some(figma_hover_bg.into()),
                    _ => None,
                },
                text_color: figma_text,
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: (control_radius - 1.0).into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(control_btn_w))
        .height(Length::Fixed(control_h))
        .padding([0, 0]),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    let base = container(base_row).width(Length::Fixed(control_w)).style(move |_theme: &Theme| {
        container::Style {
            background: Some(Background::Color(figma_control_bg)),
            border: Border { width: 1.0, color: figma_border, radius: control_radius.into() },
            shadow: iced::Shadow {
                color: Color::BLACK.scale_alpha(0.10),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    });

    let mut col = column![].spacing(4).align_x(iced::Alignment::Center);

    if show_menu {
        let menu_items = vec![
            ("适应画布", Message::Design(DesignMessage::ZoomFit)),
            ("20%", Message::Design(DesignMessage::ZoomPresetSelected("20%".into()))),
            ("30%", Message::Design(DesignMessage::ZoomPresetSelected("30%".into()))),
            ("50%", Message::Design(DesignMessage::ZoomPresetSelected("50%".into()))),
            ("80%", Message::Design(DesignMessage::ZoomPresetSelected("80%".into()))),
            ("100%", Message::Design(DesignMessage::ZoomPresetSelected("100%".into()))),
            ("200%", Message::Design(DesignMessage::ZoomPresetSelected("200%".into()))),
            ("300%", Message::Design(DesignMessage::ZoomPresetSelected("300%".into()))),
        ];

        let menu_button_style = move |_theme: &Theme, status: iced::widget::button::Status| {
            iced::widget::button::Style {
                background: match status {
                    iced::widget::button::Status::Pressed => Some(figma_menu_pressed_bg.into()),
                    iced::widget::button::Status::Hovered => Some(figma_menu_hover_bg.into()),
                    _ => None,
                },
                text_color: figma_menu_text,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 12.0.into() },
                ..Default::default()
            }
        };

        let mut menu_col = column![].spacing(2);

        for (idx, (label, msg)) in menu_items.into_iter().enumerate() {
            menu_col = menu_col.push(
                button(
                    container(text(label).size(12))
                        .width(Length::Fill)
                        .padding([4, 8])
                        .align_x(iced::alignment::Horizontal::Left),
                )
                .on_press(msg)
                .style(menu_button_style)
                .width(Length::Fill)
                .padding([0, 0]),
            );

            if idx == 0 {
                menu_col = menu_col.push(container(Space::new().height(Length::Fixed(1.0))).style(
                    move |_theme| container::Style {
                        background: Some(figma_menu_separator.into()),
                        ..Default::default()
                    },
                ));
            }
        }

        col = col.push(
            container(menu_col)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(figma_menu_bg.into()),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 14.0.into() },
                    shadow: iced::Shadow {
                        color: Color::BLACK.scale_alpha(0.35),
                        offset: iced::Vector::new(0.0, 10.0),
                        blur_radius: 30.0,
                    },
                    ..Default::default()
                })
                .padding(6)
                .width(Length::Fixed(control_w)),
        );
    }

    col = col.push(base);

    container(col).padding(5).into()
}

#[cfg(test)]
#[path = "zoom_controls_tests.rs"]
mod zoom_controls_tests;
