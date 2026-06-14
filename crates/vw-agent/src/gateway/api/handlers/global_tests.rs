use super::*;
use axum::response::IntoResponse;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[tokio::test]
async fn global_health_reports_ok_status() {
    let Json(value) = global_health().await;

    assert_eq!(value.get("status"), Some(&serde_json::Value::String("ok".to_string())));
    assert_eq!(value.get("healthy"), Some(&serde_json::Value::Bool(true)));
    assert!(value.get("version").is_some());
    assert!(value.get("health").is_some());
}

#[tokio::test]
async fn global_config_get_serializes_current_config() {
    let Json(value) = global_config_get().await;

    assert!(value.is_object());
    assert!(value.get("default_provider").is_some());
}

#[tokio::test]
async fn global_config_acp_get_returns_merged_agent_specs() {
    let Json(value) = global_config_acp_get().await;

    let specs = value.as_object().expect("acp config should serialize as an object");
    assert!(!specs.is_empty());
    assert!(specs.values().all(|spec| spec.get("command").is_some()));
}

#[tokio::test]
async fn global_config_patch_rejects_non_object_patch() {
    let err = global_config_patch(Json(serde_json::json!("bad-patch")))
        .await
        .expect_err("non-object patch should be rejected");

    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
    assert!(err.to_string().contains("config patch must be a JSON object"));
}

#[tokio::test]
async fn global_dispose_returns_true_response() {
    let Json(disposed) = global_dispose().await.expect("dispose should succeed");

    assert!(disposed);
}

#[tokio::test]
async fn global_event_sse_response_uses_event_stream_content_type() {
    let response = global_event_sse().await.into_response();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        response.headers().get(axum::http::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()),
        Some("text/event-stream")
    );
}
