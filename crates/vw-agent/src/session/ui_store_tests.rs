use super::*;

#[test]
fn session_scope_round_trips_and_clears() {
    set_session_scope(Some("/tmp/vw-scope"));
    assert_eq!(current_session_scope().as_deref(), Some("/tmp/vw-scope"));

    set_session_scope(None);
    assert_eq!(current_session_scope(), None);
}
