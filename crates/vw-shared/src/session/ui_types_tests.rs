#[test]
fn token_usage_defaults_reasoning_tokens() {
    let usage: super::TokenUsage =
        serde_json::from_str(r#"{"input_tokens":1,"output_tokens":2,"cached_tokens":3}"#).unwrap();

    assert_eq!(usage.reasoning_tokens, 0);
}

#[test]
fn chat_message_skips_empty_think_timing() {
    let message = super::ChatMessage {
        role: super::ChatRole::Assistant,
        content: "done".to_string(),
        think_timing: Vec::new(),
    };

    let value = serde_json::to_value(message).unwrap();

    assert_eq!(value["role"], "Assistant");
    assert!(value.get("think_timing").is_none());
}

#[test]
fn chat_session_defaults_collection_fields() {
    let session: super::ChatSession =
        serde_json::from_str(r#"{"id":"s1","title":"Session"}"#).unwrap();

    assert!(session.messages.is_empty());
    assert!(session.calls.is_empty());
    assert_eq!(session.created_ms, 0);
}
