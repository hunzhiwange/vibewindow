use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{
    CoordinationEnvelope, CoordinationPayload, InMemoryMessageBusLimits, InMemoryMessageBusStats,
};
use serde_json::json;

#[test]
fn metadata_reflects_registered_agents_and_empty_stats() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("agent-b").expect("register b");
    bus.register_agent("agent-a").expect("register a");

    assert_eq!(bus.registered_agents(), vec!["agent-a".to_string(), "agent-b".to_string()]);
    assert_eq!(bus.subscriber_count(), 2);
    assert_eq!(bus.stats(), InMemoryMessageBusStats::default());
}

#[test]
fn metadata_limits_report_effective_values_after_zero_capacity_normalization() {
    let bus = InMemoryMessageBus::with_limits(InMemoryMessageBusLimits {
        max_inbox_messages_per_agent: 0,
        max_dead_letters: 0,
        max_context_entries: 0,
        max_seen_message_ids: 0,
    });

    assert_eq!(
        bus.limits(),
        InMemoryMessageBusLimits {
            max_inbox_messages_per_agent: 1,
            max_dead_letters: 1,
            max_context_entries: 1,
            max_seen_message_ids: 1,
        }
    );
    assert_eq!(bus.subscriber_count(), 0);
}

#[test]
fn metadata_snapshots_are_copied_before_later_mutations() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("worker").expect("register worker");

    let initial_agents = bus.registered_agents();
    let initial_stats = bus.stats();

    let mut envelope = CoordinationEnvelope::new_direct(
        "lead",
        "worker",
        "conv-metadata",
        "coordination",
        CoordinationPayload::DelegateTask {
            task_id: "task-1".to_string(),
            summary: "metadata".to_string(),
            metadata: json!({}),
        },
    );
    envelope.id = "msg-metadata-1".to_string();
    bus.publish(envelope).expect("publish metadata message");
    bus.register_agent("observer").expect("register observer");

    assert_eq!(initial_agents, vec!["worker".to_string()]);
    assert_eq!(initial_stats, InMemoryMessageBusStats::default());

    assert_eq!(bus.registered_agents(), vec!["observer".to_string(), "worker".to_string()]);
    assert_eq!(bus.subscriber_count(), 2);
    assert_eq!(
        bus.stats(),
        InMemoryMessageBusStats {
            publish_attempts_total: 1,
            deliveries_total: 1,
            ..InMemoryMessageBusStats::default()
        }
    );
}
