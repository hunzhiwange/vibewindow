use super::super::InMemoryMessageBus;

#[test]
fn metadata_reflects_registered_agents_and_empty_stats() {
    let bus = InMemoryMessageBus::new();
    bus.register_agent("agent-b").expect("register b");
    bus.register_agent("agent-a").expect("register a");

    assert_eq!(bus.registered_agents(), vec!["agent-a".to_string(), "agent-b".to_string()]);
    assert_eq!(bus.subscriber_count(), 2);
    assert_eq!(bus.stats().publish_attempts_total, 0);
}
