use super::*;
use axum::extract::{Path, Query};
use axum::http::HeaderMap;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn oauth_request_bodies_deserialize() {
    let authorize: OAuthAuthorizeRequest =
        serde_json::from_value(serde_json::json!({"method": 2})).expect("authorize body");
    let callback: OAuthCallbackRequest = serde_json::from_value(serde_json::json!({
        "method": 2,
        "code": "abc"
    }))
    .expect("callback body");

    assert_eq!(authorize.method, 2);
    assert_eq!(callback.code.as_deref(), Some("abc"));
}

#[tokio::test]
async fn provider_auth_methods_delegates_to_auth_module() {
    let Json(methods) = provider_auth_methods().await;

    assert!(methods.is_empty());
}

#[tokio::test]
async fn provider_list_returns_defaults_and_sorted_connections() {
    let temp = tempfile::tempdir().expect("tempdir");

    let Json(response) = provider_list(
        Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) }),
        HeaderMap::new(),
    )
    .await
    .expect("providers should list");

    let mut sorted_connected = response.connected.clone();
    sorted_connected.sort();
    assert_eq!(response.connected, sorted_connected);
    for provider in &response.all {
        if !provider.models.is_empty() {
            assert!(response.default.contains_key(&provider.id));
        }
    }
}

#[tokio::test]
async fn oauth_authorize_maps_empty_authorization() {
    let Json(result) = provider_oauth_authorize(
        Path("openai".to_string()),
        Json(OAuthAuthorizeRequest { method: 0 }),
    )
    .await
    .expect("authorize should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn oauth_callback_maps_unsupported_to_bad_request() {
    let error = provider_oauth_callback(
        Path("openai".to_string()),
        Json(OAuthCallbackRequest { method: 0, code: Some("code".to_string()) }),
    )
    .await
    .expect_err("unsupported callback should be rejected");

    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "unsupported auth method");
}
