use serde_json::json;
use vw_gateway_client::{GatewayChatStreamEvent, GatewayChatUsage};

use super::stream_adapter::{
    UiRuntimeEvent, UiRuntimeTerminalEvent, adapt_gateway_stream_event,
};

fn usage() -> GatewayChatUsage {
    GatewayChatUsage { input_tokens: 1, output_tokens: 2, cached_tokens: 3, reasoning_tokens: 4 }
}

#[test]
fn done_terminal_preserves_metadata_and_normalizes_blank_finish_reason() {
    let event = adapt_gateway_stream_event(GatewayChatStreamEvent::Done {
        finish_reason: Some("  ".to_string()),
        usage: Some(json!({
            "input_tokens": 1,
            "output_tokens": 2,
            "cached_tokens": 3,
            "reasoning_tokens": 4
        })),
        message_id: Some("assistant_1".to_string()),
        parent_message_id: Some("user_1".to_string()),
    });

    assert_eq!(
        event,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::Done {
            finish_reason: None,
            usage: Some(usage()),
            message_id: Some("assistant_1".to_string()),
            parent_message_id: Some("user_1".to_string()),
        })
    );
}

#[test]
fn done_terminal_classifies_cancel_and_timeout_finish_reasons() {
    let cancelled = adapt_gateway_stream_event(GatewayChatStreamEvent::Done {
        finish_reason: Some(" user cancelled ".to_string()),
        usage: None,
        message_id: None,
        parent_message_id: None,
    });
    let timed_out = adapt_gateway_stream_event(GatewayChatStreamEvent::Done {
        finish_reason: Some("deadline exceeded".to_string()),
        usage: None,
        message_id: Some("m".to_string()),
        parent_message_id: Some("p".to_string()),
    });

    assert!(matches!(
        cancelled,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::Cancelled { .. })
    ));
    assert!(matches!(timed_out, UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::TimedOut { .. })));
}

#[test]
fn error_terminal_normalizes_empty_and_classifies_control_messages() {
    assert_eq!(
        UiRuntimeTerminalEvent::from_error_message("  ".to_string()),
        UiRuntimeTerminalEvent::Error("gateway stream failed".to_string())
    );
    assert!(matches!(
        UiRuntimeTerminalEvent::from_error_message("interrupted by user".to_string()),
        UiRuntimeTerminalEvent::Cancelled { .. }
    ));
    assert!(matches!(
        UiRuntimeTerminalEvent::from_error_message("request timeout".to_string()),
        UiRuntimeTerminalEvent::TimedOut { .. }
    ));
}

#[test]
fn structured_gateway_events_map_to_runtime_events() {
    let step_start = adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
        "type": "chat.step_start",
        "step_index": 7,
        "created_ms": 111,
        "model": "model-a"
    })));
    let step_finish = adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
        "type": "chat.step_finish",
        "step_index": 7,
        "finished_ms": 222,
        "usage": {
            "input_tokens": 1,
            "output_tokens": 2,
            "cached_tokens": 3,
            "reasoning_tokens": 4
        },
        "finish_reason": "stop",
        "model": "model-a"
    })));

    assert_eq!(
        step_start,
        UiRuntimeEvent::StepStart {
            step_index: 7,
            created_ms: 111,
            model: Some("model-a".to_string()),
        }
    );
    assert_eq!(
        step_finish,
        UiRuntimeEvent::StepFinish {
            step_index: 7,
            finished_ms: 222,
            usage: usage(),
            finish_reason: Some("stop".to_string()),
            model: Some("model-a".to_string()),
        }
    );
}

#[test]
fn task_metadata_usage_and_unknown_events_are_explicit() {
    assert_eq!(
        adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.todo_updated",
            "session_id": "s1"
        }))),
        UiRuntimeEvent::TaskStateChanged { session_id: Some("s1".to_string()) }
    );
    assert_eq!(
        adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.session_updated",
            "session_id": "s1",
            "title": "  "
        }))),
        UiRuntimeEvent::SessionMetadataChanged { session_id: Some("s1".to_string()), title: None }
    );
    assert_eq!(
        adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.usage_updated",
            "session_id": "s1",
            "usage": {
                "input_tokens": 1,
                "output_tokens": 2,
                "cached_tokens": 3,
                "reasoning_tokens": 4
            }
        }))),
        UiRuntimeEvent::UsageUpdated { session_id: Some("s1".to_string()), usage: usage() }
    );
    assert_eq!(
        adapt_gateway_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "custom.event"
        }))),
        UiRuntimeEvent::Unknown { event_type: Some("custom.event".to_string()) }
    );
}
