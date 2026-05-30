//! JSON 工具视图模块
//!
//! 本模块提供 JSON 美化工具的 UI 视图组件。主要功能包括：
//! - JSON 代码编辑器（支持文本编辑操作）
//! - 自定义滚轮滚动与独立滚动条
//! - JSON 格式化、压缩、转义等功能按钮
//! - 记忆模式开关（保持用户设置）
//! - 复制和清空操作
//! - 右键菜单（复制选择、剪切、粘贴、删除）

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_muted_text_style, settings_panel,
    settings_panel_style,
};
use crate::app::components::text_editor_context_menu::{
    TextEditorContextMenuMessages, TextEditorContextMenuState, wrap_with_context_menu,
};
use crate::app::components::text_editor_scroll_panel::{
    TextEditorScrollPanelMetrics, text_editor_scroll_panel,
};
use crate::app::message::JsonToolMessage;
use crate::app::{App, Message};
use iced::widget::{
    Space, button, checkbox, column, container, responsive, row, text, text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

/// 构建 JSON 工具视图
pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("JSON美化工具").size(20),
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
    let controls = build_controls_panel(app);
    let editor = build_editor_card(app, size);

    if size.width >= 960.0 {
        row![
            container(editor).width(Length::FillPortion(3)).height(Length::Fill),
            container(controls).width(Length::Fixed(320.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![controls, editor].spacing(16).height(Length::Fill).into()
    }
}

fn build_controls_panel<'a>(app: &'a App) -> Element<'a, Message> {
    let remember_row = row![
        text("记忆").size(13).width(Length::Fill),
        checkbox(app.json_tool_remember)
            .on_toggle(|enabled| Message::JsonTool(JsonToolMessage::ToggleRemember(enabled))),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let copy_row = row![
        text("复制").size(13).width(Length::Fill),
        button(text("复制").size(13))
            .on_press(Message::JsonTool(JsonToolMessage::Copy))
            .padding([10, 14])
            .style(rounded_action_btn_style),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    column![
        build_section_title("操作"),
        settings_panel(
            column![
                row![
                    build_action_button(app, "格式化校验", JsonToolMessage::Format, true),
                    build_action_button(app, "压缩", JsonToolMessage::Compress, false),
                ]
                .spacing(10),
                row![
                    build_action_button(app, "转义", JsonToolMessage::Escape, false),
                    build_action_button(app, "去除转义", JsonToolMessage::Unescape, false),
                ]
                .spacing(10),
                row![
                    build_action_button(app, "Unicode转中文", JsonToolMessage::UnicodeToCn, false),
                    build_action_button(app, "中文转Unicode", JsonToolMessage::CnToUnicode, false),
                ]
                .spacing(10),
                row![
                    build_action_button(app, "转GET参数", JsonToolMessage::ToGet, false),
                    Space::new().width(Length::Fill),
                ]
                .spacing(10),
            ]
            .spacing(10)
        ),
        build_section_title("偏好"),
        settings_panel(column![remember_row, copy_row].spacing(14)),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_action_button<'a>(
    app: &'a App,
    label: &'static str,
    msg: JsonToolMessage,
    is_primary: bool,
) -> Element<'a, Message> {
    let button = button(text(label).size(13)).padding([10, 12]).width(Length::Fill);
    let button =
        if app.json_tool_loading { button } else { button.on_press(Message::JsonTool(msg)) };

    if is_primary {
        button.style(primary_action_btn_style).into()
    } else {
        button.style(rounded_action_btn_style).into()
    }
}

fn build_editor_card<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let editor_panel = build_editor_panel(app, size);

    column![
        build_section_title("编辑区"),
        settings_panel(
            column![
                row![
                    text("内容").size(13).width(Length::Fill),
                    build_metric_badge(format!("{} 行", app.json_tool_editor.line_count().max(1))),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
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

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
}

fn build_status_badge<'a>(app: &'a App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Loading,
        Success,
        Idle,
    }

    let (label, tone): (String, StatusTone) = if app.json_tool_loading {
        ("处理中".to_string(), StatusTone::Loading)
    } else if let Some(message) = &app.json_tool_notification {
        (message.as_str().to_owned(), StatusTone::Success)
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
                StatusTone::Loading | StatusTone::Success => Color::WHITE,
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

fn build_editor_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let editor = text_editor(&app.json_tool_editor)
        .id(app.json_tool_editor_id.clone())
        .placeholder("输入 JSON")
        .on_action(|action| Message::JsonTool(JsonToolMessage::EditorAction(action)))
        .height(Length::Fill)
        .style(|theme: &Theme, _status| {
            let palette = theme.extended_palette();

            text_editor::Style {
                background: palette.background.base.color.into(),
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
                value: theme.palette().text,
                selection: theme.palette().primary.scale_alpha(0.30),
                placeholder: theme.palette().text.scale_alpha(0.55),
            }
        });

    let editor = wrap_with_context_menu(
        editor,
        TextEditorContextMenuState {
            open: app.json_tool_context_menu_open,
            position: app.json_tool_context_menu_pos,
        },
        |point| Message::JsonTool(JsonToolMessage::OpenContextMenu { x: point.x, y: point.y }),
        TextEditorContextMenuMessages {
            close: Message::JsonTool(JsonToolMessage::CloseContextMenu),
            copy: Message::JsonTool(JsonToolMessage::ContextMenuCopy),
            cut: Message::JsonTool(JsonToolMessage::ContextMenuCut),
            paste: Message::JsonTool(JsonToolMessage::ContextMenuPaste),
            delete: Message::JsonTool(JsonToolMessage::ContextMenuDelete),
        },
    );

    text_editor_scroll_panel(
        editor,
        size,
        TextEditorScrollPanelMetrics {
            viewport_padding: 24.0,
            line_height: app.current_line_height,
            line_count: app.json_tool_editor.line_count(),
            scroll_top_line: app.json_tool_scroll_top_line,
        },
        |delta, viewport_height| {
            Message::JsonTool(JsonToolMessage::EditorWheelScrolled { delta, viewport_height })
        },
        |top_line, viewport_height| {
            Message::JsonTool(JsonToolMessage::ScrollbarChanged { top_line, viewport_height })
        },
    )
}
#[cfg(test)]
#[path = "json_tool_tests.rs"]
mod json_tool_tests;
