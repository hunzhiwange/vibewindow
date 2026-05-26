use super::*;

#[test]
fn shell_argument_normalization_rejects_json_and_builds_curl() {
    assert_eq!(normalize_shell_command_from_raw(r#""ls -la""#), Some("ls -la".to_string()));
    assert_eq!(normalize_shell_command_from_raw(r#"{"command":"ls"}"#), None);
    assert_eq!(
        normalize_shell_command_from_raw("https://example.com"),
        Some("curl -s 'https://example.com'".to_string())
    );
}

#[test]
fn tool_argument_normalization_uses_aliases() {
    let shell = normalize_tool_arguments("bash", serde_json::json!({"cmd": "pwd"}), None);
    assert_eq!(shell["command"], "pwd");

    let grep = normalize_tool_arguments("grep", serde_json::json!({"query": "needle"}), None);
    assert_eq!(grep["pattern"], "needle");
}

#[test]
fn parses_xml_attribute_tool_calls() {
    let calls = parse_xml_attribute_tool_calls(
        r#"<invoke name="bash"><parameter name="command">pwd</parameter></invoke>"#,
    );

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments["command"], "pwd");
}

#[test]
fn default_param_matches_known_tool_families() {
    assert_eq!(default_param_for_tool("grep"), "pattern");
    assert_eq!(default_param_for_tool("codesearch"), "query");
    assert_eq!(default_param_for_tool("unknown"), "input");
}
