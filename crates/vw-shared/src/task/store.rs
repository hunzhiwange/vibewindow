//! 任务存储模块，负责任务文件、索引数据库、执行产物和看板设置的读写。

use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io;
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, OptionalExtension, Transaction, params};

#[cfg(not(target_arch = "wasm32"))]
use super::models::{SubTask, TaskExecutorBackend, TaskLogEntry};
use super::models::{Task, TaskIndex, TaskStatus};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

static TASK_INDEX_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(not(target_arch = "wasm32"))]
const RAW_ARTIFACT_EXECUTION_RESULT: &str = "execution_result";
#[cfg(not(target_arch = "wasm32"))]
const RAW_ARTIFACT_CODE_REVIEW_RESULT: &str = "code_review_result";

fn get_task_dir(project_path: &str) -> PathBuf {
    let mut path = PathBuf::from(project_path);
    path.push(".vibewindow");
    path.push("tasks");
    path
}

#[cfg(target_arch = "wasm32")]
fn get_task_file_path(project_path: &str, task_id: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push(format!("{}.json", task_id));
    path
}

#[cfg(target_arch = "wasm32")]
fn get_legacy_index_file_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.json");
    path
}

#[cfg(not(target_arch = "wasm32"))]
fn get_index_db_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.sqlite3");
    path
}

fn get_index_lock_file_path(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("_index.lock");
    path
}

fn get_project_index_lock(project_path: &str) -> Arc<Mutex<()>> {
    let mut locks = TASK_INDEX_LOCKS.lock();
    locks.entry(project_path.to_string()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
}

fn with_index_lock<T, F>(project_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    ensure_task_dir(project_path).expect("failed to create task directory before locking index");
    let lock = get_project_index_lock(project_path);
    let _memory_guard = lock.lock();
    let _file_guard =
        IndexFileLockGuard::acquire(project_path).expect("failed to acquire task index file lock");
    f()
}

struct IndexFileLockGuard {
    _file: File,
}

impl IndexFileLockGuard {
    fn acquire(project_path: &str) -> io::Result<Self> {
        let lock_path = get_index_lock_file_path(project_path);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        acquire_exclusive_file_lock(&file)?;
        Ok(Self { _file: file })
    }
}

#[cfg(unix)]
fn acquire_exclusive_file_lock(file: &File) -> io::Result<()> {
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
    if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

#[cfg(not(unix))]
fn acquire_exclusive_file_lock(_file: &File) -> io::Result<()> {
    Ok(())
}

fn get_task_log_dir(project_path: &str) -> PathBuf {
    let mut path = get_task_dir(project_path);
    path.push("logs");
    path
}

fn ensure_task_dir(project_path: &str) -> std::io::Result<()> {
    let dir = get_task_dir(project_path);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

fn max_sequence_for_date(index: &TaskIndex, date: &str) -> u32 {
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

/// 处理任务存储中的 rebuild index from task files 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
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

#[cfg(not(target_arch = "wasm32"))]
fn sqlite_to_io_error(err: rusqlite::Error) -> io::Error {
    io::Error::other(err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn json_to_io_error(err: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn open_index_connection(project_path: &str) -> io::Result<Connection> {
    ensure_task_dir(project_path)?;
    let db_path = get_index_db_path(project_path);
    let conn = Connection::open(db_path).map_err(sqlite_to_io_error)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS task_index_meta (
             key TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS tasks (
             id TEXT PRIMARY KEY,
             priority INTEGER NOT NULL,
             assignee TEXT NOT NULL,
             model TEXT NOT NULL,
             executor_id TEXT NOT NULL,
             description TEXT NOT NULL,
             prompt TEXT NOT NULL,
             status_key TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             updated_at_ms INTEGER NOT NULL,
             order_no INTEGER NOT NULL,
             deleted INTEGER NOT NULL,
             archived INTEGER NOT NULL,
             auto_promote_delay_ms INTEGER,
             last_error TEXT,
             pause_reason TEXT,
             retry_count INTEGER NOT NULL DEFAULT 0,
             last_active_at_ms INTEGER NOT NULL DEFAULT 0,
             execution_started_at_ms INTEGER,
             last_execution_duration_ms INTEGER,
             merge_source_branch TEXT,
             merge_target_branch TEXT,
             selected_worktree_path TEXT,
             subtasks_json TEXT NOT NULL,
             logs_json TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_tasks_status_order ON tasks(status_key, order_no, created_at_ms);
         CREATE TABLE IF NOT EXISTS task_artifacts (
             task_id TEXT NOT NULL,
             kind TEXT NOT NULL,
             payload_json TEXT NOT NULL,
             sha256_hex TEXT NOT NULL,
             created_at_ms INTEGER NOT NULL,
             PRIMARY KEY(task_id, kind),
             FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
         );",
    )
    .map_err(sqlite_to_io_error)?;
    Ok(conn)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_index_from_sqlite(conn: &Connection) -> io::Result<TaskIndex> {
    let mut index = TaskIndex::new();
    let mut stmt =
        conn.prepare("SELECT key, value FROM task_index_meta").map_err(sqlite_to_io_error)?;
    let mut rows = stmt.query([]).map_err(sqlite_to_io_error)?;
    while let Some(row) = rows.next().map_err(sqlite_to_io_error)? {
        let key: String = row.get(0).map_err(sqlite_to_io_error)?;
        let value: String = row.get(1).map_err(sqlite_to_io_error)?;
        match key.as_str() {
            "last_task_date" => index.last_task_date = Some(value),
            "last_task_seq" => {
                index.last_task_seq = value.parse::<u32>().unwrap_or(0);
            }
            _ => {}
        }
    }

    let mut stmt = conn
        .prepare("SELECT id, status_key FROM tasks ORDER BY status_key ASC, order_no ASC, created_at_ms ASC")
        .map_err(sqlite_to_io_error)?;
    let task_rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let status_key: String = row.get(1)?;
            Ok((id, status_key))
        })
        .map_err(sqlite_to_io_error)?;

    for row in task_rows {
        let (id, status_key) = row.map_err(sqlite_to_io_error)?;
        index.tasks.insert(id.clone(), status_key.clone());
        index.order_by_status.entry(status_key).or_default().push(id);
    }

    Ok(index)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_index_with_tx(tx: &Transaction<'_>, index: &TaskIndex) -> io::Result<()> {
    tx.execute("DELETE FROM task_index_meta", []).map_err(sqlite_to_io_error)?;
    if let Some(date) = &index.last_task_date {
        tx.execute(
            "INSERT INTO task_index_meta (key, value) VALUES (?1, ?2)",
            params!["last_task_date", date],
        )
        .map_err(sqlite_to_io_error)?;
    }
    tx.execute(
        "INSERT INTO task_index_meta (key, value) VALUES (?1, ?2)",
        params!["last_task_seq", index.last_task_seq.to_string()],
    )
    .map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_task_with_tx(tx: &Transaction<'_>, task: &Task) -> io::Result<()> {
    let subtasks_json = serde_json::to_string(&task.subtasks).map_err(json_to_io_error)?;
    let logs_json = serde_json::to_string(&task.logs).map_err(json_to_io_error)?;
    tx.execute(
        "INSERT OR REPLACE INTO tasks (
            id, priority, assignee, model, executor_id, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, auto_promote_delay_ms,
            last_error, pause_reason, retry_count, last_active_at_ms, execution_started_at_ms,
            last_execution_duration_ms, merge_source_branch, merge_target_branch,
            selected_worktree_path, subtasks_json, logs_json
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            ?9, ?10, ?11, ?12, ?13, ?14,
            ?15, ?16, ?17, ?18, ?19,
            ?20, ?21, ?22,
            ?23, ?24, ?25
        )",
        params![
            task.id,
            task.priority,
            task.assignee,
            task.model,
            task.executor.id(),
            task.description,
            task.prompt,
            task.status.to_string_key(),
            task.created_at_ms,
            task.updated_at_ms,
            task.order,
            task.deleted,
            task.archived,
            task.auto_promote_delay_ms,
            task.last_error,
            task.pause_reason,
            task.retry_count,
            task.last_active_at_ms,
            task.execution_started_at_ms,
            task.last_execution_duration_ms,
            task.merge_source_branch,
            task.merge_target_branch,
            task.selected_worktree_path,
            subtasks_json,
            logs_json,
        ],
    )
    .map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn delete_task_with_tx(tx: &Transaction<'_>, task_id: &str) -> io::Result<()> {
    tx.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]).map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_task_from_sqlite(conn: &Connection, task_id: &str) -> io::Result<Option<Task>> {
    conn.query_row(
        "SELECT
            id, priority, assignee, model, executor_id, description, prompt, status_key,
            created_at_ms, updated_at_ms, order_no, deleted, archived, auto_promote_delay_ms,
            last_error, pause_reason, retry_count, last_active_at_ms, execution_started_at_ms,
            last_execution_duration_ms, merge_source_branch, merge_target_branch,
            selected_worktree_path, subtasks_json, logs_json
         FROM tasks WHERE id = ?1",
        params![task_id],
        |row| {
            let subtasks_json: String = row.get(23)?;
            let logs_json: String = row.get(24)?;
            let subtasks = serde_json::from_str::<Vec<SubTask>>(&subtasks_json).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    subtasks_json.len(),
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;
            let logs = serde_json::from_str::<Vec<TaskLogEntry>>(&logs_json).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    logs_json.len(),
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;
            let executor_id: String = row.get(4)?;
            let status_key: String = row.get(7)?;

            Ok(Task {
                id: row.get(0)?,
                priority: row.get(1)?,
                assignee: row.get(2)?,
                model: row.get(3)?,
                executor: TaskExecutorBackend::from_id(&executor_id)
                    .unwrap_or(TaskExecutorBackend::Internal),
                description: row.get(5)?,
                prompt: row.get(6)?,
                status: TaskStatus::parse_key(&status_key).unwrap_or(TaskStatus::Pool),
                created_at_ms: row.get(8)?,
                updated_at_ms: row.get(9)?,
                order: row.get(10)?,
                deleted: row.get(11)?,
                archived: row.get(12)?,
                auto_promote_delay_ms: row.get(13)?,
                last_error: row.get(14)?,
                pause_reason: row.get(15)?,
                retry_count: row.get(16)?,
                last_active_at_ms: row.get(17)?,
                execution_started_at_ms: row.get(18)?,
                last_execution_duration_ms: row.get(19)?,
                merge_source_branch: row.get(20)?,
                merge_target_branch: row.get(21)?,
                selected_worktree_path: row.get(22)?,
                subtasks,
                logs,
            })
        },
    )
    .optional()
    .map_err(sqlite_to_io_error)
}

/// 加载 index 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
pub fn load_index(project_path: &str) -> TaskIndex {
    with_index_lock(project_path, || load_index_unlocked(project_path))
}

fn load_index_unlocked(project_path: &str) -> TaskIndex {
    #[cfg(not(target_arch = "wasm32"))]
    {
        match open_index_connection(project_path).and_then(|conn| load_index_from_sqlite(&conn)) {
            Ok(index) => index,
            Err(e) => {
                eprintln!("Failed to load task index: {}", e);
                TaskIndex::new()
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let path = get_legacy_index_file_path(project_path);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(index) => return index,
                    Err(e) => {
                        eprintln!("Failed to parse task index: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read task index: {}", e);
                }
            }
        }
        rebuild_index_from_task_files_unlocked(project_path).unwrap_or_default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_index_unlocked(project_path: &str, index: &TaskIndex) -> std::io::Result<()> {
    let mut conn = open_index_connection(project_path)?;
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    save_index_with_tx(&tx, index)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn save_index_unlocked(project_path: &str, index: &TaskIndex) -> std::io::Result<()> {
    ensure_task_dir(project_path)?;
    let path = get_legacy_index_file_path(project_path);
    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(index)?;
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, &path)?;
    Ok(())
}

/// 保存 index 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
pub fn save_index(project_path: &str, index: &TaskIndex) -> std::io::Result<()> {
    with_index_lock(project_path, || save_index_unlocked(project_path, index))
}

/// 加载 task 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
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

/// 加载 task 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
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

/// 保存 task 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task(project_path: &str, task: &Task) -> std::io::Result<()> {
    with_index_lock(project_path, || {
        let mut conn = open_index_connection(project_path)?;
        let tx = conn.transaction().map_err(sqlite_to_io_error)?;
        save_task_with_tx(&tx, task)?;
        tx.commit().map_err(sqlite_to_io_error)?;
        Ok(())
    })
}

/// 保存 task 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
#[cfg(target_arch = "wasm32")]
pub fn save_task(project_path: &str, task: &Task) -> std::io::Result<()> {
    ensure_task_dir(project_path)?;
    let path = get_task_file_path(project_path, &task.id);
    let content = serde_json::to_string_pretty(task)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// 删除 task file 数据。
///
/// 删除失败会通过 I/O 错误返回给调用方，避免静默丢失状态。
#[cfg(not(target_arch = "wasm32"))]
pub fn delete_task_file(project_path: &str, task_id: &str) -> std::io::Result<()> {
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

/// 删除 task file 数据。
///
/// 删除失败会通过 I/O 错误返回给调用方，避免静默丢失状态。
#[cfg(target_arch = "wasm32")]
pub fn delete_task_file(project_path: &str, task_id: &str) -> std::io::Result<()> {
    let path = get_task_file_path(project_path, task_id);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// 加载 all tasks 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
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

/// 加载 all tasks 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
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

/// 加载 tasks by status 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
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

/// 处理任务存储中的 create task 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
pub fn create_task(project_path: &str, mut task: Task) -> std::io::Result<Task> {
    ensure_task_dir(project_path)?;
    with_index_lock(project_path, || {
        let mut index = load_index_unlocked(project_path);

        let now_ms = crate::time::now_ms();
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

/// 处理任务存储中的 update task status 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
pub fn update_task_status(
    project_path: &str,
    task_id: &str,
    new_status: TaskStatus,
) -> std::io::Result<Option<Task>> {
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
                index.order_by_status.get(&new_status_key).map_or(0, |list| list.len() as u32);
            task.order = new_order;
            task.set_status(new_status);

            index.tasks.insert(task_id.to_string(), new_status_key.clone());
            index.order_by_status.entry(new_status_key).or_default().push(task_id.to_string());

            let tx = conn.transaction().map_err(sqlite_to_io_error)?;
            save_task_with_tx(&tx, &task)?;
            save_index_with_tx(&tx, &index)?;
            tx.commit().map_err(sqlite_to_io_error)?;

            Ok(Some(task))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let mut task = match load_task(project_path, task_id) {
                Some(task) => task,
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
                index.order_by_status.get(&new_status_key).map_or(0, |list| list.len() as u32);
            task.order = new_order;
            task.set_status(new_status);

            index.tasks.insert(task_id.to_string(), new_status_key.clone());
            index.order_by_status.entry(new_status_key).or_default().push(task_id.to_string());

            save_task(project_path, &task)?;
            save_index_unlocked(project_path, &index)?;

            Ok(Some(task))
        }
    })
}

/// 保存 task board settings 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
pub fn save_task_board_settings(
    project_path: &str,
    settings: &super::models::TaskBoardSettings,
) -> io::Result<()> {
    let mut path = get_task_dir(project_path);
    path.push("board_settings.json");
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)
}

/// 加载 task board settings 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
pub fn load_task_board_settings(project_path: &str) -> super::models::TaskBoardSettings {
    let mut path = get_task_dir(project_path);
    path.push("board_settings.json");
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => super::models::TaskBoardSettings::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn artifact_sha256_hex(payload: &serde_json::Value) -> io::Result<String> {
    let bytes = serde_json::to_vec(payload).map_err(json_to_io_error)?;
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{:02x}", byte);
    }
    Ok(hex)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_task_artifact(
    project_path: &str,
    task_id: &str,
    kind: &str,
    payload: &serde_json::Value,
) -> io::Result<()> {
    let mut conn = open_index_connection(project_path)?;
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    let payload_json = serde_json::to_string(payload).map_err(json_to_io_error)?;
    let sha256_hex = artifact_sha256_hex(payload)?;
    tx.execute(
        "INSERT OR REPLACE INTO task_artifacts (task_id, kind, payload_json, sha256_hex, created_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![task_id, kind, payload_json, sha256_hex, crate::time::now_ms()],
    )
    .map_err(sqlite_to_io_error)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_task_artifact(
    project_path: &str,
    task_id: &str,
    kind: &str,
) -> io::Result<Option<serde_json::Value>> {
    let conn = open_index_connection(project_path)?;
    let payload_json = conn
        .query_row(
            "SELECT payload_json FROM task_artifacts WHERE task_id = ?1 AND kind = ?2",
            params![task_id, kind],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(sqlite_to_io_error)?;
    payload_json.map(|payload| serde_json::from_str(&payload).map_err(json_to_io_error)).transpose()
}

/// 保存 task execution result 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task_execution_result(
    project_path: &str,
    task_id: &str,
    payload: &serde_json::Value,
) -> io::Result<()> {
    save_task_artifact(project_path, task_id, RAW_ARTIFACT_EXECUTION_RESULT, payload)
}

/// 加载 task execution result 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_task_execution_result(
    project_path: &str,
    task_id: &str,
) -> io::Result<Option<serde_json::Value>> {
    load_task_artifact(project_path, task_id, RAW_ARTIFACT_EXECUTION_RESULT)
}

/// 保存 task code review result 数据。
///
/// 返回 I/O 结果，调用方据此决定是否展示错误或继续刷新索引。
#[cfg(not(target_arch = "wasm32"))]
pub fn save_task_code_review_result(
    project_path: &str,
    task_id: &str,
    payload: &serde_json::Value,
) -> io::Result<()> {
    save_task_artifact(project_path, task_id, RAW_ARTIFACT_CODE_REVIEW_RESULT, payload)
}

/// 加载 task code review result 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_task_code_review_result(
    project_path: &str,
    task_id: &str,
) -> io::Result<Option<serde_json::Value>> {
    load_task_artifact(project_path, task_id, RAW_ARTIFACT_CODE_REVIEW_RESULT)
}

/// 处理任务存储中的 get task logs dir 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
pub fn get_task_logs_dir(project_path: &str) -> PathBuf {
    get_task_log_dir(project_path)
}

/// 处理任务存储中的 get task root dir 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
pub fn get_task_root_dir(project_path: &str) -> PathBuf {
    get_task_dir(project_path)
}

/// 处理任务存储中的 task file exists 行为。
///
/// 参数中的项目路径用于定位本地任务目录，返回值保持与持久化状态一致。
pub fn task_file_exists(project_path: &str, task_id: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        load_task(project_path, task_id).is_some()
    }

    #[cfg(target_arch = "wasm32")]
    {
        get_task_file_path(project_path, task_id).exists()
    }
}

/// 提供 sanitize project path 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub fn sanitize_project_path(project_path: &str) -> &Path {
    Path::new(project_path)
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;
