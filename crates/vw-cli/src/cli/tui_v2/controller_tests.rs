use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use super::controller::{
    TuiController, TuiControllerCommand, TuiControllerEvent, TuiOverlayCommand,
    TuiRenderFeedbackLike, build_prompt_submission,
};
use super::model::{
    PromptSubmission, UiConfirmOverlay, UiMessage, UiMessageBase, UiMessageId, UiOverlay,
    UiQuestionOption, UiQuestionOverlay, UiQuestionPrompt, UiSearchMatch, UiSearchOverlay,
    UiSystemMessage, UiSystemMessageLevel, UiTaskOverlay, UiTaskStepItem, UiTodoItem,
    UiTodoOverlay, UiTokenUsage, UiTurnTerminal,
};
use super::render::layout::FullscreenLayoutSlots;
use super::state::{TuiAction, TuiState, reduce_tui_state};

#[test]
fn controller_tick_and_sync_layout_are_stable() {
    let mut controller = TuiController::new(Duration::from_millis(10));
    let mut state = TuiState::default();
    assert_eq!(
        controller.handle_event(&mut state, TuiControllerEvent::Tick),
        TuiControllerCommand::Continue
    );

    controller.sync_layout(
        &mut state,
        &FakeFeedback(FullscreenLayoutSlots {
            scrollable: Rect::new(0, 3, 80, 20),
            ..FullscreenLayoutSlots::default()
        }),
    );
    assert_eq!(state.scroll.viewport_height, 18);
    assert_eq!(state.scroll.viewport_width, 78);
}

#[test]
fn build_prompt_submission_trims_text_and_attaches_context() {
    let mut state = TuiState::default();
    state.session.session_id = Some("session-1".to_string());
    state.project.workspace_root = Some(PathBuf::from("/tmp/work"));
    state.status.model_name = Some("gpt-5.4".to_string());
    state.prompt.history.entries = vec!["old".to_string()];

    assert!(build_prompt_submission(&state, "   ").is_none());
    let submission = build_prompt_submission(&state, "  hello  ").expect("non-empty submit");
    assert_eq!(submission.text, "hello");
    assert_eq!(submission.session_id.as_deref(), Some("session-1"));
    assert_eq!(submission.root.as_deref(), Some("/tmp/work"));
    assert_eq!(submission.model.as_deref(), Some("gpt-5.4"));
    assert_eq!(submission.history_len, 1);
    assert!(submission.stream_id.is_some());
}

#[test]
fn prompt_key_events_edit_accept_suggestions_execute_submit_and_escape() {
    let mut controller = TuiController::default();
    let mut state = TuiState::default();

    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Char('/')),
        TuiControllerCommand::Continue
    );
    assert_eq!(key(&mut controller, &mut state, KeyCode::Tab), TuiControllerCommand::Continue);
    assert_eq!(state.prompt.value, "/help");

    let command = key(&mut controller, &mut state, KeyCode::Enter);
    let TuiControllerCommand::ExecuteSlashCommand(invocation) = command else {
        panic!("slash command should execute");
    };
    assert_eq!(invocation.raw, "/help");

    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Char('h')),
        TuiControllerCommand::Continue
    );
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Char('i')),
        TuiControllerCommand::Continue
    );
    let command = key(&mut controller, &mut state, KeyCode::Enter);
    let TuiControllerCommand::SubmitPrompt(submission) = command else {
        panic!("plain text should submit");
    };
    assert_eq!(submission.text, "hi");

    reduce_tui_state(&mut state, TuiAction::PromptValueSet("draft".to_string()));
    assert_eq!(key(&mut controller, &mut state, KeyCode::Esc), TuiControllerCommand::Continue);
    assert_eq!(state.prompt.value, "");
    assert_eq!(key(&mut controller, &mut state, KeyCode::Esc), TuiControllerCommand::Quit);
}

#[test]
fn busy_prompt_enter_queues_and_escape_cancels() {
    let mut controller = TuiController::default();
    let mut state = TuiState::default();
    state.prompt.start_submission(PromptSubmission::new("running"));
    reduce_tui_state(&mut state, TuiAction::PromptValueSet("/help".to_string()));

    assert_eq!(key(&mut controller, &mut state, KeyCode::Enter), TuiControllerCommand::Continue);
    assert_eq!(state.prompt.value, "");
    assert_eq!(state.prompt.queued_commands[0].raw, "/help");
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Esc),
        TuiControllerCommand::CancelActiveSubmission
    );
}

#[test]
fn prompt_shortcuts_open_expected_overlay_commands_and_help_overlay() {
    let mut controller = TuiController::default();
    let mut state = TuiState::default();

    assert_eq!(
        key_mod(&mut controller, &mut state, KeyCode::Char('f'), KeyModifiers::CONTROL),
        TuiControllerCommand::Overlay(TuiOverlayCommand::OpenSearchOverlay)
    );
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::F(2)),
        TuiControllerCommand::Overlay(TuiOverlayCommand::OpenPendingQuestions)
    );
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::F(3)),
        TuiControllerCommand::Overlay(TuiOverlayCommand::OpenTodoPanel)
    );
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::F(4)),
        TuiControllerCommand::Overlay(TuiOverlayCommand::OpenTaskPanel)
    );
    assert_eq!(key(&mut controller, &mut state, KeyCode::F(1)), TuiControllerCommand::Continue);
    assert!(matches!(state.overlays.active(), Some(UiOverlay::Error(_))));
}

#[test]
fn confirm_error_search_question_todo_and_task_overlay_keys_are_handled() {
    let mut controller = TuiController::default();
    let mut state = TuiState::default();
    let confirm = UiConfirmOverlay {
        title: "Exit".to_string(),
        body: "Leave?".to_string(),
        confirm_label: "Exit".to_string(),
        cancel_label: "Stay".to_string(),
        destructive: false,
    };
    state.overlays.push(UiOverlay::Confirm(confirm.clone()));
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Enter),
        TuiControllerCommand::Overlay(TuiOverlayCommand::ConfirmAccepted(confirm))
    );
    assert_eq!(key(&mut controller, &mut state, KeyCode::Esc), TuiControllerCommand::Continue);

    state.overlays.push(UiOverlay::Error(super::model::UiErrorOverlay {
        title: "Recoverable".to_string(),
        message: "try again".to_string(),
        recoverable: true,
    }));
    assert_eq!(key(&mut controller, &mut state, KeyCode::Enter), TuiControllerCommand::Continue);

    push_system_message(&mut state, "Alpha beta");
    let message_id = state.messages[0].id().clone();
    state.overlays.push(UiOverlay::Search(UiSearchOverlay {
        query: String::new(),
        matches: vec![UiSearchMatch {
            message_id: Some(message_id.clone()),
            start: 0,
            end: 5,
            preview: "Alpha".to_string(),
        }],
        selected_index: Some(0),
        case_sensitive: false,
    }));
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Char('a')),
        TuiControllerCommand::Continue
    );
    assert_eq!(
        key_mod(&mut controller, &mut state, KeyCode::Char('s'), KeyModifiers::CONTROL),
        TuiControllerCommand::Continue
    );
    assert_eq!(key(&mut controller, &mut state, KeyCode::Enter), TuiControllerCommand::Continue);

    state.overlays.push(UiOverlay::Question(question_overlay()));
    assert_eq!(
        key(&mut controller, &mut state, KeyCode::Char('1')),
        TuiControllerCommand::Continue
    );
    assert_eq!(key(&mut controller, &mut state, KeyCode::Tab), TuiControllerCommand::Continue);
    assert!(matches!(
        key(&mut controller, &mut state, KeyCode::Enter),
        TuiControllerCommand::Overlay(TuiOverlayCommand::QuestionSubmitted(_))
    ));
    assert!(matches!(
        key_mod(&mut controller, &mut state, KeyCode::Char('r'), KeyModifiers::CONTROL),
        TuiControllerCommand::Overlay(TuiOverlayCommand::QuestionRejected(_))
    ));

    state.overlays.push(UiOverlay::Todo(UiTodoOverlay {
        session_id: Some("session".to_string()),
        items: vec![UiTodoItem {
            id: "1".to_string(),
            content: "one".to_string(),
            status: "pending".to_string(),
            priority: "medium".to_string(),
        }],
        selected_index: 0,
        dirty: false,
    }));
    assert_eq!(key(&mut controller, &mut state, KeyCode::Enter), TuiControllerCommand::Continue);
    assert!(matches!(
        key(&mut controller, &mut state, KeyCode::Char('s')),
        TuiControllerCommand::Overlay(TuiOverlayCommand::TodoSave(_))
    ));

    state.overlays.push(UiOverlay::Task(UiTaskOverlay {
        session_id: None,
        turn_terminal: UiTurnTerminal::Pending,
        pending_questions: 0,
        todo_count: 0,
        sync_error: None,
        steps: vec![UiTaskStepItem {
            message_id,
            step_index: 1,
            state: super::model::UiStepState::Complete,
            started_ms: 1,
            finished_ms: Some(2),
            model: None,
            finish_reason: None,
            usage: UiTokenUsage::default(),
        }],
        selected_index: 0,
    }));
    assert_eq!(key(&mut controller, &mut state, KeyCode::Enter), TuiControllerCommand::Continue);
}

fn key(
    controller: &mut TuiController,
    state: &mut TuiState,
    code: KeyCode,
) -> TuiControllerCommand {
    key_mod(controller, state, code, KeyModifiers::NONE)
}

fn key_mod(
    controller: &mut TuiController,
    state: &mut TuiState,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> TuiControllerCommand {
    controller.handle_event(
        state,
        TuiControllerEvent::Terminal(CrosstermEvent::Key(KeyEvent::new(code, modifiers))),
    )
}

fn push_system_message(state: &mut TuiState, text: &str) {
    reduce_tui_state(
        state,
        TuiAction::MessagePushed(UiMessage::System(UiSystemMessage {
            base: UiMessageBase::new(UiMessageId::new(format!("message-{}", state.messages.len()))),
            text: text.to_string(),
            level: UiSystemMessageLevel::Info,
        })),
    );
}

fn question_overlay() -> UiQuestionOverlay {
    UiQuestionOverlay {
        request_id: "req".to_string(),
        session_id: "session".to_string(),
        prompts: vec![
            UiQuestionPrompt {
                header: "Approval".to_string(),
                question: "Allow?".to_string(),
                options: vec![UiQuestionOption {
                    label: "Allow".to_string(),
                    description: "continue".to_string(),
                    preview: None,
                }],
                multiple: false,
                allow_custom_input: false,
            },
            UiQuestionPrompt {
                header: "Details".to_string(),
                question: "Why?".to_string(),
                options: Vec::new(),
                multiple: false,
                allow_custom_input: true,
            },
        ],
        answers: vec![Vec::new(), Vec::new()],
        tool: None,
        selected_index: 0,
    }
}

struct FakeFeedback(FullscreenLayoutSlots);

impl TuiRenderFeedbackLike for FakeFeedback {
    fn layout(&self) -> &FullscreenLayoutSlots {
        &self.0
    }
}
