use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[tokio::test]
async fn not_implemented_returns_501() {
    let error = not_implemented().await.expect_err("endpoint is intentionally unavailable");

    assert_eq!(error.status, axum::http::StatusCode::NOT_IMPLEMENTED);
    assert_eq!(error.message, "not implemented");
}
