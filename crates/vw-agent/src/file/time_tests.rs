use super::{Error, get, read, with_lock_sync};

#[test]
fn read_records_file_time_by_session() {
    read("session-a", "/tmp/example.txt");

    assert!(get("session-a", "/tmp/example.txt").is_some());
    assert!(get("session-b", "/tmp/example.txt").is_none());
}

#[test]
fn display_errors_include_target_path() {
    let err = Error::MustReadFirst { filepath: "/tmp/example.txt".to_string() };

    assert!(err.to_string().contains("/tmp/example.txt"));
    assert_eq!(with_lock_sync("/tmp/example.txt", || 7), 7);
}
