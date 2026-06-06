//! 提供任务看板消息处理过程中复用的局部辅助逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

/// 执行 visible_task_ids_for_status 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn visible_task_ids_for_status(
    app: &crate::app::App,
    status: TaskStatus,
) -> Vec<String> {
    app.task_board_tasks
        .iter()
        .filter(|task| task.status == status && !task.deleted && !task.archived)
        .map(|task| task.id.clone())
        .collect()
}

/// 执行 selected_task_ids_for_status 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn selected_task_ids_for_status(
    app: &crate::app::App,
    status: TaskStatus,
) -> Vec<String> {
    visible_task_ids_for_status(app, status)
        .into_iter()
        .filter(|task_id| app.task_board_selected_tasks.contains(task_id))
        .collect()
}

/// 执行 prune_bulk_selection 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn prune_bulk_selection(app: &mut crate::app::App) {
    let valid_ids = app
        .task_board_tasks
        .iter()
        .filter(|task| !task.deleted && !task.archived)
        .map(|task| task.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    app.task_board_selected_tasks.retain(|task_id| valid_ids.contains(task_id.as_str()));
}

/// 执行 clear_selected_task_ids 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn clear_selected_task_ids(app: &mut crate::app::App, task_ids: &[String]) {
    for task_id in task_ids {
        app.task_board_selected_tasks.remove(task_id);
    }
}

/// 执行 clear_selection_for_status 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn clear_selection_for_status(app: &mut crate::app::App, status: TaskStatus) {
    let task_ids = visible_task_ids_for_status(app, status);
    clear_selected_task_ids(app, &task_ids);
}

/// 执行 selected_tasks_for_status 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn selected_tasks_for_status(app: &crate::app::App, status: TaskStatus) -> Vec<Task> {
    app.task_board_tasks
        .iter()
        .filter(|task| {
            task.status == status
                && !task.deleted
                && !task.archived
                && app.task_board_selected_tasks.contains(&task.id)
        })
        .cloned()
        .collect()
}

/// 执行 persist_updated_tasks 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn persist_updated_tasks(
    project_path: &str,
    tasks: &[Task],
    action_label: &str,
) -> Result<(), String> {
    for task in tasks {
        crate::app::task::update_task(project_path, task)
            .map_err(|e| format!("{}失败({}): {}", action_label, task.id, e))?;
    }
    Ok(())
}

/// 执行 normalized_bulk_model_input 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn normalized_bulk_model_input(value: &str) -> String {
    normalize_task_model_input(value)
}

/// 执行 reset_bulk_operation_inputs 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn reset_bulk_operation_inputs(app: &mut crate::app::App) {
    app.task_board_bulk_priority_input =
        parse_priority_or_default(&app.task_board_draft.priority, 999).to_string();
    app.task_board_bulk_model_input = normalized_bulk_model_input(&app.task_board_last_model);
    app.task_board_bulk_agent = app
        .task_board_draft
        .agent
        .clone()
        .unwrap_or_else(|| crate::app::task::TASK_AGENT_MAIN.to_string());
    app.task_board_bulk_acp_agent = app.task_board_last_acp_agent.clone();
}

/// 执行 deactivate_bulk_selection_mode 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn deactivate_bulk_selection_mode(app: &mut crate::app::App) {
    if let Some(active_status) = app.task_board_bulk_active_status.take() {
        clear_selection_for_status(app, active_status);
    }
}

/// 执行 batch_delete_tasks 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn batch_delete_tasks(project_path: &str, task_ids: &[String]) -> Result<(), String> {
    for task_id in task_ids {
        crate::app::task::soft_delete_task(project_path, task_id)
            .map_err(|e| format!("批量删除失败({}): {}", task_id, e))?;
    }
    Ok(())
}

/// 执行 batch_archive_tasks 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn batch_archive_tasks(project_path: &str, task_ids: &[String]) -> Result<(), String> {
    for task_id in task_ids {
        crate::app::task::archive_task(project_path, task_id)
            .map_err(|e| format!("批量归档失败({}): {}", task_id, e))?;
    }
    Ok(())
}

/// 执行 batch_move_tasks_to_status 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn batch_move_tasks_to_status(
    project_path: &str,
    task_ids: &[String],
    target_status: TaskStatus,
) -> Result<(), String> {
    for task_id in task_ids {
        crate::app::task::update_task_status(project_path, task_id, target_status)
            .map_err(|e| format!("批量移动失败({}): {}", task_id, e))?;
    }
    Ok(())
}
#[cfg(test)]
#[path = "bulk_tests.rs"]
mod bulk_tests;
