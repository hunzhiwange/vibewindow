use super::*;
use std::io;
use std::io::ErrorKind;

#[test]
fn should_retry_queue_connect_only_retries_transient_socket_errors() {
    assert!(should_retry_queue_connect(&io::Error::new(ErrorKind::NotFound, "missing")));
    assert!(should_retry_queue_connect(&io::Error::new(ErrorKind::ConnectionRefused, "refused")));
    assert!(!should_retry_queue_connect(&io::Error::new(ErrorKind::PermissionDenied, "denied")));
}

#[test]
fn to_queue_connection_error_marks_retryable_from_source_kind() {
    let retryable = to_queue_connection_error(io::Error::new(ErrorKind::NotFound, "missing"));
    assert_eq!(retryable.retryable(), Some(true));

    let terminal = to_queue_connection_error(io::Error::new(ErrorKind::PermissionDenied, "denied"));
    assert_eq!(terminal.retryable(), Some(false));
}
