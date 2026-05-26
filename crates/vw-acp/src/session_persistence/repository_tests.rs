use super::*;

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vw-acp-repository-{name}-{}", std::process::id()))
}

#[test]
fn percent_encode_component_keeps_safe_bytes_and_encodes_path_separators() {
    assert_eq!(percent_encode_component("abc-_.!~*'()"), "abc-_.!~*'()");
    assert_eq!(percent_encode_component("a/b c"), "a%2Fb%20c");
}

#[test]
fn session_file_path_percent_encodes_record_id() {
    let path = session_file_path(Path::new("/tmp/sessions"), "record/1");

    assert_eq!(path, Path::new("/tmp/sessions").join("record%2F1.json"));
}

#[test]
fn absolute_path_removes_dot_and_parent_components() {
    let absolute = absolute_path("/tmp/project/./src/../Cargo.toml");

    assert_eq!(absolute, PathBuf::from("/tmp/project/Cargo.toml"));
}

#[test]
fn normalize_name_trims_and_rejects_empty_input() {
    assert_eq!(normalize_name(Some("  main ")).as_deref(), Some("main"));
    assert_eq!(normalize_name(Some("  ")), None);
    assert_eq!(normalize_name(None), None);
}

#[tokio::test]
async fn load_session_index_entries_reads_entries_from_index() {
    let dir = temp_dir("entries");
    tokio::fs::create_dir_all(&dir).await.expect("create dir");
    let index = crate::SessionIndex {
        files: vec!["record.json".to_string()],
        entries: vec![crate::SessionIndexEntry {
            file: "record.json".to_string(),
            vwacp_record_id: "record".to_string(),
            acp_session_id: "acp".to_string(),
            agent_command: "agent".to_string(),
            cwd: "/tmp/project".to_string(),
            name: None,
            closed: false,
            last_used_at: "2026-01-01T00:00:00Z".to_string(),
        }],
    };
    let record = crate::SessionRecord {
        schema: crate::SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record".to_string(),
        acp_session_id: "acp".to_string(),
        agent_session_id: None,
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:00:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: crate::SessionEventLog {
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
        cumulative_token_usage: crate::SessionTokenUsage::default(),
        request_token_usage: std::collections::HashMap::new(),
        vwacp: None,
    };
    let record_payload = crate::serialize_session_record_for_disk(&record);
    tokio::fs::write(dir.join("record.json"), serde_json::to_vec(&record_payload).expect("json"))
        .await
        .expect("write record");
    crate::write_session_index(&dir, &index).await.expect("write index");

    let entries = load_session_index_entries(&dir).await.expect("load entries");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].vwacp_record_id, "record");

    let _ = tokio::fs::remove_dir_all(&dir).await;
}
