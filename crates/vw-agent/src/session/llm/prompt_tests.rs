use super::super::Error;
use super::super::types::{PromptStreamEvent, StreamEvent};
use super::*;

use crate::app::agent::provider::provider;
use crate::app::agent::session::message;
use serde_json::json;
use std::sync::{Arc, Mutex};

fn test_model(provider_id: &str, api_id: &str) -> provider::Model {
    serde_json::from_value(json!({
        "id": api_id,
        "providerID": provider_id,
        "api": {
            "id": api_id,
            "url": "http://localhost",
            "adapter": "openai-compatible"
        },
        "name": api_id,
        "family": null,
        "capabilities": {
            "temperature": true,
            "reasoning": true,
            "attachment": false,
            "toolcall": true,
            "input": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "output": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "interleaved": false
        },
        "cost": {
            "input": 0.0,
            "output": 0.0,
            "cache": {
                "read": 0.0,
                "write": 0.0
            },
            "experimental_over_200k": null
        },
        "limit": {
            "context": 8192,
            "input": null,
            "output": 4096
        },
        "status": "active",
        "options": {},
        "headers": {},
        "release_date": "2026-01-01",
        "variants": {}
    }))
    .expect("test model should deserialize")
}

#[test]
fn now_ms_returns_a_unix_timestamp() {
    assert!(now_ms() > 0);
}

#[test]
fn dummy_user_carries_session_and_model_references() {
    let model = test_model("provider-1", "model-1");

    let user = dummy_user("session-1", &model);

    assert!(user.id.starts_with("prompt-"));
    assert_eq!(user.session_id, "session-1");
    assert_eq!(user.agent, "build");
    assert_eq!(user.model.provider_id, "provider-1");
    assert_eq!(user.model.model_id, "model-1");
    assert!(user.time.created > 0);
    assert!(user.summary.is_none());
    assert!(user.system.is_none());
    assert!(user.tools.is_none());
    assert!(user.variant.is_none());
}

#[test]
fn block_on_runs_futures_without_an_existing_runtime() {
    let value = block_on(async { 42 });

    assert_eq!(value, 42);
}

#[tokio::test(flavor = "multi_thread")]
async fn block_on_runs_futures_inside_a_multi_thread_runtime() {
    let value = block_on(async { "inside runtime".to_string() });

    assert_eq!(value, "inside runtime");
}

#[test]
fn assistant_error_to_string_formats_all_variants() {
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::ProviderAuthError {
            provider_id: "openai".to_string(),
            message: "missing key".to_string(),
        }),
        "未配置 openai 的 API Key：missing key"
    );
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::MessageOutputLengthError),
        "模型输出过长"
    );
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::MessageAbortedError {
            message: "stopped".to_string(),
        }),
        "stopped"
    );
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::ContextOverflowError {
            message: "too long".to_string(),
            response_body: Some("body".to_string()),
        }),
        "too long"
    );
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::APIError {
            message: "rate limited".to_string(),
            status_code: Some(429),
            is_retryable: true,
            response_headers: None,
            response_body: None,
            metadata: None,
        }),
        "rate limited"
    );
    assert_eq!(
        assistant_error_to_string(&message::AssistantError::Unknown {
            message: "unknown".to_string(),
        }),
        "unknown"
    );
}

#[test]
fn missing_stream_error_message_maps_outer_errors_only() {
    assert!(missing_stream_error_message(&Ok(())).is_none());
    assert!(missing_stream_error_message(&Err(Error::Aborted)).is_none());
    assert!(
        missing_stream_error_message(&Err(Error::Api(message::AssistantError::Unknown {
            message: "already emitted".to_string(),
        })))
        .is_none()
    );
    assert_eq!(
        missing_stream_error_message(&Err(Error::ProviderNotFound("missing-provider".to_string()))),
        Some("未找到 provider：missing-provider".to_string())
    );
}

#[test]
fn prompt_entrypoints_report_parse_errors_without_streaming() {
    let prompt_events = Arc::new(Mutex::new(Vec::new()));
    let prompt_events_for_callback = prompt_events.clone();
    stream_prompt("hello", Some("/model".to_string()), move |event| {
        prompt_events_for_callback.lock().unwrap().push(event);
    });

    let events = prompt_events.lock().unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        PromptStreamEvent::Error(message) => assert!(message.contains("模型格式错误")),
        other => panic!("expected error event, got {other:?}"),
    }
    drop(events);

    let tool_events = Arc::new(Mutex::new(Vec::new()));
    let tool_events_for_callback = tool_events.clone();
    stream_prompt_with_tools(
        "hello",
        Some("provider/".to_string()),
        Default::default(),
        move |event| {
            tool_events_for_callback.lock().unwrap().push(event);
        },
    );

    let events = tool_events.lock().unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        StreamEvent::Error(message::AssistantError::Unknown { message }) => {
            assert!(message.contains("模型格式错误"));
        }
        other => panic!("expected stream error event, got {other:?}"),
    }
}

#[test]
fn chat_entrypoint_reports_parse_errors_without_streaming() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_for_callback = events.clone();

    stream_chat_with_tools_for_session(
        "session-1",
        vec![json!({ "role": "user", "content": "hello" })],
        vec!["system".to_string()],
        Some("provider/".to_string()),
        json!({ "temperature": 0.5 }),
        Default::default(),
        move |event| {
            events_for_callback.lock().unwrap().push(event);
        },
    );

    let events = events.lock().unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        StreamEvent::Error(message::AssistantError::Unknown { message }) => {
            assert!(message.contains("模型格式错误"));
        }
        other => panic!("expected stream error event, got {other:?}"),
    }
}

#[test]
fn send_prompt_returns_parse_errors_without_streaming() {
    let err = send_prompt("hello", Some("provider/".to_string()))
        .expect_err("invalid model ref should fail before streaming");

    assert!(err.contains("模型格式错误"));
}
