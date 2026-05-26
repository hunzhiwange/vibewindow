use crate::session::info::Info;
pub use crate::session::ui_types::{
    ChatMessage, ChatRole, ChatSession, ChatSessionMeta, ChatSessionStep, SessionTodoItem,
    ThinkTiming, TokenUsage,
};
#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, OptionalExtension, params};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::io;
use std::path::{Path, PathBuf};

use super::path;

#[cfg(not(target_arch = "wasm32"))]
fn sqlite_to_io_error(err: rusqlite::Error) -> io::Error {
    io::Error::other(err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn json_to_io_error(err: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn open_sessions_connection(data_dir: &Path, scope: Option<&str>) -> io::Result<Connection> {
    let Some(db_path) = path::sessions_db_path_for_scope(data_dir, scope) else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "session scope is unavailable"));
    };
    if let Some(dir) = db_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let conn = Connection::open(&db_path).map_err(sqlite_to_io_error)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         CREATE TABLE IF NOT EXISTS session_index (
             session_id TEXT PRIMARY KEY,
             title TEXT NOT NULL,
             created_ms INTEGER NOT NULL,
             updated_ms INTEGER NOT NULL,
             message_count INTEGER NOT NULL,
             call_count INTEGER NOT NULL,
             last_content TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_session_index_updated
         ON session_index(updated_ms DESC, session_id);
         CREATE TABLE IF NOT EXISTS session_messages (
             session_id TEXT NOT NULL,
             seq_no INTEGER NOT NULL,
             role TEXT NOT NULL,
             message_id TEXT,
             content TEXT NOT NULL,
             think_timing_json TEXT NOT NULL,
             PRIMARY KEY (session_id, seq_no)
         );
         CREATE INDEX IF NOT EXISTS idx_session_messages_session_seq
         ON session_messages(session_id, seq_no);
         CREATE TABLE IF NOT EXISTS session_calls (
             session_id TEXT NOT NULL,
             seq_no INTEGER NOT NULL,
             payload_json TEXT NOT NULL,
             PRIMARY KEY (session_id, seq_no)
         );
         CREATE INDEX IF NOT EXISTS idx_session_calls_session_seq
         ON session_calls(session_id, seq_no);
         CREATE TABLE IF NOT EXISTS session_steps (
             session_id TEXT NOT NULL,
             step_index INTEGER NOT NULL,
             started_ms INTEGER NOT NULL,
             finished_ms INTEGER,
             start_snapshot_path TEXT,
             finish_snapshot_path TEXT,
             usage_json TEXT NOT NULL,
             cost_usd REAL,
             finish_reason TEXT,
             model TEXT,
             PRIMARY KEY (session_id, step_index)
         );
         CREATE INDEX IF NOT EXISTS idx_session_steps_session_index
         ON session_steps(session_id, step_index);
         CREATE TABLE IF NOT EXISTS session_todos (
             session_id TEXT NOT NULL,
             order_no INTEGER NOT NULL,
             todo_id TEXT NOT NULL,
             content TEXT NOT NULL,
             status TEXT NOT NULL,
             priority TEXT NOT NULL,
             PRIMARY KEY (session_id, order_no),
             UNIQUE (session_id, todo_id)
         );
         CREATE INDEX IF NOT EXISTS idx_session_todos_session_order
         ON session_todos(session_id, order_no, todo_id);
         CREATE TABLE IF NOT EXISTS agent_sessions (
             session_id TEXT PRIMARY KEY,
             project_id TEXT NOT NULL,
             parent_id TEXT,
             updated_ms INTEGER NOT NULL,
             info_json TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_agent_sessions_project_updated
         ON agent_sessions(project_id, updated_ms DESC, session_id DESC);
         CREATE INDEX IF NOT EXISTS idx_agent_sessions_parent
         ON agent_sessions(parent_id, session_id);",
    )
    .map_err(sqlite_to_io_error)?;
    ensure_session_message_id_column(&conn)?;
    Ok(conn)
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_session_message_id_column(conn: &Connection) -> io::Result<()> {
    let mut columns = HashSet::new();
    let mut stmt =
        conn.prepare("PRAGMA table_info(session_messages)").map_err(sqlite_to_io_error)?;
    let mut rows = stmt.query([]).map_err(sqlite_to_io_error)?;
    while let Some(row) = rows.next().map_err(sqlite_to_io_error)? {
        columns.insert(row.get::<_, String>(1).map_err(sqlite_to_io_error)?);
    }
    if !columns.contains("message_id") {
        conn.execute("ALTER TABLE session_messages ADD COLUMN message_id TEXT", [])
            .map_err(sqlite_to_io_error)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn chat_role_from_payload(role: &str) -> Option<ChatRole> {
    match role {
        "user" => Some(ChatRole::User),
        "assistant" => Some(ChatRole::Assistant),
        "system" => Some(ChatRole::System),
        "tool" => Some(ChatRole::Tool),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn session_last_content(messages: &[ChatMessage]) -> Option<String> {
    messages.iter().rev().find_map(|message| {
        if message.content.trim().is_empty() { None } else { Some(message.content.clone()) }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn load_sessions_index_from_sqlite(conn: &Connection) -> io::Result<Vec<ChatSessionMeta>> {
    let mut stmt = conn
        .prepare(
            "SELECT session_id, title, updated_ms, message_count, call_count, last_content
             FROM session_index
             ORDER BY updated_ms DESC, session_id DESC
             LIMIT 50",
        )
        .map_err(sqlite_to_io_error)?;
    let mut rows = stmt.query([]).map_err(sqlite_to_io_error)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(sqlite_to_io_error)? {
        let updated_ms = row.get::<_, i64>(2).map_err(sqlite_to_io_error)?;
        let message_count = row.get::<_, i64>(3).map_err(sqlite_to_io_error)?;
        let call_count = row.get::<_, i64>(4).map_err(sqlite_to_io_error)?;
        out.push(ChatSessionMeta {
            id: row.get::<_, String>(0).map_err(sqlite_to_io_error)?,
            title: row.get::<_, String>(1).map_err(sqlite_to_io_error)?,
            updated_ms: u64::try_from(updated_ms).unwrap_or_default(),
            message_count: usize::try_from(message_count).unwrap_or_default(),
            call_count: usize::try_from(call_count).unwrap_or_default(),
            last_content: row.get::<_, Option<String>>(5).map_err(sqlite_to_io_error)?,
        });
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_sessions_index_to_sqlite(
    conn: &mut Connection,
    sessions: &[ChatSessionMeta],
) -> io::Result<()> {
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_index", []).map_err(sqlite_to_io_error)?;
    for session in sessions {
        tx.execute(
            "INSERT INTO session_index (
                 session_id, title, created_ms, updated_ms, message_count, call_count, last_content
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &session.id,
                &session.title,
                session.updated_ms as i64,
                session.updated_ms as i64,
                session.message_count as i64,
                session.call_count as i64,
                session.last_content.clone(),
            ],
        )
        .map_err(sqlite_to_io_error)?;
    }
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_session_to_sqlite(conn: &mut Connection, session: &ChatSession) -> io::Result<()> {
    ensure_session_message_id_column(conn)?;
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute(
        "INSERT OR REPLACE INTO session_index (
             session_id, title, created_ms, updated_ms, message_count, call_count, last_content
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &session.id,
            &session.title,
            session.created_ms as i64,
            session.updated_ms as i64,
            session.messages.len() as i64,
            session.calls.len() as i64,
            session_last_content(&session.messages),
        ],
    )
    .map_err(sqlite_to_io_error)?;

    tx.execute("DELETE FROM session_messages WHERE session_id = ?1", params![&session.id])
        .map_err(sqlite_to_io_error)?;
    for (seq_no, message) in session.messages.iter().enumerate() {
        let think_timing_json =
            serde_json::to_string(&message.think_timing).map_err(json_to_io_error)?;
        let message_id = session.message_ids.get(seq_no).cloned().flatten();
        tx.execute(
            "INSERT INTO session_messages (session_id, seq_no, role, message_id, content, think_timing_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &session.id,
                seq_no as i64,
                match message.role {
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                    ChatRole::System => "system",
                    ChatRole::Tool => "tool",
                },
                message_id,
                &message.content,
                think_timing_json,
            ],
        )
        .map_err(sqlite_to_io_error)?;
    }

    tx.execute("DELETE FROM session_calls WHERE session_id = ?1", params![&session.id])
        .map_err(sqlite_to_io_error)?;
    for (seq_no, payload) in session.calls.iter().enumerate() {
        let payload_json = serde_json::to_string(payload).map_err(json_to_io_error)?;
        tx.execute(
            "INSERT INTO session_calls (session_id, seq_no, payload_json) VALUES (?1, ?2, ?3)",
            params![&session.id, seq_no as i64, payload_json],
        )
        .map_err(sqlite_to_io_error)?;
    }

    tx.execute("DELETE FROM session_steps WHERE session_id = ?1", params![&session.id])
        .map_err(sqlite_to_io_error)?;
    for step in &session.steps {
        let usage_json = serde_json::to_string(&step.usage).map_err(json_to_io_error)?;
        tx.execute(
            "INSERT INTO session_steps (
                 session_id, step_index, started_ms, finished_ms, start_snapshot_path,
                 finish_snapshot_path, usage_json, cost_usd, finish_reason, model
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &session.id,
                step.index as i64,
                step.started_ms as i64,
                step.finished_ms.map(|value| value as i64),
                step.start_snapshot_path.clone(),
                step.finish_snapshot_path.clone(),
                usage_json,
                step.cost_usd,
                step.finish_reason.clone(),
                step.model.clone(),
            ],
        )
        .map_err(sqlite_to_io_error)?;
    }

    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_session_from_sqlite(
    conn: &Connection,
    session_id: &str,
) -> io::Result<Option<ChatSession>> {
    ensure_session_message_id_column(conn)?;
    let header = conn
        .query_row(
            "SELECT title, created_ms, updated_ms FROM session_index WHERE session_id = ?1",
            params![session_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
        )
        .optional()
        .map_err(sqlite_to_io_error)?;
    let Some((title, created_ms, updated_ms)) = header else {
        return Ok(None);
    };

    let mut messages = Vec::new();
    let mut message_ids = Vec::new();
    let mut message_stmt = conn
        .prepare(
            "SELECT role, message_id, content, think_timing_json
             FROM session_messages
             WHERE session_id = ?1
             ORDER BY seq_no",
        )
        .map_err(sqlite_to_io_error)?;
    let mut message_rows = message_stmt.query(params![session_id]).map_err(sqlite_to_io_error)?;
    while let Some(row) = message_rows.next().map_err(sqlite_to_io_error)? {
        let role = row.get::<_, String>(0).map_err(sqlite_to_io_error)?;
        let think_timing_json = row.get::<_, String>(3).map_err(sqlite_to_io_error)?;
        let Some(role) = chat_role_from_payload(&role) else {
            continue;
        };
        message_ids.push(row.get::<_, Option<String>>(1).map_err(sqlite_to_io_error)?);
        messages.push(ChatMessage {
            role,
            content: row.get::<_, String>(2).map_err(sqlite_to_io_error)?,
            think_timing: serde_json::from_str(&think_timing_json).map_err(json_to_io_error)?,
        });
    }

    let mut calls = Vec::new();
    let mut call_stmt = conn
        .prepare("SELECT payload_json FROM session_calls WHERE session_id = ?1 ORDER BY seq_no")
        .map_err(sqlite_to_io_error)?;
    let mut call_rows = call_stmt.query(params![session_id]).map_err(sqlite_to_io_error)?;
    while let Some(row) = call_rows.next().map_err(sqlite_to_io_error)? {
        let payload_json = row.get::<_, String>(0).map_err(sqlite_to_io_error)?;
        calls.push(serde_json::from_str(&payload_json).map_err(json_to_io_error)?);
    }

    let mut steps = Vec::new();
    let mut step_stmt = conn
        .prepare(
            "SELECT step_index, started_ms, finished_ms, start_snapshot_path, finish_snapshot_path,
                    usage_json, cost_usd, finish_reason, model
             FROM session_steps
             WHERE session_id = ?1
             ORDER BY step_index",
        )
        .map_err(sqlite_to_io_error)?;
    let mut step_rows = step_stmt.query(params![session_id]).map_err(sqlite_to_io_error)?;
    while let Some(row) = step_rows.next().map_err(sqlite_to_io_error)? {
        let usage_json = row.get::<_, String>(5).map_err(sqlite_to_io_error)?;
        steps.push(ChatSessionStep {
            index: u32::try_from(row.get::<_, i64>(0).map_err(sqlite_to_io_error)?)
                .unwrap_or_default(),
            started_ms: u64::try_from(row.get::<_, i64>(1).map_err(sqlite_to_io_error)?)
                .unwrap_or_default(),
            finished_ms: row
                .get::<_, Option<i64>>(2)
                .map_err(sqlite_to_io_error)?
                .and_then(|value| u64::try_from(value).ok()),
            start_snapshot_path: row.get::<_, Option<String>>(3).map_err(sqlite_to_io_error)?,
            finish_snapshot_path: row.get::<_, Option<String>>(4).map_err(sqlite_to_io_error)?,
            usage: serde_json::from_str(&usage_json).map_err(json_to_io_error)?,
            cost_usd: row.get::<_, Option<f64>>(6).map_err(sqlite_to_io_error)?,
            finish_reason: row.get::<_, Option<String>>(7).map_err(sqlite_to_io_error)?,
            model: row.get::<_, Option<String>>(8).map_err(sqlite_to_io_error)?,
        });
    }

    Ok(Some(ChatSession {
        id: session_id.to_string(),
        title,
        messages,
        message_ids,
        calls,
        steps,
        created_ms: u64::try_from(created_ms).unwrap_or_default(),
        updated_ms: u64::try_from(updated_ms).unwrap_or_default(),
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn delete_session_from_sqlite(conn: &mut Connection, session_id: &str) -> io::Result<()> {
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_messages WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_calls WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_steps WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_todos WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_index WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_session_todos_from_sqlite(
    conn: &Connection,
    session_id: &str,
) -> io::Result<Vec<SessionTodoItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT todo_id, content, status, priority
             FROM session_todos
             WHERE session_id = ?1
             ORDER BY order_no, todo_id",
        )
        .map_err(sqlite_to_io_error)?;
    let mut rows = stmt.query(params![session_id]).map_err(sqlite_to_io_error)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(sqlite_to_io_error)? {
        out.push(SessionTodoItem {
            id: row.get::<_, String>(0).map_err(sqlite_to_io_error)?,
            content: row.get::<_, String>(1).map_err(sqlite_to_io_error)?,
            status: row.get::<_, String>(2).map_err(sqlite_to_io_error)?,
            priority: row.get::<_, String>(3).map_err(sqlite_to_io_error)?,
        });
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_session_todos_to_sqlite(
    conn: &mut Connection,
    session_id: &str,
    todos: &[SessionTodoItem],
) -> io::Result<()> {
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    tx.execute("DELETE FROM session_todos WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    for (order_no, todo) in todos.iter().enumerate() {
        tx.execute(
            "INSERT INTO session_todos (session_id, order_no, todo_id, content, status, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_id,
                order_no as i64,
                &todo.id,
                &todo.content,
                &todo.status,
                &todo.priority,
            ],
        )
        .map_err(sqlite_to_io_error)?;
    }
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_agent_session_from_sqlite(conn: &Connection, session_id: &str) -> io::Result<Option<Info>> {
    let info_json = conn
        .query_row(
            "SELECT info_json FROM agent_sessions WHERE session_id = ?1",
            params![session_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(sqlite_to_io_error)?;
    let Some(info_json) = info_json else {
        return Ok(None);
    };
    serde_json::from_str::<Info>(&info_json).map(Some).map_err(json_to_io_error)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_agent_sessions_from_sqlite(conn: &Connection) -> io::Result<Vec<Info>> {
    let mut stmt = conn
        .prepare(
            "SELECT info_json
             FROM agent_sessions
             ORDER BY updated_ms DESC, session_id DESC",
        )
        .map_err(sqlite_to_io_error)?;
    let mut rows = stmt.query([]).map_err(sqlite_to_io_error)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(sqlite_to_io_error)? {
        let info_json = row.get::<_, String>(0).map_err(sqlite_to_io_error)?;
        let info = serde_json::from_str::<Info>(&info_json).map_err(json_to_io_error)?;
        out.push(info);
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_agent_session_to_sqlite(conn: &mut Connection, info: &Info) -> io::Result<()> {
    let tx = conn.transaction().map_err(sqlite_to_io_error)?;
    let info_json = serde_json::to_string(info).map_err(json_to_io_error)?;
    tx.execute(
        "INSERT OR REPLACE INTO agent_sessions (
             session_id, project_id, parent_id, updated_ms, info_json
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &info.id,
            &info.project_id,
            info.parent_id.clone(),
            info.time.updated as i64,
            info_json,
        ],
    )
    .map_err(sqlite_to_io_error)?;
    tx.commit().map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn delete_agent_session_from_sqlite(conn: &mut Connection, session_id: &str) -> io::Result<()> {
    conn.execute("DELETE FROM agent_sessions WHERE session_id = ?1", params![session_id])
        .map_err(sqlite_to_io_error)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn session_scope_db_paths(data_dir: &Path) -> Vec<PathBuf> {
    let base = data_dir.join("storage").join("session").join("scoped");
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let db_path = path.join("index.sqlite3");
        if db_path.is_file() {
            out.push(db_path);
        }
    }
    out.sort();
    out
}

// === 对外 CRUD 接口 ===

/// 保存某个步骤的会话快照文件。
pub fn save_session_step_snapshot(
    data_dir: &Path,
    session: &ChatSession,
    step_index: u32,
    kind: &str,
    scope: Option<&str>,
) -> Option<PathBuf> {
    let path =
        path::session_step_snapshot_file_path(data_dir, &session.id, step_index, kind, scope)?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let content = serde_json::to_string_pretty(session).ok()?;
    let _ = std::fs::write(&path, content);
    Some(path)
}

/// 持久化某一步的原始 LLM 输出。
pub fn persist_llm_raw_step(
    data_dir: &Path,
    session_id: &str,
    step_index: u32,
    payload: &serde_json::Value,
    scope: Option<&str>,
) -> Option<PathBuf> {
    let path =
        path::session_step_llm_raw_file_path_scoped(data_dir, session_id, step_index, scope)?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let content = serde_json::to_string_pretty(payload).ok()?;
    let _ = std::fs::write(&path, content);
    Some(path)
}

/// 读取指定 scope 下某个会话的待办列表。
pub fn load_session_todos_scoped(
    _data_dir: &Path,
    session_id: &str,
    scope: Option<&str>,
) -> Vec<SessionTodoItem> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(conn) = open_sessions_connection(_data_dir, scope) else {
            return Vec::new();
        };
        load_session_todos_from_sqlite(&conn, session_id).unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (session_id, scope);
        Vec::new()
    }
}

/// 保存指定 scope 下某个会话的待办列表。
pub fn save_session_todos_scoped(
    _data_dir: &Path,
    session_id: &str,
    todos: &[SessionTodoItem],
    scope: Option<&str>,
) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = path::sessions_db_path_for_scope(_data_dir, scope)?;
        let mut conn = open_sessions_connection(_data_dir, scope).ok()?;
        save_session_todos_to_sqlite(&mut conn, session_id, todos).ok()?;
        Some(path)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (session_id, todos, scope);
        None
    }
}

/// 读取指定 scope 下的会话索引列表。
pub fn load_sessions_scoped(_data_dir: &Path, scope: Option<&str>) -> Vec<ChatSessionMeta> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(conn) = open_sessions_connection(_data_dir, scope) else {
            return vec![];
        };
        load_sessions_index_from_sqlite(&conn).unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = scope;
        vec![]
    }
}

/// 覆盖保存指定 scope 下的会话索引列表。
pub fn save_sessions_scoped(
    _data_dir: &Path,
    sessions: &[ChatSessionMeta],
    scope: Option<&str>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(mut conn) = open_sessions_connection(_data_dir, scope) else {
            return;
        };
        let _ = save_sessions_index_to_sqlite(&mut conn, sessions);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (sessions, scope);
    }
}

/// 读取指定 scope 下的完整会话内容。
pub fn load_session_scoped(data_dir: &Path, id: &str, scope: Option<&str>) -> Option<ChatSession> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(conn) = open_sessions_connection(data_dir, scope) else {
            return None;
        };
        load_session_from_sqlite(&conn, id).ok().flatten()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let dir = path::sessions_dir_for_scope(data_dir, scope)?;
        let path = dir.join(format!("session-{id}.json"));
        let Ok(content) = std::fs::read_to_string(&path) else {
            return None;
        };
        serde_json::from_str::<ChatSession>(&content).ok()
    }
}

/// 在任意可访问 scope 中查找指定完整会话内容。
pub fn load_session_any(data_dir: &Path, id: &str, scope: Option<&str>) -> Option<ChatSession> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(session) = load_session_scoped(data_dir, id, scope) {
            return Some(session);
        }

        let current_db_path = path::sessions_db_path_for_scope(data_dir, scope);
        for db_path in session_scope_db_paths(data_dir) {
            if current_db_path.as_ref() == Some(&db_path) {
                continue;
            }
            let Ok(conn) = Connection::open(&db_path).map_err(sqlite_to_io_error) else {
                continue;
            };
            if let Ok(Some(session)) = load_session_from_sqlite(&conn, id) {
                return Some(session);
            }
        }

        None
    }

    #[cfg(target_arch = "wasm32")]
    {
        load_session_scoped(data_dir, id, scope)
    }
}

/// 保存指定 scope 下的完整会话内容。
pub fn save_session_scoped(
    data_dir: &Path,
    session: &ChatSession,
    scope: Option<&str>,
) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let db_path = path::sessions_db_path_for_scope(data_dir, scope)?;
        let mut conn = open_sessions_connection(data_dir, scope).ok()?;
        save_session_to_sqlite(&mut conn, session).ok()?;
        Some(db_path)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let dir = path::sessions_dir_for_scope(data_dir, scope)?;
        let path = dir.join(format!("session-{}.json", session.id));
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(session).ok()?;
        let _ = std::fs::write(&path, content);
        Some(path)
    }
}

/// 删除指定 scope 下的会话内容。
pub fn delete_session_scoped(data_dir: &Path, id: &str, scope: Option<&str>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(mut conn) = open_sessions_connection(data_dir, scope) else {
            return;
        };
        let _ = delete_session_from_sqlite(&mut conn, id);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let dir = path::sessions_dir_for_scope(data_dir, scope);
        let Some(dir) = dir else { return };
        let path = dir.join(format!("session-{id}.json"));
        let _ = std::fs::remove_file(path);
    }
}

/// 读取指定 scope 下的单个代理会话元数据。
pub fn load_agent_session_scoped(
    _data_dir: &Path,
    session_id: &str,
    scope: Option<&str>,
) -> Option<Info> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(conn) = open_sessions_connection(_data_dir, scope) else {
            return None;
        };
        load_agent_session_from_sqlite(&conn, session_id).ok().flatten()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (session_id, scope);
        None
    }
}

/// 读取指定 scope 下全部代理会话元数据。
pub fn load_agent_sessions_scoped(_data_dir: &Path, scope: Option<&str>) -> Vec<Info> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(conn) = open_sessions_connection(_data_dir, scope) else {
            return Vec::new();
        };
        load_agent_sessions_from_sqlite(&conn).unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = scope;
        Vec::new()
    }
}

/// 保存指定 scope 下的代理会话元数据。
pub fn save_agent_session_scoped(
    _data_dir: &Path,
    info: &Info,
    scope: Option<&str>,
) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let db_path = path::sessions_db_path_for_scope(_data_dir, scope)?;
        let mut conn = open_sessions_connection(_data_dir, scope).ok()?;
        save_agent_session_to_sqlite(&mut conn, info).ok()?;
        Some(db_path)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (info, scope);
        None
    }
}

/// 删除指定 scope 下的代理会话元数据。
pub fn delete_agent_session_scoped(_data_dir: &Path, id: &str, scope: Option<&str>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(mut conn) = open_sessions_connection(_data_dir, scope) else {
            return;
        };
        let _ = delete_agent_session_from_sqlite(&mut conn, id);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (id, scope);
    }
}

/// 读取当前及其他 scope 中可见的全部代理会话元数据。
pub fn load_agent_sessions_all(_data_dir: &Path, scope: Option<&str>) -> Vec<Info> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let current_db_path = path::sessions_db_path_for_scope(_data_dir, scope);

        if let Some(scope_str) = scope {
            for info in load_agent_sessions_scoped(_data_dir, Some(scope_str)) {
                if seen.insert(info.id.clone()) {
                    out.push(info);
                }
            }
        }

        for db_path in session_scope_db_paths(_data_dir) {
            if current_db_path.as_ref() == Some(&db_path) {
                continue;
            }
            let Ok(conn) = Connection::open(&db_path).map_err(sqlite_to_io_error) else {
                continue;
            };
            let Ok(sessions) = load_agent_sessions_from_sqlite(&conn) else {
                continue;
            };
            for info in sessions {
                if seen.insert(info.id.clone()) {
                    out.push(info);
                }
            }
        }

        out.sort_by(|a, b| b.time.updated.cmp(&a.time.updated).then_with(|| b.id.cmp(&a.id)));
        out
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = scope;
        Vec::new()
    }
}

/// 在任意可访问 scope 中查找指定代理会话。
pub fn load_agent_session_any(
    _data_dir: &Path,
    session_id: &str,
    _scope: Option<&str>,
) -> Option<Info> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(info) = load_agent_session_scoped(_data_dir, session_id, _scope) {
            return Some(info);
        }
        for db_path in session_scope_db_paths(_data_dir) {
            let Ok(conn) = Connection::open(&db_path).map_err(sqlite_to_io_error) else {
                continue;
            };
            if let Ok(Some(info)) = load_agent_session_from_sqlite(&conn, session_id) {
                return Some(info);
            }
        }
        None
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = session_id;
        None
    }
}

/// 读取会话预览所需的列表元数据。
pub fn session_preview_meta(
    data_dir: &Path,
    id: &str,
    scope: Option<&str>,
) -> Option<ChatSessionMeta> {
    load_sessions_scoped(data_dir, scope).into_iter().find(|meta| meta.id == id)
}

#[cfg(test)]
#[path = "ui_store_tests.rs"]
mod ui_store_tests;
