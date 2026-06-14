use super::*;
use crate::queue_paths::default_home_dir;
use std::path::Path;

#[test]
fn safe_session_id_percent_encodes_path_separators_and_spaces() {
    assert_eq!(safe_session_id("abc-_.!~*'()"), "abc-_.!~*'()");
    assert_eq!(safe_session_id("a b/c"), "a%20b%2Fc");
    assert_eq!(safe_session_id("雪"), "%E9%9B%AA");
}

#[test]
fn safe_session_id_handles_empty_and_reserved_ascii() {
    assert_eq!(safe_session_id(""), "");
    assert_eq!(safe_session_id("a+b?c#d%e\n"), "a%2Bb%3Fc%23d%25e%0A");
}

#[test]
fn session_event_paths_use_safe_session_id_under_base_dir() {
    let home = Path::new("/tmp/home");
    let base = vw_config_types::paths::home_config_dir(home).join("acp").join("sessions");

    assert_eq!(session_base_dir(home), base);
    assert_eq!(session_event_active_path("a b", home), base.join("a%20b.stream.ndjson"));
    assert_eq!(session_event_segment_path("a b", 2, home), base.join("a%20b.stream.2.ndjson"));
    assert_eq!(session_event_lock_path("a b", home), base.join("a%20b.stream.lock"));
}

#[test]
fn session_event_log_uses_default_rotation_limits() {
    let log = session_event_log("s1", "/tmp/home");

    assert_eq!(
        log.active_path,
        vw_config_types::paths::home_config_dir("/tmp/home")
            .join("acp")
            .join("sessions")
            .join("s1.stream.ndjson")
            .to_string_lossy()
    );
    assert_eq!(log.segment_count, DEFAULT_EVENT_MAX_SEGMENTS);
    assert_eq!(log.max_segments, DEFAULT_EVENT_MAX_SEGMENTS);
    assert_eq!(log.max_segment_bytes, DEFAULT_EVENT_SEGMENT_MAX_BYTES);
    assert_eq!(log.last_write_at, None);
    assert_eq!(log.last_write_error, None);
}

#[test]
fn default_session_event_paths_use_default_home_dir() {
    let Some(home) = default_home_dir() else {
        assert_eq!(default_session_base_dir(), None);
        assert_eq!(default_session_event_active_path("s1"), None);
        assert_eq!(default_session_event_segment_path("s1", 1), None);
        assert_eq!(default_session_event_lock_path("s1"), None);
        assert_eq!(default_session_event_log("s1"), None);
        return;
    };

    assert_eq!(default_session_base_dir(), Some(session_base_dir(&home)));
    assert_eq!(
        default_session_event_active_path("a b"),
        Some(session_event_active_path("a b", &home))
    );
    assert_eq!(
        default_session_event_segment_path("a b", 3),
        Some(session_event_segment_path("a b", 3, &home))
    );
    assert_eq!(default_session_event_lock_path("a b"), Some(session_event_lock_path("a b", &home)));
    assert_eq!(default_session_event_log("a b"), Some(session_event_log("a b", &home)));
}
