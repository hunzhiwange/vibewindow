//! 会话对话模型持久化测试。
//!
//! 该文件验证 ACP 会话更新被转换为可持久化的对话 DTO 后，仍满足持久化 key
//! 策略，并保留工具结果的结构化渲染信息。

use std::collections::HashMap;

use agent_client_protocol::{
    ContentBlock, ContentChunk, SessionNotification, SessionUpdate, TextContent,
};
use serde_json::json;
use vw_api_types::tools::{StructuredPatchHunkDto, ToolResultContentDto, ToolResultDto};

use super::*;
use crate::{
    ClientOperation, ClientOperationMethod, ClientOperationStatus, SessionAgentContent,
    SessionAgentMessage, SessionMention, SessionMessage, SessionMessageImage, SessionThinking,
    SessionToolResult, SessionToolResultContent, SessionToolUse, SessionUserContent,
    SessionUserMessage, find_persisted_key_policy_violations, parse_prompt_source,
};

const T0: &str = "2026-04-21T00:00:00Z";
const T1: &str = "2026-04-21T00:00:01Z";

fn tool_call_update_notification(raw_output: serde_json::Value) -> SessionNotification {
    notification(json!({
        "sessionUpdate": "tool_call_update",
        "toolCallId": "tool-1",
        "kind": "apply_patch",
        "title": "apply_patch",
        "status": "completed",
        "rawOutput": raw_output
    }))
}

fn notification(update: serde_json::Value) -> SessionNotification {
    serde_json::from_value(json!({
        "sessionId": "session-1",
        "update": update
    }))
    .expect("valid ACP session notification")
}

fn text_from_tool_result(content: &SessionToolResultContent) -> &str {
    match content {
        SessionToolResultContent::Text(text) => text,
        other => panic!("expected text tool result, got {other:?}"),
    }
}

fn last_agent(conversation: &SessionConversation) -> &SessionAgentMessage {
    match conversation.messages.last() {
        Some(SessionMessage::Agent(agent)) => agent,
        other => panic!("expected trailing agent message, got {other:?}"),
    }
}

fn long_text(len: usize) -> String {
    "x".repeat(len)
}

#[test]
fn record_session_update_persists_tool_result_dto() {
    let mut conversation = create_session_conversation(Some(T0));
    let notification = tool_call_update_notification(json!({
        "tool_use_id": "tool-1",
        "success": true,
        "content": [
            {
                "type": "structured_patch",
                "hunks": [
                    {
                        "path": "src/main.rs",
                        "header": "@@ -1,1 +1,1 @@",
                        "lines": ["-old", "+new"]
                    }
                ]
            }
        ],
        "data": {
            "changed_files": ["src/main.rs"]
        },
        "model_result": "patched src/main.rs",
        "render_hint": {
            "kind": "structured_patch",
            "summary": "Updated 1 file",
            "metadata": {
                "outputPath": "src/main.rs"
            }
        }
    }));

    let _state = record_session_update(&mut conversation, None, &notification, Some(T1));

    let agent = last_agent(&conversation);
    let tool_result = agent.tool_results.get("tool-1").expect("tool result persisted");

    assert_eq!(tool_result.tool_use_id, "tool-1");
    assert_eq!(tool_result.tool_name, "apply_patch");
    assert!(!tool_result.is_error);
    assert!(matches!(
        &tool_result.content,
        SessionToolResultContent::Text(text) if text == "Updated 1 file"
    ));

    let stored = tool_result.result.as_ref().expect("structured dto persisted");
    assert_eq!(stored.tool_use_id.as_deref(), Some("tool-1"));
    assert_eq!(stored.tool_id.as_ref().map(|tool_id| tool_id.as_ref()), Some("apply_patch"));
    assert_eq!(
        stored.render_hint.as_ref().and_then(|hint| hint.summary.as_deref()),
        Some("Updated 1 file")
    );

    let serialized = serde_json::to_value(&conversation).expect("conversation serializes");
    let violations = find_persisted_key_policy_violations(&serialized);
    assert!(violations.is_empty(), "unexpected persisted key policy violations: {violations:?}");
}

#[test]
fn append_legacy_history_skips_blank_entries_and_updates_timestamp() {
    let mut conversation = create_session_conversation(Some(T0));
    append_legacy_history(
        &mut conversation,
        &[
            LegacyHistoryEntry {
                role: LegacyHistoryRole::User,
                timestamp: T1.to_string(),
                text_preview: "  hello  ".to_string(),
            },
            LegacyHistoryEntry {
                role: LegacyHistoryRole::Assistant,
                timestamp: "2026-04-21T00:00:02Z".to_string(),
                text_preview: "answer".to_string(),
            },
            LegacyHistoryEntry {
                role: LegacyHistoryRole::User,
                timestamp: "2026-04-21T00:00:03Z".to_string(),
                text_preview: "   ".to_string(),
            },
        ],
    );

    assert_eq!(conversation.messages.len(), 2);
    assert_eq!(conversation.updated_at, "2026-04-21T00:00:02Z");
    match &conversation.messages[0] {
        SessionMessage::User(user) => {
            assert!(matches!(&user.content[0], SessionUserContent::Text(text) if text == "hello"));
        }
        other => panic!("expected user message, got {other:?}"),
    }
    match &conversation.messages[1] {
        SessionMessage::Agent(agent) => {
            assert!(
                matches!(&agent.content[0], SessionAgentContent::Text(text) if text == "answer")
            );
            assert!(agent.tool_results.is_empty());
        }
        other => panic!("expected agent message, got {other:?}"),
    }
}

#[test]
fn clone_helpers_preserve_or_create_values() {
    let conversation = create_session_conversation(Some(T0));
    assert_eq!(clone_session_conversation(Some(&conversation)), conversation);
    assert!(clone_session_conversation(None).messages.is_empty());

    let state = SessionAcpxState {
        current_mode_id: Some("plan".to_string()),
        desired_mode_id: Some("code".to_string()),
        current_model_id: Some("model-a".to_string()),
        available_models: Some(vec!["model-a".to_string()]),
        available_commands: Some(vec!["run".to_string()]),
        config_options: None,
        session_options: None,
    };
    assert_eq!(clone_session_vwacp_state(Some(&state)), Some(state));
    assert_eq!(clone_session_vwacp_state(None), None);
}

#[test]
fn record_prompt_submission_converts_supported_content_blocks() {
    let mut conversation = create_session_conversation(Some(T0));
    let prompt = parse_prompt_source(
        r#"[
            {"type":"text","text":"hello"},
            {"type":"resource_link","uri":"file:///a.md","name":"A","title":"A"},
            {"type":"resource","resource":{"uri":"file:///b.md","text":"inline text"}},
            {"type":"resource","resource":{"uri":"file:///c.md","text":""}},
            {"type":"image","mimeType":"image/png","data":"QUJDRA=="}
        ]"#,
    )
    .expect("valid prompt");

    record_prompt_submission(&mut conversation, &prompt, Some(T1));

    assert_eq!(conversation.updated_at, T1);
    let user = match conversation.messages.last() {
        Some(SessionMessage::User(user)) => user,
        other => panic!("expected user message, got {other:?}"),
    };
    assert_eq!(user.content.len(), 5);
    assert!(matches!(&user.content[0], SessionUserContent::Text(text) if text == "hello"));
    assert_eq!(
        user.content[1],
        SessionUserContent::Mention(SessionMention {
            uri: "file:///a.md".to_string(),
            content: "A".to_string()
        })
    );
    assert!(matches!(&user.content[2], SessionUserContent::Text(text) if text == "inline text"));
    assert_eq!(
        user.content[3],
        SessionUserContent::Mention(SessionMention {
            uri: "file:///c.md".to_string(),
            content: "file:///c.md".to_string()
        })
    );
    assert_eq!(
        user.content[4],
        SessionUserContent::Image(SessionMessageImage {
            source: "QUJDRA==".to_string(),
            size: None
        })
    );
}

#[test]
fn record_text_prompt_submission_ignores_empty_prompt() {
    let mut conversation = create_session_conversation(Some(T0));

    record_text_prompt_submission(&mut conversation, "", Some(T1));

    assert!(conversation.messages.is_empty());
    assert_eq!(conversation.updated_at, T0);
}

#[test]
fn record_session_update_appends_user_agent_and_thinking_chunks() {
    let mut conversation = create_session_conversation(Some(T0));

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "user_message_chunk",
            "content": {"type":"text","text":"user"}
        })),
        Some(T1),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type":"text","text":"first"}
        })),
        Some("2026-04-21T00:00:02Z"),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type":"resource_link","uri":"file:///result.md","name":"result"}
        })),
        Some("2026-04-21T00:00:03Z"),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": {"type":"resource","resource":{"uri":"file:///thought.md","text":"file:///thought.md"}}
        })),
        Some("2026-04-21T00:00:04Z"),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": {"type":"text","text":" more"}
        })),
        Some("2026-04-21T00:00:05Z"),
    );

    assert_eq!(conversation.messages.len(), 2);
    match &conversation.messages[0] {
        SessionMessage::User(user) => {
            assert!(matches!(&user.content[0], SessionUserContent::Text(text) if text == "user"));
        }
        other => panic!("expected user message, got {other:?}"),
    }
    let agent = last_agent(&conversation);
    assert_eq!(agent.content.len(), 2);
    assert!(matches!(&agent.content[0], SessionAgentContent::Text(text) if text == "firstresult"));
    assert!(matches!(
        &agent.content[1],
        SessionAgentContent::Thinking(thinking) if thinking.text == "file:///thought.md more"
    ));
}

#[test]
fn record_session_update_ignores_blank_agent_chunks_and_unknown_updates() {
    let mut conversation = create_session_conversation(Some(T0));

    let state = record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type":"text","text":"   "}
        })),
        Some(T1),
    );
    let next_state = state.clone();

    assert_eq!(conversation.updated_at, T1);
    assert!(conversation.messages.is_empty());
    assert_eq!(next_state, state);
}

#[test]
fn record_session_update_records_usage_for_last_user_message() {
    let mut conversation = create_session_conversation(Some(T0));
    record_text_prompt_submission(&mut conversation, "question", Some(T1));
    let user_id = match &conversation.messages[0] {
        SessionMessage::User(user) => user.id.clone(),
        other => panic!("expected user message, got {other:?}"),
    };

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "usage_update",
            "used": 18,
            "size": 100,
            "_meta": {
                "usage": {
                    "inputTokens": 11,
                    "output_tokens": 7,
                    "cachedWriteTokens": 3,
                    "cachedReadTokens": 2
                }
            },
            "inputTokens": -1
        })),
        Some("2026-04-21T00:00:02Z"),
    );

    assert_eq!(conversation.cumulative_token_usage.input_tokens, Some(11));
    assert_eq!(conversation.cumulative_token_usage.output_tokens, Some(7));
    assert_eq!(conversation.cumulative_token_usage.cache_creation_input_tokens, Some(3));
    assert_eq!(conversation.cumulative_token_usage.cache_read_input_tokens, Some(2));
    assert_eq!(
        conversation.request_token_usage.get(&user_id),
        Some(&conversation.cumulative_token_usage)
    );
}

#[test]
fn record_session_update_ignores_empty_usage_payload() {
    let mut conversation = create_session_conversation(Some(T0));

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "usage_update",
            "used": 0,
            "size": 100,
            "inputTokens": -1,
            "outputTokens": null
        })),
        Some(T1),
    );

    assert_eq!(conversation.cumulative_token_usage, SessionTokenUsage::default());
    assert!(conversation.request_token_usage.is_empty());
}

#[test]
fn record_session_update_syncs_session_info_commands_modes_and_config() {
    let mut conversation = create_session_conversation(Some(T0));
    let mut state = record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "session_info_update",
            "title": "Chat",
            "updatedAt": "2026-04-21T00:00:09Z"
        })),
        None,
    );
    assert_eq!(conversation.title.as_deref(), Some("Chat"));
    assert_eq!(conversation.updated_at, "2026-04-21T00:00:09Z");

    state = record_session_update(
        &mut conversation,
        Some(&state),
        &notification(json!({
            "sessionUpdate": "available_commands_update",
            "availableCommands": [
                {"name":" run ","description":"Run command"},
                {"name":" ","description":"Blank command"}
            ]
        })),
        Some(T1),
    );
    assert_eq!(state.available_commands, Some(vec!["run".to_string()]));

    state = record_session_update(
        &mut conversation,
        Some(&state),
        &notification(json!({
            "sessionUpdate": "current_mode_update",
            "currentModeId": "plan"
        })),
        Some(T1),
    );
    assert_eq!(state.current_mode_id.as_deref(), Some("plan"));

    state = record_session_update(
        &mut conversation,
        Some(&state),
        &notification(json!({
            "sessionUpdate": "config_option_update",
            "configOptions": [{
                "id": "model",
                "name": "Model",
                "type": "select",
                "currentValue": "fast",
                "options": [{"value":"fast","name":"Fast"}]
            }]
        })),
        Some(T1),
    );
    assert_eq!(state.config_options.as_ref().map(Vec::len), Some(1));
}

#[test]
fn record_client_operation_updates_timestamp_and_preserves_state() {
    let mut conversation = create_session_conversation(Some(T0));
    let state = SessionAcpxState {
        current_mode_id: Some("plan".to_string()),
        desired_mode_id: None,
        current_model_id: None,
        available_models: None,
        available_commands: None,
        config_options: None,
        session_options: None,
    };
    let operation = ClientOperation {
        method: ClientOperationMethod::TerminalWaitForExit,
        status: ClientOperationStatus::Completed,
        summary: "done".to_string(),
        details: Some("exit 0".to_string()),
        timestamp: T1.to_string(),
    };

    let returned = record_client_operation(&mut conversation, Some(&state), &operation, Some(T1));

    assert_eq!(returned, state);
    assert_eq!(conversation.updated_at, T1);
}

#[test]
fn tool_call_update_creates_and_updates_tool_use_and_result() {
    let mut conversation = create_session_conversation(Some(T0));

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "tool_call",
            "toolCallId": "tool-2",
            "kind": "shell",
            "title": "shell",
            "status": "in_progress",
            "rawInput": {"command":"echo hi"}
        })),
        Some(T1),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool-2",
            "title": "Shell",
            "status": "failed",
            "rawOutput": {"stderr":"boom"}
        })),
        Some(T1),
    );

    let agent = last_agent(&conversation);
    match &agent.content[0] {
        SessionAgentContent::ToolUse(tool) => {
            assert_eq!(tool.id, "tool-2");
            assert_eq!(tool.name, "Shell");
            assert_eq!(tool.raw_input, r#"{"command":"echo hi"}"#);
            assert!(tool.is_input_complete);
        }
        other => panic!("expected tool use, got {other:?}"),
    }

    let result = agent.tool_results.get("tool-2").expect("tool result");
    assert_eq!(result.tool_name, "Shell");
    assert!(result.is_error);
    assert_eq!(text_from_tool_result(&result.content), "boom");
    assert_eq!(result.output.as_ref(), Some(&json!({"stderr":"boom"})));
}

#[test]
fn tool_call_update_preserves_existing_result_fields() {
    let mut conversation = create_session_conversation(Some(T0));

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool-3",
            "kind": "grep",
            "title": "grep",
            "status": "completed",
            "rawOutput": {"content":"found"}
        })),
        Some(T1),
    );
    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool-3",
            "status": "in_progress"
        })),
        Some(T1),
    );

    let agent = last_agent(&conversation);
    let result = agent.tool_results.get("tool-3").expect("tool result");
    assert_eq!(result.tool_name, "grep");
    assert!(!result.is_error);
    assert_eq!(text_from_tool_result(&result.content), "found");
    assert_eq!(result.output.as_ref(), Some(&json!({"content":"found"})));
}

#[test]
fn tool_call_update_handles_null_input_and_missing_id() {
    let mut conversation = create_session_conversation(Some(T0));

    let mut agent = SessionAgentMessage {
        content: Vec::new(),
        tool_results: HashMap::new(),
        reasoning_details: None,
    };
    let missing_id = json!({
        "sessionUpdate": "tool_call_update",
        "kind": "shell",
        "status": "completed",
        "rawInput": null
    });
    apply_tool_call_update(&mut agent, missing_id.as_object().expect("update object"));
    assert!(agent.content.is_empty());
    assert!(conversation.messages.is_empty());

    record_session_update(
        &mut conversation,
        None,
        &notification(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool-4",
            "status": "completed",
            "rawInput": null
        })),
        Some(T1),
    );
    let agent = last_agent(&conversation);
    match &agent.content[0] {
        SessionAgentContent::ToolUse(tool) => {
            assert_eq!(tool.name, "tool_call");
            assert_eq!(tool.input, json!({}));
            assert_eq!(tool.raw_input, "{}");
        }
        other => panic!("expected tool use, got {other:?}"),
    }
}

#[test]
fn extract_tool_result_dto_text_prefers_summary_and_content_fallbacks() {
    let with_summary: ToolResultDto = serde_json::from_value(json!({
        "content": [{"type":"text","text":"content"}],
        "render_hint": {"summary":"summary"}
    }))
    .expect("dto");
    assert_eq!(extract_tool_result_dto_text(&with_summary).as_deref(), Some("summary"));

    let with_json: ToolResultDto = serde_json::from_value(json!({
        "content": [{"type":"json","value":{"message":"json message"}}]
    }))
    .expect("dto");
    assert_eq!(extract_tool_result_dto_text(&with_json).as_deref(), Some("json message"));

    let with_patch: ToolResultDto = serde_json::from_value(json!({
        "content": [{
            "type":"structured_patch",
            "hunks":[
                {"path":"a.rs","header":"@@ -1 +1 @@","lines":["-a","+b"]},
                {"path":"   ","header":"ignored","lines":["ignored"]},
                {"path":"b.rs","header":"","lines":["+c"]}
            ]
        }]
    }))
    .expect("dto");
    assert_eq!(
        extract_tool_result_dto_text(&with_patch).as_deref(),
        Some("--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-a\n+b\n--- a/b.rs\n+++ b/b.rs\n+c\n")
    );

    let with_model_result: ToolResultDto = serde_json::from_value(json!({
        "model_result": {"output": "model"}
    }))
    .expect("dto");
    assert_eq!(extract_tool_result_dto_text(&with_model_result).as_deref(), Some("model"));

    let with_data: ToolResultDto = serde_json::from_value(json!({
        "data": {"value": "data"}
    }))
    .expect("dto");
    assert_eq!(extract_tool_result_dto_text(&with_data).as_deref(), Some("data"));
}

#[test]
fn extract_tool_result_helpers_cover_scalar_array_and_empty_values() {
    assert_eq!(extract_tool_result_text(&serde_json::Value::Null), None);
    assert_eq!(extract_tool_result_text(&json!("text")).as_deref(), Some("text"));
    assert_eq!(extract_tool_result_text(&json!(42)).as_deref(), Some("42"));
    assert_eq!(extract_tool_result_text(&json!(true)).as_deref(), Some("true"));
    assert_eq!(extract_tool_result_text(&json!([null, {"stdout":"out"}])).as_deref(), Some("out"));
    assert_eq!(extract_tool_result_text(&json!({"unknown":"value"})), None);
    assert_eq!(structured_patch_diff_text(&[]), None);
}

#[test]
fn parse_tool_result_dto_fills_missing_ids_and_tool_kind() {
    let parsed = parse_tool_result_dto(
        &json!({
            "success": false,
            "content": [{"type":"text","text":"failed"}]
        }),
        "tool-5",
        Some(" shell "),
    )
    .expect("dto parsed");

    assert_eq!(parsed.tool_use_id.as_deref(), Some("tool-5"));
    assert_eq!(parsed.tool_id.as_ref().map(|id| id.as_ref()), Some("shell"));
    assert_eq!(parsed.success, Some(false));
    assert!(parse_tool_result_dto(&json!("not a dto"), "tool-5", Some("shell")).is_none());
}

#[test]
fn status_helpers_classify_complete_and_error_values() {
    assert!(status_indicates_complete(Some("completed")));
    assert!(status_indicates_complete(Some("DONE")));
    assert!(status_indicates_complete(Some("success")));
    assert!(status_indicates_complete(Some("failed")));
    assert!(status_indicates_complete(Some("error")));
    assert!(status_indicates_complete(Some("cancelled")));
    assert!(!status_indicates_complete(Some("running")));
    assert!(!status_indicates_complete(None));

    assert!(status_indicates_error(Some("FAILED")));
    assert!(status_indicates_error(Some("error")));
    assert!(!status_indicates_error(Some("completed")));
    assert!(!status_indicates_error(None));
}

#[test]
fn trim_conversation_for_runtime_limits_messages_text_tool_io_and_usage_entries() {
    let mut messages = Vec::new();
    for index in 0..(MAX_RUNTIME_MESSAGES + 1) {
        messages.push(SessionMessage::User(SessionUserMessage {
            id: format!("user-{index:03}"),
            content: vec![SessionUserContent::Text(long_text(MAX_RUNTIME_AGENT_TEXT_CHARS + 1))],
        }));
    }
    messages.push(SessionMessage::Agent(SessionAgentMessage {
        content: vec![
            SessionAgentContent::Text(long_text(MAX_RUNTIME_AGENT_TEXT_CHARS + 1)),
            SessionAgentContent::Thinking(SessionThinking {
                text: long_text(MAX_RUNTIME_THINKING_CHARS + 1),
                signature: Some("sig".to_string()),
            }),
            SessionAgentContent::ToolUse(SessionToolUse {
                id: "tool".to_string(),
                name: "shell".to_string(),
                raw_input: long_text(MAX_RUNTIME_TOOL_IO_CHARS + 1),
                input: json!({}),
                is_input_complete: false,
                thought_signature: None,
            }),
            SessionAgentContent::RedactedThinking("redacted".to_string()),
        ],
        tool_results: HashMap::from([(
            "tool".to_string(),
            SessionToolResult {
                tool_use_id: "tool".to_string(),
                tool_name: "shell".to_string(),
                is_error: false,
                content: SessionToolResultContent::Text(long_text(MAX_RUNTIME_TOOL_IO_CHARS + 1)),
                output: Some(json!(long_text(MAX_RUNTIME_TOOL_IO_CHARS + 1))),
                result: None,
            },
        )]),
        reasoning_details: None,
    }));

    let request_token_usage = (0..(MAX_RUNTIME_REQUEST_TOKEN_USAGE + 1))
        .map(|index| {
            (
                format!("user-{index:03}"),
                SessionTokenUsage {
                    input_tokens: Some(index as i64),
                    output_tokens: None,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            )
        })
        .collect();
    let mut conversation = SessionConversation {
        title: None,
        messages,
        updated_at: T0.to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage,
    };

    trim_conversation_for_runtime(&mut conversation);

    assert_eq!(conversation.messages.len(), MAX_RUNTIME_MESSAGES);
    assert_eq!(conversation.request_token_usage.len(), MAX_RUNTIME_REQUEST_TOKEN_USAGE);
    assert!(!conversation.request_token_usage.contains_key("user-000"));
    match &conversation.messages[0] {
        SessionMessage::User(user) => {
            assert!(
                matches!(&user.content[0], SessionUserContent::Text(text) if text.len() == MAX_RUNTIME_AGENT_TEXT_CHARS)
            );
        }
        other => panic!("expected user message, got {other:?}"),
    }
    let agent = last_agent(&conversation);
    assert!(
        matches!(&agent.content[0], SessionAgentContent::Text(text) if text.len() == MAX_RUNTIME_AGENT_TEXT_CHARS)
    );
    assert!(
        matches!(&agent.content[1], SessionAgentContent::Thinking(thinking) if thinking.text.len() == MAX_RUNTIME_THINKING_CHARS)
    );
    assert!(
        matches!(&agent.content[2], SessionAgentContent::ToolUse(tool) if tool.raw_input.len() == MAX_RUNTIME_TOOL_IO_CHARS)
    );
    let result = agent.tool_results.get("tool").expect("tool result");
    assert_eq!(text_from_tool_result(&result.content).len(), MAX_RUNTIME_TOOL_IO_CHARS);
    assert!(
        matches!(result.output.as_ref(), Some(serde_json::Value::String(text)) if text.len() == MAX_RUNTIME_TOOL_IO_CHARS)
    );
}

#[test]
fn trim_runtime_text_handles_boundary_and_unicode() {
    assert_eq!(trim_runtime_text("abc", 3), "abc");
    assert_eq!(trim_runtime_text("abcd", 3), "...");
    assert_eq!(trim_runtime_text("你好世界", 4), "你好世界");
    assert_eq!(trim_runtime_text("你好世界啊", 4), "你...");
}

#[test]
fn content_block_conversion_rejects_unsupported_blocks() {
    let unsupported = serde_json::from_value(json!({
        "type":"audio",
        "mimeType":"audio/wav",
        "data":"AAAA"
    }))
    .expect("unknown ACP content block can deserialize");
    assert_eq!(extract_text(&unsupported), None);
    assert_eq!(content_to_user_content(&unsupported), None);
}

#[test]
fn notification_with_invalid_inner_content_is_ignored() {
    let conversation = create_session_conversation(Some(T0));
    let invalid_content = json!({"type":"text","text": 7});

    assert!(serde_json::from_value::<ContentBlock>(invalid_content).is_err());

    assert!(conversation.messages.is_empty());
    assert_eq!(conversation.updated_at, T0);
}

#[test]
fn record_session_update_handles_manual_session_update_variants() {
    let mut conversation = create_session_conversation(Some(T0));
    let notification = SessionNotification::new(
        "session-1",
        SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "hello from typed update",
        )))),
    );

    record_session_update(&mut conversation, None, &notification, Some(T1));

    let agent = last_agent(&conversation);
    assert!(matches!(
        &agent.content[0],
        SessionAgentContent::Text(text) if text == "hello from typed update"
    ));
}

#[test]
fn dto_content_text_skips_blank_and_uses_next_available_source() {
    let result = ToolResultDto {
        tool_use_id: None,
        tool_id: None,
        success: None,
        content: vec![
            ToolResultContentDto::Text { text: "   ".to_string() },
            ToolResultContentDto::Json { value: json!({"summary":"json summary"}) },
        ],
        data: serde_json::Value::Null,
        model_result: serde_json::Value::Null,
        render_hint: None,
        permission_request: None,
        context_updates: Vec::new(),
        extra_messages: Vec::new(),
        telemetry: None,
    };

    assert_eq!(extract_tool_result_dto_text(&result).as_deref(), Some("json summary"));
}

#[test]
fn structured_patch_diff_text_omits_empty_path_only_hunks() {
    let hunks = vec![StructuredPatchHunkDto {
        header: "@@ -1 +1 @@".to_string(),
        path: Some("   ".to_string()),
        old_start: None,
        old_lines: None,
        new_start: None,
        new_lines: None,
        lines: vec!["+ignored".to_string()],
    }];

    assert_eq!(structured_patch_diff_text(&hunks), None);
}
