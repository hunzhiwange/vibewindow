use serde_json::{Map, Value, json};
use vw_acp::AcpJsonRpcMessage;
use vw_api_types::tools::{StructuredPatchHunkDto, ToolResultContentDto, ToolResultDto};

use crate::app::agent::session::{llm::StreamEvent, ui_types};

use super::*;

fn acp_message(value: Value) -> AcpJsonRpcMessage {
    serde_json::from_value(value).expect("valid ACP JSON-RPC message")
}

fn update_message(update: Value) -> AcpJsonRpcMessage {
    acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "update": update
        }
    }))
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().expect("value should be object").clone()
}

#[test]
fn extract_text_content_accepts_text_and_resource_uri_blocks() {
    assert_eq!(extract_text_content(&json!({"type": "text", "text": "hello"})), Some("hello"));
    assert_eq!(
        extract_text_content(&json!({"type": "resource_link", "uri": "file:///tmp/a.txt"})),
        Some("file:///tmp/a.txt")
    );
    assert_eq!(
        extract_text_content(&json!({
            "type": "resource",
            "resource": {"uri": "file:///tmp/b.txt"}
        })),
        Some("file:///tmp/b.txt")
    );
    assert_eq!(extract_text_content(&json!({"type": "image", "data": "..."})), None);
    assert_eq!(extract_text_content(&json!("plain")), None);
}

#[test]
fn numeric_usage_helpers_accept_common_json_number_shapes() {
    assert_eq!(json_number_to_i64(&json!(-3)), Some(-3));
    assert_eq!(json_number_to_i64(&json!(7_u64)), Some(7));
    assert_eq!(json_number_to_i64(&json!(8.75)), Some(8));
    assert_eq!(json_number_to_i64(&json!("8")), None);

    let source = object(json!({
        "inputTokens": 11,
        "output_tokens": 12,
        "cachedTokens": 13,
        "thought_tokens": 14
    }));
    assert_eq!(token_usage_value(&source, &["input_tokens", "inputTokens"]), 11);
    assert_eq!(token_usage_value(&source, &["missing"]), 0);
}

#[test]
fn session_update_payload_is_extracted_only_from_update_params() {
    let message = update_message(json!({"sessionUpdate": "usage_update", "input_tokens": 1}));
    let payload = session_update_payload_from_message(&message).expect("payload");
    assert_eq!(payload.get("sessionUpdate").and_then(Value::as_str), Some("usage_update"));

    let missing = acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {}
    }));
    assert!(session_update_payload_from_message(&missing).is_none());
}

#[test]
fn message_delta_extractors_accept_string_and_content_blocks() {
    assert_eq!(
        extract_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type": "text", "text": "delta"}
        })))
        .as_deref(),
        Some("delta")
    );
    assert_eq!(
        extract_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": "plain"
        })))
        .as_deref(),
        Some("plain")
    );
    assert_eq!(
        extract_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type": "resource_link", "uri": "file:///tmp/out"}
        })))
        .as_deref(),
        Some("file:///tmp/out")
    );
    assert!(
        extract_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": ""
        })))
        .is_none()
    );
    assert!(
        extract_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": "not text"
        })))
        .is_none()
    );

    assert_eq!(
        extract_reasoning_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": {"type": "text", "text": "thinking"}
        })))
        .as_deref(),
        Some("thinking")
    );
    assert!(
        extract_reasoning_delta_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": {"type": "unknown"}
        })))
        .is_none()
    );
}

#[test]
fn tool_update_name_title_and_input_helpers_normalize_values() {
    let update = object(json!({
        "kind": " shell ",
        "title": " Terminal ",
        "rawInput": {"cmd": "pwd"}
    }));
    assert_eq!(tool_kind_from_update(&update), Some("shell"));
    assert_eq!(tool_title_from_update(&update), Some("Terminal"));
    assert_eq!(transcript_tool_name(&update), "shell");
    assert_eq!(transcript_tool_title(&update, "shell"), "Terminal");
    assert_eq!(tool_input_string(&update), r#"{"cmd":"pwd"}"#);

    let update = object(json!({"input": "already serialized"}));
    assert_eq!(transcript_tool_name(&update), "tool_call");
    assert_eq!(transcript_tool_title(&update, "tool_call"), "tool_call");
    assert_eq!(tool_input_string(&update), "already serialized");

    let update = object(json!({"rawInput": null}));
    assert_eq!(tool_input_string(&update), "{}");
}

#[test]
fn tool_result_text_helpers_cover_scalars_arrays_objects_and_patches() {
    assert_eq!(extract_tool_result_text_value(&Value::Null), None);
    assert_eq!(extract_tool_result_text_value(&json!("text")).as_deref(), Some("text"));
    assert_eq!(extract_tool_result_text_value(&json!(42)).as_deref(), Some("42"));
    assert_eq!(extract_tool_result_text_value(&json!(true)).as_deref(), Some("true"));
    assert_eq!(
        extract_tool_result_text_value(&json!([null, {"stdout": "out"}])).as_deref(),
        Some("out")
    );
    assert_eq!(
        extract_tool_result_text_value(&json!({"content": [{"text": "nested"}]})).as_deref(),
        Some("nested")
    );
    assert_eq!(extract_tool_result_text_value(&json!({"unknown": "value"})), None);

    let hunks = vec![
        StructuredPatchHunkDto {
            header: "@@ -1 +1 @@".to_string(),
            path: Some("a.rs".to_string()),
            old_start: None,
            old_lines: None,
            new_start: None,
            new_lines: None,
            lines: vec!["-old".to_string(), "+new".to_string()],
        },
        StructuredPatchHunkDto {
            header: String::new(),
            path: Some("b.rs".to_string()),
            old_start: None,
            old_lines: None,
            new_start: None,
            new_lines: None,
            lines: vec!["+line".to_string()],
        },
    ];
    assert_eq!(
        structured_patch_diff_text(&hunks).as_deref(),
        Some("--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-old\n+new\n--- a/b.rs\n+++ b/b.rs\n+line\n")
    );
    assert_eq!(
        structured_patch_diff_text(&[StructuredPatchHunkDto {
            header: "@@ ignored @@".to_string(),
            path: Some(" ".to_string()),
            old_start: None,
            old_lines: None,
            new_start: None,
            new_lines: None,
            lines: vec!["+ignored".to_string()],
        }]),
        None
    );
}

#[test]
fn tool_result_dto_text_prefers_content_then_model_result_then_data() {
    let content_result = ToolResultDto {
        tool_use_id: None,
        tool_id: None,
        success: None,
        content: vec![
            ToolResultContentDto::Text { text: "   ".to_string() },
            ToolResultContentDto::Json { value: json!({"message": "json message"}) },
        ],
        data: Value::Null,
        model_result: Value::Null,
        render_hint: None,
        permission_request: None,
        context_updates: Vec::new(),
        extra_messages: Vec::new(),
        telemetry: None,
    };
    assert_eq!(extract_tool_result_dto_text(&content_result).as_deref(), Some("json message"));

    let patch_result: ToolResultDto = serde_json::from_value(json!({
        "content": [{
            "type": "structured_patch",
            "hunks": [{"path": "a.rs", "header": "@@ -1 +1 @@", "lines": ["-a", "+b"]}]
        }]
    }))
    .expect("tool result dto");
    assert_eq!(
        extract_tool_result_dto_text(&patch_result).as_deref(),
        Some("--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-a\n+b\n")
    );

    let model_result: ToolResultDto =
        serde_json::from_value(json!({"model_result": {"output": "model"}})).expect("dto");
    assert_eq!(extract_tool_result_dto_text(&model_result).as_deref(), Some("model"));

    let data_result: ToolResultDto =
        serde_json::from_value(json!({"data": {"value": "data"}})).expect("dto");
    assert_eq!(extract_tool_result_dto_text(&data_result).as_deref(), Some("data"));
}

#[test]
fn tool_result_dto_parser_accepts_raw_output_and_result_aliases() {
    let raw = object(json!({
        "rawOutput": {
            "success": true,
            "content": [{"type": "text", "text": "raw"}]
        }
    }));
    assert_eq!(
        parse_tool_result_dto(&raw).and_then(|dto| extract_tool_result_dto_text(&dto)).as_deref(),
        Some("raw")
    );

    let result = object(json!({
        "result": {
            "success": false,
            "model_result": {"message": "from result"}
        }
    }));
    let parsed = parse_tool_result_dto(&result).expect("result alias should parse");
    assert_eq!(parsed.success, Some(false));
    assert_eq!(extract_tool_result_dto_text(&parsed).as_deref(), Some("from result"));
}

#[test]
fn tool_status_metadata_output_and_error_helpers_cover_fallbacks() {
    assert_eq!(normalize_tool_status(&object(json!({"status": "still running"})), None), "running");
    assert_eq!(normalize_tool_status(&object(json!({"status": "denied"})), None), "denied");
    assert_eq!(normalize_tool_status(&object(json!({"status": "success"})), None), "completed");
    assert_eq!(normalize_tool_status(&object(json!({"status": "custom"})), None), "custom");

    let success: ToolResultDto = serde_json::from_value(json!({"success": true})).expect("dto");
    let failed: ToolResultDto = serde_json::from_value(json!({"success": false})).expect("dto");
    assert_eq!(normalize_tool_status(&object(json!({})), Some(&success)), "completed");
    assert_eq!(normalize_tool_status(&object(json!({})), Some(&failed)), "error");
    assert_eq!(normalize_tool_status(&object(json!({"rawOutput": "ok"})), None), "completed");
    assert_eq!(normalize_tool_status(&object(json!({})), None), "running");

    let result: ToolResultDto = serde_json::from_value(json!({
        "content": [{"type": "text", "text": "content"}],
        "render_hint": {
            "summary": "summary",
            "metadata": {"path": "a.rs"}
        }
    }))
    .expect("dto");
    assert_eq!(metadata_from_tool_result(Some(&result)), json!({"path": "a.rs"}));
    assert_eq!(metadata_from_tool_result(None), json!({}));
    assert_eq!(output_from_tool_update(&object(json!({})), Some(&result)), "content");
    assert_eq!(
        output_from_tool_update(&object(json!({"rawOutput": {"stderr": "boom"}})), None),
        "boom"
    );
    assert_eq!(
        output_from_tool_update(&object(json!({"output": {"nested": "value"}})), None),
        r#"{"nested":"value"}"#
    );
    assert_eq!(error_from_tool_update("error", " boom ").as_deref(), Some("boom"));
    assert_eq!(error_from_tool_update("completed", "boom"), None);
    assert_eq!(error_from_tool_update("error", " "), None);
}

#[test]
fn transcript_tool_delta_accepts_input_output_aliases() {
    let update = object(json!({
        "sessionUpdate": "tool_call_update",
        "toolCallId": "call-1",
        "kind": "execute",
        "title": "Terminal",
        "status": "completed",
        "input": {
            "command": "date '+%Y-%m-%d %H:%M:%S'",
            "description": "Get current date and time"
        },
        "output": "2026-06-05 12:34:56\n"
    }));

    let delta = transcript_tool_delta_from_update(&update).expect("delta should be built");
    let payload: Value =
        serde_json::from_str(delta.lines().nth(1).expect("json payload")).expect("payload");

    assert!(delta.starts_with("tool execute\n"));
    assert_eq!(payload["status"], "completed");
    assert_eq!(payload["toolCallId"], "call-1");
    assert_eq!(payload["title"], "Terminal");
    assert_eq!(payload["output"], "2026-06-05 12:34:56\n");
    assert!(payload["input"].as_str().expect("input").contains("date '+%Y-%m-%d %H:%M:%S'"));
}

#[test]
fn transcript_tool_delta_renders_dto_metadata_summary_and_errors() {
    let update = object(json!({
        "sessionUpdate": "tool_call",
        "toolCallId": "call-2",
        "title": "Apply Patch",
        "rawInput": {"path": "a.rs"},
        "rawOutput": {
            "success": false,
            "content": [{"type": "text", "text": "patch failed"}],
            "render_hint": {
                "summary": "failed summary",
                "metadata": {"kind": "patch"}
            }
        }
    }));

    let delta = transcript_tool_delta_from_update(&update).expect("delta should be built");
    let payload: Value =
        serde_json::from_str(delta.lines().nth(1).expect("json payload")).expect("payload");

    assert!(delta.starts_with("tool Apply Patch\n"));
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["metadata"], json!({"kind": "patch"}));
    assert_eq!(payload["output"], "patch failed");
    assert_eq!(payload["error"], "patch failed");
    assert_eq!(payload["summary"], "failed summary");
    assert!(payload["result"].is_object());
}

#[test]
fn transcript_tool_delta_ignores_non_tool_or_missing_id_updates() {
    assert!(
        transcript_tool_delta_from_update(&object(json!({
            "sessionUpdate": "agent_message_chunk",
            "toolCallId": "call-1"
        })))
        .is_none()
    );
    assert!(
        transcript_tool_delta_from_update(&object(json!({
            "sessionUpdate": "tool_call",
            "toolCallId": " "
        })))
        .is_none()
    );
}

#[test]
fn extract_tool_call_from_message_returns_remote_tool_summary() {
    let tool_call = extract_tool_call_from_acp_message(&update_message(json!({
        "sessionUpdate": "tool_call",
        "toolCallId": " call-1 ",
        "kind": "shell",
        "title": "Terminal",
        "rawInput": {"cmd": "pwd"}
    })))
    .expect("tool call");

    assert_eq!(tool_call.id, "call-1");
    assert_eq!(tool_call.name, "Terminal");
    assert_eq!(tool_call.arguments, r#"{"cmd":"pwd"}"#);

    let fallback = extract_tool_call_from_acp_message(&update_message(json!({
        "sessionUpdate": "tool_call_update",
        "toolCallId": "call-2",
        "kind": "shell"
    })))
    .expect("tool call");
    assert_eq!(fallback.name, "shell");
    assert_eq!(fallback.arguments, "{}");

    assert!(
        extract_tool_call_from_acp_message(&update_message(json!({
            "sessionUpdate": "tool_call",
            "toolCallId": " "
        })))
        .is_none()
    );
}

#[test]
fn usage_updates_accept_top_level_and_meta_aliases() {
    let top_level = extract_usage_from_acp_message(&update_message(json!({
        "sessionUpdate": "usage_update",
        "inputTokens": 10,
        "output_tokens": 20,
        "cachedTokens": 30,
        "reasoning_tokens": 40
    })))
    .expect("usage");
    assert_eq!(
        top_level,
        ui_types::TokenUsage {
            input_tokens: 10,
            output_tokens: 20,
            cached_tokens: 30,
            reasoning_tokens: 40,
        }
    );

    let meta = extract_usage_from_acp_message(&update_message(json!({
        "sessionUpdate": "usage_update",
        "_meta": {
            "usage": {
                "input_tokens": 1.9,
                "outputTokens": 2,
                "cacheReadInputTokens": 3,
                "thought_tokens": 4
            }
        }
    })))
    .expect("usage");
    assert_eq!(
        meta,
        ui_types::TokenUsage {
            input_tokens: 1,
            output_tokens: 2,
            cached_tokens: 3,
            reasoning_tokens: 4,
        }
    );

    assert!(
        extract_usage_from_acp_message(&update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": "not usage"
        })))
        .is_none()
    );
}

#[test]
fn forward_acp_message_emits_deltas_reasoning_tools_and_usage() {
    let mut latest_usage = ui_types::TokenUsage::default();
    let mut delta_count = 0usize;
    let mut events = Vec::new();

    forward_acp_message(
        &update_message(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": "hello"
        })),
        &mut |event| events.push(event),
        &mut latest_usage,
        &mut delta_count,
    );
    forward_acp_message(
        &update_message(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": "thinking"
        })),
        &mut |event| events.push(event),
        &mut latest_usage,
        &mut delta_count,
    );
    forward_acp_message(
        &update_message(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "call-1",
            "kind": "shell",
            "status": "completed",
            "output": "done"
        })),
        &mut |event| events.push(event),
        &mut latest_usage,
        &mut delta_count,
    );
    forward_acp_message(
        &update_message(json!({
            "sessionUpdate": "usage_update",
            "input_tokens": 5,
            "output_tokens": 6
        })),
        &mut |event| events.push(event),
        &mut latest_usage,
        &mut delta_count,
    );

    assert_eq!(delta_count, 2);
    assert_eq!(
        latest_usage,
        ui_types::TokenUsage {
            input_tokens: 5,
            output_tokens: 6,
            cached_tokens: 0,
            reasoning_tokens: 0,
        }
    );
    assert!(matches!(&events[0], StreamEvent::Delta(text) if text == "hello"));
    assert!(matches!(&events[1], StreamEvent::ReasoningDelta(text) if text == "thinking"));
    assert!(matches!(&events[2], StreamEvent::Delta(text) if text.starts_with("tool shell\n")));
}
