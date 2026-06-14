use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{CoordinationEnvelope, CoordinationPayload};

#[test]
fn dead_letter_readers_filter_by_correlation() {
    let bus = InMemoryMessageBus::new();
    let mut envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "stop".to_string(), note: None },
    );
    envelope.correlation_id = Some("corr-a".to_string());

    bus.push_dead_letter(envelope, "undeliverable".to_string());

    assert_eq!(bus.dead_letter_count(), 1);
    assert_eq!(bus.dead_letter_count_for_correlation("corr-a"), 1);
    assert_eq!(bus.dead_letters_recent_for_correlation("corr-a", 0, 1)[0].reason, "undeliverable");
}

#[test]
fn dead_letter_readers_return_empty_for_blank_or_missing_correlation() {
    let bus = InMemoryMessageBus::new();
    let envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "stop".to_string(), note: None },
    );

    bus.push_dead_letter(envelope, "undeliverable".to_string());

    assert!(bus.dead_letters_recent_for_correlation(" ", 0, 0).is_empty());
    assert!(bus.dead_letters_recent_for_correlation("missing", 0, 0).is_empty());
    assert_eq!(bus.dead_letter_count_for_correlation(" "), 0);
    assert_eq!(bus.dead_letter_count_for_correlation("missing"), 0);
    assert_eq!(bus.dead_letters_recent(0, 0).len(), 1);
}
