use super::*;

#[test]
fn audit_event_builder_preserves_actor_action_and_result() {
    let event = AuditEvent::new(AuditEventType::CommandExecution)
        .with_actor("agent".into(), Some("user-1".into()), Some("Ada".into()))
        .with_action("ls".into(), "low".into(), true, true)
        .with_result(true, Some(0), 7, None);

    assert_eq!(event.actor.unwrap().channel, "agent");
    assert_eq!(event.action.unwrap().command.as_deref(), Some("ls"));
    assert!(event.result.unwrap().success);
}
