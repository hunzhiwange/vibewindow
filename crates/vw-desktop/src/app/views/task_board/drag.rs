//! 任务看板视图模块，负责看板列、拖拽预览和整体页面组织。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use iced::widget::{Space, column, container, row, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::task::Task;
use crate::app::{App, Message};

fn task_status_tag_colors(status: crate::app::task::TaskStatus) -> (Color, Color) {
    match status {
        crate::app::task::TaskStatus::Pool => {
            (Color::from_rgb8(107, 114, 128), Color::from_rgb8(243, 244, 246))
        }
        crate::app::task::TaskStatus::Pending => {
            (Color::from_rgb8(37, 99, 235), Color::from_rgb8(219, 234, 254))
        }
        crate::app::task::TaskStatus::Running => {
            (Color::from_rgb8(147, 51, 234), Color::from_rgb8(243, 232, 255))
        }
        crate::app::task::TaskStatus::Failed => {
            (Color::from_rgb8(220, 38, 38), Color::from_rgb8(254, 226, 226))
        }
        crate::app::task::TaskStatus::Paused => {
            (Color::from_rgb8(202, 138, 4), Color::from_rgb8(254, 249, 195))
        }
        crate::app::task::TaskStatus::CodeComplete => {
            (Color::from_rgb8(5, 150, 105), Color::from_rgb8(209, 250, 229))
        }
        crate::app::task::TaskStatus::CodeReview => {
            (Color::from_rgb8(217, 119, 6), Color::from_rgb8(254, 243, 199))
        }
        crate::app::task::TaskStatus::PrSubmitted => {
            (Color::from_rgb8(8, 145, 178), Color::from_rgb8(207, 250, 254))
        }
        crate::app::task::TaskStatus::Completed => {
            (Color::from_rgb8(22, 163, 74), Color::from_rgb8(220, 252, 231))
        }
        crate::app::task::TaskStatus::Archived => {
            (Color::from_rgb8(100, 116, 139), Color::from_rgb8(241, 245, 249))
        }
    }
}

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
pub fn build_drag_preview_layer<'a>(app: &'a App) -> Element<'a, Message> {
    if let Some((task_id, _)) = &app.task_board_dragging
        && let Some(task) = app.task_board_tasks.iter().find(|t| &t.id == task_id) {
            let mut x = app.cursor_position.x;
            let mut y = app.cursor_position.y;
            x = x.clamp(0.0, (app.window_size.0 - 220.0).max(0.0));
            y = y.clamp(0.0, (app.window_size.1 - 80.0).max(0.0));

            let preview_card = build_drag_preview_card(task);

            return container(preview_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding { top: y - 40.0, right: 0.0, bottom: 0.0, left: x - 100.0 })
                .into();
        }
    container(Space::new()).width(Length::Fill).height(Length::Fill).into()
}

fn build_drag_preview_card<'a>(task: &'a Task) -> Element<'a, Message> {
    let priority_color = if task.priority <= 100 {
        Color::from_rgb8(239, 68, 68)
    } else if task.priority <= 500 {
        Color::from_rgb8(245, 158, 11)
    } else {
        Color::from_rgb8(107, 114, 128)
    };

    let priority_badge = container(text(format!("P{}", task.priority)).size(9))
        .padding([1, 4])
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(priority_color.scale_alpha(0.2))),
            border: Border { radius: 4.0.into(), ..Default::default() },
            text_color: Some(priority_color),
            ..Default::default()
        });

    let header_row = row![Space::new().width(Length::Fill), priority_badge,]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    let content =
        if task.prompt.trim().is_empty() { "（无内容）" } else { task.prompt.as_str() };
    let (status_color, status_bg_color) = task_status_tag_colors(task.status);
    let status_badge = container(text(task.status.label()).size(9)).padding([1, 6]).style(
        move |_theme: &Theme| container::Style {
            background: Some(Background::Color(status_bg_color)),
            border: Border { radius: 999.0.into(), ..Default::default() },
            text_color: Some(status_color),
            ..Default::default()
        },
    );
    let title_row = row![
        text(&task.id).size(9).style(|theme: &Theme| {
            iced::widget::text::Style { color: Some(theme.extended_palette().background.base.text) }
        }),
        container(text(content).size(12).wrapping(iced::widget::text::Wrapping::Word))
            .width(Length::Fill),
        status_badge,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);

    let content_col =
        column![header_row, Space::new().height(4.0), title_row,].spacing(2).width(Length::Fill);

    container(content_col)
        .width(Length::Fixed(200.0))
        .padding(10)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(theme.palette().background)),
            border: Border {
                width: 2.0,
                color: Color::from_rgb8(59, 130, 246),
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgb8(59, 130, 246).scale_alpha(0.3),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
#[path = "drag_tests.rs"]
mod drag_tests;
