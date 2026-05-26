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
    app.task_board_draft.acp_agent = executor.clone();
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
    if app.task_board_draft.subtasks.len() > 1 && index < app.task_board_draft.subtasks.len() {
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
                Ok(created_task) => Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task)),
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
        task.acp_agent = None;
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
                Ok(created_task) => Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task)),
                Err(e) => {
                    eprintln!("Failed to create task: {}", e);
                    Message::None
                }
            },
        );
    }
    iced::Task::none()
}
TaskBoardMessage::AddTaskFromInputWithOptions {
    content,
    priority,
    model,
    subtasks,
} => {
    let raw = content.trim();
    if raw.is_empty() {
        return iced::Task::none();
    }

    let parsed_priority = priority.trim().parse::<u32>().ok().filter(|p| *p > 0);
    let priority_value = parsed_priority.unwrap_or(app.task_board_settings.default_priority);
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
        task.acp_agent = runtime.task_mode_executor.clone();
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
                Ok(created_task) => Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task)),
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
        app.task_board_draft.priority = app.task_board_viewing_logs.as_ref().unwrap().priority.to_string();
        app.task_board_draft.model = app.task_board_viewing_logs.as_ref().unwrap().model.clone();
        app.task_board_draft.acp_agent = app.task_board_viewing_logs.as_ref().unwrap().acp_agent.clone();
        app.task_board_draft.prompt = app.task_board_viewing_logs.as_ref().unwrap().prompt.clone();
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
    app.task_board_draft.acp_agent = executor;
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
                async move { crate::app::task::update_task(&path, &task_clone).map_err(|e| e.to_string()) },
                move |result| Message::TaskBoard(TaskBoardMessage::EditingTaskSaved(result)),
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
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        return iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            move |_| Message::None,
        );
    }
    iced::Task::none()
}
TaskBoardMessage::RemoveSubTask {
    task_id,
    subtask_id,
} => {
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
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        return iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            move |_| Message::None,
        );
    }
    iced::Task::none()
}
TaskBoardMessage::MoveSubTaskUp {
    task_id,
    subtask_id,
} => {
    let mut should_sync_viewing_logs = false;
    let mut task_clone = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        let idx = task.subtasks.iter().position(|s| s.id == subtask_id);
        if let Some(i) = idx && i > 0 {
            task.subtasks.swap(i, i - 1);
            should_sync_viewing_logs = true;
            task.updated_at_ms = crate::app::time::now_ms();
            task_clone = Some(task.clone());
        }
    }
    if should_sync_viewing_logs {
        sync_viewing_logs(app, &task_id);
    }
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        return iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            move |_| Message::None,
        );
    }
    iced::Task::none()
}
TaskBoardMessage::MoveSubTaskDown {
    task_id,
    subtask_id,
} => {
    let mut should_sync_viewing_logs = false;
    let mut task_clone = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id) {
        let idx = task.subtasks.iter().position(|s| s.id == subtask_id);
        if let Some(i) = idx && i < task.subtasks.len() - 1 {
            task.subtasks.swap(i, i + 1);
            should_sync_viewing_logs = true;
            task.updated_at_ms = crate::app::time::now_ms();
            task_clone = Some(task.clone());
        }
    }
    if should_sync_viewing_logs {
        sync_viewing_logs(app, &task_id);
    }
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        return iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            move |_| Message::None,
        );
    }
    iced::Task::none()
}
TaskBoardMessage::ToggleSubTaskCompleted {
    task_id,
    subtask_id,
} => {
    let mut should_sync_viewing_logs = false;
    let mut task_clone = None;
    if let Some(task) = app.task_board_tasks.iter_mut().find(|t| t.id == task_id)
        && let Some(subtask) = task.subtasks.iter_mut().find(|s| s.id == subtask_id)
    {
        subtask.completed = !subtask.completed;
        should_sync_viewing_logs = true;
        task.updated_at_ms = crate::app::time::now_ms();
        task_clone = Some(task.clone());
    }
    if should_sync_viewing_logs {
        sync_viewing_logs(app, &task_id);
    }
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        return iced::Task::perform(
            async move { crate::app::task::update_task(&path, &task_clone) },
            move |_| Message::None,
        );
    }
    iced::Task::none()
}
TaskBoardMessage::UpdateSubTaskContent {
    task_id,
    subtask_id,
    content,
} => {
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
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
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
    if let Some(task_clone) = task_clone && let Some(project_path) = &app.project_path {
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
TaskBoardMessage::LogsViewerEditorWheelScrolled {
    delta,
    viewport_height,
} => {
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
        app.task_board_logs_editor.perform(iced::widget::text_editor::Action::Scroll {
            lines: whole_lines,
        });
    }

    iced::Task::none()
}
TaskBoardMessage::LogsViewerScrollbarChanged {
    top_line,
    viewport_height,
} => {
    close_logs_context_menu(app);
    app.task_board_logs_viewport_height = viewport_height.max(0.0);

    let max_scroll = task_logs_max_scroll_top_line(app);
    let target_top_line = top_line.round().clamp(0.0, max_scroll);
    let current_top_line = app.task_board_logs_scroll_top_line.round();
    let delta = (target_top_line - current_top_line) as i32;

    if delta != 0 {
        apply_task_logs_scroll_lines(app, delta);
        app.task_board_logs_editor.perform(iced::widget::text_editor::Action::Scroll {
            lines: delta,
        });
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
    let (_outcome, task) =
        selection_delete_task(&mut app.task_board_logs_editor, &app.task_board_logs_editor_id);
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
    )
}
#[cfg(test)]
#[path = "draft_tests.rs"]
mod draft_tests;
