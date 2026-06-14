use super::*;
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::{Memory, MemoryCategory, MemoryEntry, NoneMemory};
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequest,
    http::{HeaderValue, Request, header},
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Default)]
struct StaticProvider {
    calls: AtomicUsize,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        message: &str,
        model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("{model}:{message}"))
    }
}

struct FailingProvider;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Provider for FailingProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        anyhow::bail!("provider token sk-test-secret failed")
    }
}

#[derive(Default)]
struct TrackingMemory {
    entries: Mutex<Vec<(String, String, MemoryCategory)>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Memory for TrackingMemory {
    fn name(&self) -> &str {
        "tracking"
    }

    async fn store(
        &self,
        key: &str,
        content: &str,
        category: MemoryCategory,
        _session_id: Option<&str>,
    ) -> anyhow::Result<()> {
        self.entries.lock().push((key.to_string(), content.to_string(), category));
        Ok(())
    }

    async fn recall(
        &self,
        _query: &str,
        _limit: usize,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(None)
    }

    async fn list(
        &self,
        _category: Option<&MemoryCategory>,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.entries.lock().len())
    }

    async fn health_check(&self) -> bool {
        true
    }
}

fn peer(ip: &str) -> SocketAddr {
    format!("{ip}:30300").parse().expect("test socket address should parse")
}

fn state_with(
    provider: Arc<dyn Provider>,
    mem: Arc<dyn Memory>,
    pairing: PairingGuard,
    webhook_secret_hash: Option<Arc<str>>,
    webhook_rate_limit: u32,
    auto_save: bool,
) -> AppState {
    AppState {
        config: Arc::new(parking_lot::Mutex::new(Config::default())),
        provider,
        model: "test-model".to_string(),
        temperature: 0.0,
        mem,
        auto_save,
        webhook_secret_hash,
        pairing: Arc::new(pairing),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, webhook_rate_limit, 100)),
        idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300), 100)),
        whatsapp: None,
        whatsapp_app_secret: None,
        linq: None,
        linq_signing_secret: None,
        nextcloud_talk: None,
        nextcloud_talk_webhook_secret: None,
        wati: None,
        qq: None,
        qq_webhook_enabled: false,
        observer: Arc::new(crate::app::agent::observability::NoopObserver),
        tools_registry: Arc::new(Vec::new()),
        tools_registry_exec: Arc::new(Vec::new()),
        multimodal: crate::app::agent::config::MultimodalConfig::default(),
        max_tool_iterations: 10,
        event_tx: tokio::sync::broadcast::channel(16).0,
        session_query_engines: Default::default(),
    }
}

fn basic_state() -> AppState {
    state_with(
        Arc::new(StaticProvider::default()),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(false, &[]),
        None,
        100,
        false,
    )
}

fn payload(response: (StatusCode, Json<Value>)) -> (StatusCode, Value) {
    (response.0, response.1.0)
}

#[test]
fn extract_idempotency_key_handles_missing_blank_trimmed_and_invalid_values() {
    let mut headers = HeaderMap::new();
    assert_eq!(extract_idempotency_key(&headers), None);

    headers.insert("X-Idempotency-Key", HeaderValue::from_static(""));
    assert_eq!(extract_idempotency_key(&headers), None);

    headers.insert("X-Idempotency-Key", HeaderValue::from_static("  request-123  "));
    assert_eq!(extract_idempotency_key(&headers), Some("request-123"));

    headers.insert(
        "X-Idempotency-Key",
        HeaderValue::from_bytes(b"\xff").expect("invalid utf-8 header value should build"),
    );
    assert_eq!(extract_idempotency_key(&headers), None);
}

#[test]
fn extract_webhook_secret_header_hash_trims_and_hashes_nonempty_secret() {
    let mut headers = HeaderMap::new();
    assert_eq!(extract_webhook_secret_header_hash(&headers), None);

    headers.insert("X-Webhook-Secret", HeaderValue::from_static("   "));
    assert_eq!(extract_webhook_secret_header_hash(&headers), None);

    headers.insert("X-Webhook-Secret", HeaderValue::from_static("  shared-secret  "));
    assert_eq!(
        extract_webhook_secret_header_hash(&headers),
        Some(hash_webhook_secret("shared-secret"))
    );
}

#[test]
fn parse_webhook_body_accepts_valid_json_body() {
    let parsed = parse_webhook_body(Ok(Json(WebhookBody { message: "hello".to_string() })))
        .expect("valid body should parse");

    assert_eq!(parsed.message, "hello");
}

#[tokio::test]
async fn parse_webhook_body_rejects_invalid_json_body() {
    let request = Request::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .expect("request should build");
    let rejected = Json::<WebhookBody>::from_request(request, &()).await;

    let response = match parse_webhook_body(rejected) {
        Ok(_) => panic!("body should reject"),
        Err(response) => response,
    };
    let (status, body) = payload(response);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON body. Expected: {\"message\": \"...\"}");
}

#[test]
fn enforce_idempotency_allows_missing_and_first_key_then_rejects_duplicate() {
    let state = basic_state();
    let mut headers = HeaderMap::new();

    assert!(enforce_idempotency(&state, &headers).is_none());

    headers.insert("X-Idempotency-Key", HeaderValue::from_static("same-request"));
    assert!(enforce_idempotency(&state, &headers).is_none());

    let (status, body) = payload(
        enforce_idempotency(&state, &headers).expect("duplicate key should return response"),
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "duplicate");
    assert_eq!(body["idempotent"], true);
}

#[test]
fn enforce_rate_limit_returns_retry_after_after_limit_is_exceeded() {
    let state = state_with(
        Arc::new(StaticProvider::default()),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(false, &[]),
        None,
        1,
        false,
    );
    let headers = HeaderMap::new();

    assert!(enforce_rate_limit(&state, peer("127.0.0.1"), &headers).is_none());
    let (status, body) = payload(
        enforce_rate_limit(&state, peer("127.0.0.1"), &headers)
            .expect("second request should be rate limited"),
    );

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["retry_after"], RATE_LIMIT_WINDOW_SECS);
}

#[test]
fn authorize_webhook_request_allows_loopback_without_auth_and_rejects_remote() {
    let state = basic_state();

    assert!(authorize_webhook_request(&state, peer("127.0.0.1"), &HeaderMap::new()).is_none());

    let (status, body) = payload(
        authorize_webhook_request(&state, peer("203.0.113.10"), &HeaderMap::new())
            .expect("remote request without auth should reject"),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        body["error"].as_str().expect("error should be string").contains("configure gateway auth")
    );
}

#[test]
fn authorize_webhook_request_requires_valid_bearer_when_pairing_is_enabled() {
    let paired_token = "paired-token".to_string();
    let state = state_with(
        Arc::new(StaticProvider::default()),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(true, std::slice::from_ref(&paired_token)),
        None,
        100,
        false,
    );
    let mut headers = HeaderMap::new();

    let (status, _) = payload(
        authorize_webhook_request(&state, peer("127.0.0.1"), &headers)
            .expect("missing bearer should reject"),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer paired-token"));
    assert!(authorize_webhook_request(&state, peer("203.0.113.10"), &headers).is_none());
}

#[test]
fn authorize_webhook_request_requires_matching_webhook_secret_when_configured() {
    let state = state_with(
        Arc::new(StaticProvider::default()),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(false, &[]),
        Some(Arc::from(hash_webhook_secret("shared-secret"))),
        100,
        false,
    );
    let mut headers = HeaderMap::new();

    let (status, _) = payload(
        authorize_webhook_request(&state, peer("127.0.0.1"), &headers)
            .expect("missing secret should reject"),
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    headers.insert("X-Webhook-Secret", HeaderValue::from_static("shared-secret"));
    assert!(authorize_webhook_request(&state, peer("203.0.113.10"), &headers).is_none());
}

#[tokio::test]
async fn maybe_persist_inbound_message_skips_when_auto_save_is_disabled() {
    let tracking = Arc::new(TrackingMemory::default());
    let state = state_with(
        Arc::new(StaticProvider::default()),
        tracking.clone(),
        PairingGuard::new(false, &[]),
        None,
        100,
        false,
    );

    maybe_persist_inbound_message(&state, "hello").await;

    assert!(tracking.entries.lock().is_empty());
}

#[tokio::test]
async fn maybe_persist_inbound_message_stores_conversation_when_auto_save_is_enabled() {
    let tracking = Arc::new(TrackingMemory::default());
    let state = state_with(
        Arc::new(StaticProvider::default()),
        tracking.clone(),
        PairingGuard::new(false, &[]),
        None,
        100,
        true,
    );

    maybe_persist_inbound_message(&state, "hello").await;

    let entries = tracking.entries.lock();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].0.starts_with("webhook_msg_"));
    assert_eq!(entries[0].1, "hello");
    assert_eq!(entries[0].2, MemoryCategory::Conversation);
}

#[tokio::test]
async fn handle_webhook_inner_returns_success_payload_for_authorized_loopback_request() {
    let provider = Arc::new(StaticProvider::default());
    let state = state_with(
        provider.clone(),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(false, &[]),
        None,
        100,
        false,
    );

    let (status, body) = payload(
        handle_webhook_inner(
            state,
            peer("127.0.0.1"),
            HeaderMap::new(),
            Ok(Json(WebhookBody { message: "hello".to_string() })),
        )
        .await,
    );

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["response"], "test-model:hello");
    assert_eq!(body["model"], "test-model");
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn handle_webhook_inner_returns_provider_error_without_leaking_details() {
    let state = state_with(
        Arc::new(FailingProvider),
        Arc::new(NoneMemory::new()),
        PairingGuard::new(false, &[]),
        None,
        100,
        false,
    );

    let (status, body) = payload(
        handle_webhook_inner(
            state,
            peer("127.0.0.1"),
            HeaderMap::new(),
            Ok(Json(WebhookBody { message: "hello".to_string() })),
        )
        .await,
    );

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "LLM request failed");
}
