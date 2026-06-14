use serde_json::json;

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{SocketAddr, TcpListener};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;

use super::{
    GatewayChatStreamEvent, GatewayChatStreamRequest, GatewayChatUsage, GatewayTypedChatStreamEvent,
};

#[cfg(not(target_arch = "wasm32"))]
struct StreamServer {
    addr: SocketAddr,
    handle: Option<JoinHandle<String>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl StreamServer {
    fn respond(status: &str, body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let addr = listener.local_addr().expect("test server addr should be available");
        let status = status.to_string();
        let body = body.as_bytes().to_vec();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test connection should be accepted");
            let mut request = [0_u8; 4096];
            let bytes_read = stream.read(&mut request).expect("request should be readable");
            let request = String::from_utf8_lossy(&request[..bytes_read]).to_string();
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("headers should be written");
            stream.write_all(&body).expect("body should be written");
            request
        });
        Self { addr, handle: Some(handle) }
    }

    fn endpoint(&self) -> crate::endpoint::GatewayEndpoint {
        crate::endpoint::GatewayEndpoint::new("127.0.0.1", self.addr.port())
    }

    fn join(&mut self) -> String {
        self.handle.take().expect("server should not be joined").join().unwrap()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for StreamServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[test]
fn usage_serializes_defaults_reasoning_tokens() {
    let usage: GatewayChatUsage =
        serde_json::from_value(json!({"input_tokens": 1, "output_tokens": 2, "cached_tokens": 3}))
            .unwrap();

    assert_eq!(
        usage,
        GatewayChatUsage {
            input_tokens: 1,
            output_tokens: 2,
            cached_tokens: 3,
            reasoning_tokens: 0,
        }
    );
}

#[test]
fn parse_stream_event_maps_known_gateway_events() {
    assert_eq!(
        super::parse_stream_event(json!({"type": "chat.delta", "delta": "hello"})),
        GatewayChatStreamEvent::Delta("hello".to_string())
    );
    assert_eq!(
        super::parse_stream_event(json!({"type": "chat.delta"})),
        GatewayChatStreamEvent::Delta(String::new())
    );
    assert_eq!(
        super::parse_stream_event(json!({
            "type": "chat.done",
            "finish_reason": "stop",
            "usage": {"input_tokens": 1, "output_tokens": 2, "cached_tokens": 3},
            "message_id": "msg-1",
            "parent_message_id": "msg-0"
        })),
        GatewayChatStreamEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: Some(json!({"input_tokens": 1, "output_tokens": 2, "cached_tokens": 3})),
            message_id: Some("msg-1".to_string()),
            parent_message_id: Some("msg-0".to_string()),
        }
    );
    assert_eq!(
        super::parse_stream_event(json!({"type": "chat.error", "error": "failed"})),
        GatewayChatStreamEvent::Error("failed".to_string())
    );
    assert_eq!(
        super::parse_stream_event(json!({"type": "chat.error"})),
        GatewayChatStreamEvent::Error("gateway stream failed".to_string())
    );
}

#[test]
fn parse_stream_event_preserves_unknown_payload() {
    let payload = json!({"type": "chat.custom", "value": 1});

    assert_eq!(super::parse_stream_event(payload.clone()), GatewayChatStreamEvent::Other(payload));
}

#[test]
fn take_next_sse_event_handles_lf_and_crlf_separators() {
    let mut lf = "data: one\n\ndata: two".to_string();
    assert_eq!(super::take_next_sse_event(&mut lf).as_deref(), Some("data: one"));
    assert_eq!(lf, "data: two");

    let mut crlf = "data: one\r\n\r\ndata: two".to_string();
    assert_eq!(super::take_next_sse_event(&mut crlf).as_deref(), Some("data: one"));
    assert_eq!(crlf, "data: two");

    assert_eq!(super::take_next_sse_event(&mut "data: partial".to_string()), None);
}

#[test]
fn event_name_returns_stable_names() {
    assert_eq!(super::event_name(&GatewayChatStreamEvent::Delta(String::new())), "chat.delta");
    assert_eq!(
        super::event_name(&GatewayChatStreamEvent::Done {
            finish_reason: None,
            usage: None,
            message_id: None,
            parent_message_id: None,
        }),
        "chat.done"
    );
    assert_eq!(super::event_name(&GatewayChatStreamEvent::Error(String::new())), "chat.error");
    assert_eq!(super::event_name(&GatewayChatStreamEvent::Other(json!({}))), "other");
}

#[test]
fn normalize_chat_stream_event_maps_direct_events() {
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Delta("delta".to_string())),
        GatewayTypedChatStreamEvent::Delta("delta".to_string())
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Error("bad".to_string())),
        GatewayTypedChatStreamEvent::Error("bad".to_string())
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: Some(json!({
                "input_tokens": 1,
                "output_tokens": 2,
                "cached_tokens": 3,
                "reasoning_tokens": 4
            })),
            message_id: Some("msg".to_string()),
            parent_message_id: Some("parent".to_string()),
        }),
        GatewayTypedChatStreamEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: Some(GatewayChatUsage {
                input_tokens: 1,
                output_tokens: 2,
                cached_tokens: 3,
                reasoning_tokens: 4,
            }),
            message_id: Some("msg".to_string()),
            parent_message_id: Some("parent".to_string()),
        }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Done {
            finish_reason: None,
            usage: Some(json!({"bad": true})),
            message_id: None,
            parent_message_id: None,
        }),
        GatewayTypedChatStreamEvent::Done {
            finish_reason: None,
            usage: None,
            message_id: None,
            parent_message_id: None,
        }
    );
}

#[test]
fn normalize_chat_stream_event_maps_structured_other_events() {
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.step_start",
            "step_index": 2,
            "created_ms": 100,
            "model": "m"
        }))),
        GatewayTypedChatStreamEvent::StepStart(super::GatewayChatStepStartEvent {
            step_index: 2,
            created_ms: 100,
            model: Some("m".to_string()),
        })
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.step_finish",
            "step_index": 3,
            "finished_ms": 200,
            "usage": {"input_tokens": 4, "output_tokens": 5, "cached_tokens": 6},
            "finish_reason": "tool",
            "model": "m2"
        }))),
        GatewayTypedChatStreamEvent::StepFinish(super::GatewayChatStepFinishEvent {
            step_index: 3,
            finished_ms: 200,
            usage: GatewayChatUsage {
                input_tokens: 4,
                output_tokens: 5,
                cached_tokens: 6,
                reasoning_tokens: 0,
            },
            finish_reason: Some("tool".to_string()),
            model: Some("m2".to_string()),
        })
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.post_tool_round",
            "step_index": 4
        }))),
        GatewayTypedChatStreamEvent::PostToolRound(super::GatewayChatPostToolRoundEvent {
            step_index: 4,
        })
    );
}

#[test]
fn normalize_chat_stream_event_maps_session_side_effect_events() {
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.todo_updated",
            "session_id": "s1"
        }))),
        GatewayTypedChatStreamEvent::TodoUpdated { session_id: Some("s1".to_string()) }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.question_raised",
            "session_id": "s2"
        }))),
        GatewayTypedChatStreamEvent::QuestionRaised { session_id: Some("s2".to_string()) }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.question_resolved",
            "session_id": "s3"
        }))),
        GatewayTypedChatStreamEvent::QuestionResolved { session_id: Some("s3".to_string()) }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.usage_updated",
            "session_id": "s4",
            "usage": {"input_tokens": 7, "output_tokens": 8, "cached_tokens": 9}
        }))),
        GatewayTypedChatStreamEvent::UsageUpdated {
            session_id: Some("s4".to_string()),
            usage: GatewayChatUsage {
                input_tokens: 7,
                output_tokens: 8,
                cached_tokens: 9,
                reasoning_tokens: 0,
            },
        }
    );
}

#[test]
fn normalize_chat_stream_event_maps_title_session_and_unknown_events() {
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.title_updated",
            "session_id": "s5",
            "title": "New title"
        }))),
        GatewayTypedChatStreamEvent::TitleUpdated {
            session_id: Some("s5".to_string()),
            title: "New title".to_string(),
        }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.session_updated",
            "session_id": "s6",
            "session": {"title": "Session title"}
        }))),
        GatewayTypedChatStreamEvent::SessionUpdated {
            session_id: Some("s6".to_string()),
            title: Some("Session title".to_string()),
        }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.future"
        }))),
        GatewayTypedChatStreamEvent::Unknown { event_type: Some("chat.future".to_string()) }
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({}))),
        GatewayTypedChatStreamEvent::Unknown { event_type: None }
    );
}

#[test]
fn structured_event_parsers_default_invalid_fields() {
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.step_start",
            "step_index": u64::from(u32::MAX) + 1,
            "created_ms": "bad",
            "model": 1
        }))),
        GatewayTypedChatStreamEvent::StepStart(super::GatewayChatStepStartEvent {
            step_index: 0,
            created_ms: 0,
            model: None,
        })
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.step_finish",
            "step_index": u64::from(u32::MAX) + 1,
            "finished_ms": "bad",
            "usage": {"bad": true},
            "finish_reason": 1,
            "model": 2
        }))),
        GatewayTypedChatStreamEvent::StepFinish(super::GatewayChatStepFinishEvent {
            step_index: 0,
            finished_ms: 0,
            usage: GatewayChatUsage::default(),
            finish_reason: None,
            model: None,
        })
    );
    assert_eq!(
        super::normalize_chat_stream_event(GatewayChatStreamEvent::Other(json!({
            "type": "chat.post_tool_round",
            "step_index": u64::from(u32::MAX) + 1
        }))),
        GatewayTypedChatStreamEvent::PostToolRound(super::GatewayChatPostToolRoundEvent {
            step_index: 0,
        })
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn stream_chat_reads_sse_events_and_stops_when_callback_returns_false() {
    let mut server = StreamServer::respond(
        "200 OK",
        "data: {\"type\":\"chat.delta\",\"delta\":\"a\"}\n\n\
         data: {\"type\":\"chat.done\",\"finish_reason\":\"stop\"}\n\n",
    );
    let endpoint = server.endpoint();
    let body = GatewayChatStreamRequest::default();
    let mut events = Vec::new();

    super::stream_chat(&endpoint, Some(" /tmp/project "), &body, |event| {
        events.push(event);
        false
    })
    .await
    .unwrap();
    let request = server.join();

    assert_eq!(events, vec![GatewayChatStreamEvent::Delta("a".to_string())]);
    assert!(request.starts_with("POST /v1/chat/stream?directory="));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn stream_chat_skips_empty_payload_frames_and_collects_multiline_data() {
    let mut server = StreamServer::respond(
        "200 OK",
        "event: ping\n\n\
         data: {\"type\":\"chat.delta\",\n\
         data: \"delta\":\"joined\"}\n\n",
    );
    let endpoint = server.endpoint();
    let body = GatewayChatStreamRequest::default();
    let mut events = Vec::new();

    super::stream_chat(&endpoint, Some(""), &body, |event| {
        events.push(event);
        true
    })
    .await
    .unwrap();
    let request = server.join();

    assert_eq!(events, vec![GatewayChatStreamEvent::Delta("joined".to_string())]);
    assert!(request.starts_with("POST /v1/chat/stream HTTP/1.1"));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn stream_chat_reports_http_and_json_errors() {
    let mut http_server = StreamServer::respond("500 Internal Server Error", " down ");
    let http_error = super::stream_chat(
        &http_server.endpoint(),
        None,
        &GatewayChatStreamRequest::default(),
        |_| true,
    )
    .await
    .unwrap_err();
    let _ = http_server.join();
    assert!(http_error.contains("gateway stream failed: 500 Internal Server Error down"));

    let mut json_server = StreamServer::respond("200 OK", "data: not-json\n\n");
    let json_error = super::stream_chat(
        &json_server.endpoint(),
        None,
        &GatewayChatStreamRequest::default(),
        |_| true,
    )
    .await
    .unwrap_err();
    let _ = json_server.join();
    assert!(json_error.contains("expected ident") || json_error.contains("expected value"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn stream_chat_blocking_reads_events_and_honors_callback_stop() {
    let mut server =
        StreamServer::respond("200 OK", "data: {\"type\":\"chat.delta\",\"delta\":\"b\"}\r\n\r\n");
    let endpoint = server.endpoint();
    let body = GatewayChatStreamRequest::default();
    let mut events = Vec::new();

    super::stream_chat_blocking(&endpoint, None, &body, |event| {
        events.push(event);
        false
    })
    .unwrap();
    let request = server.join();

    assert_eq!(events, vec![GatewayChatStreamEvent::Delta("b".to_string())]);
    assert!(request.starts_with("POST /v1/chat/stream HTTP/1.1"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn stream_chat_blocking_reports_http_and_json_errors() {
    let mut http_server = StreamServer::respond("503 Service Unavailable", "offline");
    let http_error = super::stream_chat_blocking(
        &http_server.endpoint(),
        None,
        &GatewayChatStreamRequest::default(),
        |_| true,
    )
    .unwrap_err();
    let _ = http_server.join();
    assert!(http_error.contains("gateway stream failed: 503 Service Unavailable offline"));

    let mut json_server = StreamServer::respond("200 OK", "data: not-json\n\n");
    let json_error = super::stream_chat_blocking(
        &json_server.endpoint(),
        None,
        &GatewayChatStreamRequest::default(),
        |_| true,
    )
    .unwrap_err();
    let _ = json_server.join();
    assert!(json_error.contains("expected ident") || json_error.contains("expected value"));
}
