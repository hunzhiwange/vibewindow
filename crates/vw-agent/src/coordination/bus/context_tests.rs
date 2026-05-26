use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{CoordinationEnvelope, CoordinationPayload};
use serde_json::json;

#[test]
fn context_readers_return_recent_entries_with_offsets() {
    let bus = InMemoryMessageBus::new();
    let envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::ContextPatch {
            key: "key-a".to_string(),
            expected_version: 0,
            value: json!(1),
        },
    );

    bus.publish(envelope).expect("context patch should publish");

    assert_eq!(bus.context_count(), 1);
    assert_eq!(bus.context_entries_recent_with_offset(0, 1)[0].1.key, "key-a");
    assert!(bus.context_entries_recent_with_offset(1, 1).is_empty());
}
