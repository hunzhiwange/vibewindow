use super::*;
use crate::app::agent::tools::{ToolResult, ToolSpec};
use std::collections::HashSet;

struct NamedTool {
    id: &'static str,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Tool for NamedTool {
    fn name(&self) -> &str {
        self.id
    }

    fn description(&self) -> &str {
        "test tool"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: String::new(), error: None })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.id, self.description(), self.parameters_schema())
    }
}

#[test]
fn strip_tool_call_tags_removes_closed_tool_blocks() {
    let message = "before <tool>{\"name\":\"secret\"}</tool> after";

    assert_eq!(strip_tool_call_tags(message), "before  after");
}

#[test]
fn strip_tool_call_tags_handles_aliases_unclosed_json_and_sentinel() {
    assert_eq!(strip_tool_call_tags("a <toolcall>{\"name\":\"x\"}</toolcall> b"), "a  b");
    assert_eq!(strip_tool_call_tags("a <invoke>{\"name\":\"x\"} trailing"), "a trailing");
    assert_eq!(strip_tool_call_tags("a ฦ{\"name\":\"x\"}ฦ b"), "a  b");
}

#[test]
fn strip_tool_call_tags_preserves_unparseable_unclosed_blocks() {
    let original = "before <tool>not-json";

    assert_eq!(strip_tool_call_tags(original), original);
}

#[test]
fn split_internal_progress_delta_detects_known_prefix() {
    let delta = format!("{}working", crate::app::agent::agent::loop_::DRAFT_PROGRESS_SENTINEL);
    let (is_progress, content) = split_internal_progress_delta(&delta);

    assert!(is_progress);
    assert_eq!(content, "working");
}

#[test]
fn split_internal_progress_delta_hides_ws_event_payloads_and_preserves_normal_text() {
    let ws_delta = format!(
        "{}{}payload",
        crate::app::agent::agent::loop_::DRAFT_PROGRESS_SENTINEL,
        crate::app::agent::agent::loop_::DRAFT_WS_EVENT_SENTINEL
    );

    assert_eq!(split_internal_progress_delta(&ws_delta), (true, ""));
    assert_eq!(split_internal_progress_delta("plain"), (false, "plain"));
}

#[test]
fn channel_delivery_instructions_are_known_for_chat_channels() {
    assert!(channel_delivery_instructions("telegram").is_some());
    assert!(channel_delivery_instructions("whatsapp").is_some());
    assert!(channel_delivery_instructions("unknown").is_none());
}

#[test]
fn should_expose_internal_tool_details_respects_positive_and_negative_intents() {
    assert!(should_expose_internal_tool_details("please show tool call json"));
    assert!(should_expose_internal_tool_details("把执行过程列出来"));
    assert!(!should_expose_internal_tool_details("do not show command output"));
    assert!(!should_expose_internal_tool_details("不要显示工具调用"));
    assert!(!should_expose_internal_tool_details("   "));
}

#[test]
fn extract_tool_context_summary_collects_unique_tool_names_after_start_index() {
    let history = vec![
        ChatMessage::assistant(r#"<tool_call>{"name":"ignored"}</tool_call>"#),
        ChatMessage::assistant(r#"<toolcall>{"name":"shell"}</toolcall>"#),
        ChatMessage::assistant(
            r#"{"tool_calls":[{"function":{"name":"file_read","arguments":{}}},{"name":"shell","arguments":{}}]}"#,
        ),
        ChatMessage::user(r#"<tool_result name="grep">ok</tool_result>"#),
    ];

    assert_eq!(extract_tool_context_summary(&history, 1), "[Used tools: shell, file_read, grep]");
    assert_eq!(extract_tool_context_summary(&history, history.len()), "");
}

#[test]
fn tool_json_payload_detection_requires_known_tool_and_arguments() {
    let known = HashSet::from(["shell".to_string()]);

    assert!(is_tool_call_payload(
        &serde_json::json!({"name": "shell", "arguments": {"cmd": "ls"}}),
        &known
    ));
    assert!(is_tool_call_payload(
        &serde_json::json!({"function": {"name": "shell", "parameters": {}}}),
        &known
    ));
    assert!(!is_tool_call_payload(
        &serde_json::json!({"name": "unknown", "arguments": {}}),
        &known
    ));
    assert!(!is_tool_call_payload(&serde_json::json!({"name": "shell"}), &known));
}

#[test]
fn tool_result_payload_requires_prior_tool_call_and_result_shape() {
    let object = serde_json::json!({"result": "ok", "tool_call_id": "1"})
        .as_object()
        .cloned()
        .expect("object");

    assert!(is_tool_result_payload(&object, true));
    assert!(!is_tool_result_payload(&object, false));

    let noisy =
        serde_json::json!({"result": "ok", "extra": true}).as_object().cloned().expect("object");
    assert!(!is_tool_result_payload(&noisy, true));
}

#[test]
fn sanitize_tool_json_value_strips_calls_and_keeps_message_content() {
    let known = HashSet::from(["shell".to_string()]);

    assert_eq!(
        sanitize_tool_json_value(
            &serde_json::json!({"name": "shell", "arguments": {"cmd": "ls"}}),
            &known,
            false
        ),
        Some((String::new(), true))
    );
    assert_eq!(
        sanitize_tool_json_value(
            &serde_json::json!({
                "content": "visible",
                "tool_calls": [{"function": {"name": "shell", "arguments": {}}}]
            }),
            &known,
            false
        ),
        Some(("visible".to_string(), true))
    );
    assert_eq!(
        sanitize_tool_json_value(&serde_json::json!({"result": "done"}), &known, true),
        Some((String::new(), false))
    );
}

#[test]
fn isolated_tool_json_artifacts_are_removed_without_touching_inline_json() {
    let known = HashSet::from(["shell".to_string()]);
    let message = concat!(
        "before\n",
        "{\"name\":\"shell\",\"arguments\":{\"cmd\":\"ls\"}}\n",
        "{\"result\":\"ok\"}\n",
        "inline {\"name\":\"shell\",\"arguments\":{}} stays"
    );

    let cleaned = strip_isolated_tool_json_artifacts(message, &known);

    assert_eq!(cleaned, "before\n\ninline {\"name\":\"shell\",\"arguments\":{}} stays");
}

#[test]
fn sanitize_channel_response_removes_known_tool_json_and_tags() {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(NamedTool { id: "shell" })];
    let response = concat!(
        "hello\n",
        "<tool>{\"name\":\"shell\",\"arguments\":{}}</tool>\n",
        "{\"name\":\"shell\",\"arguments\":{\"cmd\":\"pwd\"}}\n",
        "done"
    );

    assert_eq!(sanitize_channel_response(response, &tools), "hello\n\ndone");
}

#[test]
fn line_isolated_json_segment_detects_whitespace_boundaries() {
    let message = "prefix\n  {\"a\":1}  \nsuffix";
    let start = message.find('{').expect("json start");
    let end = start + "{\"a\":1}".len();

    assert!(is_line_isolated_json_segment(message, start, end));

    let inline = "prefix {\"a\":1} suffix";
    let start = inline.find('{').expect("json start");
    let end = start + "{\"a\":1}".len();
    assert!(!is_line_isolated_json_segment(inline, start, end));
}
