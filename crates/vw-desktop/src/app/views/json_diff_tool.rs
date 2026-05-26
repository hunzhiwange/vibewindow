//! JSON 比对工具视图模块
//!
//! 该视图采用与 JSON 美化工具一致的新版卡片式布局，提供：
//! - 双编辑器响应式工作区
//! - 左右独立滚动与右键菜单
//! - 左右就地格式化、复制、清空
//! - 差异结果卡片化展示
//! - 示例数据与比对说明

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_muted_text_style, settings_panel,
    settings_panel_style, settings_section_card,
};
use crate::app::components::text_editor_context_menu::{
    TextEditorContextMenuMessages, TextEditorContextMenuState, wrap_with_context_menu,
};
use crate::app::components::text_editor_scroll_panel::{
    TextEditorScrollPanelMetrics, text_editor_scroll_panel,
};
use crate::app::message::JsonDiffToolMessage;
use crate::app::message::json_diff_tool::JsonDiffEntry;
use crate::app::{App, Message};
use iced::widget::{
    Space, button, column, container, responsive, row, scrollable, text, text_editor,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

struct JsonDiffExample {
    title: &'static str,
    left: &'static str,
    right: &'static str,
}

const JSON_DIFF_EXAMPLES: [JsonDiffExample; 2] = [
    JsonDiffExample {
        title: "用户资料变更",
        left: r#"{
  "id": 1001,
  "name": "张三",
  "age": 28,
  "email": "zhangsan@example.com",
  "address": {
    "city": "上海",
    "district": "黄浦区",
    "street": "南京路88号"
  },
  "tags": ["前端", "JavaScript", "Vue"],
  "nickname": ""
}"#,
        right: r#"{
  "id": 1001,
  "name": "张三",
  "age": 30,
  "email": "zhangsan@example.com",
  "address": {
    "city": "上海",
    "street": "淮海路408号"
  },
  "tags": ["前端", "JavaScript", "React"],
  "nickname": ""
}"#,
    },
    JsonDiffExample {
        title: "配置字段增减",
        left: r#"{
  "service": "gateway",
  "enabled": true,
  "limits": {
    "qps": 120,
    "burst": 10
  },
  "regions": ["cn", "us"]
}"#,
        right: r#"{
  "service": "gateway",
  "enabled": false,
  "limits": {
    "qps": 120
  },
  "regions": ["cn", "eu", "us"],
  "owner": "platform"
}"#,
    },
];

#[derive(Clone, Copy)]
enum EditorSide {
    Left,
    Right,
}

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("JSON对比工具").size(20),
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
    let main = build_main_panel(app, size);
    let controls = build_controls_panel(app);

    if size.width >= 1120.0 {
        row![
            container(main).width(Length::FillPortion(3)).height(Length::Fill),
            container(controls).width(Length::Fixed(320.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![controls, main].spacing(16).height(Length::Fill).into()
    }
}

fn build_main_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    column![
        container(build_editors_panel(app, size)).height(Length::FillPortion(3)),
        container(build_results_panel(app)).height(Length::FillPortion(4)),
    ]
    .spacing(16)
    .height(Length::Fill)
    .into()
}

fn build_controls_panel<'a>(app: &'a App) -> Element<'a, Message> {
    column![
        build_section_title("概览"),
        settings_panel(
            column![
                build_metric_row("差异结果", format!("{} 项", app.json_diff_results.len())),
                build_metric_row(
                    "左侧行数",
                    format!("{} 行", app.json_diff_left_editor.line_count().max(1))
                ),
                build_metric_row(
                    "右侧行数",
                    format!("{} 行", app.json_diff_right_editor.line_count().max(1))
                ),
            ]
            .spacing(12)
        ),
        build_section_title("操作"),
        settings_panel(
            column![
                build_action_button(app, "开始对比", JsonDiffToolMessage::Compare, true),
                build_action_button(app, "双侧格式化", JsonDiffToolMessage::FormatBoth, false),
                row![
                    build_action_button(app, "交换左右", JsonDiffToolMessage::Swap, false),
                    build_action_button(app, "复制左侧", JsonDiffToolMessage::CopyLeft, false),
                ]
                .spacing(10),
                row![
                    build_action_button(app, "复制右侧", JsonDiffToolMessage::CopyRight, false),
                    Space::new().width(Length::Fill),
                ]
                .spacing(10),
            ]
            .spacing(10)
        ),
        build_section_title("示例"),
        settings_panel(build_examples(app)),
        build_section_title("说明"),
        column![
            settings_section_card(
                "结构比对",
                "基于 JSON 语义进行递归比对，忽略空白格式和对象键顺序。"
            ),
            settings_section_card(
                "结果可读",
                "对象和数组差异会按美化后的多行 JSON 展示，便于直接定位问题。"
            ),
        ]
        .spacing(10),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_examples<'a>(app: &'a App) -> Element<'a, Message> {
    let mut content = column![];

    for example in JSON_DIFF_EXAMPLES {
        let row = row![
            text(example.title).size(13).width(Length::Fill),
            build_action_button(
                app,
                "填左右",
                JsonDiffToolMessage::InsertPair(
                    example.left.to_string(),
                    example.right.to_string()
                ),
                false
            ),
            build_action_button(
                app,
                "填左",
                JsonDiffToolMessage::InsertLeft(example.left.to_string()),
                false
            ),
            build_action_button(
                app,
                "填右",
                JsonDiffToolMessage::InsertRight(example.right.to_string()),
                false
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        content = content.push(row);
    }

    content.spacing(10).into()
}

fn build_editors_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let left = build_editor_card(app, size, EditorSide::Left);
    let right = build_editor_card(app, size, EditorSide::Right);
    let layout: Element<'a, Message> = if size.width >= 1320.0 {
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

    column![build_section_title("编辑区"), layout].spacing(12).height(Length::Fill).into()
}

fn build_editor_card<'a>(app: &'a App, size: Size, side: EditorSide) -> Element<'a, Message> {
    let (title, line_count, format_message, copy_message, clear_message, editor_panel) = match side
    {
        EditorSide::Left => (
            "左侧 JSON",
            app.json_diff_left_editor.line_count().max(1),
            JsonDiffToolMessage::FormatLeft,
            JsonDiffToolMessage::CopyLeft,
            JsonDiffToolMessage::ClearLeft,
            build_editor_panel(app, size, EditorSide::Left),
        ),
        EditorSide::Right => (
            "右侧 JSON",
            app.json_diff_right_editor.line_count().max(1),
            JsonDiffToolMessage::FormatRight,
            JsonDiffToolMessage::CopyRight,
            JsonDiffToolMessage::ClearRight,
            build_editor_panel(app, size, EditorSide::Right),
        ),
    };

    settings_panel(
        column![
            row![
                text(title).size(13).width(Length::Fill),
                build_metric_badge(format!("{line_count} 行")),
                build_action_button(app, "格式化", format_message, false),
                build_action_button(app, "复制", copy_message, false),
                build_action_button(app, "清空", clear_message, false),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            editor_panel,
        ]
        .spacing(14),
    )
    .height(Length::Fill)
    .into()
}

fn build_editor_panel<'a>(app: &'a App, size: Size, side: EditorSide) -> Element<'a, Message> {
    match side {
        EditorSide::Left => {
            let editor = text_editor(&app.json_diff_left_editor)
                .id(app.json_diff_left_editor_id.clone())
                .placeholder("输入左侧 JSON")
                .on_action(|action| {
                    Message::JsonDiffTool(JsonDiffToolMessage::LeftEditorAction(action))
                })
                .height(Length::Fill)
                .style(editor_style);

            let editor = wrap_with_context_menu(
                editor,
                TextEditorContextMenuState {
                    open: app.json_diff_left_context_menu_open,
                    position: app.json_diff_left_context_menu_pos,
                },
                |point| {
                    Message::JsonDiffTool(JsonDiffToolMessage::LeftOpenContextMenu {
                        x: point.x,
                        y: point.y,
                    })
                },
                TextEditorContextMenuMessages {
                    close: Message::JsonDiffTool(JsonDiffToolMessage::LeftCloseContextMenu),
                    copy: Message::JsonDiffTool(JsonDiffToolMessage::LeftContextMenuCopy),
                    cut: Message::JsonDiffTool(JsonDiffToolMessage::LeftContextMenuCut),
                    paste: Message::JsonDiffTool(JsonDiffToolMessage::LeftContextMenuPaste),
                    delete: Message::JsonDiffTool(JsonDiffToolMessage::LeftContextMenuDelete),
                },
            );

            text_editor_scroll_panel(
                editor,
                size,
                TextEditorScrollPanelMetrics {
                    viewport_padding: 24.0,
                    line_height: app.current_line_height,
                    line_count: app.json_diff_left_editor.line_count(),
                    scroll_top_line: app.json_diff_left_scroll_top_line,
                },
                |delta, viewport_height| {
                    Message::JsonDiffTool(JsonDiffToolMessage::LeftEditorWheelScrolled {
                        delta,
                        viewport_height,
                    })
                },
                |top_line, viewport_height| {
                    Message::JsonDiffTool(JsonDiffToolMessage::LeftScrollbarChanged {
                        top_line,
                        viewport_height,
                    })
                },
            )
        }
        EditorSide::Right => {
            let editor = text_editor(&app.json_diff_right_editor)
                .id(app.json_diff_right_editor_id.clone())
                .placeholder("输入右侧 JSON")
                .on_action(|action| {
                    Message::JsonDiffTool(JsonDiffToolMessage::RightEditorAction(action))
                })
                .height(Length::Fill)
                .style(editor_style);

            let editor = wrap_with_context_menu(
                editor,
                TextEditorContextMenuState {
                    open: app.json_diff_right_context_menu_open,
                    position: app.json_diff_right_context_menu_pos,
                },
                |point| {
                    Message::JsonDiffTool(JsonDiffToolMessage::RightOpenContextMenu {
                        x: point.x,
                        y: point.y,
                    })
                },
                TextEditorContextMenuMessages {
                    close: Message::JsonDiffTool(JsonDiffToolMessage::RightCloseContextMenu),
                    copy: Message::JsonDiffTool(JsonDiffToolMessage::RightContextMenuCopy),
                    cut: Message::JsonDiffTool(JsonDiffToolMessage::RightContextMenuCut),
                    paste: Message::JsonDiffTool(JsonDiffToolMessage::RightContextMenuPaste),
                    delete: Message::JsonDiffTool(JsonDiffToolMessage::RightContextMenuDelete),
                },
            );

            text_editor_scroll_panel(
                editor,
                size,
                TextEditorScrollPanelMetrics {
                    viewport_padding: 24.0,
                    line_height: app.current_line_height,
                    line_count: app.json_diff_right_editor.line_count(),
                    scroll_top_line: app.json_diff_right_scroll_top_line,
                },
                |delta, viewport_height| {
                    Message::JsonDiffTool(JsonDiffToolMessage::RightEditorWheelScrolled {
                        delta,
                        viewport_height,
                    })
                },
                |top_line, viewport_height| {
                    Message::JsonDiffTool(JsonDiffToolMessage::RightScrollbarChanged {
                        top_line,
                        viewport_height,
                    })
                },
            )
        }
    }
}

fn build_results_panel<'a>(app: &'a App) -> Element<'a, Message> {
    let body = if app.json_diff_results.is_empty() {
        build_empty_results(app)
    } else {
        let mut rows = column![];
        for entry in &app.json_diff_results {
            rows = rows.push(build_diff_row(entry));
        }
        scrollable(rows.spacing(12)).height(Length::Fill).into()
    };

    column![
        build_section_title("差异结果"),
        settings_panel(
            column![
                row![
                    text("结构化差异").size(13).width(Length::Fill),
                    build_metric_badge(format!("{} 项", app.json_diff_results.len())),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("对象键会按稳定顺序展开，缺失字段会明确标记在左侧或右侧。")
                    .size(12)
                    .style(settings_muted_text_style),
                container(body).height(Length::Fill),
            ]
            .spacing(14)
        )
        .height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn build_empty_results<'a>(app: &'a App) -> Element<'a, Message> {
    let message = if app.json_diff_loading {
        "正在对比两侧 JSON…"
    } else if app.json_diff_notification_is_error {
        app.json_diff_notification.as_deref().unwrap_or("JSON 对比失败")
    } else {
        "输入左右两侧 JSON 后点击“开始对比”，这里会展示结构差异。"
    };

    let description = if app.json_diff_notification_is_error {
        "优先修正提示中的 JSON 解析位置，再重新执行对比。"
    } else {
        "如果只是格式不一致，可以先使用每侧顶部的“格式化”或右侧面板中的“双侧格式化”。"
    };

    container(
        column![
            text(message).size(14),
            text(description).size(12).style(settings_muted_text_style),
        ]
        .spacing(6)
        .align_x(Alignment::Start),
    )
    .padding([18, 16])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let is_error = app.json_diff_notification_is_error;

        iced::widget::container::Style {
            background: Some(
                if is_error {
                    palette.danger.weak.color.scale_alpha(0.38)
                } else {
                    palette.background.weak.color.scale_alpha(0.22)
                }
                .into(),
            ),
            border: Border {
                width: 1.0,
                color: if is_error {
                    palette.danger.base.color.scale_alpha(0.42)
                } else {
                    palette.background.strong.color.scale_alpha(0.72)
                },
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn build_diff_row<'a>(entry: &'a JsonDiffEntry) -> Element<'a, Message> {
    let kind_label = diff_kind_label(entry);

    container(
        column![
            row![
                build_path_badge(&entry.path),
                Space::new().width(Length::Fill),
                build_kind_badge(kind_label),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                container(build_value_column("左侧", entry.left.as_deref(), entry.left.is_none()))
                    .width(Length::Fill),
                container(build_value_column(
                    "右侧",
                    entry.right.as_deref(),
                    entry.right.is_none()
                ))
                .width(Length::Fill),
            ]
            .spacing(10),
        ]
        .spacing(10),
    )
    .padding([14, 14])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::container::Style {
            background: Some(
                if is_dark {
                    palette.background.base.color.scale_alpha(0.80)
                } else {
                    Color::WHITE.scale_alpha(0.84)
                }
                .into(),
            ),
            border: Border {
                width: 1.0,
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.86)
                } else {
                    Color::from_rgba8(15, 23, 42, 0.08)
                },
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn build_value_column<'a>(
    label: &'a str,
    value: Option<&'a str>,
    missing: bool,
) -> Element<'a, Message> {
    column![
        text(label).size(12).style(settings_muted_text_style),
        build_value_card(value.unwrap_or("缺失"), missing),
    ]
    .spacing(6)
    .into()
}

fn build_value_card<'a>(value: &'a str, missing: bool) -> Element<'a, Message> {
    container(text(value).size(13))
        .padding([12, 12])
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let is_dark = theme.palette().background.r
                + theme.palette().background.g
                + theme.palette().background.b
                < 1.5;

            iced::widget::container::Style {
                background: Some(
                    if missing {
                        palette.danger.weak.color.scale_alpha(if is_dark { 0.45 } else { 0.70 })
                    } else if is_dark {
                        palette.background.weak.color.scale_alpha(0.26)
                    } else {
                        Color::from_rgba8(248, 250, 252, 0.94)
                    }
                    .into(),
                ),
                border: Border {
                    width: 1.0,
                    color: if missing {
                        palette.danger.base.color.scale_alpha(0.36)
                    } else if is_dark {
                        palette.background.strong.color.scale_alpha(0.72)
                    } else {
                        Color::from_rgba8(148, 163, 184, 0.18)
                    },
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_path_badge<'a>(path: &'a str) -> Element<'a, Message> {
    container(text(path).size(12).style(settings_muted_text_style))
        .padding([6, 10])
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            iced::widget::container::Style {
                background: Some(palette.background.weak.color.scale_alpha(0.24).into()),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color.scale_alpha(0.70),
                    radius: 999.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn build_kind_badge<'a>(label: &'a str) -> Element<'a, Message> {
    container(
        text(label)
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style { color: Some(Color::WHITE) }),
    )
    .padding([6, 10])
    .style(move |_theme: &Theme| {
        let background = match label {
            "仅左侧" => Color::from_rgba8(37, 99, 235, 0.92),
            "仅右侧" => Color::from_rgba8(217, 119, 6, 0.92),
            _ => Color::from_rgba8(22, 163, 74, 0.92),
        };

        iced::widget::container::Style {
            background: Some(Background::Color(background)),
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 999.0.into() },
            ..Default::default()
        }
    })
    .into()
}

fn build_action_button<'a>(
    app: &'a App,
    label: &'static str,
    message: JsonDiffToolMessage,
    is_primary: bool,
) -> Element<'a, Message> {
    let button = button(text(label).size(13)).padding([10, 12]).width(Length::Fill);
    let button = if app.json_diff_loading {
        button
    } else {
        button.on_press(Message::JsonDiffTool(message))
    };

    if is_primary {
        button.style(primary_action_btn_style).into()
    } else {
        button.style(rounded_action_btn_style).into()
    }
}

fn build_metric_row<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![text(label).size(13).width(Length::Fill), build_metric_badge(value),]
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

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
}

fn build_status_badge<'a>(app: &App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Loading,
        Success,
        Error,
        Idle,
    }

    let (label, tone): (String, StatusTone) = if app.json_diff_loading {
        ("处理中".to_string(), StatusTone::Loading)
    } else if let Some(message) = &app.json_diff_notification {
        if app.json_diff_notification_is_error {
            (message.as_str().to_owned(), StatusTone::Error)
        } else {
            (message.as_str().to_owned(), StatusTone::Success)
        }
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
                StatusTone::Error => Color::from_rgba8(220, 38, 38, 0.94),
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

fn diff_kind_label(entry: &JsonDiffEntry) -> &'static str {
    match (entry.left.is_some(), entry.right.is_some()) {
        (true, false) => "仅左侧",
        (false, true) => "仅右侧",
        _ => "值变更",
    }
}

fn editor_style(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
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
#[path = "json_diff_tool_tests.rs"]
mod json_diff_tool_tests;
