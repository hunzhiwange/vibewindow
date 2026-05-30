//! 任务存储层的 operations.rs 子模块。
//!
//! 该模块负责任务索引、持久化或产物写入中的一部分能力。实现保持文件系统与 SQLite 路径清晰分离，让上层任务流程只依赖稳定的存储函数。

use std::collections::HashMap;
use std::io;

use time::OffsetDateTime;

use crate::app::task::models::{Task, TaskStatus};

use super::paths::{ensure_task_dir, with_index_lock};
use super::persistence::{load_all_tasks, load_index, load_task, max_sequence_for_date, save_task};
#[cfg(not(target_arch = "wasm32"))]
use super::persistence::{
    load_index_from_sqlite, load_index_unlocked, load_task_from_sqlite, open_index_connection,
    save_index_with_tx, save_task_with_tx, sqlite_to_io_error,
};
#[cfg(target_arch = "wasm32")]
use super::persistence::{load_index_unlocked, save_index_unlocked};

/// 公开的 load_tasks_by_status 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn load_tasks_by_status(project_path: &str) -> HashMap<TaskStatus, Vec<Task>> {
    let index = load_index(project_path);
    let mut result = HashMap::new();

    for status in TaskStatus::all() {
        result.insert(status, Vec::new());
    }

    for task_id in index.tasks.keys() {
        if let Some(task) = load_task(project_path, task_id)
            && !task.deleted
            && let Some(list) = result.get_mut(&task.status)
        {
            list.push(task);
        }
    }

    for list in result.values_mut() {
        list.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.order.cmp(&b.order))
                .then_with(|| a.created_at_ms.cmp(&b.created_at_ms))
        });
    }

    result
}

/// 公开的 create_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_task(project_path: &str, mut task: Task) -> io::Result<Task> {
    ensure_task_dir(project_path)?;
    with_index_lock(project_path, || {
        let mut index = load_index_unlocked(project_path);

        let now_ms = crate::app::time::now_ms();
        let secs = (now_ms / 1000) as i64;
        let dt = OffsetDateTime::from_unix_timestamp(secs).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        let month: u8 = dt.month().into();
        let date = format!("{:04}{:02}{:02}", dt.year(), month, dt.day());
        let mut last_seq =
            if index.last_task_date.as_deref() == Some(&date) { index.last_task_seq } else { 0 };
        let existing_seq = max_sequence_for_date(&index, &date);
        if existing_seq > last_seq {
            last_seq = existing_seq;
        }
        let next_seq = last_seq.saturating_add(1);
        let task_id = format!("T{}.{:04}", date, next_seq);

        task.id = task_id.clone();
        index.last_task_date = Some(date);
        index.last_task_seq = next_seq;

        let status_key = task.status.to_string_key().to_string();
        let order_no = index.order_by_status.get(&status_key).map(|v| v.len() as u32).unwrap_or(0);
        task.order = order_no;

        index.tasks.insert(task_id.clone(), status_key.clone());
        index.order_by_status.entry(status_key).or_default().push(task_id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut conn = open_index_connection(project_path)?;
            let tx = conn.transaction().map_err(sqlite_to_io_error)?;
            save_task_with_tx(&tx, &task)?;
            save_index_with_tx(&tx, &index)?;
            tx.commit().map_err(sqlite_to_io_error)?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            save_task(project_path, &task)?;
            save_index_unlocked(project_path, &index)?;
        }

        Ok(task)
    })
}

/// 公开的 update_task_status 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn update_task_status(
    project_path: &str,
    task_id: &str,
    new_status: TaskStatus,
) -> io::Result<Option<Task>> {
    with_index_lock(project_path, || {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut conn = open_index_connection(project_path)?;
            let mut task = match load_task_from_sqlite(&conn, task_id)? {
                Some(task) => task,
                None => return Ok(None),
            };

            let old_status = task.status;
            if old_status == new_status {
                return Ok(Some(task));
            }

            let mut index = load_index_from_sqlite(&conn)?;
            let old_status_key = old_status.to_string_key().to_string();
            let new_status_key = new_status.to_string_key().to_string();

            if let Some(old_list) = index.order_by_status.get_mut(&old_status_key) {
                old_list.retain(|id| id != task_id);
            }

            let new_order =
                index.order_by_status.get(&new_status_key).map(|v| v.len() as u32).unwrap_or(0);
            index
                .order_by_status
                .entry(new_status_key.clone())
                .or_default()
                .push(task_id.to_string());
            index.tasks.insert(task_id.to_string(), new_status_key);

            task.order = new_order;
            task.set_status(new_status);

            let tx = conn.transaction().map_err(sqlite_to_io_error)?;
            save_task_with_tx(&tx, &task)?;
            save_index_with_tx(&tx, &index)?;
            tx.commit().map_err(sqlite_to_io_error)?;

            Ok(Some(task))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let mut task = match load_task(project_path, task_id) {
                Some(t) => t,
                None => return Ok(None),
            };

            let old_status = task.status;
            if old_status == new_status {
                return Ok(Some(task));
            }

            let mut index = load_index_unlocked(project_path);
            let old_status_key = old_status.to_string_key().to_string();
            let new_status_key = new_status.to_string_key().to_string();

            if let Some(old_list) = index.order_by_status.get_mut(&old_status_key) {
                old_list.retain(|id| id != task_id);
            }

            let new_order =
                index.order_by_status.get(&new_status_key).map(|v| v.len() as u32).unwrap_or(0);
            index
                .order_by_status
                .entry(new_status_key.clone())
                .or_default()
                .push(task_id.to_string());
            index.tasks.insert(task_id.to_string(), new_status_key);

            task.order = new_order;
            task.set_status(new_status);

            save_task(project_path, &task)?;
            save_index_unlocked(project_path, &index)?;

            Ok(Some(task))
        }
    })
}

/// 公开的 soft_delete_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn soft_delete_task(project_path: &str, task_id: &str) -> io::Result<Option<Task>> {
    let mut task = match load_task(project_path, task_id) {
        Some(t) => t,
        None => return Ok(None),
    };

    task.deleted = true;
    task.add_log("任务已删除".to_string());

    update_task(project_path, &task)?;

    Ok(Some(task))
}

/// 公开的 archive_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn archive_task(project_path: &str, task_id: &str) -> io::Result<Option<Task>> {
    let mut task = match load_task(project_path, task_id) {
        Some(t) => t,
        None => return Ok(None),
    };

    task.archived = true;
    task.status = TaskStatus::Archived;
    task.add_log("任务已归档".to_string());

    update_task(project_path, &task)?;

    Ok(Some(task))
}

/// 公开的 archive_completed_tasks 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn archive_completed_tasks(project_path: &str) -> io::Result<Vec<Task>> {
    let mut archived_tasks = Vec::new();
    for task in load_all_tasks(project_path) {
        if task.deleted || task.archived || task.status != TaskStatus::Completed {
            continue;
        }
        if let Some(updated_task) = archive_task(project_path, &task.id)? {
            archived_tasks.push(updated_task);
        }
    }
    Ok(archived_tasks)
}

/// 公开的 update_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn update_task(project_path: &str, task: &Task) -> io::Result<()> {
    save_task(project_path, task)
}

/// 公开的 reorder_tasks_in_status 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn reorder_tasks_in_status(
    project_path: &str,
    status: TaskStatus,
    task_ids: Vec<String>,
) -> io::Result<()> {
    with_index_lock(project_path, || {
        let mut index = load_index_unlocked(project_path);
        let status_key = status.to_string_key().to_string();

        index.order_by_status.insert(status_key.clone(), task_ids.clone());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut conn = open_index_connection(project_path)?;
            let mut reordered_tasks = Vec::new();

            for (order, task_id) in task_ids.iter().enumerate() {
                if let Some(mut task) = load_task_from_sqlite(&conn, task_id)? {
                    task.order = order as u32;
                    reordered_tasks.push(task);
                }
            }

            let tx = conn.transaction().map_err(sqlite_to_io_error)?;
            for task in &reordered_tasks {
                save_task_with_tx(&tx, task)?;
            }

            save_index_with_tx(&tx, &index)?;
            tx.commit().map_err(sqlite_to_io_error)?;
            Ok(())
        }

        #[cfg(target_arch = "wasm32")]
        {
            for (order, task_id) in task_ids.iter().enumerate() {
                if let Some(mut task) = load_task(project_path, task_id) {
                    task.order = order as u32;
                    save_task(project_path, &task)?;
                }
            }

            save_index_unlocked(project_path, &index)?;
            Ok(())
        }
    })
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod operations_tests;
