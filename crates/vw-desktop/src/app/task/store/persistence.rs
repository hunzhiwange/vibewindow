//! 任务存储层的 persistence.rs 子模块。
//!
//! 该模块负责任务索引、持久化或产物写入中的一部分能力。实现保持文件系统与 SQLite 路径清晰分离，让上层任务流程只依赖稳定的存储函数。

use std::io;

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, OptionalExtension, Transaction, params};

#[cfg(not(target_arch = "wasm32"))]
use crate::app::task::models::TaskStatus;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::task::models::{SubTask, TaskLogEntry};
use crate::app::task::models::{Task, TaskIndex};

#[cfg(not(target_arch = "wasm32"))]
use super::paths::get_index_db_path;
use super::paths::{ensure_task_dir, with_index_lock};
#[cfg(target_arch = "wasm32")]
use super::paths::{get_legacy_index_file_path, get_task_dir, get_task_file_path};

/// 模块内部可见的 max_sequence_for_date 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn max_sequence_for_date(index: &TaskIndex, date: &str) -> u32 {
    let prefix = format!("T{}.", date);
    let mut max_seq = 0u32;
    for task_id in index.tasks.keys() {
        if let Some(rest) = task_id.strip_prefix(&prefix)
            && rest.len() == 4
            && let Ok(seq) = rest.parse::<u32>()
            && seq > max_seq
        {
            max_seq = seq;
        }
    }
    max_seq
}

#[cfg(target_arch = "wasm32")]
fn parse_task_id_sequence(task_id: &str) -> Option<(&str, u32)> {
    let rest = task_id.strip_prefix('T')?;
    let (date, seq) = rest.split_once('.')?;
    if date.len() != 8 || seq.len() != 4 {
        return None;
    }
    let seq = seq.parse::<u32>().ok()?;
    Some((date, seq))
}

#[cfg(target_arch = "wasm32")]
fn rebuild_index_from_task_files_unlocked(project_path: &str) -> io::Result<TaskIndex> {
    let mut index = TaskIndex::new();
    let dir = get_task_dir(project_path);
    if !dir.exists() {
        return Ok(index);
    }

    let mut tasks = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if file_name == "_index.json" {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read task file {}: {}", path.display(), e);
                continue;
            }
        };

        let task = match serde_json::from_str::<Task>(&content) {
            Ok(task) => task,
            Err(e) => {
                eprintln!("Failed to parse task file {}: {}", path.display(), e);
                continue;
            }
        };

        tasks.push(task);
    }

    tasks.sort_by(|a, b| {
        a.status
            .to_string_key()
            .cmp(b.status.to_string_key())
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.created_at_ms.cmp(&b.created_at_ms))
            .then_with(|| a.id.cmp(&b.id))
    });

    for task in tasks {
        let status_key = task.status.to_string_key().to_string();
        let order_list = index.order_by_status.entry(status_key.clone()).or_default();
        index.tasks.insert(task.id.clone(), status_key);
        order_list.push(task.id.clone());

        if let Some((date, seq)) = parse_task_id_sequence(&task.id) {
            let should_update = match index.last_task_date.as_deref() {
                Some(current_date) if current_date > date => false,
                Some(current_date) if current_date == date && index.last_task_seq >= seq => false,
                _ => true,
            };
            if should_update {
                index.last_task_date = Some(date.to_string());
                index.last_task_seq = seq;
            }
        }
    }

    Ok(index)
}

/// 公开的 rebuild_index_from_task_files 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn rebuild_index_from_task_files(project_path: &str) -> io::Result<TaskIndex> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let index = load_index(project_path);
        save_index(project_path, &index)?;
        Ok(index)
    }

    #[cfg(target_arch = "wasm32")]
    {
        with_index_lock(project_path, || {
            let index = rebuild_index_from_task_files_unlocked(project_path)?;
            save_index_unlocked(project_path, &index)?;
            Ok(index)
        })
    }
}

/// 模块内部可见的 sqlite_to_io_error 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn sqlite_to_io_error(err: rusqlite::Error) -> io::Error {
    io::Error::other(err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn json_to_io_error(err: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn table_has_column(conn: &Connection, table: &str, column: &str) -> io::Result<bool> {
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&sql).map_err(sqlite_to_io_error)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1)).map_err(sqlite_to_io_error)?;

    for row in rows {
        if row.map_err(sqlite_to_io_error)? == column {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn migrate_tasks_table_to_acp_agent_only(conn: &mut Connection) -> io::Result<()> {
    if !table_has_column(conn, "tasks", "executor")? {
        return Ok(());
    }

    let has_acp_agent = table_has_column(conn, "tasks", "acp_agent")?;
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute_batch(
        "DROP INDEX IF EXISTS idx_tasks_status_order;
         ALTER TABLE tasks RENAME TO tasks_legacy_executor;
         CREATE TABLE tasks (
             id TEXT PRIMARY KEY,
             priority INTEGER NOT NULL,
             assignee TEXT NOT NULL,
             model TEXT NOT NULL,
             acp_agent TEXT,
             description TEXT NOT NULL,
             prompt TEXT NOT NULL,
             status_key TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             updated_at_ms INTEGER NOT NULL,
             order_no INTEGER NOT NULL,
             deleted INTEGER NOT NULL,
             archived INTEGER NOT NULL,
             subtasks_json TEXT NOT NULL,
             auto_promote_delay_ms INTEGER,
             last_error TEXT,
             pause_reason TEXT,
             retry_count INTEGER NOT NULL,
             last_active_at_ms INTEGER NOT NULL,
             execution_started_at_ms INTEGER,
             last_execution_duration_ms INTEGER,
             merge_source_branch TEXT,
             merge_target_branch TEXT,
             selected_worktree_path TEXT
         );",
    )
    .map_err(sqlite_to_io_error)?;

    let copy_sql = if has_acp_agent {
        "INSERT INTO tasks (
            id, priority, assignee, model, acp_agent, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
            auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
            execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
            merge_target_branch, selected_worktree_path
        )
        SELECT
            id, priority, assignee, model, acp_agent, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
            auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
            execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
            merge_target_branch, selected_worktree_path
        FROM tasks_legacy_executor"
    } else {
        "INSERT INTO tasks (
            id, priority, assignee, model, acp_agent, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
            auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
            execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
            merge_target_branch, selected_worktree_path
        )
        SELECT
            id, priority, assignee, model, NULL, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
            auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
            execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
            merge_target_branch, selected_worktree_path
        FROM tasks_legacy_executor"
    };
    tx.execute(copy_sql, []).map_err(sqlite_to_io_error)?;
    tx.execute_batch(
        "DROP TABLE tasks_legacy_executor;
         CREATE INDEX idx_tasks_status_order ON tasks(status_key, order_no, id);",
    )
    .map_err(sqlite_to_io_error)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn migrate_task_raw_artifacts_table_to_acp_agent(conn: &mut Connection) -> io::Result<()> {
    if !table_has_column(conn, "task_raw_artifacts", "executor")? {
        return Ok(());
    }

    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute_batch(
        "DROP INDEX IF EXISTS idx_task_raw_artifacts_task_kind_time;
         ALTER TABLE task_raw_artifacts RENAME TO task_raw_artifacts_legacy_executor;
         CREATE TABLE task_raw_artifacts (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             task_id TEXT NOT NULL,
             artifact_type TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             acp_agent TEXT NOT NULL,
             model TEXT NOT NULL,
             file_path TEXT,
             content_text TEXT NOT NULL,
             content_sha256 TEXT NOT NULL,
             status TEXT,
             UNIQUE(task_id, artifact_type, created_at_ms)
         );
         INSERT INTO task_raw_artifacts (
             id, task_id, artifact_type, created_at_ms, acp_agent, model, file_path,
             content_text, content_sha256, status
         )
         SELECT
             id, task_id, artifact_type, created_at_ms, executor, model, file_path,
             content_text, content_sha256, status
         FROM task_raw_artifacts_legacy_executor;
         DROP TABLE task_raw_artifacts_legacy_executor;
         CREATE INDEX idx_task_raw_artifacts_task_kind_time
         ON task_raw_artifacts(task_id, artifact_type, created_at_ms);",
    )
    .map_err(sqlite_to_io_error)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

/// 模块内部可见的 open_index_connection 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn open_index_connection(project_path: &str) -> io::Result<Connection> {
    ensure_task_dir(project_path)?;
    let db_path = get_index_db_path(project_path);
    let mut conn = Connection::open(db_path).map_err(sqlite_to_io_error)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS task_index_meta (
             key TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS task_index_entries (
             task_id TEXT PRIMARY KEY,
             status_key TEXT NOT NULL,
             order_no INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_task_index_entries_status_order
         ON task_index_entries(status_key, order_no, task_id);
         CREATE TABLE IF NOT EXISTS tasks (
             id TEXT PRIMARY KEY,
             priority INTEGER NOT NULL,
             assignee TEXT NOT NULL,
             model TEXT NOT NULL,
             acp_agent TEXT,
             description TEXT NOT NULL,
             prompt TEXT NOT NULL,
             status_key TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             updated_at_ms INTEGER NOT NULL,
             order_no INTEGER NOT NULL,
             deleted INTEGER NOT NULL,
             archived INTEGER NOT NULL,
             subtasks_json TEXT NOT NULL,
             auto_promote_delay_ms INTEGER,
             last_error TEXT,
             pause_reason TEXT,
             retry_count INTEGER NOT NULL,
             last_active_at_ms INTEGER NOT NULL,
             execution_started_at_ms INTEGER,
             last_execution_duration_ms INTEGER,
             merge_source_branch TEXT,
             merge_target_branch TEXT,
             selected_worktree_path TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_tasks_status_order
         ON tasks(status_key, order_no, id);
         CREATE TABLE IF NOT EXISTS task_logs (
             task_id TEXT NOT NULL,
             seq_no INTEGER NOT NULL,
             timestamp_ms INTEGER NOT NULL,
             status_from TEXT,
             status_to TEXT,
             message TEXT NOT NULL,
             PRIMARY KEY (task_id, seq_no)
         );
         CREATE INDEX IF NOT EXISTS idx_task_logs_task_seq
         ON task_logs(task_id, seq_no);
         CREATE TABLE IF NOT EXISTS task_raw_artifacts (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             task_id TEXT NOT NULL,
             artifact_type TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             acp_agent TEXT NOT NULL,
             model TEXT NOT NULL,
             file_path TEXT,
             content_text TEXT NOT NULL,
             content_sha256 TEXT NOT NULL,
             status TEXT,
             UNIQUE(task_id, artifact_type, created_at_ms)
         );
         CREATE INDEX IF NOT EXISTS idx_task_raw_artifacts_task_kind_time
         ON task_raw_artifacts(task_id, artifact_type, created_at_ms);",
    )
    .map_err(sqlite_to_io_error)?;
    migrate_tasks_table_to_acp_agent_only(&mut conn)?;
    migrate_task_raw_artifacts_table_to_acp_agent(&mut conn)?;
    Ok(conn)
}

/// 模块内部可见的 load_index_from_sqlite 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_index_from_sqlite(conn: &Connection) -> io::Result<TaskIndex> {
    let mut index = TaskIndex::new();

    let mut meta_stmt =
        conn.prepare("SELECT key, value FROM task_index_meta").map_err(sqlite_to_io_error)?;
    let meta_rows = meta_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(sqlite_to_io_error)?;
    for row in meta_rows {
        let (key, value) = row.map_err(sqlite_to_io_error)?;
        match key.as_str() {
            "last_task_date" if !value.is_empty() => index.last_task_date = Some(value),
            "last_task_seq" => {
                if let Ok(seq) = value.parse::<u32>() {
                    index.last_task_seq = seq;
                }
            }
            _ => {}
        }
    }

    let mut stmt = conn
        .prepare(
            "SELECT task_id, status_key FROM task_index_entries ORDER BY status_key, order_no, task_id",
        )
        .map_err(sqlite_to_io_error)?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(sqlite_to_io_error)?;

    for row in rows {
        let (task_id, status_key) = row.map_err(sqlite_to_io_error)?;
        index.tasks.insert(task_id.clone(), status_key.clone());
        index.order_by_status.entry(status_key).or_default().push(task_id);
    }

    Ok(index)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_index_to_sqlite(conn: &mut Connection, index: &TaskIndex) -> io::Result<()> {
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    save_index_with_tx(&tx, index)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

/// 模块内部可见的 save_index_with_tx 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn save_index_with_tx(tx: &Transaction<'_>, index: &TaskIndex) -> io::Result<()> {
    tx.execute("DELETE FROM task_index_entries", []).map_err(sqlite_to_io_error)?;

    for (status_key, task_ids) in &index.order_by_status {
        for (order_no, task_id) in task_ids.iter().enumerate() {
            tx.execute(
                "INSERT OR REPLACE INTO task_index_entries (task_id, status_key, order_no) VALUES (?1, ?2, ?3)",
                params![task_id, status_key, order_no as i64],
            )
            .map_err(sqlite_to_io_error)?;
        }
    }

    for (task_id, status_key) in &index.tasks {
        let exists = tx
            .query_row(
                "SELECT 1 FROM task_index_entries WHERE task_id = ?1 LIMIT 1",
                params![task_id],
                |_row| Ok(()),
            )
            .optional()
            .map_err(sqlite_to_io_error)?
            .is_some();
        if !exists {
            let next_order = tx
                .query_row(
                    "SELECT COALESCE(MAX(order_no), -1) + 1 FROM task_index_entries WHERE status_key = ?1",
                    params![status_key],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(sqlite_to_io_error)?;
            tx.execute(
                "INSERT OR REPLACE INTO task_index_entries (task_id, status_key, order_no) VALUES (?1, ?2, ?3)",
                params![task_id, status_key, next_order],
            )
            .map_err(sqlite_to_io_error)?;
        }
    }

    tx.execute(
        "INSERT OR REPLACE INTO task_index_meta (key, value) VALUES ('last_task_date', ?1)",
        params![index.last_task_date.clone().unwrap_or_default()],
    )
    .map_err(sqlite_to_io_error)?;
    tx.execute(
        "INSERT OR REPLACE INTO task_index_meta (key, value) VALUES ('last_task_seq', ?1)",
        params![index.last_task_seq.to_string()],
    )
    .map_err(sqlite_to_io_error)?;
    Ok(())
}

/// 模块内部可见的 save_task_with_tx 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn save_task_with_tx(tx: &Transaction<'_>, task: &Task) -> io::Result<()> {
    let subtasks_json = serde_json::to_string(&task.subtasks).map_err(json_to_io_error)?;
    tx.execute(
        "INSERT OR REPLACE INTO tasks (
            id, priority, assignee, model, acp_agent, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
            auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
            execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
            merge_target_branch, selected_worktree_path
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            ?9, ?10, ?11, ?12, ?13, ?14,
            ?15, ?16, ?17, ?18, ?19,
            ?20, ?21, ?22,
            ?23, ?24
        )",
        params![
            task.id,
            task.priority as i64,
            task.assignee,
            task.model,
            task.acp_agent,
            task.description,
            task.prompt,
            task.status.to_string_key(),
            task.created_at_ms as i64,
            task.updated_at_ms as i64,
            task.order as i64,
            task.deleted,
            task.archived,
            subtasks_json,
            task.auto_promote_delay_ms.map(|value| value as i64),
            task.last_error,
            task.pause_reason,
            task.retry_count as i64,
            task.last_active_at_ms as i64,
            task.execution_started_at_ms.map(|value| value as i64),
            task.last_execution_duration_ms.map(|value| value as i64),
            task.merge_source_branch,
            task.merge_target_branch,
            task.selected_worktree_path,
        ],
    )
    .map_err(sqlite_to_io_error)?;

    tx.execute("DELETE FROM task_logs WHERE task_id = ?1", params![task.id])
        .map_err(sqlite_to_io_error)?;
    for (seq_no, log) in task.logs.iter().enumerate() {
        tx.execute(
            "INSERT INTO task_logs (task_id, seq_no, timestamp_ms, status_from, status_to, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task.id,
                seq_no as i64,
                log.timestamp_ms as i64,
                log.status_from.map(|status| status.to_string_key().to_string()),
                log.status_to.map(|status| status.to_string_key().to_string()),
                log.message,
            ],
        )
        .map_err(sqlite_to_io_error)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_task_logs_from_sqlite(conn: &Connection, task_id: &str) -> io::Result<Vec<TaskLogEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT timestamp_ms, status_from, status_to, message
             FROM task_logs
             WHERE task_id = ?1
             ORDER BY seq_no ASC",
        )
        .map_err(sqlite_to_io_error)?;
    let rows = stmt
        .query_map(params![task_id], |row| {
            let timestamp_ms = row.get::<_, i64>(0)?;
            let status_from = row.get::<_, Option<String>>(1)?;
            let status_to = row.get::<_, Option<String>>(2)?;
            Ok(TaskLogEntry {
                timestamp_ms: timestamp_ms.max(0) as u64,
                status_from: status_from.as_deref().and_then(TaskStatus::parse_key),
                status_to: status_to.as_deref().and_then(TaskStatus::parse_key),
                message: row.get::<_, String>(3)?,
            })
        })
        .map_err(sqlite_to_io_error)?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(sqlite_to_io_error)?);
    }
    Ok(logs)
}

/// 模块内部可见的 load_task_from_sqlite 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_task_from_sqlite(conn: &Connection, task_id: &str) -> io::Result<Option<Task>> {
    let task_row = conn
        .query_row(
            "SELECT
                id, priority, assignee, model, acp_agent, description, prompt, status_key,
                created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
                auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
                execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
                merge_target_branch, selected_worktree_path
             FROM tasks
             WHERE id = ?1
             LIMIT 1",
            params![task_id],
            |row| {
                let status_key = row.get::<_, String>(7)?;
                let subtasks_json = row.get::<_, String>(13)?;
                let mut subtasks =
                    serde_json::from_str::<Vec<SubTask>>(&subtasks_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            13,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                for subtask in &mut subtasks {
                    if subtask.completed
                        && subtask.status == crate::app::task::SubTaskStatus::Pending
                    {
                        subtask.status = crate::app::task::SubTaskStatus::Completed;
                    }
                }

                Ok(Task {
                    id: row.get::<_, String>(0)?,
                    priority: row.get::<_, i64>(1)?.max(0) as u32,
                    assignee: row.get::<_, String>(2)?,
                    model: row.get::<_, String>(3)?,
                    acp_agent: row.get::<_, Option<String>>(4)?,
                    description: row.get::<_, String>(5)?,
                    prompt: row.get::<_, String>(6)?,
                    status: TaskStatus::parse_key(&status_key).unwrap_or(TaskStatus::Pool),
                    created_at_ms: row.get::<_, i64>(8)?.max(0) as u64,
                    updated_at_ms: row.get::<_, i64>(9)?.max(0) as u64,
                    logs: Vec::new(),
                    order: row.get::<_, i64>(10)?.max(0) as u32,
                    deleted: row.get::<_, bool>(11)?,
                    archived: row.get::<_, bool>(12)?,
                    subtasks,
                    auto_promote_delay_ms: row
                        .get::<_, Option<i64>>(14)?
                        .map(|value| value.max(0) as u64),
                    last_error: row.get::<_, Option<String>>(15)?,
                    pause_reason: row.get::<_, Option<String>>(16)?,
                    retry_count: row.get::<_, i64>(17)?.max(0) as u32,
                    last_active_at_ms: row.get::<_, i64>(18)?.max(0) as u64,
                    execution_started_at_ms: row
                        .get::<_, Option<i64>>(19)?
                        .map(|value| value.max(0) as u64),
                    last_execution_duration_ms: row
                        .get::<_, Option<i64>>(20)?
                        .map(|value| value.max(0) as u64),
                    merge_source_branch: row.get::<_, Option<String>>(21)?,
                    merge_target_branch: row.get::<_, Option<String>>(22)?,
                    selected_worktree_path: row.get::<_, Option<String>>(23)?,
                })
            },
        )
        .optional()
        .map_err(sqlite_to_io_error)?;

    let Some(mut task) = task_row else {
        return Ok(None);
    };
    task.logs = load_task_logs_from_sqlite(conn, &task.id)?;
    Ok(Some(task))
}

/// 模块内部可见的 delete_task_with_tx 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn delete_task_with_tx(tx: &Transaction<'_>, task_id: &str) -> io::Result<()> {
    tx.execute("DELETE FROM task_logs WHERE task_id = ?1", params![task_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM task_raw_artifacts WHERE task_id = ?1", params![task_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]).map_err(sqlite_to_io_error)?;
    Ok(())
}

/// 模块内部可见的 load_index_unlocked 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_index_unlocked(project_path: &str) -> TaskIndex {
    match open_index_connection(project_path).and_then(|conn| load_index_from_sqlite(&conn)) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("Failed to load task index from SQLite: {}", e);
            TaskIndex::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_legacy_index(project_path: &str) -> TaskIndex {
    let path = get_legacy_index_file_path(project_path);
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(index) => return index,
                Err(e) => eprintln!("Failed to parse legacy task index: {}", e),
            },
            Err(e) => eprintln!("Failed to read legacy task index: {}", e),
        }
    }
    TaskIndex::new()
}

/// 模块内部可见的 load_index_unlocked 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub(super) fn load_index_unlocked(project_path: &str) -> TaskIndex {
    load_legacy_index(project_path)
}

/// 公开的 load_index 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn load_index(project_path: &str) -> TaskIndex {
    with_index_lock(project_path, || load_index_unlocked(project_path))
}

/// 模块内部可见的 save_index_unlocked 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn save_index_unlocked(project_path: &str, index: &TaskIndex) -> io::Result<()> {
    let mut conn = open_index_connection(project_path)?;
    save_index_to_sqlite(&mut conn, index)
}

/// 模块内部可见的 save_index_unlocked 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub(super) fn save_index_unlocked(project_path: &str, index: &TaskIndex) -> io::Result<()> {
    ensure_task_dir(project_path)?;
    let path = get_legacy_index_file_path(project_path);
    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(index)?;
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, &path)?;
    Ok(())
}

/// 公开的 save_index 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn save_index(project_path: &str, index: &TaskIndex) -> io::Result<()> {
    with_index_lock(project_path, || save_index_unlocked(project_path, index))
}

/// 公开的 load_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_task(project_path: &str, task_id: &str) -> Option<Task> {
    with_index_lock(project_path, || match open_index_connection(project_path) {
        Ok(conn) => match load_task_from_sqlite(&conn, task_id) {
            Ok(task) => task,
            Err(e) => {
                eprintln!("Failed to load task {} from SQLite: {}", task_id, e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to open task database for {}: {}", task_id, e);
            None
        }
    })
}

/// 公开的 load_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn load_task(project_path: &str, task_id: &str) -> Option<Task> {
    let path = get_task_file_path(project_path, task_id);
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(task) => return Some(task),
                Err(e) => {
                    eprintln!("Failed to parse task {}: {}", task_id, e);
                }
            },
            Err(e) => {
                eprintln!("Failed to read task {}: {}", task_id, e);
            }
        }
    }
    None
}

/// 公开的 save_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task(project_path: &str, task: &Task) -> io::Result<()> {
    with_index_lock(project_path, || {
        let mut conn = open_index_connection(project_path)?;
        let tx = conn.transaction().map_err(sqlite_to_io_error)?;
        save_task_with_tx(&tx, task)?;
        tx.commit().map_err(sqlite_to_io_error)?;
        Ok(())
    })
}

/// 公开的 save_task 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn save_task(project_path: &str, task: &Task) -> io::Result<()> {
    ensure_task_dir(project_path)?;
    let path = get_task_file_path(project_path, &task.id);
    let content = serde_json::to_string_pretty(task)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// 公开的 delete_task_file 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn delete_task_file(project_path: &str, task_id: &str) -> io::Result<()> {
    with_index_lock(project_path, || {
        let mut conn = open_index_connection(project_path)?;
        let mut index = load_index_from_sqlite(&conn)?;
        if let Some(status_key) = index.tasks.remove(task_id)
            && let Some(order_list) = index.order_by_status.get_mut(&status_key)
        {
            order_list.retain(|id| id != task_id);
        }
        let tx = conn.transaction().map_err(sqlite_to_io_error)?;
        delete_task_with_tx(&tx, task_id)?;
        save_index_with_tx(&tx, &index)?;
        tx.commit().map_err(sqlite_to_io_error)?;
        Ok(())
    })
}

/// 公开的 delete_task_file 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn delete_task_file(project_path: &str, task_id: &str) -> io::Result<()> {
    let path = get_task_file_path(project_path, task_id);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// 公开的 load_all_tasks 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_all_tasks(project_path: &str) -> Vec<Task> {
    with_index_lock(project_path, || {
        let index = load_index_unlocked(project_path);
        let conn = match open_index_connection(project_path) {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to open task database: {}", e);
                return Vec::new();
            }
        };

        let mut tasks = Vec::new();
        for task_id in index.tasks.keys() {
            match load_task_from_sqlite(&conn, task_id) {
                Ok(Some(task)) if !task.deleted => tasks.push(task),
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to load task {} from SQLite: {}", task_id, e);
                }
            }
        }
        tasks
    })
}

/// 公开的 load_all_tasks 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
#[cfg(target_arch = "wasm32")]
pub fn load_all_tasks(project_path: &str) -> Vec<Task> {
    let index = load_index(project_path);
    let mut tasks = Vec::new();
    for task_id in index.tasks.keys() {
        if let Some(task) = load_task(project_path, task_id) {
            if !task.deleted {
                tasks.push(task);
            }
        }
    }
    tasks
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod persistence_tests;
