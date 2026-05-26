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
