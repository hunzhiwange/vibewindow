use super::*;

#[test]
fn heartbeat_payload_uses_null_for_unknown_sequence() {
    assert_eq!(heartbeat_payload(-1), serde_json::json!({"op": 1, "d": null}));
    assert_eq!(heartbeat_payload(42), serde_json::json!({"op": 1, "d": 42}));
}

#[test]
fn heartbeat_payload_preserves_zero_sequence() {
    assert_eq!(heartbeat_payload(0), serde_json::json!({"op": 1, "d": 0}));
}
