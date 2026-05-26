use super::*;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn question_reply_request_deserializes_answers() {
    let body: QuestionReplyRequest =
        serde_json::from_value(serde_json::json!({"answers": [["yes"]]})).expect("valid reply");

    assert_eq!(body.answers, vec![vec!["yes".to_string()]]);
}
