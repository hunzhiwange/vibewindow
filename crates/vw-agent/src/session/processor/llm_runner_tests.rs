#[test]
fn llm_runner_tests_module_is_wired() {
    let marker = String::from("llm_runner_tests");
    assert_eq!(marker.as_str(), "llm_runner_tests");
}

use crate::app::agent::session::llm;
use crate::app::agent::session::message::AssistantError;
use crate::session::ui_types as models;
use std::sync::mpsc;
use std::time::Duration;

fn collect_events(
    events: impl IntoIterator<Item = llm::StreamEvent>,
) -> (Result<super::LlmStep, String>, Vec<String>) {
    let (tx, rx) = mpsc::channel();
    for event in events {
        tx.send(event).expect("stream event should be queued");
    }
    drop(tx);

    let mut deltas = Vec::new();
    let result = super::collect_llm_step_from_stream(
        rx,
        &mut |event| {
            if let super::StreamEvent::Delta(text) = event {
                deltas.push(text);
            }
            true
        },
        Duration::from_millis(5),
    );
    (result, deltas)
}

fn usage(input_tokens: i64, output_tokens: i64) -> models::TokenUsage {
    models::TokenUsage { input_tokens, output_tokens, cached_tokens: 3, reasoning_tokens: 4 }
}

#[test]
fn acp_request_detection_accepts_test_flag_and_agent_name() {
    assert!(super::is_acp_request(&serde_json::json!({ "acp_test": true })));
    assert!(super::is_acp_request(&serde_json::json!({ "acp_agent": "codex" })));
    assert!(!super::is_acp_request(&serde_json::json!({ "acp_agent": "  " })));
    assert!(!super::is_acp_request(&serde_json::json!({})));
}

#[test]
fn acp_retryable_error_matches_timeout_and_disconnect_messages() {
    assert!(super::is_acp_retryable_error("模型响应超时"));
    assert!(super::is_acp_retryable_error("request TIMED OUT"));
    assert!(super::is_acp_retryable_error("acp agent disconnected during request"));
    assert!(super::is_acp_retryable_error("queue owner disconnected before prompt completion"));
    assert!(!super::is_acp_retryable_error("invalid api key"));
}

#[test]
fn non_retryable_llm_error_matches_model_resolution_failures() {
    assert!(super::is_non_retryable_llm_error(
        r#"{"name":"Unknown","message":"未找到模型：llama3"}"#
    ));
    assert!(super::is_non_retryable_llm_error("模型格式错误：/llama3，请使用 provider/model"));
    assert!(super::is_non_retryable_llm_error("模型ID存在歧义：llama3，请使用 provider/model"));
    assert!(super::is_non_retryable_llm_error("model not found"));
    assert!(!super::is_non_retryable_llm_error("模型响应超时"));
}

#[test]
fn llm_step_retry_allowed_blocks_model_config_errors_only() {
    assert!(!super::llm_step_retry_allowed(
        r#"{"name":"Unknown","message":"未找到模型：llama3"}"#,
        false,
    ));
    assert!(super::llm_step_retry_allowed("模型响应超时", false));
    assert!(super::llm_step_retry_allowed("模型响应超时", true));
    assert!(!super::llm_step_retry_allowed("invalid api key", true));
}

#[test]
fn collect_stream_returns_text_tool_calls_usage_and_full_messages() {
    let tool_call = llm::ToolCall {
        id: "call-1".to_string(),
        name: "file_read".to_string(),
        arguments: r#"{"path":"README.md"}"#.to_string(),
    };
    let full_messages = vec![serde_json::json!({ "role": "system", "content": "base" })];

    let (result, deltas) = collect_events([
        llm::StreamEvent::Delta("hello ".to_string()),
        llm::StreamEvent::Delta("world".to_string()),
        llm::StreamEvent::ToolCalls(vec![tool_call.clone()]),
        llm::StreamEvent::FullMessages(full_messages.clone()),
        llm::StreamEvent::Done {
            usage: usage(11, 7),
            finish_reason: Some("tool_calls".to_string()),
        },
    ]);

    let step = result.expect("stream should produce a step");
    assert_eq!(step.text, "hello world");
    assert_eq!(step.tool_calls[0].id, tool_call.id);
    assert_eq!(step.full_messages, full_messages);
    assert_eq!(step.finish_reason.as_deref(), Some("tool_calls"));
    assert_eq!(step.usage.input_tokens, 11);
    assert_eq!(step.usage.output_tokens, 7);
    assert_eq!(deltas.join(""), "hello world");
}

#[test]
fn collect_stream_wraps_reasoning_before_text() {
    let (result, deltas) = collect_events([
        llm::StreamEvent::ReasoningDelta("plan".to_string()),
        llm::StreamEvent::Delta("answer".to_string()),
        llm::StreamEvent::Done {
            usage: models::TokenUsage::default(),
            finish_reason: Some("stop".to_string()),
        },
    ]);

    let step = result.expect("reasoning plus text should be valid");
    assert_eq!(step.reasoning_content, "plan");
    assert_eq!(step.text, "answer");
    assert_eq!(deltas, vec!["<think>plan", "</think>\n\n", "answer"]);
}

#[test]
fn collect_stream_uses_reasoning_as_fallback_text_when_no_content_arrives() {
    let (result, deltas) = collect_events([
        llm::StreamEvent::ReasoningDelta("only thoughts".to_string()),
        llm::StreamEvent::Done {
            usage: models::TokenUsage::default(),
            finish_reason: Some("stop".to_string()),
        },
    ]);

    let step = result.expect("reasoning fallback should count as content");
    assert_eq!(step.text, "only thoughts");
    assert_eq!(step.reasoning_content, "only thoughts");
    assert_eq!(deltas, vec!["<think>only thoughts", "</think>\n\n"]);
}

#[test]
fn collect_stream_emits_late_reasoning_after_text() {
    let (result, deltas) = collect_events([
        llm::StreamEvent::Delta("answer".to_string()),
        llm::StreamEvent::ReasoningDelta("late plan".to_string()),
        llm::StreamEvent::Done {
            usage: models::TokenUsage::default(),
            finish_reason: Some("stop".to_string()),
        },
    ]);

    let step = result.expect("text should make the step valid");
    assert_eq!(step.text, "answer");
    assert_eq!(step.reasoning_content, "late plan");
    assert_eq!(deltas, vec!["answer", "\n\n<think>late plan</think>\n\n"]);
}

#[test]
fn collect_stream_rejects_done_without_text_or_tools() {
    let (result, deltas) = collect_events([llm::StreamEvent::Done {
        usage: models::TokenUsage::default(),
        finish_reason: Some("stop".to_string()),
    }]);

    assert_eq!(result.unwrap_err(), "模型未返回内容");
    assert!(deltas.is_empty());
}

#[test]
fn collect_stream_allows_tool_only_steps() {
    let (result, deltas) = collect_events([
        llm::StreamEvent::ToolCalls(vec![llm::ToolCall {
            id: "call-1".to_string(),
            name: "TodoWrite".to_string(),
            arguments: "{}".to_string(),
        }]),
        llm::StreamEvent::Done {
            usage: models::TokenUsage::default(),
            finish_reason: Some("tool_calls".to_string()),
        },
    ]);

    let step = result.expect("tool-only response should be valid");
    assert!(step.text.is_empty());
    assert_eq!(step.tool_calls.len(), 1);
    assert!(deltas.is_empty());
}

#[test]
fn collect_stream_flushes_pending_content_on_error() {
    let (result, deltas) = collect_events([
        llm::StreamEvent::ReasoningDelta("before error".to_string()),
        llm::StreamEvent::Error(AssistantError::Unknown { message: "boom".to_string() }),
    ]);

    let message = result.unwrap_err();
    assert!(message.contains("boom"));
    assert_eq!(deltas, vec!["<think>before error", "</think>\n\n"]);
}

#[test]
fn collect_stream_times_out_and_closes_open_reasoning() {
    let (tx, rx) = mpsc::channel();
    tx.send(llm::StreamEvent::ReasoningDelta("slow".to_string()))
        .expect("reasoning delta should be queued");
    let _keep_sender_alive = tx;

    let mut deltas = Vec::new();
    let result = super::collect_llm_step_from_stream(
        rx,
        &mut |event| {
            if let super::StreamEvent::Delta(text) = event {
                deltas.push(text);
            }
            true
        },
        Duration::from_millis(1),
    );

    assert_eq!(result.unwrap_err(), "模型响应超时");
    assert_eq!(deltas, vec!["<think>slow", "</think>\n\n"]);
}

#[test]
fn collect_stream_reports_empty_when_sender_disconnects_without_done() {
    let (tx, rx) = mpsc::channel();
    drop(tx);

    let mut deltas = Vec::new();
    let result = super::collect_llm_step_from_stream(
        rx,
        &mut |event| {
            if let super::StreamEvent::Delta(text) = event {
                deltas.push(text);
            }
            true
        },
        Duration::from_millis(5),
    );

    assert_eq!(result.unwrap_err(), "模型未返回内容");
    assert!(deltas.is_empty());
}
