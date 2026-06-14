//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    settings_muted_text_style, settings_panel, settings_text_input_style, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::RedisCommandOutputEntry;
use crate::app::{App, Message};
use iced::widget::{Space, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::common::{build_detail_action_button, format_timestamp, redis_scroll_direction};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `is_busy`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_command_panel<'a>(app: &'a App, is_busy: bool) -> Element<'a, Message> {
    let mut output_column = column![].spacing(10).width(Length::Fill);
    if app.redis_tool.command_output.is_empty() {
        output_column = output_column.push(
            text("输入 Redis 命令后，会在这里显示返回结果。")
                .size(12)
                .style(settings_muted_text_style),
        );
    } else {
        for entry in &app.redis_tool.command_output {
            output_column = output_column.push(build_command_output_entry(entry));
        }
    }

    settings_panel(
        column![
            row![
                column![
                    text("命令执行").size(14),
                    text("命令通过网关下发，执行结果只保留在当前桌面运行态中。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                settings_value_badge("Gateway"),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            container(
                scrollable(output_column)
                    .direction(redis_scroll_direction())
                    .height(Length::Fixed(240.0)),
            )
            .padding([10, 0])
            .width(Length::Fill),
            row![
                text_input("例如：INFO server 或 GET my_key", &app.redis_tool.command_input)
                    .on_input(|value| Message::RedisTool(RedisToolMessage::CommandInputChanged(
                        value
                    )))
                    .on_submit(Message::RedisTool(RedisToolMessage::RunCommand))
                    .padding([10, 12])
                    .size(13)
                    .width(Length::Fill)
                    .style(settings_text_input_style),
                build_detail_action_button(
                    "执行命令",
                    Message::RedisTool(RedisToolMessage::RunCommand),
                    true,
                    !is_busy && app.redis_tool.selected_connection_id.is_some(),
                ),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
    .into()
}

fn build_command_output_entry<'a>(entry: &'a RedisCommandOutputEntry) -> Element<'a, Message> {
    container(
        column![
            row![
                text(&entry.command).size(12),
                Space::new().width(Length::Fill),
                settings_value_badge(if entry.is_error { "ERROR" } else { "OK" }),
                text(format!("{} ms", entry.cost_ms)).size(11).style(settings_muted_text_style),
                text(format_timestamp(entry.time_ms)).size(11).style(settings_muted_text_style),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            if entry.is_error {
                text(&entry.output).size(12).style(command_error_text_style)
            } else {
                text(&entry.output).size(12).style(command_success_text_style)
            },
        ]
        .spacing(8),
    )
    .padding([12, 14])
    .width(Length::Fill)
    .style(command_output_entry_style)
    .into()
}

fn command_success_text_style(_theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(Color::from_rgba8(241, 245, 249, 0.94)) }
}

fn command_error_text_style(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(theme.extended_palette().danger.base.color) }
}

fn command_output_entry_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba8(18, 24, 33, 0.96))),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.18),
            radius: 16.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;
