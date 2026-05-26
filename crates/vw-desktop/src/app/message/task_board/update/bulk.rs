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
        let path = project_path.clone();
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
        task.acp_agent = executor.clone();
        task.add_log(format!(
            "批量设置 ACP 智能体: {} -> {}",
            previous_executor,
            task_acp_agent_label(task.acp_agent.as_deref())
        ));
    }

    let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
    clear_selected_task_ids(app, &task_ids);

    if let Some(project_path) = &app.project_path {
        let path = project_path.clone();
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
        let path = project_path.clone();
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
        let path = project_path.clone();
        return iced::Task::perform(
            async move { batch_delete_tasks(&path, &task_ids) },
            |result| Message::TaskBoard(TaskBoardMessage::BulkActionCompleted(result)),
        );
    }
    iced::Task::none()
}
TaskBoardMessage::BulkMoveTasksInStatus {
    from_status,
    to_status,
} => {
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
        let path = project_path.clone();
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
    )
}
#[cfg(test)]
#[path = "bulk_tests.rs"]
mod bulk_tests;
