use super::bus_publish::publish_envelope;
use super::envelope::{CoordinationEnvelope, CoordinationPayload};
use super::state::BusState;
use super::types::InMemoryMessageBusLimits;

#[test]
fn publish_envelope_direct_delivers_to_registered_agent() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits::recommended());
    state.inboxes.entry("agent-b".to_string()).or_default();
    let envelope = CoordinationEnvelope::new_direct(
        "agent-a",
        "agent-b",
        "conversation-a",
        "topic",
        CoordinationPayload::Ack { acked_message_id: "message-a".to_string() },
    );

    let receipt = publish_envelope(&mut state, envelope).expect("publish should succeed");

    assert_eq!(receipt.delivered_to, 1);
    assert_eq!(state.inboxes["agent-b"].len(), 1);
}
