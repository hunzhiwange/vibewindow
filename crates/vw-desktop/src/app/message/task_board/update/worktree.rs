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
TaskBoardMessage::CleanAllWorktreesPressed => {
    let Some(project_path) = app.project_path.clone() else {
        return iced::Task::none();
    };
    if app.task_board_worktree_manual_action_kind.is_some() {
        return iced::Task::none();
    }
    if app.task_board_worktree_manual_confirm_kind != Some("cleanup") {
        app.task_board_worktree_manual_confirm_kind = Some("cleanup");
        app.push_notification(
            "一键清理风险较高，请再次点击“确认一键清理”继续，或点击“取消”返回".to_string(),
        );
        app.notifications_expanded = true;
        return iced::Task::none();
    }
    let (log_tx, log_rx) = std::sync::mpsc::channel();
    app.task_board_worktree_manual_confirm_kind = None;
    app.task_board_worktree_manual_action_kind = Some("cleanup");
    app.task_board_worktree_action_logs.clear();
    app.task_board_worktree_action_logs_visible_until_ms = None;
    app.task_board_worktree_action_log_rx = Some(log_rx);
    push_worktree_action_log(app, "准备开始一键清理...".to_string());
    iced::Task::batch(vec![
        iced::Task::perform(
            crate::app::task::reset_all_managed_worktrees_async_with_logs(
                project_path,
                true,
                Some(log_tx),
            ),
            |result| Message::TaskBoard(TaskBoardMessage::CleanAllWorktreesCompleted(result)),
        ),
        schedule_worktree_action_log_tick(),
    ])
}
TaskBoardMessage::CleanAllWorktreesCompleted(result) => {
    poll_worktree_action_logs(app);
    match result {
        Ok(count) => app.push_notification(format!("已完成 worktree 一键清理，共处理 {} 个槽位", count)),
        Err(error) => {
            app.push_notification(format!(
                "worktree 一键清理有失败，请查看右下角通知：{}",
                truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
            ));
            app.notifications_expanded = true;
        }
    }
    finish_worktree_action_logs(app);
    maybe_schedule_worktree_snapshot_refresh(app, true)
}
TaskBoardMessage::DeleteAllWorktreesPressed => {
    let Some(project_path) = app.project_path.clone() else {
        return iced::Task::none();
    };
    if app.task_board_worktree_manual_action_kind.is_some() {
        return iced::Task::none();
    }
    if app.task_board_worktree_manual_confirm_kind != Some("delete") {
        app.task_board_worktree_manual_confirm_kind = Some("delete");
        app.push_notification(
            "删除所有 worktree 风险极高，请再次点击“确认删除所有 worktree”继续，或点击“取消”返回".to_string(),
        );
        app.notifications_expanded = true;
        return iced::Task::none();
    }
    let (log_tx, log_rx) = std::sync::mpsc::channel();
    app.task_board_worktree_manual_confirm_kind = None;
    app.task_board_worktree_manual_action_kind = Some("delete");
    app.task_board_worktree_action_logs.clear();
    app.task_board_worktree_action_logs_visible_until_ms = None;
    app.task_board_worktree_action_log_rx = Some(log_rx);
    push_worktree_action_log(app, "准备开始删除所有 worktree...".to_string());
    iced::Task::batch(vec![
        iced::Task::perform(
            crate::app::task::delete_all_managed_worktrees_async_with_logs(project_path, Some(log_tx)),
            |result| Message::TaskBoard(TaskBoardMessage::DeleteAllWorktreesCompleted(result)),
        ),
        schedule_worktree_action_log_tick(),
    ])
}
TaskBoardMessage::DeleteAllWorktreesCompleted(result) => {
    poll_worktree_action_logs(app);
    match result {
        Ok(count) => app.push_notification(format!("已删除所有 worktree，共处理 {} 个槽位", count)),
        Err(error) => {
            app.push_notification(format!(
                "删除所有 worktree 有失败，请查看右下角通知：{}",
                truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
            ));
            app.notifications_expanded = true;
        }
    }
    finish_worktree_action_logs(app);
    maybe_schedule_worktree_snapshot_refresh(app, true)
}
TaskBoardMessage::CommitMergeAllWorktreesPressed => {
    let Some(project_path) = app.project_path.clone() else {
        return iced::Task::none();
    };
    let tasks = app.task_board_tasks.clone();
    if app.task_board_worktree_manual_action_kind.is_some() {
        return iced::Task::none();
    }
    if app.task_board_worktree_manual_confirm_kind != Some("merge") {
        app.task_board_worktree_manual_confirm_kind = Some("merge");
        app.push_notification(
            "一键合并风险较高，请再次点击“确认一键合并”继续，或点击“取消”返回".to_string(),
        );
        app.notifications_expanded = true;
        return iced::Task::none();
    }
    let (log_tx, log_rx) = std::sync::mpsc::channel();
    app.task_board_worktree_manual_confirm_kind = None;
    app.task_board_worktree_manual_action_kind = Some("merge");
    app.task_board_worktree_action_logs.clear();
    app.task_board_worktree_action_logs_visible_until_ms = None;
    app.task_board_worktree_action_log_rx = Some(log_rx);
    push_worktree_action_log(app, "准备开始一键合并...".to_string());
    iced::Task::batch(vec![
        iced::Task::perform(
            crate::app::task::commit_merge_all_worktrees_async_with_logs(project_path, tasks, Some(log_tx)),
            |result| Message::TaskBoard(TaskBoardMessage::CommitMergeAllWorktreesCompleted(result)),
        ),
        schedule_worktree_action_log_tick(),
    ])
}
TaskBoardMessage::CommitMergeAllWorktreesCompleted(result) => {
    poll_worktree_action_logs(app);
    match result {
        Ok(count) => app.push_notification(format!("已完成 worktree 一键合并，共合并 {} 个分支", count)),
        Err(error) => {
            app.push_notification(format!(
                "worktree 一键合并有失败，请查看右下角通知：{}",
                truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
            ));
            app.notifications_expanded = true;
        }
    }
    finish_worktree_action_logs(app);
    maybe_schedule_worktree_snapshot_refresh(app, true)
}
TaskBoardMessage::CancelWorktreeManualConfirm => {
    let Some(kind) = app.task_board_worktree_manual_confirm_kind.take() else {
        return iced::Task::none();
    };
    let action_label = match kind {
        "cleanup" => "一键清理",
        "delete" => "一键删除",
        "merge" => "一键合并",
        _ => "worktree 操作",
    };
    app.push_notification(format!("已取消 {} 确认，可重新发起操作", action_label));
    iced::Task::none()
}
TaskBoardMessage::WorktreeActionLogTick => {
    poll_worktree_action_logs(app);
    if app.task_board_worktree_action_log_rx.is_some() {
        return schedule_worktree_action_log_tick();
    }
    if let Some(deadline_ms) = app.task_board_worktree_action_logs_visible_until_ms
        && now_ms() >= deadline_ms
    {
        clear_worktree_action_logs(app);
    }
    iced::Task::none()
}
TaskBoardMessage::UiTick => {
    poll_worktree_action_logs(app);
    if app.task_board_worktree_action_log_rx.is_none()
        && let Some(deadline_ms) = app.task_board_worktree_action_logs_visible_until_ms
        && now_ms() >= deadline_ms
    {
        clear_worktree_action_logs(app);
    }
    iced::Task::batch(vec![maybe_schedule_worktree_snapshot_refresh(app, false)])
}
TaskBoardMessage::ColumnScrollChanged {
    status,
    has_vertical_scrollbar,
} => {
    app.task_board_column_has_vertical_scrollbar.insert(status, has_vertical_scrollbar);
    iced::Task::none()
}
TaskBoardMessage::WorktreePoolMaintained { result } => {
    app.task_board_worktree_maintenance_in_flight = false;
    let refresh_snapshot = maybe_schedule_worktree_snapshot_refresh(app, true);
    if let Err(error) = result && let Some(project_path) = &app.project_path {
        let mut changed = false;
        for task in &mut app.task_board_tasks {
            if task.status == TaskStatus::Pending
                || task.status == TaskStatus::Running
                || task.status == TaskStatus::CodeReview
                || task.status == TaskStatus::PrSubmitted
            {
                task.add_log(format!(
                    "后台 worktree 池维护失败: {}",
                    truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                ));
                changed = true;
            }
        }
        if changed {
            let persist_tasks = build_persist_tasks(project_path, &app.task_board_tasks);
            if !persist_tasks.is_empty() {
                let mut tasks = persist_tasks;
                tasks.push(refresh_snapshot);
                return iced::Task::batch(tasks);
            }
        }
    }
    refresh_snapshot
}
TaskBoardMessage::WorktreeSnapshotLoaded(snapshot) => {
    app.task_board_worktree_snapshot_loading = false;
    app.task_board_last_worktree_snapshot_at_ms = now_ms();
    app.task_board_worktree_snapshot = snapshot;
    iced::Task::none()
}
TaskBoardMessage::TaskWorktreeRecycled { task_id, result } => {
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        match result {
            Ok(()) => {
                task.selected_worktree_path = None;
                task.add_log("任务 worktree 已回收".to_string());
            }
            Err(error) => {
                task.add_log(format!(
                    "任务 worktree 回收失败: {}",
                    truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                ));
            }
        }
    }
    sync_viewing_logs(app, &task_id);

    let refresh_snapshot = maybe_schedule_worktree_snapshot_refresh(app, true);
    if let Some(project_path) = &app.project_path
        && let Some(task_clone) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
    {
        let path = project_path.clone();
        let persist_task = iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            |_| Message::None,
        );
        return iced::Task::batch(vec![persist_task, refresh_snapshot]);
    }
    refresh_snapshot
}
TaskBoardMessage::TaskWorktreeReleased { task_id, result } => {
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        match result {
            Ok(()) => {
                task.selected_worktree_path = None;
                task.add_log("任务 worktree 已释放，保留现场供下次复用".to_string());
            }
            Err(error) => {
                task.add_log(format!(
                    "任务 worktree 释放失败: {}",
                    truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                ));
            }
        }
    }
    sync_viewing_logs(app, &task_id);

    let refresh_snapshot = maybe_schedule_worktree_snapshot_refresh(app, true);
    if let Some(project_path) = &app.project_path
        && let Some(task_clone) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
    {
        let path = project_path.clone();
        let persist_task = iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            |_| Message::None,
        );
        return iced::Task::batch(vec![persist_task, refresh_snapshot]);
    }
    refresh_snapshot
}
TaskBoardMessage::ToggleWorktreePanelExpanded => {
    app.task_board_worktree_panel_expanded = !app.task_board_worktree_panel_expanded;
    maybe_schedule_worktree_snapshot_refresh(app, true)
}
    )
}
#[cfg(test)]
#[path = "worktree_tests.rs"]
mod worktree_tests;
