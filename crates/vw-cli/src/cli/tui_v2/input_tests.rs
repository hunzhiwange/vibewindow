//! 验证 TUI v2 输入模型的编辑行为。
//! 用例覆盖光标移动、文本修改和提交边界，保证终端输入可预测。

use super::input::{
    TuiSlashCommandKind, TuiSlashCommandOutcome, execute_slash_command, parse_slash_command,
    prompt_suggestions,
};
use super::state::{TuiAction, TuiModelCatalogEntry, TuiState, reduce_tui_state};

#[test]
fn parse_slash_command_maps_aliases_and_arguments() {
    let invocation = parse_slash_command("/quit").expect("slash command should parse");
    assert_eq!(invocation.kind, Some(TuiSlashCommandKind::Exit));
    assert_eq!(invocation.token, "quit");

    let model = parse_slash_command("/model gpt-5.4").expect("model slash command should parse");
    assert_eq!(model.kind, Some(TuiSlashCommandKind::Model));
    assert_eq!(model.argument.as_deref(), Some("gpt-5.4"));
}

#[test]
fn prompt_suggestions_cover_command_prefix_and_model_argument() {
    let mut state = TuiState::default();
    reduce_tui_state(&mut state, TuiAction::StatusModelSet(Some("gpt-5.4".to_string())));
    reduce_tui_state(&mut state, TuiAction::PromptValueSet("/cl".to_string()));

    let command_suggestions = prompt_suggestions(&state);
    assert!(command_suggestions.iter().any(|item| item.label == "/clear"));

    reduce_tui_state(
        &mut state,
        TuiAction::ModelCatalogReplaced(vec![TuiModelCatalogEntry {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            model_id: "gpt-5.4".to_string(),
            model_name: "GPT-5.4".to_string(),
        }]),
    );
    reduce_tui_state(&mut state, TuiAction::PromptValueSet("/model ".to_string()));
    let model_suggestions = prompt_suggestions(&state);
    assert_eq!(
        model_suggestions.first().map(|item| item.replacement.as_str()),
        Some("/model openai/gpt-5.4")
    );

    reduce_tui_state(&mut state, TuiAction::PromptValueSet("/model open".to_string()));
    let filtered_suggestions = prompt_suggestions(&state);
    assert_eq!(
        filtered_suggestions.first().map(|item| item.replacement.as_str()),
        Some("/model openai/gpt-5.4")
    );
}

#[test]
fn execute_slash_command_clear_opens_confirm_overlay_and_preserves_context() {
    let mut state = TuiState::default();
    state.session.session_id = Some("session_clear".to_string());
    reduce_tui_state(&mut state, TuiAction::StatusModelSet(Some("gpt-5.4".to_string())));
    reduce_tui_state(
        &mut state,
        TuiAction::MessagePushed(super::model::UiMessage::System(super::model::UiSystemMessage {
            base: super::model::UiMessageBase::new(super::model::UiMessageId::local("sys-1")),
            text: "stale".to_string(),
            level: super::model::UiSystemMessageLevel::Info,
        })),
    );

    let outcome = execute_slash_command(
        &mut state,
        &parse_slash_command("/clear").expect("clear command should parse"),
    );

    assert_eq!(outcome, TuiSlashCommandOutcome::Continue);
    assert_eq!(state.session.session_id.as_deref(), Some("session_clear"));
    assert_eq!(state.status.model_name.as_deref(), Some("gpt-5.4"));
    assert_eq!(state.messages.len(), 1);
    let Some(super::model::UiOverlay::Confirm(overlay)) = state.overlays.active() else {
        panic!("clear should open a confirm overlay");
    };
    assert_eq!(overlay.confirm_label, "清空");
    assert!(state.session.persisted_messages.is_empty());
}

#[test]
fn execute_slash_command_exit_opens_confirm_overlay() {
    let mut state = TuiState::default();

    let outcome = execute_slash_command(
        &mut state,
        &parse_slash_command("/exit").expect("exit command should parse"),
    );

    assert_eq!(outcome, TuiSlashCommandOutcome::Continue);
    let Some(super::model::UiOverlay::Confirm(overlay)) = state.overlays.active() else {
        panic!("exit should open a confirm overlay");
    };
    assert_eq!(overlay.confirm_label, "退出");
}

#[test]
fn execute_slash_command_model_and_unknown_emit_system_feedback() {
    let mut state = TuiState::default();
    reduce_tui_state(
        &mut state,
        TuiAction::ModelCatalogReplaced(vec![TuiModelCatalogEntry {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            model_id: "gpt-5.4".to_string(),
            model_name: "GPT-5.4".to_string(),
        }]),
    );

    let model_outcome = execute_slash_command(
        &mut state,
        &parse_slash_command("/model openai/gpt-5.4").expect("model command should parse"),
    );
    assert_eq!(model_outcome, TuiSlashCommandOutcome::Continue);
    assert_eq!(state.status.model_name.as_deref(), Some("openai/gpt-5.4"));
    assert_eq!(state.status.provider_name.as_deref(), Some("openai"));

    let unknown_outcome = execute_slash_command(
        &mut state,
        &parse_slash_command("/does-not-exist").expect("unknown slash command should still parse"),
    );
    assert_eq!(unknown_outcome, TuiSlashCommandOutcome::Continue);
    let super::model::UiMessage::System(message) =
        state.messages.last().expect("unknown command should emit warning")
    else {
        panic!("unknown slash command should emit system feedback");
    };
    assert!(message.text.contains("未知的斜杠命令"));
}

#[test]
fn execute_slash_command_resume_returns_restore_intent() {
    let mut state = TuiState::default();

    let latest = execute_slash_command(
        &mut state,
        &parse_slash_command("/resume").expect("resume command should parse"),
    );
    assert_eq!(latest, TuiSlashCommandOutcome::Resume { session_id: None });

    let explicit = execute_slash_command(
        &mut state,
        &parse_slash_command("/resume session_123").expect("resume command should parse"),
    );
    assert_eq!(
        explicit,
        TuiSlashCommandOutcome::Resume { session_id: Some("session_123".to_string()) }
    );
}
