use super::*;

#[test]
fn set_busy_then_idle_updates_state_map() {
    set("status-test-session", Info::Busy);
    assert!(matches!(get("status-test-session"), Info::Busy));
    assert!(list().contains_key("status-test-session"));

    set("status-test-session", Info::Idle);
    assert!(matches!(get("status-test-session"), Info::Idle));
    assert!(!list().contains_key("status-test-session"));
}

#[test]
fn retry_status_is_stored_listed_and_serialized() {
    let session_id = "status-retry-test-session";
    set(session_id, Info::Retry { attempt: 3, message: "rate limited".to_string(), next: 42 });

    let Info::Retry { attempt, message, next } = get(session_id) else {
        panic!("expected retry status");
    };
    assert_eq!(attempt, 3);
    assert_eq!(message, "rate limited");
    assert_eq!(next, 42);

    let listed = list();
    let json = serde_json::to_value(listed.get(session_id).expect("listed status")).unwrap();
    assert_eq!(json["type"], "retry");
    assert_eq!(json["attempt"], 3);

    set(session_id, Info::Idle);
}

#[test]
fn unknown_session_defaults_to_idle_without_inserting_state() {
    let session_id = "status-unknown-test-session";
    set(session_id, Info::Idle);

    assert!(matches!(get(session_id), Info::Idle));
    assert!(!list().contains_key(session_id));
}
