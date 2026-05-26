//! CLI 输出渲染行为的单元测试。

use std::collections::HashMap;

use serde_json::Value;

use crate::cli::output_render::{
    conversation_history_entries, write_session_details_by_format, write_session_history_by_format,
};
use crate::{
    OutputFormat, SESSION_RECORD_SCHEMA, SessionAcpxState, SessionAgentContent,
    SessionAgentMessage, SessionEventLog, SessionMessage, SessionRecord, SessionTokenUsage,
    SessionToolResult, SessionToolResultContent, SessionToolUse, SessionUserContent,
    SessionUserMessage,
};

fn sample_record() -> SessionRecord {
    let mut tool_results = HashMap::new();
    tool_results.insert(
        "tool-1".to_string(),
        SessionToolResult {
            tool_use_id: "tool-1".to_string(),
            tool_name: "shell".to_string(),
            is_error: false,
            content: SessionToolResultContent::Text("done".to_string()),
            output: None,
            result: None,
        },
    );

    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "session-1".to_string(),
        agent_session_id: Some("agent-1".to_string()),
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: Some("workspace".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:02:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/project/active.jsonl".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 4,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: Some(42),
        agent_started_at: Some("2026-01-01T00:00:00Z".to_string()),
        last_prompt_at: Some("2026-01-01T00:01:00Z".to_string()),
        last_agent_exit_code: None,
        last_agent_exit_signal: None,
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: Some("Demo".to_string()),
        messages: vec![
            SessionMessage::User(SessionUserMessage {
                id: "user-1".to_string(),
                content: vec![
                    SessionUserContent::Text("hello".to_string()),
                    SessionUserContent::Mention(crate::SessionMention {
                        uri: "file:///tmp/project/readme.md".to_string(),
                        content: "README".to_string(),
                    }),
                ],
            }),
            SessionMessage::Agent(SessionAgentMessage {
                content: vec![
                    SessionAgentContent::Text("hi".to_string()),
                    SessionAgentContent::ToolUse(SessionToolUse {
                        id: "tool-1".to_string(),
                        name: "shell".to_string(),
                        raw_input: "{\"cmd\":\"pwd\"}".to_string(),
                        input: Value::Null,
                        is_input_complete: true,
                        thought_signature: None,
                    }),
                ],
                tool_results,
                reasoning_details: None,
            }),
            SessionMessage::Resume,
        ],
        updated_at: "2026-01-01T00:03:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: Some(SessionAcpxState {
            current_mode_id: Some("focus".to_string()),
            desired_mode_id: Some("focus".to_string()),
            current_model_id: Some("model-a".to_string()),
            available_models: Some(vec!["model-a".to_string(), "model-b".to_string()]),
            available_commands: None,
            config_options: None,
            session_options: None,
        }),
    }
}

#[test]
fn conversation_history_entries_summarize_user_and_agent_messages() {
    let entries = conversation_history_entries(&sample_record());

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].role, "user");
    assert_eq!(entries[0].text_preview, "hello\nREADME");
    assert_eq!(entries[1].role, "assistant");
    assert_eq!(entries[1].text_preview, "hi\n[tool:shell]\ndone");
}

#[test]
fn write_session_details_by_format_renders_json_payload() {
    let record = sample_record();
    let mut output = Vec::new();

    write_session_details_by_format(&mut output, &record, OutputFormat::Json).unwrap();

    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["action"], "session_details");
    assert_eq!(value["session"]["vwacpRecordId"], "record-1");
    assert_eq!(value["session"]["agentCommand"], "agent");
}

#[test]
fn write_session_history_by_format_limits_text_output() {
    let record = sample_record();
    let mut output = Vec::new();

    write_session_history_by_format(&mut output, &record, OutputFormat::Text, 1).unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "[assistant] hi\n[tool:shell]\ndone\n");
}
