//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    settings_muted_text_style, settings_panel, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::RedisRuntimeOverview;
use crate::app::{App, Message};
use iced::widget::{Space, column, container, row, text, text_input};
use iced::{Alignment, Background, Border, Element, Length, Theme};

use super::super::common::{
    build_detail_action_button, masked_connection_preview, overview_row,
};

/// 构建对应界面片段。
///
/// # 参数
/// - `runtime`: 当前视图构建所需的状态、配置或消息。
/// - `compact`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_runtime_overview_cards<'a>(
    runtime: &'a RedisRuntimeOverview,
    compact: bool,
) -> Element<'a, Message> {
    let cards: Element<'a, Message> = if compact {
        column![
            build_runtime_card(
                "服务器",
                vec![
                    ("Redis版本", fallback_runtime_value(&runtime.server_version)),
                    ("OS", fallback_runtime_value(&runtime.os)),
                    ("进程ID", fallback_runtime_value(&runtime.process_id)),
                ],
            ),
            build_runtime_card(
                "内存",
                vec![
                    ("已用内存", fallback_runtime_value(&runtime.used_memory_human)),
                    ("内存占用峰值", fallback_runtime_value(&runtime.used_memory_peak_human)),
                    ("Lua占用内存", fallback_runtime_value(&runtime.used_memory_lua_human)),
                ],
            ),
            build_runtime_card(
                "状态",
                vec![
                    ("客户端连接数", runtime.connected_clients.to_string()),
                    ("历史连接数", runtime.total_connections_received.to_string()),
                    ("历史命令数", runtime.total_commands_processed.to_string()),
                ],
            ),
        ]
        .spacing(12)
        .into()
    } else {
        row![
            container(build_runtime_card(
                "服务器",
                vec![
                    ("Redis版本", fallback_runtime_value(&runtime.server_version)),
                    ("OS", fallback_runtime_value(&runtime.os)),
                    ("进程ID", fallback_runtime_value(&runtime.process_id)),
                ],
            ))
            .width(Length::Fill),
            container(build_runtime_card(
                "内存",
                vec![
                    ("已用内存", fallback_runtime_value(&runtime.used_memory_human)),
                    ("内存占用峰值", fallback_runtime_value(&runtime.used_memory_peak_human)),
                    ("Lua占用内存", fallback_runtime_value(&runtime.used_memory_lua_human)),
                ],
            ))
            .width(Length::Fill),
            container(build_runtime_card(
                "状态",
                vec![
                    ("客户端连接数", runtime.connected_clients.to_string()),
                    ("历史连接数", runtime.total_connections_received.to_string()),
                    ("历史命令数", runtime.total_commands_processed.to_string()),
                ],
            ))
            .width(Length::Fill),
        ]
        .spacing(12)
        .into()
    };

    cards
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
pub(super) fn build_runtime_empty_state<'a>(
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
                    "刷新信息",
                    Message::RedisTool(RedisToolMessage::RefreshSelectedRuntime),
                    true,
                    !is_busy && app.redis_tool.selected_connection_id.is_some(),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            if app.redis_tool.draft.ssh_tunnel.enabled {
                overview_row("当前限制", "SSH 隧道运行态仍未接入网关执行链路")
            } else {
                overview_row("连接预览", masked_connection_preview(app))
            },
        ]
        .spacing(12),
    )
    .into()
}

fn build_runtime_card<'a>(title: &'a str, rows: Vec<(&'a str, String)>) -> Element<'a, Message> {
    let mut content = column![text(title).size(14), crate::app::components::system_settings_common::settings_divider()]
        .spacing(10);
    for (label, value) in rows {
        content = content.push(overview_row(label, value));
    }
    settings_panel(content).into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `runtime`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_keyspace_panel<'a>(runtime: &'a RedisRuntimeOverview) -> Element<'a, Message> {
    let mut content = column![
        row![
            text("键值统计").size(14),
            Space::new().width(Length::Fill),
            settings_value_badge(format!("{} 个 DB", runtime.keyspace.len())),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if runtime.keyspace.is_empty() {
        content = content.push(
            text("当前连接没有返回 Keyspace 统计。")
                .size(12)
                .style(settings_muted_text_style),
        );
    } else {
        content = content.push(build_keyspace_row(
            [
                "DB".to_string(),
                "Keys".to_string(),
                "Expires".to_string(),
                "Avg TTL".to_string(),
            ],
            true,
        ));
        for stat in &runtime.keyspace {
            content = content.push(build_keyspace_row(
                [
                    stat.db.clone(),
                    stat.keys.to_string(),
                    stat.expires.to_string(),
                    stat.avg_ttl.to_string(),
                ],
                false,
            ));
        }
    }

    settings_panel(content).into()
}

fn build_keyspace_row<'a>(cells: [String; 4], header: bool) -> Element<'a, Message> {
    let widths = [140.0, 140.0, 140.0, 1.0];
    let mut row_widget = row![].spacing(10).align_y(Alignment::Center);

    for (index, cell) in cells.into_iter().enumerate() {
        let width = if index == 3 {
            Length::Fill
        } else {
            Length::Fixed(widths[index])
        };
        row_widget = row_widget.push(
            container(
                text(cell).size(if header { 12 } else { 11 }).style(move |theme: &Theme| {
                    iced::widget::text::Style {
                        color: Some(if header {
                            theme.palette().text.scale_alpha(0.76)
                        } else {
                            theme.palette().text.scale_alpha(0.92)
                        }),
                    }
                }),
            )
            .width(width),
        );
    }

    container(row_widget)
        .padding([8, 12])
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(if header {
                    palette.background.weak.color.scale_alpha(0.72)
                } else {
                    palette.background.base.color.scale_alpha(0.36)
                })),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color.scale_alpha(0.18),
                    radius: 14.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `runtime`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_info_panel<'a>(
    app: &'a App,
    runtime: &'a RedisRuntimeOverview,
) -> Element<'a, Message> {
    let query = app.redis_tool.info_filter.trim().to_ascii_lowercase();
    let mut content = column![
        row![
            column![
                text("Redis信息全集").size(14),
                text("完整展示 INFO 返回的键值，支持本地过滤。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            container(
                text_input("搜索 INFO 字段", &app.redis_tool.info_filter)
                    .on_input(|value| Message::RedisTool(RedisToolMessage::InfoFilterChanged(value)))
                    .padding([8, 10])
                    .size(12),
            )
            .width(Length::Fixed(220.0)),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    content = content.push(build_keyspace_row(
        ["Key".to_string(), "Value".to_string(), String::new(), String::new()],
        true,
    ));

    let mut matched = 0usize;
    for entry in runtime.info_entries.iter().filter(|entry| {
        query.is_empty()
            || entry.key.to_ascii_lowercase().contains(&query)
            || entry.value.to_ascii_lowercase().contains(&query)
    }) {
        matched += 1;
        content = content.push(build_info_row(&entry.key, &entry.value));
    }

    if matched == 0 {
        content = content.push(
            text("没有匹配的 INFO 字段。")
                .size(12)
                .style(settings_muted_text_style),
        );
    }

    settings_panel(content).into()
}

fn build_info_row<'a>(key: &'a str, value: &'a str) -> Element<'a, Message> {
    container(
        row![
            container(text(key).size(11)).width(Length::Fixed(240.0)),
            container(text(value).size(11)).width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(palette.background.base.color.scale_alpha(0.36))),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color.scale_alpha(0.18),
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn fallback_runtime_value(value: &str) -> String {
    if value.trim().is_empty() {
        "--".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
