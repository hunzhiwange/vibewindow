use super::*;

#[test]
fn inputs_use_gateway_field_names() {
    let summarize = serde_json::to_value(SummarizeInput {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
    })
    .unwrap();
    assert_eq!(summarize["sessionID"], "s1");
    assert_eq!(summarize["messageID"], "m1");

    let diff = serde_json::to_value(DiffInput {
        session_id: "s1".to_string(),
        message_id: None,
    })
    .unwrap();
    assert_eq!(diff["sessionID"], "s1");
    assert!(diff.get("messageID").is_none());
}
