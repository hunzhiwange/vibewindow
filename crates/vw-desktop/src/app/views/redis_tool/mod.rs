//! Redis 客户端工具视图。
//!
//! 本模块负责组织 Redis 页面入口与布局编排：
//! - 顶部概览条
//! - 左侧连接列表
//! - 右侧连接详情与高级页签
//! - 设置 / 历史弹窗

use crate::app::message::RedisToolMessage;
use crate::app::{App, Message};
use iced::widget::{Space, column, container, mouse_area, responsive, row};
use iced::{Alignment, Background, Color, Element, Length, Size, Theme};

mod common;
mod detail;
mod modal;
mod sidebar;

use common::{build_error_banner, build_status_badge, current_load_count};
use detail::build_detail_panel;
use modal::{
    build_connection_modal, build_create_key_modal, build_history_modal, build_settings_modal,
};
use sidebar::build_sidebar;

/// 渲染 Redis 工具视图。
pub fn view(app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            column![
                iced::widget::text("Redis 客户端").size(20),
                iced::widget::text("连接管理、导入导出与高级连接模板工作台").size(12).style(
                    crate::app::components::system_settings_common::settings_muted_text_style
                ),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            crate::app::components::system_settings_common::settings_value_badge(format!(
                "{} 个连接",
                app.redis_tool.connections.len()
            )),
            crate::app::components::system_settings_common::settings_value_badge(format!(
                "默认加载 {} 项",
                current_load_count(app)
            )),
            build_status_badge(app),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([18, 20])
    .width(Length::Fill)
    .style(crate::app::components::system_settings_common::settings_panel_style);

    let workspace = responsive(move |size| build_workspace(app, size));
    let mut content = column![hero].spacing(16).width(Length::Fill).height(Length::Fill);

    if let Some(error) = &app.redis_tool.gateway_error {
        content = content.push(build_error_banner(error));
    }

    content = content.push(container(workspace).width(Length::Fill).height(Length::Fill));

    let base = container(
        container(content.padding([18, 24]))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(palette.background.base.color.into()),
            ..Default::default()
        }
    });

    if app.redis_tool.show_settings_modal {
        return stack_modal(
            base.into(),
            build_settings_modal(app),
            Message::RedisTool(RedisToolMessage::CloseSettingsModal),
        );
    }

    if app.redis_tool.show_connection_modal {
        return stack_modal(
            base.into(),
            build_connection_modal(app),
            Message::RedisTool(RedisToolMessage::CloseConnectionModal),
        );
    }

    if app.redis_tool.show_create_key_modal {
        return stack_modal(
            base.into(),
            build_create_key_modal(app),
            Message::RedisTool(RedisToolMessage::CloseCreateKeyModal),
        );
    }

    if app.redis_tool.show_history_modal {
        return stack_modal(
            base.into(),
            build_history_modal(app),
            Message::RedisTool(RedisToolMessage::CloseHistoryModal),
        );
    }

    base.into()
}

fn build_workspace<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let sidebar = build_sidebar(app);
    let detail = build_detail_panel(app, size.width < 1180.0);

    if size.width >= 980.0 {
        row![
            container(sidebar).width(Length::Fixed(320.0)).height(Length::Fill),
            container(detail).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        let sidebar_height = compact_sidebar_height(size.height);
        column![
            container(sidebar).width(Length::Fill).height(Length::Fixed(sidebar_height)),
            container(detail).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn compact_sidebar_height(available_height: f32) -> f32 {
    (available_height * 0.35).clamp(140.0, 300.0)
}

fn stack_modal<'a>(
    base: Element<'a, Message>,
    modal: Element<'a, Message>,
    close_message: Message,
) -> Element<'a, Message> {
    let overlay =
        mouse_area(container(Space::new().width(Length::Fill).height(Length::Fill)).style(|_| {
            iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                ..Default::default()
            }
        }))
        .on_press(close_message);

    iced::widget::stack![
        base,
        overlay,
        container(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ]
    .into()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
