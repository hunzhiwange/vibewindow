//! 渲染应用视图中的模态窗口。
//! 本模块只描述模态内容和交互消息，不直接承担持久化策略。

use crate::app::components::system_settings_common::{
    rounded_action_btn_style, settings_muted_text_style, settings_panel, settings_section_card,
    settings_text_input_style,
};
use crate::app::views::design::properties::color_picker::parse_color;
use iced::widget::{Image, Space, button, column, container, mouse_area, row, text, text_input};
use iced::{Background, Color, Element, Length, Theme};

use super::common::{icon_image_handle, parse_hex_color};
use super::{App, Message};

/// 模块内可见函数，执行 general_tab 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn general_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let name_input = text_input("项目名称", &app.project_edit_name)
        .on_input(|value| {
            Message::Project(crate::app::message::project::ProjectMessage::ProjectEditNameChanged(
                value,
            ))
        })
        .padding([10, 12])
        .size(13)
        .style(settings_text_input_style);
    let icon_input =
        text_input("图标文本或图片路径（支持 file:/// 或本地绝对路径）", &app.project_edit_icon)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditIconChanged(value),
                )
            })
            .padding([10, 12])
            .size(13)
            .style(settings_text_input_style);

    let icon_color_presets = [
        "#60a5fa", "#34d399", "#f59e0b", "#f97316", "#ef4444", "#ec4899", "#8b5cf6", "#6366f1",
        "#22d3ee", "#14b8a6", "#a3e635", "#94a3b8",
    ];
    let mut preset_grid = column![].spacing(6);
    let mut preset_row = row![].spacing(6);
    let mut row_count = 0usize;
    let selected_preset =
        app.project_edit_icon_color.trim().trim_start_matches('#').to_ascii_lowercase();
    for preset in icon_color_presets {
        let preset_color = parse_hex_color(preset).unwrap_or(Color::from_rgb8(96, 165, 250));
        let is_selected = selected_preset == preset.trim_start_matches('#').to_ascii_lowercase();
        let preset_button = button(Space::new().width(18.0).height(18.0))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditIconColorPresetSelected(
                    preset.to_string(),
                ),
            ))
            .padding(0)
            .style(move |theme: &Theme, _status| {
                let palette = theme.extended_palette();
                let border_color = if is_selected {
                    palette.primary.base.color
                } else {
                    palette.background.strong.color
                };
                iced::widget::button::Style {
                    background: Some(Background::Color(preset_color)),
                    border: iced::Border { width: 1.0, color: border_color, radius: 4.0.into() },
                    ..Default::default()
                }
            });
        preset_row = preset_row.push(preset_button);
        row_count += 1;
        if row_count == 6 {
            preset_grid = preset_grid.push(preset_row);
            preset_row = row![].spacing(6);
            row_count = 0;
        }
    }
    if row_count > 0 {
        preset_grid = preset_grid.push(preset_row);
    }

    let icon_color_presets_row =
        column![text("预设颜色").size(12).style(settings_muted_text_style), preset_grid,]
            .spacing(4);
    let current_icon_color = parse_color(&app.project_edit_icon_color)
        .or_else(|| parse_hex_color(&app.project_edit_icon_color))
        .unwrap_or(Color::from_rgb8(96, 165, 250));
    let color_preview_btn = button(
        row![
            container(Space::new().width(16).height(16)).style(move |_: &Theme| {
                iced::widget::container::Style {
                    background: Some(current_icon_color.into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                }
            }),
            text("选择颜色").size(12)
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .on_press(Message::Project(
        crate::app::message::project::ProjectMessage::ProjectEditIconColorPickerToggled,
    ))
    .padding([6, 10])
    .style(rounded_action_btn_style);

    let preview_color =
        parse_hex_color(&app.project_edit_icon_color).unwrap_or(Color::from_rgb8(96, 165, 250));
    let preview_label =
        app.project_edit_icon.trim().chars().next().map(|ch| ch.to_string()).unwrap_or_else(|| {
            app.project_edit_name
                .chars()
                .next()
                .map(|ch| ch.to_string())
                .unwrap_or_else(|| "P".to_string())
        });
    let preview_image = icon_image_handle(&app.project_edit_icon);
    let has_preview_image = preview_image.is_some();
    let preview_badge_content: Element<'_, Message> = if let Some(handle) = preview_image {
        Image::new(handle)
            .content_fit(iced::ContentFit::Cover)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        text(preview_label).size(68).color(Color::WHITE).into()
    };
    let preview_badge = container(preview_badge_content)
        .width(Length::Fixed(128.0))
        .height(Length::Fixed(128.0))
        .clip(has_preview_image)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: (!has_preview_image).then_some(Background::Color(preview_color)),
                border: iced::Border {
                    width: 2.0,
                    color: palette.background.strong.color,
                    radius: 20.0.into(),
                },
                ..Default::default()
            }
        });
    let icon_preview_content: Element<'_, Message> =
        if has_preview_image && app.project_edit_icon_hovered {
            let clear_overlay = container(text("x").size(14).color(Color::WHITE))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .style(|_| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
                    border: iced::Border { radius: 20.0.into(), ..Default::default() },
                    ..Default::default()
                });
            container(iced::widget::stack![preview_badge, clear_overlay])
                .clip(true)
                .width(Length::Fixed(128.0))
                .height(Length::Fixed(128.0))
                .into()
        } else {
            preview_badge.into()
        };
    let icon_preview = mouse_area(icon_preview_content)
        .on_enter(Message::Project(
            crate::app::message::project::ProjectMessage::ProjectEditIconHovered(true),
        ))
        .on_exit(Message::Project(
            crate::app::message::project::ProjectMessage::ProjectEditIconHovered(false),
        ))
        .on_press(if has_preview_image && app.project_edit_icon_hovered {
            Message::Project(crate::app::message::project::ProjectMessage::ProjectEditIconChanged(
                String::new(),
            ))
        } else {
            Message::Project(crate::app::message::project::ProjectMessage::ProjectEditIconPickFile)
        });

    column![
        settings_section_card("项目展示", "配置项目名称、图标来源与图标主题色。"),
        settings_panel(
            column![
                name_input,
                row![
                    icon_preview,
                    column![
                        icon_input,
                        icon_color_presets_row,
                        row![
                            text("图标颜色")
                                .size(13)
                                .style(settings_muted_text_style)
                                .width(Length::Fill),
                            color_preview_btn
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center)
                    ]
                    .spacing(8)
                    .width(Length::Fill)
                ]
                .spacing(16)
                .align_y(iced::Alignment::Start)
            ]
            .spacing(14)
        )
    ]
    .spacing(12)
    .into()
}
#[cfg(test)]
#[path = "general_tab_tests.rs"]
mod general_tab_tests;
