//! 设计器属性面板的局部渲染模块，负责把元素布局或文字状态转换为可编辑控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use super::utils::prop_text_input_style;
use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignElement, VariableDef, parse_val};
use iced::widget::{button, checkbox, column, container, row, svg, text, text_input};
use iced::{Color, Element, Length, Theme};
use std::collections::HashMap;

/// 渲染对应界面。
///
/// # 参数
/// - `element`: 当前视图构建所需的状态、配置或消息。
/// - `variables`: 当前视图构建所需的状态、配置或消息。
/// - `theme_mode`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn render<'a>(
    element: &'a DesignElement,
    variables: &'a HashMap<String, VariableDef>,
    theme_mode: Option<&'a str>,
) -> Element<'a, Message> {
    let id = element.id.clone();

    let text_growth_mode = element.text_growth.clone().unwrap_or_else(|| "auto".to_string());
    let current_fill_width = element.fill_width.unwrap_or(false);
    let current_fill_height = element.fill_height.unwrap_or(false);

    let width_value = parse_val(&element.width, variables, theme_mode).unwrap_or(0.0).to_string();
    let height_value = parse_val(&element.height, variables, theme_mode).unwrap_or(0.0).to_string();

    let width_input: Element<'_, Message> = if text_growth_mode == "auto" {
        row![
            text("宽").size(12).width(15).align_y(iced::alignment::Vertical::Center),
            text_input("", &width_value)
                .style(prop_text_input_style)
                .padding(4)
                .size(12)
                .width(Length::Fill)
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        row![
            text("宽")
                .size(11)
                .width(15)
                .line_height(iced::widget::text::LineHeight::Relative(1.2))
                .style(text::secondary)
                .align_y(iced::alignment::Vertical::Center),
            text_input("", &width_value)
                .on_input({
                    let id = id.clone();
                    move |s| {
                        Message::Design(DesignMessage::PropertyUpdate(
                            id.clone(),
                            "width".to_string(),
                            serde_json::Value::String(s),
                        ))
                    }
                })
                .style(prop_text_input_style)
                .padding(6)
                .size(12)
                .width(Length::Fill)
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let height_input: Element<'_, Message> =
        if text_growth_mode == "fixed-width" || text_growth_mode == "auto" {
            row![
                text("高").size(12).width(15).align_y(iced::alignment::Vertical::Center),
                text_input("", &height_value)
                    .style(prop_text_input_style)
                    .padding(4)
                    .size(12)
                    .width(Length::Fill)
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
            .into()
        } else {
            row![
                text("高")
                    .size(11)
                    .width(15)
                    .line_height(iced::widget::text::LineHeight::Relative(1.2))
                    .style(text::secondary)
                    .align_y(iced::alignment::Vertical::Center),
                text_input("", &height_value)
                    .on_input({
                        let id = id.clone();
                        move |s| {
                            Message::Design(DesignMessage::PropertyUpdate(
                                id.clone(),
                                "height".to_string(),
                                serde_json::Value::String(s),
                            ))
                        }
                    })
                    .style(prop_text_input_style)
                    .padding(6)
                    .size(12)
                    .width(Length::Fill)
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into()
        };

    column![
        text("布局")
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        column![
            text("尺寸")
                .size(12)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            row![
                container(width_input).width(Length::FillPortion(1)),
                container(height_input).width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .width(Length::Fill),
            {
                let fill_width_item: Element<'_, Message> = container(
                    row![
                        text("填满宽度").size(12).width(Length::Fill),
                        checkbox(element.fill_width.unwrap_or(false)).on_toggle({
                            let id = id.clone();
                            let mode = text_growth_mode.clone();
                            let current_fill_height = current_fill_height;
                            move |b| {
                                if b && (mode == "auto" || current_fill_height) {
                                    let next_mode = if current_fill_height {
                                        "fixed-width-height"
                                    } else {
                                        "fixed-width"
                                    };
                                    Message::Design(DesignMessage::PropertiesUpdate(
                                        id.clone(),
                                        vec![
                                            ("fillWidth".to_string(), serde_json::Value::Bool(b)),
                                            (
                                                "textGrowth".to_string(),
                                                serde_json::Value::String(next_mode.to_string()),
                                            ),
                                        ],
                                    ))
                                } else {
                                    Message::Design(DesignMessage::PropertyUpdate(
                                        id.clone(),
                                        "fillWidth".to_string(),
                                        serde_json::Value::Bool(b),
                                    ))
                                }
                            }
                        })
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .into();

                let fill_height_item: Element<'_, Message> = container(
                    row![
                        text("填满高度").size(12).width(Length::Fill),
                        checkbox(element.fill_height.unwrap_or(false)).on_toggle({
                            let id = id.clone();
                            let mode = text_growth_mode.clone();
                            let current_fill_width = current_fill_width;
                            move |b| {
                                if b && (mode == "auto" || mode == "fixed-width") {
                                    Message::Design(DesignMessage::PropertiesUpdate(
                                        id.clone(),
                                        vec![
                                            ("fillHeight".to_string(), serde_json::Value::Bool(b)),
                                            (
                                                "fillWidth".to_string(),
                                                serde_json::Value::Bool(current_fill_width),
                                            ),
                                            (
                                                "textGrowth".to_string(),
                                                serde_json::Value::String(
                                                    "fixed-width-height".to_string(),
                                                ),
                                            ),
                                        ],
                                    ))
                                } else {
                                    Message::Design(DesignMessage::PropertyUpdate(
                                        id.clone(),
                                        "fillHeight".to_string(),
                                        serde_json::Value::Bool(b),
                                    ))
                                }
                            }
                        })
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                )
                .width(Length::FillPortion(1))
                .into();

                row![fill_width_item, fill_height_item].spacing(12).width(Length::Fill)
            },
        ]
        .spacing(8),
        column![
            text("调整大小")
                .size(12)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            {
                let mode = text_growth_mode.clone();

                let segment_btn =
                    |icon: Icon, selected: bool, on_press: Message| -> Element<'_, Message> {
                        let icon = svg::Svg::<iced::Theme>::new(assets::get_icon(icon))
                            .width(18)
                            .height(18);

                        button(icon)
                            .width(Length::FillPortion(1))
                            .height(Length::Fixed(24.0))
                            .padding([1, 0])
                            .style(move |theme: &Theme, status| {
                                let p = theme.palette();
                                let ext = theme.extended_palette();
                                let background = if selected {
                                    Some(ext.background.base.color.into())
                                } else if status == button::Status::Hovered {
                                    Some(ext.background.base.color.scale_alpha(0.60).into())
                                } else {
                                    None
                                };

                                button::Style {
                                    background,
                                    text_color: p.text,
                                    border: iced::Border {
                                        radius: 8.0.into(),
                                        width: 0.0,
                                        color: Color::TRANSPARENT,
                                    },
                                    ..button::Style::default()
                                }
                            })
                            .on_press(on_press)
                            .into()
                    };

                let auto_btn = segment_btn(
                    Icon::Star,
                    mode == "auto",
                    Message::Design(DesignMessage::PropertiesUpdate(
                        id.clone(),
                        vec![
                            ("fillWidth".to_string(), serde_json::Value::Bool(false)),
                            ("fillHeight".to_string(), serde_json::Value::Bool(false)),
                            ("textGrowth".to_string(), serde_json::Value::Null),
                        ],
                    )),
                );

                let width_btn = segment_btn(
                    Icon::Columns,
                    mode == "fixed-width",
                    Message::Design(DesignMessage::PropertiesUpdate(
                        id.clone(),
                        vec![
                            ("fillWidth".to_string(), serde_json::Value::Bool(current_fill_width)),
                            ("fillHeight".to_string(), serde_json::Value::Bool(false)),
                            (
                                "textGrowth".to_string(),
                                serde_json::Value::String("fixed-width".to_string()),
                            ),
                        ],
                    )),
                );

                let size_btn = segment_btn(
                    Icon::Square,
                    mode == "fixed-width-height",
                    Message::Design(DesignMessage::PropertyUpdate(
                        id.clone(),
                        "textGrowth".to_string(),
                        serde_json::Value::String("fixed-width-height".to_string()),
                    )),
                );

                let resizing_control: Element<'_, Message> = container(
                    row![auto_btn, width_btn, size_btn]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::Fixed(26.0)),
                )
                .width(Length::Fill)
                .padding(1)
                .style(|theme: &Theme| {
                    let ext = theme.extended_palette();
                    container::Style {
                        background: Some(ext.background.weak.color.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: ext.background.strong.color,
                        },
                        ..Default::default()
                    }
                })
                .into();

                resizing_control
            }
        ]
        .spacing(8),
    ]
    .spacing(10)
    .into()
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;
