//! 密码生成工具视图模块
//!
//! 本模块提供随机密码生成器的用户界面组件，
//! 在布局与交互上对齐 JSON 美化工具的设计风格。

use crate::app::components::system_settings_common::{
    SETTINGS_LABEL_WIDTH, danger_action_btn_style, primary_action_btn_style,
    rounded_action_btn_style, settings_checkbox_style, settings_divider, settings_muted_text_style,
    settings_page_intro, settings_panel, settings_panel_style, settings_section_card,
    settings_text_input_style, settings_value_badge,
};
use crate::app::components::text_editor_context_menu::{
    TextEditorContextMenuMessages, TextEditorContextMenuState, wrap_with_context_menu,
};
use crate::app::components::text_editor_scroll_panel::{
    TextEditorScrollPanelMetrics, text_editor_scroll_panel,
};
use crate::app::message::{
    PasswordToolMessage,
    password_tool::{DIGITS_CHARSET, LOWERCASE_CHARSET, SPECIAL_CHARSET, UPPERCASE_CHARSET},
};
use crate::app::{App, Message};
use iced::widget::{
    Space, button, checkbox, column, container, responsive, row, text, text_editor, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            text("随机密码生成器").size(20),
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
    let controls = build_controls_panel(app, size);
    let editor = build_editor_card(app, size);

    if size.width >= 960.0 {
        row![
            container(editor).width(Length::FillPortion(3)).height(Length::Fill),
            container(controls).width(Length::Fixed(340.0)).height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    } else {
        column![controls, editor].spacing(16).height(Length::Fill).into()
    }
}

fn build_controls_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let compact = size.width < 920.0;
    let selected_group_count = selected_group_count(app);
    let pool_size = selected_pool_size(app);
    let output_count = output_password_count(app);
    let normalized_length = normalized_length(app);
    let normalized_count = normalized_count(app);
    let output_empty = app.pwd_output_editor.text().trim().is_empty();
    let (hint, hint_is_error) = password_rule_hint(app);

    column![
        settings_page_intro(
            "生成参数",
            "表单样式对齐系统设置常规页，统一查看字符集、长度和批量生成参数。"
        ),
        settings_section_card("概览", "快速确认当前字符池规模、目标输出与已生成结果。"),
        settings_panel(
            column![
                build_overview_row(
                    "已选字符组",
                    "当前启用的字符类别数量。",
                    format!("{} 组", selected_group_count),
                    compact,
                ),
                settings_divider(),
                build_overview_row(
                    "字符池大小",
                    "按已选字符集汇总可用字符数。",
                    format!("{} 字符", pool_size),
                    compact,
                ),
                settings_divider(),
                build_overview_row(
                    "目标输出",
                    "本次计划生成的密码条数。",
                    format!("{} 条", normalized_count),
                    compact,
                ),
                settings_divider(),
                build_overview_row(
                    "当前结果",
                    "结果编辑区内当前非空密码条数。",
                    format!("{} 条", output_count),
                    compact,
                ),
            ]
            .spacing(0)
        ),
        settings_section_card("字符集", "至少选择一种字符集；每条密码都会覆盖全部已选类别。"),
        settings_panel(
            column![
                build_charset_row(
                    "数字",
                    "0-9 数字字符。",
                    DIGITS_CHARSET,
                    app.pwd_digits,
                    PasswordToolMessage::ToggleDigits,
                    compact,
                ),
                settings_divider(),
                build_charset_row(
                    "小写字母",
                    "a-z 小写英文字母。",
                    LOWERCASE_CHARSET,
                    app.pwd_lowercase,
                    PasswordToolMessage::ToggleLowercase,
                    compact,
                ),
                settings_divider(),
                build_charset_row(
                    "大写字母",
                    "A-Z 大写英文字母。",
                    UPPERCASE_CHARSET,
                    app.pwd_uppercase,
                    PasswordToolMessage::ToggleUppercase,
                    compact,
                ),
                settings_divider(),
                build_charset_row(
                    "特殊符号",
                    "常见可打印特殊符号。",
                    SPECIAL_CHARSET,
                    app.pwd_special,
                    PasswordToolMessage::ToggleSpecial,
                    compact,
                ),
            ]
            .spacing(0)
        ),
        settings_section_card("参数与操作", "设置长度和数量后即可批量生成、复制或清空结果。"),
        settings_panel(
            column![
                build_input_row(
                    "密码长度",
                    "建议至少 12 位，并且不小于已选字符组数量。",
                    &app.pwd_length_input,
                    "12",
                    PasswordToolMessage::LengthChanged,
                    compact,
                ),
                settings_divider(),
                build_input_row(
                    "生成数量",
                    "单次最多 500 条。",
                    &app.pwd_count_input,
                    "1",
                    PasswordToolMessage::CountChanged,
                    compact,
                ),
                settings_divider(),
                build_overview_row(
                    "当前规格",
                    "根据输入实时计算的单条长度与输出数量。",
                    format!("{} 位 / {} 条", normalized_length, normalized_count),
                    compact,
                ),
                settings_divider(),
                build_hint_row(hint, hint_is_error, compact),
                settings_divider(),
                build_actions_row(output_empty, compact),
            ]
            .spacing(0)
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn build_overview_row<'a>(
    label: &'a str,
    description: &'a str,
    value: String,
    compact: bool,
) -> Element<'a, Message> {
    build_form_row(label, description, settings_value_badge(value), compact)
}

fn build_charset_row<'a>(
    title: &'static str,
    description: &'static str,
    preview: &'static str,
    enabled: bool,
    on_toggle: fn(bool) -> PasswordToolMessage,
    compact: bool,
) -> Element<'a, Message> {
    let checkbox_control = row![
        settings_value_badge(preview),
        checkbox(enabled)
            .label("启用")
            .spacing(10)
            .on_toggle(move |next| Message::PasswordTool(on_toggle(next)))
            .style(settings_checkbox_style),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    build_form_row(title, description, checkbox_control, compact)
}

fn build_input_row<'a>(
    label: &'a str,
    description: &'a str,
    value: &'a str,
    placeholder: &'static str,
    on_input: fn(String) -> PasswordToolMessage,
    compact: bool,
) -> Element<'a, Message> {
    build_form_row(
        label,
        description,
        text_input(placeholder, value)
            .width(if compact { Length::Fill } else { Length::Fixed(160.0) })
            .padding([10, 12])
            .size(13)
            .style(settings_text_input_style)
            .on_input(move |next| Message::PasswordTool(on_input(next))),
        compact,
    )
}

fn build_hint_row<'a>(hint: String, hint_is_error: bool, compact: bool) -> Element<'a, Message> {
    build_form_row(
        "生成规则",
        "根据当前字符集和长度配置得到的即时校验结果。",
        text(hint).size(12).style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::text::Style {
                color: Some(if hint_is_error {
                    palette.danger.base.color
                } else {
                    settings_muted_text_style(theme)
                        .color
                        .unwrap_or(theme.palette().text.scale_alpha(0.72))
                }),
            }
        }),
        compact,
    )
}

fn build_actions_row<'a>(output_empty: bool, compact: bool) -> Element<'a, Message> {
    let actions: Element<'a, Message> = if compact {
        column![
            build_action_button(
                "生成密码",
                PasswordToolMessage::Generate,
                primary_action_btn_style,
                false,
            ),
            build_action_button(
                "复制结果",
                PasswordToolMessage::Copy,
                rounded_action_btn_style,
                output_empty,
            ),
            build_action_button(
                "清空结果",
                PasswordToolMessage::Clear,
                danger_action_btn_style,
                output_empty,
            ),
        ]
        .spacing(10)
        .into()
    } else {
        row![
            build_action_button(
                "生成密码",
                PasswordToolMessage::Generate,
                primary_action_btn_style,
                false,
            ),
            build_action_button(
                "复制结果",
                PasswordToolMessage::Copy,
                rounded_action_btn_style,
                output_empty,
            ),
            build_action_button(
                "清空结果",
                PasswordToolMessage::Clear,
                danger_action_btn_style,
                output_empty,
            ),
        ]
        .spacing(10)
        .into()
    };

    build_form_row("快捷操作", "生成、复制或重置当前结果。", actions, compact)
}

fn build_form_row<'a>(
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
        row![
            intro.width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
            container(control.into()).width(Length::Fill),
        ]
        .spacing(22)
        .align_y(Alignment::Center)
        .into()
    };

    container(layout).padding([14, 0]).width(Length::Fill).into()
}

fn build_action_button<'a>(
    label: &'static str,
    msg: PasswordToolMessage,
    style: fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style,
    disabled: bool,
) -> Element<'a, Message> {
    let button = button(text(label).size(13)).padding([10, 12]).width(Length::Fill);
    let button = if disabled { button } else { button.on_press(Message::PasswordTool(msg)) };
    button.style(style).into()
}

fn build_editor_card<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let editor_panel = build_editor_panel(app, size);

    column![
        build_section_title("结果"),
        settings_panel(
            column![
                row![
                    text("输出").size(13).width(Length::Fill),
                    build_metric_badge(format!("{} 条", output_password_count(app))),
                    build_metric_badge(format!("{} 位/条", normalized_length(app))),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
                text("支持右键复制、剪切、粘贴与删除。").size(12).style(settings_muted_text_style),
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

fn build_editor_panel<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let editor = text_editor(&app.pwd_output_editor)
        .id(app.pwd_editor_id.clone())
        .placeholder("生成后的密码会显示在这里")
        .on_action(|action| Message::PasswordTool(PasswordToolMessage::EditorAction(action)))
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
            open: app.pwd_context_menu_open,
            position: app.pwd_context_menu_pos,
        },
        |point| {
            Message::PasswordTool(PasswordToolMessage::OpenContextMenu { x: point.x, y: point.y })
        },
        TextEditorContextMenuMessages {
            close: Message::PasswordTool(PasswordToolMessage::CloseContextMenu),
            copy: Message::PasswordTool(PasswordToolMessage::ContextMenuCopy),
            cut: Message::PasswordTool(PasswordToolMessage::ContextMenuCut),
            paste: Message::PasswordTool(PasswordToolMessage::ContextMenuPaste),
            delete: Message::PasswordTool(PasswordToolMessage::ContextMenuDelete),
        },
    );

    text_editor_scroll_panel(
        editor,
        size,
        TextEditorScrollPanelMetrics {
            viewport_padding: 24.0,
            line_height: app.current_line_height,
            line_count: app.pwd_output_editor.line_count(),
            scroll_top_line: app.pwd_scroll_top_line,
        },
        |delta, viewport_height| {
            Message::PasswordTool(PasswordToolMessage::EditorWheelScrolled {
                delta,
                viewport_height,
            })
        },
        |top_line, viewport_height| {
            Message::PasswordTool(PasswordToolMessage::ScrollbarChanged {
                top_line,
                viewport_height,
            })
        },
    )
}

fn build_section_title<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(14).into()
}

fn build_status_badge<'a>(app: &App) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum StatusTone {
        Success,
        Error,
        Idle,
    }

    let (label, tone): (String, StatusTone) = if let Some(message) = &app.pwd_notification {
        let tone =
            if app.pwd_notification_is_error { StatusTone::Error } else { StatusTone::Success };
        (message.as_str().to_owned(), tone)
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

fn selected_group_count(app: &App) -> usize {
    usize::from(app.pwd_digits)
        + usize::from(app.pwd_lowercase)
        + usize::from(app.pwd_uppercase)
        + usize::from(app.pwd_special)
}

fn selected_pool_size(app: &App) -> usize {
    usize::from(app.pwd_digits) * DIGITS_CHARSET.len()
        + usize::from(app.pwd_lowercase) * LOWERCASE_CHARSET.len()
        + usize::from(app.pwd_uppercase) * UPPERCASE_CHARSET.len()
        + usize::from(app.pwd_special) * SPECIAL_CHARSET.len()
}

fn normalized_length(app: &App) -> usize {
    app.pwd_length_input.parse::<usize>().unwrap_or(12).max(1)
}

fn normalized_count(app: &App) -> usize {
    app.pwd_count_input.parse::<usize>().unwrap_or(1).clamp(1, 500)
}

fn output_password_count(app: &App) -> usize {
    app.pwd_output_editor.text().lines().filter(|line: &&str| !line.trim().is_empty()).count()
}

fn password_rule_hint(app: &App) -> (String, bool) {
    let selected_groups = selected_group_count(app);
    if selected_groups == 0 {
        return ("至少选择一种字符集后才能生成密码。".to_string(), true);
    }

    let length = normalized_length(app);
    if length < selected_groups {
        return (
            format!("当前长度不足，至少需要 {} 位才能覆盖所有已选字符集。", selected_groups),
            true,
        );
    }

    (format!("每条密码都会至少包含 {} 类已选字符。", selected_groups), false)
}
#[cfg(test)]
#[path = "password_tool_tests.rs"]
mod password_tool_tests;
