use serde_json::json;

use crate::app::agent::providers::ToolCall;

use super::{detect_tool_call_parse_issue, parse_structured_tool_calls, parse_tool_calls};

#[test]
fn parse_tool_calls_extracts_openai_json_tool_calls() {
    let payload = json!({
        "message": {
            "content": "done",
            "tool_calls": [{
                "id": "call-1",
                "type": "function",
                "function": {
                    "name": "shell",
                    "arguments": "{\"command\":\"pwd\"}"
                }
            }]
        }
    });

    let (text, calls) = parse_tool_calls(&payload.to_string());

    assert_eq!(text, "done");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments["command"], "pwd");
    assert_eq!(calls[0].tool_call_id.as_deref(), Some("call-1"));
}

#[test]
fn parse_structured_tool_calls_ignores_non_tool_payloads() {
    let calls = parse_structured_tool_calls(&[ToolCall {
        id: "call-2".to_string(),
        name: "shell".to_string(),
        arguments: "not-json".to_string(),
    }]);

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].arguments, json!({}));
}

#[test]
fn detect_tool_call_parse_issue_reports_unclosed_tag() {
    let issue = detect_tool_call_parse_issue("<tool_call>{\"name\":\"shell\"", &[]);

    assert!(issue.is_some());
}
