//! 顶部菜单栏组件。

mod controls;
mod drag;
mod external;
mod gateway;
mod menus;
mod widgets;

use crate::app::{App, Message};
use iced::widget::{container, row};
use iced::{Element, Length, Theme};
use widgets::color_with_alpha;

/// 顶部菜单栏的高度（像素）
///
/// 此常量定义了顶部菜单栏的固定高度，用于布局计算和样式设置。
pub const TOP_BAR_HEIGHT: f32 = 28.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let file_menu = menus::file_menu(app);
    let edit_menu = menus::edit_menu(app);
    let view_menu = menus::view_menu(app);
    let help_menu = menus::help_menu(app);
    let settings = controls::settings_button();
    let open_external_module = external::open_external_module(app);
    let gateway_services = gateway::gateway_services_module(app);
    let project_view_tools = controls::project_view_tools(app);
    let traffic_light_spacer = drag::traffic_light_spacer();
    let drag_spacer = drag::drag_spacer();

    container(
        row![
            traffic_light_spacer,
            file_menu,
            edit_menu,
            view_menu,
            help_menu,
            drag_spacer,
            open_external_module,
            gateway_services,
            project_view_tools,
            settings
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center),
    )
    .padding([0, 6])
    .height(Length::Fixed(TOP_BAR_HEIGHT))
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            // 半透明背景，使用弱背景色
            background: Some(iced::Background::Color(color_with_alpha(
                palette.background.weak.color,
                0.88,
            ))),
            // 底部边框
            border: iced::Border {
                width: 1.0,
                color: palette.background.strong.color.scale_alpha(0.60),
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

#[cfg(test)]
mod controls_tests;
#[cfg(test)]
mod drag_tests;
#[cfg(test)]
mod external_tests;
#[cfg(test)]
mod menus_tests;
#[cfg(test)]
#[path = "widgets_tests.rs"]
mod widgets_tests;
