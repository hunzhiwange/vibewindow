use iced::widget::{column, container, text};
use iced::{Background, Border, Element, Length, Theme};

use crate::app::Message;
use crate::app::task::{TaskStatus, WorktreeState};

use super::super::helpers::{task_status_tag_colors, worktree_state_palette};

/// 构建 Worktree 状态统计条带，显示特定状态的槽位数量。
pub(super) fn build_worktree_summary_strip<'a>(
    label: &'static str,
    value: usize,
    state: WorktreeState,
) -> Element<'a, Message> {
    container(
        column![
            text(label).size(10).style(move |theme: &Theme| {
                let (_, text_color, _) = worktree_state_palette(theme, state);
                iced::widget::text::Style { color: Some(text_color.scale_alpha(0.72)) }
            }),
            text(value.to_string()).size(14).style(move |theme: &Theme| {
                let (_, text_color, _) = worktree_state_palette(theme, state);
                iced::widget::text::Style { color: Some(text_color) }
            }),
        ]
        .spacing(2),
    )
    .padding([6, 10])
    .width(Length::FillPortion(1))
    .style(move |theme: &Theme| {
        let (bg, _, border_color) = worktree_state_palette(theme, state);
        iced::widget::container::Style {
            background: Some(Background::Color(bg.scale_alpha(0.9))),
            border: Border { width: 1.0, color: border_color.scale_alpha(0.8), radius: 2.0.into() },
            ..Default::default()
        }
    })
    .into()
}

/// 构建任务状态统计条带，显示特定状态的任务数量。
pub(super) fn build_task_status_summary_strip<'a>(
    label: &'static str,
    value: usize,
    status: TaskStatus,
) -> Element<'a, Message> {
    let (text_color, bg_color) = task_status_tag_colors(status);
    container(
        column![
            text(label).size(8).style(move |_theme: &Theme| iced::widget::text::Style {
                color: Some(text_color.scale_alpha(0.84)),
            }),
            text(value.to_string())
                .size(11)
                .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(text_color) }),
        ]
        .spacing(1),
    )
    .padding([3, 5])
    .width(Length::FillPortion(1))
    .style(move |_theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(bg_color.scale_alpha(0.92))),
        border: Border { width: 1.0, color: text_color.scale_alpha(0.42), radius: 2.0.into() },
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod summary_tests;
