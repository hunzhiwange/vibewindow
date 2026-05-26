use super::bus_inbox::push_inbox_entry_locked;
use super::envelope::{CoordinationEnvelope, CoordinationPayload};
use super::state::BusState;
use super::types::{InMemoryMessageBusLimits, SequencedEnvelope};

#[test]
fn push_inbox_entry_locked_evicts_oldest_message() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 1,
        ..InMemoryMessageBusLimits::recommended()
    });
    state.inboxes.entry("agent-a".to_string()).or_default();
    let envelope = CoordinationEnvelope::new_direct(
        "source",
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Ack { acked_message_id: "a".to_string() },
    );

    push_inbox_entry_locked(
        &mut state,
        "agent-a",
        SequencedEnvelope { sequence: 1, envelope: envelope.clone() },
    );
    push_inbox_entry_locked(&mut state, "agent-a", SequencedEnvelope { sequence: 2, envelope });

    let inbox = state.inboxes.get("agent-a").expect("inbox should exist");
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].sequence, 2);
    assert_eq!(state.stats.inbox_overflow_evictions_total, 1);
}
