use super::{Role, Session};

#[test]
fn new_session_starts_empty_and_push_preserves_message_order() {
    let mut session = Session::new("session-1".to_string());

    assert_eq!(session.id, "session-1");
    assert!(session.messages.is_empty());

    session.push(Role::User, "hello".to_string());
    session.push(Role::Assistant, "hi".to_string());

    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, Role::User);
    assert_eq!(session.messages[0].content, "hello");
    assert_eq!(session.messages[1].role, Role::Assistant);
}
