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
TaskBoardMessage::StartExecution => {
    app.task_board_executor_running = true;
    app.task_board_last_log_flush_at_ms = 0;
    app.task_board_log_scan_cursor = 0;
    app.task_board_timeout_scan_cursor = 0;
    app.task_board_schedule_scan_cursor = 0;
    app.task_board_next_scheduler_tick_at_ms = next_deadline_ms(scheduler_tick_interval_secs(app));
    app.task_board_next_auto_review_tick_at_ms =
        next_deadline_ms(TASK_AUTO_CODE_REVIEW_TICK_INTERVAL_SECS);
    iced::Task::batch(vec![
        iced::Task::done(Message::TaskBoard(TaskBoardMessage::ExecutionTick)),
        schedule_auto_review_tick_with_deadline(app),
    ])
}
TaskBoardMessage::StopExecution => {
    app.task_board_executor_running = false;
    iced::Task::none()
}
TaskBoardMessage::ExecutionTick => {
    let tick_started_at = web_time::Instant::now();
    flush_running_task_logs_throttled(app, false, tick_started_at);
    let (timeout_updated_tasks, timeout_recycle_tasks) = apply_execution_timeouts(app, tick_started_at);
    let has_timeout_recycle_tasks = !timeout_recycle_tasks.is_empty();

    if !app.task_board_executor_running {
        if app.task_board_executor.running_tasks.is_empty() {
            if let Some(project_path) = &app.project_path {
                let mut persist_tasks = build_persist_tasks(project_path, &timeout_updated_tasks);
                persist_tasks.extend(timeout_recycle_tasks);
                if !persist_tasks.is_empty() {
                    return iced::Task::batch(persist_tasks);
                }
            }
            return iced::Task::none();
        }
        let continue_tick = schedule_scheduler_tick_with_deadline(app);
        if let Some(project_path) = &app.project_path {
            let mut persist_tasks = build_persist_tasks(project_path, &timeout_updated_tasks);
            persist_tasks.extend(timeout_recycle_tasks);
            if !persist_tasks.is_empty() {
                persist_tasks.push(continue_tick);
                return iced::Task::batch(persist_tasks);
            }
        }
        if has_timeout_recycle_tasks {
            return continue_tick;
        }
        return continue_tick;
    }

    if let Some(project_path) = app.project_path.clone() {
        let should_schedule_worktree_maintenance = !app.task_board_worktree_maintenance_in_flight;
        let worktree_maintenance = schedule_worktree_pool_maintenance(
            app,
            &project_path,
            app.task_board_executor.running_tasks.len(),
        );
        let mut timeout_persist_tasks = build_persist_tasks(&project_path, &timeout_updated_tasks);
        timeout_persist_tasks.extend(timeout_recycle_tasks);
        if should_schedule_worktree_maintenance {
            timeout_persist_tasks.push(worktree_maintenance);
        }
        let running_count = current_running_execution_count(app);
        if running_count >= app.task_board_settings.max_concurrent {
            let continue_tick = schedule_scheduler_tick_with_deadline(app);
            if timeout_persist_tasks.is_empty() {
                return continue_tick;
            }
            timeout_persist_tasks.push(continue_tick);
            return iced::Task::batch(timeout_persist_tasks);
        }

        let path = project_path.clone();
        let exclude: Vec<String> = app.task_board_executor.running_tasks.clone();
        if let Some(task_id) = pick_next_pending_task_for_execution(app, tick_started_at, &exclude)
            && let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
        {
            let assigned_worktree_path = match crate::app::task::assign_task_execution_worktree(&path, &task, None) {
                Ok(path) => path,
                Err(error) => {
                    if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                        task_in_list.add_log(format!("[WORKTREE] 执行前预分配失败: {}", error));
                    }
                    sync_viewing_logs(app, &task_id);
                    let continue_tick = schedule_scheduler_tick_with_deadline(app);
                    let persist_task = if let Some(project_path) = &app.project_path {
                        app.task_board_tasks
                            .iter()
                            .find(|t| t.id == task_id)
                            .cloned()
                            .map(|task| {
                                let path = project_path.clone();
                                iced::Task::perform(
                                    async move { crate::app::task::update_task(&path, &task) },
                                    |_| Message::None,
                                )
                            })
                            .unwrap_or_else(iced::Task::none)
                    } else {
                        iced::Task::none()
                    };
                    timeout_persist_tasks.push(persist_task);
                    timeout_persist_tasks.push(continue_tick);
                    return iced::Task::batch(timeout_persist_tasks);
                }
            };

            app.task_board_executor.start_task(&task_id);
            app.task_board_executor.register_log_channel(task_id.clone());
            let log_sender = app.task_board_executor.get_log_sender(&task_id);

            if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                task_in_list.start_execution("开始执行任务".to_string());
                task_in_list.selected_worktree_path = assigned_worktree_path;
                if let Some(selected_path) = &task_in_list.selected_worktree_path {
                    task_in_list.add_log(format!("[WORKTREE] 执行前已分配工作区: {}", selected_path));
                }
                task_in_list.add_log(format!(
                    "调度参数: running_count={} max_concurrent={} exclude={:?}",
                    running_count, app.task_board_settings.max_concurrent, exclude
                ));
                task_in_list.add_log(format!(
                    "执行参数: acp_agent={} model={} prompt_chars={}",
                    task_execution_backend_label(task_in_list),
                    task_in_list.model,
                    task_in_list.prompt.chars().count()
                ));
            }
            sync_viewing_logs(app, &task_id);
            let start_task_persist = app
                .task_board_tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .map(|task| {
                    let path = path.clone();
                    iced::Task::perform(
                        async move { crate::app::task::update_task(&path, &task) },
                        |_| Message::None,
                    )
                })
                .unwrap_or_else(iced::Task::none);

            let execute_task_model = app
                .task_board_tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .unwrap_or(task);

            let path_clone = path.clone();
            let execute_task = iced::Task::perform(
                crate::app::task::execute_task_async(execute_task_model, path_clone, log_sender),
                move |(tid, result)| {
                    Message::TaskBoard(TaskBoardMessage::TaskExecutionCompleted { task_id: tid, result })
                },
            );

            let continue_tick = schedule_scheduler_tick_with_deadline(app);

            timeout_persist_tasks.push(start_task_persist);
            timeout_persist_tasks.push(execute_task);
            timeout_persist_tasks.push(continue_tick);
            return iced::Task::batch(timeout_persist_tasks);
        }
        if let Some(task_id) = pick_next_pr_submitted_task_for_merge(app, &exclude)
            && let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
        {
            if !crate::app::task::can_dispatch_merge_task(&project_path, &task) {
                let continue_tick = schedule_scheduler_tick_with_deadline(app);
                if timeout_persist_tasks.is_empty() {
                    return continue_tick;
                }
                timeout_persist_tasks.push(continue_tick);
                return iced::Task::batch(timeout_persist_tasks);
            }
            let should_schedule_worktree_maintenance = !app.task_board_worktree_maintenance_in_flight;
            let worktree_maintenance = schedule_worktree_pool_maintenance(
                app,
                &project_path,
                app.task_board_executor.running_tasks.len().saturating_add(1),
            );
            app.task_board_executor.start_task(&task_id);
            app.task_board_executor.register_log_channel(task_id.clone());
            let merge_sender = app.task_board_executor.get_log_sender(&task_id);
            if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                task_in_list.selected_worktree_path =
                    crate::app::task::current_task_worktree_path(&project_path, &task_id);
                task_in_list.start_merge_execution("开始执行合并任务".to_string());
                if let Some(selected_path) = &task_in_list.selected_worktree_path {
                    task_in_list.add_log(format!("选中工作区: {}", selected_path));
                }
                task_in_list.add_log(format!(
                    "合并调度参数: project_path={} source={} target={} lock_holder={} running_tasks={:?}",
                    project_path,
                    task_in_list.merge_source_branch.as_deref().unwrap_or("none"),
                    task_in_list.merge_target_branch.as_deref().unwrap_or("none"),
                    crate::app::task::task_merge_lock_holder(&project_path, task_in_list)
                        .unwrap_or_else(|| "none".to_string()),
                    app.task_board_executor.running_tasks
                ));
            }
            let start_task_persist = app
                .task_board_tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .map(|task| {
                    let path = path.clone();
                    iced::Task::perform(
                        async move { crate::app::task::update_task(&path, &task) },
                        |_| Message::None,
                    )
                })
                .unwrap_or_else(iced::Task::none);
            let execute_merge_task = iced::Task::perform(
                crate::app::task::execute_task_merge_async(task, path.clone(), merge_sender),
                move |(tid, result)| {
                    Message::TaskBoard(TaskBoardMessage::TaskMergeCompleted { task_id: tid, result })
                },
            );
            let continue_tick = schedule_scheduler_tick_with_deadline(app);
            if should_schedule_worktree_maintenance {
                timeout_persist_tasks.push(worktree_maintenance);
            }
            timeout_persist_tasks.push(start_task_persist);
            timeout_persist_tasks.push(execute_merge_task);
            timeout_persist_tasks.push(continue_tick);
            return iced::Task::batch(timeout_persist_tasks);
        }

        let continue_tick = schedule_scheduler_tick_with_deadline(app);
        if timeout_persist_tasks.is_empty() {
            return continue_tick;
        }
        timeout_persist_tasks.push(continue_tick);
        return iced::Task::batch(timeout_persist_tasks);
    }
    iced::Task::none()
}
TaskBoardMessage::AutoCodeReviewTick => {
    if !app.task_board_executor_running || !app.task_board_settings.code_review_enabled {
        return iced::Task::none();
    }
    let exclude: Vec<String> = app.task_board_executor.running_tasks.clone();
    if let Some(task_id) = pick_next_code_review_task(app, &exclude)
        && let (Some(task), Some(project_path)) = (
            app.task_board_tasks.iter().find(|t| t.id == task_id).cloned(),
            app.project_path.clone(),
        )
    {
        match build_code_review_prompt(&task, &project_path) {
            Ok(review_prompt) => {
                let should_schedule_worktree_maintenance = !app.task_board_worktree_maintenance_in_flight;
                let worktree_maintenance = schedule_worktree_pool_maintenance(
                    app,
                    &project_path,
                    app.task_board_executor.running_tasks.len().saturating_add(1),
                );
                app.task_board_executor.start_task(&task_id);
                app.task_board_executor.register_log_channel(task_id.clone());
                let review_sender = app.task_board_executor.get_log_sender(&task_id);
                let mut review_task = task.clone();
                review_task.prompt = review_prompt;
                if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                    task_in_list.selected_worktree_path =
                        crate::app::task::current_task_worktree_path(&project_path, &task_id);
                    if let Some(selected_path) = &task_in_list.selected_worktree_path {
                        task_in_list.add_log(format!("选中工作区: {}", selected_path));
                    }
                    task_in_list.add_log("自动审核调度触发".to_string());
                }
                sync_viewing_logs(app, &task_id);
                let mut review_tasks = Vec::new();
                if should_schedule_worktree_maintenance {
                    review_tasks.push(worktree_maintenance);
                }
                review_tasks.push(iced::Task::perform(
                    crate::app::task::execute_task_review_async(review_task, project_path, review_sender),
                    move |(tid, result)| {
                        Message::TaskBoard(TaskBoardMessage::TaskCodeReviewCompleted { task_id: tid, result })
                    },
                ));
                review_tasks.push(schedule_auto_review_tick_with_deadline(app));
                return iced::Task::batch(review_tasks);
            }
            Err(error) => {
                if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                    task_in_list.mark_paused(format!("生成审核提示失败: {}", error));
                }
                sync_viewing_logs(app, &task_id);
            }
        }
    }
    schedule_auto_review_tick_with_deadline(app)
}
TaskBoardMessage::SimulateStep { task_id } => {
    if !app.task_board_executor_running {
        return iced::Task::none();
    }

    if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id) {
        let current_status = task.status;
        if let Some(new_status) = crate::app::task::simulate_task_execution_step(
            &app.project_path.clone().unwrap_or_default(),
            &task_id,
            current_status,
        ) {
            app.task_board_executor.finish_task(&task_id);
            let recycle_task = recycle_task_worktree_if_possible(app, &task_id, None);

            return iced::Task::batch(vec![
                iced::Task::done(Message::TaskBoard(TaskBoardMessage::TaskStatusChanged {
                    task_id: task_id.clone(),
                    new_status,
                })),
                recycle_task,
                schedule_scheduler_tick_with_deadline(app),
            ]);
        }
    }

    app.task_board_executor.finish_task(&task_id);
    iced::Task::batch(vec![
        recycle_task_worktree_if_possible(app, &task_id, None),
        schedule_scheduler_tick_with_deadline(app),
    ])
}
TaskBoardMessage::PromotePoolTasksTick => {
    if !app.task_board_settings.auto_execute {
        return iced::Task::none();
    }

    let now_ms = crate::app::time::now_ms();
    let delay_ms = app.task_board_settings.auto_promote_delay_seconds * 1000;
    let max_concurrent = app.task_board_settings.max_concurrent.max(1);
    let max_pending = max_concurrent.saturating_mul(2);
    let pending_count = current_pending_task_count(app);
    if pending_count >= max_pending {
        return schedule_auto_promote_tick_with_deadline(app);
    }
    let max_promote_per_tick = max_pending.saturating_sub(pending_count) as usize;
    let mut pool_tasks_to_promote: Vec<(u32, u32, String)> = app
        .task_board_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Pool && !t.archived && !t.deleted)
        .filter(|t| {
            if let Some(promote_delay) = t.auto_promote_delay_ms {
                t.created_at_ms + promote_delay <= now_ms
            } else {
                t.created_at_ms + delay_ms <= now_ms
            }
        })
        .map(|t| (t.priority, t.order, t.id.clone()))
        .collect();
    pool_tasks_to_promote.sort_by_key(|(priority, order, _)| (*priority, *order));
    let pool_tasks_to_promote: Vec<String> = pool_tasks_to_promote
        .into_iter()
        .take(max_promote_per_tick)
        .map(|(_, _, id)| id)
        .collect();

    if pool_tasks_to_promote.is_empty() {
        return schedule_auto_promote_tick_with_deadline(app);
    }

    let mut tasks = Vec::new();
    for task_id in pool_tasks_to_promote {
        tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::TaskStatusChanged {
            task_id,
            new_status: TaskStatus::Pending,
        })));
    }

    tasks.push(schedule_auto_promote_tick_with_deadline(app));

    if !app.task_board_executor_running {
        tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::StartExecution)));
    }

    iced::Task::batch(tasks)
}
TaskBoardMessage::ExecuteTask { task_id } => {
    if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
        if let Some(project_path) = &app.project_path {
            let path = project_path.clone();
            let assigned_worktree_path = match crate::app::task::assign_task_execution_worktree(&path, &task, None) {
                Ok(path) => path,
                Err(error) => {
                    if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                        task_in_list.add_log(format!("[WORKTREE] 执行前预分配失败: {}", error));
                    }
                    sync_viewing_logs(app, &task_id);
                    if let Some(task_clone) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
                        let persist_path = path.clone();
                        return iced::Task::perform(
                            async move { crate::app::task::update_task(&persist_path, &task_clone) },
                            |_| Message::None,
                        );
                    }
                    return iced::Task::none();
                }
            };
            let should_schedule_worktree_maintenance = !app.task_board_worktree_maintenance_in_flight;
            let worktree_maintenance = schedule_worktree_pool_maintenance(
                app,
                &path,
                app.task_board_executor.running_tasks.len().saturating_add(1),
            );
            app.task_board_executor.start_task(&task_id);
            app.task_board_executor.register_log_channel(task_id.clone());
            let log_sender = app.task_board_executor.get_log_sender(&task_id);

            if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                task_in_list.start_execution("手动触发执行".to_string());
                task_in_list.selected_worktree_path = assigned_worktree_path;
                if let Some(selected_path) = &task_in_list.selected_worktree_path {
                    task_in_list.add_log(format!("[WORKTREE] 执行前已分配工作区: {}", selected_path));
                }
                task_in_list.add_log(format!(
                    "执行参数: acp_agent={} model={} prompt_chars={}",
                    task_execution_backend_label(task_in_list),
                    task_in_list.model,
                    task_in_list.prompt.chars().count()
                ));
            }
            sync_viewing_logs(app, &task_id);
            let start_task_persist = app
                .task_board_tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .map(|task| {
                    let path = path.clone();
                    iced::Task::perform(
                        async move { crate::app::task::update_task(&path, &task) },
                        |_| Message::None,
                    )
                })
                .unwrap_or_else(iced::Task::none);

            let execute_task_model = app
                .task_board_tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .unwrap_or(task);

            let mut execute_tasks = Vec::new();
            if should_schedule_worktree_maintenance {
                execute_tasks.push(worktree_maintenance);
            }
            execute_tasks.push(start_task_persist);
            execute_tasks.push(iced::Task::perform(
                crate::app::task::execute_task_async(execute_task_model, path, log_sender),
                move |(tid, result)| {
                    Message::TaskBoard(TaskBoardMessage::TaskExecutionCompleted { task_id: tid, result })
                },
            ));
            execute_tasks.push(schedule_scheduler_tick_with_deadline(app));
            return iced::Task::batch(execute_tasks);
        } else if let Some(task_in_list) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
            task_in_list.mark_execution_failed("缺少项目路径".to_string());
            sync_viewing_logs(app, &task_id);
        }
    }
    iced::Task::none()
}
    )
}
#[cfg(test)]
#[path = "execution_tests.rs"]
mod execution_tests;
