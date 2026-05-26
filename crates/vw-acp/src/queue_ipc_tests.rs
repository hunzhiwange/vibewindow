use super::*;
use crate::queue_lease_store::QueueOwnerRecord;
use crate::queue_messages::QueueOwnerMessage;
use std::path::PathBuf;

fn owner_record() -> QueueOwnerRecord {
    QueueOwnerRecord {
        pid: 1,
        session_id: "s1".to_string(),
        socket_path: PathBuf::from("/tmp/vw.sock"),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        heartbeat_at: "2026-01-01T00:00:00Z".to_string(),
        owner_generation: 7,
        queue_depth: 0,
    }
}

#[test]
fn next_queue_request_id_has_stable_prefix_and_unique_counter() {
    let first = next_queue_request_id();
    let second = next_queue_request_id();

    assert!(first.starts_with("queue-"));
    assert!(second.starts_with("queue-"));
    assert_ne!(first, second);
}

#[test]
fn assert_owner_generation_rejects_mismatched_generation() {
    let result = assert_owner_generation(
        &owner_record(),
        QueueOwnerMessage::Accepted { request_id: "req-1".to_string(), owner_generation: Some(8) },
    );

    let error = result.expect_err("generation mismatch");
    assert_eq!(error.detail_code(), Some("QUEUE_OWNER_GENERATION_MISMATCH"));
}

#[test]
fn parse_queue_owner_response_line_rejects_wrong_request_id() {
    let line = r#"{"type":"accepted","requestId":"other","ownerGeneration":7}"#;
    let error = parse_queue_owner_response_line(&owner_record(), "req-1", line)
        .expect_err("wrong request id");

    assert_eq!(error.detail_code(), Some("QUEUE_PROTOCOL_MALFORMED_MESSAGE"));
}
