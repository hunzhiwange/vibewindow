use super::*;
use std::collections::{HashSet, VecDeque};

fn channel() -> MatrixChannel {
    MatrixChannel::new_with_session_hint_and_vibewindow_dir(
        "https://matrix.example/".to_string(),
        "  token  ".to_string(),
        "  !room:matrix.example  ".to_string(),
        vec![" @Alice:matrix.example ".to_string(), "".to_string()],
        Some(" @bot:matrix.example ".to_string()),
        Some(" DEVICE ".to_string()),
        Some(PathBuf::from("/tmp/vw")),
    )
}

#[test]
fn constructors_normalize_core_fields_and_session_hints() {
    let ch = channel();

    assert_eq!(ch.homeserver, "https://matrix.example");
    assert_eq!(ch.access_token, "token");
    assert_eq!(ch.room_id, "!room:matrix.example");
    assert_eq!(ch.allowed_users, vec!["@Alice:matrix.example"]);
    assert_eq!(ch.session_owner_hint.as_deref(), Some("@bot:matrix.example"));
    assert_eq!(ch.session_device_id_hint.as_deref(), Some("DEVICE"));
    assert_eq!(ch.matrix_store_dir(), Some(PathBuf::from("/tmp/vw/state/matrix")));
}

#[test]
fn empty_session_hints_and_missing_store_dir_are_ignored() {
    let ch = MatrixChannel::new_with_session_hint(
        "https://matrix.example".to_string(),
        "token".to_string(),
        "!room:matrix.example".to_string(),
        Vec::new(),
        Some("   ".to_string()),
        Some(String::new()),
    );

    assert!(ch.session_owner_hint.is_none());
    assert!(ch.session_device_id_hint.is_none());
    assert!(ch.matrix_store_dir().is_none());
}

#[test]
fn sanitize_error_for_log_redacts_details_but_keeps_type() {
    let err = anyhow::anyhow!("secret token should not appear");

    let sanitized = MatrixChannel::sanitize_error_for_log(&err);

    assert!(sanitized.contains("details redacted"));
    assert!(!sanitized.contains("secret token"));
}

#[test]
fn otk_conflict_detection_is_case_insensitive_and_requires_both_phrases() {
    assert!(MatrixChannel::is_otk_conflict_message("ONE TIME KEY abc ALREADY EXISTS"));
    assert!(!MatrixChannel::is_otk_conflict_message("one time key upload failed"));
    assert!(!MatrixChannel::is_otk_conflict_message("already exists"));
}

#[test]
fn otk_conflict_recovery_message_includes_store_dir_when_available() {
    let message = channel().otk_conflict_recovery_message();

    assert!(message.contains("one-time key upload conflict"));
    assert!(message.contains("/tmp/vw/state/matrix"));
}

#[test]
fn encode_path_segment_percent_encodes_reserved_and_unicode_bytes() {
    assert_eq!(MatrixChannel::encode_path_segment("abc-._~XYZ"), "abc-._~XYZ");
    assert_eq!(
        MatrixChannel::encode_path_segment("#room:matrix.example/汉"),
        "%23room%3Amatrix.example%2F%E6%B1%89"
    );
}

#[test]
fn auth_header_value_uses_trimmed_token() {
    assert_eq!(channel().auth_header_value(), "Bearer token");
}

#[test]
fn is_sender_allowed_supports_wildcard_and_case_insensitive_match() {
    assert!(MatrixChannel::is_sender_allowed(&["*".to_string()], "@any:server"));
    assert!(MatrixChannel::is_sender_allowed(
        &["@Alice:matrix.example".to_string()],
        "@alice:matrix.example"
    ));
    assert!(!MatrixChannel::is_sender_allowed(
        &["@Alice:matrix.example".to_string()],
        "@bob:matrix.example"
    ));
    assert!(channel().is_user_allowed("@alice:matrix.example"));
}

#[test]
fn message_type_and_body_filters_match_supported_inputs() {
    assert!(MatrixChannel::is_supported_message_type("m.text"));
    assert!(MatrixChannel::is_supported_message_type("m.notice"));
    assert!(MatrixChannel::is_supported_message_type("m.audio"));
    assert!(!MatrixChannel::is_supported_message_type("m.image"));

    assert!(MatrixChannel::has_non_empty_body(" hello "));
    assert!(!MatrixChannel::has_non_empty_body(" \n\t "));
}

#[test]
fn should_process_message_respects_mention_only_gate() {
    assert!(MatrixChannel::should_process_message(false, false, false, false));
    assert!(MatrixChannel::should_process_message(true, true, false, false));
    assert!(MatrixChannel::should_process_message(true, false, true, false));
    assert!(MatrixChannel::should_process_message(true, false, false, true));
    assert!(!MatrixChannel::should_process_message(true, false, false, false));
}

#[test]
fn cache_event_id_detects_duplicates_and_evicts_oldest_entries() {
    let mut order = VecDeque::new();
    let mut lookup = HashSet::new();

    assert!(!MatrixChannel::cache_event_id("$event", &mut order, &mut lookup));
    assert!(MatrixChannel::cache_event_id("$event", &mut order, &mut lookup));

    for idx in 0..2050 {
        MatrixChannel::cache_event_id(&format!("${idx}"), &mut order, &mut lookup);
    }

    assert!(order.len() <= 2048);
    assert!(!lookup.contains("$event"));
}

#[test]
fn sync_filter_for_room_clamps_timeline_limit_to_at_least_one() {
    let filter = MatrixChannel::sync_filter_for_room("!room:matrix.example", 0);
    let parsed: serde_json::Value = serde_json::from_str(&filter).unwrap();

    assert_eq!(parsed["room"]["rooms"][0], "!room:matrix.example");
    assert_eq!(parsed["room"]["timeline"]["limit"], 1);
}
