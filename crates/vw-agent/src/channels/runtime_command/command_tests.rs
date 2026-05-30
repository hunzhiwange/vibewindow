use super::*;

#[test]
fn parse_runtime_command_respects_channel_model_support() {
    assert_eq!(
        parse_runtime_command("telegram", "/model gpt-5"),
        Some(ChannelRuntimeCommand::SetModel("gpt-5".to_string()))
    );
    assert_eq!(parse_runtime_command("qq", "/model gpt-5"), None);
    assert_eq!(
        parse_runtime_command("discord", "/models"),
        Some(ChannelRuntimeCommand::ShowProviders)
    );
}

#[test]
fn runtime_token_parser_rejects_free_text_and_unsafe_chars() {
    assert!(is_runtime_token("tool:name-1"));
    assert!(!is_runtime_token("tool name"));
    assert!(!is_runtime_token("tool/name"));
    assert_eq!(
        extract_runtime_tail_token("approve tool:name-1", &["approve "]),
        Some("tool:name-1".to_string())
    );
}

#[test]
fn natural_language_approval_intents_are_narrow() {
    assert!(is_natural_language_all_tools_once_intent("approve all tools once"));
    assert!(!is_natural_language_all_tools_once_intent("approve tool_x"));
    assert!(is_approval_management_command(&ChannelRuntimeCommand::RequestAllToolsOnce));
    assert!(!is_approval_management_command(&ChannelRuntimeCommand::ShowModel));
}
