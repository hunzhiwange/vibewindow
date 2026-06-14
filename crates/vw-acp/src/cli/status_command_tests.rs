use std::collections::HashMap;
use std::io::{self, Write};

use serde_json::Value;

use crate::cli::flags::GlobalFlags;
use crate::{
    AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, PermissionMode, ResolvedAcpxConfig,
    SESSION_RECORD_SCHEMA, SessionAcpxState, SessionEventLog, SessionRecord, SessionTokenUsage,
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

fn config() -> ResolvedAcpxConfig {
    ResolvedAcpxConfig {
        default_agent: "agent".to_string(),
        default_permissions: PermissionMode::ApproveReads,
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        auth_policy: AuthPolicy::Skip,
        ttl_ms: 30_000,
        timeout_ms: Some(1_000),
        queue_max_depth: 8,
        format: OutputFormat::Text,
        agents: HashMap::new(),
        auth: HashMap::new(),
        disable_exec: false,
        mcp_servers: Vec::new(),
        global_path: "/tmp/global.json".to_string(),
        project_path: "/tmp/project.json".to_string(),
        has_global_config: false,
        has_project_config: false,
    }
}

fn missing_session_flags(format: OutputFormat) -> GlobalFlags {
    let mut global_flags = flags(format);
    global_flags.cwd = format!("/tmp/vw-acp-status-command-missing-{}", std::process::id());
    global_flags
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

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("write failed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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
fn build_status_payload_omits_optional_fields_without_runtime_state() {
    let mut record = record();
    record.agent_session_id = None;
    record.agent_started_at = None;
    record.last_prompt_at = None;
    record.vwacp = None;

    let payload = build_status_payload(&record, true, None);

    assert_eq!(payload.status, "running");
    assert!(payload.pid.is_none());
    assert!(payload.model.is_none());
    assert!(payload.mode.is_none());
    assert!(payload.available_models.is_none());
    assert!(payload.uptime.is_none());
    assert!(payload.last_prompt_time.is_none());
    assert!(payload.agent_session.agent_session_id.is_none());
}

#[test]
fn build_status_snapshot_maps_alive_and_dead_summaries() {
    let record = record();
    let alive_payload = build_status_payload(&record, true, Some(99));
    let dead_payload = build_status_payload(&record, false, None);

    let alive = build_status_snapshot(&record, &alive_payload, true);
    let dead = build_status_snapshot(&record, &dead_payload, false);

    assert_eq!(alive.action, "status_snapshot");
    assert_eq!(alive.status, "alive");
    assert_eq!(alive.summary, "queue owner healthy");
    assert_eq!(alive.pid, Some(99));
    assert_eq!(alive.vwacp_record_id, "record-1");
    assert_eq!(alive.vwacp_session_id, "session-1");
    assert_eq!(alive.agent_session.agent_session_id.as_deref(), Some("agent-session-1"));
    assert_eq!(dead.status, "dead");
    assert_eq!(dead.summary, "queue owner unavailable");
    assert_eq!(dead.exit_code, Some(7));
    assert_eq!(dead.signal.as_deref(), Some("TERM"));
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
fn write_no_session_status_propagates_writer_errors() {
    let mut writer = FailingWriter;
    let error = write_no_session_status(&mut writer, &flags(OutputFormat::Text), "agent")
        .expect_err("writer error should bubble up");

    assert!(matches!(error, StatusCommandError::Io(_)));
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

#[test]
fn write_status_payload_running_text_omits_dead_exit_fields() {
    let record = record();
    let payload = build_status_payload(&record, true, Some(123));

    let mut text = Vec::new();
    write_status_payload(&mut text, &flags(OutputFormat::Text), &record, &payload, true).unwrap();
    let text = String::from_utf8(text).unwrap();

    assert!(text.contains("pid: 123"));
    assert!(text.contains("status: running"));
    assert!(text.contains("model: model-a"));
    assert!(text.contains("mode: focus"));
    assert!(text.contains("uptime: 00:00:00"));
    assert!(text.contains("lastPromptTime: 2026-01-01T00:01:00Z"));
    assert!(!text.contains("exitCode:"));
    assert!(!text.contains("signal:"));
}

#[test]
fn write_status_payload_json_skips_absent_optional_fields() {
    let mut record = record();
    record.agent_session_id = None;
    record.vwacp = None;
    record.last_prompt_at = None;
    let payload = build_status_payload(&record, false, None);

    let mut json = Vec::new();
    write_status_payload(&mut json, &flags(OutputFormat::Json), &record, &payload, false).unwrap();
    let value: Value = serde_json::from_slice(&json).unwrap();

    assert_eq!(value["status"], "dead");
    assert_eq!(value["vwacpRecordId"], "record-1");
    assert_eq!(value["vwacpSessionId"], "session-1");
    assert!(value.get("agentSessionId").is_none());
    assert!(value.get("model").is_none());
    assert!(value.get("mode").is_none());
    assert!(value.get("lastPromptTime").is_none());
}

#[test]
fn write_status_payload_propagates_writer_errors() {
    let record = record();
    let payload = build_status_payload(&record, false, None);
    let mut writer = FailingWriter;

    let error =
        write_status_payload(&mut writer, &flags(OutputFormat::Text), &record, &payload, false)
            .expect_err("writer error should bubble up");

    assert!(matches!(error, StatusCommandError::Io(_)));
}

#[tokio::test]
async fn handle_status_writes_no_session_for_missing_record() {
    let status_flags = StatusFlags { session: Some("missing-status".to_string()) };
    let global_flags = missing_session_flags(OutputFormat::Quiet);

    handle_status(None, &status_flags, &global_flags, &config()).await.unwrap();
}

#[tokio::test]
async fn handle_sessions_show_reports_missing_named_session() {
    let global_flags = missing_session_flags(OutputFormat::Quiet);
    let error = handle_sessions_show(None, Some("missing-show"), &global_flags, &config())
        .await
        .expect_err("missing session should error");

    assert_eq!(
        error.to_string(),
        format!("No named session \"missing-show\" for cwd {} and agent agent", global_flags.cwd)
    );
}

#[tokio::test]
async fn handle_sessions_history_reports_missing_cwd_session() {
    let global_flags = missing_session_flags(OutputFormat::Quiet);
    let error = handle_sessions_history(None, None, 3, &global_flags, &config())
        .await
        .expect_err("missing session should error");

    assert_eq!(
        error.to_string(),
        format!("No cwd session for {} and agent agent", global_flags.cwd)
    );
}
