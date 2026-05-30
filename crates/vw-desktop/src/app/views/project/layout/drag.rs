//! 项目工作区布局模块，负责侧栏、主区域、右侧面板和拖拽提示的组合。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::border::Border;
use iced::widget::{Space, container, row, text};
use iced::{Color, Element, Length, Theme};

use crate::app::{App, Message};

/// 构建拖拽交互界面。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub fn drag_badge_layer(app: &App) -> Element<'_, Message> {
    if let Some(path) = app.dragging_file_paths.first() {
        let display_path = if let Some(project_root) = &app.project_path {
            std::path::Path::new(path)
                .strip_prefix(project_root)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or(path)
                .replace('\\', "/")
        } else {
            path.replace('\\', "/")
        };

        let mut x = app.cursor_position.x;
        let mut y = app.cursor_position.y
            - crate::app::components::top_bar::TOP_BAR_HEIGHT
            - crate::app::components::tab_bar::TAB_BAR_HEIGHT;
        let max_x = (app.window_size.0 - 260.0).max(0.0);
        let max_y = (app.window_size.1 - 42.0).max(0.0);
        x = x.clamp(0.0, max_x);
        y = y.clamp(0.0, max_y);

        let extra_count = app.dragging_file_paths.len().saturating_sub(1);
        let badge = container(
            row![
                text("@").size(12).style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.palette().primary),
                }),
                text(display_path).size(12).style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.palette().text),
                }),
                text(if extra_count > 0 { format!("+{}", extra_count) } else { String::new() })
                    .size(12)
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().secondary.base.color),
                    })
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 10])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.base.color.into()),
                border: Border { color: p.background.strong.color, width: 1.0, radius: 8.0.into() },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.12),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            }
        });

        container(badge)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .padding(iced::Padding { top: y, right: 0.0, bottom: 0.0, left: x })
            .into()
    } else {
        container(Space::new()).width(Length::Fill).height(Length::Fill).into()
    }
}
#[cfg(test)]
#[path = "drag_tests.rs"]
mod drag_tests;
