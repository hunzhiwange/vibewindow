//! JSON/YAML 互转工具视图模块
//!
//! 本模块提供 JSON/YAML 互转工具的 UI 视图组件。主要功能包括：
//! - 左右双编辑器（支持右键菜单与独立滚动条）
//! - JSON 到 YAML / YAML 到 JSON 的转换操作
//! - 内容交换、清空和复制操作
//! - 与 JSON 美化工具统一的卡片式布局和状态呈现

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
use crate::app::message::JsonYamlToolMessage;
use crate::app::{App, Message};
use iced::widget::{Space, button, column, container, responsive, row, text, text_editor};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("JSON/YAML互转工具").size(20),
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
    let editors = build_editor_workspace(app, size);

    if size.width >= 1180.0 {
        row![
            container(editors).width(Length::FillPortion(3)).height(Length::Fill),
            container(controls).width(Length::Fixed(320.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![controls, editors].spacing(16).height(Length::Fill).into()
    }
}

fn build_controls_panel<'a>(app: &'a App) -> Element<'a, Message> {
    column![
        build_section_title("转换"),
        settings_panel(
            column![
                row![
                    build_action_button(
                        app,
                        "YAML→JSON",
                        JsonYamlToolMessage::YamlToJson,
                        true,
                        true,
                    ),
                    build_action_button(
                        app,
                        "JSON→YAML",
                        JsonYamlToolMessage::JsonToYaml,
                        false,
                        true,
                    ),
                ]
                .spacing(10),
                text("左侧输入源内容，转换结果会写入右侧。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(12)
        ),
        build_section_title("编辑"),
        settings_panel(
            column![
                row![
                    build_action_button(app, "交换左右", JsonYamlToolMessage::Swap, false, true,),
                    build_action_button(
                        app,
                        "清空左侧",
                        JsonYamlToolMessage::ClearLeft,
                        false,
                        true,
                    ),
                ]
                .spacing(10),
                row![
                    build_action_button(
                        app,
                        "清空右侧",
                        JsonYamlToolMessage::ClearRight,
                        false,
                        true,
                    ),
                    Space::new().width(Length::Fill),
                ]
                .spacing(10),
                row![
                    build_action_button(
                        app,
                        "复制左侧",
                        JsonYamlToolMessage::CopyLeft,
                        false,
                        false,
                    ),
                    build_action_button(
                        app,
                        "复制右侧",
                        JsonYamlToolMessage::CopyRight,
                        false,
                        false,
                    ),
                ]
                .spacing(10),
            ]
            .spacing(10)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_action_button<'a>(
    app: &'a App,
    label: &'static str,
    message: JsonYamlToolMessage,
    is_primary: bool,
    disable_while_loading: bool,
) -> Element<'a, Message> {
    let button = button(text(label).size(13)).padding([10, 12]).width(Length::Fill);
    let button = if disable_while_loading && app.json_yaml_loading {
        button
    } else {
        button.on_press(Message::JsonYamlTool(message))
    };

    if is_primary {
        button.style(primary_action_btn_style).into()
    } else {
        button.style(rounded_action_btn_style).into()
    }
}

fn build_editor_workspace<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let left = build_editor_card(app, size, EditorSide::Left);
    let right = build_editor_card(app, size, EditorSide::Right);

    let panels: Element<'a, Message> = if size.width >= 1040.0 {
        row![
            container(left).width(Length::Fill).height(Length::Fill),
            container(right).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![left, right].spacing(16).height(Length::Fill).into()
    };

    column![build_section_title("编辑区"), panels].spacing(12).height(Length::Fill).into()
}

#[derive(Clone, Copy)]
enum EditorSide {
    Left,
    Right,
}

fn build_editor_card<'a>(app: &'a App, size: Size, side: EditorSide) -> Element<'a, Message> {
    let (title, description, line_count, panel) = match side {
        EditorSide::Left => (
            "源内容",
            "左侧用于输入 JSON 或 YAML",
            app.json_yaml_left_editor.line_count().max(1),
            build_editor_panel(app, size, EditorSide::Left),
        ),
        EditorSide::Right => (
            "转换结果",
            "右侧显示转换后的目标格式",
            app.json_yaml_right_editor.line_count().max(1),
            build_editor_panel(app, size, EditorSide::Right),
        ),
    };

    column![
        row![
            column![
                text(title).size(14),
                text(description).size(12).style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            build_metric_badge(format!("{} 行", line_count)),
        ]
        .align_y(Alignment::Center)
        .spacing(12),
        settings_panel(panel).height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn build_editor_panel<'a>(app: &'a App, size: Size, side: EditorSide) -> Element<'a, Message> {
    match side {
        EditorSide::Left => {
            let editor = text_editor(&app.json_yaml_left_editor)
                .id(app.json_yaml_left_editor_id.clone())
                .placeholder("在左侧输入 JSON 或 YAML")
                .on_action(|action| {
                    Message::JsonYamlTool(JsonYamlToolMessage::LeftEditorAction(action))
                })
                .height(Length::Fill)
                .style(editor_style);

            let editor = wrap_with_context_menu(
                editor,
                TextEditorContextMenuState {
                    open: app.json_yaml_left_context_menu_open,
                    position: app.json_yaml_left_context_menu_pos,
                },
                |point| {
                    Message::JsonYamlTool(JsonYamlToolMessage::LeftOpenContextMenu {
                        x: point.x,
                        y: point.y,
                    })
                },
                TextEditorContextMenuMessages {
                    close: Message::JsonYamlTool(JsonYamlToolMessage::LeftCloseContextMenu),
                    copy: Message::JsonYamlTool(JsonYamlToolMessage::LeftContextMenuCopy),
                    cut: Message::JsonYamlTool(JsonYamlToolMessage::LeftContextMenuCut),
                    paste: Message::JsonYamlTool(JsonYamlToolMessage::LeftContextMenuPaste),
                    delete: Message::JsonYamlTool(JsonYamlToolMessage::LeftContextMenuDelete),
                },
            );

            text_editor_scroll_panel(
                editor,
                size,
                TextEditorScrollPanelMetrics {
                    viewport_padding: 24.0,
                    line_height: app.current_line_height,
                    line_count: app.json_yaml_left_editor.line_count(),
                    scroll_top_line: app.json_yaml_left_scroll_top_line,
                },
                |delta, viewport_height| {
                    Message::JsonYamlTool(JsonYamlToolMessage::LeftEditorWheelScrolled {
                        delta,
                        viewport_height,
                    })
                },
                |top_line, viewport_height| {
                    Message::JsonYamlTool(JsonYamlToolMessage::LeftScrollbarChanged {
                        top_line,
                        viewport_height,
                    })
                },
            )
        }
        EditorSide::Right => {
            let editor = text_editor(&app.json_yaml_right_editor)
                .id(app.json_yaml_right_editor_id.clone())
                .placeholder("右侧显示转换结果")
                .on_action(|action| {
                    Message::JsonYamlTool(JsonYamlToolMessage::RightEditorAction(action))
                })
                .height(Length::Fill)
                .style(editor_style);

            let editor = wrap_with_context_menu(
                editor,
                TextEditorContextMenuState {
                    open: app.json_yaml_right_context_menu_open,
                    position: app.json_yaml_right_context_menu_pos,
                },
                |point| {
                    Message::JsonYamlTool(JsonYamlToolMessage::RightOpenContextMenu {
                        x: point.x,
                        y: point.y,
                    })
                },
                TextEditorContextMenuMessages {
                    close: Message::JsonYamlTool(JsonYamlToolMessage::RightCloseContextMenu),
                    copy: Message::JsonYamlTool(JsonYamlToolMessage::RightContextMenuCopy),
                    cut: Message::JsonYamlTool(JsonYamlToolMessage::RightContextMenuCut),
                    paste: Message::JsonYamlTool(JsonYamlToolMessage::RightContextMenuPaste),
                    delete: Message::JsonYamlTool(JsonYamlToolMessage::RightContextMenuDelete),
                },
            );

            text_editor_scroll_panel(
                editor,
                size,
                TextEditorScrollPanelMetrics {
                    viewport_padding: 24.0,
                    line_height: app.current_line_height,
                    line_count: app.json_yaml_right_editor.line_count(),
                    scroll_top_line: app.json_yaml_right_scroll_top_line,
                },
                |delta, viewport_height| {
                    Message::JsonYamlTool(JsonYamlToolMessage::RightEditorWheelScrolled {
                        delta,
                        viewport_height,
                    })
                },
                |top_line, viewport_height| {
                    Message::JsonYamlTool(JsonYamlToolMessage::RightScrollbarChanged {
                        top_line,
                        viewport_height,
                    })
                },
            )
        }
    }
}

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
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
        Loading,
        Success,
        Idle,
    }

    let (label, tone): (String, StatusTone) = if app.json_yaml_loading {
        ("处理中".to_string(), StatusTone::Loading)
    } else if let Some(message) = &app.json_yaml_notification {
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

fn editor_style(theme: &Theme, _status: iced::widget::text_editor::Status) -> text_editor::Style {
    let palette = theme.extended_palette();
    text_editor::Style {
        background: palette.background.base.color.into(),
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
        value: theme.palette().text,
        selection: theme.palette().primary.scale_alpha(0.30),
        placeholder: theme.palette().text.scale_alpha(0.55),
    }
}
#[cfg(test)]
#[path = "json_yaml_tool_tests.rs"]
mod json_yaml_tool_tests;
