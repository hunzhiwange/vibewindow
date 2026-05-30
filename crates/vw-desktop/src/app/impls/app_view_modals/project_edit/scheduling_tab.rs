//! 渲染应用视图中的模态窗口。
//! 本模块只描述模态内容和交互消息，不直接承担持久化策略。

use crate::app::components::system_settings_common::{
    rounded_action_btn_style, settings_checkbox_style, settings_panel, settings_section_card,
    settings_text_input_style,
};
use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use super::{App, Message};

/// 模块内可见函数，执行 scheduling_tab 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn scheduling_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let settings = &app.project_edit_task_board_settings;
    let max_concurrent = settings.max_concurrent.clamp(1, 10);
    let scheduler_tick_interval_seconds = settings.scheduler_tick_interval_seconds.clamp(1, 60);
    let auto_promote_tick_interval_seconds =
        settings.auto_promote_tick_interval_seconds.clamp(1, 3600);
    let failed_retry_minutes = settings.failed_retry_minutes.clamp(1, 1440);
    let running_timeout_minutes = settings.running_timeout_minutes.clamp(1, 1440);
    let pr_submitted_stall_timeout_seconds =
        settings.pr_submitted_stall_timeout_seconds.clamp(5, 3600);

    let max_concurrent_row = row![
        text("并发任务数").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditMaxConcurrentChanged(
                    if max_concurrent <= 1 { 1 } else { max_concurrent - 1 },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("1-10", &app.project_edit_max_concurrent_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditMaxConcurrentInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditMaxConcurrentChanged(
                    if max_concurrent >= 10 { 10 } else { max_concurrent + 1 },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let scheduler_tick_interval_row = row![
        text("任务调度间隔(秒)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardSchedulerTickIntervalSecondsChanged(
                    scheduler_tick_interval_seconds.saturating_sub(1).max(1),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input(
            "1-60",
            &app.project_edit_task_board_scheduler_tick_interval_seconds_input,
        )
        .on_input(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardSchedulerTickIntervalSecondsInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardSchedulerTickIntervalSecondsChanged(
                    scheduler_tick_interval_seconds.saturating_add(1).min(60),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let auto_promote_tick_interval_row = row![
        text("自动执行任务池间隔(秒)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoPromoteTickIntervalSecondsChanged(
                    auto_promote_tick_interval_seconds.saturating_sub(5).max(1),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input(
            "1-3600",
            &app.project_edit_task_board_auto_promote_tick_interval_seconds_input,
        )
        .on_input(|value| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoPromoteTickIntervalSecondsInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoPromoteTickIntervalSecondsChanged(
                    auto_promote_tick_interval_seconds.saturating_add(5).min(3600),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let failed_retry_row = row![
        text("失败重试(分钟)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditFailedRetryMinutesChanged(
                    if failed_retry_minutes <= 1 { 1 } else { failed_retry_minutes - 1 },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("1-1440", &app.project_edit_failed_retry_minutes_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditFailedRetryMinutesInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditFailedRetryMinutesChanged(
                    if failed_retry_minutes >= 1440 { 1440 } else { failed_retry_minutes + 1 },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let running_timeout_row = row![
        text("运行超时(分钟)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditRunningTimeoutMinutesChanged(
                    if running_timeout_minutes <= 1 { 1 } else { running_timeout_minutes - 1 },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("1-1440", &app.project_edit_running_timeout_minutes_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditRunningTimeoutMinutesInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditRunningTimeoutMinutesChanged(
                    if running_timeout_minutes >= 1440 {
                        1440
                    } else {
                        running_timeout_minutes + 1
                    },
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let recycle_worktree_on_finish_toggle = checkbox(settings.recycle_worktree_on_task_finish)
        .label("完成后回收 worktree")
        .spacing(8)
        .on_toggle(|enabled| {
            Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditRecycleWorktreeOnTaskFinishToggled(
                    enabled,
                ),
            )
        })
        .style(settings_checkbox_style);

    let pr_stall_timeout_row = row![
        text("合并锁超时(秒)").size(13).width(Length::Fill),
        button(text("-").size(12))
            .on_press(Message::Project(
                crate::app::message::project::ProjectMessage::ProjectEditPrSubmittedStallTimeoutSecondsChanged(
                    pr_submitted_stall_timeout_seconds.saturating_sub(5).max(5),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
        text_input("5-3600", &app.project_edit_pr_submitted_stall_timeout_seconds_input)
            .on_input(|value| {
                Message::Project(
                    crate::app::message::project::ProjectMessage::ProjectEditPrSubmittedStallTimeoutSecondsInputChanged(
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
                crate::app::message::project::ProjectMessage::ProjectEditPrSubmittedStallTimeoutSecondsChanged(
                    pr_submitted_stall_timeout_seconds.saturating_add(5).min(3600),
                ),
            ))
            .padding([3, 8])
            .style(rounded_action_btn_style),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    column![
        settings_section_card(
            "调度与保护",
            "限制并发、设置重试与超时，并控制 worktree 回收与合并锁。"
        ),
        settings_panel(
            column![
                scheduler_tick_interval_row,
                auto_promote_tick_interval_row,
                max_concurrent_row,
                failed_retry_row,
                running_timeout_row,
                recycle_worktree_on_finish_toggle,
                pr_stall_timeout_row,
            ]
            .spacing(14)
        )
    ]
    .spacing(12)
    .into()
}
#[cfg(test)]
#[path = "scheduling_tab_tests.rs"]
mod scheduling_tab_tests;
