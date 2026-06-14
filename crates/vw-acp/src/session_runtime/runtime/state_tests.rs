use super::*;
use std::collections::HashMap;

use agent_client_protocol::{
    ContentBlock, ContentChunk, SessionNotification, SessionUpdate, TextContent,
};
use serde_json::{Value, json};

use crate::SessionAcpxState;
use crate::prompt_content::text_prompt;
use crate::types::{
    AcpMessageDirection, ClientOperationMethod, ClientOperationStatus, OutputErrorParams,
    SessionAgentContent, SessionMessage, SessionTokenUsage, SessionUserContent, SessionUserMessage,
};

fn acp_message(value: Value) -> AcpJsonRpcMessage {
    serde_json::from_value(value).expect("valid ACP JSON-RPC message")
}

fn acp_value(message: &AcpJsonRpcMessage) -> Value {
    serde_json::to_value(message).expect("ACP message serializes")
}

fn test_record() -> SessionRecord {
    SessionRecord {
        schema: crate::SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "session-1".to_string(),
        agent_session_id: None,
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:00:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/project/active.ndjson".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 4,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: None,
        agent_started_at: None,
        last_prompt_at: None,
        last_agent_exit_code: None,
        last_agent_exit_signal: None,
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: Some("Stored title".to_string()),
        messages: vec![SessionMessage::User(SessionUserMessage {
            id: "user-1".to_string(),
            content: vec![SessionUserContent::Text("previous".to_string())],
        })],
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: Some(SessionAcpxState {
            current_mode_id: Some("code".to_string()),
            desired_mode_id: None,
            current_model_id: None,
            available_models: None,
            available_commands: None,
            config_options: None,
            session_options: Some(SessionStateOptions {
                model: Some("stored-model".to_string()),
                allowed_tools: Some(vec!["shell".to_string()]),
                max_turns: Some(3),
            }),
        }),
    }
}

#[derive(Default)]
struct CapturingFormatter {
    messages: Vec<AcpJsonRpcMessage>,
    flush_count: usize,
}

impl OutputFormatter for CapturingFormatter {
    fn set_context(&mut self, _context: OutputFormatterContext) {}

    fn on_acp_message(&mut self, message: AcpJsonRpcMessage) {
        self.messages.push(message);
    }

    fn on_error(&mut self, _params: OutputErrorParams) {}

    fn flush(&mut self) {
        self.flush_count += 1;
    }
}

#[test]
fn discard_output_formatter_accepts_empty_lifecycle() {
    let mut formatter = DiscardOutputFormatter;

    formatter.set_context(OutputFormatterContext { session_id: "session-1".to_string() });
    formatter.flush();
}

#[test]
fn runtime_state_new_records_prompt_and_apply_preserves_vwacp() {
    let record = test_record();
    let state = RuntimeState::new(&record, &text_prompt("new prompt"));
    let mut next_record = record.clone();

    state.apply_to_record(&mut next_record);

    assert_eq!(next_record.title.as_deref(), Some("Stored title"));
    assert_eq!(next_record.vwacp, record.vwacp);
    assert_eq!(next_record.messages.len(), 2);
    assert!(matches!(
        next_record.messages.last(),
        Some(SessionMessage::User(user))
            if matches!(user.content.first(), Some(SessionUserContent::Text(text)) if text == "new prompt")
    ));
}

#[test]
fn runtime_state_take_methods_drain_buffers() {
    let state = Arc::new(RuntimeState::new(&test_record(), &text_prompt("prompt")));
    let (on_message, on_output, _, _) =
        build_runtime_state_callbacks(state.clone(), None, None, None);
    let message_value = json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}});
    let output_value =
        json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s1"}});

    on_message.expect("message callback")(
        AcpMessageDirection::Inbound,
        acp_message(message_value.clone()),
    );
    on_output.as_ref().expect("output callback")(
        AcpMessageDirection::Outbound,
        acp_message(output_value.clone()),
    );
    state.set_buffering_connect_output(false);
    on_output.expect("output callback")(
        AcpMessageDirection::Outbound,
        acp_message(output_value.clone()),
    );

    assert!(state.saw_acp_message());
    assert_eq!(
        state.take_pending_messages().iter().map(acp_value).collect::<Vec<_>>(),
        vec![message_value]
    );
    assert!(state.take_pending_messages().is_empty());
    assert_eq!(
        state.take_connect_output_messages().iter().map(acp_value).collect::<Vec<_>>(),
        vec![output_value.clone()]
    );
    assert!(state.take_connect_output_messages().is_empty());
    assert_eq!(
        state.take_prompt_output_messages().iter().map(acp_value).collect::<Vec<_>>(),
        vec![output_value]
    );
    assert!(state.take_prompt_output_messages().is_empty());
}

#[test]
fn runtime_state_callbacks_forward_events_and_track_side_effects() {
    let state = Arc::new(RuntimeState::new(&test_record(), &text_prompt("prompt")));
    let seen_messages = Arc::new(Mutex::new(Vec::new()));
    let seen_updates = Arc::new(Mutex::new(Vec::new()));
    let seen_operations = Arc::new(Mutex::new(Vec::new()));
    let on_acp_message = {
        let seen_messages = seen_messages.clone();
        Arc::new(move |direction, message| seen_messages.lock().push((direction, message)))
            as AcpMessageCallback
    };
    let on_session_update = {
        let seen_updates = seen_updates.clone();
        Arc::new(move |notification| seen_updates.lock().push(notification))
            as SessionUpdateCallback
    };
    let on_client_operation = {
        let seen_operations = seen_operations.clone();
        Arc::new(move |operation| seen_operations.lock().push(operation)) as ClientOperationCallback
    };
    let (on_message, _, on_update, on_operation) = build_runtime_state_callbacks(
        state.clone(),
        Some(on_acp_message),
        Some(on_session_update),
        Some(on_client_operation),
    );
    let notification = SessionNotification::new(
        "session-1",
        SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "agent said hi",
        )))),
    );
    let operation = ClientOperation {
        method: ClientOperationMethod::TerminalWaitForExit,
        status: ClientOperationStatus::Completed,
        summary: "exit 0".to_string(),
        details: None,
        timestamp: "2026-01-02T00:00:00Z".to_string(),
    };

    on_message.expect("message callback")(
        AcpMessageDirection::Inbound,
        acp_message(json!({"jsonrpc":"2.0","id":7,"result":null})),
    );
    assert!(!state.prompt_turn_had_side_effects());
    state.mark_prompt_turn_active(true);
    on_update.expect("session update callback")(notification);
    on_operation.expect("client operation callback")(operation);

    assert!(state.prompt_turn_had_side_effects());
    assert_eq!(seen_messages.lock().len(), 1);
    assert_eq!(seen_updates.lock().len(), 1);
    assert_eq!(seen_operations.lock().len(), 1);

    let mut record = test_record();
    state.apply_to_record(&mut record);
    assert!(record.messages.iter().any(|message| {
        matches!(
            message,
            SessionMessage::Agent(agent)
                if agent.content.iter().any(|content| {
                    matches!(content, SessionAgentContent::Text(text) if text == "agent said hi")
                })
        )
    }));
}

#[test]
fn replay_output_buffer_drains_messages_and_flushes_once() {
    let messages = Mutex::new(vec![
        acp_message(json!({"jsonrpc":"2.0","id":1,"result":{"a":1}})),
        acp_message(json!({"jsonrpc":"2.0","id":2,"result":{"b":2}})),
    ]);
    let mut formatter = CapturingFormatter::default();

    replay_output_buffer(&mut formatter, &messages);

    assert_eq!(formatter.messages.len(), 2);
    assert_eq!(formatter.flush_count, 1);
    assert!(messages.lock().is_empty());
}

#[test]
fn session_options_from_record_filters_blank_values() {
    let mut record = test_record();
    record.vwacp.as_mut().expect("vwacp").session_options = Some(SessionStateOptions {
        model: Some("  model-a  ".to_string()),
        allowed_tools: Some(vec![" shell ".to_string(), "  ".to_string(), "read".to_string()]),
        max_turns: Some(5),
    });

    let options = session_options_from_record(&record).expect("valid options");

    assert_eq!(options.model.as_deref(), Some("model-a"));
    assert_eq!(options.allowed_tools, Some(vec!["shell".to_string(), "read".to_string()]));
    assert_eq!(options.max_turns, Some(5));

    record.vwacp.as_mut().expect("vwacp").session_options = Some(SessionStateOptions {
        model: Some("   ".to_string()),
        allowed_tools: Some(vec![" ".to_string()]),
        max_turns: None,
    });

    assert_eq!(session_options_from_record(&record), None);
}

#[test]
fn persist_session_options_writes_values_and_clears_empty_input() {
    let mut record = test_record();
    record.vwacp = None;
    let options = AcpSessionOptions {
        model: Some(" model-b ".to_string()),
        allowed_tools: Some(vec![" shell ".to_string(), "".to_string()]),
        max_turns: Some(2),
    };

    persist_session_options(&mut record, Some(&options));

    let stored = record
        .vwacp
        .as_ref()
        .and_then(|state| state.session_options.as_ref())
        .expect("stored session options");
    assert_eq!(stored.model.as_deref(), Some("model-b"));
    assert_eq!(stored.allowed_tools, Some(vec!["shell".to_string()]));
    assert_eq!(stored.max_turns, Some(2));

    persist_session_options(&mut record, Some(&AcpSessionOptions::default()));

    assert_eq!(record.vwacp.as_ref().and_then(|state| state.session_options.as_ref()), None);
}

#[test]
fn helper_filters_trimmed_strings_and_tool_lists() {
    assert_eq!(non_empty_trimmed(Some(" value ")).as_deref(), Some("value"));
    assert_eq!(non_empty_trimmed(Some("   ")), None);
    assert_eq!(non_empty_trimmed(None), None);
    assert_eq!(
        clone_non_empty_tools(&[" shell ".to_string(), "".to_string(), " read ".to_string()]),
        Some(vec!["shell".to_string(), "read".to_string()])
    );
    assert_eq!(clone_non_empty_tools(&[" ".to_string()]), None);
}
