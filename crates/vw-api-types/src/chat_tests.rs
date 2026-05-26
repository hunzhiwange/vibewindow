use super::chat::{ChatContextDto, ChatOptionsDto, ChatRole, GatewayChatStreamEvent, GatewayChatStreamRequest, MessagePartDto, MessageStatus};
use serde_json::json;

#[test]
fn chat_enums_use_snake_case_protocol_names() {
    assert_eq!(serde_json::to_value(ChatRole::Assistant).expect("role"), json!("assistant"));
    assert_eq!(serde_json::to_value(MessageStatus::Streaming).expect("status"), json!("streaming"));
    assert_eq!(
        serde_json::to_value(MessagePartDto::Text { text: "hi".into() }).expect("part"),
        json!({"type": "text", "text": "hi"})
    );
}

#[test]
fn chat_defaults_keep_optional_fields_empty() {
    let request = GatewayChatStreamRequest::default();
    assert!(request.messages.is_empty());
    assert_eq!(ChatOptionsDto::default().model, None);
    assert!(ChatContextDto::default().selected_file_paths.is_empty());
    assert_eq!(GatewayChatStreamEvent::Error("bad".into()), GatewayChatStreamEvent::Error("bad".into()));
}
