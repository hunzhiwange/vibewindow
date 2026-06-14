use super::bus_dead_letters::push_dead_letter_locked;
use super::envelope::{CoordinationEnvelope, CoordinationPayload};
use super::state::BusState;
use super::types::InMemoryMessageBusLimits;

#[test]
fn push_dead_letter_locked_applies_capacity_limit() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits {
        max_dead_letters: 1,
        ..InMemoryMessageBusLimits::recommended()
    });
    let first = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "first".to_string(), note: None },
    );
    let second = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "second".to_string(), note: None },
    );

    push_dead_letter_locked(&mut state, first, "no target".to_string());
    push_dead_letter_locked(&mut state, second, "still no target".to_string());

    assert_eq!(state.dead_letters.len(), 1);
    assert_eq!(state.stats.dead_letter_evictions_total, 1);
}

#[test]
fn push_dead_letter_locked_removes_empty_correlation_index_on_eviction() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits {
        max_dead_letters: 1,
        ..InMemoryMessageBusLimits::recommended()
    });
    let mut first = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "first".to_string(), note: None },
    );
    first.correlation_id = Some("corr-a".to_string());
    let second = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "second".to_string(), note: None },
    );

    push_dead_letter_locked(&mut state, first, "first failure".to_string());
    push_dead_letter_locked(&mut state, second, "second failure".to_string());

    assert_eq!(state.dead_letters.len(), 1);
    assert!(!state.dead_letters_by_correlation.contains_key("corr-a"));
}
