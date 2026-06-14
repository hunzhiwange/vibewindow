use super::*;
use crate::app::agent::config::Config;
use crate::app::agent::gateway::{GatewayRateLimiter, IdempotencyStore};
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::providers::Provider;
use crate::app::agent::security::pairing::PairingGuard;
use crate::app::agent::tools::ToolResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "system={} model={model} temperature={temperature} message={message}",
            system_prompt.is_some()
        ))
    }
}

struct MockScheduleTool;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Tool for MockScheduleTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Mock schedule tool"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string" }
            }
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: "ok".to_string(), error: None })
    }
}

fn state() -> AppState {
    let (event_tx, _) = broadcast::channel(16);
    AppState {
        config: Arc::new(parking_lot::Mutex::new(Config::default())),
        provider: Arc::new(StaticProvider),
        model: "test-model".to_string(),
        temperature: 0.25,
        mem: Arc::new(NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(false, &[])),
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

#[tokio::test]
async fn run_gateway_chat_simple_uses_state_model_temperature_and_system_prompt() {
    let response = run_gateway_chat_simple(&state(), "hello gateway").await.unwrap();

    assert_eq!(response, "system=true model=test-model temperature=0.25 message=hello gateway");
}

#[tokio::test]
async fn invalidate_session_query_engine_allows_missing_session() {
    let state = state();

    invalidate_session_query_engine(&state, "missing-session").await;

    assert!(state.session_query_engines.lock().await.is_empty());
}

#[tokio::test]
async fn fork_session_query_engine_allows_missing_source_session() {
    let state = state();

    fork_session_query_engine(&state, "missing-source", "target-session").await.unwrap();

    assert!(state.session_query_engines.lock().await.is_empty());
}

#[tokio::test]
async fn session_query_engine_caches_forks_and_invalidates_engines() {
    let state = state();

    let first = session_query_engine(&state, "source-session").await.unwrap();
    let second = session_query_engine(&state, "source-session").await.unwrap();
    assert!(Arc::ptr_eq(&first, &second));

    fork_session_query_engine(&state, "source-session", "target-session").await.unwrap();

    {
        let engines = state.session_query_engines.lock().await;
        assert!(engines.contains_key("source-session"));
        assert!(engines.contains_key("target-session"));
    }

    invalidate_session_query_engine(&state, "source-session").await;

    let engines = state.session_query_engines.lock().await;
    assert!(!engines.contains_key("source-session"));
    assert!(engines.contains_key("target-session"));
}

#[tokio::test]
async fn run_gateway_chat_with_tools_returns_query_engine_creation_error() {
    let state = state();
    state.config.lock().default_provider = Some("unsupported-provider".to_string());

    let error = run_gateway_chat_with_tools(&state, "hello", "session-1").await.unwrap_err();

    assert!(error.to_string().contains("Unknown provider: unsupported-provider"));
    assert!(state.session_query_engines.lock().await.is_empty());
}

#[test]
fn sanitize_gateway_response_keeps_plain_text() {
    assert_eq!(sanitize_gateway_response("plain response", &[]), "plain response");
}

#[test]
fn sanitize_gateway_response_keeps_empty_and_drops_blank_text() {
    assert_eq!(sanitize_gateway_response("", &[]), "");
    assert_eq!(sanitize_gateway_response("   ", &[]), "");
}

#[test]
fn sanitize_gateway_response_returns_safe_message_when_nonempty_response_is_stripped() {
    let input = r#"ฦ
{"name":"schedule","arguments":{"action":"create"}}
ฦ"#;

    assert_eq!(
        sanitize_gateway_response(input, &[]),
        "I encountered malformed tool-call output and could not produce a safe reply. Please try again."
    );
}

#[test]
fn sanitize_gateway_response_removes_known_isolated_tool_json() {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockScheduleTool)];
    let input = r#"{"name":"schedule","parameters":{"action":"create"}}
{"result":{"status":"scheduled"}}
Reminder set."#;

    assert_eq!(sanitize_gateway_response(input, &tools), "Reminder set.");
}

#[test]
fn sanitize_gateway_response_truncates_long_text() {
    let input = "x".repeat(20_000);
    let sanitized = sanitize_gateway_response(&input, &[]);

    assert_eq!(sanitized.chars().count(), 16_001);
    assert!(sanitized.ends_with('…'));
    assert!(!sanitized.ends_with("..."));
}

#[test]
fn log_channel_message_accepts_long_content() {
    log_channel_message("test-channel", "sender-1", &"内容".repeat(80));
}
