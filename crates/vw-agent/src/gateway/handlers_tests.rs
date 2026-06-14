use super::*;
use crate::app::agent::channels::{
    LinqChannel, NextcloudTalkChannel, QQChannel, WatiChannel, WhatsAppChannel,
};
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use axum::body::{Bytes, to_bytes};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use hmac::{Hmac, Mac};
use parking_lot::Mutex;
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;

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
        Ok(format!("echo: {message}"))
    }
}

fn state() -> AppState {
    AppState {
        config: Arc::new(Mutex::new(Config::default())),
        provider: Arc::new(StaticProvider),
        model: "test-model".into(),
        temperature: 0.0,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(false, &[])),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100, 100)),
        idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300), 1_000)),
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
        multimodal: Default::default(),
        max_tool_iterations: 5,
        event_tx: tokio::sync::broadcast::channel(1).0,
        session_query_engines: Default::default(),
    }
}

async fn response_text(response: axum::response::Response) -> (StatusCode, String) {
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

async fn response_json(response: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

fn valid_whatsapp_signature(secret: &str, body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[test]
fn whatsapp_signature_accepts_valid_hmac() {
    let signature = valid_whatsapp_signature("secret", b"body");
    assert!(verify_whatsapp_signature("secret", b"body", &signature));
}

#[test]
fn whatsapp_signature_rejects_wrong_signature() {
    assert!(!verify_whatsapp_signature("secret", b"body", "sha256=not-valid"));
}

#[test]
fn whatsapp_signature_rejects_missing_prefix() {
    assert!(!verify_whatsapp_signature("secret", b"body", "abc123"));
}

#[test]
fn whatsapp_verify_query_deserializes_fields() {
    let query: WhatsAppVerifyQuery = serde_json::from_value(serde_json::json!({
        "hub.mode": "subscribe",
        "hub.verify_token": "token",
        "hub.challenge": "challenge"
    }))
    .expect("valid query");

    assert_eq!(query.mode.as_deref(), Some("subscribe"));
    assert_eq!(query.challenge.as_deref(), Some("challenge"));
}

#[tokio::test]
async fn whatsapp_verify_returns_not_found_when_unconfigured() {
    let response = handle_whatsapp_verify(
        State(state()),
        Query(WhatsAppVerifyQuery {
            mode: Some("subscribe".into()),
            verify_token: Some("token".into()),
            challenge: Some("challenge".into()),
        }),
    )
    .await;

    let (status, body) = response_text(response).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("not configured"));
}

#[tokio::test]
async fn whatsapp_verify_echoes_challenge_for_matching_token() {
    let mut state = state();
    state.whatsapp = Some(Arc::new(WhatsAppChannel::new(
        "access".into(),
        "phone-id".into(),
        "verify-token".into(),
        vec!["*".into()],
    )));

    let response = handle_whatsapp_verify(
        State(state),
        Query(WhatsAppVerifyQuery {
            mode: Some("subscribe".into()),
            verify_token: Some("verify-token".into()),
            challenge: Some("challenge-123".into()),
        }),
    )
    .await;

    assert_eq!(response_text(response).await, (StatusCode::OK, "challenge-123".into()));
}

#[tokio::test]
async fn whatsapp_verify_rejects_bad_or_incomplete_verification() {
    let mut state = state();
    state.whatsapp = Some(Arc::new(WhatsAppChannel::new(
        "access".into(),
        "phone-id".into(),
        "verify-token".into(),
        vec!["*".into()],
    )));

    let forbidden = handle_whatsapp_verify(
        State(state.clone()),
        Query(WhatsAppVerifyQuery {
            mode: Some("subscribe".into()),
            verify_token: Some("wrong".into()),
            challenge: Some("challenge".into()),
        }),
    )
    .await;
    assert_eq!(response_text(forbidden).await.0, StatusCode::FORBIDDEN);

    let missing_challenge = handle_whatsapp_verify(
        State(state),
        Query(WhatsAppVerifyQuery {
            mode: Some("subscribe".into()),
            verify_token: Some("verify-token".into()),
            challenge: None,
        }),
    )
    .await;
    assert_eq!(response_text(missing_challenge).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn whatsapp_message_validates_configuration_signature_json_and_empty_payloads() {
    let (status, body) = response_json(
        handle_whatsapp_message(State(state()), HeaderMap::new(), Bytes::new()).await,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "WhatsApp not configured");

    let mut configured = state();
    configured.whatsapp = Some(Arc::new(WhatsAppChannel::new(
        "access".into(),
        "phone-id".into(),
        "verify-token".into(),
        vec!["*".into()],
    )));
    configured.whatsapp_app_secret = Some(Arc::from("secret"));

    let (status, body) = response_json(
        handle_whatsapp_message(State(configured.clone()), HeaderMap::new(), Bytes::from("{}"))
            .await,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "Invalid signature");

    let payload = br#"{"entry":[]}"#;
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Hub-Signature-256",
        HeaderValue::from_str(&valid_whatsapp_signature("secret", payload)).unwrap(),
    );
    let (status, body) = response_json(
        handle_whatsapp_message(State(configured.clone()), headers, Bytes::from_static(payload))
            .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");

    configured.whatsapp_app_secret = None;
    let (status, body) = response_json(
        handle_whatsapp_message(State(configured), HeaderMap::new(), Bytes::from("{")).await,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON payload");
}

#[tokio::test]
async fn linq_webhook_handles_configuration_signature_json_and_empty_messages() {
    let (status, _) =
        response_json(handle_linq_webhook(State(state()), HeaderMap::new(), Bytes::new()).await)
            .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let mut configured = state();
    configured.linq =
        Some(Arc::new(LinqChannel::new("token".into(), "+15550000000".into(), vec!["*".into()])));
    configured.linq_signing_secret = Some(Arc::from("secret"));

    let (status, body) = response_json(
        handle_linq_webhook(State(configured.clone()), HeaderMap::new(), Bytes::from("{}")).await,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "Invalid signature");

    configured.linq_signing_secret = None;
    let (status, body) = response_json(
        handle_linq_webhook(State(configured.clone()), HeaderMap::new(), Bytes::from("{")).await,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON payload");

    let (status, body) = response_json(
        handle_linq_webhook(
            State(configured),
            HeaderMap::new(),
            Bytes::from(r#"{"event_type":"delivery.updated"}"#),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn wati_verify_and_webhook_cover_error_and_empty_paths() {
    let verify_missing = handle_wati_verify(
        State(state()),
        Query(WatiVerifyQuery { challenge: Some("challenge".into()) }),
    )
    .await;
    assert_eq!(response_text(verify_missing).await.0, StatusCode::NOT_FOUND);

    let mut configured = state();
    configured.wati = Some(Arc::new(WatiChannel::new(
        "token".into(),
        "https://wati.example".into(),
        None,
        vec!["*".into()],
    )));

    let verified = handle_wati_verify(
        State(configured.clone()),
        Query(WatiVerifyQuery { challenge: Some("wati-challenge".into()) }),
    )
    .await;
    assert_eq!(response_text(verified).await, (StatusCode::OK, "wati-challenge".into()));

    let missing_challenge =
        handle_wati_verify(State(configured.clone()), Query(WatiVerifyQuery { challenge: None }))
            .await;
    assert_eq!(response_text(missing_challenge).await.0, StatusCode::BAD_REQUEST);

    let (status, body) =
        response_json(handle_wati_webhook(State(state()), Bytes::new()).await).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "WATI not configured");

    let (status, body) =
        response_json(handle_wati_webhook(State(configured.clone()), Bytes::from("{")).await).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON payload");

    let (status, body) =
        response_json(handle_wati_webhook(State(configured), Bytes::from("{}")).await).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn nextcloud_talk_webhook_handles_security_and_empty_payloads() {
    let (status, _) = response_json(
        handle_nextcloud_talk_webhook(State(state()), HeaderMap::new(), Bytes::new()).await,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let mut configured = state();
    configured.nextcloud_talk = Some(Arc::new(NextcloudTalkChannel::new(
        "https://cloud.example".into(),
        "app-token".into(),
        vec!["*".into()],
    )));
    configured.nextcloud_talk_webhook_secret = Some(Arc::from("secret"));

    let (status, body) = response_json(
        handle_nextcloud_talk_webhook(
            State(configured.clone()),
            HeaderMap::new(),
            Bytes::from("{}"),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "Invalid signature");

    configured.nextcloud_talk_webhook_secret = None;
    let (status, body) = response_json(
        handle_nextcloud_talk_webhook(
            State(configured.clone()),
            HeaderMap::new(),
            Bytes::from("{"),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON payload");

    let (status, body) = response_json(
        handle_nextcloud_talk_webhook(
            State(configured),
            HeaderMap::new(),
            Bytes::from(r#"{"type":"delete"}"#),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn qq_webhook_handles_configuration_validation_and_empty_events() {
    let (status, _) =
        response_json(handle_qq_webhook(State(state()), HeaderMap::new(), Bytes::new()).await)
            .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let mut configured = state();
    configured.qq =
        Some(Arc::new(QQChannel::new("app-id".into(), "secret".into(), vec!["*".into()])));

    let (status, body) = response_json(
        handle_qq_webhook(State(configured.clone()), HeaderMap::new(), Bytes::from("{}")).await,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "QQ webhook mode not enabled");

    configured.qq_webhook_enabled = true;
    let mut headers = HeaderMap::new();
    headers.insert("X-Bot-Appid", HeaderValue::from_static("other-app"));
    let (status, body) = response_json(
        handle_qq_webhook(State(configured.clone()), headers, Bytes::from("{}")).await,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "Invalid X-Bot-Appid");

    let (status, body) = response_json(
        handle_qq_webhook(State(configured.clone()), HeaderMap::new(), Bytes::from("{")).await,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid JSON payload");

    let validation = r#"{"op":13,"d":{"plain_token":"plain","event_ts":"12345"}}"#;
    let (status, body) = response_json(
        handle_qq_webhook(State(configured.clone()), HeaderMap::new(), Bytes::from(validation))
            .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["plain_token"], "plain");
    assert!(body["signature"].as_str().is_some_and(|value| !value.is_empty()));

    let (status, body) = response_json(
        handle_qq_webhook(State(configured), HeaderMap::new(), Bytes::from(r#"{"op":1}"#)).await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}
