use super::*;

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
