use super::traces::run_traces;
use crate::app::agent::config::Config;
use serde_json::json;
use std::fs;

fn config_for_trace_file(path: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.workspace_dir = path.parent().expect("trace file parent").to_path_buf();
    config.observability.runtime_trace_path =
        path.file_name().expect("trace file name").to_string_lossy().to_string();
    config
}

fn write_trace_events(path: &std::path::Path) {
    let old = json!({
        "id": "old-id",
        "timestamp": "2026-01-01T00:00:00Z",
        "event_type": "tool_call",
        "success": true,
        "message": "short result",
        "payload": {"tool": "read"}
    });
    let newest = json!({
        "id": "new-id",
        "timestamp": "2026-01-01T00:00:02Z",
        "event_type": "llm_response",
        "success": false,
        "message": "this diagnostic message is intentionally long enough to be truncated in list output",
        "provider": "openrouter",
        "model": "glm",
        "payload": {"reason": "needle"}
    });
    fs::write(path, format!("{old}\nnot-json\n{newest}\n")).expect("write trace jsonl");
}

#[test]
fn run_traces_returns_ok_when_trace_file_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let trace = temp.path().join("missing.jsonl");
    let config = config_for_trace_file(&trace);

    run_traces(&config, None, None, None, 0).expect("missing trace is informational");
}

#[test]
fn run_traces_lists_matching_events_with_safe_limit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let trace = temp.path().join("trace.jsonl");
    write_trace_events(&trace);
    let config = config_for_trace_file(&trace);

    run_traces(&config, None, Some("llm_response"), Some("needle"), 0)
        .expect("filtered trace list");
}

#[test]
fn run_traces_handles_empty_query_result() {
    let temp = tempfile::tempdir().expect("tempdir");
    let trace = temp.path().join("trace.jsonl");
    write_trace_events(&trace);
    let config = config_for_trace_file(&trace);

    run_traces(&config, None, Some("missing-event"), None, 10).expect("empty query");
}

#[test]
fn run_traces_can_lookup_by_id_and_report_miss() {
    let temp = tempfile::tempdir().expect("tempdir");
    let trace = temp.path().join("trace.jsonl");
    write_trace_events(&trace);
    let config = config_for_trace_file(&trace);

    run_traces(&config, Some(" new-id "), None, None, 10).expect("trace lookup hit");
    run_traces(&config, Some("missing-id"), None, None, 10).expect("trace lookup miss");
}
