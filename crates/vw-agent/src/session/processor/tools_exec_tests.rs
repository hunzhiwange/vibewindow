use crate::app::agent::session::processor::StreamEvent;
use crate::app::agent::session::session::{Role, Session};
use crate::app::agent::tools::{ToolCallError, ToolRuntimeContext, todo};
use crate::app::agent::tools::{ToolCallResult, ToolCallTelemetry, ToolRenderHint};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);

fn unique_session(prefix: &str) -> String {
    format!("{prefix}-{}-{}", std::process::id(), NEXT_SESSION.fetch_add(1, Ordering::Relaxed))
}

fn ctx(session: &str) -> ToolRuntimeContext {
    ToolRuntimeContext::new(session.to_string(), None)
}

fn new_state() -> super::super::ToolSessionState {
    super::super::ToolSessionState::default()
}

fn delta_texts(events: &[StreamEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::Delta(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn preserve_full_output_only_for_editing_tools() {
    for name in ["apply_patch", "write", "file_write", "file_edit", "notebook_edit"] {
        assert!(super::preserve_full_output_in_session(name), "{name}");
    }
    for name in ["read", "file_read", "bash", "todowrite"] {
        assert!(!super::preserve_full_output_in_session(name), "{name}");
    }
}

#[test]
fn denied_tool_payload_includes_input_error_and_optional_permission_request() {
    let denied = ToolCallError::denied("blocked by policy");
    let payload = super::denied_tool_payload(Some(r#"{"path":"secret"}"#), &denied);

    assert_eq!(payload["status"], "denied");
    assert_eq!(payload["input"], r#"{"path":"secret"}"#);
    assert_eq!(payload["error"], "blocked by policy");
    assert!(payload.get("permission_request").is_none());

    let denied = ToolCallError::denied_with_permission_request(
        "needs approval",
        vw_api_types::tools::PermissionRequestDto {
            reason: "write outside workspace".to_string(),
            warning: Some("careful".to_string()),
            updated_input: Some(json!({ "path": "demo.md" })),
        },
    );
    let payload = super::denied_tool_payload(None, &denied);

    assert_eq!(payload["status"], "denied");
    assert!(payload.get("input").is_none());
    assert_eq!(payload["permission_request"]["reason"], "write outside workspace");
    assert_eq!(payload["permission_request"]["warning"], "careful");
    assert_eq!(payload["permission_request"]["updated_input"]["path"], "demo.md");
}

#[test]
fn completed_tool_payload_for_ui_uses_render_hint_metadata_and_omits_empty_result() {
    let result = ToolCallResult {
        render_hint: Some(ToolRenderHint {
            title: Some("Friendly title".to_string()),
            kind: Some("demo".to_string()),
            summary: Some("summary".to_string()),
            metadata: json!({ "rows": 3 }),
        }),
        telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
        ..ToolCallResult::default()
    };

    let payload =
        super::completed_tool_payload_for_ui("demo_tool", "{}", &result, "compact output");

    assert_eq!(payload["status"], "completed");
    assert_eq!(payload["input"], "{}");
    assert_eq!(payload["title"], "Friendly title");
    assert_eq!(payload["metadata"]["rows"], 3);
    assert_eq!(payload["output"], "compact output");
    assert!(payload.get("result").is_none());
}

#[test]
fn completed_tool_payload_for_ui_keeps_structured_patch_content() {
    use vw_api_types::tools::{StructuredPatchHunkDto, ToolResultContentDto};

    let result = ToolCallResult {
        content_blocks: vec![ToolResultContentDto::StructuredPatch {
            hunks: vec![StructuredPatchHunkDto {
                header: "@@ -1 +1 @@".to_string(),
                path: Some("docs/tailwind/align-self.mdx".to_string()),
                old_start: Some(1),
                old_lines: Some(1),
                new_start: Some(1),
                new_lines: Some(1),
                lines: vec!["+full replacement line".to_string()],
            }],
        }],
        telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
        ..ToolCallResult::default()
    };

    let payload = super::completed_tool_payload_for_ui(
        "file_edit",
        r#"{"file_path":"docs/tailwind/align-self.mdx"}"#,
        &result,
        "Updated docs/tailwind/align-self.mdx.",
    );

    let content = payload
        .get("result")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_array)
        .expect("structured content should be exposed to UI");

    assert_eq!(content[0].get("type").and_then(Value::as_str), Some("structured_patch"));
    assert_eq!(
        content[0]
            .get("hunks")
            .and_then(Value::as_array)
            .and_then(|hunks| hunks[0].get("lines"))
            .and_then(Value::as_array)
            .and_then(|lines| lines[0].as_str()),
        Some("+full replacement line")
    );
}

#[test]
fn describe_batch_call_formats_known_tools_and_falls_back_to_sanitized_json() {
    assert_eq!(
        super::describe_batch_call("bash", &json!({ "command": "echo ok" })),
        "- bash echo ok"
    );
    assert_eq!(
        super::describe_batch_call(
            "read",
            &json!({ "path": "src/lib.rs", "offset": -2, "limit": 10 })
        ),
        "- read src/lib.rs [offset=1, limit=10]"
    );
    assert_eq!(
        super::describe_batch_call("file_read", &json!({ "offset": 5 })),
        "- file_read [offset=5]"
    );
    assert_eq!(
        super::describe_batch_call("file_edit", &json!({ "file_path": "src/main.rs" })),
        "- file_edit src/main.rs"
    );
    assert_eq!(
        super::describe_batch_call("glob", &json!({ "pattern": "**/*.rs" })),
        "- glob **/*.rs"
    );
    assert_eq!(
        super::describe_batch_call("grep", &json!({ "pattern": "TODO", "path": "src" })),
        "- grep pattern=TODO path=src"
    );
    assert_eq!(
        super::describe_batch_call("grep", &json!({ "pattern": "TODO" })),
        "- grep pattern=TODO"
    );
    assert_eq!(super::describe_batch_call("grep", &json!({ "path": "src" })), "- grep path=src");
    assert_eq!(super::describe_batch_call("ls", &json!({ "path": "src" })), "- ls src");
    assert_eq!(super::describe_batch_call("custom", &json!({})), "- custom {}");
}

#[test]
fn run_batch_error_and_record_emits_error_event_and_deduplicates_session_message() {
    let session_id = unique_session("tools-batch-error");
    let mut session = Session::new(session_id.clone());
    let mut state = new_state();
    let mut events = Vec::new();

    super::run_batch_error_and_record(
        &mut session,
        "{bad",
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
        "bad input".to_string(),
    );
    super::run_batch_error_and_record(
        &mut session,
        "{bad",
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
        "bad input".to_string(),
    );

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, Role::Tool);
    assert!(session.messages[0].content.contains("\"status\":\"error\""));
    assert_eq!(delta_texts(&events).len(), 2);
    assert!(delta_texts(&events)[0].contains("bad input"));
}

#[test]
fn run_tool_and_record_reports_parse_errors_for_streaming_tools_and_deduplicates() {
    let session_id = unique_session("tools-single-error");
    let mut session = Session::new(session_id.clone());
    let ctx = ctx(&session_id);
    let mut state = new_state();
    let mut events = Vec::new();

    let first = super::run_tool_and_record(
        &mut session,
        "file_read",
        "{bad",
        &ctx,
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
    )
    .expect("content");
    let second = super::run_tool_and_record(
        &mut session,
        "file_read",
        "{bad",
        &ctx,
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
    )
    .expect("content");

    assert!(first.contains("invalid JSON arguments"));
    assert_eq!(first, second);
    assert_eq!(session.messages.len(), 1);
    assert!(session.messages[0].content.contains("\"status\":\"error\""));
    assert_eq!(state.non_todo_tool_runs, 2);
    assert!(delta_texts(&events).iter().any(|text| text.contains("\"status\":\"running\"")));
    assert!(delta_texts(&events).iter().any(|text| text.contains("invalid JSON arguments")));
}

#[test]
fn run_tool_and_record_delegates_batch_tool() {
    let session_id = unique_session("tools-batch-delegate");
    let mut session = Session::new(session_id.clone());
    let ctx = ctx(&session_id);
    let mut state = new_state();
    let mut events = Vec::new();

    let output = super::run_tool_and_record(
        &mut session,
        "batch",
        "not-json",
        &ctx,
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
    )
    .expect("batch content");

    assert_eq!(output, "Invalid input format for batch");
    assert_eq!(session.messages.len(), 1);
    assert!(session.messages[0].content.contains("tool batch"));
    assert!(delta_texts(&events).iter().any(|text| text.contains("\"status\":\"error\"")));
}

#[test]
fn run_tool_and_record_rewrites_initial_todowrite_completed_status_until_non_todo_work_exists() {
    let session_id = unique_session("tools-todowrite-rewrite");
    let mut session = Session::new(session_id.clone());
    let ctx = ctx(&session_id);
    let mut state = new_state();
    let input = json!({
        "todos": [
            { "id": "1", "content": "finish", "status": "completed", "priority": "high" }
        ]
    })
    .to_string();

    super::run_tool_and_record(
        &mut session,
        "todowrite",
        &input,
        &ctx,
        false,
        &mut |_event| true,
        &mut state,
    )
    .expect("todowrite output");

    let todos: Vec<todo::Todo> =
        serde_json::from_str(&todo::read(&ctx).expect("read todos")).expect("todos json");
    assert_eq!(todos[0].status, "pending");
    assert_eq!(state.non_todo_tool_runs, 0);

    state.non_todo_tool_runs = 1;
    super::run_tool_and_record(
        &mut session,
        "todowrite",
        &input,
        &ctx,
        false,
        &mut |_event| true,
        &mut state,
    )
    .expect("todowrite output");

    let todos: Vec<todo::Todo> =
        serde_json::from_str(&todo::read(&ctx).expect("read todos")).expect("todos json");
    assert_eq!(todos[0].status, "completed");
}

#[test]
fn run_batch_tool_and_record_handles_invalid_json_recursive_and_empty_batches() {
    let session_id = unique_session("tools-batch-invalid");
    let ctx = ctx(&session_id);

    let mut invalid_session = Session::new(format!("{session_id}-invalid"));
    let mut invalid_state = new_state();
    let invalid = super::run_batch_tool_and_record(
        &mut invalid_session,
        "{bad",
        &ctx,
        false,
        &mut |_event| true,
        &mut invalid_state,
    );
    assert!(invalid.contains("invalid arguments"));
    assert_eq!(invalid_session.messages.len(), 1);

    let mut recursive_session = Session::new(format!("{session_id}-recursive"));
    let mut recursive_state = new_state();
    let recursive = super::run_batch_tool_and_record(
        &mut recursive_session,
        &json!({ "tool_calls": [{ "tool": "batch", "parameters": {} }] }).to_string(),
        &ctx,
        false,
        &mut |_event| true,
        &mut recursive_state,
    );
    assert_eq!(recursive, "Recursive batch calls are not allowed");
    assert_eq!(recursive_session.messages.len(), 1);

    let mut empty_session = Session::new(format!("{session_id}-empty"));
    let mut empty_state = new_state();
    let empty = super::run_batch_tool_and_record(
        &mut empty_session,
        &json!({ "tool_calls": [{ "parameters": {} }, { "tool": null }] }).to_string(),
        &ctx,
        false,
        &mut |_event| true,
        &mut empty_state,
    );
    assert_eq!(empty, "未执行任何子任务");
    assert_eq!(empty_session.messages.len(), 1);
    assert!(empty_session.messages[0].content.contains("未执行任何子任务"));
}

#[test]
fn run_batch_tool_and_record_records_failed_subtool_and_summary() {
    let session_id = unique_session("tools-batch-failed-subtool");
    let mut session = Session::new(session_id.clone());
    let ctx = ctx(&session_id);
    let mut state = new_state();
    let mut events = Vec::new();

    let output = super::run_batch_tool_and_record(
        &mut session,
        &json!({ "tool_calls": [{ "tool": "unknown_tool", "parameters": {} }] }).to_string(),
        &ctx,
        true,
        &mut |event| {
            events.push(event);
            true
        },
        &mut state,
    );

    assert!(output.contains("已成功执行 0/1 个工具"));
    assert!(output.contains("Unknown tool: unknown_tool"));
    assert_eq!(session.messages.len(), 2);
    assert!(session.messages.iter().any(|message| message.content.contains("tool unknown_tool")));
    assert!(session.messages.iter().any(|message| message.content.contains("tool batch")));
    assert_eq!(state.non_todo_tool_runs, 1);
    assert!(delta_texts(&events).iter().any(|text| text.contains("Unknown tool")));
    assert!(delta_texts(&events).iter().any(|text| text.contains("已执行 1 个子任务")));
}

#[test]
fn run_batch_tool_and_record_summarizes_single_and_parallel_successful_batches() {
    let session_id = unique_session("tools-batch-success");
    let ctx = ctx(&session_id);

    let mut single_session = Session::new(format!("{session_id}-single"));
    let mut single_state = new_state();
    let single = super::run_batch_tool_and_record(
        &mut single_session,
        &json!({ "tool_calls": [{ "tool": "todoread", "parameters": {} }] }).to_string(),
        &ctx,
        false,
        &mut |_event| true,
        &mut single_state,
    );
    assert!(single.contains("全部 1 个工具均执行成功"));
    assert_eq!(single_session.messages.len(), 2);
    assert_eq!(single_state.non_todo_tool_runs, 0);

    let mut parallel_session = Session::new(format!("{session_id}-parallel"));
    let mut parallel_state = new_state();
    let parallel = super::run_batch_tool_and_record(
        &mut parallel_session,
        &json!({
            "calls": [
                { "tool": "todoread", "parameters": {} },
                { "tool": "todoread", "parameters": {} }
            ]
        })
        .to_string(),
        &ctx,
        false,
        &mut |_event| true,
        &mut parallel_state,
    );
    assert!(parallel.contains("全部 2 个工具均执行成功"));
    assert!(parallel_session.messages.iter().any(|message| message.content.contains("tool batch")));
    assert_eq!(parallel_state.non_todo_tool_runs, 0);
}
