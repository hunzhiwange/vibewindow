use super::*;
use std::path::{Path, PathBuf};

#[test]
fn safe_session_id_percent_encodes_path_separators_and_spaces() {
    assert_eq!(safe_session_id("abc-_.!~*'()"), "abc-_.!~*'()");
    assert_eq!(safe_session_id("a b/c"), "a%20b%2Fc");
    assert_eq!(safe_session_id("雪"), "%E9%9B%AA");
}

#[test]
fn session_event_paths_use_safe_session_id_under_base_dir() {
    let home = Path::new("/tmp/home");

    assert_eq!(session_base_dir(home), PathBuf::from("/tmp/home/.vibewindow/acp/sessions"));
    assert_eq!(
        session_event_active_path("a b", home),
        PathBuf::from("/tmp/home/.vibewindow/acp/sessions/a%20b.stream.ndjson")
    );
    assert_eq!(
        session_event_segment_path("a b", 2, home),
        PathBuf::from("/tmp/home/.vibewindow/acp/sessions/a%20b.stream.2.ndjson")
    );
    assert_eq!(
        session_event_lock_path("a b", home),
        PathBuf::from("/tmp/home/.vibewindow/acp/sessions/a%20b.stream.lock")
    );
}

#[test]
fn session_event_log_uses_default_rotation_limits() {
    let log = session_event_log("s1", "/tmp/home");

    assert_eq!(log.segment_count, DEFAULT_EVENT_MAX_SEGMENTS);
    assert_eq!(log.max_segments, DEFAULT_EVENT_MAX_SEGMENTS);
    assert_eq!(log.max_segment_bytes, DEFAULT_EVENT_SEGMENT_MAX_BYTES);
}
