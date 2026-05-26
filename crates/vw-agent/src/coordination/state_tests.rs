use super::state::BusState;
use super::types::InMemoryMessageBusLimits;

#[test]
fn bus_state_preserves_supplied_limits() {
    let limits = InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 2,
        max_dead_letters: 3,
        max_context_entries: 4,
        max_seen_message_ids: 5,
    };

    let state = BusState::with_limits(limits);

    assert_eq!(state.limits, limits);
    assert_eq!(state.next_sequence, 1);
}
