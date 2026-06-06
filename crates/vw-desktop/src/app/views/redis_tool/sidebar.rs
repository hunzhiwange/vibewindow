//! Redis 工具视图模块，负责连接列表、弹窗、状态徽标和表单控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_muted_text_style,
    settings_text_input_style,
};
use crate::app::message::RedisToolMessage;
use crate::app::{App, Message};
use iced::widget::{Space, button, column, row, text, text_input};
use iced::{Alignment, Element, Length};

use super::common::{
    build_round_icon_action, connection_mode_summary, empty_sidebar_hint, primary_icon_svg,
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
            row![primary_icon_svg(Icon::Plus, 14.0), text("新建连接").size(13)]
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

    let mut plain_connection_names = column![
        text(format!("全部连接: {}", app.redis_tool.connections.len()))
            .size(12)
            .style(settings_muted_text_style)
    ]
    .spacing(4)
    .width(Length::Fill);
    for (index, connection) in app.redis_tool.connections.iter().enumerate() {
        plain_connection_names = plain_connection_names
            .push(text(format!("连接 {}: {}", index + 1, connection.name)).size(13));
    }

    let mut list = column![].spacing(10).width(Length::Fill);

    for connection in &app.redis_tool.connections {
        let selected =
            app.redis_tool.selected_connection_id.as_deref() == Some(connection.id.as_str());
        let runtime_loaded = app
            .redis_tool
            .runtime_overview
            .as_ref()
            .is_some_and(|overview| overview.connection_id == connection.id);
        let status: Element<'a, Message> = if selected {
            text(if runtime_loaded { "已加载" } else { "已选中" })
                .size(11)
                .style(settings_muted_text_style)
                .into()
        } else {
            Space::new().width(Length::Shrink).into()
        };

        let content = row![
            column![
                text(&connection.name).size(14),
                text(format!(
                    "{}:{} / DB {} / {}",
                    connection.host,
                    connection.port,
                    connection.db,
                    connection_mode_summary(connection)
                ))
                .size(11)
                .style(settings_muted_text_style),
            ]
            .spacing(4)
            .width(Length::Fill),
            status,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let item = button(content)
            .padding([10, 12])
            .width(Length::Fill)
            .style(if selected { primary_action_btn_style } else { rounded_action_btn_style })
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
    }

    column![
        action_bar,
        search,
        plain_connection_names,
        text("连接模块列表").size(12).style(settings_muted_text_style),
        list,
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
#[path = "sidebar_tests.rs"]
mod sidebar_tests;
