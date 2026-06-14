use super::*;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use axum::body::to_bytes;
use axum::http::{HeaderValue, header};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

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

fn state(config: config::Config, require_pairing: bool, tokens: &[String]) -> AppState {
    AppState {
        config: Arc::new(Mutex::new(config)),
        provider: Arc::new(StaticProvider),
        model: "test-model".to_string(),
        temperature: 0.0,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(require_pairing, tokens)),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100, 100)),
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

fn bearer_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = format!("Bearer {token}");
    headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&value).unwrap());
    headers
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
    serde_json::from_slice(&bytes).expect("response should be json")
}

#[tokio::test]
async fn handle_api_config_get_requires_auth_when_pairing_is_enabled() {
    let token = "paired-token".to_string();
    let response = handle_api_config_get(
        State(state(config::Config::default(), true, std::slice::from_ref(&token))),
        HeaderMap::new(),
    )
    .await
    .into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response_json(response).await["error"].as_str().unwrap().contains("Unauthorized"));
}

#[tokio::test]
async fn handle_api_config_get_returns_pretty_toml_with_masked_secrets() {
    let mut cfg = config::Config::default();
    cfg.api_key = Some("sk-secret".to_string());
    cfg.gateway.paired_tokens = vec!["paired-secret".to_string()];

    let response = handle_api_config_get(State(state(cfg, false, &[])), HeaderMap::new())
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["format"], "toml");
    let content = body["content"].as_str().unwrap();
    assert!(content.contains("api_key = \"***MASKED***\""));
    assert!(!content.contains("sk-secret"));
    assert!(!content.contains("paired-secret"));
}

#[tokio::test]
async fn handle_api_config_put_rejects_unauthorized_before_parsing_body() {
    let token = "paired-token".to_string();
    let response = handle_api_config_put(
        State(state(config::Config::default(), true, std::slice::from_ref(&token))),
        HeaderMap::new(),
        "not = valid = toml".to_string(),
    )
    .await
    .into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn handle_api_config_put_rejects_invalid_toml() {
    let response = handle_api_config_put(
        State(state(config::Config::default(), false, &[])),
        HeaderMap::new(),
        "not = valid = toml".to_string(),
    )
    .await
    .into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response_json(response).await["error"].as_str().unwrap().contains("Invalid TOML"));
}

#[tokio::test]
async fn handle_api_config_put_hydrates_validates_saves_and_updates_runtime_config() {
    let temp = TempDir::new().expect("temp dir");
    let mut current = config::Config::default();
    current.config_path = temp.path().join("vibewindow.json");
    current.api_key = Some("sk-current".to_string());
    current.gateway.paired_tokens = vec!["paired-current".to_string()];

    let mut incoming = current.clone();
    incoming.default_provider = Some("openai".to_string());
    incoming.api_key = Some(crate::app::agent::gateway::api::secrets::MASKED_SECRET.to_string());
    incoming.gateway.paired_tokens =
        vec![crate::app::agent::gateway::api::secrets::MASKED_SECRET.to_string()];
    let body = toml::to_string_pretty(&incoming).expect("config should serialize as toml");
    let state = state(current, false, &[]);

    let response =
        handle_api_config_put(State(state.clone()), HeaderMap::new(), body).await.into_response();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_json(response).await["status"], "ok");
    assert_eq!(state.config.lock().default_provider.as_deref(), Some("openai"));
    assert_eq!(state.config.lock().api_key.as_deref(), Some("sk-current"));
    assert_eq!(state.config.lock().gateway.paired_tokens, vec!["paired-current"]);
    assert!(state.config.lock().config_path.exists());
}
