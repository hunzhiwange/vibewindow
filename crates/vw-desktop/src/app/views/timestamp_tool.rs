//! 时间戳转换工具视图模块
//!
//! 本模块提供时间戳转换器的完整界面，负责组织以下几个功能区域：
//! - 页面头部状态卡：显示工具标题、简介与当前状态
//! - 当前 UTC 时间卡片：展示实时 UTC 时间、Unix 秒与 Unix 毫秒
//! - 时间戳转时间卡片：支持按秒或毫秒输入并转换为 UTC 时间
//! - 时间转时间戳卡片：按 UTC 解析输入时间并输出秒/毫秒时间戳
//! - 辅助信息侧栏：展示当前模式、输入状态与使用说明
//!
//! 视觉风格与仓库中的 Markdown、JSON、进制工具保持一致，统一采用：
//! - 顶部 hero 状态卡
//! - 圆角面板与浅阴影
//! - 响应式双栏/单栏布局
//! - 暗黑主题下可读的边框和背景层次

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_muted_text_style, settings_panel,
    settings_panel_style, settings_pick_list_menu_style, settings_pick_list_style,
    settings_text_input_style, settings_value_badge,
};
use crate::app::message::{TimestampToolMessage, timestamp_tool::TsUnit};
use crate::app::{App, Message};
use iced::widget::{
    Space, button, column, container, pick_list, responsive, row, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

/// 构建时间戳转换工具主视图。
///
/// 页面由顶部状态卡和下方工作区组成：
/// - 宽屏下采用“左主内容 + 右侧信息栏”布局
/// - 窄屏下退化为纵向堆叠布局
///
/// # 参数
///
/// * `app` - 应用状态，包含时间戳工具的输入、输出和通知信息
///
/// # 返回值
///
/// 返回完整的 Iced 元素树
pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            column![
                text("时间戳转换器").size(20),
                text("UTC 时间、Unix 秒与 Unix 毫秒的双向换算工具")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(4),
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
    let converters = build_converters_panel(app, size);
    let side_panel = build_side_panel(app);

    if size.width >= 1120.0 {
        row![
            container(converters).width(Length::FillPortion(3)).height(Length::Fill),
            container(side_panel).width(Length::Fixed(320.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .align_y(Alignment::Start)
        .into()
    } else {
        column![side_panel, converters].spacing(16).height(Length::Fill).into()
    }
}

fn build_converters_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let compact = size.width < 920.0;
    let current_panel = build_current_time_panel(app, compact);
    let from_ts_panel = build_from_timestamp_panel(app, compact);
    let to_ts_panel = build_to_timestamp_panel(app, compact);

    column![current_panel, from_ts_panel, to_ts_panel].spacing(16).height(Length::Shrink).into()
}

fn build_current_time_panel<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let toggle_button = if app.ts_auto {
        button(text("暂停自动刷新").size(13))
            .padding([10, 12])
            .on_press(Message::TimestampTool(TimestampToolMessage::ToggleAuto(false)))
            .style(rounded_action_btn_style)
    } else {
        button(text("开启自动刷新").size(13))
            .padding([10, 12])
            .on_press(Message::TimestampTool(TimestampToolMessage::ToggleAuto(true)))
            .style(primary_action_btn_style)
    };

    let refresh_button = button(text("立即刷新").size(13))
        .padding([10, 12])
        .on_press(Message::TimestampTool(TimestampToolMessage::RefreshNow))
        .style(rounded_action_btn_style);

    column![
        build_section_title("当前 UTC 时间"),
        settings_panel(
            column![
                row![
                    column![
                        text("实时基准时间").size(13),
                        text("用于快速查看当前 UTC、秒级与毫秒级 Unix 时间戳。")
                            .size(11)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(4),
                    Space::new().width(Length::Fill),
                    build_metric_badge(if app.ts_auto {
                        "自动刷新中".to_string()
                    } else {
                        "手动模式".to_string()
                    }),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                form_row(
                    "UTC 时间",
                    "标准输出格式为 YYYY-MM-DD HH:MM:SS UTC。",
                    text_input("", &app.ts_now_utc_str)
                        .padding([10, 12])
                        .size(13)
                        .style(settings_text_input_style)
                        .width(Length::Fill),
                    compact,
                ),
                form_row(
                    "Unix 秒",
                    "适合日志、接口和数据库中的秒级时间戳。",
                    text_input("", &app.ts_now_unix_sec)
                        .padding([10, 12])
                        .size(13)
                        .style(settings_text_input_style)
                        .width(Length::Fill),
                    compact,
                ),
                form_row(
                    "Unix 毫秒",
                    "适合前端、JavaScript 与高精度事件记录。",
                    text_input("", &app.ts_now_unix_ms)
                        .padding([10, 12])
                        .size(13)
                        .style(settings_text_input_style)
                        .width(Length::Fill),
                    compact,
                ),
                actions_row(vec![toggle_button.into(), refresh_button.into()], compact),
            ]
            .spacing(14),
        ),
    ]
    .spacing(12)
    .into()
}

fn build_from_timestamp_panel<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let unit_picker = pick_list(Vec::from(TsUnit::all()), Some(app.ts_unit), |unit| {
        Message::TimestampTool(TimestampToolMessage::UnitSelected(unit))
    })
    .padding([10, 14])
    .text_size(13)
    .style(settings_pick_list_style)
    .menu_style(settings_pick_list_menu_style)
    .width(if compact { Length::Fill } else { Length::Fixed(170.0) });

    let convert_button = button(text("转换为 UTC 时间").size(13))
        .padding([10, 12])
        .on_press(Message::TimestampTool(TimestampToolMessage::ConvertFromTs))
        .style(primary_action_btn_style);

    let copy_button =
        button(text("复制结果").size(13)).padding([10, 12]).style(rounded_action_btn_style);
    let copy_button = if app.ts_time_output.trim().is_empty() {
        copy_button
    } else {
        copy_button.on_press(Message::TimestampTool(TimestampToolMessage::CopyTimeOutput))
    };

    column![
        build_section_title("Unix 时间戳转时间"),
        settings_panel(
            column![
                row![
                    column![
                        text("输入时间戳").size(13),
                        text("支持按秒或毫秒解析，输出结果统一为 UTC。")
                            .size(11)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(4),
                    Space::new().width(Length::Fill),
                    build_metric_badge(format!("{} 字符", app.ts_input_ts.chars().count())),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                form_row(
                    "时间戳输入",
                    "示例：1388307215 或 1388307215000。",
                    column![
                        text_input("输入 Unix 时间戳", &app.ts_input_ts)
                            .on_input(|value| {
                                Message::TimestampTool(TimestampToolMessage::InputTsChanged(value))
                            })
                            .padding([10, 12])
                            .size(13)
                            .style(settings_text_input_style)
                            .width(Length::Fill),
                        actions_row(vec![unit_picker.into(), convert_button.into()], compact),
                    ]
                    .spacing(10),
                    compact,
                ),
                form_row(
                    "转换结果",
                    "输出格式固定带 UTC 后缀，避免本地时区歧义。",
                    column![
                        text_input("转换结果会显示在这里", &app.ts_time_output)
                            .padding([10, 12])
                            .size(13)
                            .style(settings_text_input_style)
                            .width(Length::Fill),
                        actions_row(vec![copy_button.into()], compact),
                    ]
                    .spacing(10),
                    compact,
                ),
            ]
            .spacing(14),
        ),
    ]
    .spacing(12)
    .into()
}

fn build_to_timestamp_panel<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let convert_button = button(text("转换为时间戳").size(13))
        .padding([10, 12])
        .on_press(Message::TimestampTool(TimestampToolMessage::ConvertFromTime))
        .style(primary_action_btn_style);

    let copy_button = button(text("复制秒 / 毫秒结果").size(13))
        .padding([10, 12])
        .style(rounded_action_btn_style);
    let copy_button =
        if app.ts_ts_output_sec.trim().is_empty() && app.ts_ts_output_ms.trim().is_empty() {
            copy_button
        } else {
            copy_button.on_press(Message::TimestampTool(TimestampToolMessage::CopyTsOutput))
        };

    column![
        build_section_title("时间转 Unix 时间戳"),
        settings_panel(
            column![
                row![
                    column![
                        text("输入 UTC 时间").size(13),
                        text("未填写时分秒时默认补 00:00:00，可附带 .毫秒。")
                            .size(11)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(4),
                    Space::new().width(Length::Fill),
                    build_metric_badge(if app.ts_time_input.trim().is_empty() {
                        "等待输入".to_string()
                    } else {
                        format!("{} 字符", app.ts_time_input.chars().count())
                    }),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                form_row(
                    "UTC 时间输入",
                    "示例：2015-04-01 10:01:01.620。",
                    column![
                        text_input("输入 UTC 时间", &app.ts_time_input)
                            .on_input(|value| {
                                Message::TimestampTool(TimestampToolMessage::InputTimeChanged(
                                    value,
                                ))
                            })
                            .padding([10, 12])
                            .size(13)
                            .style(settings_text_input_style)
                            .width(Length::Fill),
                        actions_row(vec![convert_button.into()], compact),
                    ]
                    .spacing(10),
                    compact,
                ),
                form_row(
                    "秒级时间戳",
                    "输出为十进制整数，可直接用于多数后端与数据库。",
                    text_input("秒级结果", &app.ts_ts_output_sec)
                        .padding([10, 12])
                        .size(13)
                        .style(settings_text_input_style)
                        .width(Length::Fill),
                    compact,
                ),
                form_row(
                    "毫秒级时间戳",
                    "适合 Web 与客户端场景，常用于 JavaScript Date。",
                    column![
                        text_input("毫秒级结果", &app.ts_ts_output_ms)
                            .padding([10, 12])
                            .size(13)
                            .style(settings_text_input_style)
                            .width(Length::Fill),
                        actions_row(vec![copy_button.into()], compact),
                    ]
                    .spacing(10),
                    compact,
                ),
            ]
            .spacing(14),
        ),
    ]
    .spacing(12)
    .into()
}

fn build_side_panel<'a>(app: &'a App) -> Element<'a, Message> {
    column![
        build_section_title("当前状态"),
        settings_panel(
            column![
                status_row("运行模式", if app.ts_auto { "自动刷新" } else { "手动刷新" }),
                status_row("时间单位", app.ts_unit.to_string()),
                status_row("当前 UTC", &app.ts_now_utc_str),
                status_row("秒级长度", format!("{} 位", app.ts_now_unix_sec.chars().count())),
                status_row("毫秒长度", format!("{} 位", app.ts_now_unix_ms.chars().count())),
            ]
            .spacing(10),
        ),
        build_section_title("使用说明"),
        settings_panel(
            column![
                text("页面内所有时间默认按 UTC 展示和解析，不做本地时区推断。")
                    .size(12)
                    .style(settings_muted_text_style),
                text("时间戳输入支持秒与毫秒两种模式，切换单位后再执行转换。")
                    .size(12)
                    .style(settings_muted_text_style),
                text("时间输入可只写日期，未填写的时分秒会自动补零。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(10),
        ),
        build_section_title("快捷参考"),
        settings_panel(
            column![
                status_row("UTC 示例", "2026-04-16 08:30:00 UTC"),
                status_row("秒级示例", "1713256200"),
                status_row("毫秒示例", "1713256200000"),
            ]
            .spacing(10),
        ),
    ]
    .spacing(12)
    .into()
}

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
}

fn form_row<'a>(
    label: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
    compact: bool,
) -> Element<'a, Message> {
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

    container(layout).width(Length::Fill).into()
}

fn actions_row<'a>(items: Vec<Element<'a, Message>>, compact: bool) -> Element<'a, Message> {
    let mut row_layout = row![].spacing(10).align_y(Alignment::Center);
    for item in items {
        row_layout = row_layout.push(item);
    }

    if compact {
        container(row_layout.wrap()).width(Length::Fill).into()
    } else {
        container(row_layout).width(Length::Shrink).into()
    }
}

fn status_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    row![text(label).size(13).width(Length::Fill), settings_value_badge(value),]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
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

    let (label, tone): (String, StatusTone) = match app.ts_notification.as_deref() {
        Some("转换成功") | Some("已复制") => {
            (app.ts_notification.clone().unwrap_or_default(), StatusTone::Success)
        }
        Some(message) => (message.to_owned(), StatusTone::Error),
        None if app.ts_auto => ("自动刷新中".to_string(), StatusTone::Idle),
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
#[path = "timestamp_tool_tests.rs"]
mod timestamp_tool_tests;
