//! 会话 UI 存储适配层，负责把桌面会话、快照、原始模型调用和归档状态落盘。

use crate::project;
use crate::session::session as agent_session;
use crate::session::ui_config::{load_app_config, set_config_field};
/// 当前公开项是模块对外契约的一部分。
pub use crate::session::ui_types::{
    ChatMessage, ChatRole, ChatSession, ChatSessionMeta, ChatSessionStep, SessionTodoItem,
    ThinkTiming, TokenUsage,
};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::Mutex;

use vw_shared::session::path;
use vw_shared::session::ui_store;

static SESSION_SCOPE: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));
static SESSION_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(resolve_session_data_dir);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopeProjectMetadata {
    scope: String,
    scope_key: String,
    project_id: String,
    directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vcs: Option<project::Vcs>,
    sandboxes: Vec<String>,
    time: project::TimeInfo,
}

impl ScopeProjectMetadata {
    fn from_project(scope: &str, info: &project::Info) -> Self {
        Self {
            scope: scope.to_string(),
            scope_key: path::session_scope_key(scope),
            project_id: info.id.clone(),
            directory: info.worktree.clone(),
            name: info.name.clone(),
            vcs: info.vcs.clone(),
            sandboxes: info.sandboxes.clone(),
            time: info.time.clone(),
        }
    }
}

fn session_scope() -> Option<String> {
    SESSION_SCOPE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// 执行 current_session_scope 操作，并返回调用方需要的结果。
pub fn current_session_scope() -> Option<String> {
    session_scope()
}

/// 执行 set_session_scope 操作，并返回调用方需要的结果。
pub fn set_session_scope(path: Option<&str>) {
    let mut lock = SESSION_SCOPE.lock().unwrap_or_else(|e| e.into_inner());
    *lock = path.map(|s| s.to_string());
}

fn data_dir() -> &'static Path {
    SESSION_DATA_DIR.as_path()
}

fn resolve_session_data_dir() -> PathBuf {
    let paths = crate::global::paths();
    let session_data_dir = paths.home.join(".vibewindow");
    migrate_legacy_session_storage_if_needed(&paths.data, &session_data_dir);
    let _ = std::fs::create_dir_all(session_data_dir.join("storage").join("session"));
    backfill_scope_project_metadata(&session_data_dir, &paths.data.join("storage").join("project"));
    tracing::info!(
        target: "vw_agent",
        session_data_dir = %session_data_dir.display(),
        legacy_data_dir = %paths.data.display(),
        "session ui data directory ready"
    );
    session_data_dir
}

fn migrate_legacy_session_storage_if_needed(legacy_data_dir: &Path, session_data_dir: &Path) {
    let legacy_session_dir = legacy_data_dir.join("storage").join("session");
    let target_session_dir = session_data_dir.join("storage").join("session");
    if legacy_session_dir == target_session_dir
        || !legacy_session_dir.exists()
        || target_session_dir.exists()
    {
        return;
    }

    let Some(parent) = target_session_dir.parent() else {
        return;
    };
    if let Err(error) = std::fs::create_dir_all(parent) {
        tracing::warn!(
            target: "vw_agent",
            legacy_session_dir = %legacy_session_dir.display(),
            target_session_dir = %target_session_dir.display(),
            error = %error,
            "failed to create target directory for session ui migration"
        );
        return;
    }

    match std::fs::rename(&legacy_session_dir, &target_session_dir) {
        Ok(()) => {
            tracing::info!(
                target: "vw_agent",
                legacy_session_dir = %legacy_session_dir.display(),
                target_session_dir = %target_session_dir.display(),
                "migrated legacy session ui storage to ~/.vibewindow"
            );
            return;
        }
        Err(rename_error) => {
            if let Err(copy_error) = copy_dir_recursive(&legacy_session_dir, &target_session_dir) {
                tracing::warn!(
                    target: "vw_agent",
                    legacy_session_dir = %legacy_session_dir.display(),
                    target_session_dir = %target_session_dir.display(),
                    rename_error = %rename_error,
                    copy_error = %copy_error,
                    "failed to migrate legacy session ui storage"
                );
                return;
            }
            tracing::info!(
                target: "vw_agent",
                legacy_session_dir = %legacy_session_dir.display(),
                target_session_dir = %target_session_dir.display(),
                rename_error = %rename_error,
                "copied legacy session ui storage to ~/.vibewindow"
            );
        }
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
            continue;
        }
        if file_type.is_file() {
            std::fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn project_storage_file_path(project_id: &str) -> PathBuf {
    crate::global::paths().data.join("storage").join("project").join(format!("{project_id}.json"))
}

fn load_project_info(project_id: &str) -> Option<project::Info> {
    let path = project_storage_file_path(project_id);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<project::Info>(&content).ok()
}

fn scope_project_metadata_path(data_dir: &Path, scope: &str) -> Option<PathBuf> {
    Some(path::sessions_dir_for_scope(data_dir, Some(scope))?.join("project.json"))
}

fn write_scope_project_metadata(data_dir: &Path, scope: &str, info: &project::Info) -> Option<PathBuf> {
    let metadata_path = scope_project_metadata_path(data_dir, scope)?;
    if let Some(parent) = metadata_path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    let metadata = ScopeProjectMetadata::from_project(scope, info);
    let content = serde_json::to_string_pretty(&metadata).ok()?;
    std::fs::write(&metadata_path, content).ok()?;
    Some(metadata_path)
}

fn ensure_scope_project_metadata(scope: Option<&str>) {
    let Some(scope) = scope.map(str::trim).filter(|scope| !scope.is_empty()) else {
        return;
    };
    let Some(info) = load_project_info(scope) else {
        return;
    };
    let _ = write_scope_project_metadata(data_dir(), scope, &info);
}

fn backfill_scope_project_metadata(session_data_dir: &Path, project_storage_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(project_storage_dir) else {
        return;
    };
    let mut written = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(info) = serde_json::from_str::<project::Info>(&content) else {
            continue;
        };
        if write_scope_project_metadata(session_data_dir, &info.id, &info).is_some() {
            written += 1;
        }
    }
    if written > 0 {
        tracing::info!(
            target: "vw_agent",
            written,
            project_storage_dir = %project_storage_dir.display(),
            "backfilled scoped project metadata files"
        );
    }
}

/// 执行 session_file_path 操作，并返回调用方需要的结果。
pub fn session_file_path(id: &str) -> Option<PathBuf> {
    path::session_file_path(data_dir(), id, session_scope().as_deref())
}

/// 执行 session_step_snapshot_file_path 操作，并返回调用方需要的结果。
pub fn session_step_snapshot_file_path(
    session_id: &str,
    step_index: u32,
    kind: &str,
) -> Option<PathBuf> {
    path::session_step_snapshot_file_path(
        data_dir(),
        session_id,
        step_index,
        kind,
        session_scope().as_deref(),
    )
}

/// 执行 session_step_llm_raw_file_path 操作，并返回调用方需要的结果。
pub fn session_step_llm_raw_file_path(session_id: &str, step_index: u32) -> Option<PathBuf> {
    path::session_step_llm_raw_file_path(
        data_dir(),
        session_id,
        step_index,
        session_scope().as_deref(),
    )
}

/// 执行 session_step_llm_raw_file_path_scoped 操作，并返回调用方需要的结果。
pub fn session_step_llm_raw_file_path_scoped(
    session_id: &str,
    step_index: u32,
    scope: Option<&str>,
) -> Option<PathBuf> {
    path::session_step_llm_raw_file_path_scoped(data_dir(), session_id, step_index, scope)
}

/// 执行 save_session_step_snapshot 操作，并返回调用方需要的结果。
pub fn save_session_step_snapshot(
    session: &ChatSession,
    step_index: u32,
    kind: &str,
) -> Option<PathBuf> {
    ui_store::save_session_step_snapshot(
        data_dir(),
        session,
        step_index,
        kind,
        session_scope().as_deref(),
    )
}

/// 执行 persist_llm_raw_step 操作，并返回调用方需要的结果。
pub fn persist_llm_raw_step(
    session_id: &str,
    step_index: u32,
    payload: &serde_json::Value,
    scope: Option<&str>,
) -> Option<PathBuf> {
    ui_store::persist_llm_raw_step(data_dir(), session_id, step_index, payload, scope)
}

/// 执行 resolve_session_scope_id 操作，并返回调用方需要的结果。
pub fn resolve_session_scope_id(path: Option<&str>, project_id: Option<&str>) -> Option<String> {
    if let Some(id) = project_id.map(str::trim).filter(|s| !s.is_empty()) {
        ensure_scope_project_metadata(Some(id));
        return Some(id.to_string());
    }
    let path = path.map(str::trim).filter(|s| !s.is_empty())?;
    let path_buf = PathBuf::from(path);

    #[cfg(target_arch = "wasm32")]
    {
        let _ = path_buf;
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let fut = async move { project::from_directory(path_buf).await.ok() };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return tokio::task::block_in_place(|| handle.block_on(fut)).map(|(info, _)| {
                let scope = info.id.clone();
                let _ = write_scope_project_metadata(data_dir(), &scope, &info);
                scope
            });
        }

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()
            .and_then(|rt| rt.block_on(fut))
            .map(|(info, _)| {
                let scope = info.id.clone();
                let _ = write_scope_project_metadata(data_dir(), &scope, &info);
                scope
            })
    }
}

/// 执行 load_session_todos 操作，并返回调用方需要的结果。
pub fn load_session_todos(session_id: &str) -> Vec<SessionTodoItem> {
    ui_store::load_session_todos_scoped(data_dir(), session_id, session_scope().as_deref())
}

/// 执行 load_session_todos_scoped 操作，并返回调用方需要的结果。
pub fn load_session_todos_scoped(session_id: &str, scope: Option<&str>) -> Vec<SessionTodoItem> {
    ui_store::load_session_todos_scoped(data_dir(), session_id, scope)
}

/// 执行 save_session_todos 操作，并返回调用方需要的结果。
pub fn save_session_todos(session_id: &str, todos: &[SessionTodoItem]) -> Option<PathBuf> {
    ui_store::save_session_todos_scoped(data_dir(), session_id, todos, session_scope().as_deref())
}

/// 执行 save_session_todos_scoped 操作，并返回调用方需要的结果。
pub fn save_session_todos_scoped(
    session_id: &str,
    todos: &[SessionTodoItem],
    scope: Option<&str>,
) -> Option<PathBuf> {
    ui_store::save_session_todos_scoped(data_dir(), session_id, todos, scope)
}

/// 执行 load_sessions_scoped 操作，并返回调用方需要的结果。
pub fn load_sessions_scoped(scope: Option<&str>) -> Vec<ChatSessionMeta> {
    ui_store::load_sessions_scoped(data_dir(), scope)
}

/// 执行 load_sessions 操作，并返回调用方需要的结果。
pub fn load_sessions() -> Vec<ChatSessionMeta> {
    ui_store::load_sessions_scoped(data_dir(), session_scope().as_deref())
}

/// 执行 load_archived_session_ids_scoped 操作，并返回调用方需要的结果。
pub fn load_archived_session_ids_scoped(scope: Option<&str>) -> HashSet<String> {
    let cfg = load_app_config();
    let key = if let Some(scope) = scope {
        format!("archived_session_ids:{}", path::session_scope_key(scope))
    } else {
        "archived_session_ids".to_string()
    };
    cfg.get(&key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<HashSet<String>>()
        })
        .unwrap_or_default()
}

/// 执行 load_archived_session_ids 操作，并返回调用方需要的结果。
pub fn load_archived_session_ids() -> HashSet<String> {
    load_archived_session_ids_scoped(session_scope().as_deref())
}

/// 执行 save_archived_session_ids 操作，并返回调用方需要的结果。
pub fn save_archived_session_ids(ids: &HashSet<String>) {
    save_archived_session_ids_scoped(ids, session_scope().as_deref());
}

/// 执行 save_archived_session_ids_scoped 操作，并返回调用方需要的结果。
pub fn save_archived_session_ids_scoped(ids: &HashSet<String>, scope: Option<&str>) {
    let arr = serde_json::Value::Array(
        ids.iter().map(|v| serde_json::Value::String(v.clone())).collect(),
    );
    let key = if let Some(scope) = scope {
        format!("archived_session_ids:{}", path::session_scope_key(scope))
    } else {
        "archived_session_ids".to_string()
    };
    set_config_field(&key, arr);
}

/// 执行 save_sessions_scoped 操作，并返回调用方需要的结果。
pub fn save_sessions_scoped(sessions: &[ChatSessionMeta], scope: Option<&str>) {
    ensure_scope_project_metadata(scope);
    ui_store::save_sessions_scoped(data_dir(), sessions, scope)
}

/// 执行 save_sessions 操作，并返回调用方需要的结果。
pub fn save_sessions(sessions: &[ChatSessionMeta]) {
    ui_store::save_sessions_scoped(data_dir(), sessions, session_scope().as_deref())
}

/// 执行 load_session_scoped 操作，并返回调用方需要的结果。
pub fn load_session_scoped(id: &str, scope: Option<&str>) -> Option<ChatSession> {
    ui_store::load_session_scoped(data_dir(), id, scope)
}

/// 执行 load_session 操作，并返回调用方需要的结果。
pub fn load_session(id: &str) -> Option<ChatSession> {
    ui_store::load_session_scoped(data_dir(), id, session_scope().as_deref())
}

/// 执行 load_session_any 操作，并返回调用方需要的结果。
pub fn load_session_any(id: &str) -> Option<ChatSession> {
    ui_store::load_session_any(data_dir(), id, session_scope().as_deref())
}

/// 执行 load_agent_session_scoped 操作，并返回调用方需要的结果。
pub fn load_agent_session_scoped(
    session_id: &str,
    scope: Option<&str>,
) -> Option<agent_session::Info> {
    ui_store::load_agent_session_scoped(data_dir(), session_id, scope)
}

/// 执行 load_agent_sessions_scoped 操作，并返回调用方需要的结果。
pub fn load_agent_sessions_scoped(scope: Option<&str>) -> Vec<agent_session::Info> {
    ui_store::load_agent_sessions_scoped(data_dir(), scope)
}

/// 执行 load_agent_sessions_all 操作，并返回调用方需要的结果。
pub fn load_agent_sessions_all() -> Vec<agent_session::Info> {
    ui_store::load_agent_sessions_all(data_dir(), session_scope().as_deref())
}

/// 执行 load_agent_session_any 操作，并返回调用方需要的结果。
pub fn load_agent_session_any(session_id: &str) -> Option<agent_session::Info> {
    ui_store::load_agent_session_any(data_dir(), session_id, session_scope().as_deref())
}

/// 执行 session_preview_meta 操作，并返回调用方需要的结果。
pub fn session_preview_meta(id: &str) -> Option<ChatSessionMeta> {
    ui_store::session_preview_meta(data_dir(), id, session_scope().as_deref())
}

/// 执行 save_session 操作，并返回调用方需要的结果。
pub fn save_session(session: &ChatSession) -> Option<PathBuf> {
    ui_store::save_session_scoped(data_dir(), session, session_scope().as_deref())
}

/// 执行 save_agent_session_scoped 操作，并返回调用方需要的结果。
pub fn save_agent_session_scoped(
    info: &agent_session::Info,
    scope: Option<&str>,
) -> Option<PathBuf> {
    ensure_scope_project_metadata(scope.or(Some(&info.project_id)));
    ui_store::save_agent_session_scoped(data_dir(), info, scope)
}

/// 执行 save_session_scoped 操作，并返回调用方需要的结果。
pub fn save_session_scoped(session: &ChatSession, scope: Option<&str>) -> Option<PathBuf> {
    ensure_scope_project_metadata(scope);
    ui_store::save_session_scoped(data_dir(), session, scope)
}

/// 执行 delete_session 操作，并返回调用方需要的结果。
pub fn delete_session(id: &str) {
    ui_store::delete_session_scoped(data_dir(), id, session_scope().as_deref());
}

/// 执行 delete_agent_session_scoped 操作，并返回调用方需要的结果。
pub fn delete_agent_session_scoped(id: &str, scope: Option<&str>) {
    ui_store::delete_agent_session_scoped(data_dir(), id, scope);
}

/// 执行 delete_session_scoped 操作，并返回调用方需要的结果。
pub fn delete_session_scoped(id: &str, scope: Option<&str>) {
    ui_store::delete_session_scoped(data_dir(), id, scope);
}

/// 执行 append_session_call 操作，并返回调用方需要的结果。
pub fn append_session_call(session: &str, payload: &serde_json::Value) -> Option<PathBuf> {
    let mut s = load_session(session).unwrap_or(ChatSession {
        id: session.to_string(),
        title: "新会话".to_string(),
        messages: vec![],
        message_ids: vec![],
        calls: vec![],
        steps: vec![],
        created_ms: agent_session::now_ms(),
        updated_ms: 0,
    });
    s.calls.push(payload.clone());
    s.updated_ms = agent_session::now_ms();
    save_session(&s)
}

/// 执行 persist_ai_call_payload 操作，并返回调用方需要的结果。
pub fn persist_ai_call_payload(
    session: &str,
    stream: u64,
    payload: &serde_json::Value,
    scope: Option<&str>,
) -> Option<PathBuf> {
    let mut s = load_session_scoped(session, scope).unwrap_or(ChatSession {
        id: session.to_string(),
        title: "新会话".to_string(),
        messages: vec![],
        message_ids: vec![],
        calls: vec![],
        steps: vec![],
        created_ms: agent_session::now_ms(),
        updated_ms: 0,
    });
    let stream_id =
        payload.get("stream_id").and_then(serde_json::Value::as_u64).or(Some(stream)).unwrap_or(0);
    if stream_id != 0 {
        if let Some(pos) = s.calls.iter().rposition(|c| {
            c.get("stream_id")
                .and_then(serde_json::Value::as_u64)
                .map(|id| id == stream_id)
                .unwrap_or(false)
        }) {
            s.calls[pos] = payload.clone();
        } else {
            s.calls.push(payload.clone());
        }
    } else {
        s.calls.push(payload.clone());
    }
    s.updated_ms = agent_session::now_ms();
    save_session_scoped(&s, scope)
}

/// 执行 save_session_snapshot 操作，并返回调用方需要的结果。
pub fn save_session_snapshot(session: &ChatSession) -> Option<PathBuf> {
    save_session(session)
}
#[cfg(test)]
#[path = "ui_store_tests.rs"]
mod ui_store_tests;
