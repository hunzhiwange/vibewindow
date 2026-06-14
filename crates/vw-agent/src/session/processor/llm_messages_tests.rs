use super::*;
use crate::session::session::{Message, Role, Session};
use serde_json::json;

fn message(role: Role, content: &str) -> Message {
    Message { role, content: content.to_string() }
}

fn session_with_roles() -> Session {
    Session {
        id: "session-1".to_string(),
        messages: vec![
            message(Role::System, "system prompt"),
            message(Role::User, "user turn"),
            message(Role::Assistant, "assistant turn"),
            message(Role::Tool, "tool result"),
        ],
    }
}

#[test]
fn session_message_to_llm_message_maps_all_roles() {
    assert_eq!(
        session_message_to_llm_message(&message(Role::User, "hello")),
        Some(json!({"role": "user", "content": "hello"}))
    );
    assert_eq!(
        session_message_to_llm_message(&message(Role::Assistant, "hi")),
        Some(json!({"role": "assistant", "content": "hi"}))
    );
    assert_eq!(
        session_message_to_llm_message(&message(Role::System, "rules")),
        Some(json!({"role": "system", "content": "rules"}))
    );
    assert_eq!(
        session_message_to_llm_message(&message(Role::Tool, "tool output")),
        Some(json!({"role": "assistant", "content": "tool output"}))
    );
}

#[test]
fn session_messages_to_llm_messages_preserves_order_and_content() {
    let converted = session_messages_to_llm_messages(&session_with_roles());

    assert_eq!(
        converted,
        vec![
            json!({"role": "system", "content": "system prompt"}),
            json!({"role": "user", "content": "user turn"}),
            json!({"role": "assistant", "content": "assistant turn"}),
            json!({"role": "assistant", "content": "tool result"}),
        ]
    );
}

#[test]
fn extend_llm_messages_from_session_range_appends_tail_only() {
    let session = session_with_roles();
    let mut llm_messages = vec![json!({"role": "system", "content": "existing"})];

    extend_llm_messages_from_session_range(&mut llm_messages, &session, 2);

    assert_eq!(
        llm_messages,
        vec![
            json!({"role": "system", "content": "existing"}),
            json!({"role": "assistant", "content": "assistant turn"}),
            json!({"role": "assistant", "content": "tool result"}),
        ]
    );
}

#[test]
fn extend_llm_messages_from_session_range_allows_start_past_end() {
    let session = session_with_roles();
    let mut llm_messages = vec![json!({"role": "user", "content": "existing"})];

    extend_llm_messages_from_session_range(&mut llm_messages, &session, usize::MAX);

    assert_eq!(llm_messages, vec![json!({"role": "user", "content": "existing"})]);
}
