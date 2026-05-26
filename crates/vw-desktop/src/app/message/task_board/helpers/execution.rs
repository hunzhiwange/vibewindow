//! 提供任务看板消息处理过程中复用的局部辅助逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

/// TASK_LOG_UI_MAX_ENTRY_CHARS 使用的固定配置值。
pub(crate) const TASK_LOG_UI_MAX_ENTRY_CHARS: usize = 1200;
/// TASK_LOG_UI_MAX_DETAIL_CHARS 使用的固定配置值。
pub(crate) const TASK_LOG_UI_MAX_DETAIL_CHARS: usize = 2000;
/// TASK_LOG_FLUSH_INTERVAL_MS 使用的固定配置值。
pub(crate) const TASK_LOG_FLUSH_INTERVAL_MS: u64 = 800;
/// TASK_EXECUTION_SCAN_BUDGET_MS 使用的固定配置值。
pub(crate) const TASK_EXECUTION_SCAN_BUDGET_MS: u64 = 8;
/// TASK_RUNNING_LOG_SCAN_BATCH 使用的固定配置值。
pub(crate) const TASK_RUNNING_LOG_SCAN_BATCH: usize = 6;
/// TASK_TIMEOUT_SCAN_BATCH 使用的固定配置值。
pub(crate) const TASK_TIMEOUT_SCAN_BATCH: usize = 32;
/// TASK_SCHEDULE_SCAN_BATCH 使用的固定配置值。
pub(crate) const TASK_SCHEDULE_SCAN_BATCH: usize = 48;
#[allow(dead_code)]
/// TASK_BOARD_REFRESH_INTERVAL_SECS 使用的固定配置值。
pub(crate) const TASK_BOARD_REFRESH_INTERVAL_SECS: u64 = 60;
#[allow(dead_code)]
/// TASK_SCHEDULER_TICK_INTERVAL_SECS 使用的固定配置值。
pub(crate) const TASK_SCHEDULER_TICK_INTERVAL_SECS: u64 = 1;
/// TASK_AUTO_CODE_REVIEW_TICK_INTERVAL_SECS 使用的固定配置值。
pub(crate) const TASK_AUTO_CODE_REVIEW_TICK_INTERVAL_SECS: u64 = 30;
#[allow(dead_code)]
/// TASK_AUTO_PROMOTE_TICK_INTERVAL_SECS 使用的固定配置值。
pub(crate) const TASK_AUTO_PROMOTE_TICK_INTERVAL_SECS: u64 = 30;
/// TASK_BOARD_WORKTREE_SNAPSHOT_INTERVAL_MS 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_SNAPSHOT_INTERVAL_MS: u64 = 15_000;
/// TASK_BOARD_WORKTREE_SNAPSHOT_EXPANDED_INTERVAL_MS 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_SNAPSHOT_EXPANDED_INTERVAL_MS: u64 = 5_000;
/// TASK_BOARD_WORKTREE_ACTION_LOG_TICK_MS 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_ACTION_LOG_TICK_MS: u64 = 120;
/// TASK_BOARD_WORKTREE_ACTION_LOG_HIDE_DELAY_MS 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_ACTION_LOG_HIDE_DELAY_MS: u64 = 1200;
/// TASK_BOARD_WORKTREE_ACTION_LOG_MAX_LINES 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_ACTION_LOG_MAX_LINES: usize = 14;
/// TASK_BOARD_WORKTREE_ACTION_LOG_MAX_ENTRY_CHARS 使用的固定配置值。
pub(crate) const TASK_BOARD_WORKTREE_ACTION_LOG_MAX_ENTRY_CHARS: usize = 320;

/// 执行 truncate_for_ui 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn truncate_for_ui(value: &str, max_chars: usize) -> String {
    let total = value.chars().count();
    if total <= max_chars {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str(&format!(" ...(已截断，共{}字符)", total));
    truncated
}

/// 执行 format_task_log_stream_for_ui 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn format_task_log_stream_for_ui(
    log: &TaskLogStream,
    max_chars: usize,
) -> Option<String> {
    match log {
        TaskLogStream::Stdout(line) | TaskLogStream::Stderr(line) => {
            if line.trim().is_empty() {
                return None;
            }
            Some(truncate_for_ui(line, max_chars))
        }
        TaskLogStream::ExitStatus {
            success,
            code,
            signal,
            ..
        } => Some(if *success {
            format!("执行成功 code={:?}", code)
        } else {
            match signal {
                Some(signal) => format!("执行失败 code={:?} signal={}", code, signal),
                None => format!("执行失败 code={:?}", code),
            }
        }),
    }
}

/// 执行 push_worktree_action_log 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn push_worktree_action_log(app: &mut crate::app::App, line: String) {
    app.task_board_worktree_action_logs.push(line);
    if app.task_board_worktree_action_logs.len() > TASK_BOARD_WORKTREE_ACTION_LOG_MAX_LINES {
        let overflow =
            app.task_board_worktree_action_logs.len() - TASK_BOARD_WORKTREE_ACTION_LOG_MAX_LINES;
        app.task_board_worktree_action_logs.drain(0..overflow);
    }
}

/// 执行 clear_worktree_action_logs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn clear_worktree_action_logs(app: &mut crate::app::App) {
    app.task_board_worktree_manual_action_kind = None;
    app.task_board_worktree_manual_confirm_kind = None;
    app.task_board_worktree_action_logs.clear();
    app.task_board_worktree_action_logs_visible_until_ms = None;
    app.task_board_worktree_action_log_rx = None;
}

/// 执行 finish_worktree_action_logs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn finish_worktree_action_logs(app: &mut crate::app::App) {
    app.task_board_worktree_manual_action_kind = None;
    app.task_board_worktree_manual_confirm_kind = None;
    app.task_board_worktree_action_log_rx = None;
    if app.task_board_worktree_action_logs.is_empty() {
        app.task_board_worktree_action_logs_visible_until_ms = None;
    } else {
        app.task_board_worktree_action_logs_visible_until_ms =
            Some(now_ms().saturating_add(TASK_BOARD_WORKTREE_ACTION_LOG_HIDE_DELAY_MS));
    }
}

/// 执行 poll_worktree_action_logs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn poll_worktree_action_logs(app: &mut crate::app::App) {
    let Some(receiver) = app.task_board_worktree_action_log_rx.take() else {
        return;
    };

    loop {
        match receiver.try_recv() {
            Ok(log) => {
                if let Some(line) = format_task_log_stream_for_ui(
                    &log,
                    TASK_BOARD_WORKTREE_ACTION_LOG_MAX_ENTRY_CHARS,
                ) {
                    push_worktree_action_log(app, line);
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                app.task_board_worktree_action_log_rx = Some(receiver);
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                app.task_board_worktree_action_log_rx = None;
                break;
            }
        }
    }
}

/// 执行 append_task_log_stream 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn append_task_log_stream(task: &mut Task, log: &TaskLogStream) {
    if let Some(line) = format_task_log_stream_for_ui(log, TASK_LOG_UI_MAX_ENTRY_CHARS) {
        task.add_log(line);
    }
}

/// 执行 flush_running_task_logs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn flush_running_task_logs(
    app: &mut crate::app::App,
    tick_started_at: web_time::Instant,
) {
    let running = app.task_board_executor.running_tasks.clone();
    if running.is_empty() {
        app.task_board_log_scan_cursor = 0;
        return;
    }
    if app.task_board_log_scan_cursor >= running.len() {
        app.task_board_log_scan_cursor = 0;
    }
    let max_batch = TASK_RUNNING_LOG_SCAN_BATCH.min(running.len());
    let viewing_task_id = app.task_board_viewing_logs.as_ref().map(|task| task.id.clone());
    let mut scanned = 0usize;

    while scanned < max_batch {
        if scanned > 0
            && tick_started_at.elapsed()
                >= std::time::Duration::from_millis(TASK_EXECUTION_SCAN_BUDGET_MS)
        {
            break;
        }
        let idx = (app.task_board_log_scan_cursor + scanned) % running.len();
        let task_id = running[idx].clone();
        let task_logs = app.task_board_executor.poll_task_logs(&task_id);
        if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
            for log in &task_logs {
                append_task_log_stream(task, log);
            }
        }
        if viewing_task_id.as_deref() == Some(task_id.as_str()) {
            sync_viewing_logs(app, &task_id);
        }
        scanned += 1;
    }
    if scanned > 0 {
        app.task_board_log_scan_cursor = (app.task_board_log_scan_cursor + scanned) % running.len();
    }
}

/// 执行 flush_running_task_logs_throttled 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn flush_running_task_logs_throttled(
    app: &mut crate::app::App,
    force: bool,
    tick_started_at: web_time::Instant,
) {
    let now = now_ms();
    if !force
        && now.saturating_sub(app.task_board_last_log_flush_at_ms) < TASK_LOG_FLUSH_INTERVAL_MS
    {
        return;
    }
    flush_running_task_logs(app, tick_started_at);
    app.task_board_last_log_flush_at_ms = now;
}

/// 执行 release_task_worktree_if_possible 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn release_task_worktree_if_possible(
    app: &crate::app::App,
    task_id: &str,
) -> iced::Task<crate::app::Message> {
    if let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        let task_id = task_id.to_string();
        return iced::Task::perform(
            crate::app::task::release_task_worktree_async(path, task_id.clone()),
            |(task_id, result)| {
                crate::app::Message::TaskBoard(TaskBoardMessage::TaskWorktreeReleased {
                    task_id,
                    result,
                })
            },
        );
    }
    iced::Task::none()
}

/// 执行 schedule_worktree_action_log_tick 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn schedule_worktree_action_log_tick() -> iced::Task<crate::app::Message> {
    crate::app::message::after(
        std::time::Duration::from_millis(TASK_BOARD_WORKTREE_ACTION_LOG_TICK_MS),
        crate::app::Message::TaskBoard(TaskBoardMessage::WorktreeActionLogTick),
    )
}

/// 执行 schedule_worktree_pool_maintenance 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn schedule_worktree_pool_maintenance(
    app: &mut crate::app::App,
    project_path: &str,
    running_tasks: usize,
) -> iced::Task<crate::app::Message> {
    use crate::app::Message;

    if app.task_board_worktree_maintenance_in_flight
        || app.task_board_worktree_manual_action_kind.is_some()
        || !crate::app::task::worktree_pool_needs_maintenance(project_path, running_tasks)
    {
        return iced::Task::none();
    }

    app.task_board_worktree_maintenance_in_flight = true;
    let path = project_path.to_string();
    iced::Task::perform(
        async move {
            crate::app::message::spawn_blocking_opt(move || {
                Some(crate::app::task::maintain_worktree_pool(&path, running_tasks))
            })
            .await
            .unwrap_or_else(|| Err("worktree 池维护任务未返回结果".to_string()))
        },
        |result| Message::TaskBoard(TaskBoardMessage::WorktreePoolMaintained { result }),
    )
}

/// 执行 worktree_snapshot_refresh_interval_ms 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn worktree_snapshot_refresh_interval_ms(app: &crate::app::App) -> u64 {
    if app.task_board_worktree_panel_expanded {
        TASK_BOARD_WORKTREE_SNAPSHOT_EXPANDED_INTERVAL_MS
    } else {
        TASK_BOARD_WORKTREE_SNAPSHOT_INTERVAL_MS
    }
}

/// 执行 maybe_schedule_worktree_snapshot_refresh 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn maybe_schedule_worktree_snapshot_refresh(
    app: &mut crate::app::App,
    force: bool,
) -> iced::Task<crate::app::Message> {
    use crate::app::Message;

    let Some(project_path) = app.project_path.clone() else {
        app.task_board_worktree_snapshot = None;
        app.task_board_worktree_snapshot_loading = false;
        app.task_board_last_worktree_snapshot_at_ms = 0;
        return iced::Task::none();
    };

    if app.task_board_worktree_snapshot_loading {
        return iced::Task::none();
    }

    let now = now_ms();
    let interval_ms = worktree_snapshot_refresh_interval_ms(app);
    if !force && now.saturating_sub(app.task_board_last_worktree_snapshot_at_ms) < interval_ms {
        return iced::Task::none();
    }

    app.task_board_worktree_snapshot_loading = true;
    let path = project_path.clone();
    iced::Task::perform(
        async move {
            crate::app::message::spawn_blocking_opt(move || {
                Some(crate::app::task::worktree_pool_snapshot(&path))
            })
            .await
            .flatten()
        },
        |snapshot| Message::TaskBoard(TaskBoardMessage::WorktreeSnapshotLoaded(snapshot)),
    )
}

/// 执行 current_running_execution_count 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn current_running_execution_count(app: &crate::app::App) -> u32 {
    app.task_board_executor
        .running_tasks
        .iter()
        .filter(|task_id| {
            app.task_board_tasks
                .iter()
                .find(|task| task.id == **task_id)
                .is_some_and(|task| task.status != TaskStatus::PrSubmitted)
        })
        .count() as u32
}

/// 执行 current_pending_task_count 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn current_pending_task_count(app: &crate::app::App) -> u32 {
    app.task_board_tasks
        .iter()
        .filter(|task| !task.deleted && !task.archived && task.status == TaskStatus::Pending)
        .count() as u32
}

/// 执行 minutes_to_ms 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn minutes_to_ms(minutes: u32) -> u64 {
    (minutes.max(1) as u64).saturating_mul(60_000)
}

/// 执行 build_auto_execute_bootstrap_tasks 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn build_auto_execute_bootstrap_tasks(
    app: &mut crate::app::App,
) -> Vec<iced::Task<crate::app::Message>> {
    use crate::app::Message;

    app.task_board_next_auto_promote_tick_at_ms =
        next_deadline_ms(auto_promote_tick_interval_secs(app));
    let mut tasks =
        vec![iced::Task::done(Message::TaskBoard(TaskBoardMessage::PromotePoolTasksTick))];
    tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::AutoCodeReviewTick)));
    if app.task_board_executor_running {
        tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::ExecutionTick)));
    } else {
        tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::StartExecution)));
    }
    tasks
}

/// 执行 apply_execution_timeouts 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn apply_execution_timeouts(
    app: &mut crate::app::App,
    tick_started_at: web_time::Instant,
) -> (Vec<Task>, Vec<iced::Task<crate::app::Message>>) {
    let now = now_ms();
    let failed_retry_ms = minutes_to_ms(app.task_board_settings.failed_retry_minutes);
    let running_timeout_ms = minutes_to_ms(app.task_board_settings.running_timeout_minutes);
    let pr_stall_timeout_secs = pr_submitted_stall_timeout_secs(app);
    let pr_stall_timeout_ms = pr_stall_timeout_secs.saturating_mul(1000);
    let mut changed_tasks = Vec::new();
    let mut recycle_tasks = Vec::new();
    let mut changed_task_ids = Vec::new();
    let mut timed_out_running_ids = Vec::new();
    let mut stalled_merge_ids = Vec::new();

    if app.task_board_tasks.is_empty() {
        app.task_board_timeout_scan_cursor = 0;
        return (changed_tasks, recycle_tasks);
    }
    if app.task_board_timeout_scan_cursor >= app.task_board_tasks.len() {
        app.task_board_timeout_scan_cursor = 0;
    }
    let max_batch = TASK_TIMEOUT_SCAN_BATCH.min(app.task_board_tasks.len());
    let mut scanned = 0usize;

    while scanned < max_batch {
        if scanned > 0
            && tick_started_at.elapsed()
                >= std::time::Duration::from_millis(TASK_EXECUTION_SCAN_BUDGET_MS)
        {
            break;
        }
        let idx = (app.task_board_timeout_scan_cursor + scanned) % app.task_board_tasks.len();
        let task = &mut app.task_board_tasks[idx];
        if task.deleted || task.archived {
            scanned += 1;
            continue;
        }
        match task.status {
            TaskStatus::Failed => {
                if now.saturating_sub(task.last_active_at_ms) >= failed_retry_ms {
                    task.set_status(TaskStatus::Pending);
                    task.add_log(format!(
                        "失败后等待 {} 分钟，自动回推到待执行",
                        app.task_board_settings.failed_retry_minutes.max(1)
                    ));
                    changed_task_ids.push(task.id.clone());
                    changed_tasks.push(task.clone());
                }
            }
            TaskStatus::Running => {
                let started_at = task.execution_started_at_ms.unwrap_or(task.last_active_at_ms);
                if now.saturating_sub(started_at) >= running_timeout_ms {
                    task.mark_execution_failed(format!(
                        "执行超过 {} 分钟仍未完成，已自动标记失败",
                        app.task_board_settings.running_timeout_minutes.max(1)
                    ));
                    timed_out_running_ids.push(task.id.clone());
                    changed_task_ids.push(task.id.clone());
                    changed_tasks.push(task.clone());
                }
            }
            TaskStatus::PrSubmitted => {
                let inactive_for_ms = now.saturating_sub(task.last_active_at_ms);
                if inactive_for_ms >= pr_stall_timeout_ms {
                    if let Some(project_path) = app.project_path.as_deref() {
                        let holder = crate::app::task::task_merge_lock_holder(project_path, task)
                            .unwrap_or_else(|| "none".to_string());
                        let selected_worktree = task
                            .selected_worktree_path
                            .clone()
                            .unwrap_or_else(|| "none".to_string());
                        let target_branch = task
                            .merge_target_branch
                            .as_deref()
                            .unwrap_or("none")
                            .to_string();
                        task.add_log(format!(
                            "合并阶段超时检测: inactive_ms={} worktree={} target={} lock_holder={}",
                            inactive_for_ms, selected_worktree, target_branch, holder
                        ));
                        crate::app::task::force_unlock_task_merge_target(project_path, task);
                        let holder_after_release =
                            crate::app::task::task_merge_lock_holder(project_path, task)
                                .unwrap_or_else(|| "none".to_string());
                        task.add_log(format!(
                            "合并阶段超时已释放锁: target={} holder_after_release={}",
                            target_branch, holder_after_release
                        ));
                    }
                    task.mark_paused(format!(
                        "合并阶段静默超过 {} 秒未完成，已自动暂停",
                        pr_stall_timeout_secs
                    ));
                    stalled_merge_ids.push(task.id.clone());
                    changed_task_ids.push(task.id.clone());
                    changed_tasks.push(task.clone());
                }
            }
            _ => {}
        }
        scanned += 1;
    }
    if scanned > 0 {
        app.task_board_timeout_scan_cursor =
            (app.task_board_timeout_scan_cursor + scanned) % app.task_board_tasks.len();
    }

    for task_id in timed_out_running_ids {
        app.task_board_executor.finish_task(&task_id);
        let worktree_task = if should_recycle_worktree_on_task_finish(app) {
            recycle_task_worktree_if_possible(
                app,
                &task_id,
                Some("任务执行超时，回收 worktree".to_string()),
            )
        } else {
            release_task_worktree_if_possible(app, &task_id)
        };
        recycle_tasks.push(worktree_task);
    }
    for task_id in stalled_merge_ids {
        app.task_board_executor.finish_task(&task_id);
        let worktree_task = if should_recycle_worktree_on_task_finish(app) {
            recycle_task_worktree_if_possible(
                app,
                &task_id,
                Some("合并阶段超时，回收 worktree".to_string()),
            )
        } else {
            release_task_worktree_if_possible(app, &task_id)
        };
        recycle_tasks.push(worktree_task);
    }
    for task_id in changed_task_ids {
        sync_viewing_logs(app, &task_id);
    }

    (changed_tasks, recycle_tasks)
}

/// 执行 pick_next_pending_task_for_execution 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn pick_next_pending_task_for_execution(
    app: &mut crate::app::App,
    tick_started_at: web_time::Instant,
    exclude_ids: &[String],
) -> Option<String> {
    if app.task_board_tasks.is_empty() {
        app.task_board_schedule_scan_cursor = 0;
        return None;
    }
    if app.task_board_schedule_scan_cursor >= app.task_board_tasks.len() {
        app.task_board_schedule_scan_cursor = 0;
    }
    let max_batch = TASK_SCHEDULE_SCAN_BATCH.min(app.task_board_tasks.len());
    let mut scanned = 0usize;
    let mut selected_idx: Option<usize> = None;

    while scanned < max_batch {
        if scanned > 0
            && tick_started_at.elapsed()
                >= std::time::Duration::from_millis(TASK_EXECUTION_SCAN_BUDGET_MS)
        {
            break;
        }
        let idx = (app.task_board_schedule_scan_cursor + scanned) % app.task_board_tasks.len();
        let task = &app.task_board_tasks[idx];
        if !task.deleted
            && !task.archived
            && task.status == TaskStatus::Pending
            && !exclude_ids.contains(&task.id)
        {
            let should_replace = match selected_idx {
                Some(prev_idx) => {
                    let prev = &app.task_board_tasks[prev_idx];
                    (task.priority, task.order) < (prev.priority, prev.order)
                }
                None => true,
            };
            if should_replace {
                selected_idx = Some(idx);
            }
        }
        scanned += 1;
    }
    if scanned > 0 {
        app.task_board_schedule_scan_cursor =
            (app.task_board_schedule_scan_cursor + scanned) % app.task_board_tasks.len();
    }
    selected_idx.map(|idx| app.task_board_tasks[idx].id.clone())
}

/// 执行 pick_next_pr_submitted_task_for_merge 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn pick_next_pr_submitted_task_for_merge(
    app: &crate::app::App,
    exclude_ids: &[String],
) -> Option<String> {
    app.task_board_tasks
        .iter()
        .filter(|task| {
            !task.deleted
                && !task.archived
                && task.status == TaskStatus::PrSubmitted
                && !exclude_ids.contains(&task.id)
        })
        .min_by_key(|task| (task.priority, task.order))
        .map(|task| task.id.clone())
}

/// 执行 pick_next_code_review_task 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn pick_next_code_review_task(
    app: &crate::app::App,
    exclude_ids: &[String],
) -> Option<String> {
    app.task_board_tasks
        .iter()
        .filter(|task| {
            !task.deleted
                && !task.archived
                && task.status == TaskStatus::CodeReview
                && !exclude_ids.contains(&task.id)
        })
        .min_by_key(|task| (task.priority, task.order))
        .map(|task| task.id.clone())
}

/// 执行 build_persist_tasks 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn build_persist_tasks(
    project_path: &str,
    tasks: &[Task],
) -> Vec<iced::Task<crate::app::Message>> {
    use crate::app::Message;

    tasks
        .iter()
        .cloned()
        .map(|task| {
            let path = project_path.to_string();
            iced::Task::perform(
                async move { crate::app::task::update_task(&path, &task) },
                |_| Message::None,
            )
        })
        .collect()
}

/// 执行 recycle_task_worktree_if_possible 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn recycle_task_worktree_if_possible(
    app: &crate::app::App,
    task_id: &str,
    taint_reason: Option<String>,
) -> iced::Task<crate::app::Message> {
    if !should_recycle_worktree_on_task_finish(app) {
        return iced::Task::none();
    }
    if let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        let task_id = task_id.to_string();
        return iced::Task::perform(
            crate::app::task::recycle_task_worktree_async(path, task_id.clone(), taint_reason),
            |(task_id, result)| {
                crate::app::Message::TaskBoard(TaskBoardMessage::TaskWorktreeRecycled {
                    task_id,
                    result,
                })
            },
        );
    }
    iced::Task::none()
}
#[cfg(test)]
#[path = "execution_tests.rs"]
mod execution_tests;
