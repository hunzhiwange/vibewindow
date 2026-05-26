//! 二维码工具视图模块，负责二维码输入、预览和操作控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_divider,
    settings_close_button, settings_modal_backdrop_style, settings_modal_card_style,
    settings_muted_text_style, settings_page_intro, settings_panel, settings_panel_style,
    settings_pick_list_menu_style, settings_pick_list_style, settings_section_card,
    settings_text_input_style,
};
use crate::app::components::text_editor_scroll_panel::{
    TextEditorScrollPanelMetrics, text_editor_scroll_panel,
};
use crate::app::message::qr_tool::{QrEcLevel, QrIconMode, QrToolMessage};
use crate::app::views::design::properties::color_picker::{
    format_rgba_to_hex, parse_color, render_color_picker,
};
use crate::app::{App, Message};
use iced::widget::{
    Image, Space, button, column, container, mouse_area, opaque, pick_list, responsive, row,
    stack, text, text_editor, text_input,
};
use iced::{Alignment, Background, Border, Color, ContentFit, Element, Length, Size, Theme};

const FORM_LABEL_WIDTH: f32 = 104.0;
const COLOR_DRAWER_MIN_WIDTH: f32 = 380.0;
const COLOR_DRAWER_MAX_WIDTH: f32 = 520.0;

/// 渲染对应界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("二维码生成器").size(20),
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

    let base: Element<'_, Message> = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .into();

    if app.show_qr_color_picker {
        stack![base, build_color_picker_drawer(app)].into()
    } else {
        base
    }
}

fn build_color_picker_drawer<'a>(app: &'a App) -> Element<'a, Message> {
    let color = parse_color(&app.qr_color_hex).unwrap_or(Color::from_rgb8(0, 0, 0));
    let color_hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
    let panel_width = (app.window_size.0 * 0.36).clamp(COLOR_DRAWER_MIN_WIDTH, COLOR_DRAWER_MAX_WIDTH);
    let close_message = Message::QrTool(QrToolMessage::ToggleColorPicker);

    let overlay = opaque(
        mouse_area(
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .style(settings_modal_backdrop_style),
        )
        .on_press(close_message.clone()),
    );

    let drawer_card: Element<'a, Message> = container(
        column![
            row![
                column![
                    text("颜色面板").size(16),
                    text("从左侧独立弹出，不受右侧参数栏宽度影响。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(6)
                .width(Length::Fill),
                build_metric_badge(color_hex),
                settings_close_button(close_message.clone()),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            container(
                render_color_picker(
                    color,
                    app.qr_color_format,
                    false,
                    |picked| {
                        let hex = format_rgba_to_hex(picked.r, picked.g, picked.b, picked.a);
                        Message::QrTool(QrToolMessage::ColorChanged(hex))
                    },
                    |format| Message::QrTool(QrToolMessage::ColorFormatChanged(format)),
                    || Message::None,
                ),
            )
            .padding([18, 18])
            .width(Length::Fill)
            .style(settings_panel_style),
        ]
        .spacing(18),
    )
    .padding([22, 24])
    .width(Length::Fixed(panel_width))
    .style(settings_modal_card_style)
    .into();

    let drawer_layer: Element<'a, Message> = opaque(
        container(mouse_area(drawer_card).on_press(Message::None))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([24, 20])
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center),
    );

    stack![overlay, drawer_layer].into()
}

fn build_workspace<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let controls = build_controls_panel(app);
    let main = column![
        container(build_editor_card(app, size)).height(Length::FillPortion(2)),
        container(build_preview_card(app)).height(Length::FillPortion(3)),
    ]
    .spacing(16)
    .height(Length::Fill);

    if size.width >= 960.0 {
        row![
            container(main).width(Length::FillPortion(3)).height(Length::Fill),
            container(controls).width(Length::Fixed(360.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![controls, main].spacing(16).height(Length::Fill).into()
    }
}

fn build_controls_panel<'a>(app: &'a App) -> Element<'a, Message> {
    let color = parse_color(&app.qr_color_hex).unwrap_or(Color::from_rgb8(0, 0, 0));
    let icon_mode_selector = pick_list(Vec::from(QrIconMode::all()), Some(app.qr_icon_mode), |mode| {
        Message::QrTool(QrToolMessage::IconModeSelected(mode))
    })
    .padding([10, 14])
    .text_size(13)
    .style(settings_pick_list_style)
    .menu_style(settings_pick_list_menu_style)
    .width(Length::Fill);

    let upload_row: Element<'a, Message> = if app.qr_icon_mode == QrIconMode::Upload {
        row![
            text(if app.qr_icon_bytes.is_some() { "已选择自定义图标" } else { "未选择图标" })
                .size(12)
                .style(settings_muted_text_style)
                .width(Length::Fill),
            build_action_button(app, "选择图片", QrToolMessage::PickUploadedIcon, false),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    } else {
        Space::new().height(Length::Shrink).into()
    };

    column![
        settings_page_intro("生成设置", "表单样式对齐系统设置常规页，统一管理尺寸、容错、颜色与中心图标。"),
        settings_section_card("生成操作", "先填写内容，再选择样式参数并生成二维码。"),
        settings_panel(
            column![
                row![
                    build_action_button(app, "生成二维码", QrToolMessage::Generate, true),
                    build_action_button(app, "保存 PNG", QrToolMessage::SavePng, false),
                ]
                .spacing(10),
                row![
                    build_action_button(app, "清空", QrToolMessage::Clear, false),
                    Space::new().width(Length::Fill),
                ]
                .spacing(10),
            ]
            .spacing(10)
        ),
        settings_section_card("参数表单", "尺寸、容错等级与样式会直接影响输出质量与扫码稳定性。"),
        settings_panel(
            column![
                build_form_row(
                    "尺寸",
                    "输出 PNG 的边长，支持 64-2048 像素。",
                    text_input("256", &app.qr_size_input)
                        .padding([10, 14])
                        .size(13)
                        .style(settings_text_input_style)
                        .width(Length::Fill)
                        .on_input(|value| Message::QrTool(QrToolMessage::SizeChanged(value))),
                ),
                settings_divider(),
                build_form_row(
                    "容错等级",
                    "中心图标建议搭配 Q 或 H，识别率更稳定。",
                    pick_list(Vec::from(QrEcLevel::all()), Some(app.qr_level), |level| {
                        Message::QrTool(QrToolMessage::LevelSelected(level))
                    })
                    .padding([10, 14])
                    .text_size(13)
                    .style(settings_pick_list_style)
                    .menu_style(settings_pick_list_menu_style)
                    .width(Length::Fill),
                ),
                settings_divider(),
                build_form_row(
                    "颜色",
                    "支持直接输入颜色值，也可以点击左侧色块打开独立颜色面板。",
                    column![
                        row![
                            build_color_swatch_button(color),
                            text_input("#000000", &app.qr_color_hex)
                                .padding([10, 14])
                                .size(13)
                                .style(settings_text_input_style)
                                .width(Length::Fill)
                                .on_input(|value| {
                                    Message::QrTool(QrToolMessage::ColorChanged(value))
                                }),
                            button(
                                text(if app.show_qr_color_picker { "收起" } else { "打开" })
                                    .size(13),
                            )
                            .padding([10, 12])
                            .style(rounded_action_btn_style)
                            .on_press(Message::QrTool(QrToolMessage::ToggleColorPicker)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        text("支持 #RRGGBB、#RRGGBBAA 等格式，左侧浮层会提供更大的取色空间。")
                            .size(11)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(12),
                ),
                settings_divider(),
                build_form_row(
                    "中心图标",
                    "可选默认 Logo 或上传图片，建议使用透明底 PNG。",
                    column![
                        icon_mode_selector,
                        upload_row,
                        text(qr_icon_mode_description(app.qr_icon_mode))
                        .size(12)
                        .style(settings_muted_text_style),
                    ]
                    .spacing(10),
                ),
            ]
            .spacing(0)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_editor_card<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let content = app.qr_editor.text();
    let editor_panel = build_editor_panel(app, size);

    column![
        settings_page_intro("内容输入", "输入网址、文本或任意字符串；表单参数会直接影响右侧预览与导出 PNG。"),
        settings_panel(
            column![
                row![
                    text("输入").size(13).width(Length::Fill),
                    build_metric_badge(format!("{} 行", app.qr_editor.line_count().max(1))),
                    build_metric_badge(format!("{} 字符", content.chars().count())),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                editor_panel,
            ]
            .spacing(14)
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn build_preview_card<'a>(app: &'a App) -> Element<'a, Message> {
    let preview_body: Element<'a, Message> = if app.qr_loading {
        container(
            column![
                text("正在生成预览").size(16),
                text("完成后会在这里显示最新二维码。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(preview_surface_style)
        .into()
    } else if let Some(handle) = &app.qr_image {
        let handle: iced::widget::image::Handle = handle.clone();
        container(
            Image::new(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(preview_surface_style)
        .into()
    } else {
        container(
            column![
                text("生成后将在此处预览二维码").size(16),
                text("支持调整尺寸、颜色、容错等级和中心图标。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(preview_surface_style)
        .into()
    };

    column![
        settings_page_intro("预览输出", "生成结果会在这里实时显示，可直接导出 PNG 供扫码、分享或落地页使用。"),
        settings_panel(
            column![
                row![
                    text("输出").size(13).width(Length::Fill),
                    build_metric_badge(format!("{} px", app.qr_size)),
                    build_metric_badge(format!("ECC {}", app.qr_level)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                container(preview_body)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            ]
            .spacing(14)
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn build_editor_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let editor = text_editor(&app.qr_editor)
        .placeholder("输入网址、文本或任意内容")
        .on_action(|action| Message::QrTool(QrToolMessage::EditorAction(action)))
        .height(Length::Fill)
        .style(|theme: &Theme, _status| {
            let style = crate::app::components::system_settings_common::settings_text_editor_style(
                theme,
                text_editor::Status::Active,
            );

            text_editor::Style {
                background: style.background,
                border: style.border,
                value: style.value,
                selection: style.selection,
                placeholder: style.placeholder,
            }
        });

    text_editor_scroll_panel(
        editor,
        size,
        TextEditorScrollPanelMetrics {
            viewport_padding: 24.0,
            line_height: app.current_line_height,
            line_count: app.qr_editor.line_count(),
            scroll_top_line: app.qr_scroll_top_line,
        },
        |delta, viewport_height| {
            Message::QrTool(QrToolMessage::EditorWheelScrolled { delta, viewport_height })
        },
        |top_line, viewport_height| {
            Message::QrTool(QrToolMessage::ScrollbarChanged { top_line, viewport_height })
        },
    )
}

fn build_action_button<'a>(
    app: &'a App,
    label: &'static str,
    message: QrToolMessage,
    is_primary: bool,
) -> Element<'a, Message> {
    let button = button(text(label).size(13)).padding([10, 12]).width(Length::Fill);
    let button = if app.qr_loading {
        button
    } else {
        button.on_press(Message::QrTool(message))
    };

    if is_primary {
        button.style(primary_action_btn_style).into()
    } else {
        button.style(rounded_action_btn_style).into()
    }
}

fn build_status_badge<'a>(app: &'a App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Loading,
        Success,
        Error,
        Idle,
    }

    let (label, tone) = if app.qr_loading {
        ("处理中".to_string(), StatusTone::Loading)
    } else if let Some(message) = &app.qr_notification {
        (
            message.as_str().to_owned(),
            if app.qr_notification_is_error {
                StatusTone::Error
            } else {
                StatusTone::Success
            },
        )
    } else {
        ("已就绪".to_string(), StatusTone::Idle)
    };

    container(text(label).size(12).style(move |theme: &Theme| {
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::text::Style {
            color: Some(match tone {
                StatusTone::Loading | StatusTone::Success | StatusTone::Error => Color::WHITE,
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
                StatusTone::Loading => Color::from_rgba8(37, 99, 235, 0.92),
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

fn build_form_row<'a>(
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
        .align_y(Alignment::Start),
    )
    .padding([14, 0])
    .width(Length::Fill)
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

fn qr_icon_mode_description(mode: QrIconMode) -> &'static str {
    match mode {
        QrIconMode::None => "保持标准二维码结构，兼容性最佳。",
        QrIconMode::Default => "使用内置 Logo，适合快速生成品牌二维码。",
        QrIconMode::Upload => "建议上传透明底 PNG，中心区域会自动留白。",
    }
}

fn build_color_swatch_button<'a>(color: Color) -> Element<'a, Message> {
    button(
        container(Space::new().width(Length::Fixed(22.0)).height(Length::Fixed(22.0))).style(
            move |_theme: &Theme| iced::widget::container::Style {
                background: Some(color.into()),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgba8(148, 163, 184, 0.35),
                    radius: 7.0.into(),
                },
                ..Default::default()
            },
        ),
    )
    .width(Length::Fixed(34.0))
    .height(Length::Fixed(34.0))
    .padding(5)
    .style(rounded_action_btn_style)
    .on_press(Message::QrTool(QrToolMessage::ToggleColorPicker))
    .into()
}

fn preview_surface_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let is_dark = theme.palette().background.r
        + theme.palette().background.g
        + theme.palette().background.b
        < 1.5;

    iced::widget::container::Style {
        background: Some(Background::Color(if is_dark {
            Color::from_rgba8(248, 250, 252, 0.98)
        } else {
            Color::WHITE
        })),
        border: Border {
            width: 1.0,
            color: if is_dark {
                palette.background.strong.color.scale_alpha(0.75)
            } else {
                Color::from_rgba8(148, 163, 184, 0.18)
            },
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "qr_tool_tests.rs"]
mod qr_tool_tests;
