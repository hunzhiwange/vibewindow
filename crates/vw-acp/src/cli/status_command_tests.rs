use std::collections::HashMap;

use serde_json::Value;

use crate::cli::flags::GlobalFlags;
use crate::{
    AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, SESSION_RECORD_SCHEMA,
    SessionAcpxState, SessionEventLog, SessionRecord, SessionTokenUsage,
};

use super::*;

fn flags(format: OutputFormat) -> GlobalFlags {
    GlobalFlags {
        agent: None,
        cwd: "/tmp/repo".to_string(),
        auth_policy: Some(AuthPolicy::Skip),
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        json_strict: false,
        suppress_reads: false,
        timeout: None,
        ttl: 30_000,
        verbose: false,
        format,
        model: None,
        allowed_tools: None,
        max_turns: None,
        prompt_retries: None,
        approve_all: false,
        approve_reads: false,
        deny_all: false,
    }
}

fn record() -> SessionRecord {
    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "session-1".to_string(),
        agent_session_id: Some("agent-session-1".to_string()),
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/repo".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:02:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/repo/active.jsonl".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 4,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: Some(42),
        agent_started_at: Some("2999-01-01T00:00:00Z".to_string()),
        last_prompt_at: Some("2026-01-01T00:01:00Z".to_string()),
        last_agent_exit_code: Some(7),
        last_agent_exit_signal: Some("TERM".to_string()),
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: None,
        messages: Vec::new(),
        updated_at: "2026-01-01T00:03:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: Some(SessionAcpxState {
            current_mode_id: Some("focus".to_string()),
            desired_mode_id: Some("focus".to_string()),
            current_model_id: Some("model-a".to_string()),
            available_models: Some(vec!["model-a".to_string(), "model-b".to_string()]),
            available_commands: None,
            config_options: None,
            session_options: None,
        }),
    }
}

#[test]
fn format_uptime_handles_missing_invalid_and_future_timestamps() {
    assert!(format_uptime(None).is_none());
    assert!(format_uptime(Some("not-a-date")).is_none());
    assert_eq!(format_uptime(Some("2999-01-01T00:00:00Z")), Some("00:00:00".to_string()));
}

#[test]
fn build_status_payload_maps_running_and_dead_fields() {
    let record = record();
    let running = build_status_payload(&record, true, Some(123));
    let dead = build_status_payload(&record, false, None);

    assert_eq!(running.status, "running");
    assert_eq!(running.pid, Some(123));
    assert_eq!(running.model, Some("model-a".to_string()));
    assert!(running.exit_code.is_none());
    assert_eq!(dead.status, "dead");
    assert_eq!(dead.exit_code, Some(7));
    assert_eq!(dead.signal, Some("TERM".to_string()));
}

#[test]
fn write_no_session_status_supports_quiet_text_and_json() {
    let mut quiet = Vec::new();
    write_no_session_status(&mut quiet, &flags(OutputFormat::Quiet), "agent").unwrap();
    assert_eq!(String::from_utf8(quiet).unwrap(), "no-session\n");

    let mut text = Vec::new();
    write_no_session_status(&mut text, &flags(OutputFormat::Text), "agent").unwrap();
    assert!(String::from_utf8(text).unwrap().contains("status: no-session"));

    let mut json = Vec::new();
    write_no_session_status(&mut json, &flags(OutputFormat::Json), "agent").unwrap();
    let value: Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(value["status"], "no-session");
}

#[test]
fn write_status_payload_supports_quiet_text_and_json() {
    let record = record();
    let payload = build_status_payload(&record, false, None);

    let mut quiet = Vec::new();
    write_status_payload(&mut quiet, &flags(OutputFormat::Quiet), &record, &payload, false)
        .unwrap();
    assert_eq!(String::from_utf8(quiet).unwrap(), "dead\n");

    let mut text = Vec::new();
    write_status_payload(&mut text, &flags(OutputFormat::Text), &record, &payload, false).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(text.contains("agentSessionId: agent-session-1"));
    assert!(text.contains("exitCode: 7"));

    let mut json = Vec::new();
    write_status_payload(&mut json, &flags(OutputFormat::Json), &record, &payload, false).unwrap();
    let value: Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(value["status"], "dead");
    assert_eq!(value["agentSessionId"], "agent-session-1");
}
