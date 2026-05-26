use super::*;
use crate::{SESSION_RECORD_SCHEMA, SessionEventLog, SessionTokenUsage};

fn test_record(id: &str, last_used_at: &str) -> SessionRecord {
    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: id.to_string(),
        acp_session_id: format!("acp-{id}"),
        agent_session_id: None,
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: Some("main".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: last_used_at.to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/active.ndjson".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 3,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: None,
        agent_started_at: None,
        last_prompt_at: None,
        last_agent_exit_code: None,
        last_agent_exit_signal: None,
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: None,
        messages: Vec::new(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: std::collections::HashMap::new(),
        vwacp: None,
    }
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vw-acp-index-{name}-{}", std::process::id()))
}

#[test]
fn parse_index_entry_requires_typed_required_fields_and_allows_missing_name() {
    let entry = parse_index_entry(&serde_json::json!({
        "file": "record.json",
        "vwacpRecordId": "record",
        "acpSessionId": "acp",
        "agentCommand": "agent",
        "cwd": "/tmp/project",
        "closed": false,
        "lastUsedAt": "2026-01-02T00:00:00Z"
    }))
    .expect("valid entry");

    assert_eq!(entry.name, None);
    assert_eq!(entry.file, "record.json");
    assert!(parse_index_entry(&serde_json::json!({"file": 1})).is_none());
}

#[test]
fn to_session_index_entry_copies_stable_record_fields() {
    let record = test_record("record-1", "2026-01-02T00:00:00Z");
    let entry = to_session_index_entry(&record, "record-1.json");

    assert_eq!(entry.file, "record-1.json");
    assert_eq!(entry.vwacp_record_id, "record-1");
    assert_eq!(entry.acp_session_id, "acp-record-1");
    assert_eq!(entry.name.as_deref(), Some("main"));
    assert!(!entry.closed);
}

#[tokio::test]
async fn write_and_read_session_index_sorts_files_and_entries() {
    let dir = temp_dir("roundtrip");
    tokio::fs::create_dir_all(&dir).await.expect("create dir");
    let older = to_session_index_entry(&test_record("old", "2026-01-01T00:00:00Z"), "b.json");
    let newer = to_session_index_entry(&test_record("new", "2026-01-02T00:00:00Z"), "a.json");
    let index = SessionIndex {
        files: vec!["b.json".to_string(), "a.json".to_string()],
        entries: vec![older, newer],
    };

    write_session_index(&dir, &index).await.expect("write index");
    let payload = tokio::fs::read_to_string(session_index_path(&dir)).await.expect("read index");
    let read = read_session_index(&dir).await.expect("parse index");

    assert!(payload.contains(SESSION_INDEX_SCHEMA));
    assert_eq!(read.files, vec!["a.json".to_string(), "b.json".to_string()]);
    assert_eq!(read.entries[0].vwacp_record_id, "new");

    let _ = tokio::fs::remove_dir_all(&dir).await;
}
