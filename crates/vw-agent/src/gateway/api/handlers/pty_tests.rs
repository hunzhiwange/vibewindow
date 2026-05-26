use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn pty_connect_query_deserializes_cursor() {
    let query: PtyConnectQuery =
        serde_json::from_value(serde_json::json!({"cursor": 42})).expect("valid query");

    assert_eq!(query.cursor, Some(42));
}
