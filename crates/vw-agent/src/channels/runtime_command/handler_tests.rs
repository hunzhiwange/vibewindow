use super::command::{parse_runtime_command, ChannelRuntimeCommand};

#[test]
fn runtime_command_parser_keeps_model_switch_payload() {
    assert_eq!(
        parse_runtime_command("telegram", "/model gpt-4.1"),
        Some(ChannelRuntimeCommand::SetModel("gpt-4.1".to_string()))
    );
}
