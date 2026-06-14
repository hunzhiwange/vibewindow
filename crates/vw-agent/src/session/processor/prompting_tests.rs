#[test]
fn prompting_tests_module_is_wired() {
    let marker = String::from("prompting_tests");
    assert_eq!(marker.as_str(), "prompting_tests");
}

use crate::app::agent::session::session::{Role, Session};

fn session_with(messages: &[(Role, &str)]) -> Session {
    let mut session = Session::new("prompting-tests".to_string());
    for (role, content) in messages {
        session.push(*role, (*content).to_string());
    }
    session
}

#[test]
fn build_prompt_preserves_history_order_and_role_labels() {
    let session = session_with(&[
        (Role::User, "hello"),
        (Role::Assistant, "hi there"),
        (Role::System, "system note"),
        (Role::Tool, "tool output"),
    ]);

    let prompt = super::build_prompt(&session, Some("test-model"), Some("/tmp/project"), None);

    let user_pos = prompt.find("user: hello\n").expect("user message should be present");
    let assistant_pos =
        prompt.find("assistant: hi there\n").expect("assistant message should be present");
    let system_pos =
        prompt.find("system: system note\n").expect("system message should be present");
    let tool_pos = prompt.find("tool: tool output\n").expect("tool message should be present");

    assert!(user_pos < assistant_pos);
    assert!(assistant_pos < system_pos);
    assert!(system_pos < tool_pos);
}

#[test]
fn build_prompt_appends_trimmed_extra_assistant_text() {
    let session = session_with(&[(Role::User, "question")]);

    let prompt = super::build_prompt(&session, None, None, Some("  partial answer  \n"));

    assert!(prompt.ends_with("assistant: partial answer\n"));
}

#[test]
fn build_prompt_ignores_blank_extra_assistant_text() {
    let session = session_with(&[(Role::User, "question")]);

    let prompt = super::build_prompt(&session, None, None, Some("   \n"));

    assert!(!prompt.ends_with("assistant: \n"));
    assert!(!prompt.contains("assistant:    "));
}

#[test]
fn build_prompt_truncates_tool_messages_on_utf8_boundary() {
    let repeated = "界".repeat(2_000);
    let session = session_with(&[(Role::Tool, repeated.as_str())]);

    let prompt = super::build_prompt(&session, None, None, None);

    assert!(prompt.contains(&format!("tool: {}\n", "界".repeat(1_024))));
    assert!(!prompt.contains(&"界".repeat(1_025)));
}

#[test]
fn build_prompt_stops_before_oversized_older_history() {
    let mut session = Session::new("prompting-tests-budget".to_string());
    session.push(Role::User, format!("old {}", "x".repeat(130 * 1024)));
    session.push(Role::Assistant, "new answer".to_string());

    let prompt = super::build_prompt(&session, None, None, None);

    assert!(prompt.contains("assistant: new answer\n"));
    assert!(!prompt.contains("old "));
}

#[test]
fn build_prompt_adds_missing_newline_for_message_chunks() {
    let session = session_with(&[(Role::User, "no trailing newline")]);

    let prompt = super::build_prompt(&session, None, None, None);

    assert!(prompt.contains("user: no trailing newline\n"));
}

#[test]
fn build_prompt_keeps_existing_message_newline_without_double_required_suffix() {
    let session = session_with(&[(Role::Assistant, "already has newline\n")]);

    let prompt = super::build_prompt(&session, None, None, None);

    assert!(prompt.contains("assistant: already has newline\n"));
    assert!(!prompt.contains("assistant: already has newline\n\nassistant:"));
}
