use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[tokio::test]
async fn global_health_reports_ok_status() {
    let Json(value) = global_health().await;

    assert_eq!(value.get("status"), Some(&serde_json::Value::String("ok".to_string())));
}
