use super::*;
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::tools::ToolResult;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Testcov0090Tool {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Tool for Testcov0090Tool {
    fn name(&self) -> &str {
        "testcov_0090_echo"
    }

    fn description(&self) -> &str {
        "echoes test input"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "value": { "type": "string" }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ToolResult {
            success: true,
            output: format!("echo:{}", args["value"].as_str().unwrap_or("missing")),
            error: None,
        })
    }
}

fn parsed_call(
    name: &str,
    arguments: serde_json::Value,
    tool_call_id: Option<&str>,
) -> ParsedToolCall {
    ParsedToolCall {
        name: name.to_string(),
        arguments,
        tool_call_id: tool_call_id.map(str::to_string),
    }
}

fn test_context() -> Arc<ToolUseContext> {
    Arc::new(ToolUseContext::default())
}

fn test_tool(calls: &Arc<AtomicUsize>) -> Box<dyn Tool> {
    Box::new(Testcov0090Tool { calls: Arc::clone(calls) })
}

fn assert_tool_result_message(message: &ChatMessage, expected: &str) {
    assert_eq!(message.role, "user");
    assert!(message.content.contains(expected), "{}", message.content);
}

#[test]
fn immediate_failure_preserves_tool_identity_and_marks_failure() {
    let (tool_name, call_id, outcome) =
        immediate_failure("shell".to_string(), Some("call-1".to_string()), "blocked".to_string());

    assert_eq!(tool_name, "shell");
    assert_eq!(call_id.as_deref(), Some("call-1"));
    assert_eq!(outcome.tool_name, "shell");
    assert_eq!(outcome.error_reason.as_deref(), Some("blocked"));
    assert!(!outcome.success);
}

#[test]
fn history_tool_call_id_uses_native_fallback_only_when_needed() {
    assert_eq!(
        history_tool_call_id(Some("call-testcov-0090"), true, 2, 3).as_deref(),
        Some("call-testcov-0090")
    );
    assert_eq!(history_tool_call_id(None, true, 2, 3).as_deref(), Some("fallback_2_3"));
    assert!(history_tool_call_id(None, false, 2, 3).is_none());
}

#[tokio::test]
async fn excluded_tool_writes_blocked_result_without_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let tools = vec![test_tool(&calls)];
    let observer = NoopObserver;
    let context = test_context();
    let mut history = vec![ChatMessage::user("run blocked")];
    let tool_calls =
        vec![parsed_call("testcov_0090_echo", serde_json::json!({ "value": "blocked" }), None)];
    let mut seen = HashSet::new();

    execute_tool_calls_and_update_history(
        &mut history,
        &tool_calls,
        "assistant requested blocked tool".to_string(),
        &[],
        &tools,
        &observer,
        "test-channel",
        None,
        "test-provider",
        "test-model",
        "turn-testcov-0090",
        0,
        &context,
        None,
        &["testcov_0090_echo".to_string()],
        false,
        None,
        &mut seen,
        false,
    )
    .await
    .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(history.len(), 3);
    assert_eq!(history[1].role, "assistant");
    assert_tool_result_message(
        &history[2],
        "Tool 'testcov_0090_echo' is not available in this channel.",
    );
}

#[tokio::test]
async fn duplicate_text_tool_call_is_reported_without_second_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let tools = vec![test_tool(&calls)];
    let observer = NoopObserver;
    let context = test_context();
    let mut history = vec![ChatMessage::user("run twice")];
    let tool_calls = vec![
        parsed_call("testcov_0090_echo", serde_json::json!({ "value": "same" }), None),
        parsed_call("testcov_0090_echo", serde_json::json!({ "value": "same" }), None),
    ];
    let mut seen = HashSet::new();

    execute_tool_calls_and_update_history(
        &mut history,
        &tool_calls,
        "assistant requested duplicates".to_string(),
        &[],
        &tools,
        &observer,
        "test-channel",
        None,
        "test-provider",
        "test-model",
        "turn-testcov-0090",
        1,
        &context,
        None,
        &[],
        false,
        None,
        &mut seen,
        false,
    )
    .await
    .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(history.len(), 3);
    assert_tool_result_message(&history[2], "echo:same");
    assert_tool_result_message(
        &history[2],
        "Skipped duplicate tool call 'testcov_0090_echo' with identical arguments in this turn.",
    );
}

#[tokio::test]
async fn native_tool_ids_keep_same_arguments_distinct_and_write_tool_messages() {
    let calls = Arc::new(AtomicUsize::new(0));
    let tools = vec![test_tool(&calls)];
    let observer = NoopObserver;
    let context = test_context();
    let mut history = vec![ChatMessage::user("native calls")];
    let tool_calls = vec![
        parsed_call("testcov_0090_echo", serde_json::json!({ "value": "native" }), Some("call-a")),
        parsed_call("testcov_0090_echo", serde_json::json!({ "value": "native" }), Some("call-b")),
    ];
    let native_tool_calls = vec![
        ToolCall {
            id: "call-a".to_string(),
            name: "testcov_0090_echo".to_string(),
            arguments: r#"{"value":"native"}"#.to_string(),
        },
        ToolCall {
            id: "call-b".to_string(),
            name: "testcov_0090_echo".to_string(),
            arguments: r#"{"value":"native"}"#.to_string(),
        },
    ];
    let mut seen = HashSet::new();

    execute_tool_calls_and_update_history(
        &mut history,
        &tool_calls,
        "assistant requested native calls".to_string(),
        &native_tool_calls,
        &tools,
        &observer,
        "test-channel",
        None,
        "test-provider",
        "test-model",
        "turn-testcov-0090",
        2,
        &context,
        None,
        &[],
        false,
        None,
        &mut seen,
        true,
    )
    .await
    .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(history.len(), 4);
    assert_eq!(history[2].role, "tool");
    assert_eq!(history[3].role, "tool");
    assert!(history[2].content.contains("\"tool_call_id\":\"call-a\""));
    assert!(history[3].content.contains("\"tool_call_id\":\"call-b\""));
}

#[tokio::test]
async fn successful_tool_execution_emits_progress_and_structured_result() {
    let calls = Arc::new(AtomicUsize::new(0));
    let tools = vec![test_tool(&calls)];
    let observer = NoopObserver;
    let context = test_context();
    let mut history = vec![ChatMessage::user("run once")];
    let tool_calls = vec![parsed_call(
        "testcov_0090_echo",
        serde_json::json!({ "value": "progress" }),
        Some("call-progress"),
    )];
    let mut seen = HashSet::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);

    execute_tool_calls_and_update_history(
        &mut history,
        &tool_calls,
        "assistant requested one tool".to_string(),
        &[],
        &tools,
        &observer,
        "test-channel",
        None,
        "test-provider",
        "test-model",
        "turn-testcov-0090",
        3,
        &context,
        Some(&tx),
        &[],
        false,
        None,
        &mut seen,
        false,
    )
    .await
    .unwrap();
    drop(tx);

    let mut progress_messages = Vec::new();
    while let Some(message) = rx.recv().await {
        progress_messages.push(message);
    }

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(history[1].content, "assistant requested one tool");
    assert_tool_result_message(&history[2], "echo:progress");
    assert!(progress_messages.iter().any(|message| message.contains("⏳")));
    assert!(progress_messages.iter().any(|message| message.contains("✅")));
    assert!(progress_messages.iter().any(|message| {
        message.contains(DRAFT_WS_EVENT_SENTINEL) && message.contains("\"event\":\"tool_result\"")
    }));
}
