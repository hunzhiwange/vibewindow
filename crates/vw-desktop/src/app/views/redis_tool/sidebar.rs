//! Redis 工具视图模块，负责连接列表、弹窗、状态徽标和表单控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    primary_action_btn_style, settings_muted_text_style, settings_text_input_style,
    settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::{App, Message};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Theme};

use super::common::{
    build_round_icon_action, connection_badge_labels, connection_item_style,
    connection_mode_summary, empty_sidebar_hint, format_timestamp, redis_scroll_direction,
};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_sidebar<'a>(app: &'a App) -> Element<'a, Message> {
    let is_busy = app.redis_tool.is_gateway_loading();
    let search = text_input("搜索连接名称或主机", &app.redis_tool.connection_search_query)
        .on_input(|value| Message::RedisTool(RedisToolMessage::SearchConnectionsChanged(value)))
        .padding([10, 12])
        .size(13)
        .style(settings_text_input_style);

    let action_bar = row![
        button(
            row![
                crate::app::components::system_settings_common::icon_svg(Icon::Plus, 14.0),
                text("新建连接").size(13)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press_maybe((!is_busy).then_some(Message::RedisTool(RedisToolMessage::NewConnection)))
        .padding([10, 14])
        .width(Length::Fill)
        .style(primary_action_btn_style),
        build_round_icon_action(
            Icon::GearWideConnected,
            Message::RedisTool(RedisToolMessage::OpenSettingsModal),
            !is_busy,
        ),
        build_round_icon_action(
            Icon::Clock,
            Message::RedisTool(RedisToolMessage::OpenHistoryModal),
            !is_busy,
        ),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let query = app.redis_tool.connection_search_query.trim().to_ascii_lowercase();
    let mut list = column![].spacing(10).width(Length::Fill);
    let mut matched_connections = 0usize;

    for connection in app.redis_tool.connections.iter().filter(|connection| {
        query.is_empty()
            || connection.name.to_ascii_lowercase().contains(&query)
            || connection.host.to_ascii_lowercase().contains(&query)
    }) {
        matched_connections += 1;
        let selected =
            app.redis_tool.selected_connection_id.as_deref() == Some(connection.id.as_str());
        let runtime_loaded = app
            .redis_tool
            .runtime_overview
            .as_ref()
            .is_some_and(|overview| overview.connection_id == connection.id);
        let last_used =
            connection.last_used_ms.map(format_timestamp).unwrap_or_else(|| "未进入".to_string());

        let badges = build_badges(connection);
        let mut content = column![
            row![text(&connection.name).size(14), Space::new().width(Length::Fill), badges,]
                .align_y(Alignment::Center),
            text(format!("{}:{}  /  DB {}", connection.host, connection.port, connection.db))
                .size(12)
                .style(settings_muted_text_style),
            text(connection_mode_summary(connection)).size(11).style(settings_muted_text_style),
            text(format!("最近使用：{last_used}")).size(11).style(settings_muted_text_style),
        ]
        .spacing(6);

        if selected {
            content = content.push(
                row![
                    settings_value_badge("已展开"),
                    settings_value_badge(if runtime_loaded {
                        "信息已加载"
                    } else {
                        "待加载"
                    }),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            );
            content = content.push(
                text("右侧已显示该连接的 Redis 信息与命令控制台，其它连接会自动折叠。")
                    .size(11)
                    .style(settings_muted_text_style),
            );
        }

        let item = button(
            container(content)
                .padding([14, 14])
                .width(Length::Fill)
                .style(move |theme: &Theme| connection_item_style(theme, selected)),
        )
        .padding(0)
        .width(Length::Fill)
        .style(button::text)
        .on_press_maybe((!is_busy).then_some(Message::RedisTool(
            RedisToolMessage::SelectConnection(connection.id.clone()),
        )));

        list = list.push(item);
    }

    if app.redis_tool.connections.is_empty() {
        list = list.push(empty_sidebar_hint(
            "还没有保存的连接",
            "点击“新建连接”创建第一个 Redis 连接配置。",
        ));
    } else if !query.is_empty() && matched_connections == 0 {
        list = list.push(empty_sidebar_hint("没有匹配结果", "调整搜索关键字后重试。"));
    }

    column![
        action_bar,
        search,
        container(scrollable(list).direction(redis_scroll_direction()).height(Length::Fill),)
            .height(Length::Fill),
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_badges<'a>(
    connection: &'a crate::app::state::RedisConnectionConfig,
) -> Element<'a, Message> {
    let labels = connection_badge_labels(connection);
    if labels.is_empty() {
        return Space::new().width(Length::Shrink).into();
    }

    let mut row_widget = row![].spacing(6).align_y(Alignment::Center);
    for label in labels {
        row_widget = row_widget.push(settings_value_badge(label));
    }
    row_widget.into()
}

#[cfg(test)]
#[path = "sidebar_tests.rs"]
mod sidebar_tests;
