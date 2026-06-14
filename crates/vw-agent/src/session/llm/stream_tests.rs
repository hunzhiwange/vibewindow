use super::super::types::{AgentInfo, Error, StreamInput};
use super::*;

use crate::app::agent::provider::provider;
use crate::app::agent::session::message;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn test_model(provider_id: &str, api_id: &str, adapter: &str) -> provider::Model {
    serde_json::from_value(json!({
        "id": api_id,
        "providerID": provider_id,
        "api": {
            "id": api_id,
            "url": "http://localhost",
            "adapter": adapter
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

fn test_input(provider_id: &str) -> StreamInput {
    let model = test_model(provider_id, "model-1", "openai-compatible");
    StreamInput {
        user: message::UserInfo {
            id: "user-1".to_string(),
            session_id: "session-1".to_string(),
            time: message::UserTime { created: 123 },
            summary: None,
            agent: "build".to_string(),
            model: message::ModelRef {
                provider_id: model.provider_id.clone(),
                model_id: model.id.clone(),
            },
            system: None,
            tools: None,
            variant: None,
        },
        session_id: "session-1".to_string(),
        model,
        agent: AgentInfo {
            name: "build".to_string(),
            mode: "build".to_string(),
            prompt: None,
            temperature: None,
            top_p: None,
            options: HashMap::new(),
            permission: Default::default(),
        },
        system: Vec::new(),
        abort: None,
        messages: vec![json!({ "role": "user", "content": "hello" })],
        small: false,
        tools: HashMap::new(),
        retries: 0,
    }
}

#[test]
fn should_abort_reflects_watch_receiver_state() {
    assert!(!should_abort(None));

    let (tx, rx) = tokio::sync::watch::channel(false);
    assert!(!should_abort(Some(&rx)));

    tx.send(true).expect("abort signal should send");
    assert!(should_abort(Some(&rx)));
}

#[test]
fn is_retryable_assistant_error_uses_api_retry_flag_and_acp_session_change() {
    assert!(is_retryable_assistant_error(&message::AssistantError::APIError {
        message: "temporary".to_string(),
        status_code: Some(503),
        is_retryable: true,
        response_headers: None,
        response_body: None,
        metadata: None,
    }));
    assert!(!is_retryable_assistant_error(&message::AssistantError::APIError {
        message: "bad request".to_string(),
        status_code: Some(400),
        is_retryable: false,
        response_headers: None,
        response_body: None,
        metadata: None,
    }));
    assert!(is_retryable_assistant_error(&message::AssistantError::Unknown {
        message: "acp session changed: expected=a actual=b".to_string(),
    }));
    assert!(!is_retryable_assistant_error(&message::AssistantError::MessageAbortedError {
        message: "aborted".to_string(),
    }));
}

#[test]
fn is_acp_retryable_assistant_error_accepts_transient_acp_failures() {
    for message in [
        "timed out waiting for agent",
        "timeout while prompting",
        "ACP agent disconnected during request",
        "queue owner disconnected before prompt completion",
        "acp session changed: expected=a actual=b",
    ] {
        assert!(is_acp_retryable_assistant_error(&message::AssistantError::Unknown {
            message: message.to_string(),
        }));
    }

    assert!(is_acp_retryable_assistant_error(&message::AssistantError::APIError {
        message: "request timeout".to_string(),
        status_code: Some(504),
        is_retryable: false,
        response_headers: None,
        response_body: None,
        metadata: None,
    }));
    assert!(!is_acp_retryable_assistant_error(&message::AssistantError::ProviderAuthError {
        provider_id: "openai".to_string(),
        message: "missing key".to_string(),
    }));
}

#[test]
fn is_acp_session_changed_message_is_case_insensitive() {
    assert!(is_acp_session_changed_message("ACP SESSION CHANGED: expected=a actual=b"));
    assert!(is_acp_session_changed_message("prefix acp session changed: expected=a actual=b"));
    assert!(!is_acp_session_changed_message("session changed without acp prefix"));
}

#[tokio::test]
async fn stream_returns_provider_not_found_before_requesting() {
    let input = test_input("__missing_provider_for_llm_stream_test__");
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_for_callback = events.clone();

    let result = stream(input, move |event| {
        events_for_callback.lock().unwrap().push(event);
    })
    .await;

    match result {
        Err(Error::ProviderNotFound(provider_id)) => {
            assert_eq!(provider_id, "__missing_provider_for_llm_stream_test__");
        }
        other => panic!("expected provider not found, got {other:?}"),
    }
    assert!(events.lock().unwrap().is_empty());
}
