use super::*;
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use axum::body::to_bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::broadcast;

#[test]
fn exported_handlers_are_available() {
    let _ = handle_api_integrations;
    let _ = handle_api_integrations_settings;
    let _ = handle_api_integration_credentials_put;
}

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(message.to_string())
    }
}

fn config_in(temp: &TempDir) -> Config {
    let mut config = Config::default();
    config.config_path = temp.path().join("vibewindow.json");
    config.workspace_dir = temp.path().join("workspace");
    config
}

fn state_with(config: Config, require_pairing: bool, token: Option<&str>) -> AppState {
    let (event_tx, _) = broadcast::channel(16);
    let paired_tokens = token.map(|token| vec![token.to_string()]).unwrap_or_default();
    AppState {
        config: Arc::new(parking_lot::Mutex::new(config)),
        provider: Arc::new(StaticProvider),
        model: "test-model".to_string(),
        temperature: 0.0,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(require_pairing, &paired_tokens)),
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
        event_tx,
        session_query_engines: Default::default(),
    }
}

fn auth_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).expect("valid auth header"),
    );
    headers
}

async fn response_json(response: Response) -> (StatusCode, serde_json::Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
    let value = serde_json::from_slice(&bytes).expect("response should be json");
    (status, value)
}

#[tokio::test]
async fn integrations_requires_auth_when_pairing_enabled() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), true, Some("paired-token"));

    let (status, value) = response_json(
        handle_api_integrations(State(state), HeaderMap::new()).await.into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(value["error"].as_str().unwrap_or_default().contains("Unauthorized"));
}

#[tokio::test]
async fn integrations_lists_registry_entries_with_status() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = config_in(&temp);
    config.default_provider = Some("openai".to_string());
    let state = state_with(config, false, None);

    let (status, value) = response_json(
        handle_api_integrations(State(state), HeaderMap::new()).await.into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let integrations = value["integrations"].as_array().expect("integrations array");
    assert!(integrations.iter().any(|entry| {
        entry["name"] == "OpenAI" && entry["status"] == serde_json::json!("Active")
    }));
}

#[tokio::test]
async fn integration_settings_include_revision_and_active_provider() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = config_in(&temp);
    config.default_provider = Some("gemini".to_string());
    config.api_key = Some("test-key".to_string());
    let state = state_with(config, false, None);

    let (status, value) = response_json(
        handle_api_integrations_settings(State(state), HeaderMap::new()).await.into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(value["revision"].as_str().is_some_and(|revision| !revision.is_empty()));
    assert_eq!(value["active_default_provider_integration_id"], "google");
    assert!(value["integrations"].as_array().is_some_and(|items| !items.is_empty()));
}

#[tokio::test]
async fn integration_credentials_put_rejects_stale_revision() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), false, None);
    let body = IntegrationCredentialsUpdateBody {
        revision: Some("stale".to_string()),
        fields: BTreeMap::new(),
    };

    let (status, value) = response_json(
        handle_api_integration_credentials_put(
            State(state),
            HeaderMap::new(),
            Path("openai".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(value["revision"].as_str().is_some());
}

#[tokio::test]
async fn integration_credentials_put_maps_unknown_integration_to_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), false, None);
    let body = IntegrationCredentialsUpdateBody { revision: None, fields: BTreeMap::new() };

    let (status, value) = response_json(
        handle_api_integration_credentials_put(
            State(state),
            HeaderMap::new(),
            Path("missing".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(value["error"].as_str().unwrap_or_default().contains("Unknown integration id"));
}

#[tokio::test]
async fn integration_credentials_put_rejects_unsupported_field() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), false, None);
    let mut fields = BTreeMap::new();
    fields.insert("api_url".to_string(), "https://example.invalid".to_string());
    let body = IntegrationCredentialsUpdateBody { revision: None, fields };

    let (status, value) = response_json(
        handle_api_integration_credentials_put(
            State(state),
            HeaderMap::new(),
            Path("openai".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(value["error"].as_str().unwrap_or_default().contains("does not support api_url"));
}

#[tokio::test]
async fn integration_credentials_put_reports_unchanged_when_revision_is_same() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut config = config_in(&temp);
    config.default_provider = Some("openai".to_string());
    let revision = crate::app::agent::gateway::api::integrations::config_revision(&config);
    let state = state_with(config, false, None);
    let body =
        IntegrationCredentialsUpdateBody { revision: Some(revision), fields: BTreeMap::new() };

    let (status, value) = response_json(
        handle_api_integration_credentials_put(
            State(state),
            HeaderMap::new(),
            Path("openai".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["unchanged"], true);
}

#[tokio::test]
async fn integration_credentials_put_saves_and_updates_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), false, None);
    let mut fields = BTreeMap::new();
    fields.insert("api_key".to_string(), " sk-test ".to_string());
    fields.insert("default_model".to_string(), "gpt-5.2".to_string());
    let body = IntegrationCredentialsUpdateBody { revision: None, fields };

    let (status, value) = response_json(
        handle_api_integration_credentials_put(
            State(state.clone()),
            HeaderMap::new(),
            Path("openai".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["status"], "ok");
    assert_eq!(state.config.lock().default_provider.as_deref(), Some("openai"));
    assert_eq!(state.config.lock().api_key.as_deref(), Some("sk-test"));
    assert!(temp.path().join("vibewindow.json").exists());
}

#[tokio::test]
async fn integration_credentials_put_accepts_valid_bearer_token() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state = state_with(config_in(&temp), true, Some("paired-token"));
    let body = IntegrationCredentialsUpdateBody { revision: None, fields: BTreeMap::new() };

    let (status, _value) = response_json(
        handle_api_integration_credentials_put(
            State(state),
            auth_headers("paired-token"),
            Path("openai".to_string()),
            JsonResponse(body),
        )
        .await
        .into_response(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
}
