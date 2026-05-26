//! Axum-based HTTP gateway with proper HTTP/1.1 compliance, body limits, and timeouts.
//!
//! This module replaces the raw TCP implementation with axum for:
//! - Proper HTTP/1.1 parsing and compliance
//! - Content-Length validation (handled by hyper)
//! - Request body size limits (64KB max)
//! - Request timeouts (30s) to prevent slow-loris attacks
//! - Header sanitization (handled by axum/hyper)

mod agent;
mod approval_state;
pub mod api;
mod chat;
mod error;
mod handlers;
mod health;
mod instance;
mod limits;
mod middleware;
mod node_control;
mod openai_compat;
mod options;
mod pairing;
mod router;
mod runtime;
pub mod sse;
mod state;
#[cfg(test)]
mod tests;
mod types;
mod util;
mod webhook;
mod webhook_ingress;
pub mod ws;

pub use error::ApiError;
pub use handlers::{WatiVerifyQuery, WhatsAppVerifyQuery, verify_whatsapp_signature};
pub use limits::{
    GatewayRateLimiter, IDEMPOTENCY_MAX_KEYS_DEFAULT, IdempotencyStore,
    RATE_LIMIT_MAX_KEYS_DEFAULT, RATE_LIMIT_WINDOW_SECS, RATE_LIMITER_SWEEP_INTERVAL_SECS,
    SlidingWindowRateLimiter,
};
pub use node_control::NodeControlRequest;
pub use options::ServeOptions;
pub use runtime::serve;
pub use runtime::start;
pub use state::AppState;
pub use types::{AgentBody, WebhookBody};
pub use util::{
    client_key_from_request, forwarded_client_ip, hash_webhook_secret, linq_memory_key,
    nextcloud_talk_memory_key, normalize_max_keys, parse_client_ip, qq_memory_key, wati_memory_key,
    webhook_memory_key, whatsapp_memory_key,
};

use crate::app::agent::channels::{
    LinqChannel, NextcloudTalkChannel, QQChannel, WatiChannel, WhatsAppChannel,
};
use crate::app::agent::config::Config;
use crate::app::agent::memory::{self, Memory};
use crate::app::agent::providers::{self, Provider};
use crate::app::agent::runtime as agent_runtime;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::pairing::{PairingGuard, is_public_bind};
use crate::app::agent::tools::traits::ToolSpec;
use crate::app::agent::tools::{self, Tool};
use anyhow::Result;
use axum::{
    Router,
    http::HeaderValue,
    http::StatusCode,
    routing::{delete, get, post, put},
};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;

/// Maximum request body size (5MB) — prevents memory exhaustion
pub const MAX_BODY_SIZE: usize = 5_242_880;
/// Request timeout (30s) — prevents slow-loris attacks
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Run the HTTP gateway using axum with proper HTTP/1.1 compliance.
#[allow(clippy::too_many_lines)]
pub async fn run_gateway(host: &str, port: u16, config: Config) -> Result<()> {
    // ── Security: refuse public bind without tunnel or explicit opt-in ──
    if is_public_bind(host) && config.tunnel.provider == "none" && !config.gateway.allow_public_bind
    {
        anyhow::bail!(
            "🛑 Refusing to bind to {host} — gateway would be exposed to the internet.\n\
             Fix: use --host 127.0.0.1 (default), configure a tunnel, or set\n\
             [gateway] allow_public_bind = true in vibewindow.json (NOT recommended)."
        );
    }
    let config_state = Arc::new(Mutex::new(config.clone()));

    // ── Hooks ──────────────────────────────────────────────────────
    let hooks: Option<std::sync::Arc<crate::app::agent::hooks::HookRunner>> =
        if config.hooks.enabled {
            Some(std::sync::Arc::new(crate::app::agent::hooks::HookRunner::new()))
        } else {
            None
        };

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::warn!(
                "Failed to bind gateway on {host}:{port}: {err}; trying random fallback port"
            );
            tokio::net::TcpListener::bind((host, 0))
                .await
                .map_err(|fallback_err| anyhow::anyhow!(fallback_err.to_string()))?
        }
    };
    let actual_port = listener.local_addr()?.port();
    let display_addr = format!("{host}:{actual_port}");
    let cors_whitelist: Arc<Vec<String>> = Arc::new(Vec::new());
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            middleware::cors_origin_allowed(origin, cors_whitelist.as_ref())
        }))
        .allow_methods(Any)
        .allow_headers(Any);

    let provider: Arc<dyn Provider> = Arc::from(providers::create_resilient_provider_with_options(
        config.default_provider.as_deref().unwrap_or("openrouter"),
        config.api_key.as_deref(),
        config.api_url.as_deref(),
        &config.reliability,
        &providers::ProviderRuntimeOptions {
            auth_profile_override: None,
            provider_api_url: config.api_url.clone(),
            vibewindow_dir: config.config_path.parent().map(std::path::PathBuf::from),
            secrets_encrypt: config.secrets.encrypt,
            reasoning_enabled: config.runtime.reasoning_enabled,
            reasoning_level: config.effective_provider_reasoning_level(),
            custom_provider_api_mode: config.provider_api.map(|mode| mode.as_compatible_mode()),
            max_tokens_override: None,
            model_support_vision: config.model_support_vision,
        },
    )?);
    let model = config.default_model.clone().unwrap_or_else(|| "anthropic/claude-sonnet-4".into());
    let temperature = config.default_temperature;
    let mem: Arc<dyn Memory> = Arc::from(memory::create_memory_with_storage(
        &config.memory,
        Some(&config.storage.provider.config),
        &config.workspace_dir,
        config.api_key.as_deref(),
    )?);
    let runtime: Arc<dyn agent_runtime::RuntimeAdapter> =
        Arc::from(agent_runtime::create_runtime(&config.runtime)?);
    let security = Arc::new(SecurityPolicy::from_config(&config.autonomy, &config.workspace_dir));

    let (composio_key, composio_entity_id) = if config.composio.enabled {
        (config.composio.api_key.as_deref(), Some(config.composio.entity_id.as_str()))
    } else {
        (None, None)
    };

    let tools_registry_exec: Arc<Vec<Box<dyn Tool>>> = Arc::new(tools::all_tools_with_runtime(
        Arc::new(config.clone()),
        &security,
        runtime,
        Arc::clone(&mem),
        composio_key,
        composio_entity_id,
        &config.browser,
        &config.http_request,
        &config.web_fetch,
        &config.workspace_dir,
        &config.agents,
        config.api_key.as_deref(),
        &config,
        None,
    ));
    let tools_registry: Arc<Vec<ToolSpec>> =
        Arc::new(tools_registry_exec.iter().map(|t| t.spec()).collect());
    let max_tool_iterations = config.agent.max_tool_iterations;
    let multimodal_config = config.multimodal.clone();

    // SSE broadcast channel for real-time events
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel::<serde_json::Value>(256);
    // Extract webhook secret for authentication
    let webhook_secret_hash: Option<Arc<str>> =
        config.channels_config.webhook.as_ref().and_then(|webhook| {
            webhook.secret.as_ref().and_then(|raw_secret| {
                let trimmed_secret = raw_secret.trim();
                (!trimmed_secret.is_empty())
                    .then(|| Arc::<str>::from(hash_webhook_secret(trimmed_secret)))
            })
        });

    // WhatsApp channel (if configured)
    let whatsapp_channel: Option<Arc<WhatsAppChannel>> =
        config.channels_config.whatsapp.as_ref().filter(|wa| wa.is_cloud_config()).map(|wa| {
            Arc::new(WhatsAppChannel::new(
                wa.access_token.clone().unwrap_or_default(),
                wa.phone_number_id.clone().unwrap_or_default(),
                wa.verify_token.clone().unwrap_or_default(),
                wa.allowed_numbers.clone(),
            ))
        });

    // WhatsApp app secret for webhook signature verification
    // Priority: environment variable > config file
    let whatsapp_app_secret: Option<Arc<str>> = std::env::var("VIBEWINDOW_WHATSAPP_APP_SECRET")
        .ok()
        .and_then(|secret| {
            let secret = secret.trim();
            (!secret.is_empty()).then(|| secret.to_owned())
        })
        .or_else(|| {
            config.channels_config.whatsapp.as_ref().and_then(|wa| {
                wa.app_secret
                    .as_deref()
                    .map(str::trim)
                    .filter(|secret| !secret.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .map(Arc::from);

    // Linq channel (if configured)
    let linq_channel: Option<Arc<LinqChannel>> = config.channels_config.linq.as_ref().map(|lq| {
        Arc::new(LinqChannel::new(
            lq.api_token.clone(),
            lq.from_phone.clone(),
            lq.allowed_senders.clone(),
        ))
    });

    // Linq signing secret for webhook signature verification
    // Priority: environment variable > config file
    let linq_signing_secret: Option<Arc<str>> = std::env::var("VIBEWINDOW_LINQ_SIGNING_SECRET")
        .ok()
        .and_then(|secret| {
            let secret = secret.trim();
            (!secret.is_empty()).then(|| secret.to_owned())
        })
        .or_else(|| {
            config.channels_config.linq.as_ref().and_then(|lq| {
                lq.signing_secret
                    .as_deref()
                    .map(str::trim)
                    .filter(|secret| !secret.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .map(Arc::from);

    // WATI channel (if configured)
    let wati_channel: Option<Arc<WatiChannel>> =
        config.channels_config.wati.as_ref().map(|wati_cfg| {
            Arc::new(WatiChannel::new(
                wati_cfg.api_token.clone(),
                wati_cfg.api_url.clone(),
                wati_cfg.tenant_id.clone(),
                wati_cfg.allowed_numbers.clone(),
            ))
        });

    // QQ channel (if configured)
    let qq_channel: Option<Arc<QQChannel>> = config.channels_config.qq.as_ref().map(|qq_cfg| {
        Arc::new(QQChannel::new(
            qq_cfg.app_id.clone(),
            qq_cfg.app_secret.clone(),
            qq_cfg.allowed_users.clone(),
        ))
    });
    let qq_webhook_enabled = config.channels_config.qq.as_ref().is_some_and(|qq| {
        qq.receive_mode == crate::app::agent::config::schema::QQReceiveMode::Webhook
    });

    // Nextcloud Talk channel (if configured)
    let nextcloud_talk_channel: Option<Arc<NextcloudTalkChannel>> =
        config.channels_config.nextcloud_talk.as_ref().map(|nc| {
            Arc::new(NextcloudTalkChannel::new(
                nc.base_url.clone(),
                nc.app_token.clone(),
                nc.allowed_users.clone(),
            ))
        });

    // Nextcloud Talk webhook secret for signature verification
    // Priority: environment variable > config file
    let nextcloud_talk_webhook_secret: Option<Arc<str>> =
        std::env::var("VIBEWINDOW_NEXTCLOUD_TALK_WEBHOOK_SECRET")
            .ok()
            .and_then(|secret| {
                let secret = secret.trim();
                (!secret.is_empty()).then(|| secret.to_owned())
            })
            .or_else(|| {
                config.channels_config.nextcloud_talk.as_ref().and_then(|nc| {
                    nc.webhook_secret
                        .as_deref()
                        .map(str::trim)
                        .filter(|secret| !secret.is_empty())
                        .map(ToOwned::to_owned)
                })
            })
            .map(Arc::from);

    // ── Pairing guard ──────────────────────────────────────
    let pairing =
        Arc::new(PairingGuard::new(config.gateway.require_pairing, &config.gateway.paired_tokens));
    let rate_limit_max_keys =
        normalize_max_keys(config.gateway.rate_limit_max_keys, RATE_LIMIT_MAX_KEYS_DEFAULT);
    let rate_limiter = Arc::new(GatewayRateLimiter::new(
        config.gateway.pair_rate_limit_per_minute,
        config.gateway.webhook_rate_limit_per_minute,
        rate_limit_max_keys,
    ));
    let idempotency_max_keys =
        normalize_max_keys(config.gateway.idempotency_max_keys, IDEMPOTENCY_MAX_KEYS_DEFAULT);
    let idempotency_store = Arc::new(IdempotencyStore::new(
        Duration::from_secs(config.gateway.idempotency_ttl_secs.max(1)),
        idempotency_max_keys,
    ));

    // ── Tunnel ────────────────────────────────────────────────
    let tunnel = crate::app::agent::tunnel::create_tunnel(&config.tunnel)?;
    let mut tunnel_url: Option<String> = None;

    if let Some(ref tun) = tunnel {
        println!("🔗 Starting {} tunnel...", tun.name());
        match tun.start(host, actual_port).await {
            Ok(url) => {
                println!("🌐 Tunnel active: {url}");
                tunnel_url = Some(url);
            }
            Err(e) => {
                println!("⚠️  Tunnel failed to start: {e}");
                println!("   Falling back to local-only mode.");
            }
        }
    }

    println!("🦀 VibeWindow Gateway listening on http://{display_addr}");
    if let Some(ref url) = tunnel_url {
        println!("  🌐 Public URL: {url}");
    }
    println!("  POST /v1/pair              — pair a new client (X-Pairing-Code header)");
    println!("  POST /v1/webhook           — {{\"message\": \"your prompt\"}}");
    println!(
        "  POST /v1/agent             — tool-enabled agent chat {{\"message\": \"your prompt\"}}"
    );
    if whatsapp_channel.is_some() {
        println!("  GET  /v1/whatsapp          — Meta webhook verification");
        println!("  POST /v1/whatsapp          — WhatsApp message webhook");
    }
    if linq_channel.is_some() {
        println!("  POST /v1/linq              — Linq message webhook (iMessage/RCS/SMS)");
    }
    if wati_channel.is_some() {
        println!("  GET  /v1/wati              — WATI webhook verification");
        println!("  POST /v1/wati              — WATI message webhook");
    }
    if nextcloud_talk_channel.is_some() {
        println!("  POST /v1/nextcloud-talk    — Nextcloud Talk bot webhook");
    }
    if qq_webhook_enabled {
        println!("  POST /v1/qq                — QQ Bot webhook (validation + events)");
    }
    if config.gateway.node_control.enabled {
        println!("  POST /v1/node-control      — experimental node-control RPC scaffold");
    }
    println!("  POST /v1/chat/completions  — OpenAI-compatible chat");
    println!("  GET  /v1/models            — list available models");
    println!("  GET  /v1/*                 — REST API (bearer token required)");
    println!("  GET  /v1/ws/chat           — WebSocket agent chat");
    println!("  GET  /v1/health            — health check");
    println!("  GET  /v1/metrics           — Prometheus metrics");
    if let Some(code) = pairing.pairing_code() {
        println!();
        println!("  🔐 PAIRING REQUIRED — use this one-time code:");
        println!("     ┌──────────────┐");
        println!("     │  {code}  │");
        println!("     └──────────────┘");
        println!("     Send: POST /v1/pair with header X-Pairing-Code: {code}");
    } else if pairing.require_pairing() {
        println!("  🔒 Pairing: ACTIVE (bearer token required)");
    } else {
        println!("  ⚠️  Pairing: DISABLED (all requests accepted)");
    }
    println!("  Press Ctrl+C to stop.\n");

    crate::app::agent::health::mark_component_ok("gateway");

    // Fire gateway start hook
    if let Some(ref hooks) = hooks {
        hooks.fire_gateway_start(host, actual_port).await;
    }

    // Wrap observer with broadcast capability for SSE
    let broadcast_observer: Arc<dyn crate::app::agent::observability::Observer> =
        Arc::new(sse::BroadcastObserver::new(
            crate::app::agent::observability::create_observer(&config.observability),
            event_tx.clone(),
        ));

    let state = AppState {
        config: config_state,
        provider,
        model,
        temperature,
        mem,
        auto_save: config.memory.auto_save,
        webhook_secret_hash,
        pairing,
        trust_forwarded_headers: config.gateway.trust_forwarded_headers,
        rate_limiter,
        idempotency_store,
        whatsapp: whatsapp_channel,
        whatsapp_app_secret,
        linq: linq_channel,
        linq_signing_secret,
        nextcloud_talk: nextcloud_talk_channel,
        nextcloud_talk_webhook_secret,
        wati: wati_channel,
        qq: qq_channel,
        qq_webhook_enabled,
        observer: broadcast_observer,
        tools_registry,
        tools_registry_exec,
        multimodal: multimodal_config,
        max_tool_iterations,
        event_tx,
        session_query_engines: Default::default(),
    };

    // The OpenAI-compatible chat completions endpoint uses a larger body limit (512KB).
    let openai_compat_route = Router::<AppState>::new()
        .route("/chat/completions", post(openai_compat::handle_v1_chat_completions))
        .layer(RequestBodyLimitLayer::new(openai_compat::CHAT_COMPLETIONS_MAX_BODY_SIZE));

    // Dashboard config (TOML) needs larger body limit (1MB)
    let dashboard_config_route = Router::<AppState>::new()
        .route("/config", get(api::handle_api_config_get).put(api::handle_api_config_put))
        .layer(RequestBodyLimitLayer::new(1_048_576));

    // ── Handler module routers (instance-aware, no overlapping with legacy routes) ──
    let handler_router = Router::<AppState>::new()
        .merge(api::handlers::auth::router())
        .merge(api::handlers::file::router())
        .merge(api::handlers::git::router())
        .merge(api::handlers::global::router())
        .merge(api::handlers::instance::router())
        .merge(api::handlers::misc::router())
        .merge(api::handlers::permission::router())
        .merge(api::handlers::project::router::<AppState>())
        .merge(api::handlers::provider::router())
        .merge(api::handlers::pty::router())
        .merge(api::handlers::question::router())
        .merge(api::handlers::data::router())
        .merge(api::handlers::redis::router())
        .merge(api::handlers::workflow::router())
        .merge(api::handlers::session::router())
        .merge(api::handlers::config::router())
        .merge(api::handlers::desktop::router());

    let app = Router::<AppState>::new()
        // ── Infrastructure routes ──
        .route("/v1/health", get(health::handle_health))
        .route("/v1/metrics", get(health::handle_metrics))
        .route("/v1/pair", post(pairing::handle_pair))
        .route("/v1/pair-code", get(pairing::handle_pair_code))
        .route("/v1/webhook", post(webhook::handle_webhook))
        .route("/v1/agent", post(agent::handle_agent))
        .route("/v1/whatsapp", get(handlers::handle_whatsapp_verify))
        .route("/v1/whatsapp", post(handlers::handle_whatsapp_message))
        .route("/v1/linq", post(handlers::handle_linq_webhook))
        .route("/v1/wati", get(handlers::handle_wati_verify))
        .route("/v1/wati", post(handlers::handle_wati_webhook))
        .route("/v1/nextcloud-talk", post(handlers::handle_nextcloud_talk_webhook))
        .route("/v1/qq", post(handlers::handle_qq_webhook))
        // ── OpenAI-compatible endpoints ──
        .route("/v1/models", get(openai_compat::handle_v1_models))
        .nest("/v1", openai_compat_route)
        // ── WebSocket agent chat ──
        .route("/v1/ws/chat", get(ws::handle_ws_chat))
        // ── Node control ──
        .route("/v1/node-control", post(node_control::handle_node_control))
        // ── SSE event stream ──
        .route("/v1/events", get(sse::handle_sse_events))
        // ── Dashboard API routes (State-based auth) ──
        .route("/v1/status", get(api::handle_api_status))
        .route("/v1/tools", get(api::handle_api_tools))
        .route("/v1/cron", get(api::handle_api_cron_list))
        .route("/v1/cron", post(api::handle_api_cron_add))
        .route("/v1/cron/{id}", delete(api::handle_api_cron_delete))
        .route("/v1/integrations", get(api::handle_api_integrations))
        .route("/v1/integrations/settings", get(api::handle_api_integrations_settings))
        .route(
            "/v1/integrations/{id}/credentials",
            put(api::handle_api_integration_credentials_put),
        )
        .route("/v1/doctor", get(api::handle_api_doctor).post(api::handle_api_doctor))
        .route("/v1/memory", get(api::handle_api_memory_list))
        .route("/v1/memory", post(api::handle_api_memory_store))
        .route("/v1/memory/{key}", delete(api::handle_api_memory_delete))
        .route("/v1/cli-tools", get(api::handle_api_cli_tools))
        .route("/v1/api-health", get(api::handle_api_health))
        // ── Dashboard config (TOML) ──
        .nest("/v1/dashboard", dashboard_config_route)
        // ── All handler module routers nested under /v1 ──
        .nest("/v1", handler_router)
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
        ))
        .layer(cors);

    // Run the server
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
