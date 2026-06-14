use super::TelegramChannel;
use crate::app::agent::channels::traits::{Channel, SendMessage};
use crate::app::agent::config::StreamMode;
use axum::{
    Router,
    body::Bytes,
    extract::{OriginalUri, State},
    http::StatusCode,
    routing::post,
};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

#[test]
fn channel_name_and_draft_support_are_stable() {
    let channel = TelegramChannel::new("token".to_string(), vec![], false);

    assert_eq!(channel.name(), "telegram");
    assert!(channel.supports_draft_updates());
}

#[derive(Clone, Debug)]
struct RecordedRequest {
    path: String,
    body: serde_json::Value,
}

struct TestServerState {
    requests: Mutex<Vec<RecordedRequest>>,
    status: StatusCode,
}

struct TestServer {
    base_url: String,
    state: Arc<TestServerState>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestServer {
    async fn spawn(status: StatusCode) -> Self {
        let state = Arc::new(TestServerState { requests: Mutex::new(Vec::new()), status });
        let app = Router::new().route("/{*path}", post(record_request)).with_state(state.clone());
        let listener =
            tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("bind test server");
        let addr = listener.local_addr().expect("server addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("serve test server");
        });

        Self { base_url: format!("http://{addr}"), state, shutdown: Some(shutdown_tx) }
    }

    async fn requests(&self) -> Vec<RecordedRequest> {
        self.state.requests.lock().await.clone()
    }

    async fn wait_for_requests(&self, expected: usize) -> Vec<RecordedRequest> {
        for _ in 0..50 {
            let requests = self.requests().await;
            if requests.len() >= expected {
                return requests;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        self.requests().await
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

async fn record_request(
    State(state): State<Arc<TestServerState>>,
    uri: OriginalUri,
    body: Bytes,
) -> StatusCode {
    let body = serde_json::from_slice(&body).unwrap_or_else(|_| serde_json::Value::Null);
    state.requests.lock().await.push(RecordedRequest { path: uri.path().to_string(), body });
    state.status
}

fn test_channel(base_url: &str) -> TelegramChannel {
    TelegramChannel::new("123:ABC".to_string(), vec!["tester".to_string()], false)
        .with_api_base(base_url.to_string())
}

#[test]
fn supports_draft_updates_depends_on_stream_mode_or_pairing_allowlist() {
    let configured = TelegramChannel::new("token".to_string(), vec!["alice".to_string()], false);
    assert!(!configured.supports_draft_updates());

    let streaming = configured.with_streaming(StreamMode::Partial, 100);
    assert!(streaming.supports_draft_updates());

    let pairing_mode = TelegramChannel::new("token".to_string(), vec![], false);
    assert!(pairing_mode.supports_draft_updates());
}

#[tokio::test]
async fn channel_send_delegates_to_outbound_text_sender() {
    let server = TestServer::spawn(StatusCode::OK).await;
    let channel = test_channel(&server.base_url);

    channel
        .send(&SendMessage::new("hello **there**", "chat-1:5"))
        .await
        .expect("send should succeed");

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/bot123:ABC/sendMessage");
    assert_eq!(requests[0].body["chat_id"], "chat-1");
    assert_eq!(requests[0].body["message_thread_id"], "5");
    assert_eq!(requests[0].body["parse_mode"], "HTML");
    assert_eq!(requests[0].body["text"], "hello <b>there</b>");
}

#[tokio::test]
async fn approval_prompt_builds_inline_keyboard_and_prefers_recipient_thread() {
    let server = TestServer::spawn(StatusCode::OK).await;
    let channel = test_channel(&server.base_url);

    channel
        .send_approval_prompt(
            "chat-1:from-recipient",
            "req-123",
            "shell",
            &serde_json::json!({"cmd": "echo hello"}),
            Some("from-arg".to_string()),
        )
        .await
        .expect("approval prompt");

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/bot123:ABC/sendMessage");
    assert_eq!(requests[0].body["chat_id"], "chat-1");
    assert_eq!(requests[0].body["message_thread_id"], "from-recipient");
    assert_eq!(
        requests[0].body["reply_markup"]["inline_keyboard"][0][0]["callback_data"],
        "zcapr:yes:req-123"
    );
    assert_eq!(
        requests[0].body["reply_markup"]["inline_keyboard"][0][1]["callback_data"],
        "zcapr:no:req-123"
    );
}

#[tokio::test]
async fn approval_prompt_reports_sanitized_error() {
    let server = TestServer::spawn(StatusCode::BAD_REQUEST).await;
    let channel = test_channel(&server.base_url);

    let error = channel
        .send_approval_prompt("chat-1", "req", "tool", &serde_json::json!({}), None)
        .await
        .expect_err("bad status should fail")
        .to_string();

    assert!(error.contains("Telegram approval prompt failed"));
    assert!(error.contains("400 Bad Request"));
}

#[tokio::test]
async fn typing_indicator_starts_and_stops_background_task() {
    let server = TestServer::spawn(StatusCode::OK).await;
    let channel = test_channel(&server.base_url);

    channel.start_typing("chat-1").await.expect("start typing");
    let requests = server.wait_for_requests(1).await;
    assert_eq!(requests[0].path, "/bot123:ABC/sendChatAction");
    assert_eq!(requests[0].body["chat_id"], "chat-1");
    assert_eq!(requests[0].body["action"], "typing");
    assert!(channel.typing_handle.lock().is_some());

    channel.stop_typing("chat-1").await.expect("stop typing");
    assert!(channel.typing_handle.lock().is_none());
}
