use super::*;
use crate::app::agent::providers::ToolCall;
use crate::app::agent::tools::ToolResult;
use async_trait::async_trait;
use serde_json::json;

fn response(text: Option<&str>, tool_calls: Vec<ToolCall>) -> ChatResponse {
    ChatResponse {
        text: text.map(str::to_string),
        tool_calls,
        usage: None,
        reasoning_content: None,
    }
}

struct TestTool;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for TestTool {
    fn name(&self) -> &str {
        "testcov_0080"
    }

    fn description(&self) -> &str {
        "test dispatcher tool"
    }

    fn parameters_schema(&self) -> Value {
        json!({"type": "object", "properties": {"path": {"type": "string"}}})
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: "ok".to_string(), error: None })
    }
}

#[test]
fn xml_parse_response_extracts_text_and_supported_tag_variants() {
    let dispatcher = XmlToolDispatcher;
    let input = concat!(
        "before\n",
        "<tool_call>{\"name\":\"shell\",\"arguments\":{\"command\":\"pwd\"}}</tool_call>\n",
        "middle\n",
        "<toolcall>{\"name\":\"file_read\",\"arguments\":{\"path\":\"Cargo.toml\"}}</toolcall>\n",
        "<tool-call>{\"name\":\"grep\",\"arguments\":{\"pattern\":\"fn\"}}</tool-call>\n",
        "<invoke>{\"name\":\"ls\"}</invoke>\n",
        "after"
    );

    let (text, calls) = dispatcher.parse_response(&response(Some(input), vec![]));

    assert_eq!(text, "before\nmiddle\nafter");
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments, json!({"command": "pwd"}));
    assert_eq!(calls[1].name, "file_read");
    assert_eq!(calls[2].name, "grep");
    assert_eq!(calls[3].name, "ls");
    assert_eq!(calls[3].arguments, json!({}));
    assert!(calls.iter().all(|call| call.tool_call_id.is_none()));
}

#[test]
fn xml_parse_response_skips_invalid_json_and_empty_names() {
    let dispatcher = XmlToolDispatcher;
    let input = concat!(
        "kept",
        "<tool_call_result>{not-json}</tool_call_result>",
        "<tool_call_result>{\"name\":\"\",\"arguments\":{\"x\":1}}</tool_call_result>",
        "<tool_call_result>{\"name\":\"ok\",\"arguments\":[1,2]}</tool_call_result>"
    );

    let (text, calls) = dispatcher.parse_response(&response(Some(input), vec![]));

    assert_eq!(text, "kept");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "ok");
    assert_eq!(calls[0].arguments, json!([1, 2]));
}

#[test]
fn xml_parse_response_keeps_unclosed_tool_call_as_text_tail() {
    let dispatcher = XmlToolDispatcher;

    let (text, calls) = dispatcher
        .parse_response(&response(Some("intro <tool_call_result>{\"name\":\"shell\"}"), vec![]));

    assert_eq!(text, "intro\nintro <tool_call_result>{\"name\":\"shell\"}");
    assert!(calls.is_empty());
}

#[test]
fn xml_format_results_marks_success_and_error_statuses() {
    let dispatcher = XmlToolDispatcher;

    let message = dispatcher.format_results(&[
        ToolExecutionResult {
            name: "shell".to_string(),
            output: "done".to_string(),
            success: true,
            tool_call_id: Some("ignored".to_string()),
        },
        ToolExecutionResult {
            name: "file_read".to_string(),
            output: "missing".to_string(),
            success: false,
            tool_call_id: None,
        },
    ]);

    match message {
        ConversationMessage::Chat(chat) => {
            assert_eq!(chat.role, "user");
            assert!(chat.content.contains("[Tool results]"));
            assert!(chat.content.contains("<tool_result name=\"shell\" status=\"ok\">"));
            assert!(chat.content.contains("<tool_result name=\"file_read\" status=\"error\">"));
        }
        other => panic!("expected chat message, got {other:?}"),
    }
}

#[test]
fn xml_prompt_instructions_and_specs_describe_tools() {
    let dispatcher = XmlToolDispatcher;
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];

    let specs = XmlToolDispatcher::tool_specs(&tools);
    let instructions = dispatcher.prompt_instructions(&tools);

    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].id, "testcov_0080");
    assert_eq!(specs[0].description, "test dispatcher tool");
    assert!(instructions.contains("## Tool Use Protocol"));
    assert!(instructions.contains("testcov_0080"));
    assert!(instructions.contains("test dispatcher tool"));
    assert!(!dispatcher.should_send_tool_specs());
}

#[test]
fn xml_to_provider_messages_converts_supported_history_variants() {
    let dispatcher = XmlToolDispatcher;
    let history = vec![
        ConversationMessage::Chat(ChatMessage::system("system")),
        ConversationMessage::AssistantToolCalls {
            text: None,
            tool_calls: vec![ToolCall {
                id: "tc-ignored".to_string(),
                name: "shell".to_string(),
                arguments: "{}".to_string(),
            }],
            reasoning_content: Some("ignored".to_string()),
        },
        ConversationMessage::ToolResults(vec![ToolResultMessage {
            tool_call_id: "tc-1".to_string(),
            content: "ok".to_string(),
        }]),
    ];

    let messages = dispatcher.to_provider_messages(&history);

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "system");
    assert_eq!(messages[0].content, "system");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "");
    assert_eq!(messages[2].role, "user");
    assert!(messages[2].content.contains("<tool_result id=\"tc-1\">"));
    assert!(messages[2].content.contains("ok"));
}

#[test]
fn native_parse_response_preserves_text_ids_and_arguments() {
    let dispatcher = NativeToolDispatcher;

    let (text, calls) = dispatcher.parse_response(&response(
        Some("answer"),
        vec![ToolCall {
            id: "tc-1".to_string(),
            name: "shell".to_string(),
            arguments: "{\"command\":\"ls\"}".to_string(),
        }],
    ));

    assert_eq!(text, "answer");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments, json!({"command": "ls"}));
    assert_eq!(calls[0].tool_call_id.as_deref(), Some("tc-1"));
}

#[test]
fn native_parse_response_defaults_empty_text_and_bad_arguments() {
    let dispatcher = NativeToolDispatcher;

    let (text, calls) = dispatcher.parse_response(&response(
        None,
        vec![ToolCall {
            id: "tc-bad".to_string(),
            name: "broken".to_string(),
            arguments: "{bad json".to_string(),
        }],
    ));

    assert_eq!(text, "");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].arguments, json!({}));
    assert_eq!(calls[0].tool_call_id.as_deref(), Some("tc-bad"));
}

#[test]
fn native_format_results_uses_unknown_for_missing_call_id() {
    let dispatcher = NativeToolDispatcher;

    let message = dispatcher.format_results(&[ToolExecutionResult {
        name: "shell".to_string(),
        output: "ok".to_string(),
        success: true,
        tool_call_id: None,
    }]);

    match message {
        ConversationMessage::ToolResults(results) => {
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].tool_call_id, "unknown");
            assert_eq!(results[0].content, "ok");
        }
        other => panic!("expected tool results, got {other:?}"),
    }
}

#[test]
fn native_to_provider_messages_serializes_tool_calls_and_tool_results() {
    let dispatcher = NativeToolDispatcher;
    let history = vec![
        ConversationMessage::Chat(ChatMessage::user("hello")),
        ConversationMessage::AssistantToolCalls {
            text: Some("need tool".to_string()),
            tool_calls: vec![ToolCall {
                id: "tc-1".to_string(),
                name: "shell".to_string(),
                arguments: "{}".to_string(),
            }],
            reasoning_content: Some("thinking".to_string()),
        },
        ConversationMessage::ToolResults(vec![ToolResultMessage {
            tool_call_id: "tc-1".to_string(),
            content: "done".to_string(),
        }]),
    ];

    let messages = dispatcher.to_provider_messages(&history);

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    let assistant_payload: Value = serde_json::from_str(&messages[1].content).unwrap();
    assert_eq!(assistant_payload["content"].as_str(), Some("need tool"));
    assert_eq!(assistant_payload["reasoning_content"].as_str(), Some("thinking"));
    assert_eq!(assistant_payload["tool_calls"][0]["id"].as_str(), Some("tc-1"));

    assert_eq!(messages[2].role, "tool");
    let tool_payload: Value = serde_json::from_str(&messages[2].content).unwrap();
    assert_eq!(tool_payload["tool_call_id"].as_str(), Some("tc-1"));
    assert_eq!(tool_payload["content"].as_str(), Some("done"));
    assert!(dispatcher.prompt_instructions(&[]).is_empty());
    assert!(dispatcher.should_send_tool_specs());
}
