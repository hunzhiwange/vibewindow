use super::{
    VibeWindowChatRequest, VibeWindowResponseMode, VibeWindowSseEvent, WORKFLOW_INPUT_FULL_ACCESS,
    WORKFLOW_INPUT_ROOT, WorkflowSseContext, apply_path_chat_workflow_uuid,
    chat_request_to_workflow_request, compact_sse_node_outputs, node_finished_data,
    node_started_data, node_started_data_from_event, node_status, normalized_optional_string,
    router, strip_workflow_runtime_inputs, text_chunk_data, truncate_sse_node_output,
    validate_path_workflow_uuid, workflow_answer, workflow_delta_should_stream_text,
    workflow_finished_data, workflow_request_full_access, workflow_request_root,
    workflow_response_to_chat_response, workflow_response_to_sse_events,
    workflow_sse_output_preview, workflow_started_data, workflow_status,
    workflow_tool_call_error_text, workflow_tool_runtime_context,
};
use crate::app::agent::tools::ToolCallError;
use crate::workflow::{WorkflowNodeDeltaEvent, WorkflowNodeStartedEvent, WorkflowRunEvent};
use serde_json::json;
use std::collections::BTreeMap;
use tokio::sync::mpsc;
use vw_api_types::workflow::{
    WorkflowNodeRunDto, WorkflowNodeRunStatus, WorkflowRunRequest, WorkflowRunResponse,
    WorkflowRunStatus,
};

#[test]
fn workflow_router_is_wired() {
    let _router = router();
}

#[test]
fn workflow_runtime_inputs_enable_full_access_without_start_input_leak() {
    let mut request = WorkflowRunRequest {
        inputs: BTreeMap::from([
            (WORKFLOW_INPUT_FULL_ACCESS.to_string(), json!(true)),
            (WORKFLOW_INPUT_ROOT.to_string(), json!("/tmp/workflow-root")),
            ("query".to_string(), json!("hello")),
        ]),
        ..WorkflowRunRequest::default()
    };

    assert!(workflow_request_full_access(&request));
    assert_eq!(workflow_request_root(&request).as_deref(), Some("/tmp/workflow-root"));

    let ctx = workflow_tool_runtime_context(&request);
    let tool_use_context = ctx.tool_use_context();
    assert!(tool_use_context.full_access_enabled());
    assert!(tool_use_context.bypass_non_cli_approval_for_turn());
    assert_eq!(ctx.root.as_deref(), Some("/tmp/workflow-root"));

    strip_workflow_runtime_inputs(&mut request);
    assert!(!request.inputs.contains_key(WORKFLOW_INPUT_FULL_ACCESS));
    assert!(!request.inputs.contains_key(WORKFLOW_INPUT_ROOT));
    assert_eq!(request.inputs.get("query"), Some(&json!("hello")));
}

#[test]
fn chat_response_maps_workflow_answer_and_conversation() {
    let response = WorkflowRunResponse {
        run_id: "run-1".to_string(),
        status: WorkflowRunStatus::Succeeded,
        answer: None,
        outputs: BTreeMap::from([("answer".to_string(), json!("ok"))]),
        nodes: Vec::new(),
        error: None,
        pause: None,
    };

    let chat = workflow_response_to_chat_response(&response, Some("conv-1".to_string()));

    assert_eq!(chat.task_id, "run-1");
    assert_eq!(chat.conversation_id, "conv-1");
    assert_eq!(chat.answer, "ok");
    assert_eq!(chat.mode, "advanced-chat");
}

#[test]
fn streaming_events_keep_conversation_id() {
    let response = WorkflowRunResponse {
        run_id: "run-1".to_string(),
        status: WorkflowRunStatus::Succeeded,
        answer: Some("done".to_string()),
        outputs: BTreeMap::new(),
        nodes: Vec::new(),
        error: None,
        pause: None,
    };

    let events = workflow_response_to_sse_events(response, Some("conv-1".to_string()));

    assert!(events.iter().any(|event| matches!(
        event,
        VibeWindowSseEvent::Message { conversation_id, .. } if conversation_id == "conv-1"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        VibeWindowSseEvent::MessageEnd { conversation_id, .. } if conversation_id == "conv-1"
    )));
}

#[test]
fn chat_request_to_workflow_request_accepts_uuid_and_strips_max_steps() {
    let body = VibeWindowChatRequest {
        query: "hello".to_string(),
        application_uuid: Some(" app-1 ".to_string()),
        application_workflow: None,
        inputs: BTreeMap::from([
            ("max_steps".to_string(), json!(12)),
            ("topic".to_string(), json!("rust")),
        ]),
        response_mode: VibeWindowResponseMode::Blocking,
        user: "user-1".to_string(),
        conversation_id: None,
        files: None,
        auto_generate_name: None,
    };

    let request = chat_request_to_workflow_request(body).expect("request should convert");

    assert_eq!(request.workflow_uuid.as_deref(), Some("app-1"));
    assert_eq!(request.workflow_yaml, None);
    assert_eq!(request.query.as_deref(), Some("hello"));
    assert_eq!(request.max_steps, 12);
    assert_eq!(request.inputs, BTreeMap::from([("topic".to_string(), json!("rust"))]));
}

#[test]
fn chat_request_to_workflow_request_rejects_legacy_input_fields() {
    for key in ["application_uuid", "application_workflow", "workflow_uuid", "workflow_yaml"] {
        let body = VibeWindowChatRequest {
            query: "hello".to_string(),
            application_uuid: Some("app-1".to_string()),
            inputs: BTreeMap::from([(key.to_string(), json!("legacy"))]),
            user: "user-1".to_string(),
            ..VibeWindowChatRequest::default()
        };

        let error = chat_request_to_workflow_request(body).expect_err("legacy key should fail");

        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
        assert!(error.to_string().contains(key));
    }
}

#[test]
fn chat_request_to_workflow_request_requires_uuid_or_yaml_and_defaults_steps() {
    let missing_workflow = VibeWindowChatRequest {
        query: "hello".to_string(),
        user: "user-1".to_string(),
        ..VibeWindowChatRequest::default()
    };

    let error =
        chat_request_to_workflow_request(missing_workflow).expect_err("workflow is required");

    assert_eq!(error.to_string(), "application_uuid or application_workflow is required");

    let invalid_max_steps = VibeWindowChatRequest {
        query: "hello".to_string(),
        application_workflow: Some("yaml".to_string()),
        inputs: BTreeMap::from([("max_steps".to_string(), json!("many"))]),
        user: "user-1".to_string(),
        ..VibeWindowChatRequest::default()
    };

    let request =
        chat_request_to_workflow_request(invalid_max_steps).expect("invalid max uses default");

    assert_eq!(request.max_steps, 200);
    assert!(!request.inputs.contains_key("max_steps"));
}

#[test]
fn path_workflow_uuid_is_trimmed_and_blocks_body_overrides() {
    assert_eq!(validate_path_workflow_uuid(" app-1 ").expect("valid uuid"), "app-1");
    assert_eq!(
        validate_path_workflow_uuid("   ").expect_err("blank uuid").to_string(),
        "workflow uuid is required"
    );

    let mut body = VibeWindowChatRequest {
        query: "hello".to_string(),
        user: "user-1".to_string(),
        ..VibeWindowChatRequest::default()
    };

    apply_path_chat_workflow_uuid(&mut body, " app-2 ").expect("path uuid should apply");

    assert_eq!(body.application_uuid.as_deref(), Some("app-2"));

    for (field, body) in [
        (
            "application_uuid",
            VibeWindowChatRequest {
                application_uuid: Some("body-app".to_string()),
                ..VibeWindowChatRequest::default()
            },
        ),
        (
            "application_workflow",
            VibeWindowChatRequest {
                application_workflow: Some("yaml".to_string()),
                ..VibeWindowChatRequest::default()
            },
        ),
        (
            "inputs.workflow_yaml",
            VibeWindowChatRequest {
                inputs: BTreeMap::from([("workflow_yaml".to_string(), json!("yaml"))]),
                ..VibeWindowChatRequest::default()
            },
        ),
    ] {
        let mut body = body;
        let error =
            apply_path_chat_workflow_uuid(&mut body, "app-1").expect_err("override should fail");

        assert!(error.to_string().contains(field));
    }
}

#[test]
fn optional_string_normalization_drops_blank_values() {
    assert_eq!(normalized_optional_string(Some(" value ".to_string())).as_deref(), Some("value"));
    assert_eq!(normalized_optional_string(Some("   ".to_string())), None);
    assert_eq!(normalized_optional_string(None), None);
}

#[test]
fn runtime_inputs_default_to_restricted_context() {
    let request = WorkflowRunRequest {
        inputs: BTreeMap::from([
            (WORKFLOW_INPUT_FULL_ACCESS.to_string(), json!("true")),
            (WORKFLOW_INPUT_ROOT.to_string(), json!("   ")),
        ]),
        ..WorkflowRunRequest::default()
    };

    let ctx = workflow_tool_runtime_context(&request);

    assert!(!workflow_request_full_access(&request));
    assert_eq!(workflow_request_root(&request), None);
    assert!(!ctx.tool_use_context().full_access_enabled());
    assert!(!ctx.tool_use_context().bypass_non_cli_approval_for_turn());
    assert_eq!(ctx.root, None);
}

#[test]
fn workflow_answer_prefers_explicit_answer_then_known_outputs() {
    let explicit = WorkflowRunResponse {
        answer: Some("explicit".to_string()),
        outputs: BTreeMap::from([("answer".to_string(), json!("output answer"))]),
        ..workflow_response("run-1")
    };
    let from_text = WorkflowRunResponse {
        outputs: BTreeMap::from([("text".to_string(), json!("text answer"))]),
        ..workflow_response("run-2")
    };
    let from_result = WorkflowRunResponse {
        outputs: BTreeMap::from([("result".to_string(), json!("result answer"))]),
        ..workflow_response("run-3")
    };

    assert_eq!(workflow_answer(&explicit), "explicit");
    assert_eq!(workflow_answer(&from_text), "text answer");
    assert_eq!(workflow_answer(&from_result), "result answer");
    assert_eq!(workflow_answer(&workflow_response("run-4")), "");
}

#[test]
fn output_preview_uses_known_text_keys_before_json_fallback() {
    let outputs = BTreeMap::from([
        ("answer".to_string(), json!("  answer text  ")),
        ("data".to_string(), json!("data text")),
    ]);
    let json_outputs = BTreeMap::from([("count".to_string(), json!(2))]);

    assert_eq!(workflow_sse_output_preview(&outputs), "answer text");
    assert!(workflow_sse_output_preview(&json_outputs).contains("\"count\": 2"));
}

#[test]
fn compact_outputs_preserve_usage_and_mark_truncation() {
    let long_answer = "a".repeat(super::WORKFLOW_SSE_NODE_OUTPUT_MAX_CHARS + 1);
    let outputs = BTreeMap::from([
        ("answer".to_string(), json!(long_answer)),
        ("usage".to_string(), json!({"total_tokens": 9})),
    ]);

    let compacted = compact_sse_node_outputs(&outputs);

    assert_eq!(compacted["truncated"], json!(true));
    assert_eq!(compacted["usage"], json!({"total_tokens": 9}));
    assert_eq!(compacted["answer"], compacted["text"]);
    assert_eq!(compacted["result"], compacted["text"]);
}

#[test]
fn truncation_keeps_short_text_and_splits_long_text() {
    let (short, short_truncated) = truncate_sse_node_output("短文本");
    let long =
        format!("{}{}", "头".repeat(super::WORKFLOW_SSE_NODE_OUTPUT_MAX_CHARS), "尾".repeat(10));

    let (truncated, long_truncated) = truncate_sse_node_output(&long);

    assert_eq!(short, "短文本");
    assert!(!short_truncated);
    assert!(long_truncated);
    assert!(truncated.contains("已截断"));
    assert!(truncated.starts_with('头'));
    assert!(truncated.ends_with('尾'));
}

#[test]
fn node_and_workflow_statuses_match_protocol_values() {
    assert_eq!(workflow_status(&WorkflowRunStatus::Running), "running");
    assert_eq!(workflow_status(&WorkflowRunStatus::Paused), "running");
    assert_eq!(workflow_status(&WorkflowRunStatus::Succeeded), "succeeded");
    assert_eq!(workflow_status(&WorkflowRunStatus::Failed), "failed");
    assert_eq!(node_status(&WorkflowNodeRunStatus::Paused), "running");
    assert_eq!(node_status(&WorkflowNodeRunStatus::Succeeded), "succeeded");
    assert_eq!(node_status(&WorkflowNodeRunStatus::Failed), "failed");
}

#[test]
fn node_data_maps_started_and_finished_payloads() {
    let node = workflow_node(
        "node-1",
        "llm",
        WorkflowNodeRunStatus::Failed,
        BTreeMap::from([("answer".to_string(), json!("node answer"))]),
    );

    let started = node_started_data(&node, 2, 111);
    let finished = node_finished_data(&node, 2, 111);

    assert_eq!(started.id, "node-1-started");
    assert_eq!(started.inputs, Some(json!({"input": "value"})));
    assert_eq!(finished.id, "node-1-finished");
    assert_eq!(finished.status, "failed");
    assert_eq!(finished.error.as_deref(), Some("boom"));
    assert_eq!(finished.elapsed_time, Some(1.5));
    assert_eq!(finished.outputs.expect("outputs")["text"], json!("node answer"));
}

#[test]
fn event_node_started_data_omits_runtime_inputs() {
    let event = WorkflowNodeStartedEvent {
        node_id: "node-1".to_string(),
        node_type: "answer".to_string(),
        title: "Answer".to_string(),
        index: 3,
    };

    let data = node_started_data_from_event(&event, 222);

    assert_eq!(data.id, "node-1-started");
    assert_eq!(data.node_type, "answer");
    assert_eq!(data.title, "Answer");
    assert_eq!(data.index, 3);
    assert_eq!(data.inputs, None);
}

#[test]
fn workflow_started_and_finished_data_match_protocol_shape() {
    let response = WorkflowRunResponse {
        status: WorkflowRunStatus::Failed,
        outputs: BTreeMap::from([("text".to_string(), json!("failed output"))]),
        nodes: vec![
            workflow_node("a", "start", WorkflowNodeRunStatus::Succeeded, BTreeMap::new()),
            workflow_node("b", "end", WorkflowNodeRunStatus::Failed, BTreeMap::new()),
        ],
        error: Some("failed".to_string()),
        ..workflow_response("run-1")
    };

    let started = workflow_started_data("run-1", 10);
    let finished = workflow_finished_data(&response, 10);

    assert_eq!(started.id, "run-1");
    assert_eq!(started.workflow_id, "vibewindow-workflow");
    assert_eq!(started.sequence_number, Some(1));
    assert_eq!(finished.id, "run-1");
    assert_eq!(finished.status, "failed");
    assert_eq!(finished.error.as_deref(), Some("failed"));
    assert_eq!(finished.total_steps, 2);
    assert_eq!(finished.outputs.expect("outputs")["text"], json!("failed output"));
}

#[test]
fn delta_streaming_rules_only_emit_supported_text_once() {
    let final_llm = workflow_delta("final_response", "llm", false);
    let replacement_llm = workflow_delta("final_response", "llm", true);
    let answer = workflow_delta("answer-1", "answer", false);

    assert!(workflow_delta_should_stream_text(&final_llm, false));
    assert!(workflow_delta_should_stream_text(&final_llm, true));
    assert!(!workflow_delta_should_stream_text(&replacement_llm, false));
    assert!(!workflow_delta_should_stream_text(&workflow_delta("other", "llm", false), false));
    assert!(workflow_delta_should_stream_text(&answer, false));
    assert!(!workflow_delta_should_stream_text(&answer, true));
    assert!(!workflow_delta_should_stream_text(&workflow_delta("code", "tool", false), false));
}

#[test]
fn text_chunk_uses_node_text_selector() {
    let event = workflow_delta("answer-1", "answer", false);

    let data = text_chunk_data(&event);

    assert_eq!(data.text, "delta text");
    assert_eq!(data.from_variable_selector, vec!["answer-1", "text"]);
}

#[test]
fn sse_events_include_node_lifecycle_message_and_end() {
    let response = WorkflowRunResponse {
        answer: None,
        outputs: BTreeMap::from([("answer".to_string(), json!("final answer"))]),
        nodes: vec![workflow_node(
            "node-1",
            "answer",
            WorkflowNodeRunStatus::Succeeded,
            BTreeMap::from([("answer".to_string(), json!("node answer"))]),
        )],
        ..workflow_response("run-1")
    };

    let events = workflow_response_to_sse_events(response, None);

    assert_eq!(events.len(), 6);
    assert!(matches!(events[0], VibeWindowSseEvent::WorkflowStarted { .. }));
    assert!(matches!(events[1], VibeWindowSseEvent::NodeStarted { .. }));
    assert!(matches!(events[2], VibeWindowSseEvent::NodeFinished { .. }));
    assert!(matches!(
        &events[3],
        VibeWindowSseEvent::Message { answer, .. } if answer == "final answer"
    ));
    assert!(matches!(events[4], VibeWindowSseEvent::WorkflowFinished { .. }));
    assert!(matches!(events[5], VibeWindowSseEvent::MessageEnd { .. }));
}

#[test]
fn sse_events_skip_empty_message() {
    let events = workflow_response_to_sse_events(workflow_response("run-1"), None);

    assert!(!events.iter().any(|event| matches!(event, VibeWindowSseEvent::Message { .. })));
    assert_eq!(events.len(), 3);
}

#[tokio::test]
async fn sse_context_ignores_node_events_until_run_starts() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut context = WorkflowSseContext::new(Some("conv-1".to_string()));

    context.send_event(&tx, WorkflowRunEvent::NodeStarted(workflow_started_event("node-1")));
    context.send_event(&tx, WorkflowRunEvent::NodeDelta(workflow_delta("answer", "answer", false)));
    context.send_event(&tx, WorkflowRunEvent::WorkflowStarted { run_id: "run-1".to_string() });

    let _ = rx.recv().await.expect("workflow started event");
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn sse_context_streams_first_delta_then_suppresses_finished_message() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut context = WorkflowSseContext::new(Some("conv-1".to_string()));

    context.send_event(&tx, WorkflowRunEvent::WorkflowStarted { run_id: "run-1".to_string() });
    context.send_event(&tx, WorkflowRunEvent::NodeDelta(workflow_delta("answer", "answer", false)));
    context.send_finished(
        &tx,
        WorkflowRunResponse {
            answer: Some("final answer".to_string()),
            ..workflow_response("run-1")
        },
    );

    let event_count = drain_sse_count(&mut rx);

    assert_eq!(event_count, 4);
}

#[tokio::test]
async fn sse_context_finished_emits_message_when_no_text_was_streamed() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut context = WorkflowSseContext::new(Some("conv-1".to_string()));

    context.send_finished(
        &tx,
        WorkflowRunResponse {
            answer: Some("final answer".to_string()),
            ..workflow_response("run-1")
        },
    );

    let event_count = drain_sse_count(&mut rx);

    assert_eq!(event_count, 3);
}

#[tokio::test]
async fn sse_context_error_uses_current_run_or_default_task() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut context = WorkflowSseContext::new(None);

    context.send_error(&tx, 400, "request_error", "bad request");
    context.send_event(&tx, WorkflowRunEvent::WorkflowStarted { run_id: "run-1".to_string() });
    context.send_error(&tx, 500, "run_error", "failed");

    let event_count = drain_sse_count(&mut rx);

    assert_eq!(event_count, 3);
}

#[test]
fn tool_call_error_text_keeps_denied_or_failed_message_only() {
    let denied = ToolCallError::Denied { message: "denied".to_string(), permission_request: None };
    let failed = ToolCallError::Failed("failed".to_string());

    assert_eq!(workflow_tool_call_error_text(denied), "denied");
    assert_eq!(workflow_tool_call_error_text(failed), "failed");
}

fn workflow_response(run_id: &str) -> WorkflowRunResponse {
    WorkflowRunResponse {
        run_id: run_id.to_string(),
        status: WorkflowRunStatus::Succeeded,
        answer: None,
        outputs: BTreeMap::new(),
        nodes: Vec::new(),
        error: None,
        pause: None,
    }
}

fn workflow_node(
    node_id: &str,
    node_type: &str,
    status: WorkflowNodeRunStatus,
    outputs: BTreeMap<String, serde_json::Value>,
) -> WorkflowNodeRunDto {
    WorkflowNodeRunDto {
        node_id: node_id.to_string(),
        node_type: node_type.to_string(),
        title: "Node".to_string(),
        status,
        inputs: BTreeMap::from([("input".to_string(), json!("value"))]),
        outputs,
        selected_handle: None,
        error: Some("boom".to_string()),
        elapsed_ms: 1500,
    }
}

fn workflow_started_event(node_id: &str) -> WorkflowNodeStartedEvent {
    WorkflowNodeStartedEvent {
        node_id: node_id.to_string(),
        node_type: "answer".to_string(),
        title: "Answer".to_string(),
        index: 1,
    }
}

fn workflow_delta(node_id: &str, node_type: &str, replace: bool) -> WorkflowNodeDeltaEvent {
    WorkflowNodeDeltaEvent {
        node_id: node_id.to_string(),
        node_type: node_type.to_string(),
        title: "Delta".to_string(),
        index: 1,
        text: "delta text".to_string(),
        replace,
    }
}

fn drain_sse_count(rx: &mut mpsc::UnboundedReceiver<axum::response::sse::Event>) -> usize {
    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }
    count
}
