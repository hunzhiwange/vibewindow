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
