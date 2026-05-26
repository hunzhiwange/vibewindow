use super::bus_context::apply_context_patch_locked;
use super::envelope::{CoordinationEnvelope, CoordinationPayload};
use super::state::BusState;
use super::types::InMemoryMessageBusLimits;
use serde_json::json;

fn context_envelope(correlation_id: Option<&str>) -> CoordinationEnvelope {
    let mut envelope = CoordinationEnvelope::new_broadcast(
        "agent-a",
        "conversation-a",
        "context.patch",
        CoordinationPayload::ContextPatch {
            key: "shared/key".to_string(),
            expected_version: 0,
            value: json!({"ok": true}),
        },
    );
    envelope.correlation_id = correlation_id.map(str::to_string);
    envelope
}

#[test]
fn apply_context_patch_locked_increments_version_and_indexes_correlation() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits::recommended());
    let envelope = context_envelope(Some("corr-a"));

    apply_context_patch_locked(&mut state, &envelope, "shared/key", 0, &json!("value"))
        .expect("initial patch should be accepted");

    let entry = state.context.get("shared/key").expect("entry should exist");
    assert_eq!(entry.version, 1);
    assert_eq!(state.context_order_by_correlation["corr-a"][0], "shared/key");
}

#[test]
fn apply_context_patch_locked_rejects_stale_version() {
    let mut state = BusState::with_limits(InMemoryMessageBusLimits::recommended());
    let envelope = context_envelope(None);

    apply_context_patch_locked(&mut state, &envelope, "shared/key", 1, &json!("value"))
        .expect_err("stale expected version should fail");
}
