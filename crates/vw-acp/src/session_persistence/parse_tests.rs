use super::*;
use crate::SESSION_RECORD_SCHEMA;

fn valid_record() -> Value {
    serde_json::json!({
        "schema": SESSION_RECORD_SCHEMA,
        "vwacp_record_id": "record-1",
        "acp_session_id": "acp-1",
        "agent_session_id": "agent-1",
        "agent_command": "agent",
        "cwd": "/tmp/project",
        "created_at": "2026-01-01T00:00:00Z",
        "last_used_at": "2026-01-02T00:00:00Z",
        "last_seq": 2,
        "name": "  main  ",
        "closed": false,
        "conversation": "ignored",
        "messages": [],
        "updated_at": "2026-01-02T00:00:00Z",
        "cumulative_token_usage": {"input_tokens": 1, "output_tokens": 2},
        "request_token_usage": {"req-1": {"input_tokens": 3}},
        "event_log": {
            "active_path": "/tmp/active.ndjson",
            "segment_count": 1,
            "max_segment_bytes": 1024,
            "max_segments": 3,
            "last_write_at": null,
            "last_write_error": "disk full"
        },
        "vwacp": {
            "desired_mode_id": "plan",
            "available_models": ["a", "b"],
            "available_commands": ["cmd"],
            "session_options": {"model": "gpt", "allowed_tools": ["shell"], "max_turns": 4}
        }
    })
}

#[test]
fn parse_non_negative_number_rejects_negative_and_non_numeric_values() {
    let record = serde_json::json!({"ok": 0, "bad": -1, "text": "1"});
    let record = record.as_object().unwrap();

    assert_eq!(parse_non_negative_number(record, "ok"), Some(Some(0)));
    assert_eq!(parse_non_negative_number(record, "missing"), Some(None));
    assert_eq!(parse_non_negative_number(record, "bad"), None);
    assert_eq!(parse_non_negative_number(record, "text"), None);
}

#[test]
fn parse_session_record_accepts_valid_record_and_normalizes_optional_fields() {
    let record = parse_session_record(&valid_record()).expect("valid record");

    assert_eq!(record.vwacp_record_id, "record-1");
    assert_eq!(record.name.as_deref(), Some("main"));
    assert_eq!(record.cumulative_token_usage.input_tokens, Some(1));
    assert_eq!(record.request_token_usage["req-1"].input_tokens, Some(3));
    assert_eq!(record.event_log.last_write_error.as_deref(), Some("disk full"));
    assert_eq!(
        record.vwacp.as_ref().unwrap().session_options.as_ref().unwrap().model.as_deref(),
        Some("gpt")
    );
}

#[test]
fn parse_session_record_rejects_wrong_schema_and_negative_sequence() {
    let mut wrong_schema = valid_record();
    wrong_schema["schema"] = Value::String("other".to_string());
    assert!(parse_session_record(&wrong_schema).is_none());

    let mut negative_seq = valid_record();
    negative_seq["last_seq"] = Value::Number((-1).into());
    assert!(parse_session_record(&negative_seq).is_none());
}

#[test]
fn parse_event_log_falls_back_when_required_values_are_invalid() {
    let fallback =
        parse_event_log(Some(&serde_json::json!({"active_path": "/tmp/a"})), "session-1");

    assert!(fallback.active_path.contains("session-1"));
    assert!(fallback.segment_count > 0);
}

#[test]
fn normalize_optional_name_trims_blank_and_rejects_non_strings() {
    assert_eq!(
        normalize_optional_name(Some(&Value::String("  name ".to_string()))),
        Some(Some("name".to_string()))
    );
    assert_eq!(normalize_optional_name(Some(&Value::String(" ".to_string()))), Some(None));
    assert_eq!(normalize_optional_name(Some(&Value::Bool(true))), None);
}
