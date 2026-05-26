use super::*;

#[test]
fn parse_arguments_accepts_string_object_and_missing_values() {
    assert_eq!(
        parse_arguments_value(Some(&serde_json::json!(r#"{"a":1}"#))),
        serde_json::json!({"a": 1})
    );
    assert_eq!(
        parse_arguments_value(Some(&serde_json::json!({"b": 2}))),
        serde_json::json!({"b": 2})
    );
    assert_eq!(parse_arguments_value(None), serde_json::json!({}));
}

#[test]
fn tool_signature_is_order_independent_and_name_normalized() {
    let (name_a, args_a) = tool_call_signature(" Shell ", &serde_json::json!({"b": 2, "a": 1}));
    let (name_b, args_b) = tool_call_signature("shell", &serde_json::json!({"a": 1, "b": 2}));

    assert_eq!(name_a, "shell");
    assert_eq!(name_b, "shell");
    assert_eq!(args_a, args_b);
}

#[test]
fn parses_nested_tool_calls_from_json_value() {
    let value = serde_json::json!({
        "choices": [{
            "message": {
                "tool_calls": [{
                    "id": "call-1",
                    "function": {"name": "shell", "arguments": "{\"command\":\"pwd\"}"}
                }]
            }
        }]
    });

    let calls = parse_tool_calls_from_json_value(&value);

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(calls[0].arguments["command"], "pwd");
}

#[test]
fn extracts_first_json_value_after_close_tags() {
    let input = "</tool_call> text {\"a\":1} tail";
    let stripped = strip_leading_close_tags(input);
    let (value, end) = extract_first_json_value_with_end(stripped).unwrap();

    assert_eq!(value, serde_json::json!({"a": 1}));
    assert!(end <= stripped.len());
    assert_eq!(find_json_end("{\"a\":1} tail"), Some(7));
}
