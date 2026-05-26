use super::*;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn parse_queue_owner_record_rejects_zero_pid_or_generation() {
    let valid = json!({
        "pid": 1,
        "sessionId": "s1",
        "socketPath": "/tmp/vw.sock",
        "createdAt": "2026-01-01T00:00:00Z",
        "heartbeatAt": "2026-01-01T00:00:00Z",
        "ownerGeneration": 1,
        "queueDepth": 0
    })
    .to_string();
    assert_eq!(parse_queue_owner_record(&valid).expect("valid record").pid, 1);

    let zero_pid = valid.replace(r#""pid":1"#, r#""pid":0"#);
    assert_eq!(parse_queue_owner_record(&zero_pid), None);

    let zero_generation = valid.replace(r#""ownerGeneration":1"#, r#""ownerGeneration":0"#);
    assert_eq!(parse_queue_owner_record(&zero_generation), None);
}

#[test]
fn heartbeat_with_invalid_timestamp_is_stale() {
    let owner = QueueOwnerRecord {
        pid: 1,
        session_id: "s1".to_string(),
        socket_path: PathBuf::from("/tmp/vw.sock"),
        created_at: "bad".to_string(),
        heartbeat_at: "bad".to_string(),
        owner_generation: 1,
        queue_depth: 0,
    };

    assert!(is_queue_owner_heartbeat_stale(&owner));
    assert!(create_owner_generation() > 0);
}
