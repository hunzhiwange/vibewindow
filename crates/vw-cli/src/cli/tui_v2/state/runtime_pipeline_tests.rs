use vw_gateway_client::GatewayChatUsage;

use super::apply_runtime_event;
use crate::cli::tui_v2::model::{UiMessage, UiToolCallState, UiTurnTerminal};
use crate::cli::tui_v2::runtime::stream_adapter::{UiRuntimeEvent, UiRuntimeTerminalEvent};
use crate::cli::tui_v2::state::TuiState;

fn usage() -> GatewayChatUsage {
    GatewayChatUsage { input_tokens: 10, output_tokens: 20, cached_tokens: 3, reasoning_tokens: 4 }
}

#[test]
fn thinking_tags_split_delta_and_close_before_terminal() {
    let mut state = TuiState::default();

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Delta("visible <think>\nplan\n</think>\nafter".to_string()),
    );
    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::Done {
            finish_reason: Some("stop".to_string()),
            usage: None,
            message_id: Some("assistant-id".to_string()),
            parent_message_id: None,
        }),
    );

    assert!(!state.runtime.thinking_open);
    assert_eq!(
        state.status.turn_terminal,
        UiTurnTerminal::Done { finish_reason: Some("stop".to_string()) }
    );
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::Thinking(block) if block.content.contains("plan") && block.timing.last().and_then(|timing| timing.end_ms).is_some())));
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::Assistant(assistant) if assistant.text == "visible after" && assistant.base.id.as_str() == "gateway:assistant-id")));
}

#[test]
fn open_thinking_block_is_closed_by_error_terminal_and_side_message_is_added() {
    let mut state = TuiState::default();

    apply_runtime_event(&mut state, UiRuntimeEvent::Delta("<think>partial".to_string()));
    assert!(state.runtime.thinking_open);

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::Error("gateway failed".to_string())),
    );

    assert!(!state.runtime.thinking_open);
    assert_eq!(
        state.status.turn_terminal,
        UiTurnTerminal::Error { message: "gateway failed".to_string() }
    );
    assert!(state.messages.iter().any(
        |message| matches!(message, UiMessage::Error(error) if error.message == "gateway failed")
    ));
}

#[test]
fn tool_delta_creates_call_and_result_messages_for_success_and_failure() {
    let mut state = TuiState::default();

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Delta(
            "tool grep\n{\"status\":\"running\",\"title\":\"Search repo\",\"input\":{\"q\":\"needle\"}}"
                .to_string(),
        ),
    );
    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Delta(
            "tool grep\n{\"status\":\"completed\",\"output\":\"2 matches\"}".to_string(),
        ),
    );
    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Delta("tool write\n{\"status\":\"denied\"}".to_string()),
    );

    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::ToolCall(call) if call.tool_name == "grep" && call.summary.as_deref() == Some("Search repo") && call.arguments.as_deref() == Some("{\"q\":\"needle\"}") && call.state == UiToolCallState::Complete)));
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::ToolResult(result) if result.tool_name == "grep" && result.content == "2 matches" && !result.is_error)));
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::ToolResult(result) if result.tool_name == "write" && result.content == "tool write failed" && result.is_error)));
}

#[test]
fn step_and_timeout_terminal_update_state_and_usage() {
    let mut state = TuiState::default();

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::StepStart {
            step_index: 3,
            created_ms: 40,
            model: Some("model-a".to_string()),
        },
    );
    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::StepFinish {
            step_index: 3,
            finished_ms: 50,
            usage: usage(),
            finish_reason: Some("stop".to_string()),
            model: Some("model-b".to_string()),
        },
    );
    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::Terminal(UiRuntimeTerminalEvent::TimedOut {
            message: "deadline exceeded".to_string(),
            usage: Some(usage()),
            message_id: Some("assistant-final".to_string()),
            parent_message_id: Some("user-parent".to_string()),
        }),
    );

    assert_eq!(
        state.status.turn_terminal,
        UiTurnTerminal::TimedOut { message: "deadline exceeded".to_string() }
    );
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::Step(step) if step.step_index == 3 && step.finished_ms == Some(50) && step.usage.output_tokens == 20 && step.model.as_deref() == Some("model-b"))));
    assert!(state.messages.iter().any(
        |message| matches!(message, UiMessage::Error(error) if error.message == "deadline exceeded")
    ));
}

#[test]
fn metadata_and_unknown_events_update_only_expected_state() {
    let mut state = TuiState::default();
    state.session.session_id = Some("current".to_string());
    state.session.title = "Old".to_string();
    state.refresh_session_preview();

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::SessionMetadataChanged {
            session_id: Some("other".to_string()),
            title: Some("Ignored".to_string()),
        },
    );
    assert_eq!(state.session.title, "Old");

    apply_runtime_event(
        &mut state,
        UiRuntimeEvent::SessionMetadataChanged {
            session_id: Some("current".to_string()),
            title: Some("Renamed".to_string()),
        },
    );
    assert_eq!(state.session.title, "Renamed");

    apply_runtime_event(&mut state, UiRuntimeEvent::Unknown { event_type: None });
    assert!(state.messages.iter().any(|message| matches!(message, UiMessage::System(system) if system.text == "Unsupported gateway event: unknown.")));
}
