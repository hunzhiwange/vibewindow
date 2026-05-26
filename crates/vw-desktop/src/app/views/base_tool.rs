//! 进制转换工具视图模块
//!
//! 本模块提供进制转换器的用户界面组件，支持在不同进制（2-36）之间进行数字转换。
//! 视觉风格与 JSON 工具、系统设置常规页保持一致，采用卡片式布局与设置表单样式。

use crate::app::assets::{self, Icon};
use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_divider,
    settings_muted_text_style, settings_panel, settings_panel_style, settings_pick_list_menu_style,
    settings_pick_list_style, settings_segment_button_style, settings_text_input_style,
    settings_value_badge,
};
use crate::app::message::BaseToolMessage;
use crate::app::{App, Message};
use iced::widget::svg::Svg;
use iced::widget::{
    Space, button, column, container, pick_list, responsive, row, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

/// 快速选择进制列表。
const QUICK_BASES: [u32; 5] = [2, 4, 8, 10, 16];

#[derive(Clone, Copy)]
enum NumberCard {
    Source,
    Target,
}

fn bases_vec() -> Vec<u32> {
    (2..=36).collect::<Vec<u32>>()
}

fn icon_svg(icon: Icon, size: f32) -> Svg<'static> {
    Svg::new(assets::get_icon(icon)).width(Length::Fixed(size)).height(Length::Fixed(size))
}

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("进制转换器").size(20),
            Space::new().width(Length::Fill),
            build_status_badge(app),
        ]
        .align_y(Alignment::Center)
        .spacing(16),
    )
    .padding([18, 20])
    .width(Length::Fill)
    .style(settings_panel_style);

    let workspace = responsive(move |size| build_workspace(app, size));

    let content = column![hero, workspace]
        .spacing(16)
        .padding([18, 24])
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .into()
}

fn build_workspace<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let conversion = build_conversion_panel(app, size);
    let side_panel = build_side_panel(app);

    if size.width >= 1100.0 {
        row![
            container(conversion).width(Length::FillPortion(3)),
            container(side_panel).width(Length::Fixed(320.0)),
        ]
        .spacing(16)
        .align_y(Alignment::Start)
        .into()
    } else {
        column![side_panel, conversion].spacing(16).into()
    }
}

fn build_conversion_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let compact = size.width < 920.0;
    let source = build_number_card(app, NumberCard::Source, compact);
    let target = build_number_card(app, NumberCard::Target, compact);
    let swap = build_swap_bridge();

    let cards: Element<'a, Message> = if size.width >= 1180.0 {
        row![
            container(source).width(Length::Fill),
            container(swap).width(Length::Fixed(150.0)).center_x(Length::Fill),
            container(target).width(Length::Fill),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
        .into()
    } else {
        column![source, container(swap).width(Length::Fill).center_x(Length::Fill), target,]
            .spacing(16)
            .into()
    };

    column![build_section_title("转换表单"), cards].spacing(12).width(Length::Fill).into()
}

fn build_number_card<'a>(app: &'a App, card: NumberCard, compact: bool) -> Element<'a, Message> {
    let (title, description, base, placeholder, metric, value_help, field_help) = match card {
        NumberCard::Source => (
            "源数值",
            "按当前源进制输入待转换的整数。",
            app.base_from,
            "输入待转换数字，例如 1010 / ff / -42",
            format!("{} 字符", app.base_input.chars().count()),
            format!("当前允许：{}", valid_digits_label(app.base_from)),
            "输入时会自动移除当前进制无效字符，保留前导负号。",
        ),
        NumberCard::Target => (
            "转换结果",
            "根据目标进制实时生成结果。",
            app.base_to,
            "转换结果会显示在这里",
            format!("{} 字符", app.base_output.chars().count()),
            format!("输出规则：{}", valid_digits_label(app.base_to)),
            "结果使用大写字母显示，可直接复制到剪贴板。",
        ),
    };

    let selector = build_base_selector(base, matches!(card, NumberCard::Source), compact);
    let field = build_value_field(app, card);

    column![
        row![
            column![
                text(title).size(14),
                text(description).size(12).style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            build_metric_badge(metric),
        ]
        .align_y(Alignment::Center)
        .spacing(12),
        settings_panel(
            column![
                form_row("进制", value_help, selector, compact),
                settings_divider(),
                form_row(
                    if matches!(card, NumberCard::Source) {
                        "原始数字"
                    } else {
                        "结果数字"
                    },
                    field_help,
                    column![field, text(placeholder).size(11).style(settings_muted_text_style),]
                        .spacing(8),
                    compact,
                ),
            ]
            .spacing(0)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_side_panel<'a>(app: &'a App) -> Element<'a, Message> {
    column![
        build_section_title("快捷操作"),
        settings_panel(
            column![
                button(
                    row![icon_svg(Icon::ArrowRepeat, 16.0), text("交换源与目标").size(13)]
                        .spacing(8)
                        .align_y(Alignment::Center),
                )
                .padding([10, 12])
                .width(Length::Fill)
                .on_press(Message::BaseTool(BaseToolMessage::Swap))
                .style(primary_action_btn_style),
                build_copy_button(app).width(Length::Fill),
            ]
            .spacing(10)
        ),
        build_section_title("当前状态"),
        settings_panel(
            column![
                status_row("源进制", format!("{} 进制", app.base_from)),
                status_row("目标进制", format!("{} 进制", app.base_to)),
                status_row("输入长度", format!("{} 字符", app.base_input.chars().count())),
                status_row("输出长度", format!("{} 字符", app.base_output.chars().count())),
                status_row("支持范围", "2 - 36 进制"),
            ]
            .spacing(10)
        ),
        build_section_title("说明"),
        settings_panel(
            column![
                text("支持 2-36 进制整数互转，并允许输入前导负号。")
                    .size(12)
                    .style(settings_muted_text_style),
                text("切换源进制后，会自动移除不符合当前进制的字符，减少无效输入。")
                    .size(12)
                    .style(settings_muted_text_style),
                text("内部按 u128 范围转换，超出范围时会直接提示。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(10)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_base_selector<'a>(selected: u32, is_from: bool, compact: bool) -> Element<'a, Message> {
    let mut quick_row = row![].spacing(8).align_y(Alignment::Center);
    for base in QUICK_BASES {
        quick_row = quick_row.push(build_base_button(base, selected == base, is_from));
    }

    let pick = pick_list(bases_vec(), Some(selected), move |base| {
        if is_from {
            Message::BaseTool(BaseToolMessage::SelectFrom(base))
        } else {
            Message::BaseTool(BaseToolMessage::SelectTo(base))
        }
    })
    .padding([10, 14])
    .text_size(13)
    .style(settings_pick_list_style)
    .menu_style(settings_pick_list_menu_style)
    .width(if compact { Length::Fill } else { Length::Fixed(156.0) });

    if compact {
        column![quick_row, pick].spacing(10).into()
    } else {
        row![quick_row.width(Length::Fill), pick].spacing(12).align_y(Alignment::Center).into()
    }
}

fn build_base_button<'a>(
    base: u32,
    is_active: bool,
    is_from: bool,
) -> iced::widget::Button<'a, Message> {
    let message = if is_from {
        Message::BaseTool(BaseToolMessage::SelectFrom(base))
    } else {
        Message::BaseTool(BaseToolMessage::SelectTo(base))
    };

    button(text(base.to_string()).size(13))
        .padding([8, 12])
        .on_press(message)
        .style(move |theme: &Theme, status| settings_segment_button_style(theme, status, is_active))
}

fn build_value_field<'a>(app: &'a App, card: NumberCard) -> Element<'a, Message> {
    match card {
        NumberCard::Source => text_input("输入待转换数字", &app.base_input)
            .on_input(|value| Message::BaseTool(BaseToolMessage::InputChanged(value)))
            .padding([10, 12])
            .size(13)
            .style(settings_text_input_style)
            .width(Length::Fill)
            .into(),
        NumberCard::Target => text_input("转换结果会显示在这里", &app.base_output)
            .padding([10, 12])
            .size(13)
            .style(settings_text_input_style)
            .width(Length::Fill)
            .into(),
    }
}

fn build_swap_bridge<'a>() -> Element<'a, Message> {
    button(
        row![icon_svg(Icon::ArrowRepeat, 16.0), text("交换").size(13),]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([10, 14])
    .on_press(Message::BaseTool(BaseToolMessage::Swap))
    .style(rounded_action_btn_style)
    .into()
}

fn build_copy_button<'a>(app: &'a App) -> iced::widget::Button<'a, Message> {
    let button = button(
        row![icon_svg(Icon::Copy, 16.0), text("复制结果").size(13)]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .style(rounded_action_btn_style);

    if app.base_output.is_empty() {
        button
    } else {
        button.on_press(Message::BaseTool(BaseToolMessage::CopyOutput))
    }
}

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
}

fn form_row<'a>(
    label: &'a str,
    description: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
    compact: bool,
) -> Element<'a, Message> {
    let description = description.into();
    let intro =
        column![text(label).size(13), text(description).size(11).style(settings_muted_text_style),]
            .spacing(4);

    let layout: Element<'a, Message> = if compact {
        column![intro, control.into()].spacing(12).into()
    } else {
        row![intro.width(Length::Fixed(220.0)), container(control.into()).width(Length::Fill),]
            .spacing(22)
            .align_y(Alignment::Center)
            .into()
    };

    container(layout).padding([14, 0]).width(Length::Fill).into()
}

fn status_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    row![text(label).size(13).width(Length::Fill), settings_value_badge(value),]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
}

fn valid_digits_label(base: u32) -> String {
    match base {
        2..=10 => format!("0-{}", char::from_digit(base - 1, 10).unwrap_or('9')),
        11..=36 => format!("0-9 / A-{}", digit_char(base - 1)),
        _ => "0-9".to_string(),
    }
}

fn digit_char(value: u32) -> char {
    match value {
        0..=9 => char::from_digit(value, 10).unwrap_or('?'),
        10..=35 => (b'A' + (value - 10) as u8) as char,
        _ => '?',
    }
}

fn build_metric_badge<'a>(label: String) -> Element<'a, Message> {
    container(text(label).size(12).style(settings_muted_text_style))
        .padding([6, 10])
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;

            iced::widget::container::Style {
                background: Some(Background::Color(if is_dark {
                    palette.background.weak.color.scale_alpha(0.34)
                } else {
                    Color::from_rgba8(248, 250, 252, 0.98)
                })),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        palette.background.strong.color.scale_alpha(0.80)
                    } else {
                        Color::from_rgba8(148, 163, 184, 0.18)
                    },
                    radius: 999.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_status_badge<'a>(app: &'a App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Idle,
        Success,
        Error,
    }

    let (label, tone) = match app.base_notification.as_deref() {
        Some("已复制结果") => ("已复制结果".to_string(), StatusTone::Success),
        Some(message) => (message.to_string(), StatusTone::Error),
        None if app.base_input.trim().is_empty() => ("等待输入".to_string(), StatusTone::Idle),
        None => ("已就绪".to_string(), StatusTone::Idle),
    };

    container(text(label).size(12).style(move |theme: &Theme| {
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::text::Style {
            color: Some(match tone {
                StatusTone::Success | StatusTone::Error => Color::WHITE,
                StatusTone::Idle if is_dark => theme.palette().text.scale_alpha(0.92),
                StatusTone::Idle => Color::from_rgba8(71, 85, 105, 1.0),
            }),
        }
    }))
    .padding([8, 12])
    .style(move |theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::container::Style {
            background: Some(Background::Color(match tone {
                StatusTone::Success => Color::from_rgba8(22, 163, 74, 0.92),
                StatusTone::Error => Color::from_rgba8(220, 38, 38, 0.92),
                StatusTone::Idle if is_dark => palette.background.strong.color.scale_alpha(0.82),
                StatusTone::Idle => Color::from_rgba8(241, 245, 249, 0.96),
            })),
            border: Border {
                width: 1.0,
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.88)
                } else {
                    Color::from_rgba8(148, 163, 184, 0.22)
                },
                radius: 999.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

#[cfg(test)]
#[path = "base_tool_tests.rs"]
mod base_tool_tests;
