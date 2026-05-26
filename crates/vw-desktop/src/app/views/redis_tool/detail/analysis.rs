//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    settings_divider, settings_muted_text_style, settings_panel, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::RedisKeyAnalysis;
use crate::app::{App, Message};
use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::super::common::{build_detail_action_button, overview_row, redis_scroll_direction};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `analysis`: 当前视图构建所需的状态、配置或消息。
/// - `is_busy`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_key_analysis_panel<'a>(
    app: &'a App,
    analysis: &'a RedisKeyAnalysis,
    is_busy: bool,
) -> Element<'a, Message> {
    settings_panel(
        column![
            row![
                column![
                    text("内容分析").size(14),
                    text("聚焦单个 Key 的类型、TTL、内存占用与值预览。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                settings_value_badge(analysis.key_type.clone()),
                settings_value_badge(ttl_label(analysis.ttl_secs)),
                settings_value_badge(memory_usage_label(analysis.memory_usage_bytes)),
                build_detail_action_button(
                    "刷新内容",
                    Message::RedisTool(RedisToolMessage::RefreshSelectedKeyAnalysis),
                    false,
                    !is_busy && app.redis_tool.selected_key.is_some(),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            settings_divider(),
            overview_row("Key", analysis.key.clone()),
            overview_row("数据类型", analysis.key_type.clone()),
            overview_row("TTL", ttl_label(analysis.ttl_secs)),
            overview_row("内存占用", memory_usage_label(analysis.memory_usage_bytes)),
            overview_row("预览命令", analysis.preview_command.clone()),
            settings_divider(),
            column![
                text("值预览").size(13),
                text("当前仅提供只读预览，后续可在此基础上扩展编辑与保存。")
                    .size(12)
                    .style(settings_muted_text_style),
                container(
                    scrollable(
                        container(
                            text(non_empty_preview(&analysis.preview_output))
                                .size(12)
                                .width(Length::Fill),
                        )
                        .padding([14, 16])
                        .width(Length::Fill)
                    )
                    .direction(redis_scroll_direction())
                    .height(Length::Fixed(320.0))
                )
                .padding([2, 0])
                .width(Length::Fill)
                .style(preview_panel_style),
            ]
            .spacing(10),
        ]
        .spacing(12),
    )
    .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `title`: 当前视图构建所需的状态、配置或消息。
/// - `description`: 当前视图构建所需的状态、配置或消息。
/// - `is_busy`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_key_analysis_empty_state<'a>(
    app: &'a App,
    title: &'a str,
    description: &'a str,
    is_busy: bool,
) -> Element<'a, Message> {
    settings_panel(
        column![
            row![
                column![
                    text(title).size(14),
                    text(description).size(12).style(settings_muted_text_style),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                build_detail_action_button(
                    "刷新内容",
                    Message::RedisTool(RedisToolMessage::RefreshSelectedKeyAnalysis),
                    true,
                    !is_busy && app.redis_tool.selected_key.is_some(),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            if let Some(key) = &app.redis_tool.selected_key {
                overview_row("当前 Key", key.clone())
            } else {
                overview_row("当前 Key", "未选择")
            },
        ]
        .spacing(12),
    )
    .into()
}

fn ttl_label(ttl_secs: i64) -> String {
    match ttl_secs {
        -2 => "不存在".to_string(),
        -1 => "永久".to_string(),
        value if value >= 0 => format!("TTL {value}s"),
        value => format!("TTL {value}"),
    }
}

fn memory_usage_label(memory_usage_bytes: Option<u64>) -> String {
    let Some(bytes) = memory_usage_bytes else {
        return "内存未知".to_string();
    };

    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
}

fn non_empty_preview(value: &str) -> String {
    if value.trim().is_empty() {
        "(empty)".to_string()
    } else {
        value.to_string()
    }
}

fn preview_panel_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(Background::Color(palette.background.base.color.scale_alpha(0.34))),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.18),
            radius: 16.0.into(),
        },
        text_color: Some(theme.palette().text),
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(0.04),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "analysis_tests.rs"]
mod analysis_tests;
