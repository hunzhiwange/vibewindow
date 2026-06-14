#![cfg(not(target_arch = "wasm32"))]

use super::{
    ChatMessage, ChatRole, ChatSession, ChatSessionMeta, ChatSessionStep, Info, SessionTodoItem,
    ThinkTiming, TokenUsage, delete_agent_session_scoped, delete_session_scoped, json_to_io_error,
    load_agent_session_any, load_agent_session_scoped, load_agent_sessions_all,
    load_agent_sessions_scoped, load_session_any, load_session_scoped, load_session_todos_scoped,
    load_sessions_scoped, open_sessions_connection, persist_llm_raw_step,
    save_agent_session_scoped, save_session_scoped, save_session_step_snapshot,
    save_session_todos_scoped, save_sessions_scoped, session_preview_meta, sqlite_to_io_error,
};
use crate::session::info::TimeInfo;
use rusqlite::{Connection, params};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

fn sample_session(id: &str, updated_ms: u64) -> ChatSession {
    ChatSession {
        id: id.to_string(),
        title: format!("Title {id}"),
        messages: vec![
            ChatMessage {
                role: ChatRole::System,
                content: "system seed".to_string(),
                think_timing: Vec::new(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "   ".to_string(),
                think_timing: vec![ThinkTiming {
                    start_ms: 10,
                    end_ms: Some(20),
                    last_update_ms: 18,
                }],
            },
            ChatMessage {
                role: ChatRole::Assistant,
                content: "assistant answer".to_string(),
                think_timing: vec![ThinkTiming { start_ms: 30, end_ms: None, last_update_ms: 35 }],
            },
            ChatMessage {
                role: ChatRole::Tool,
                content: "tool output".to_string(),
                think_timing: Vec::new(),
            },
        ],
        message_ids: vec![
            Some("msg-system".to_string()),
            None,
            Some("msg-assistant".to_string()),
            Some("msg-tool".to_string()),
        ],
        calls: vec![json!({"name": "shell", "args": {"cmd": "true"}})],
        steps: vec![ChatSessionStep {
            index: 7,
            started_ms: 100,
            finished_ms: Some(130),
            start_snapshot_path: Some("start.json".to_string()),
            finish_snapshot_path: Some("finish.json".to_string()),
            usage: TokenUsage {
                input_tokens: 11,
                output_tokens: 12,
                cached_tokens: 3,
                reasoning_tokens: 4,
            },
            cost_usd: Some(0.25),
            finish_reason: Some("stop".to_string()),
            model: Some("model-a".to_string()),
        }],
        created_ms: 40,
        updated_ms,
    }
}

fn sample_info(id: &str, project_id: &str, updated: u64, parent_id: Option<&str>) -> Info {
    Info {
        id: id.to_string(),
        slug: format!("slug-{id}"),
        project_id: project_id.to_string(),
        directory: "/work".to_string(),
        parent_id: parent_id.map(str::to_string),
        summary: None,
        share: None,
        title: format!("Agent {id}"),
        version: "1".to_string(),
        time: TimeInfo { created: updated - 1, updated, compacting: None, archived: None },
        permission: None,
        revert: None,
    }
}

fn db_path(data_dir: &Path, scope: Option<&str>) -> std::path::PathBuf {
    crate::session::path::sessions_db_path_for_scope(data_dir, scope).unwrap()
}

#[test]
fn error_converters_preserve_io_error_kinds() {
    let sqlite_error = sqlite_to_io_error(rusqlite::Error::InvalidQuery);
    assert_eq!(sqlite_error.kind(), io::ErrorKind::Other);

    let json_error = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let io_error = json_to_io_error(json_error);
    assert_eq!(io_error.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn save_and_load_session_preserves_all_sqlite_fields() {
    let temp = TempDir::new().unwrap();
    let session = sample_session("session-a", 200);
    let path = save_session_scoped(temp.path(), &session, Some("repo")).unwrap();

    assert_eq!(path, db_path(temp.path(), Some("repo")));

    let loaded = load_session_scoped(temp.path(), "session-a", Some("repo")).unwrap();
    assert_eq!(loaded.created_ms, 40);
    assert_eq!(loaded.updated_ms, 200);
    assert_eq!(loaded.messages.len(), 4);
    assert_eq!(loaded.messages[1].role, ChatRole::User);
    assert_eq!(loaded.messages[1].think_timing[0].end_ms, Some(20));
    assert_eq!(loaded.message_ids, session.message_ids);
    assert_eq!(loaded.calls, session.calls);
    assert_eq!(loaded.steps[0].usage.reasoning_tokens, 4);

    let meta = session_preview_meta(temp.path(), "session-a", Some("repo")).unwrap();
    assert_eq!(meta.message_count, 4);
    assert_eq!(meta.last_content.as_deref(), Some("tool output"));
}

#[test]
fn save_sessions_replaces_index_and_loads_sorted_with_limit() {
    let temp = TempDir::new().unwrap();
    let original = vec![ChatSessionMeta {
        id: "old".to_string(),
        title: "Old".to_string(),
        updated_ms: 1,
        message_count: 1,
        call_count: 0,
        last_content: None,
    }];
    save_sessions_scoped(temp.path(), &original, None);

    let sessions: Vec<_> = (0..55)
        .map(|index| ChatSessionMeta {
            id: format!("session-{index:02}"),
            title: format!("Session {index}"),
            updated_ms: index,
            message_count: index as usize,
            call_count: (index % 3) as usize,
            last_content: Some(format!("last {index}")),
        })
        .collect();
    save_sessions_scoped(temp.path(), &sessions, None);

    let loaded = load_sessions_scoped(temp.path(), None);
    assert_eq!(loaded.len(), 50);
    assert_eq!(loaded[0].id, "session-54");
    assert_eq!(loaded[49].id, "session-05");
    assert!(loaded.iter().all(|meta| meta.id != "old"));
    assert_eq!(loaded[0].last_content.as_deref(), Some("last 54"));
}

#[test]
fn load_sessions_defaults_negative_numeric_columns_to_zero() {
    let temp = TempDir::new().unwrap();
    let conn = open_sessions_connection(temp.path(), None).unwrap();
    conn.execute(
        "INSERT INTO session_index (
             session_id, title, created_ms, updated_ms, message_count, call_count, last_content
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params!["bad", "Bad", -9_i64, -8_i64, -7_i64, -6_i64, "body"],
    )
    .unwrap();

    let loaded = load_sessions_scoped(temp.path(), None);
    assert_eq!(loaded[0].updated_ms, 0);
    assert_eq!(loaded[0].message_count, 0);
}

#[test]
fn load_session_returns_none_for_missing_or_invalid_payloads() {
    let temp = TempDir::new().unwrap();
    assert!(load_session_scoped(temp.path(), "missing", Some("repo")).is_none());

    let mut session = sample_session("invalid-think", 10);
    session.messages[0].think_timing.push(ThinkTiming {
        start_ms: 1,
        end_ms: None,
        last_update_ms: 1,
    });
    save_session_scoped(temp.path(), &session, Some("repo"));
    let conn = Connection::open(db_path(temp.path(), Some("repo"))).unwrap();
    conn.execute(
        "UPDATE session_messages SET think_timing_json = 'not-json' WHERE session_id = ?1",
        params!["invalid-think"],
    )
    .unwrap();

    assert!(load_session_scoped(temp.path(), "invalid-think", Some("repo")).is_none());

    let session = sample_session("invalid-call", 11);
    save_session_scoped(temp.path(), &session, Some("repo"));
    conn.execute(
        "UPDATE session_calls SET payload_json = 'not-json' WHERE session_id = ?1",
        params!["invalid-call"],
    )
    .unwrap();
    assert!(load_session_scoped(temp.path(), "invalid-call", Some("repo")).is_none());

    let session = sample_session("invalid-usage", 12);
    save_session_scoped(temp.path(), &session, Some("repo"));
    conn.execute(
        "UPDATE session_steps SET usage_json = 'not-json' WHERE session_id = ?1",
        params!["invalid-usage"],
    )
    .unwrap();
    assert!(load_session_scoped(temp.path(), "invalid-usage", Some("repo")).is_none());
}

#[test]
fn load_session_skips_unknown_roles_and_defaults_negative_numbers() {
    let temp = TempDir::new().unwrap();
    let session = sample_session("mixed", 20);
    save_session_scoped(temp.path(), &session, None);
    let conn = Connection::open(db_path(temp.path(), None)).unwrap();
    conn.execute(
        "UPDATE session_index SET created_ms = -1, updated_ms = -2 WHERE session_id = ?1",
        params!["mixed"],
    )
    .unwrap();
    conn.execute(
        "UPDATE session_messages SET role = 'unknown' WHERE session_id = ?1 AND seq_no = 0",
        params!["mixed"],
    )
    .unwrap();
    conn.execute(
        "UPDATE session_steps SET step_index = -1, started_ms = -3, finished_ms = -4
         WHERE session_id = ?1",
        params!["mixed"],
    )
    .unwrap();

    let loaded = load_session_scoped(temp.path(), "mixed", None).unwrap();
    assert_eq!(loaded.created_ms, 0);
    assert_eq!(loaded.updated_ms, 0);
    assert_eq!(loaded.messages.len(), 3);
    assert_eq!(loaded.message_ids.len(), 3);
    assert_eq!(loaded.steps[0].index, 0);
    assert_eq!(loaded.steps[0].started_ms, 0);
    assert_eq!(loaded.steps[0].finished_ms, None);
}

#[test]
fn open_connection_adds_message_id_column_to_legacy_database() {
    let temp = TempDir::new().unwrap();
    let db_path = db_path(temp.path(), Some("legacy"));
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "CREATE TABLE session_index (
             session_id TEXT PRIMARY KEY,
             title TEXT NOT NULL,
             created_ms INTEGER NOT NULL,
             updated_ms INTEGER NOT NULL,
             message_count INTEGER NOT NULL,
             call_count INTEGER NOT NULL,
             last_content TEXT
         );
         CREATE TABLE session_messages (
             session_id TEXT NOT NULL,
             seq_no INTEGER NOT NULL,
             role TEXT NOT NULL,
             content TEXT NOT NULL,
             think_timing_json TEXT NOT NULL,
             PRIMARY KEY (session_id, seq_no)
         );",
    )
    .unwrap();
    drop(conn);

    let session = sample_session("legacy-session", 50);
    save_session_scoped(temp.path(), &session, Some("legacy"));

    let loaded = load_session_scoped(temp.path(), "legacy-session", Some("legacy")).unwrap();
    assert_eq!(loaded.message_ids[0].as_deref(), Some("msg-system"));
}

#[test]
fn delete_session_removes_session_rows_and_todos_only_for_scope() {
    let temp = TempDir::new().unwrap();
    let session = sample_session("delete-me", 60);
    save_session_scoped(temp.path(), &session, Some("repo"));
    save_session_todos_scoped(
        temp.path(),
        "delete-me",
        &[SessionTodoItem {
            id: "todo-1".to_string(),
            content: "done".to_string(),
            status: "completed".to_string(),
            priority: "high".to_string(),
        }],
        Some("repo"),
    );
    save_session_scoped(temp.path(), &sample_session("delete-me", 61), Some("other"));

    delete_session_scoped(temp.path(), "delete-me", Some("repo"));

    assert!(load_session_scoped(temp.path(), "delete-me", Some("repo")).is_none());
    assert!(load_session_todos_scoped(temp.path(), "delete-me", Some("repo")).is_empty());
    assert!(load_session_scoped(temp.path(), "delete-me", Some("other")).is_some());
}

#[test]
fn scoped_todos_round_trip_and_replace_in_order() {
    let temp = TempDir::new().unwrap();
    let first = vec![
        SessionTodoItem {
            id: "b".to_string(),
            content: "second".to_string(),
            status: "pending".to_string(),
            priority: "low".to_string(),
        },
        SessionTodoItem {
            id: "a".to_string(),
            content: "first".to_string(),
            status: "in_progress".to_string(),
            priority: "high".to_string(),
        },
    ];
    let path = save_session_todos_scoped(temp.path(), "session", &first, Some("repo")).unwrap();
    assert_eq!(path, db_path(temp.path(), Some("repo")));
    assert_eq!(load_session_todos_scoped(temp.path(), "session", Some("repo"))[0].id, "b");

    let second = vec![SessionTodoItem {
        id: "c".to_string(),
        content: "replacement".to_string(),
        status: "done".to_string(),
        priority: "medium".to_string(),
    }];
    save_session_todos_scoped(temp.path(), "session", &second, Some("repo"));

    let loaded = load_session_todos_scoped(temp.path(), "session", Some("repo"));
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "c");
}

#[test]
fn save_snapshot_and_raw_payload_write_pretty_json() {
    let temp = TempDir::new().unwrap();
    let session = sample_session("snap", 70);

    let snapshot =
        save_session_step_snapshot(temp.path(), &session, 2, "finish", Some("repo")).unwrap();
    let raw =
        persist_llm_raw_step(temp.path(), "snap", 2, &json!({"ok": true}), Some("repo")).unwrap();

    assert!(snapshot.ends_with("session-snap-step-2-finish.json"));
    assert!(raw.ends_with("session-snap-step-2-llm_raw.json"));
    assert!(fs::read_to_string(snapshot).unwrap().contains("\n  \"id\": \"snap\""));
    assert!(fs::read_to_string(raw).unwrap().contains("\n  \"ok\": true"));
}

#[test]
fn load_session_any_searches_current_scope_then_other_scopes() {
    let temp = TempDir::new().unwrap();
    save_session_scoped(temp.path(), &sample_session("same", 80), Some("current"));
    save_session_scoped(temp.path(), &sample_session("other-only", 81), Some("other"));
    let scoped_base = temp.path().join("storage/session/scoped");
    fs::write(scoped_base.join("plain-file"), "ignored").unwrap();
    fs::create_dir_all(scoped_base.join("bad/index.sqlite3")).unwrap();

    let current = load_session_any(temp.path(), "same", Some("current")).unwrap();
    let other = load_session_any(temp.path(), "other-only", Some("current")).unwrap();

    assert_eq!(current.updated_ms, 80);
    assert_eq!(other.updated_ms, 81);
    assert!(load_session_any(temp.path(), "missing", Some("current")).is_none());
}

#[test]
fn agent_sessions_round_trip_sort_dedupe_and_delete() {
    let temp = TempDir::new().unwrap();
    let current = sample_info("agent-current", "project-a", 100, Some("parent"));
    let older = sample_info("agent-older", "project-a", 90, None);
    let duplicate_current = sample_info("agent-dup", "project-a", 80, None);
    let duplicate_other = sample_info("agent-dup", "project-a", 120, None);
    let newest_other = sample_info("agent-new", "project-b", 110, None);

    assert_eq!(
        save_agent_session_scoped(temp.path(), &current, Some("current")).unwrap(),
        db_path(temp.path(), Some("current"))
    );
    save_agent_session_scoped(temp.path(), &older, Some("current"));
    save_agent_session_scoped(temp.path(), &duplicate_current, Some("current"));
    save_agent_session_scoped(temp.path(), &duplicate_other, Some("other"));
    save_agent_session_scoped(temp.path(), &newest_other, Some("other"));

    let loaded = load_agent_session_scoped(temp.path(), "agent-current", Some("current")).unwrap();
    assert_eq!(loaded.parent_id.as_deref(), Some("parent"));
    assert_eq!(
        load_agent_session_any(temp.path(), "agent-current", Some("current")).unwrap().id,
        "agent-current"
    );

    let scoped = load_agent_sessions_scoped(temp.path(), Some("current"));
    assert_eq!(
        scoped.iter().map(|info| info.id.as_str()).collect::<Vec<_>>(),
        vec!["agent-current", "agent-older", "agent-dup",]
    );

    let all = load_agent_sessions_all(temp.path(), Some("current"));
    assert_eq!(
        all.iter().map(|info| info.id.as_str()).collect::<Vec<_>>(),
        vec!["agent-new", "agent-current", "agent-older", "agent-dup",]
    );
    assert_eq!(
        load_agent_session_any(temp.path(), "agent-new", Some("current")).unwrap().id,
        "agent-new"
    );
    assert!(load_agent_session_any(temp.path(), "missing", Some("current")).is_none());

    delete_agent_session_scoped(temp.path(), "agent-current", Some("current"));
    assert!(load_agent_session_scoped(temp.path(), "agent-current", Some("current")).is_none());
}

#[test]
fn invalid_agent_session_payloads_are_ignored_by_public_loaders() {
    let temp = TempDir::new().unwrap();
    let info = sample_info("bad-agent", "project", 130, None);
    save_agent_session_scoped(temp.path(), &info, Some("repo"));
    let conn = Connection::open(db_path(temp.path(), Some("repo"))).unwrap();
    conn.execute(
        "UPDATE agent_sessions SET info_json = 'not-json' WHERE session_id = ?1",
        params!["bad-agent"],
    )
    .unwrap();

    assert!(load_agent_session_scoped(temp.path(), "bad-agent", Some("repo")).is_none());
    assert!(load_agent_sessions_scoped(temp.path(), Some("repo")).is_empty());
    assert!(load_agent_sessions_all(temp.path(), Some("repo")).is_empty());
    assert!(load_agent_session_any(temp.path(), "bad-agent", Some("repo")).is_none());
}

#[test]
fn public_loaders_return_empty_values_when_storage_cannot_open() {
    let blocked_data_dir = NamedTempFile::new().unwrap();
    let session = sample_session("blocked", 140);
    let info = sample_info("blocked-agent", "project", 140, None);
    let path = blocked_data_dir.path();

    assert!(open_sessions_connection(path, Some("repo")).is_err());
    assert!(save_session_scoped(path, &session, Some("repo")).is_none());
    assert!(save_agent_session_scoped(path, &info, Some("repo")).is_none());
    assert!(save_session_todos_scoped(path, "blocked", &[], Some("repo")).is_none());
    save_sessions_scoped(path, &[], Some("repo"));
    delete_session_scoped(path, "blocked", Some("repo"));
    delete_agent_session_scoped(path, "blocked-agent", Some("repo"));
    assert!(load_sessions_scoped(path, Some("repo")).is_empty());
    assert!(load_session_scoped(path, "blocked", Some("repo")).is_none());
    assert!(load_session_todos_scoped(path, "blocked", Some("repo")).is_empty());
    assert!(load_agent_session_scoped(path, "blocked-agent", Some("repo")).is_none());
    assert!(load_agent_sessions_scoped(path, Some("repo")).is_empty());
}

#[test]
fn all_scope_loaders_handle_missing_and_unopenable_scope_databases() {
    let temp = TempDir::new().unwrap();

    let scoped_base = temp.path().join("storage/session/scoped");
    fs::create_dir_all(scoped_base.join("bad-session/index.sqlite3")).unwrap();
    fs::create_dir_all(scoped_base.join("bad-agent/index.sqlite3")).unwrap();
    fs::create_dir_all(scoped_base.join("not-sqlite")).unwrap();
    fs::write(scoped_base.join("not-sqlite/index.sqlite3"), "not sqlite").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(scoped_base.join("unopenable")).unwrap();
        let unopenable = scoped_base.join("unopenable/index.sqlite3");
        fs::write(&unopenable, "").unwrap();
        fs::set_permissions(&unopenable, fs::Permissions::from_mode(0o000)).unwrap();
    }

    assert!(load_session_any(temp.path(), "missing", None).is_none());
    assert!(load_agent_sessions_all(temp.path(), None).is_empty());
    assert!(load_agent_session_any(temp.path(), "missing", None).is_none());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let unopenable = scoped_base.join("unopenable/index.sqlite3");
        fs::set_permissions(unopenable, fs::Permissions::from_mode(0o600)).unwrap();
    }
}
