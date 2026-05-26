//! 渲染应用视图中的模态窗口。
//! 本模块只描述模态内容和交互消息，不直接承担持久化策略。

use crate::app::components::system_settings_common::{
    rounded_action_btn_style, settings_panel, settings_section_card, settings_text_input_style,
};
use iced::widget::{button, column, row, text, text_input, toggler};
use iced::{Element, Length};

use super::{App, Message};

/// 模块内可见函数，执行 refresh_tab 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn refresh_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let settings = &app.project_edit_task_board_settings;
    let session_refresh_interval_seconds = app
        .project_edit_session_refresh_interval_seconds_input
        .trim()
        .parse::<u64>()
        .ok()
        .map(|value| value.clamp(1, 3600))
        .unwrap_or(60);
    let task_board_refresh_interval_seconds = settings.refresh_interval_seconds.clamp(1, 3600);

    let auto_promote_toggle = row![
        text("自动拉起任务池").size(13).width(Length::Fill),
        toggler(settings.auto_promote_pool_tasks).on_toggle(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditAutoPromotePoolTasksToggled(
                    value,
                ),
            )
        })
    ]
    .align_y(iced::Alignment::Center);

    let task_board_auto_refresh_toggle = row![
        text("任务看板自动刷新").size(13).width(Length::Fill),
        toggler(app.project_edit_task_board_auto_refresh).on_toggle(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoRefreshToggled(
                    value,
                ),
            )
        })
    ]
    .align_y(iced::Alignment::Center);

    let session_auto_refresh_toggle = row![
        text("项目会话自动刷新").size(13).width(Length::Fill),
        toggler(app.project_edit_session_auto_refresh).on_toggle(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditSessionAutoRefreshToggled(
                    value,
                ),
            )
        })
    ]
    .align_y(iced::Alignment::Center);

    let code_review_toggle = row![
        text("执行代码审查").size(13).width(Length::Fill),
        toggler(settings.code_review_enabled).on_toggle(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditCodeReviewToggled(value),
            )
        })
    ]
    .align_y(iced::Alignment::Center);

    let session_refresh_interval_row = row![
        text("会话刷新(秒)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditSessionRefreshIntervalSecondsChanged(
                    session_refresh_interval_seconds.saturating_sub(5).max(1),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("1-3600", &app.project_edit_session_refresh_interval_seconds_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditSessionRefreshIntervalSecondsInputChanged(
                        value,
                    ),
                )
            })
            .padding([8, 10])
            .size(13)
            .width(Length::Fixed(72.0))
            .style(settings_text_input_style),
        button(text("+").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditSessionRefreshIntervalSecondsChanged(
                    session_refresh_interval_seconds.saturating_add(5).min(3600),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let task_board_refresh_interval_row = row![
        text("任务看板刷新(秒)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardRefreshIntervalSecondsChanged(
                    task_board_refresh_interval_seconds.saturating_sub(5).max(1),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("1-3600", &app.project_edit_task_board_refresh_interval_seconds_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditTaskBoardRefreshIntervalSecondsInputChanged(
                        value,
                    ),
                )
            })
            .padding([8, 10])
            .size(13)
            .width(Length::Fixed(72.0))
            .style(settings_text_input_style),
        button(text("+").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardRefreshIntervalSecondsChanged(
                    task_board_refresh_interval_seconds.saturating_add(5).min(3600),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    column![
        settings_section_card("刷新策略", "统一项目会话、任务看板与代码审查的自动化策略。"),
        settings_panel(
            column![
                auto_promote_toggle,
                task_board_auto_refresh_toggle,
                session_auto_refresh_toggle,
                code_review_toggle,
                session_refresh_interval_row,
                task_board_refresh_interval_row,
            ]
            .spacing(14)
        )
    ]
    .spacing(12)
    .into()
}
#[cfg(test)]
#[path = "refresh_tab_tests.rs"]
mod refresh_tab_tests;
