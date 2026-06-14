use super::command::{ExecutedEvent, Info, Source, State, r#default, event, hints};

#[test]
fn hints_extracts_unique_template_placeholders_in_order() {
    assert_eq!(hints("run $1 then $2 then $1"), vec!["$1".to_string(), "$2".to_string()]);
}

#[test]
fn hints_sorts_numeric_placeholders_and_appends_arguments_hint() {
    assert_eq!(
        hints("use $10 with $2 and $ARGUMENTS"),
        vec!["$10".to_string(), "$2".to_string(), "$ARGUMENTS".to_string()]
    );
}

#[test]
fn hints_ignores_non_numeric_dollar_words_except_arguments() {
    assert!(hints("price is $abc and $$ and $").is_empty());
    assert_eq!(hints("$ARGUMENTS_ONLY $ARGUMENTS"), vec!["$ARGUMENTS".to_string()]);
}

#[test]
fn source_serializes_as_lowercase() {
    assert_eq!(serde_json::to_value(Source::Command).unwrap(), "command");
    assert_eq!(serde_json::to_value(Source::Mcp).unwrap(), "mcp");
    assert_eq!(serde_json::to_value(Source::Skill).unwrap(), "skill");
}

#[test]
fn executed_event_uses_frontend_field_names() {
    let value = serde_json::to_value(ExecutedEvent {
        name: "init".to_string(),
        session_id: "session-1".to_string(),
        arguments: "--force".to_string(),
        message_id: "message-1".to_string(),
    })
    .unwrap();

    assert_eq!(value["sessionID"], "session-1");
    assert_eq!(value["messageID"], "message-1");
    assert!(value.get("session_id").is_none());
    assert!(value.get("message_id").is_none());
}

#[test]
fn default_command_names_and_event_type_are_stable() {
    assert_eq!(r#default::INIT, "init");
    assert_eq!(r#default::REVIEW, "review");
    assert_eq!(event::EXECUTED.r#type, "command.executed");
}

#[test]
fn state_stores_command_info_by_name() {
    let mut state = State::default();
    let info = Info {
        name: "review".to_string(),
        description: Some("Review changes".to_string()),
        agent: None,
        model: None,
        source: Some(Source::Command),
        template: "review $1".to_string(),
        subtask: None,
        hints: vec!["$1".to_string()],
    };

    state.commands.insert(info.name.clone(), info);

    assert!(state.commands.contains_key("review"));
}
