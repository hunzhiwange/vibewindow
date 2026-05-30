use super::command::{Info, Source, State, hints};

#[test]
fn hints_extracts_unique_template_placeholders_in_order() {
    assert_eq!(hints("run $1 then $2 then $1"), vec!["$1".to_string(), "$2".to_string()]);
}

#[test]
fn source_serializes_as_lowercase() {
    assert_eq!(serde_json::to_value(Source::Command).unwrap(), "command");
    assert_eq!(serde_json::to_value(Source::Skill).unwrap(), "skill");
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
