use super::*;
use crate::session_event_log::{session_event_active_path, session_event_segment_path};

fn temp_home(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vw-acp-session-events-{name}-{}", std::process::id()))
}

#[test]
fn parse_event_lock_payload_accepts_valid_payload_and_defaults_invalid() {
    let parsed = parse_event_lock_payload(r#"{"pid":42,"created_at":"2026-01-01T00:00:00Z"}"#);

    assert_eq!(parsed.pid, Some(42));
    assert_eq!(parsed.created_at.as_deref(), Some("2026-01-01T00:00:00Z"));

    let parsed = parse_event_lock_payload("not-json");

    assert_eq!(parsed.pid, None);
    assert_eq!(parsed.created_at, None);
}

#[test]
fn normalize_positive_i64_uses_fallback_for_missing_zero_and_negative_values() {
    assert_eq!(normalize_positive_i64(Some(9), 3), 9);
    assert_eq!(normalize_positive_i64(Some(0), 3), 3);
    assert_eq!(normalize_positive_i64(Some(-1), 3), 3);
    assert_eq!(normalize_positive_i64(None, 3), 3);
}

#[tokio::test]
async fn stat_size_and_count_existing_segments_handle_missing_and_existing_files() {
    let home = temp_home("segments");
    let session_id = "session/with space";
    let session_dir = ensure_session_dir(&home).await.expect("create session dir");
    let active_path = session_event_active_path(session_id, &home);
    let segment_path = session_event_segment_path(session_id, 1, &home);

    tokio::fs::write(&active_path, b"active").await.expect("write active");
    tokio::fs::write(&segment_path, b"segment").await.expect("write segment");

    assert!(session_dir.ends_with(".vibewindow/acp/sessions"));
    assert!(path_exists(&active_path).await);
    assert_eq!(stat_size(&active_path).await, 6);
    assert_eq!(stat_size(&home.join("missing")).await, 0);
    assert_eq!(count_existing_segments(session_id, 3, &home).await, 2);

    let _ = tokio::fs::remove_dir_all(&home).await;
}
