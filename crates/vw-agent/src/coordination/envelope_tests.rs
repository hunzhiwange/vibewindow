use super::envelope::{CoordinationEnvelope, CoordinationPayload, DeliveryScope};
use super::errors::CoordinationError;
use serde_json::json;

#[test]
fn direct_envelope_validates_target() {
    let envelope = CoordinationEnvelope::new_direct(
        "agent-a",
        "agent-b",
        "conversation-a",
        "topic",
        CoordinationPayload::DelegateTask {
            task_id: "task-a".to_string(),
            summary: "summary".to_string(),
            metadata: json!({}),
        },
    );

    assert_eq!(envelope.scope, DeliveryScope::Direct);
    assert!(envelope.validate().is_ok());
}

#[test]
fn broadcast_envelope_rejects_direct_only_payload() {
    let envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::DelegateTask {
            task_id: "task-a".to_string(),
            summary: "summary".to_string(),
            metadata: json!({}),
        },
    );

    assert!(envelope.validate().is_err());
}

#[test]
fn broadcast_envelope_rejects_explicit_target() {
    let mut envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: "pause".to_string(), note: None },
    );
    envelope.id = "broadcast-with-target".to_string();
    envelope.to = Some("agent-b".to_string());

    assert_eq!(
        envelope.validate(),
        Err(CoordinationError::BroadcastHasTarget {
            message_id: "broadcast-with-target".to_string(),
        })
    );
}

#[test]
fn control_payload_rejects_empty_action() {
    let envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "topic",
        CoordinationPayload::Control { action: " ".to_string(), note: None },
    );

    assert_eq!(envelope.validate(), Err(CoordinationError::EmptyField { field: "action" }));
}
