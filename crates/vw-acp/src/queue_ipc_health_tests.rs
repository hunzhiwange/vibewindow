use super::*;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn queue_owner_health_serializes_with_camel_case_fields() {
    let health = QueueOwnerHealth {
        session_id: "session-1".to_string(),
        has_lease: true,
        healthy: false,
        socket_reachable: false,
        pid_alive: true,
        pid: Some(42),
        socket_path: Some(PathBuf::from("/tmp/vw.sock")),
        owner_generation: Some(7),
        queue_depth: Some(3),
    };

    let value = serde_json::to_value(health).expect("serialize health");
    assert_eq!(value["sessionId"], "session-1");
    assert_eq!(value["hasLease"], true);
    assert_eq!(value["queueDepth"], 3);
    assert_eq!(value["socketPath"], json!("/tmp/vw.sock"));
}
