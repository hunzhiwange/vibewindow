use super::*;

#[test]
fn parses_minimax_invoke_and_keeps_surrounding_text() {
    let response = r#"before
<minimax:toolcall>
<invoke name="shell"><parameter name="command">pwd</parameter></invoke>
</minimax:toolcall>
after"#;

    let (text, calls) = parse_minimax_invoke_calls(response).unwrap();

    assert!(text.contains("before"));
    assert!(text.contains("after"));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments["command"], "pwd");
}

#[test]
fn parses_single_quoted_attributes_json_parameters_and_body_fallbacks() {
    let response = r#"
before <minimax:tool_call>
<invoke name='memory_store'>
  <parameter name='content'>{"topic":"rust","ok":true}</parameter>
  <parameter name="empty">   </parameter>
</invoke>
<invoke name="file_write">{"path":"/tmp/a.txt","content":"hello"}</invoke>
<invoke name="custom_value">[1,2,3]</invoke>
<invoke name="custom_text">plain text</invoke>
</minimax:tool_call> after
"#;

    let (text, calls) = parse_minimax_invoke_calls(response).unwrap();

    assert!(text.contains("before"));
    assert!(text.contains("after"));
    assert!(!text.contains("minimax:tool_call"));
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].name, "memory_store");
    assert_eq!(calls[0].arguments["content"], serde_json::json!({"topic": "rust", "ok": true}));
    assert_eq!(calls[1].arguments, serde_json::json!({"path": "/tmp/a.txt", "content": "hello"}));
    assert_eq!(calls[2].arguments, serde_json::json!({"value": [1, 2, 3]}));
    assert_eq!(calls[3].arguments, serde_json::json!({"content": "plain text"}));
    assert!(calls.iter().all(|call| call.tool_call_id.is_none()));
}

#[test]
fn minimax_parser_returns_none_without_complete_invoke_calls() {
    assert!(parse_minimax_invoke_calls("plain text only").is_none());
    assert!(
        parse_minimax_invoke_calls(
            r#"<invoke name="shell"><parameter name="command">pwd</parameter>"#
        )
        .is_none()
    );
}
