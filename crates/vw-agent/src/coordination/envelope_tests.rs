use super::envelope::{CoordinationEnvelope, CoordinationPayload, DeliveryScope};
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
