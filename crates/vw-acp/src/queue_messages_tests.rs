use super::*;
use std::collections::HashMap;

use crate::types::{
    NonInteractivePermissionPolicy, OutputErrorCode, OutputErrorOrigin, PermissionMode,
    PermissionStats, SESSION_RECORD_SCHEMA, SessionEventLog, SessionRecord, SessionResumePolicy,
    SessionSendResult, SessionTokenUsage,
};
use agent_client_protocol::StopReason;
use serde_json::json;

fn test_record() -> SessionRecord {
    SessionRecord {
        schema: SESSION_RECORD_SCHEMA.to_string(),
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
            active_path: "/tmp/project/events.jsonl".to_string(),
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
        title: None,
        messages: Vec::new(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: None,
    }
}

fn test_session_send_result() -> SessionSendResult {
    SessionSendResult {
        stop_reason: StopReason::EndTurn,
        permission_stats: PermissionStats::default(),
        session_id: "session-1".to_string(),
        record: test_record(),
        resumed: false,
        load_error: None,
    }
}

#[test]
fn parse_queue_request_defaults_prompt_from_message() {
    let request = parse_queue_request(&json!({
        "type": "submit_prompt",
        "requestId": "req-1",
        "message": "hello",
        "permissionMode": "approve-reads",
        "waitForCompletion": true,
        "timeoutMs": 10.4
    }))
    .expect("valid request");

    match request {
        QueueRequest::SubmitPrompt { prompt, permission_mode, timeout_ms, .. } => {
            assert_eq!(permission_mode, PermissionMode::ApproveReads);
            assert_eq!(timeout_ms, Some(10));
            assert_eq!(prompt.len(), 1);
        }
        _ => panic!("unexpected request variant"),
    }
}

#[test]
fn parse_queue_request_accepts_full_submit_prompt_options() {
    let request = parse_queue_request(&json!({
        "type": "submit_prompt",
        "requestId": "req-1",
        "ownerGeneration": 7,
        "message": "display text",
        "prompt": [{"type": "text", "text": "structured text"}],
        "permissionMode": "deny-all",
        "resumePolicy": "same-session-only",
        "nonInteractivePermissions": "fail",
        "timeoutMs": 10.6,
        "suppressSdkConsoleErrors": false,
        "waitForCompletion": false
    }))
    .expect("valid request");

    match request {
        QueueRequest::SubmitPrompt {
            owner_generation,
            message,
            prompt,
            permission_mode,
            resume_policy,
            non_interactive_permissions,
            timeout_ms,
            suppress_sdk_console_errors,
            wait_for_completion,
            ..
        } => {
            assert_eq!(owner_generation, Some(7));
            assert_eq!(message, "display text");
            assert_eq!(prompt.len(), 1);
            assert_eq!(permission_mode, PermissionMode::DenyAll);
            assert_eq!(resume_policy, Some(SessionResumePolicy::SameSessionOnly));
            assert_eq!(non_interactive_permissions, Some(NonInteractivePermissionPolicy::Fail));
            assert_eq!(timeout_ms, Some(11));
            assert_eq!(suppress_sdk_console_errors, Some(false));
            assert!(!wait_for_completion);
        }
        _ => panic!("unexpected request variant"),
    }
}

#[test]
fn parse_queue_request_rejects_invalid_generation_and_empty_mode() {
    assert!(
        parse_queue_request(&json!({
            "type": "cancel_prompt",
            "requestId": "req-1",
            "ownerGeneration": 0
        }))
        .is_none()
    );

    assert!(
        parse_queue_request(&json!({
            "type": "set_mode",
            "requestId": "req-1",
            "modeId": " "
        }))
        .is_none()
    );
}

#[test]
fn parse_queue_request_handles_control_variants_and_timeouts() {
    let cancel = parse_queue_request(&json!({
        "type": "cancel_prompt",
        "requestId": "req-cancel",
        "ownerGeneration": null
    }))
    .expect("valid cancel request");
    assert!(matches!(
        cancel,
        QueueRequest::CancelPrompt { request_id, owner_generation }
            if request_id == "req-cancel" && owner_generation.is_none()
    ));

    let set_model = parse_queue_request(&json!({
        "type": "set_model",
        "requestId": "req-model",
        "modelId": "model-a",
        "timeoutMs": 0
    }))
    .expect("valid model request");
    assert!(matches!(
        set_model,
        QueueRequest::SetModel { model_id, timeout_ms, .. }
            if model_id == "model-a" && timeout_ms.is_none()
    ));

    let set_config = parse_queue_request(&json!({
        "type": "set_config_option",
        "requestId": "req-config",
        "configId": "temperature",
        "value": "0.2",
        "timeoutMs": 1.5
    }))
    .expect("valid config request");
    assert!(matches!(
        set_config,
        QueueRequest::SetConfigOption { config_id, value, timeout_ms, .. }
            if config_id == "temperature" && value == "0.2" && timeout_ms == Some(2)
    ));
}

#[test]
fn parse_queue_request_rejects_malformed_submit_and_control_inputs() {
    assert!(
        parse_queue_request(&json!({
            "type": "submit_prompt",
            "requestId": "req-1",
            "message": "hello",
            "prompt": [{"type": "unknown"}],
            "permissionMode": "approve-all",
            "waitForCompletion": true
        }))
        .is_none()
    );

    assert!(
        parse_queue_request(&json!({
            "type": "submit_prompt",
            "requestId": "req-1",
            "message": "hello",
            "permissionMode": "approve-all",
            "suppressSdkConsoleErrors": "no",
            "waitForCompletion": true
        }))
        .is_none()
    );

    assert!(
        parse_queue_request(&json!({
            "type": "set_config_option",
            "requestId": "req-1",
            "configId": "temperature",
            "value": " "
        }))
        .is_none()
    );

    assert!(
        parse_queue_request(&json!({
            "type": "set_model",
            "requestId": "req-1",
            "modelId": " "
        }))
        .is_none()
    );

    assert!(parse_queue_request(&json!({"type": "unknown", "requestId": "req-1"})).is_none());
    assert!(parse_queue_request(&json!("not an object")).is_none());
}

#[test]
fn parse_queue_owner_message_accepts_non_error_variants() {
    let accepted = parse_queue_owner_message(&json!({
        "type": "accepted",
        "requestId": "req-1",
        "ownerGeneration": 3
    }))
    .expect("valid accepted message");
    assert!(matches!(
        accepted,
        QueueOwnerMessage::Accepted { request_id, owner_generation }
            if request_id == "req-1" && owner_generation == Some(3)
    ));

    let event = parse_queue_owner_message(&json!({
        "type": "event",
        "requestId": "req-1",
        "message": {"jsonrpc": "2.0", "method": "session/update"}
    }))
    .expect("valid event message");
    assert!(matches!(event, QueueOwnerMessage::Event { .. }));

    let result = parse_queue_owner_message(&json!({
        "type": "result",
        "requestId": "req-1",
        "result": serde_json::to_value(test_session_send_result()).expect("result json")
    }))
    .expect("valid result message");
    assert!(matches!(
        result,
        QueueOwnerMessage::Result { result, .. } if result.session_id == "session-1"
    ));

    let cancel_result = parse_queue_owner_message(&json!({
        "type": "cancel_result",
        "requestId": "req-1",
        "cancelled": true
    }))
    .expect("valid cancel result message");
    assert!(matches!(cancel_result, QueueOwnerMessage::CancelResult { cancelled: true, .. }));

    let set_mode = parse_queue_owner_message(&json!({
        "type": "set_mode_result",
        "requestId": "req-1",
        "modeId": "focus"
    }))
    .expect("valid set mode result");
    assert!(matches!(
        set_mode,
        QueueOwnerMessage::SetModeResult { mode_id, .. } if mode_id == "focus"
    ));

    let set_model = parse_queue_owner_message(&json!({
        "type": "set_model_result",
        "requestId": "req-1",
        "modelId": "model-a"
    }))
    .expect("valid set model result");
    assert!(matches!(
        set_model,
        QueueOwnerMessage::SetModelResult { model_id, .. } if model_id == "model-a"
    ));

    let set_config = parse_queue_owner_message(&json!({
        "type": "set_config_option_result",
        "requestId": "req-1",
        "response": {"configOptions": []}
    }))
    .expect("valid set config result");
    assert!(matches!(
        set_config,
        QueueOwnerMessage::SetConfigOptionResult { response, .. }
            if response.config_options.is_empty()
    ));
}

#[test]
fn parse_queue_owner_error_keeps_optional_acp_payload() {
    let message = parse_queue_owner_message(&json!({
        "type": "error",
        "requestId": "req-1",
        "code": "RUNTIME",
        "origin": "queue",
        "message": "failed",
        "detailCode": " DETAIL ",
        "retryable": true,
        "acp": {"code": -1, "message": "bad", "data": {"x": 1}},
        "outputAlreadyEmitted": true
    }))
    .expect("valid error message");

    match message {
        QueueOwnerMessage::Error {
            code,
            origin,
            detail_code,
            retryable,
            acp,
            output_already_emitted,
            ..
        } => {
            assert_eq!(code, OutputErrorCode::Runtime);
            assert_eq!(origin, OutputErrorOrigin::Queue);
            assert_eq!(detail_code.as_deref(), Some("DETAIL"));
            assert_eq!(retryable, Some(true));
            assert_eq!(acp.expect("acp payload").message, "bad");
            assert_eq!(output_already_emitted, Some(true));
        }
        _ => panic!("unexpected message variant"),
    }
}

#[test]
fn parse_queue_owner_message_rejects_malformed_variants() {
    assert!(
        parse_queue_owner_message(&json!({
            "type": "event",
            "requestId": "req-1",
            "message": {"jsonrpc": "2.0"}
        }))
        .is_none()
    );

    assert!(
        parse_queue_owner_message(&json!({
            "type": "cancel_result",
            "requestId": "req-1",
            "cancelled": "yes"
        }))
        .is_none()
    );

    assert!(
        parse_queue_owner_message(&json!({
            "type": "error",
            "requestId": "req-1",
            "code": "RUNTIME",
            "origin": "queue",
            "message": "failed",
            "ownerGeneration": -1
        }))
        .is_none()
    );

    assert!(parse_queue_owner_message(&json!({"type": "unknown", "requestId": "req-1"})).is_none());
    assert!(parse_queue_owner_message(&json!([])).is_none());
}

#[test]
fn private_parsers_handle_boundary_values() {
    assert_eq!(parse_owner_generation(None), Ok(None));
    assert_eq!(parse_owner_generation(Some(&json!(null))), Ok(None));
    assert_eq!(parse_owner_generation(Some(&json!(1))), Ok(Some(1)));
    assert_eq!(parse_owner_generation(Some(&json!(0))), Err(()));
    assert_eq!(parse_owner_generation(Some(&json!("1"))), Err(()));

    assert_eq!(parse_timeout_ms(Some(&json!(1.49))), Some(1));
    assert_eq!(parse_timeout_ms(Some(&json!(1.5))), Some(2));
    assert_eq!(parse_timeout_ms(Some(&json!(null))), None);
    assert_eq!(parse_timeout_ms(Some(&json!(-1))), None);
    assert_eq!(parse_timeout_ms(Some(&json!("10"))), None);

    assert_eq!(
        parse_acp_error(Some(&json!({"code": -32000, "message": " detail "})))
            .expect("valid acp error")
            .message,
        "detail"
    );
    assert!(parse_acp_error(Some(&json!({"code": -32000, "message": " "}))).is_none());
    assert!(parse_acp_error(Some(&json!({"code": "bad", "message": "detail"}))).is_none());
}
