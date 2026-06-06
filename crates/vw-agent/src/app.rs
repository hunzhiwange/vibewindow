//! 应用层兼容入口。
//!
//! 本文件把 crate 内部模块重新组织到 `crate::app` 命名空间下，供历史
//! 调用点、测试和桌面侧集成以稳定路径访问 agent 运行时能力。

/// Agent 运行时模块重导出。
///
/// 该模块不承载业务逻辑，只提供兼容路径，避免调用方直接依赖 crate
/// 根部模块布局。按平台条件隐藏不可用能力，特别是 wasm 目标下的守护
/// 进程、网关和 PTY 相关模块。
pub(crate) mod agent {
    pub use crate::agent;
    pub use crate::approval;
    pub use crate::auth;
    pub use crate::bus;
    pub use crate::channels;
    pub use crate::command;
    pub use crate::config;
    pub use crate::coordination;
    pub(crate) use crate::cron;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::daemon;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::doctor;
    pub use crate::env;
    pub use crate::file;
    pub use crate::flag;

    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::gateway;
    pub use crate::global;

    pub(crate) use crate::health;
    pub(crate) use crate::heartbeat;
    pub use crate::hooks;
    pub use crate::id;
    pub use crate::installation;
    pub(crate) use crate::integrations;

    pub use crate::memory;
    pub(crate) use crate::multimodal;
    pub use crate::observability;
    pub use crate::patch;
    pub use crate::permission;
    pub use crate::project;
    pub use crate::provider;
    pub use crate::providers;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::pty;
    pub use crate::question;
    pub use crate::runtime;
    pub use crate::scheduler;
    pub use crate::security;

    pub use crate::session;
    pub use crate::shell;
    pub use crate::skill;

    pub use crate::skills;
    pub use crate::snapshot;
    pub use crate::sop;
    pub use crate::storage;
    pub use crate::tools;
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use crate::tunnel;

    pub use crate::util;
}

/// 应用配置桥接模块。
///
/// 该模块将桌面/UI 配置读写暴露到 `crate::app::config` 路径，实际存储
/// 与序列化逻辑仍由 session UI 配置模块负责。
pub(crate) mod config {
    /// 读取当前应用配置。
    ///
    /// # 返回值
    ///
    /// 返回序列化为 `serde_json::Value` 的配置快照；读取失败时由底层
    /// 配置模块决定默认值或错误表示。
    pub fn load_app_config() -> serde_json::Value {
        crate::session::ui_config::load_app_config()
    }

    /// 设置单个配置字段。
    ///
    /// # 参数
    ///
    /// - `key`: 要更新的配置键。
    /// - `value`: 写入该键的 JSON 值。
    ///
    /// # 错误处理
    ///
    /// 本函数保持历史 fire-and-forget 语义，不向调用方返回错误；失败
    /// 行为由底层 UI 配置模块处理。
    pub fn set_config_field(key: &str, value: serde_json::Value) {
        crate::session::ui_config::set_config_field(key, value)
    }
}

/// 启动期任务兼容模型。
///
/// 当前实现保存创建任务所需的最小字段，并按桌面任务看板 SQLite 契约
/// 投递到项目任务池。
pub(crate) mod task {
    #[cfg(not(target_arch = "wasm32"))]
    use std::fs::{File, OpenOptions};
    use std::io;
    #[cfg(unix)]
    use std::os::fd::AsRawFd;
    #[cfg(not(target_arch = "wasm32"))]
    use std::path::{Path, PathBuf};

    #[cfg(not(target_arch = "wasm32"))]
    use chrono::{Datelike, Utc};
    #[cfg(not(target_arch = "wasm32"))]
    use rusqlite::{Connection, OptionalExtension, Transaction, params};

    /// 轻量任务描述。
    ///
    /// 字段保持公开以兼容既有构造与测试代码，调用方可以直接检查任务
    /// 标识、模型和提示词。
    #[derive(Debug, Clone)]
    pub struct Task {
        /// 任务唯一标识。
        pub id: String,
        /// 任务优先级。
        pub priority: u32,
        /// 执行任务时使用的模型标识。
        pub model: String,
        /// 委托代理配置 key。
        pub agent: Option<String>,
        /// ACP 智能体 key。
        pub acp_agent: Option<String>,
        /// 用户或系统提供的任务提示词。
        pub prompt: String,
    }

    impl Task {
        /// 创建一个启动占位任务。
        ///
        /// # 参数
        ///
        /// - `_priority`: 历史接口保留的优先级参数，当前兼容实现不使用。
        ///
        /// # 返回值
        ///
        /// 返回带有稳定占位 id、自动模型和空提示词的任务。
        pub fn new(_priority: u32) -> Self {
            Self {
                id: "bootstrap-task".to_string(),
                priority: _priority,
                model: "auto".to_string(),
                agent: Some("main".to_string()),
                acp_agent: None,
                prompt: String::new(),
            }
        }
    }

    fn trimmed_optional(value: Option<String>) -> Option<String> {
        value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
    }

    /// 创建任务并返回该任务。
    ///
    /// # 参数
    ///
    /// - `_project_path`: 历史接口保留的项目路径，当前兼容实现不使用。
    /// - `task`: 要创建的任务对象。
    ///
    /// # 返回值
    ///
    /// 成功时原样返回 `task`。
    ///
    /// # 错误处理
    ///
    /// 当前实现不会主动产生 I/O 错误，返回 `std::io::Result` 是为了保持
    /// 调用方签名兼容。
    #[cfg(target_arch = "wasm32")]
    pub fn create_task(_project_path: &str, task: Task) -> std::io::Result<Task> {
        Ok(task)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_task(project_path: &str, mut task: Task) -> std::io::Result<Task> {
        let task_dir = task_dir(project_path);
        std::fs::create_dir_all(&task_dir)?;
        let _guard = IndexFileLockGuard::acquire(&task_dir.join("_index.lock"))?;
        let mut conn = open_task_connection(&task_dir)?;
        let tx = conn.transaction().map_err(sqlite_to_io_error)?;

        let now = Utc::now();
        let date = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());
        let next_seq = next_task_sequence(&tx, &date)?;
        let task_id = format!("T{date}.{next_seq:04}");
        let status_key = "pool";
        let order_no = next_order_no(&tx, status_key)?;
        let now_ms = now.timestamp_millis().max(0);
        let model = normalize_model(&task.model);
        let agent = trimmed_optional(task.agent.clone()).or_else(|| Some("main".to_string()));
        let acp_agent = trimmed_optional(task.acp_agent.clone());
        let subtasks_json = "[]";

        tx.execute(
            "INSERT OR REPLACE INTO tasks (
                id, priority, assignee, model, agent, acp_agent, description, prompt, status_key,
                created_at_ms, updated_at_ms, order_no, deleted, archived, subtasks_json,
                auto_promote_delay_ms, last_error, pause_reason, retry_count, last_active_at_ms,
                execution_started_at_ms, last_execution_duration_ms, merge_source_branch,
                merge_target_branch, selected_worktree_path
            ) VALUES (
                ?1, ?2, 'VibeWindow', ?3, ?4, ?5, '', ?6, ?7,
                ?8, ?9, ?10, 0, 0, ?11,
                NULL, NULL, NULL, 0, ?12,
                NULL, NULL, NULL, NULL, NULL
            )",
            params![
                task_id,
                i64::from(task.priority),
                model,
                agent,
                acp_agent,
                task.prompt,
                status_key,
                now_ms,
                now_ms,
                order_no,
                subtasks_json,
                now_ms,
            ],
        )
        .map_err(sqlite_to_io_error)?;
        tx.execute(
            "INSERT OR REPLACE INTO task_index_entries (task_id, status_key, order_no)
             VALUES (?1, ?2, ?3)",
            params![task_id, status_key, order_no],
        )
        .map_err(sqlite_to_io_error)?;
        tx.execute(
            "INSERT OR REPLACE INTO task_index_meta (key, value)
             VALUES ('last_task_date', ?1)",
            params![date],
        )
        .map_err(sqlite_to_io_error)?;
        tx.execute(
            "INSERT OR REPLACE INTO task_index_meta (key, value)
             VALUES ('last_task_seq', ?1)",
            params![next_seq.to_string()],
        )
        .map_err(sqlite_to_io_error)?;
        tx.execute(
            "INSERT INTO task_logs (task_id, seq_no, timestamp_ms, status_from, status_to, message)
             VALUES (?1, 0, ?2, NULL, NULL, '任务创建')",
            params![task_id, now_ms],
        )
        .map_err(sqlite_to_io_error)?;

        tx.commit().map_err(sqlite_to_io_error)?;
        task.id = task_id;
        task.model = model;
        task.agent = agent;
        task.acp_agent = acp_agent;
        Ok(task)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn task_dir(project_path: &str) -> PathBuf {
        PathBuf::from(project_path).join(".vibewindow").join("tasks")
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_task_connection(task_dir: &Path) -> io::Result<Connection> {
        let conn = Connection::open(task_dir.join("_index.sqlite3")).map_err(sqlite_to_io_error)?;
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
                 agent TEXT,
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
             ON task_logs(task_id, seq_no);",
        )
        .map_err(sqlite_to_io_error)?;
        Ok(conn)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn next_task_sequence(tx: &Transaction<'_>, date: &str) -> io::Result<u32> {
        let meta_seq = tx
            .query_row("SELECT value FROM task_index_meta WHERE key = 'last_task_seq'", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(sqlite_to_io_error)?
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let meta_date = tx
            .query_row(
                "SELECT value FROM task_index_meta WHERE key = 'last_task_date'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(sqlite_to_io_error)?;
        let meta_seq = if meta_date.as_deref() == Some(date) { meta_seq } else { 0 };
        let max_seq = max_sequence_for_date(tx, date)?;
        Ok(meta_seq.max(max_seq).saturating_add(1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn max_sequence_for_date(tx: &Transaction<'_>, date: &str) -> io::Result<u32> {
        let like = format!("T{date}.%");
        let mut stmt =
            tx.prepare("SELECT id FROM tasks WHERE id LIKE ?1").map_err(sqlite_to_io_error)?;
        let rows = stmt
            .query_map(params![like], |row| row.get::<_, String>(0))
            .map_err(sqlite_to_io_error)?;
        let mut max_seq = 0;
        for row in rows {
            let task_id = row.map_err(sqlite_to_io_error)?;
            if let Some(seq) = task_id
                .strip_prefix(&format!("T{date}."))
                .filter(|value| value.len() == 4)
                .and_then(|value| value.parse::<u32>().ok())
            {
                max_seq = max_seq.max(seq);
            }
        }
        Ok(max_seq)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn next_order_no(tx: &Transaction<'_>, status_key: &str) -> io::Result<i64> {
        tx.query_row(
            "SELECT COALESCE(MAX(order_no), -1) + 1
             FROM task_index_entries
             WHERE status_key = ?1",
            params![status_key],
            |row| row.get::<_, i64>(0),
        )
        .map_err(sqlite_to_io_error)
    }

    fn normalize_model(model: &str) -> String {
        let model = model.trim();
        if model.is_empty() { "auto".to_string() } else { model.to_string() }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn sqlite_to_io_error(error: rusqlite::Error) -> io::Error {
        io::Error::other(error)
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct IndexFileLockGuard {
        _file: File,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl IndexFileLockGuard {
        fn acquire(path: &Path) -> io::Result<Self> {
            let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
            acquire_exclusive_file_lock(&file)?;
            Ok(Self { _file: file })
        }
    }

    #[cfg(unix)]
    fn acquire_exclusive_file_lock(file: &File) -> io::Result<()> {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    #[cfg(all(not(unix), not(target_arch = "wasm32")))]
    fn acquire_exclusive_file_lock(_file: &File) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
