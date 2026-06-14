use super::*;

use crate::app::agent::config::Config;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::get;
use std::time::Duration;
use tower::util::ServiceExt;

#[test]
fn workflow_chat_messages_path_matches_collection_and_application_routes() {
    assert!(is_workflow_chat_messages_path("/v1/workflow/applications/chat-messages"));
    assert!(is_workflow_chat_messages_path("/v1/workflow/applications/demo/chat-messages"));
    assert!(is_workflow_chat_messages_path(
        "/v1/workflow/applications/demo/sub-path/chat-messages"
    ));
}

#[test]
fn workflow_chat_messages_path_rejects_empty_or_partial_matches() {
    assert!(!is_workflow_chat_messages_path("/v1/workflow/applications//chat-messages"));
    assert!(!is_workflow_chat_messages_path("/v1/workflow/applications/   /chat-messages"));
    assert!(!is_workflow_chat_messages_path("/v1/workflow/applications/demo/chat-messages/extra"));
    assert!(!is_workflow_chat_messages_path("/v1/workflow/application/demo/chat-messages"));
}

#[test]
fn timeout_for_path_uses_extended_window_only_for_workflow_chat_messages() {
    assert_eq!(
        request_timeout_for_path("/v1/workflow/applications/chat-messages"),
        Duration::from_secs(WORKFLOW_CHAT_MESSAGES_TIMEOUT_SECS)
    );
    assert_eq!(
        request_timeout_for_path("/v1/workflow/applications/demo/chat-messages"),
        Duration::from_secs(WORKFLOW_CHAT_MESSAGES_TIMEOUT_SECS)
    );
    assert_eq!(
        request_timeout_for_path("/v1/workflow/applications/demo/messages"),
        Duration::from_secs(REQUEST_TIMEOUT_SECS)
    );
}

#[tokio::test]
async fn run_gateway_rejects_public_bind_without_tunnel_or_opt_in() {
    let mut config = Config::default();
    config.tunnel.provider = "none".to_string();
    config.gateway.allow_public_bind = false;

    let error = run_gateway("0.0.0.0", 0, config).await.unwrap_err();

    assert!(error.to_string().contains("Refusing to bind to 0.0.0.0"));
}

#[tokio::test]
async fn request_timeout_middleware_returns_inner_response_before_deadline() {
    let app = Router::new()
        .route("/ok", get(|| async { StatusCode::CREATED }))
        .layer(from_fn(request_timeout_middleware));

    let response = app.oneshot(Request::get("/ok").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test(start_paused = true)]
async fn request_timeout_middleware_returns_timeout_after_deadline() {
    let app = Router::new()
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(REQUEST_TIMEOUT_SECS + 1)).await;
                StatusCode::OK
            }),
        )
        .layer(from_fn(request_timeout_middleware));

    let response = app.oneshot(Request::get("/slow").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
}
