//! 验证 CLI processor 的输入处理和工具调用输出行为。
//! 测试关注用户可见转录结果，防止交互协议变化破坏终端体验。

use crate::app::agent::session::processor as legacy_processor;
use crate::session::ui_types as models;
use serde_json::json;

use super::processor::{
    SessionProcessorComparableResult, SessionProcessorComparableTerminal,
    gateway_stream_request_from_processor_request,
};

fn sample_request() -> legacy_processor::Request {
    legacy_processor::Request {
        stream: 7,
        session: "session_bridge".to_string(),
        query: "continue the migration".to_string(),
        root: Some("/tmp/bridge-workspace".to_string()),
        model: Some("provider/model".to_string()),
        options: json!({
            "chat_system_prompt": "be precise",
            "temperature": 0.1,
        }),
        approval: None,
        channel_name: None,
        non_cli_approval_context: None,
        assistant_message_id: None,
        history: vec![
            models::ChatMessage {
                role: models::ChatRole::System,
                content: "system prompt".to_string(),
                think_timing: Vec::new(),
            },
            models::ChatMessage {
                role: models::ChatRole::Assistant,
                content: "previous answer".to_string(),
                think_timing: Vec::new(),
            },
        ],
        persist_app_session_artifacts: true,
    }
}

#[test]
fn gateway_stream_request_from_processor_request_preserves_history_and_query() {
    let request = sample_request();

    let gateway = gateway_stream_request_from_processor_request(&request);

    assert_eq!(gateway.session_id.as_ref().map(AsRef::as_ref), Some("session_bridge"));
    assert_eq!(gateway.model.as_deref(), Some("provider/model"));
    assert_eq!(gateway.options, Some(request.options.clone()));
    assert_eq!(gateway.messages.len(), 3);
    assert_eq!(gateway.messages[0], json!({ "role": "system", "content": "system prompt" }));
    assert_eq!(gateway.messages[1], json!({ "role": "assistant", "content": "previous answer" }));
    assert_eq!(gateway.messages[2], json!({ "role": "user", "content": "continue the migration" }));
}

#[test]
fn gateway_stream_request_from_processor_request_drops_blank_session_and_null_options() {
    let mut request = sample_request();
    request.session = "   ".to_string();
    request.options = serde_json::Value::Null;

    let gateway = gateway_stream_request_from_processor_request(&request);

    assert!(gateway.session_id.is_none());
    assert!(gateway.options.is_none());
}

#[test]
fn comparable_result_into_cli_result_keeps_done_payload() {
    let result = SessionProcessorComparableResult {
        output: "assistant output".to_string(),
        usage: models::TokenUsage {
            input_tokens: 11,
            output_tokens: 23,
            cached_tokens: 5,
            reasoning_tokens: 7,
        },
        step_finishes: 2,
        terminal: SessionProcessorComparableTerminal::Done {
            finish_reason: Some("stop".to_string()),
            message_id: Some("msg_a".to_string()),
            parent_message_id: Some("msg_u".to_string()),
        },
    };

    let cli = result.into_cli_result().expect("done terminal should stay successful");

    assert_eq!(cli.output, "assistant output");
    assert_eq!(cli.usage.input_tokens, 11);
    assert_eq!(cli.usage.output_tokens, 23);
    assert_eq!(cli.usage.cached_tokens, 5);
    assert_eq!(cli.usage.reasoning_tokens, 7);
    assert_eq!(cli.step_finishes, 2);
}

#[test]
fn comparable_result_into_cli_result_turns_timeout_into_error() {
    let result = SessionProcessorComparableResult {
        output: "partial output".to_string(),
        usage: models::TokenUsage::default(),
        step_finishes: 1,
        terminal: SessionProcessorComparableTerminal::TimedOut {
            message: "request timed out after 90s".to_string(),
            message_id: None,
            parent_message_id: None,
        },
    };

    let err = result.into_cli_result().expect_err("timeout terminal should become error");

    assert_eq!(err.to_string(), "request timed out after 90s");
}
