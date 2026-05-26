use super::*;
use crate::queue_messages::{QueueOwnerMessage, QueueRequest};
use crate::types::{OutputErrorCode, OutputErrorOrigin};

#[test]
fn queue_request_helpers_extract_id_and_generation() {
    let request = QueueRequest::SetMode {
        request_id: "req-1".to_string(),
        owner_generation: Some(9),
        mode_id: "plan".to_string(),
        timeout_ms: Some(100),
    };

    assert_eq!(queue_request_id(&request), "req-1");
    assert_eq!(queue_request_owner_generation(&request), Some(9));
}

#[test]
fn with_owner_generation_overwrites_message_generation() {
    let message = with_owner_generation(
        QueueOwnerMessage::CancelResult {
            request_id: "req-1".to_string(),
            owner_generation: Some(1),
            cancelled: true,
        },
        Some(2),
    );

    match message {
        QueueOwnerMessage::CancelResult { owner_generation, cancelled, .. } => {
            assert_eq!(owner_generation, Some(2));
            assert!(cancelled);
        }
        _ => panic!("unexpected message variant"),
    }
}

#[test]
fn make_queue_owner_error_sets_queue_runtime_shape() {
    let message = make_queue_owner_error(
        "req-1".to_string(),
        "closed",
        "QUEUE_OWNER_CLOSED",
        Some(true),
        Some(4),
    );

    match message {
        QueueOwnerMessage::Error {
            code, origin, detail_code, retryable, owner_generation, ..
        } => {
            assert_eq!(code, OutputErrorCode::Runtime);
            assert_eq!(origin, OutputErrorOrigin::Queue);
            assert_eq!(detail_code.as_deref(), Some("QUEUE_OWNER_CLOSED"));
            assert_eq!(retryable, Some(true));
            assert_eq!(owner_generation, Some(4));
        }
        _ => panic!("unexpected message variant"),
    }
}
