use vw_shared::session::ui_types as session_ui;

use super::ui_message::{
    UiAssistantMessage, UiErrorMessage, UiMessage, UiMessageBase, UiMessageId, UiMessageKind,
    UiStep, UiStepState, UiSystemMessage, UiSystemMessageLevel, UiThinkingBlock, UiThinkingTiming,
    UiTokenUsage, UiToolCall, UiToolCallState, UiToolResult, UiTurnTerminal, UiUserMessage,
};

#[test]
fn ui_message_id_constructors_preserve_namespaces_and_raw_values() {
    assert_eq!(UiMessageId::new("raw-1").as_str(), "raw-1");
    assert_eq!(UiMessageId::local("draft").as_str(), "local:draft");
    assert_eq!(UiMessageId::gateway("msg").as_str(), "gateway:msg");
}

#[test]
fn ui_message_base_builders_attach_optional_metadata() {
    let base = UiMessageBase::new(UiMessageId::new("child"))
        .with_parent_id(UiMessageId::new("parent"))
        .with_session_id("session-1")
        .with_created_ms(123);

    assert_eq!(base.id.as_str(), "child");
    assert_eq!(base.parent_id.as_ref().map(UiMessageId::as_str), Some("parent"));
    assert_eq!(base.session_id.as_deref(), Some("session-1"));
    assert_eq!(base.created_ms, Some(123));
}

#[test]
fn token_usage_and_thinking_timing_convert_from_shared_owned_and_borrowed_values() {
    let usage = session_ui::TokenUsage {
        input_tokens: 1,
        output_tokens: 2,
        cached_tokens: 3,
        reasoning_tokens: 4,
    };

    assert_eq!(
        UiTokenUsage::from(&usage),
        UiTokenUsage { input_tokens: 1, output_tokens: 2, cached_tokens: 3, reasoning_tokens: 4 }
    );
    assert_eq!(UiTokenUsage::from(usage).reasoning_tokens, 4);

    let timing = session_ui::ThinkTiming { start_ms: 10, end_ms: Some(20), last_update_ms: 30 };
    assert_eq!(
        UiThinkingTiming::from(&timing),
        UiThinkingTiming { start_ms: 10, end_ms: Some(20), last_update_ms: 30 }
    );
    assert_eq!(UiThinkingTiming::from(timing).last_update_ms, 30);
}

#[test]
fn ui_message_kind_and_base_cover_every_variant() {
    let cases = vec![
        (
            UiMessage::User(UiUserMessage {
                base: UiMessageBase::new(UiMessageId::new("user")),
                text: "hello".to_string(),
            }),
            UiMessageKind::User,
        ),
        (
            UiMessage::Assistant(UiAssistantMessage {
                base: UiMessageBase::new(UiMessageId::new("assistant")),
                text: "answer".to_string(),
                usage: UiTokenUsage::default(),
                step_count: 0,
                terminal: UiTurnTerminal::Done { finish_reason: Some("stop".to_string()) },
                model: Some("gpt".to_string()),
            }),
            UiMessageKind::Assistant,
        ),
        (
            UiMessage::ToolCall(UiToolCall {
                base: UiMessageBase::new(UiMessageId::new("call")),
                call_id: Some("call-1".to_string()),
                tool_name: "read".to_string(),
                summary: None,
                arguments: Some("{}".to_string()),
                state: UiToolCallState::Running,
            }),
            UiMessageKind::ToolCall,
        ),
        (
            UiMessage::ToolResult(UiToolResult {
                base: UiMessageBase::new(UiMessageId::new("result")),
                call_id: Some("call-1".to_string()),
                tool_name: "read".to_string(),
                content: "ok".to_string(),
                is_error: false,
            }),
            UiMessageKind::ToolResult,
        ),
        (
            UiMessage::Thinking(UiThinkingBlock {
                base: UiMessageBase::new(UiMessageId::new("thinking")),
                summary: Some("plan".to_string()),
                content: "details".to_string(),
                timing: Vec::new(),
                collapsed: false,
            }),
            UiMessageKind::Thinking,
        ),
        (
            UiMessage::Step(UiStep {
                base: UiMessageBase::new(UiMessageId::new("step")),
                step_index: 1,
                started_ms: 10,
                finished_ms: None,
                usage: UiTokenUsage::default(),
                finish_reason: None,
                model: None,
                state: UiStepState::Pending,
            }),
            UiMessageKind::Step,
        ),
        (
            UiMessage::System(UiSystemMessage {
                base: UiMessageBase::new(UiMessageId::new("system")),
                text: "note".to_string(),
                level: UiSystemMessageLevel::Warning,
            }),
            UiMessageKind::System,
        ),
        (
            UiMessage::Error(UiErrorMessage {
                base: UiMessageBase::new(UiMessageId::new("error")),
                message: "boom".to_string(),
                recoverable: true,
            }),
            UiMessageKind::Error,
        ),
    ];

    for (message, kind) in cases {
        assert_eq!(message.kind(), kind);
        assert_eq!(message.id(), &message.base().id);
    }
}

#[test]
fn default_terminal_and_execution_states_match_initial_lifecycle() {
    assert_eq!(UiTurnTerminal::default(), UiTurnTerminal::Pending);
    assert_eq!(UiToolCallState::default(), UiToolCallState::Queued);
    assert_eq!(UiStepState::default(), UiStepState::Pending);
    assert_eq!(UiSystemMessageLevel::default(), UiSystemMessageLevel::Info);
}
