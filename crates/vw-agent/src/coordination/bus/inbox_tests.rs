use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{CoordinationEnvelope, CoordinationPayload};

#[test]
fn inbox_readers_peek_and_drain_registered_agent() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("agent-b").expect("register should succeed");
    let envelope = CoordinationEnvelope::new_direct(
        "agent-a",
        "agent-b",
        "conversation-a",
        "topic",
        CoordinationPayload::Ack { acked_message_id: "message-a".to_string() },
    );

    bus.publish(envelope).expect("publish should succeed");

    assert_eq!(bus.pending_for_agent("agent-b").expect("pending count"), 1);
    assert_eq!(bus.peek_for_agent("agent-b", 1).expect("peek").len(), 1);
    assert_eq!(bus.drain_for_agent("agent-b", 0).expect("drain").len(), 1);
    assert_eq!(bus.pending_for_agent("agent-b").expect("pending count"), 0);
}
