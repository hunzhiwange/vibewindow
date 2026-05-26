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
        let path = project_path.clone();
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
    app.task_board_next_refresh_at_ms = next_deadline_ms(task_board_refresh_interval_secs(app));
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
    app.task_board_tasks = tasks;
    sync_task_log_cache_for_loaded_tasks(app);
    app.task_board_loading = false;
    prune_bulk_selection(app);
    app.task_board_settings = sanitized_task_board_settings(app.task_board_settings.clone());
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
            async move { crate::app::task::update_task_status(&path, &task_id_clone, new_status) },
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
TaskBoardMessage::DragStarted {
    task_id,
    from_status,
} => {
    app.task_board_dragging = Some((task_id, from_status));
    app.task_board_drag_pending = None;
    iced::Task::none()
}
TaskBoardMessage::DragPending {
    task_id,
    from_status,
    press_position,
} => {
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
            app.task_board_draft.acp_agent = task.acp_agent.clone();
            app.task_board_draft.prompt = task.prompt.clone();
            app.task_board_draft.subtasks =
                task.subtasks.iter().map(|subtask| subtask.content.clone()).collect();
            app.task_board_prompt_editor =
                iced::widget::text_editor::Content::with_text(&task.prompt);
        }
    }
    iced::Task::none()
}
TaskBoardMessage::DropOnStatus {
    to_status,
    insert_index,
} => {
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
    let reordered_ids: Vec<String> = status_tasks.iter().map(|task| task.id.clone()).collect();

    if let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        let task_id_clone = task_id.clone();
        return iced::Task::perform(
            async move {
                let _ = crate::app::task::update_task_status(&path, &task_id_clone, to_status);
                let _ = crate::app::task::reorder_tasks_in_status(&path, to_status, reordered_ids);
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
    let Some(source_task) = app.task_board_tasks.iter().find(|t| t.id == task_id).cloned() else {
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
                Ok(created_task) => Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task)),
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
    )
}
#[cfg(test)]
#[path = "board_tests.rs"]
mod board_tests;
