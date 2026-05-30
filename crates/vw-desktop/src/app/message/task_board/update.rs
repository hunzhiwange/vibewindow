//! 处理任务看板状态更新分支，将 UI 消息转换为应用状态变更和异步任务。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use crate::app::Message;
use crate::app::task::SubTaskStatus;

use super::helpers::*;
use super::*;

fn task_pool_status_for_gateway(
    status: TaskStatus,
) -> vw_gateway_client::vw_api_types::task::TaskPoolStatus {
    use vw_gateway_client::vw_api_types::task::TaskPoolStatus;

    match status {
        TaskStatus::Pool => TaskPoolStatus::Pool,
        TaskStatus::Pending => TaskPoolStatus::Pending,
        TaskStatus::Planning => TaskPoolStatus::Planning,
        TaskStatus::Running => TaskPoolStatus::Running,
        TaskStatus::Failed => TaskPoolStatus::Failed,
        TaskStatus::Paused => TaskPoolStatus::Paused,
        TaskStatus::CodeComplete => TaskPoolStatus::CodeComplete,
        TaskStatus::CodeReview => TaskPoolStatus::CodeReview,
        TaskStatus::PrSubmitted => TaskPoolStatus::PrSubmitted,
        TaskStatus::Completed => TaskPoolStatus::Completed,
        TaskStatus::Archived => TaskPoolStatus::Archived,
    }
}

fn build_task_pool_schedule_request(
    app: &crate::app::App,
    now_ms: u64,
) -> vw_gateway_client::vw_api_types::task::TaskPoolScheduleRequest {
    use vw_gateway_client::vw_api_types::task::{
        TaskPoolScheduleRequest, TaskPoolScheduleSettingsDto, TaskPoolScheduleTaskDto,
    };

    TaskPoolScheduleRequest {
        now_ms,
        settings: TaskPoolScheduleSettingsDto {
            auto_execute: app.task_board_settings.auto_execute,
            auto_promote_pool_tasks: app.task_board_settings.auto_promote_pool_tasks,
            max_concurrent: app.task_board_settings.max_concurrent,
            auto_promote_delay_seconds: app.task_board_settings.auto_promote_delay_seconds,
        },
        tasks: app
            .task_board_tasks
            .iter()
            .map(|task| TaskPoolScheduleTaskDto {
                id: task.id.clone(),
                status: task_pool_status_for_gateway(task.status),
                priority: task.priority,
                order: task.order,
                created_at_ms: task.created_at_ms,
                auto_promote_delay_ms: task.auto_promote_delay_ms,
                deleted: task.deleted,
                archived: task.archived,
            })
            .collect(),
    }
}

fn merge_loaded_tasks_preserving_running_state(
    loaded_tasks: Vec<Task>,
    current_tasks: &[Task],
    running_task_ids: &[String],
) -> Vec<Task> {
    let mut merged = loaded_tasks
        .into_iter()
        .map(|loaded_task| {
            if running_task_ids.iter().any(|id| id == &loaded_task.id)
                && let Some(current_task) =
                    current_tasks.iter().find(|task| task.id == loaded_task.id)
            {
                return current_task.clone();
            }
            loaded_task
        })
        .collect::<Vec<_>>();

    for current_task in current_tasks {
        if running_task_ids.iter().any(|id| id == &current_task.id)
            && !merged.iter().any(|task| task.id == current_task.id)
        {
            merged.push(current_task.clone());
        }
    }

    merged
}

/// 执行 update 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub fn update(
    app: &mut crate::app::App,
    message: TaskBoardMessage,
) -> iced::Task<crate::app::Message> {
    match message {
        TaskBoardMessage::ToggleBoard => {
            let board_is_open = app.screen == crate::app::Screen::TaskBoard;

            if board_is_open {
                app.show_task_board = false;
                app.screen = crate::app::Screen::Project;
                app.task_board_worktree_snapshot_loading = false;
                return iced::Task::none();
            }

            app.show_task_board = true;
            app.screen = crate::app::Screen::TaskBoard;

            if let Some(project_path) = &app.project_path {
                let path: String = project_path.clone();
                return iced::Task::batch(vec![
                    maybe_schedule_worktree_snapshot_refresh(app, true),
                    iced::Task::perform(
                        async move { crate::app::task::load_all_tasks(&path) },
                        |tasks| Message::TaskBoard(TaskBoardMessage::TasksLoaded(tasks)),
                    ),
                ]);
            }

            iced::Task::none()
        }
        TaskBoardMessage::CloseBoard => {
            app.show_task_board = false;
            app.screen = crate::app::Screen::Project;
            app.task_board_worktree_snapshot_loading = false;
            app.task_board_bulk_active_status = None;
            app.task_board_selected_tasks.clear();
            iced::Task::none()
        }
        TaskBoardMessage::LoadTasks => {
            app.task_board_next_refresh_at_ms =
                next_deadline_ms(task_board_refresh_interval_secs(app));
            let refresh_snapshot = maybe_schedule_worktree_snapshot_refresh(app, false);
            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                return iced::Task::batch(vec![
                    refresh_snapshot,
                    iced::Task::perform(
                        async move { crate::app::task::load_all_tasks(&path) },
                        |tasks| Message::TaskBoard(TaskBoardMessage::TasksLoaded(tasks)),
                    ),
                ]);
            }
            refresh_snapshot
        }
        TaskBoardMessage::TasksLoaded(tasks) => {
            app.task_board_tasks = merge_loaded_tasks_preserving_running_state(
                tasks,
                &app.task_board_tasks,
                &app.task_board_executor.running_tasks,
            );
            sync_task_log_cache_for_loaded_tasks(app);
            app.task_board_loading = false;
            prune_bulk_selection(app);
            app.task_board_settings =
                sanitized_task_board_settings(app.task_board_settings.clone());
            if app.task_board_settings.auto_execute {
                return iced::Task::batch(build_auto_execute_bootstrap_tasks(app));
            }
            if app.task_board_executor_running {
                return iced::Task::done(Message::TaskBoard(TaskBoardMessage::ExecutionTick));
            }
            iced::Task::none()
        }
        TaskBoardMessage::TaskStatusChanged { task_id, new_status } => {
            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_id_clone = task_id.clone();
                return iced::Task::perform(
                    async move {
                        crate::app::task::update_task_status(&path, &task_id_clone, new_status)
                    },
                    move |result| match result {
                        Ok(Some(updated_task)) => {
                            Message::TaskBoard(TaskBoardMessage::TaskUpdated(updated_task))
                        }
                        _ => Message::None,
                    },
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::TaskUpdated(task) => {
            if let Some(existing) = app.task_board_tasks.iter_mut().find(|t| t.id == task.id) {
                *existing = task.clone();
            }
            prune_bulk_selection(app);
            iced::Task::none()
        }
        TaskBoardMessage::TaskDeleted(task_id) => {
            app.task_board_selected_tasks.remove(&task_id);
            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_id_clone = task_id.clone();
                return iced::Task::perform(
                    async move { crate::app::task::soft_delete_task(&path, &task_id_clone) },
                    move |_| Message::TaskBoard(TaskBoardMessage::LoadTasks),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::DragStarted { task_id, from_status } => {
            app.task_board_dragging = Some((task_id, from_status));
            app.task_board_drag_pending = None;
            iced::Task::none()
        }
        TaskBoardMessage::DragPending { task_id, from_status, press_position } => {
            app.task_board_drag_pending = Some((task_id, from_status, press_position));
            iced::Task::none()
        }
        TaskBoardMessage::DragEnded => {
            app.task_board_dragging = None;
            app.task_board_drag_pending = None;
            iced::Task::none()
        }
        TaskBoardMessage::CardReleased { task_id } => {
            if app.task_board_dragging.is_some() {
                app.task_board_dragging = None;
                app.task_board_drag_pending = None;
            } else if app.task_board_drag_pending.is_some() {
                app.task_board_drag_pending = None;
                if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
                    app.task_board_editing_task_id = Some(task_id);
                    set_viewing_logs(app, Some(task.clone()));
                    app.task_board_draft.priority = task.priority.to_string();
                    app.task_board_draft.model = task.model.clone();
                    set_draft_executor_selection(&mut app.task_board_draft, task.acp_agent.clone());
                    app.task_board_draft.prompt = task.prompt.clone();
                    app.task_board_draft.subtasks =
                        task.subtasks.iter().map(|subtask| subtask.content.clone()).collect();
                    app.task_board_prompt_editor =
                        iced::widget::text_editor::Content::with_text(&task.prompt);
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::DropOnStatus { to_status, insert_index } => {
            let Some((task_id, _from_status)) = app.task_board_dragging.take() else {
                return iced::Task::none();
            };
            app.task_board_drag_pending = None;

            let mut status_tasks: Vec<_> = app
                .task_board_tasks
                .iter()
                .filter(|t| t.status == to_status && t.id != task_id)
                .collect();
            status_tasks.sort_by(|a, b| {
                a.order.cmp(&b.order).then_with(|| a.created_at_ms.cmp(&b.created_at_ms))
            });
            let insert_at = insert_index.unwrap_or(status_tasks.len()).min(status_tasks.len());
            let Some(drag_task) = app.task_board_tasks.iter().find(|t| t.id == task_id) else {
                return iced::Task::none();
            };
            status_tasks.insert(insert_at, drag_task);
            let reordered_ids: Vec<String> =
                status_tasks.iter().map(|task| task.id.clone()).collect();

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_id_clone = task_id.clone();
                return iced::Task::perform(
                    async move {
                        let _ =
                            crate::app::task::update_task_status(&path, &task_id_clone, to_status);
                        let _ = crate::app::task::reorder_tasks_in_status(
                            &path,
                            to_status,
                            reordered_ids,
                        );
                    },
                    move |_| Message::TaskBoard(TaskBoardMessage::LoadTasks),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::SelectTask(task_id) => {
            app.task_board_selected_task = Some(task_id);
            iced::Task::none()
        }
        TaskBoardMessage::DeselectTask => {
            app.task_board_selected_task = None;
            iced::Task::none()
        }
        TaskBoardMessage::FilterByStatus(status) => {
            app.task_board_filter_status = status;
            iced::Task::none()
        }
        TaskBoardMessage::FilterByPriority(range) => {
            app.task_board_filter_priority = range;
            iced::Task::none()
        }
        TaskBoardMessage::SortByPriority(ascending) => {
            app.task_board_sort_by_priority = true;
            app.task_board_sort_ascending = ascending;
            iced::Task::none()
        }
        TaskBoardMessage::SortByDate(ascending) => {
            app.task_board_sort_by_priority = false;
            app.task_board_sort_ascending = ascending;
            iced::Task::none()
        }
        TaskBoardMessage::OpenTaskInSession { task_id } => {
            if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id) {
                let _prompt = if !task.prompt.is_empty() {
                    task.prompt.clone()
                } else {
                    format!("执行任务: {}\n\n{}", task.id, task.description)
                };

                app.show_task_board = false;
                return iced::Task::batch(vec![iced::Task::done(Message::TaskBoard(
                    TaskBoardMessage::TaskStatusChanged {
                        task_id: task_id.clone(),
                        new_status: TaskStatus::Pending,
                    },
                ))]);
            }
            iced::Task::none()
        }
        TaskBoardMessage::TaskArchived(task_id) => {
            app.task_board_selected_tasks.remove(&task_id);
            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_id_clone = task_id.clone();
                return iced::Task::perform(
                    async move { crate::app::task::archive_task(&path, &task_id_clone) },
                    move |_| Message::TaskBoard(TaskBoardMessage::LoadTasks),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ArchiveCompletedTasks => {
            app.task_board_context_menu = None;
            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::archive_completed_tasks(&path) },
                    move |_| Message::TaskBoard(TaskBoardMessage::LoadTasks),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::DuplicateTask(task_id) => {
            let Some(source_task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
            else {
                return iced::Task::none();
            };

            let mut copied_task = Task::new(source_task.priority);
            copied_task.assignee = source_task.assignee;
            copied_task.model = source_task.model;
            copied_task.acp_agent = source_task.acp_agent;
            copied_task.description = source_task.description;
            copied_task.prompt = source_task.prompt;
            copied_task.status = TaskStatus::Pool;
            copied_task.auto_promote_delay_ms = source_task.auto_promote_delay_ms;
            copied_task.subtasks = source_task
                .subtasks
                .iter()
                .map(|subtask| SubTask::new(subtask.content.clone()))
                .collect();

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::create_task(&path, copied_task) },
                    move |result| match result {
                        Ok(created_task) => {
                            Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task))
                        }
                        Err(e) => {
                            eprintln!("Failed to duplicate task: {}", e);
                            Message::None
                        }
                    },
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ContextMenuOpened { task_id, x, y } => {
            app.task_board_context_menu = Some((task_id, x, y));
            iced::Task::none()
        }
        TaskBoardMessage::ContextMenuClosed => {
            app.task_board_context_menu = None;
            app.task_board_worktree_manual_confirm_kind = None;
            iced::Task::none()
        }
        TaskBoardMessage::CreateTask => {
            app.task_board_create_modal_open = true;
            app.task_board_create_submit_success = false;
            reset_create_draft(app);
            iced::Task::none()
        }
        TaskBoardMessage::CreateTaskCancelled => {
            app.task_board_create_modal_open = false;
            app.task_board_create_submit_success = false;
            reset_create_draft(app);
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftPriority(v) => {
            app.task_board_draft.priority = v;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftAssignee(v) => {
            app.task_board_draft.assignee = v;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftModel(v) => {
            app.task_board_draft.model = v;
            app.task_board_last_model = app.task_board_draft.model.clone();
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftExecutor(executor) => {
            set_draft_executor_selection(&mut app.task_board_draft, executor.clone());
            app.task_board_last_acp_agent = executor;
            app.task_board_executor_popover = false;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftDescription(v) => {
            app.task_board_draft.description = v;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftPrompt(v) => {
            app.task_board_draft.prompt = v;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftSubtask { index, value } => {
            if let Some(subtask) = app.task_board_draft.subtasks.get_mut(index) {
                *subtask = value;
            }
            iced::Task::none()
        }
        TaskBoardMessage::AddDraftSubtask => {
            app.task_board_draft.subtasks.push(String::new());
            iced::Task::none()
        }
        TaskBoardMessage::RemoveDraftSubtask(index) => {
            if app.task_board_draft.subtasks.len() > 1
                && index < app.task_board_draft.subtasks.len()
            {
                app.task_board_draft.subtasks.remove(index);
            } else if let Some(subtask) = app.task_board_draft.subtasks.get_mut(index) {
                subtask.clear();
            }
            iced::Task::none()
        }
        TaskBoardMessage::MoveDraftSubtaskUp(index) => {
            if index > 0 && index < app.task_board_draft.subtasks.len() {
                app.task_board_draft.subtasks.swap(index, index - 1);
            }
            iced::Task::none()
        }
        TaskBoardMessage::MoveDraftSubtaskDown(index) => {
            if index + 1 < app.task_board_draft.subtasks.len() {
                app.task_board_draft.subtasks.swap(index, index + 1);
            }
            iced::Task::none()
        }
        TaskBoardMessage::UpdateDraftAutoPromoteDelay(v) => {
            app.task_board_draft.auto_promote_delay_seconds = v;
            iced::Task::none()
        }
        TaskBoardMessage::CreateTaskSubmitted => {
            if app.task_board_draft.prompt.trim().is_empty() {
                return iced::Task::none();
            }

            let priority = app.task_board_draft.priority.parse().unwrap_or(999);
            let auto_promote_delay_seconds: u64 =
                app.task_board_draft.auto_promote_delay_seconds.parse().unwrap_or(0);
            let auto_promote_delay_ms = auto_promote_delay_seconds.saturating_mul(1000);
            let model = normalize_task_model(&app.task_board_draft.model);

            let mut task = Task::new(priority);
            task.model = model;
            task.acp_agent = app.task_board_draft.acp_agent.clone();
            task.prompt = app.task_board_draft.prompt.clone();
            task.subtasks = app
                .task_board_draft
                .subtasks
                .iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .map(SubTask::new)
                .collect::<Vec<_>>();
            task.auto_promote_delay_ms =
                if auto_promote_delay_ms > 0 { Some(auto_promote_delay_ms) } else { None };

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_for_result = task.clone();
                return iced::Task::perform(
                    async move { crate::app::task::create_task(&path, task_for_result) },
                    move |result| match result {
                        Ok(created_task) => {
                            Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task))
                        }
                        Err(e) => {
                            eprintln!("Failed to create task: {}", e);
                            Message::None
                        }
                    },
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::AddTaskFromInput(content) => {
            let raw = content.trim();
            if raw.is_empty() {
                return iced::Task::none();
            }

            let priority = app.task_board_settings.default_priority;
            let mut task = Task::new(priority);
            task.prompt = build_task_prompt_from_input(raw, Some(priority), &[]);

            {
                let runtime = app.current_session_runtime();
                if !runtime.auto_model && !runtime.model.trim().is_empty() {
                    task.model = runtime.model.trim().to_string();
                }
                set_task_executor_selection(&mut task, None);
            }

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_for_result = task.clone();
                let runtime = app.current_session_runtime_mut();
                runtime.input_editor = iced::widget::text_editor::Content::new();
                if app.active_session_id.is_none() {
                    let runtime = app.current_session_runtime();
                    app.input_editor = runtime.input_editor;
                }
                return iced::Task::perform(
                    async move { crate::app::task::create_task(&path, task_for_result) },
                    move |result| match result {
                        Ok(created_task) => {
                            Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task))
                        }
                        Err(e) => {
                            eprintln!("Failed to create task: {}", e);
                            Message::None
                        }
                    },
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::AddTaskFromInputWithOptions { content, priority, model, subtasks } => {
            let raw = content.trim();
            if raw.is_empty() {
                return iced::Task::none();
            }

            let parsed_priority = priority.trim().parse::<u32>().ok().filter(|p| *p > 0);
            let priority_value =
                parsed_priority.unwrap_or(app.task_board_settings.default_priority);
            let mut task = Task::new(priority_value);

            let parsed_subtasks = subtasks
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            task.prompt = build_task_prompt_from_input(raw, Some(priority_value), &parsed_subtasks);
            task.subtasks = parsed_subtasks.into_iter().map(SubTask::new).collect::<Vec<_>>();

            {
                let runtime = app.current_session_runtime();
                if runtime.task_mode_enabled {
                    task.model = normalize_task_model(&model);
                } else if !runtime.auto_model && !runtime.model.trim().is_empty() {
                    task.model = runtime.model.trim().to_string();
                }
                set_task_executor_selection(&mut task, runtime.task_mode_executor.clone());
            }

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                let task_for_result = task.clone();
                let runtime = app.current_session_runtime_mut();
                runtime.input_editor = iced::widget::text_editor::Content::new();
                runtime.task_mode_subtasks = vec![String::new(), String::new(), String::new()];
                runtime.task_mode_subtask_editors = runtime
                    .task_mode_subtasks
                    .iter()
                    .map(|value| iced::widget::text_editor::Content::with_text(value))
                    .collect();
                if app.active_session_id.is_none() {
                    let runtime = app.current_session_runtime();
                    app.input_editor = runtime.input_editor;
                }
                return iced::Task::perform(
                    async move { crate::app::task::create_task(&path, task_for_result) },
                    move |result| match result {
                        Ok(created_task) => {
                            Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task))
                        }
                        Err(e) => {
                            eprintln!("Failed to create task: {}", e);
                            Message::None
                        }
                    },
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::TaskCreated(task) => {
            app.task_board_tasks.push(task);
            prune_bulk_selection(app);
            app.task_board_create_submit_success = true;
            if app.task_board_clear_prompt_after_create {
                app.task_board_draft.prompt.clear();
                app.task_board_prompt_editor = iced::widget::text_editor::Content::new();
            }
            if app.task_board_close_after_create {
                app.task_board_create_modal_open = false;
                app.task_board_create_submit_success = false;
                reset_create_draft(app);
                return iced::Task::none();
            }
            crate::app::message::after(
                std::time::Duration::from_secs(2),
                Message::TaskBoard(TaskBoardMessage::ClearCreateSubmitSuccess),
            )
        }
        TaskBoardMessage::ViewTaskLogs(task_id) => {
            if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id) {
                app.task_board_logs_auto_scroll = true;
                set_viewing_logs(app, Some(task.clone()));
                flush_running_task_logs_throttled(app, true, web_time::Instant::now());
                return iced::Task::none();
            }
            iced::Task::none()
        }
        TaskBoardMessage::CloseTaskLogs => {
            set_viewing_logs(app, None);
            app.task_board_logs_auto_scroll = true;
            app.task_board_editing_task_id = None;
            app.task_board_edit_submit_success = false;
            iced::Task::none()
        }
        TaskBoardMessage::EditTask(task_id) => {
            if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
                app.task_board_logs_auto_scroll = true;
                app.task_board_editing_task_id = Some(task_id);
                app.task_board_edit_submit_success = false;
                set_viewing_logs(app, Some(task));
                app.task_board_draft.priority =
                    app.task_board_viewing_logs.as_ref().unwrap().priority.to_string();
                app.task_board_draft.model =
                    app.task_board_viewing_logs.as_ref().unwrap().model.clone();
                set_draft_executor_selection(
                    &mut app.task_board_draft,
                    app.task_board_viewing_logs.as_ref().unwrap().acp_agent.clone(),
                );
                app.task_board_draft.prompt =
                    app.task_board_viewing_logs.as_ref().unwrap().prompt.clone();
                app.task_board_draft.subtasks = app
                    .task_board_viewing_logs
                    .as_ref()
                    .unwrap()
                    .subtasks
                    .iter()
                    .map(|subtask| subtask.content.clone())
                    .collect();
                app.task_board_prompt_editor =
                    iced::widget::text_editor::Content::with_text(&app.task_board_draft.prompt);
            }
            iced::Task::none()
        }
        TaskBoardMessage::UpdateEditingTaskPriority(priority) => {
            app.task_board_draft.priority = priority;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateEditingTaskModel(model) => {
            app.task_board_draft.model = model;
            app.task_board_last_model = normalize_task_model(&app.task_board_draft.model);
            app.task_board_model_popover = false;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateEditingTaskModelInput(model) => {
            app.task_board_draft.model = model;
            app.task_board_last_model = normalize_task_model(&app.task_board_draft.model);
            iced::Task::none()
        }
        TaskBoardMessage::UpdateEditingTaskExecutor(executor) => {
            set_draft_executor_selection(&mut app.task_board_draft, executor);
            app.task_board_executor_popover = false;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateEditingTaskPrompt(prompt) => {
            app.task_board_draft.prompt = prompt;
            iced::Task::none()
        }
        TaskBoardMessage::SaveEditingTask => {
            if let Some(task_id) = &app.task_board_editing_task_id.clone()
                && let Some(task) = app.task_board_tasks.iter_mut().find(|t| &t.id == task_id)
            {
                task.priority = app.task_board_draft.priority.parse().unwrap_or(task.priority);
                task.model = normalize_task_model(&app.task_board_draft.model);
                task.acp_agent = app.task_board_draft.acp_agent.clone();
                task.prompt = app.task_board_draft.prompt.clone();
                task.updated_at_ms = crate::app::time::now_ms();
                if let Some(project_path) = &app.project_path {
                    let path = project_path.clone();
                    let task_clone = task.clone();
                    return iced::Task::perform(
                        async move {
                            crate::app::task::update_task(&path, &task_clone)
                                .map_err(|e| e.to_string())
                        },
                        move |result| {
                            Message::TaskBoard(TaskBoardMessage::EditingTaskSaved(result))
                        },
                    );
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::EditingTaskSaved(result) => {
            if result.is_ok() {
                if app.task_board_close_after_edit {
                    set_viewing_logs(app, None);
                    app.task_board_logs_auto_scroll = true;
                    app.task_board_editing_task_id = None;
                    app.task_board_edit_submit_success = false;
                    return iced::Task::none();
                }
                app.task_board_edit_submit_success = true;
                return crate::app::message::after(
                    std::time::Duration::from_secs(2),
                    Message::TaskBoard(TaskBoardMessage::ClearEditSubmitSuccess),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ClearCreateSubmitSuccess => {
            app.task_board_create_submit_success = false;
            iced::Task::none()
        }
        TaskBoardMessage::ClearEditSubmitSuccess => {
            app.task_board_edit_submit_success = false;
            iced::Task::none()
        }
        TaskBoardMessage::CloseTaskPanel => {
            app.task_board_create_modal_open = false;
            set_viewing_logs(app, None);
            app.task_board_logs_auto_scroll = true;
            app.task_board_editing_task_id = None;
            app.task_board_create_submit_success = false;
            app.task_board_edit_submit_success = false;
            iced::Task::none()
        }
        TaskBoardMessage::SetAutoPromoteDelay(seconds) => {
            app.task_board_settings.auto_promote_delay_seconds = seconds;
            save_settings(app);
            iced::Task::none()
        }
        TaskBoardMessage::AddSubTask { task_id, content } => {
            if content.trim().is_empty() {
                return iced::Task::none();
            }
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let subtask = SubTask::new(content);
                task.subtasks.push(subtask);
                should_sync_viewing_logs = true;
                task.updated_at_ms = crate::app::time::now_ms();
                task_clone = Some(task.clone());
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::RemoveSubTask { task_id, subtask_id } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                task.subtasks.retain(|s| s.id != subtask_id);
                should_sync_viewing_logs = true;
                task.updated_at_ms = crate::app::time::now_ms();
                task_clone = Some(task.clone());
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::MoveSubTaskUp { task_id, subtask_id } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let idx = task.subtasks.iter().position(|s| s.id == subtask_id);
                if let Some(i) = idx
                    && i > 0
                {
                    task.subtasks.swap(i, i - 1);
                    should_sync_viewing_logs = true;
                    task.updated_at_ms = crate::app::time::now_ms();
                    task_clone = Some(task.clone());
                }
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::MoveSubTaskDown { task_id, subtask_id } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let idx = task.subtasks.iter().position(|s| s.id == subtask_id);
                if let Some(i) = idx
                    && i < task.subtasks.len() - 1
                {
                    task.subtasks.swap(i, i + 1);
                    should_sync_viewing_logs = true;
                    task.updated_at_ms = crate::app::time::now_ms();
                    task_clone = Some(task.clone());
                }
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ToggleSubTaskCompleted { task_id, subtask_id } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                && let Some(subtask) = task.subtasks.iter_mut().find(|s| s.id == subtask_id)
            {
                subtask.completed = !subtask.completed;
                if subtask.completed {
                    subtask.status = SubTaskStatus::Completed;
                } else {
                    subtask.status = SubTaskStatus::Pending;
                    subtask.execution_started_at_ms = None;
                    subtask.last_execution_duration_ms = None;
                }
                should_sync_viewing_logs = true;
                task.updated_at_ms = crate::app::time::now_ms();
                task_clone = Some(task.clone());
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let _ = crate::app::task::write_task_plan_files(project_path, &task_clone);
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::UpdateSubTaskContent { task_id, subtask_id, content } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                && let Some(subtask) = task.subtasks.iter_mut().find(|s| s.id == subtask_id)
            {
                subtask.content = content;
                should_sync_viewing_logs = true;
                task.updated_at_ms = crate::app::time::now_ms();
                task_clone = Some(task.clone());
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::SubTaskUpdated { task_id, subtasks } => {
            let mut should_sync_viewing_logs = false;
            let mut task_clone = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                task.subtasks = subtasks;
                should_sync_viewing_logs = true;
                task.updated_at_ms = crate::app::time::now_ms();
                task_clone = Some(task.clone());
            }
            if should_sync_viewing_logs {
                sync_viewing_logs(app, &task_id);
            }
            if let Some(task_clone) = task_clone
                && let Some(project_path) = &app.project_path
            {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    move |_| Message::None,
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::UpdateNewSubtaskContent(content) => {
            app.task_board_new_subtask_content = content;
            iced::Task::none()
        }
        TaskBoardMessage::DescEditorAction(action) => {
            app.task_board_desc_editor.perform(action);
            app.task_board_draft.description = app.task_board_desc_editor.text().to_string();
            iced::Task::none()
        }
        TaskBoardMessage::PromptEditorAction(action) => {
            app.task_board_prompt_editor.perform(action);
            app.task_board_draft.prompt = app.task_board_prompt_editor.text().to_string();
            iced::Task::none()
        }
        TaskBoardMessage::LogsViewerEditorAction(action) => {
            close_logs_context_menu(app);
            if let iced::widget::text_editor::Action::Scroll { lines } = &action {
                apply_task_logs_scroll_lines(app, *lines);
            }
            app.task_board_logs_editor.perform(action);
            iced::Task::none()
        }
        TaskBoardMessage::LogsViewerEditorWheelScrolled { delta, viewport_height } => {
            close_logs_context_menu(app);
            app.task_board_logs_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                iced::mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                iced::mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.task_board_logs_scroll_remainder += delta_lines;

            let whole_lines = if app.task_board_logs_scroll_remainder >= 0.0 {
                app.task_board_logs_scroll_remainder.floor() as i32
            } else {
                app.task_board_logs_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.task_board_logs_scroll_remainder -= whole_lines as f32;
                apply_task_logs_scroll_lines(app, whole_lines);
                app.task_board_logs_editor
                    .perform(iced::widget::text_editor::Action::Scroll { lines: whole_lines });
            }

            iced::Task::none()
        }
        TaskBoardMessage::LogsViewerScrollbarChanged { top_line, viewport_height } => {
            close_logs_context_menu(app);
            app.task_board_logs_viewport_height = viewport_height.max(0.0);

            let max_scroll = task_logs_max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.task_board_logs_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_task_logs_scroll_lines(app, delta);
                app.task_board_logs_editor
                    .perform(iced::widget::text_editor::Action::Scroll { lines: delta });
            }

            iced::Task::none()
        }
        TaskBoardMessage::LogsViewerOpenContextMenu { x, y } => {
            app.task_board_logs_context_menu_open = true;
            app.task_board_logs_context_menu_pos = Some((x, y));
            iced::Task::none()
        }
        TaskBoardMessage::LogsViewerCloseContextMenu => {
            close_logs_context_menu(app);
            focus_editor_task(&app.task_board_logs_editor_id)
        }
        TaskBoardMessage::LogsViewerContextMenuCopy => {
            close_logs_context_menu(app);
            let (_outcome, task) =
                selection_copy_task(&app.task_board_logs_editor, &app.task_board_logs_editor_id);
            task
        }
        TaskBoardMessage::LogsViewerContextMenuCut => {
            close_logs_context_menu(app);
            let (_outcome, task) =
                selection_cut_task(&mut app.task_board_logs_editor, &app.task_board_logs_editor_id);
            task
        }
        TaskBoardMessage::LogsViewerContextMenuPaste => {
            close_logs_context_menu(app);
            paste_task(&app.task_board_logs_editor_id, |content| {
                Message::TaskBoard(TaskBoardMessage::LogsViewerEditorAction(paste_action(content)))
            })
        }
        TaskBoardMessage::LogsViewerContextMenuDelete => {
            close_logs_context_menu(app);
            let (_outcome, task) = selection_delete_task(
                &mut app.task_board_logs_editor,
                &app.task_board_logs_editor_id,
            );
            task
        }
        TaskBoardMessage::ToggleModelPopover => {
            let new = !app.task_board_model_popover;
            app.task_board_model_popover = new;
            if new {
                app.task_board_bulk_model_popover = false;
                app.model_popover_hover = None;
                if !app.model_settings.loading && app.model_settings.providers.is_empty() {
                    return iced::Task::done(Message::Settings(
                        crate::app::message::SettingsMessage::ModelsRefresh,
                    ));
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::ModelSelected(model) => {
            app.task_board_draft.model = model;
            app.task_board_last_model = app.task_board_draft.model.clone();
            app.task_board_model_popover = false;
            app.model_popover_hover = None;
            iced::Task::none()
        }
        TaskBoardMessage::CloseModelPopover => {
            app.task_board_model_popover = false;
            app.model_popover_hover = None;
            iced::Task::none()
        }
        TaskBoardMessage::ToggleExecutorPopover => {
            app.task_board_executor_popover = !app.task_board_executor_popover;
            if app.task_board_executor_popover {
                app.task_board_bulk_executor_popover = false;
            }
            iced::Task::none()
        }
        TaskBoardMessage::ToggleClearPromptAfterCreate(enabled) => {
            app.task_board_clear_prompt_after_create = enabled;
            crate::app::set_config_field(
                "task_board_clear_prompt_after_create",
                serde_json::Value::Bool(enabled),
            );
            iced::Task::none()
        }
        TaskBoardMessage::ToggleCloseAfterCreate(enabled) => {
            app.task_board_close_after_create = enabled;
            crate::app::set_config_field(
                "task_board_close_after_create",
                serde_json::Value::Bool(enabled),
            );
            iced::Task::none()
        }
        TaskBoardMessage::ToggleCloseAfterEdit(enabled) => {
            app.task_board_close_after_edit = enabled;
            crate::app::set_config_field(
                "task_board_close_after_edit",
                serde_json::Value::Bool(enabled),
            );
            iced::Task::none()
        }
        TaskBoardMessage::ToggleTaskSelection { task_id, selected } => {
            if selected {
                app.task_board_selected_tasks.insert(task_id);
            } else {
                app.task_board_selected_tasks.remove(&task_id);
            }
            iced::Task::none()
        }
        TaskBoardMessage::ToggleBulkSelectionMode(status) => {
            app.task_board_context_menu = None;
            if app.task_board_bulk_active_status == Some(status) {
                deactivate_bulk_selection_mode(app);
            } else {
                deactivate_bulk_selection_mode(app);
                app.task_board_bulk_active_status = Some(status);
                reset_bulk_operation_inputs(app);
            }
            iced::Task::none()
        }
        TaskBoardMessage::SelectAllTasksInStatus(status) => {
            for task_id in visible_task_ids_for_status(app, status) {
                app.task_board_selected_tasks.insert(task_id);
            }
            iced::Task::none()
        }
        TaskBoardMessage::InvertTaskSelectionInStatus(status) => {
            for task_id in visible_task_ids_for_status(app, status) {
                if !app.task_board_selected_tasks.remove(&task_id) {
                    app.task_board_selected_tasks.insert(task_id);
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::UpdateBulkPriorityInput(value) => {
            app.task_board_bulk_priority_input = value;
            iced::Task::none()
        }
        TaskBoardMessage::UpdateBulkModelInput(value) => {
            app.task_board_bulk_model_input = value;
            iced::Task::none()
        }
        TaskBoardMessage::ToggleBulkModelPopover => {
            let new = !app.task_board_bulk_model_popover;
            app.task_board_bulk_model_popover = new;
            if new {
                app.task_board_model_popover = false;
                app.model_popover_hover = None;
                if !app.model_settings.loading && app.model_settings.providers.is_empty() {
                    return iced::Task::done(Message::Settings(
                        crate::app::message::SettingsMessage::ModelsRefresh,
                    ));
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkModelSelected(model) => {
            app.task_board_bulk_model_input = normalize_task_model(&model);
            app.task_board_last_model = app.task_board_bulk_model_input.clone();
            app.task_board_bulk_model_popover = false;
            app.model_popover_hover = None;
            iced::Task::none()
        }
        TaskBoardMessage::CloseBulkModelPopover => {
            app.task_board_bulk_model_popover = false;
            app.model_popover_hover = None;
            iced::Task::none()
        }
        TaskBoardMessage::BulkSetPriorityInStatus(status) => {
            app.task_board_context_menu = None;
            let mut tasks = selected_tasks_for_status(app, status);
            if tasks.is_empty() {
                return iced::Task::none();
            }

            let priority = parse_priority_or_default(&app.task_board_bulk_priority_input, 999);
            app.task_board_bulk_priority_input = priority.to_string();

            for task in &mut tasks {
                let previous_priority = task.priority;
                task.priority = priority;
                task.add_log(format!("批量设置优先级: {} -> {}", previous_priority, priority));
            }

            let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
            clear_selected_task_ids(app, &task_ids);

            if let Some(project_path) = &app.project_path {
                let path = project_path.clone();
                return iced::Task::perform(
                    async move { persist_updated_tasks(&path, &tasks, "批量设置优先级") },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkSetModelInStatus(status) => {
            app.task_board_context_menu = None;
            let mut tasks = selected_tasks_for_status(app, status);
            if tasks.is_empty() {
                return iced::Task::none();
            }

            let model = normalized_bulk_model_input(&app.task_board_bulk_model_input);
            app.task_board_bulk_model_input = model.clone();
            app.task_board_last_model = model.clone();

            for task in &mut tasks {
                let previous_model = task.model.clone();
                task.model = model.clone();
                task.add_log(format!("批量设置模型: {} -> {}", previous_model, task.model));
            }

            let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
            clear_selected_task_ids(app, &task_ids);

            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                return iced::Task::perform(
                    async move { persist_updated_tasks(&path, &tasks, "批量设置模型") },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ToggleBulkExecutorPopover => {
            app.task_board_bulk_executor_popover = !app.task_board_bulk_executor_popover;
            if app.task_board_bulk_executor_popover {
                app.task_board_executor_popover = false;
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkExecutorSelected(executor) => {
            app.task_board_bulk_acp_agent = executor.clone();
            app.task_board_last_acp_agent = executor;
            app.task_board_bulk_executor_popover = false;
            iced::Task::none()
        }
        TaskBoardMessage::CloseBulkExecutorPopover => {
            app.task_board_bulk_executor_popover = false;
            iced::Task::none()
        }
        TaskBoardMessage::BulkSetExecutorInStatus { status, executor } => {
            app.task_board_context_menu = None;
            let mut tasks = selected_tasks_for_status(app, status);
            if tasks.is_empty() {
                return iced::Task::none();
            }

            app.task_board_bulk_acp_agent = executor.clone();
            app.task_board_last_acp_agent = executor.clone();

            for task in &mut tasks {
                let previous_executor = task_acp_agent_label(task.acp_agent.as_deref());
                set_task_executor_selection(task, executor.clone());
                task.add_log(format!(
                    "批量设置 ACP 智能体: {} -> {}",
                    previous_executor,
                    task_acp_agent_label(task.acp_agent.as_deref())
                ));
            }

            let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
            clear_selected_task_ids(app, &task_ids);

            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                return iced::Task::perform(
                    async move { persist_updated_tasks(&path, &tasks, "批量设置 ACP 智能体") },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkArchiveTasksInStatus(status) => {
            app.task_board_context_menu = None;
            let task_ids = selected_task_ids_for_status(app, status);
            if task_ids.is_empty() {
                return iced::Task::none();
            }
            clear_selected_task_ids(app, &task_ids);
            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                return iced::Task::perform(
                    async move { batch_archive_tasks(&path, &task_ids) },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkDeleteTasksInStatus(status) => {
            app.task_board_context_menu = None;
            let task_ids = selected_task_ids_for_status(app, status);
            if task_ids.is_empty() {
                return iced::Task::none();
            }
            clear_selected_task_ids(app, &task_ids);
            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                return iced::Task::perform(
                    async move { batch_delete_tasks(&path, &task_ids) },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkMoveTasksInStatus { from_status, to_status } => {
            app.task_board_context_menu = None;
            if from_status == to_status {
                return iced::Task::none();
            }
            let task_ids = selected_task_ids_for_status(app, from_status);
            if task_ids.is_empty() {
                return iced::Task::none();
            }
            clear_selected_task_ids(app, &task_ids);
            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                return iced::Task::perform(
                    async move { batch_move_tasks_to_status(&path, &task_ids, to_status) },
                    |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::BulkActionCompleted(result) => {
            if let Err(error) = result {
                app.push_notification(error);
            }
            iced::Task::done(Message::TaskBoard(TaskBoardMessage::LoadTasks))
        }
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
            app.task_board_next_refresh_at_ms =
                next_deadline_ms(task_board_refresh_interval_secs(app));
            save_settings(app);
            iced::Task::none()
        }
        TaskBoardMessage::SetSchedulerTickIntervalSeconds(seconds) => {
            app.task_board_settings.scheduler_tick_interval_seconds = seconds.clamp(1, 60);
            app.task_board_next_scheduler_tick_at_ms =
                next_deadline_ms(scheduler_tick_interval_secs(app));
            save_settings(app);
            iced::Task::none()
        }
        TaskBoardMessage::SetAutoPromoteTickIntervalSeconds(seconds) => {
            app.task_board_settings.auto_promote_tick_interval_seconds = seconds.clamp(1, 3600);
            app.task_board_next_auto_promote_tick_at_ms =
                next_deadline_ms(auto_promote_tick_interval_secs(app));
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
            app.task_board_next_refresh_at_ms =
                next_deadline_ms(task_board_refresh_interval_secs(app));
            app.task_board_next_scheduler_tick_at_ms =
                next_deadline_ms(scheduler_tick_interval_secs(app));
            app.task_board_next_auto_promote_tick_at_ms =
                next_deadline_ms(auto_promote_tick_interval_secs(app));
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
                    |result| {
                        Message::TaskBoard(TaskBoardMessage::CleanAllWorktreesCompleted(result))
                    },
                ),
                schedule_worktree_action_log_tick(),
            ])
        }
        TaskBoardMessage::CleanAllWorktreesCompleted(result) => {
            poll_worktree_action_logs(app);
            match result {
                Ok(count) => app.push_notification(format!(
                    "已完成 worktree 一键清理，共处理 {} 个槽位",
                    count
                )),
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
                    crate::app::task::delete_all_managed_worktrees_async_with_logs(
                        project_path,
                        Some(log_tx),
                    ),
                    |result| {
                        Message::TaskBoard(TaskBoardMessage::DeleteAllWorktreesCompleted(result))
                    },
                ),
                schedule_worktree_action_log_tick(),
            ])
        }
        TaskBoardMessage::DeleteAllWorktreesCompleted(result) => {
            poll_worktree_action_logs(app);
            match result {
                Ok(count) => {
                    app.push_notification(format!("已删除所有 worktree，共处理 {} 个槽位", count))
                }
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
                    crate::app::task::commit_merge_all_worktrees_async_with_logs(
                        project_path,
                        tasks,
                        Some(log_tx),
                    ),
                    |result| {
                        Message::TaskBoard(TaskBoardMessage::CommitMergeAllWorktreesCompleted(
                            result,
                        ))
                    },
                ),
                schedule_worktree_action_log_tick(),
            ])
        }
        TaskBoardMessage::CommitMergeAllWorktreesCompleted(result) => {
            poll_worktree_action_logs(app);
            match result {
                Ok(count) => app.push_notification(format!(
                    "已完成 worktree 一键合并，共合并 {} 个分支",
                    count
                )),
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
        TaskBoardMessage::ColumnScrollChanged { status, has_vertical_scrollbar } => {
            app.task_board_column_has_vertical_scrollbar.insert(status, has_vertical_scrollbar);
            iced::Task::none()
        }
        TaskBoardMessage::WorktreePoolMaintained { result } => {
            app.task_board_worktree_maintenance_in_flight = false;
            let refresh_snapshot = maybe_schedule_worktree_snapshot_refresh(app, true);
            if let Err(error) = result
                && let Some(project_path) = &app.project_path
            {
                let mut changed = false;
                for task in &mut app.task_board_tasks {
                    if task.status == TaskStatus::Pending
                        || task.status == TaskStatus::Planning
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
                && let Some(task_clone) =
                    app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
            {
                let path = project_path.to_string();
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
                && let Some(task_clone) =
                    app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
            {
                let path = project_path.to_string();
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
        TaskBoardMessage::StartExecution => {
            app.task_board_executor_running = true;
            app.task_board_last_log_flush_at_ms = 0;
            app.task_board_log_scan_cursor = 0;
            app.task_board_timeout_scan_cursor = 0;
            app.task_board_schedule_scan_cursor = 0;
            app.task_board_next_scheduler_tick_at_ms =
                next_deadline_ms(scheduler_tick_interval_secs(app));
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
            let (timeout_updated_tasks, timeout_recycle_tasks): (
                Vec<Task>,
                Vec<iced::Task<crate::app::Message>>,
            ) = apply_execution_timeouts(app, tick_started_at);
            let has_timeout_recycle_tasks = !timeout_recycle_tasks.is_empty();

            if !app.task_board_executor_running {
                if app.task_board_executor.running_tasks.is_empty() {
                    if let Some(project_path) = &app.project_path {
                        let mut persist_tasks =
                            build_persist_tasks(project_path, &timeout_updated_tasks);
                        persist_tasks.extend(timeout_recycle_tasks);
                        if !persist_tasks.is_empty() {
                            return iced::Task::batch(persist_tasks);
                        }
                    }
                    return iced::Task::none();
                }
                let continue_tick = schedule_scheduler_tick_with_deadline(app);
                if let Some(project_path) = &app.project_path {
                    let mut persist_tasks =
                        build_persist_tasks(project_path, &timeout_updated_tasks);
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

            if let Some(project_path) = app.project_path.as_deref() {
                let project_path = project_path.to_string();
                let should_schedule_worktree_maintenance =
                    !app.task_board_worktree_maintenance_in_flight;
                let worktree_maintenance = schedule_worktree_pool_maintenance(
                    app,
                    &project_path,
                    app.task_board_executor.running_tasks.len(),
                );
                let mut timeout_persist_tasks =
                    build_persist_tasks(&project_path, &timeout_updated_tasks);
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
                if let Some(task_id) =
                    pick_next_pending_task_for_execution(app, tick_started_at, &exclude)
                    && let Some(task) =
                        app.task_board_tasks.iter().find(|t| t.id == task_id).cloned()
                {
                    app.task_board_executor.start_task(&task_id);
                    app.task_board_executor.register_log_channel(task_id.clone());
                    let log_sender = app.task_board_executor.get_log_sender(&task_id);

                    if let Some(task_in_list) =
                        app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                    {
                        task_in_list.set_status(TaskStatus::Planning);
                        task_in_list.add_log("开始任务拆分".to_string());
                        task_in_list.add_log(format!(
                            "调度参数: running_count={} max_concurrent={} exclude={:?}",
                            running_count, app.task_board_settings.max_concurrent, exclude
                        ));
                        task_in_list.add_log(format!(
                            "拆分参数: acp_agent={} model={} prompt_chars={}",
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
                    let plan_task = iced::Task::perform(
                        crate::app::task::execute_task_plan_async(
                            execute_task_model,
                            path_clone,
                            log_sender,
                        ),
                        move |(tid, result)| {
                            Message::TaskBoard(TaskBoardMessage::TaskPlanningCompleted {
                                task_id: tid,
                                result,
                            })
                        },
                    );

                    let continue_tick = schedule_scheduler_tick_with_deadline(app);

                    timeout_persist_tasks.push(start_task_persist);
                    timeout_persist_tasks.push(plan_task);
                    timeout_persist_tasks.push(continue_tick);
                    return iced::Task::batch(timeout_persist_tasks);
                }
                let task_id: String = match pick_next_pr_submitted_task_for_merge(app, &exclude) {
                    Some(task_id) => task_id,
                    None => {
                        let continue_tick = schedule_scheduler_tick_with_deadline(app);
                        if timeout_persist_tasks.is_empty() {
                            return continue_tick;
                        }
                        timeout_persist_tasks.push(continue_tick);
                        return iced::Task::batch(timeout_persist_tasks);
                    }
                };
                if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
                    if !crate::app::task::can_dispatch_merge_task(&project_path, &task) {
                        let continue_tick = schedule_scheduler_tick_with_deadline(app);
                        if timeout_persist_tasks.is_empty() {
                            return continue_tick;
                        }
                        timeout_persist_tasks.push(continue_tick);
                        return iced::Task::batch(timeout_persist_tasks);
                    }
                    let should_schedule_worktree_maintenance =
                        !app.task_board_worktree_maintenance_in_flight;
                    let worktree_maintenance = schedule_worktree_pool_maintenance(
                        app,
                        &project_path,
                        app.task_board_executor.running_tasks.len().saturating_add(1),
                    );
                    app.task_board_executor.start_task(&task_id);
                    app.task_board_executor.register_log_channel(task_id.clone());
                    let merge_sender = app.task_board_executor.get_log_sender(&task_id);
                    if let Some(task_in_list) =
                        app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                    {
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
                        crate::app::task::execute_task_merge_async(
                            task,
                            path.clone(),
                            merge_sender,
                        ),
                        move |(tid, result)| {
                            Message::TaskBoard(TaskBoardMessage::TaskMergeCompleted {
                                task_id: tid,
                                result,
                            })
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
            let task_id: String = match pick_next_code_review_task(app, &exclude) {
                Some(task_id) => task_id,
                None => return iced::Task::none(),
            };
            if let (Some(task), Some(project_path)) = (
                app.task_board_tasks.iter().find(|t| t.id == task_id).cloned(),
                app.project_path.as_deref().map(str::to_owned),
            ) {
                match build_code_review_prompt(&task, &project_path) {
                    Ok(review_prompt) => {
                        let should_schedule_worktree_maintenance =
                            !app.task_board_worktree_maintenance_in_flight;
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
                        if let Some(task_in_list) =
                            app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                        {
                            task_in_list.selected_worktree_path =
                                crate::app::task::current_task_worktree_path(
                                    &project_path,
                                    &task_id,
                                );
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
                            crate::app::task::execute_task_review_async(
                                review_task,
                                project_path,
                                review_sender,
                            ),
                            move |(tid, result)| {
                                Message::TaskBoard(TaskBoardMessage::TaskCodeReviewCompleted {
                                    task_id: tid,
                                    result,
                                })
                            },
                        ));
                        review_tasks.push(schedule_auto_review_tick_with_deadline(app));
                        return iced::Task::batch(review_tasks);
                    }
                    Err(error) => {
                        if let Some(task_in_list) =
                            app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                        {
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

            let request = build_task_pool_schedule_request(app, crate::app::time::now_ms());
            iced::Task::perform(
                async move {
                    let client = crate::app::config::gateway_client()?;
                    client.task_pool_schedule(&request).await
                },
                |result| Message::TaskBoard(TaskBoardMessage::PoolTasksScheduled(result)),
            )
        }
        TaskBoardMessage::PoolTasksScheduled(result) => {
            let response = match result {
                Ok(response) => response,
                Err(error) => {
                    tracing::warn!(error = %error, "gateway task pool schedule failed");
                    return schedule_auto_promote_tick_with_deadline(app);
                }
            };
            if response.promote_task_ids.is_empty() {
                return schedule_auto_promote_tick_with_deadline(app);
            }

            let mut tasks = Vec::new();
            for task_id in response.promote_task_ids {
                tasks.push(iced::Task::done(Message::TaskBoard(
                    TaskBoardMessage::TaskStatusChanged {
                        task_id,
                        new_status: TaskStatus::Pending,
                    },
                )));
            }

            tasks.push(schedule_auto_promote_tick_with_deadline(app));

            if !app.task_board_executor_running {
                tasks.push(iced::Task::done(Message::TaskBoard(TaskBoardMessage::StartExecution)));
            }

            iced::Task::batch(tasks)
        }
        TaskBoardMessage::ExecuteTask { task_id } => {
            if let Some(task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() {
                if let Some(project_path) = app.project_path.as_deref() {
                    let path = project_path.to_string();
                    app.task_board_executor.start_task(&task_id);
                    app.task_board_executor.register_log_channel(task_id.clone());
                    let log_sender = app.task_board_executor.get_log_sender(&task_id);

                    if let Some(task_in_list) =
                        app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                    {
                        task_in_list.set_status(TaskStatus::Planning);
                        task_in_list.add_log("手动触发任务拆分".to_string());
                        task_in_list.add_log(format!(
                            "拆分参数: acp_agent={} model={} prompt_chars={}",
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
                    execute_tasks.push(start_task_persist);
                    execute_tasks.push(iced::Task::perform(
                        crate::app::task::execute_task_plan_async(
                            execute_task_model,
                            path,
                            log_sender,
                        ),
                        move |(tid, result)| {
                            Message::TaskBoard(TaskBoardMessage::TaskPlanningCompleted {
                                task_id: tid,
                                result,
                            })
                        },
                    ));
                    execute_tasks.push(schedule_scheduler_tick_with_deadline(app));
                    return iced::Task::batch(execute_tasks);
                } else if let Some(task_in_list) =
                    app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
                {
                    task_in_list.mark_execution_failed("缺少项目路径".to_string());
                    sync_viewing_logs(app, &task_id);
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::TaskPlanningCompleted { task_id, result } => {
            let final_logs = app.task_board_executor.poll_task_logs_all(&task_id);
            app.task_board_executor.finish_task(&task_id);
            let Some(project_path) = app.project_path.clone() else {
                return iced::Task::none();
            };
            let mut task_to_persist: Option<Task> = None;
            let mut execute_task_to_start: Option<Task> = None;
            let mut plan_failed = false;

            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                for log in final_logs {
                    let _ = append_task_log_stream(task, &log);
                }
                if task.status != TaskStatus::Planning {
                    task.add_log("任务状态已变化，忽略本次拆分结果".to_string());
                } else {
                    match result {
                        Ok(outcome) => {
                            task.subtasks.clear();
                            for (offset, plan_subtask) in outcome.subtasks.into_iter().enumerate() {
                                let mut subtask = SubTask::new(plan_subtask.title);
                                subtask.order = offset as u32;
                                subtask.boundary = plan_subtask.boundary;
                                subtask.acceptance_criteria = plan_subtask.acceptance_criteria;
                                subtask.target_files = plan_subtask.target_files;
                                task.subtasks.push(subtask);
                            }
                            task.add_log(format!("任务拆分完成: {} 个子任务", task.subtasks.len()));
                            if !outcome.raw_output.trim().is_empty() {
                                task.add_log(format!(
                                    "拆分结果: {}",
                                    truncate_for_ui(
                                        &outcome.raw_output,
                                        TASK_LOG_UI_MAX_DETAIL_CHARS
                                    )
                                ));
                            }
                            let _ = crate::app::task::write_task_plan_files(&project_path, task);

                            match crate::app::task::assign_task_execution_worktree(
                                &project_path,
                                task,
                                None,
                            ) {
                                Ok(assigned_worktree_path) => {
                                    task.start_execution("开始执行任务".to_string());
                                    task.selected_worktree_path = assigned_worktree_path;
                                    if let Some(selected_path) = &task.selected_worktree_path {
                                        task.add_log(format!(
                                            "[WORKTREE] 执行前已分配工作区: {}",
                                            selected_path
                                        ));
                                    }
                                    task.add_log("子任务将按顺序串行执行".to_string());
                                    let _ = crate::app::task::write_task_plan_files(
                                        &project_path,
                                        task,
                                    );
                                    execute_task_to_start = Some(task.clone());
                                    task_to_persist = Some(task.clone());
                                }
                                Err(error) => {
                                    task.add_log(format!("[WORKTREE] 执行前预分配失败: {}", error));
                                    task.mark_execution_failed(error);
                                    plan_failed = true;
                                    task_to_persist = Some(task.clone());
                                }
                            }
                        }
                        Err(error) => {
                            task.mark_execution_failed(format!("任务拆分失败: {}", error));
                            plan_failed = true;
                            task_to_persist = Some(task.clone());
                        }
                    }
                }
            }

            sync_viewing_logs(app, &task_id);
            let persist_task = task_to_persist
                .clone()
                .map(|task| {
                    let path = project_path.clone();
                    iced::Task::perform(
                        async move { crate::app::task::update_task(&path, &task) },
                        |_| Message::None,
                    )
                })
                .unwrap_or_else(iced::Task::none);
            if plan_failed {
                return iced::Task::batch(vec![
                    persist_task,
                    schedule_scheduler_tick_with_deadline(app),
                ]);
            }
            if let Some(execute_task_model) = execute_task_to_start {
                app.task_board_executor.start_task(&task_id);
                app.task_board_executor.register_log_channel(task_id.clone());
                let log_sender = app.task_board_executor.get_log_sender(&task_id);
                let path_clone = project_path.clone();
                let execute_task = iced::Task::perform(
                    crate::app::task::execute_task_async(
                        execute_task_model,
                        path_clone,
                        log_sender,
                    ),
                    move |(tid, result)| {
                        Message::TaskBoard(TaskBoardMessage::TaskExecutionCompleted {
                            task_id: tid,
                            result,
                        })
                    },
                );
                return iced::Task::batch(vec![
                    persist_task,
                    execute_task,
                    schedule_scheduler_tick_with_deadline(app),
                ]);
            }
            iced::Task::batch(vec![persist_task, schedule_scheduler_tick_with_deadline(app)])
        }
        TaskBoardMessage::TaskExecutionCompleted { task_id, result } => {
            let final_logs = app.task_board_executor.poll_task_logs_all(&task_id);
            app.task_board_executor.finish_task(&task_id);
            let mut task_to_persist: Option<Task> = None;
            let result_snapshot = result.clone();
            let project_path_snapshot = app.project_path.clone();
            let mut should_recycle_worktree = false;
            let mut recycle_reason: Option<String> = None;
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let mut plan_changed = false;
                for log in final_logs {
                    plan_changed |= append_task_log_stream(task, &log);
                }
                if plan_changed && let Some(project_path) = project_path_snapshot.as_deref() {
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
                }
                if task.status != TaskStatus::Running {
                    task.add_log("任务状态已变化，忽略本次执行结果".to_string());
                } else {
                    match result {
                        Ok(output) => {
                            let (body, git_summary, source_branch, target_branch, worktree_path): (
                                String,
                                Option<String>,
                                Option<String>,
                                Option<String>,
                                Option<String>,
                            ) = split_output_and_git_metadata(&output);
                            let mut message = if body.is_empty() {
                                "执行完成".to_string()
                            } else {
                                format!(
                                    "执行完成: {}",
                                    truncate_for_ui(&body, TASK_LOG_UI_MAX_DETAIL_CHARS)
                                )
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
                                            task.mark_paused(format!(
                                                "生成审核提示失败: {}",
                                                error
                                            ));
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
                                match validate_ready_for_merge(
                                    task,
                                    project_path_snapshot.as_deref(),
                                ) {
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
                    match crate::app::task::write_task_execution_result_log(
                        project_path,
                        task,
                        &result_snapshot,
                    ) {
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
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
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
                if let (Some(project_path), Some(task_clone)) =
                    (app.project_path.as_deref(), task_to_persist.clone())
                {
                    let path = project_path.to_string();
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
                return iced::Task::batch(vec![
                    worktree_task,
                    schedule_scheduler_tick_with_deadline(app),
                ]);
            }
            sync_viewing_logs(app, &task_id);
            if let (Some(project_path), Some(task_clone)) =
                (app.project_path.as_deref(), task_to_persist.clone())
            {
                let path = project_path.to_string();
                let persist_task = iced::Task::perform(
                    async move { crate::app::task::update_task(&path, &task_clone) },
                    |_| Message::None,
                );
                return iced::Task::batch(vec![
                    persist_task,
                    schedule_scheduler_tick_with_deadline(app),
                ]);
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
            let project_path_snapshot = app.project_path.clone();
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let mut plan_changed = false;
                for log in final_logs {
                    plan_changed |= append_task_log_stream(task, &log);
                }
                if plan_changed && let Some(project_path) = project_path_snapshot.as_deref() {
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
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
                                    match validate_ready_for_merge(
                                        task,
                                        app.project_path.as_deref(),
                                    ) {
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
                    let review_context_full =
                        build_code_review_prompt_context(task, project_path, None).ok();
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
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
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
                if let (Some(project_path), Some(task_clone)) =
                    (app.project_path.as_deref(), task_to_persist.clone())
                {
                    let path = project_path.to_string();
                    let persist_task = iced::Task::perform(
                        async move { crate::app::task::update_task(&path, &task_clone) },
                        |_| Message::None,
                    );
                    return iced::Task::batch(vec![persist_task, worktree_task]);
                }
                return worktree_task;
            }
            sync_viewing_logs(app, &task_id);
            if let (Some(project_path), Some(task_clone)) =
                (app.project_path.as_deref(), task_to_persist.clone())
            {
                let path = project_path.to_string();
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
            let project_path_snapshot = app.project_path.clone();
            if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
                let mut plan_changed = false;
                for log in final_logs {
                    plan_changed |= append_task_log_stream(task, &log);
                }
                if plan_changed && let Some(project_path) = project_path_snapshot.as_deref() {
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
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
                                task.add_log(
                                    "合并结果晚于超时暂停返回，按成功结果完成任务".to_string(),
                                );
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
                if let Some(project_path) = &app.project_path {
                    let _ = crate::app::task::write_task_plan_files(project_path, task);
                }
                task_to_persist = Some(task.clone());
            }
            sync_viewing_logs(app, &task_id);
            if let (Some(project_path), Some(task_clone)) =
                (app.project_path.as_deref(), task_to_persist)
            {
                let path = project_path.to_string();
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
        TaskBoardMessage::ToggleImportMode(enabled) => {
            app.task_board_is_import_mode = enabled;
            iced::Task::none()
        }
        TaskBoardMessage::SetImportPromptFormat(format) => {
            app.task_board_import_prompt_format = format;
            iced::Task::none()
        }
        TaskBoardMessage::ToggleImportPromptCollapsed => {
            app.task_board_import_prompt_collapsed = !app.task_board_import_prompt_collapsed;
            iced::Task::none()
        }
        TaskBoardMessage::CopyImportPromptTemplate => {
            let selected_priority = parse_priority_or_default(
                &app.task_board_draft.priority,
                app.task_board_settings.default_priority,
            );
            let template = import_prompt_template(
                app.task_board_import_prompt_format,
                selected_priority,
                &app.task_board_draft.model,
                app.task_board_draft.acp_agent.as_deref(),
            );
            app.push_notification("已复制导入提示词模板".to_string());
            iced::clipboard::write(template).map(|_: ()| Message::None)
        }
        TaskBoardMessage::ImportEditorAction(action) => {
            app.task_board_import_editor.perform(action);
            iced::Task::none()
        }
        TaskBoardMessage::ImportFilePick => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                iced::Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .add_filter("导入文件", &["json", "csv", "tsv"])
                            .pick_file()
                            .await
                            .map(|handle| handle.path().to_string_lossy().to_string())
                    },
                    |picked| Message::TaskBoard(TaskBoardMessage::ImportFilePicked(picked)),
                )
            }
            #[cfg(target_arch = "wasm32")]
            {
                app.push_notification("当前平台暂不支持文件选择，请粘贴内容导入".to_string());
                iced::Task::none()
            }
        }
        TaskBoardMessage::ImportFilePicked(picked) => {
            if let Some(path) = picked {
                return iced::Task::perform(
                    async move {
                        std::fs::read_to_string(&path)
                            .map_err(|e| format!("读取导入文件失败: {}", e))
                    },
                    |result| Message::TaskBoard(TaskBoardMessage::ImportFileLoaded(result)),
                );
            }
            iced::Task::none()
        }
        TaskBoardMessage::ImportFileLoaded(result) => {
            match result {
                Ok(content) => {
                    app.task_board_import_editor =
                        iced::widget::text_editor::Content::with_text(&content);
                    app.push_notification("已将文件内容填入导入表单".to_string());
                }
                Err(err) => {
                    app.push_notification(err);
                }
            }
            iced::Task::none()
        }
        TaskBoardMessage::InsertDemoData(template) => {
            if let Some(content) = import_demo_content(&template) {
                app.task_board_import_editor =
                    iced::widget::text_editor::Content::with_text(content);
            }
            iced::Task::none()
        }
        TaskBoardMessage::ClearImportEditor => {
            app.task_board_import_editor = iced::widget::text_editor::Content::new();
            iced::Task::none()
        }
        TaskBoardMessage::ImportTasksSubmitted => {
            let content = app.task_board_import_editor.text().to_string();
            if content.trim().is_empty() {
                return iced::Task::none();
            }

            let mut tasks_to_create = Vec::new();
            let default_priority = app.task_board_settings.default_priority;

            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = json_val.as_array() {
                    for item in arr {
                        let priority = item
                            .get("priority")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32)
                            .unwrap_or(default_priority);
                        let prompt =
                            item.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let mut task = Task::new(priority);
                        task.prompt = prompt;
                        if let Some(model) = item.get("model").and_then(|v| v.as_str()) {
                            task.model = model.to_string();
                        } else {
                            task.model = app.task_board_draft.model.clone();
                        }
                        set_task_executor_selection(
                            &mut task,
                            item.get("acp_agent")
                                .and_then(|value| value.as_str())
                                .and_then(normalize_task_acp_agent_input)
                                .or_else(|| app.task_board_draft.acp_agent.clone()),
                        );

                        tasks_to_create.push(task);
                    }
                }
            } else {
                let lines: Vec<&str> = content.lines().collect();
                if !lines.is_empty() {
                    let header = lines[0].to_lowercase();
                    let delimiter = if header.contains('\t') { '\t' } else { ',' };
                    let headers: Vec<&str> =
                        header.split(delimiter).map(|s: &str| s.trim().trim_matches('"')).collect();

                    let prompt_idx = headers.iter().position(|&h| h == "prompt" || h == "提示词");
                    let priority_idx =
                        headers.iter().position(|&h| h == "priority" || h == "优先级");
                    let model_idx = headers.iter().position(|&h| h == "model" || h == "模型");
                    let acp_agent_idx = headers
                        .iter()
                        .position(|&h| h == "acp_agent" || h == "智能体" || h == "acp智能体");

                    if prompt_idx.is_some() {
                        for line in lines.iter().skip(1).copied() {
                            let line: &str = line;
                            if line.trim().is_empty() {
                                continue;
                            }
                            let parts: Vec<&str> = line
                                .split(delimiter)
                                .map(|s: &str| s.trim().trim_matches('"'))
                                .collect();

                            let prompt = if let Some(idx) = prompt_idx {
                                parts.get(idx).unwrap_or(&"").to_string()
                            } else {
                                "".to_string()
                            };

                            let priority = if let Some(idx) = priority_idx {
                                parts
                                    .get(idx)
                                    .copied()
                                    .and_then(|s: &str| s.parse::<u32>().ok())
                                    .unwrap_or(default_priority)
                            } else {
                                default_priority
                            };

                            let mut task = Task::new(priority);
                            task.prompt = prompt;
                            task.model = model_idx
                                .and_then(|idx| parts.get(idx).copied())
                                .map(str::trim)
                                .filter(|value: &&str| !value.is_empty())
                                .map(str::to_string)
                                .unwrap_or_else(|| app.task_board_draft.model.clone());
                            set_task_executor_selection(
                                &mut task,
                                acp_agent_idx
                                    .and_then(|idx| parts.get(idx).copied())
                                    .and_then(normalize_task_acp_agent_input)
                                    .or_else(|| app.task_board_draft.acp_agent.clone()),
                            );
                            tasks_to_create.push(task);
                        }
                    }
                }
            }

            if tasks_to_create.is_empty() {
                return iced::Task::none();
            }

            if let Some(project_path) = &app.project_path {
                let path = project_path.to_string();
                let mut tasks = Vec::new();
                for task in tasks_to_create {
                    let p = path.clone();
                    tasks.push(iced::Task::perform(
                        async move { crate::app::task::create_task(&p, task) },
                        move |result| match result {
                            Ok(created_task) => {
                                Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task))
                            }
                            Err(e) => {
                                eprintln!("Failed to create task: {}", e);
                                Message::None
                            }
                        },
                    ));
                }
                return iced::Task::batch(tasks);
            }
            iced::Task::none()
        }
    }
}
#[cfg(test)]
#[path = "update_tests.rs"]
mod update_tests;
