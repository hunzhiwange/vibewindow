use super::*;
use crate::{SESSION_RECORD_SCHEMA, SessionEventLog, SessionTokenUsage};

fn test_record() -> SessionRecord {
    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "acp-1".to_string(),
        agent_session_id: None,
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:00:00Z".to_string(),
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

#[test]
fn normalize_mode_id_trims_and_rejects_empty_values() {
    assert_eq!(normalize_mode_id(Some("  default ")).as_deref(), Some("default"));
    assert_eq!(normalize_mode_id(Some("  ")), None);
    assert_eq!(normalize_mode_id(None), None);
}

#[test]
fn set_desired_mode_id_creates_state_and_clears_blank_values() {
    let mut record = test_record();

    set_desired_mode_id(&mut record, Some("  plan "));
    assert_eq!(get_desired_mode_id(record.vwacp.as_ref()).as_deref(), Some("plan"));

    set_desired_mode_id(&mut record, Some(" "));
    assert_eq!(get_desired_mode_id(record.vwacp.as_ref()), None);
    assert!(record.vwacp.is_some());
}

#[test]
fn set_desired_model_id_preserves_other_session_options_and_drops_empty_options() {
    let mut record = test_record();

    set_desired_model_id(&mut record, Some(" gpt "));
    assert_eq!(get_desired_model_id(record.vwacp.as_ref()).as_deref(), Some("gpt"));

    let session_options = record.vwacp.as_mut().unwrap().session_options.as_mut().unwrap();
    session_options.allowed_tools = Some(vec!["shell".to_string()]);
    set_desired_model_id(&mut record, None);

    let session_options = record.vwacp.as_ref().unwrap().session_options.as_ref().unwrap();
    assert_eq!(session_options.model, None);
    assert_eq!(session_options.allowed_tools.as_deref(), Some(&["shell".to_string()][..]));

    record.vwacp.as_mut().unwrap().session_options.as_mut().unwrap().allowed_tools = None;
    set_desired_model_id(&mut record, None);
    assert_eq!(record.vwacp.as_ref().unwrap().session_options, None);
}

#[test]
fn set_current_model_id_normalizes_current_model() {
    let mut record = test_record();

    set_current_model_id(&mut record, Some(" model-a "));
    assert_eq!(record.vwacp.as_ref().unwrap().current_model_id.as_deref(), Some("model-a"));

    set_current_model_id(&mut record, Some(""));
    assert_eq!(record.vwacp.as_ref().unwrap().current_model_id, None);
}
