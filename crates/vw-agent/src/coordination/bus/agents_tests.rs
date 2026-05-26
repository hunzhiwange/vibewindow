use super::super::InMemoryMessageBus;

#[test]
fn register_and_unregister_agent_are_idempotent() {
    let bus = InMemoryMessageBus::new();

    bus.register_agent("agent-a").expect("register should succeed");
    bus.register_agent("agent-a").expect("second register should succeed");
    assert_eq!(bus.registered_agents(), vec!["agent-a".to_string()]);

    bus.unregister_agent("agent-a");
    bus.unregister_agent("agent-a");
    assert!(bus.registered_agents().is_empty());
}
