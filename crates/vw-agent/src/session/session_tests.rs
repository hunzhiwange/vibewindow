use super::{Error, Role, Session, create_default_title, extra, forked_title, project_id};
use serde_json::Value;

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

#[test]
fn session_supports_all_roles_and_message_cloning() {
    let mut session = Session::new("session-roles".to_string());

    session.push(Role::System, "system".to_string());
    session.push(Role::Tool, "tool".to_string());

    let cloned = session.clone();

    assert_eq!(cloned.messages[0].role, Role::System);
    assert_eq!(cloned.messages[1].role, Role::Tool);
    assert_eq!(format!("{:?}", cloned.messages[1]), "Message { role: Tool, content: \"tool\" }");
}

#[test]
fn default_titles_use_expected_prefixes() {
    let parent = create_default_title(false);
    let child = create_default_title(true);

    assert!(parent.starts_with("New session - "));
    assert!(child.starts_with("Child session - "));
    assert!(parent.ends_with('Z'));
    assert!(child.ends_with('Z'));
}

#[test]
fn forked_title_adds_or_increments_suffix() {
    assert_eq!(forked_title("Design"), "Design (fork #1)");
    assert_eq!(forked_title("Design (fork #1)"), "Design (fork #2)");
    assert_eq!(forked_title("Design (fork #001)"), "Design (fork #2)");
    assert_eq!(forked_title("Design (fork #x)"), "Design (fork #x) (fork #1)");
}

#[test]
fn extra_builds_json_map_from_static_pairs() {
    let map = extra([("id", Value::String("s".to_string())), ("n", serde_json::json!(1))]);

    assert_eq!(map.get("id").and_then(Value::as_str), Some("s"));
    assert_eq!(map.get("n").and_then(Value::as_i64), Some(1));
}

#[test]
fn project_id_reports_missing_context_when_unset() {
    if let Err(err) = project_id() {
        assert!(matches!(err, Error::NoProjectContext));
        assert_eq!(err.to_string(), "no active project context");
    }
}
