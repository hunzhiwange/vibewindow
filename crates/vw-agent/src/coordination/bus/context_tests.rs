use super::super::InMemoryMessageBus;
use crate::app::agent::coordination::{CoordinationEnvelope, CoordinationPayload};
use serde_json::{Value, json};

fn publish_context(
    bus: &InMemoryMessageBus,
    key: &str,
    expected_version: u64,
    value: Value,
    correlation_id: Option<&str>,
) {
    let mut envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::ContextPatch { key: key.to_string(), expected_version, value },
    );
    envelope.correlation_id = correlation_id.map(str::to_string);
    bus.publish(envelope).expect("context patch should publish");
}

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

#[test]
fn context_readers_handle_unbounded_queries_and_empty_correlations() {
    let bus = InMemoryMessageBus::new();

    publish_context(&bus, "key-a", 0, json!(1), Some("corr-a"));
    publish_context(&bus, "key-b", 0, json!(2), None);
    publish_context(&bus, "delegate/corr-a/state", 0, json!(3), Some("corr-a"));

    let recent_keys =
        bus.context_entries_recent(0).into_iter().map(|(key, _)| key).collect::<Vec<_>>();
    assert_eq!(recent_keys, vec!["delegate/corr-a/state", "key-b", "key-a"]);

    let correlated_keys = bus
        .context_entries_recent_for_correlation(" corr-a ", 0)
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    assert_eq!(correlated_keys, vec!["delegate/corr-a/state", "key-a"]);
    assert!(bus.context_entries_recent_for_correlation(" ", 0).is_empty());
    assert!(bus.context_entries_recent_for_correlation("missing", 0).is_empty());
    assert_eq!(bus.context_count_for_correlation(" "), 0);
    assert_eq!(bus.context_count_for_correlation("missing"), 0);

    assert_eq!(bus.delegate_context_entries_recent_with_offset(0, 0).len(), 1);
    assert_eq!(
        bus.delegate_context_entries_recent_for_correlation_with_offset("corr-a", 0, 0).len(),
        1
    );
    assert!(bus.delegate_context_entries_recent_for_correlation_with_offset(" ", 0, 0).is_empty());
    assert!(
        bus.delegate_context_entries_recent_for_correlation_with_offset("missing", 0, 0).is_empty()
    );
    assert_eq!(bus.delegate_context_count(), 1);
    assert_eq!(bus.delegate_context_count_for_correlation(" "), 0);
    assert_eq!(bus.delegate_context_count_for_correlation("missing"), 0);
    assert_eq!(bus.context_entry("key-b").expect("key-b should exist").value, json!(2));
}
