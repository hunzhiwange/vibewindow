use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::filesystem::{FileSystemHandlers, FileSystemHandlersOptions};
use crate::terminal::{TerminalManager, TerminalManagerOptions};
use crate::types::{PermissionMode, PermissionStats};

use super::{AcpEventClient, InternalEvent};

fn event_client(
    event_tx: mpsc::UnboundedSender<InternalEvent>,
    expected_session_id: Option<String>,
) -> AcpEventClient {
    AcpEventClient {
        expected_session_id: Arc::new(Mutex::new(expected_session_id)),
        event_tx,
        filesystem: FileSystemHandlers::new(FileSystemHandlersOptions {
            cwd: std::env::temp_dir(),
            ..FileSystemHandlersOptions::default()
        }),
        terminal_manager: TerminalManager::new(TerminalManagerOptions {
            cwd: PathBuf::from("."),
            ..TerminalManagerOptions::default()
        }),
        permission_mode: PermissionMode::ApproveAll,
        non_interactive_permissions: None,
        on_session_update: None,
        permission_stats: Arc::new(Mutex::new(PermissionStats::default())),
        cancelling_session_ids: Arc::new(Mutex::new(HashSet::new())),
    }
}

#[tokio::test]
async fn session_notification_forwards_text_delta() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let client = event_client(event_tx, Some("session-1".to_string()));

    client
        .session_notification(acp::SessionNotification::new(
            "session-1",
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new("hello"),
            ))),
        ))
        .await
        .expect("session notification");

    match event_rx.recv().await.expect("event") {
        InternalEvent::Delta(delta) => assert_eq!(delta, "hello"),
        other => panic!("unexpected event: {other:?}"),
    }
}

#[tokio::test]
async fn session_notification_accepts_session_change_and_forwards_delta() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let client = event_client(event_tx, Some("expected".to_string()));

    client
        .session_notification(acp::SessionNotification::new(
            "actual",
            acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new("hidden"),
            ))),
        ))
        .await
        .expect("session notification");

    match event_rx.recv().await.expect("event") {
        InternalEvent::SessionChanged { expected, actual } => {
            assert_eq!(expected, "expected");
            assert_eq!(actual, "actual");
        }
        other => panic!("unexpected event: {other:?}"),
    }
    match event_rx.recv().await.expect("event") {
        InternalEvent::Delta(delta) => assert_eq!(delta, "hidden"),
        other => panic!("unexpected event: {other:?}"),
    }
    assert!(event_rx.try_recv().is_err());
    assert_eq!(client.expected_session_id.lock().as_deref(), Some("actual"));
}

#[tokio::test]
async fn ext_method_is_explicitly_unsupported() {
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let client = event_client(event_tx, None);
    let params = serde_json::value::RawValue::from_string("{}".to_string()).expect("raw value");

    let err = client
        .ext_method(acp::ExtRequest::new("custom/test", params.into()))
        .await
        .expect_err("ext method unsupported");

    assert!(err.to_string().to_ascii_lowercase().contains("method not found"));
}
