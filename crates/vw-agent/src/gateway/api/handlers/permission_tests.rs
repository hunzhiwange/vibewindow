use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn permission_reply_request_deserializes_reply_field() {
    let body: PermissionReplyRequest = serde_json::from_value(serde_json::json!({"reply": "once"}))
        .expect("valid permission reply");

    assert_eq!(body.reply, permission_next::Reply::Once);
    assert!(body.message.is_none());
}
