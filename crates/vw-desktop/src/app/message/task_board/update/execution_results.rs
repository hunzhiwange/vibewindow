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
TaskBoardMessage::TaskExecutionCompleted { task_id, result } => {
    let final_logs = app.task_board_executor.poll_task_logs_all(&task_id);
    app.task_board_executor.finish_task(&task_id);
    let mut task_to_persist: Option<Task> = None;
    let result_snapshot = result.clone();
    let project_path_snapshot = app.project_path.clone();
    let mut should_recycle_worktree = false;
    let mut recycle_reason: Option<String> = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        for log in final_logs {
            append_task_log_stream(task, &log);
        }
        if task.status != TaskStatus::Running {
            task.add_log("任务状态已变化，忽略本次执行结果".to_string());
        } else {
            match result {
                Ok(output) => {
                    let (body, git_summary, source_branch, target_branch, worktree_path) =
                        split_output_and_git_metadata(&output);
                    let mut message = if body.is_empty() {
                        "执行完成".to_string()
                    } else {
                        format!("执行完成: {}", truncate_for_ui(&body, TASK_LOG_UI_MAX_DETAIL_CHARS))
                    };
                    if let Some(summary) = git_summary {
                        message.push_str(" | ");
                        message.push_str(&summary);
                    }
                    if source_branch.is_some() {
                        task.merge_source_branch = source_branch;
                    }
                    if target_branch.is_some() {
                        task.merge_target_branch = target_branch;
                    }
                    if worktree_path.is_some() {
                        task.selected_worktree_path = worktree_path;
                    }
                    task.add_log(message);
                    task.mark_execution_succeeded();
                    task.set_status(TaskStatus::CodeComplete);
                    if app.task_board_settings.code_review_enabled {
                        if let Some(path) = &project_path_snapshot {
                            match build_code_review_prompt(task, path) {
                                Ok(_) => {
                                    task.add_log("进入代码审核阶段".to_string());
                                    task.set_status(TaskStatus::CodeReview);
                                    task.add_log(format!(
                                        "等待自动审核调度，{} 秒后执行",
                                        TASK_AUTO_CODE_REVIEW_TICK_INTERVAL_SECS
                                    ));
                                }
                                Err(error) => {
                                    task.mark_paused(format!("生成审核提示失败: {}", error));
                                    should_recycle_worktree = true;
                                    recycle_reason = task.pause_reason.clone();
                                }
                            }
                        } else {
                            task.mark_paused("缺少项目路径，无法进入代码审核".to_string());
                            should_recycle_worktree = true;
                            recycle_reason = task.pause_reason.clone();
                        }
                    } else {
                        task.add_log("进入代码审核阶段".to_string());
                        task.set_status(TaskStatus::CodeReview);
                        task.add_log("自动审核已跳过".to_string());
                        task.add_log(format!(
                            "合并前校验: source={} target={} worktree={} project_path={}",
                            task.merge_source_branch.as_deref().unwrap_or("none"),
                            task.merge_target_branch.as_deref().unwrap_or("none"),
                            task.selected_worktree_path.as_deref().unwrap_or("none"),
                            project_path_snapshot.as_deref().unwrap_or("none")
                        ));
                        match validate_ready_for_merge(task, project_path_snapshot.as_deref()) {
                            Ok(()) => {
                                task.add_log("代码审核通过，进入合并阶段".to_string());
                                task.set_status(TaskStatus::PrSubmitted);
                            }
                            Err(reason) => {
                                task.mark_paused(reason);
                                should_recycle_worktree = true;
                                recycle_reason = task.pause_reason.clone();
                            }
                        }
                    }
                }
                Err(error) => {
                    task.add_log(format!(
                        "执行失败: {}",
                        truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                    ));
                    task.mark_execution_failed(error);
                    should_recycle_worktree = true;
                    recycle_reason = task.last_error.clone();
                }
            }
        }
        if let Some(project_path) = &app.project_path {
            match crate::app::task::write_task_execution_result_log(project_path, task, &result_snapshot) {
                Ok(path) => {
                    let file_url = to_file_url(&path);
                    task.add_log(format!("执行结果文件:\n{}", file_url));
                    match crate::app::task::save_task_execution_result_artifact(
                        project_path,
                        task,
                        &result_snapshot,
                        Some(path.as_path()),
                    ) {
                        Ok(()) => {
                            task.add_log("执行结果已备份到 SQLite".to_string());
                        }
                        Err(error) => {
                            task.add_log(format!("执行结果 SQLite 备份失败: {}", error));
                        }
                    }
                }
                Err(error) => {
                    task.add_log(format!("执行结果落盘失败: {}", error));
                }
            }
        }
        task_to_persist = Some(task.clone());
    }
    if should_recycle_worktree {
        let worktree_task = if should_recycle_worktree_on_task_finish(app) {
            recycle_task_worktree_if_possible(app, &task_id, recycle_reason)
        } else {
            release_task_worktree_if_possible(app, &task_id)
        };
        sync_viewing_logs(app, &task_id);
        if let (Some(project_path), Some(task_clone)) = (&app.project_path, task_to_persist.clone()) {
            let path = project_path.clone();
            let persist_task = iced::Task::perform(
                async move { crate::app::task::update_task(&path, &task_clone) },
                |_| Message::None,
            );
            return iced::Task::batch(vec![
                persist_task,
                worktree_task,
                schedule_scheduler_tick_with_deadline(app),
            ]);
        }
        return iced::Task::batch(vec![worktree_task, schedule_scheduler_tick_with_deadline(app)]);
    }
    sync_viewing_logs(app, &task_id);
    if let (Some(project_path), Some(task_clone)) = (&app.project_path, task_to_persist.clone()) {
        let path = project_path.clone();
        let persist_task = iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            |_| Message::None,
        );
        return iced::Task::batch(vec![persist_task, schedule_scheduler_tick_with_deadline(app)]);
    }
    schedule_scheduler_tick_with_deadline(app)
}
TaskBoardMessage::TaskCodeReviewCompleted { task_id, result } => {
    let final_logs = app.task_board_executor.poll_task_logs_all(&task_id);
    app.task_board_executor.finish_task(&task_id);
    let mut task_to_persist: Option<Task> = None;
    let review_result_snapshot = result.clone();
    let mut should_recycle_worktree = false;
    let mut recycle_reason: Option<String> = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        for log in final_logs {
            append_task_log_stream(task, &log);
        }
        if task.status != TaskStatus::CodeReview {
            task.add_log("任务状态已变化，忽略本次审核结果".to_string());
        } else {
            match result {
                Ok(output) => {
                    let review_output = output.trim().to_string();
                    if !review_output.is_empty() {
                        task.add_log(format!(
                            "审核结果: {}",
                            truncate_for_ui(&review_output, TASK_LOG_UI_MAX_DETAIL_CHARS)
                        ));
                    } else {
                        task.add_log("审核结果: 无输出".to_string());
                    }
                    match parse_review_decision(&output) {
                        Ok(()) => {
                            task.add_log(format!(
                                "合并前校验: source={} target={} worktree={} project_path={}",
                                task.merge_source_branch.as_deref().unwrap_or("none"),
                                task.merge_target_branch.as_deref().unwrap_or("none"),
                                task.selected_worktree_path.as_deref().unwrap_or("none"),
                                app.project_path.as_deref().unwrap_or("none")
                            ));
                            match validate_ready_for_merge(task, app.project_path.as_deref()) {
                                Ok(()) => {
                                    task.add_log("代码审核通过，进入合并阶段".to_string());
                                    task.set_status(TaskStatus::PrSubmitted);
                                }
                                Err(reason) => {
                                    task.mark_paused(reason);
                                    should_recycle_worktree = true;
                                    recycle_reason = task.pause_reason.clone();
                                }
                            }
                        }
                        Err(reason) => {
                            task.mark_paused(reason);
                            should_recycle_worktree = true;
                            recycle_reason = task.pause_reason.clone();
                        }
                    }
                }
                Err(error) => {
                    task.mark_paused(format!(
                        "代码审核执行失败: {}",
                        truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                    ));
                    should_recycle_worktree = true;
                    recycle_reason = task.pause_reason.clone();
                }
            }
        }
        if let Some(project_path) = &app.project_path {
            let review_context_full = build_code_review_prompt_context(task, project_path, None).ok();
            let full_system_prompt = review_context_full
                .as_ref()
                .map(|ctx| format!("task_id={}\n{}", task.id, ctx.prompt));
            match crate::app::task::write_task_code_review_result_log(
                project_path,
                task,
                &review_result_snapshot,
                full_system_prompt.as_deref(),
            ) {
                Ok(path) => {
                    let file_url = to_file_url(&path);
                    task.add_log(format!("审核结果文件:\n{}", file_url));
                    match crate::app::task::save_task_code_review_result_artifact(
                        project_path,
                        task,
                        &review_result_snapshot,
                        full_system_prompt.as_deref(),
                        Some(path.as_path()),
                    ) {
                        Ok(()) => {
                            task.add_log("审核结果已备份到 SQLite".to_string());
                        }
                        Err(error) => {
                            task.add_log(format!("审核结果 SQLite 备份失败: {}", error));
                        }
                    }
                }
                Err(error) => {
                    task.add_log(format!("审核结果落盘失败: {}", error));
                }
            }
        }
        task_to_persist = Some(task.clone());
    }
    if should_recycle_worktree {
        let worktree_task = if should_recycle_worktree_on_task_finish(app) {
            recycle_task_worktree_if_possible(app, &task_id, recycle_reason)
        } else {
            release_task_worktree_if_possible(app, &task_id)
        };
        sync_viewing_logs(app, &task_id);
        if let (Some(project_path), Some(task_clone)) = (&app.project_path, task_to_persist.clone()) {
            let path = project_path.clone();
            let persist_task = iced::Task::perform(
                async move { crate::app::task::update_task(&path, &task_clone) },
                |_| Message::None,
            );
            return iced::Task::batch(vec![persist_task, worktree_task]);
        }
        return worktree_task;
    }
    sync_viewing_logs(app, &task_id);
    if let (Some(project_path), Some(task_clone)) = (&app.project_path, task_to_persist.clone()) {
        let path = project_path.clone();
        let persist_task = iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            |_| Message::None,
        );
        return persist_task;
    }
    iced::Task::none()
}
TaskBoardMessage::TaskMergeCompleted { task_id, result } => {
    let final_logs = app.task_board_executor.poll_task_logs_all(&task_id);
    app.task_board_executor.finish_task(&task_id);
    let recycle_reason = result.clone().err();
    let worktree_task = if should_recycle_worktree_on_task_finish(app) {
        recycle_task_worktree_if_possible(app, &task_id, recycle_reason)
    } else {
        release_task_worktree_if_possible(app, &task_id)
    };
    let mut task_to_persist: Option<Task> = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        for log in final_logs {
            append_task_log_stream(task, &log);
        }
        let late_success_after_timeout = task.status == TaskStatus::Paused
            && task.pause_reason.as_deref().is_some_and(|reason| {
                reason.contains("合并阶段") && reason.contains("自动暂停")
            })
            && result.is_ok();
        if task.status != TaskStatus::PrSubmitted && !late_success_after_timeout {
            task.add_log("任务状态已变化，忽略本次合并结果".to_string());
        } else {
            match result {
                Ok(output) => {
                    if late_success_after_timeout {
                        task.add_log("合并结果晚于超时暂停返回，按成功结果完成任务".to_string());
                    }
                    if output.trim().is_empty() {
                        task.add_log("合并完成".to_string());
                    } else {
                        task.add_log(format!(
                            "合并完成: {}",
                            truncate_for_ui(&output, TASK_LOG_UI_MAX_DETAIL_CHARS)
                        ));
                    }
                    task.mark_execution_succeeded();
                    task.set_status(TaskStatus::Completed);
                }
                Err(error) => {
                    task.mark_paused(format!(
                        "合并失败: {}",
                        truncate_for_ui(&error, TASK_LOG_UI_MAX_DETAIL_CHARS)
                    ));
                }
            }
        }
        task_to_persist = Some(task.clone());
    }
    sync_viewing_logs(app, &task_id);
    if let (Some(project_path), Some(task_clone)) = (&app.project_path, task_to_persist) {
        let path = project_path.clone();
        return iced::Task::batch(vec![
            iced::Task::perform(
                async move { crate::app::task::update_task(&path, &task_clone) },
                move |_| Message::None,
            ),
            worktree_task,
        ]);
    }
    worktree_task
}
    )
}
#[cfg(test)]
#[path = "execution_results_tests.rs"]
mod execution_results_tests;
