//! 处理任务看板状态更新分支，将 UI 消息转换为应用状态变更和异步任务。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

/// 执行 update 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn update(
    app: &mut crate::app::App,
    message: TaskBoardMessage,
) -> iced::Task<crate::app::Message> {
    dispatch_task_board_messages!(message,
TaskBoardMessage::OpenSettingsModal => {
    app.task_board_settings_modal_open = true;
    app.task_board_settings_modal_tab = TaskBoardSettingsModalTab::default();
    iced::Task::none()
}
TaskBoardMessage::CloseSettingsModal => {
    app.task_board_settings_modal_open = false;
    iced::Task::none()
}
TaskBoardMessage::SelectSettingsModalTab(tab) => {
    app.task_board_settings_modal_tab = tab;
    iced::Task::none()
}
TaskBoardMessage::ToggleAutoExecute(enabled) => {
    app.task_board_settings.auto_execute = enabled;
    app.task_board_settings.auto_promote_pool_tasks = enabled;
    crate::app::set_config_field(
        "task_board_auto_promote_pool_tasks",
        serde_json::Value::Bool(enabled),
    );
    save_settings(app);
    if enabled {
        return iced::Task::batch(build_auto_execute_bootstrap_tasks(app));
    }
    iced::Task::none()
}
TaskBoardMessage::SetMaxConcurrent(count) => {
    app.task_board_settings.max_concurrent = count.clamp(1, 10);
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::ToggleAutoRefresh(enabled) => {
    app.task_board_settings.auto_refresh = enabled;
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetRefreshIntervalSeconds(seconds) => {
    app.task_board_settings.refresh_interval_seconds = seconds.clamp(1, 3600);
    app.task_board_next_refresh_at_ms = next_deadline_ms(task_board_refresh_interval_secs(app));
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetSchedulerTickIntervalSeconds(seconds) => {
    app.task_board_settings.scheduler_tick_interval_seconds = seconds.clamp(1, 60);
    app.task_board_next_scheduler_tick_at_ms = next_deadline_ms(scheduler_tick_interval_secs(app));
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetAutoPromoteTickIntervalSeconds(seconds) => {
    app.task_board_settings.auto_promote_tick_interval_seconds = seconds.clamp(1, 3600);
    app.task_board_next_auto_promote_tick_at_ms = next_deadline_ms(auto_promote_tick_interval_secs(app));
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetFailedRetryMinutes(minutes) => {
    app.task_board_settings.failed_retry_minutes = minutes.clamp(1, 1440);
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetRunningTimeoutMinutes(minutes) => {
    app.task_board_settings.running_timeout_minutes = minutes.clamp(1, 1440);
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::ToggleRecycleWorktreeOnTaskFinish(enabled) => {
    app.task_board_settings.recycle_worktree_on_task_finish = enabled;
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::SetPrSubmittedStallTimeoutSeconds(seconds) => {
    app.task_board_settings.pr_submitted_stall_timeout_seconds = seconds.clamp(5, 3600);
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::ToggleCodeReview(enabled) => {
    app.task_board_settings.code_review_enabled = enabled;
    crate::app::set_config_field(
        "task_board_code_review_enabled",
        serde_json::Value::Bool(enabled),
    );
    save_settings(app);
    if enabled && app.task_board_executor_running {
        return iced::Task::done(Message::TaskBoard(TaskBoardMessage::AutoCodeReviewTick));
    }
    iced::Task::none()
}
TaskBoardMessage::SettingsUpdated(mut settings) => {
    settings = settings.sanitized();
    app.task_board_settings = settings;
    app.task_board_next_refresh_at_ms = next_deadline_ms(task_board_refresh_interval_secs(app));
    app.task_board_next_scheduler_tick_at_ms = next_deadline_ms(scheduler_tick_interval_secs(app));
    app.task_board_next_auto_promote_tick_at_ms = next_deadline_ms(auto_promote_tick_interval_secs(app));
    save_settings(app);
    iced::Task::none()
}
TaskBoardMessage::ToggleAutoPromotePoolTasks(enabled) => {
    app.task_board_settings.auto_promote_pool_tasks = enabled;
    app.task_board_settings.auto_execute = enabled;
    crate::app::set_config_field(
        "task_board_auto_promote_pool_tasks",
        serde_json::Value::Bool(enabled),
    );
    save_settings(app);
    if enabled {
        return iced::Task::batch(build_auto_execute_bootstrap_tasks(app));
    }
    iced::Task::none()
}
TaskBoardMessage::ToggleWorktreePixelOffice(enabled) => {
    app.task_board_worktree_pixel_office = enabled;
    crate::app::set_config_field(
        "task_board_worktree_pixel_office",
        serde_json::Value::Bool(enabled),
    );
    iced::Task::none()
}
    )
}
#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;
