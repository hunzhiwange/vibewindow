//! 验证 TUI v2 runtime 的异步事件桥接。
//! 测试聚焦运行时与模型之间的边界，避免后台事件破坏 UI 状态。

use std::path::PathBuf;

use vw_gateway_client::vw_api_types::todo::{TodoPriority, TodoStatus};
use vw_gateway_client::{
    GatewayChatStreamEvent, GatewayChatStreamRequest, GatewayClient, GatewayEndpoint,
    GatewaySessionTodoItem, GatewaySessionTodoPutBody,
};
use vw_shared::question::{Info, OptionInfo, Request};
use vw_shared::todo::Todo;

use super::gateway::{GatewaySessionSeed, GatewayUiRuntime};
use super::question_poller::{filter_questions_for_session, todo_put_body};
use super::stream_adapter::{UiRuntimeEvent, UiRuntimeTerminalEvent, adapt_gateway_stream_event};

fn runtime_with_session_id(session_id: &str) -> GatewayUiRuntime {
    let client = GatewayClient::new(GatewayEndpoint::new("127.0.0.1", 42617))
        .expect("gateway client should construct in unit tests");
    let session = GatewaySessionSeed::new(PathBuf::from("/tmp/runtime-tests"))
        .with_id(Some(session_id.to_string()));
    GatewayUiRuntime::new(client, session)
}

#[test]
fn adapt_gateway_stream_event_maps_done_to_terminal_done() {
    let event = adapt_gateway_stream_event(GatewayChatStreamEvent::Done {
        finish_reason: Some("stop".to_string()),
        usage: None,
        message_id: Some("msg_assistant".to_string()),
        parent_message_id: Some("msg_user".to_string()),
    });

    assert_eq!(
        event,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: None,
            message_id: Some("msg_assistant".to_string()),
            parent_message_id: Some("msg_user".to_string()),
        })
    );
}

#[test]
fn adapt_gateway_stream_event_maps_timeout_error_to_terminal_timeout() {
    let event = adapt_gateway_stream_event(GatewayChatStreamEvent::Error(
        "request timed out after 30s".to_string(),
    ));

    assert_eq!(
        event,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::TimedOut {
            message: "request timed out after 30s".to_string(),
            usage: None,
            message_id: None,
            parent_message_id: None,
        })
    );
}

#[test]
fn adapt_gateway_stream_event_maps_task_and_metadata_refresh_events() {
    let question_event = adapt_gateway_stream_event(GatewayChatStreamEvent::Other(serde_json::json!({
        "type": "chat.question_raised",
        "session_id": "session_123"
    })));

    assert_eq!(
        question_event,
        UiRuntimeEvent::TaskStateChanged {
            session_id: Some("session_123".to_string()),
        }
    );

    let title_event = adapt_gateway_stream_event(GatewayChatStreamEvent::Other(serde_json::json!({
        "type": "chat.title_updated",
        "session_id": "session_123",
        "title": "Renamed"
    })));

    assert_eq!(
        title_event,
        UiRuntimeEvent::SessionMetadataChanged {
            session_id: Some("session_123".to_string()),
            title: Some("Renamed".to_string()),
        }
    );
}

#[test]
fn prepare_stream_request_uses_runtime_session_id() {
    let runtime = runtime_with_session_id("session_123");
    let body = GatewayChatStreamRequest::default();

    let prepared = runtime.prepare_stream_request(&body);

    assert_eq!(prepared.session_id.as_ref().map(AsRef::as_ref), Some("session_123"));
}

#[test]
fn bind_session_seed_updates_runtime_context() {
    let mut runtime = runtime_with_session_id("session_123");

    runtime.bind_session_seed(
        Some("session_456".to_string()),
        Some("workspace-write".to_string()),
        Some("Renamed Session".to_string()),
    );

    assert_eq!(runtime.session_id(), Some("session_456"));
    assert_eq!(runtime.scope(), Some("workspace-write"));
    assert_eq!(runtime.title(), Some("Renamed Session"));
}

#[test]
fn filter_questions_for_session_keeps_only_matching_requests() {
    let question = Info {
        question: "Which path should be used?".to_string(),
        header: "Path".to_string(),
        options: vec![OptionInfo {
            label: "Current".to_string(),
            description: "Use the current workspace".to_string(),
            preview: None,
        }],
        multiple: Some(false),
        custom: Some(false),
    };

    let requests = vec![
        Request {
            id: "q1".to_string(),
            session_id: "session_a".to_string(),
            questions: vec![question.clone()],
            tool: None,
        },
        Request {
            id: "q2".to_string(),
            session_id: "session_b".to_string(),
            questions: vec![question],
            tool: None,
        },
    ];

    let filtered = filter_questions_for_session(requests, Some("session_b"));

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "q2");
}

#[test]
fn todo_put_body_converts_shared_todos_to_gateway_items() {
    let todos = vec![Todo {
        id: "todo_1".to_string(),
        content: "Hook runtime question polling into state".to_string(),
        status: "in_progress".to_string(),
        priority: "high".to_string(),
    }];

    let body = todo_put_body(&todos).expect("todo conversion should succeed");

    assert_eq!(
        body,
        GatewaySessionTodoPutBody {
            todos: vec![GatewaySessionTodoItem {
                id: "todo_1".to_string(),
                content: "Hook runtime question polling into state".to_string(),
                status: TodoStatus::InProgress,
                priority: TodoPriority::High,
            }],
        }
    );
}