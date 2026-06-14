use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{
    CoordinationEnvelope, CoordinationError, CoordinationPayload,
};

fn ack_envelope(message_id: &str) -> CoordinationEnvelope {
    let mut envelope = CoordinationEnvelope::new_direct(
        "agent-a",
        "agent-b",
        "conversation-a",
        "topic",
        CoordinationPayload::Ack { acked_message_id: message_id.to_string() },
    );
    envelope.id = format!("envelope-{message_id}");
    envelope
}

#[test]
fn inbox_readers_peek_and_drain_registered_agent() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("agent-b").expect("register should succeed");
    let envelope = ack_envelope("message-a");

    bus.publish(envelope).expect("publish should succeed");

    assert_eq!(bus.pending_for_agent("agent-b").expect("pending count"), 1);
    assert_eq!(bus.peek_for_agent("agent-b", 1).expect("peek").len(), 1);
    assert_eq!(bus.drain_for_agent("agent-b", 0).expect("drain").len(), 1);
    assert_eq!(bus.pending_for_agent("agent-b").expect("pending count"), 0);
}

#[test]
fn inbox_readers_cover_errors_empty_correlation_and_unbounded_peek() {
    let bus = InMemoryMessageBus::new();

    assert!(matches!(
        bus.pending_for_agent("missing"),
        Err(CoordinationError::UnknownAgent { agent }) if agent == "missing"
    ));
    assert_eq!(bus.pending_for_agent_correlation("missing", " "), Ok(0));
    assert!(matches!(
        bus.pending_for_agent_correlation("missing", "corr-a"),
        Err(CoordinationError::UnknownAgent { agent }) if agent == "missing"
    ));

    bus.register_agent("agent-b").expect("register should succeed");
    let mut first = ack_envelope("message-a");
    first.correlation_id = Some(" corr-a ".to_string());
    let mut second = ack_envelope("message-b");
    second.correlation_id = Some("corr-a".to_string());
    bus.publish(first).expect("first publish should succeed");
    bus.publish(second).expect("second publish should succeed");

    assert_eq!(bus.peek_for_agent_with_offset("agent-b", 1, 0).expect("peek").len(), 1);
    assert!(
        bus.peek_for_agent_correlation_with_offset("agent-b", " ", 0, 1)
            .expect("blank correlation should succeed")
            .is_empty()
    );
    assert!(matches!(
        bus.peek_for_agent_correlation_with_offset("missing", "corr-a", 0, 1),
        Err(CoordinationError::UnknownAgent { agent }) if agent == "missing"
    ));

    assert_eq!(bus.pending_for_agent_correlation("agent-b", "corr-a").expect("count"), 2);
    assert_eq!(bus.drain_for_agent("agent-b", 1).expect("drain one").len(), 1);
    assert_eq!(bus.pending_for_agent_correlation("agent-b", "corr-a").expect("count"), 1);
}
