use super::*;
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use axum::routing::post;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tower::util::ServiceExt;

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok("ok".to_string())
    }
}

fn peer() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030)
}

fn state(require_pairing: bool, paired_tokens: &[String], agent_rate_limit: u32) -> AppState {
    AppState {
        config: Arc::new(parking_lot::Mutex::new(Config::default())),
        provider: Arc::new(StaticProvider),
        model: "test-model".to_string(),
        temperature: 0.0,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(require_pairing, paired_tokens)),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, agent_rate_limit, 100)),
        idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300), 1000)),
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

async fn response_json(response: Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
    serde_json::from_slice(&body).expect("response should be json")
}

#[tokio::test]
async fn handle_agent_rejects_unauthorized_requests_before_body_validation() {
    let token = "paired-token".to_string();
    let response = handle_agent(
        State(state(true, std::slice::from_ref(&token), 100)),
        ConnectInfo(peer()),
        HeaderMap::new(),
        Ok(Json(AgentBody { message: "hello".to_string() })),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response_json(response).await["error"].as_str().unwrap().contains("Unauthorized"));
}

#[tokio::test]
async fn handle_agent_rejects_invalid_json_body() {
    let app = Router::new().route("/agent", post(handle_agent)).with_state(state(false, &[], 100));
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/agent")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{ invalid json"))
        .expect("request should build");
    request.extensions_mut().insert(ConnectInfo(peer()));

    let response = app.oneshot(request).await.expect("router should respond");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(
        response_json(response.into_response()).await["error"]
            .as_str()
            .unwrap()
            .contains("Invalid JSON")
    );
}

#[tokio::test]
async fn handle_agent_rejects_empty_message() {
    let response = handle_agent(
        State(state(false, &[], 100)),
        ConnectInfo(peer()),
        HeaderMap::new(),
        Ok(Json(AgentBody { message: "   \n\t".to_string() })),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_json(response).await["error"], "message must not be empty");
}

#[tokio::test]
async fn handle_agent_enforces_agent_rate_limit_before_auth() {
    let state = state(false, &[], 1);
    let body = || Ok(Json(AgentBody { message: String::new() }));

    let first =
        handle_agent(State(state.clone()), ConnectInfo(peer()), HeaderMap::new(), body()).await;
    let second = handle_agent(State(state), ConnectInfo(peer()), HeaderMap::new(), body()).await;

    assert_eq!(first.status(), StatusCode::BAD_REQUEST);
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response_json(second).await["retry_after"], RATE_LIMIT_WINDOW_SECS);
}

#[tokio::test]
async fn handle_agent_returns_bad_gateway_when_agent_engine_fails() {
    let state = state(false, &[], 100);
    state.config.lock().default_provider = Some("unsupported-provider".to_string());

    let response = handle_agent(
        State(state),
        ConnectInfo(peer()),
        HeaderMap::new(),
        Ok(Json(AgentBody { message: "hello".to_string() })),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(response_json(response).await["error"].as_str().unwrap().contains("Provider error"));
}
