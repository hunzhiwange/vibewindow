//! 颜色转换工具视图模块
//!
//! 对齐 JSON 美化工具的页头与工作区布局，同时复用系统设置表单样式，
//! 提供统一的颜色输入、预览和复制体验。

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_divider,
    settings_muted_text_style, settings_page_intro, settings_panel, settings_panel_style,
    settings_text_input_style, settings_value_badge,
};
use crate::app::message::ColorToolMessage;
use crate::app::views::design::properties::color_picker::{
    Hsv, format_rgba_to_css, format_rgba_to_hex, render_color_picker, rgba_to_hsla,
};
use crate::app::{App, Message};
use iced::widget::{Space, button, column, container, responsive, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

const FORM_LABEL_WIDTH: f32 = 116.0;

struct FormattedOutputs {
    hex: String,
    rgb: String,
    hsl: String,
    hsv: String,
    hsv_info: Hsv,
    alpha_percent: u8,
}

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("颜色转换工具").size(20),
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
    let preview = build_preview_panel(app);
    let forms = build_forms_panel(app);

    if size.width >= 1080.0 {
        row![
            container(preview).width(Length::FillPortion(3)).height(Length::Fill),
            container(forms).width(Length::Fixed(420.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![forms, preview].spacing(16).height(Length::Fill).into()
    }
}

fn build_preview_panel<'a>(app: &'a App) -> Element<'a, Message> {
    let color = app.color_tool_color;
    let outputs = format_outputs(color);
    let summary = row![
        preview_box(color, 88.0, 88.0),
        column![
            text("当前颜色").size(14),
            text("拖动取色板、色相条和透明度滑块后，右侧表单会同步刷新。")
                .size(12)
                .style(settings_muted_text_style),
            row![
                settings_value_badge(outputs.hex.clone()),
                settings_value_badge(format!("Alpha {}%", outputs.alpha_percent)),
                settings_value_badge(format!("Hue {:.0}°", outputs.hsv_info.h.round())),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .width(Length::Fill),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    column![
        settings_page_intro("颜色预览", "参考 JSON 工具的工作区布局，预览与输入拆分为清晰的左右区域。"),
        settings_panel(
            column![
                summary,
                render_color_picker(
                    color,
                    app.color_tool_format,
                    false,
                    |c| Message::ColorTool(ColorToolMessage::ColorChanged(c)),
                    |fmt| Message::ColorTool(ColorToolMessage::ColorFormatChanged(fmt)),
                    || Message::None,
                ),
            ]
            .spacing(18)
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn build_forms_panel<'a>(app: &'a App) -> Element<'a, Message> {
    let outputs = format_outputs(app.color_tool_color);

    column![
        settings_page_intro("格式输入", "表单样式对齐系统设置常规页，支持 HEX、RGB、HSL、HSV 四种语法。"),
        settings_panel(
            column![
                build_input_row(
                    "HEX",
                    "输入 #RRGGBB 或 #RRGGBBAA。",
                    "例如 #43ad7fff",
                    &app.color_hex_input,
                    |value| Message::ColorTool(ColorToolMessage::HexInputChanged(value)),
                    Message::ColorTool(ColorToolMessage::HexValidate),
                ),
                settings_divider(),
                build_input_row(
                    "RGB",
                    "输入 rgb(...) 或 rgba(...)。",
                    "例如 rgba(67, 173, 127, 0.50)",
                    &app.color_rgb_input,
                    |value| Message::ColorTool(ColorToolMessage::RgbInputChanged(value)),
                    Message::ColorTool(ColorToolMessage::RgbValidate),
                ),
                settings_divider(),
                build_input_row(
                    "HSL",
                    "输入 hsl(...) 或 hsla(...)。",
                    "例如 hsla(154, 44%, 47%, 0.50)",
                    &app.color_hsl_input,
                    |value| Message::ColorTool(ColorToolMessage::HslInputChanged(value)),
                    Message::ColorTool(ColorToolMessage::HslValidate),
                ),
                settings_divider(),
                build_input_row(
                    "HSV",
                    "输入 hsv(...) 或 hsva(...)。",
                    "例如 hsva(154, 61%, 68%, 0.50)",
                    &app.color_hsv_input,
                    |value| Message::ColorTool(ColorToolMessage::HsvInputChanged(value)),
                    Message::ColorTool(ColorToolMessage::HsvValidate),
                ),
            ]
            .spacing(0)
        ),
        settings_page_intro("复制结果", "当前颜色会规范化输出，方便直接贴到样式变量、设计稿或 CSS 中。"),
        settings_panel(
            column![
                build_output_row("HEX", "适合设计 Token 与变量。", outputs.hex.clone()),
                settings_divider(),
                build_output_row("RGB", "适合 CSS rgba() 场景。", outputs.rgb.clone()),
                settings_divider(),
                build_output_row("HSL", "适合基于色相与亮度调色。", outputs.hsl.clone()),
                settings_divider(),
                build_output_row("HSV", "适合取色和视觉校对。", outputs.hsv.clone()),
            ]
            .spacing(0)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_input_row<'a>(
    label: &'a str,
    description: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'static,
    on_apply: Message,
) -> Element<'a, Message> {
    form_row(
        label,
        description,
        row![
            text_input(placeholder, value)
                .padding([10, 14])
                .width(Length::Fill)
                .style(settings_text_input_style)
                .on_input(on_input),
            button(text("应用").size(13))
                .padding([10, 12])
                .style(primary_action_btn_style)
                .on_press(on_apply),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
}

fn build_output_row<'a>(label: &'a str, description: &'a str, value: String) -> Element<'a, Message> {
    form_row(
        label,
        description,
        row![
            readonly_value_field(value.clone()),
            button(text("复制").size(13))
                .padding([10, 12])
                .style(rounded_action_btn_style)
                .on_press(Message::ColorTool(ColorToolMessage::Copy(value))),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
}

fn form_row<'a>(
    label: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(label).size(13),
                text(description).size(11).style(settings_muted_text_style),
            ]
            .spacing(4)
            .width(Length::Fixed(FORM_LABEL_WIDTH)),
            container(control.into()).width(Length::Fill),
        ]
        .spacing(18)
        .align_y(Alignment::Center),
    )
    .padding([14, 0])
    .width(Length::Fill)
    .into()
}

fn readonly_value_field(value: String) -> Element<'static, Message> {
    container(text(value).size(13))
        .padding([11, 14])
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let style = settings_text_input_style(theme, text_input::Status::Active);
            iced::widget::container::Style {
                text_color: Some(style.value),
                background: Some(style.background),
                border: style.border,
                ..Default::default()
            }
        })
        .into()
}

fn preview_box(color: Color, width: f32, height: f32) -> Element<'static, Message> {
    container(Space::new().width(width).height(height))
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;

            iced::widget::container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    width: 1.0,
                    color: if is_dark {
                        palette.background.strong.color.scale_alpha(0.86)
                    } else {
                        Color::from_rgba8(15, 23, 42, 0.10)
                    },
                    radius: 18.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_status_badge<'a>(app: &'a App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Error,
        Success,
        Idle,
    }

    let (label, tone) = match &app.color_notification {
        Some(message) if message.contains("错误") => (message.clone(), StatusTone::Error),
        Some(message) => (message.clone(), StatusTone::Success),
        None => ("实时同步".to_string(), StatusTone::Idle),
    };

    container(text(label).size(12).style(move |theme: &Theme| {
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::text::Style {
            color: Some(match tone {
                StatusTone::Error | StatusTone::Success => Color::WHITE,
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
                StatusTone::Error => Color::from_rgba8(220, 38, 38, 0.92),
                StatusTone::Success => Color::from_rgba8(22, 163, 74, 0.92),
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

fn format_outputs(color: Color) -> FormattedOutputs {
    let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
    let rgb = format_rgba_to_css(color.r, color.g, color.b, color.a);
    let (h, s, l, a) = rgba_to_hsla(color);
    let hsv_info = Hsv::from_color(color);

    FormattedOutputs {
        hex,
        rgb,
        hsl: format_hsla(h, s, l, a),
        hsv: format_hsva(hsv_info.h, hsv_info.s, hsv_info.v, color.a),
        hsv_info,
        alpha_percent: (color.a * 100.0).round() as u8,
    }
}

/// 格式化 HSLA 颜色值为字符串
///
/// 将 HSLA 颜色分量格式化为标准的 CSS HSLA 字符串格式。
///
/// # 参数
///
/// * `h` - 色相值，范围 0-360
/// * `s` - 饱和度，范围 0.0-1.0
/// * `l` - 亮度，范围 0.0-1.0
/// * `a` - 透明度，范围 0.0-1.0
///
/// # 返回值
///
/// 返回格式化的 HSLA 字符串，如 `"hsla(154, 44%, 47%, 0.50)"`
///
/// # 示例
///
/// ```rust,ignore
/// let hsla_str = format_hsla(154.0, 0.44, 0.47, 0.50);
/// assert_eq!(hsla_str, "hsla(154, 44%, 47%, 0.50)");
/// ```
fn format_hsla(h: f32, s: f32, l: f32, a: f32) -> String {
    let h_int = h.round();
    let s_pct = (s * 100.0).round();
    let l_pct = (l * 100.0).round();
    format!("hsla({h_int}, {s_pct}%, {l_pct}%, {:.2})", a)
}

/// 格式化 HSVA 颜色值为字符串
///
/// 将 HSVA 颜色分量格式化为标准的 HSVA 字符串格式。
///
/// # 参数
///
/// * `h` - 色相值，范围 0-360
/// * `s` - 饱和度，范围 0.0-1.0
/// * `v` - 明度值，范围 0.0-1.0
/// * `a` - 透明度，范围 0.0-1.0
///
/// # 返回值
///
/// 返回格式化的 HSVA 字符串，如 `"hsva(154, 61%, 68%, 0.50)"`
///
/// # 示例
///
/// ```rust,ignore
/// let hsva_str = format_hsva(154.0, 0.61, 0.68, 0.50);
/// assert_eq!(hsva_str, "hsva(154, 61%, 68%, 0.50)");
/// ```
fn format_hsva(h: f32, s: f32, v: f32, a: f32) -> String {
    let h_int = h.round();
    let s_pct = (s * 100.0).round();
    let v_pct = (v * 100.0).round();
    format!("hsva({h_int}, {s_pct}%, {v_pct}%, {:.2})", a)
}

#[cfg(test)]
#[path = "color_tool_tests.rs"]
mod color_tool_tests;
