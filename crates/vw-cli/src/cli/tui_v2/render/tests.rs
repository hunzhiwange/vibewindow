use std::path::PathBuf;

use super::{
    TuiPill, TuiTone, build_modal_host, build_modified_files_host, build_project_context_host,
    build_prompt_host, build_status_footer, build_status_header, clip_sidebar_body_lines,
    compact_path_label, last_submission_pill, overlay_kind_label, prompt_mode_label,
    queue_command_pill, render_prompt_value_lines, sidebar_lines, step_state_label,
    step_timing_line, terminal_tone, truncate_label, truncate_start,
    visible_prompt_suggestion_window,
};
use crate::cli::session::GitWorkspaceStatus;
use crate::cli::tui_v2::model::{
    PromptMode, PromptSubmission, PromptSubmissionStatus, QueuedPromptCommand,
    QueuedPromptCommandKind, UiConfirmOverlay, UiMessage, UiMessageBase, UiMessageId, UiOverlay,
    UiStepState, UiTaskStepItem, UiTokenUsage, UiTurnTerminal, UiUserMessage,
};
use crate::cli::tui_v2::state::{
    TuiAction, TuiState, TuiStatusSummary, reduce_tui_state, select_status_summary,
    select_visible_grouped_transcript_window,
};

#[test]
fn tui_pill_new_preserves_label_and_tone() {
    let pill = TuiPill::new("Ready", TuiTone::Success);
    assert_eq!(pill.label, "Ready");
    assert_eq!(pill.tone, TuiTone::Success);
}

#[test]
fn status_header_uses_fallbacks_truncation_and_spinner_modulo() {
    let mut state = TuiState::default();
    state.session.scope = Some("workspace".to_string());
    state.project.workspace_root = Some(PathBuf::from("/tmp/example/workspace"));
    let status = TuiStatusSummary {
        session_id: Some("session-123".to_string()),
        title: "   ".to_string(),
        provider_name: None,
        model_name: Some("gpt".to_string()),
        message_count: 0,
        assistant_message_count: 0,
        step_count: 0,
        pending_questions: 0,
        todo_count: 0,
        overlay_depth: 0,
        prompt_busy: false,
        turn_terminal: UiTurnTerminal::Streaming,
        token_usage: UiTokenUsage::default(),
    };

    let header = build_status_header(&state, &status, "VW", "http://localhost:1234567890/path", 5);
    assert_eq!(header.title, "新会话");
    assert_eq!(header.terminal, "输出中");
    assert_eq!(header.terminal_tone, TuiTone::Accent);
    assert_eq!(header.provider, "-");
    assert_eq!(header.scope, "workspace");
    assert_eq!(header.spinner, "/");
    assert!(header.gateway.ends_with("..."));
}

#[test]
fn status_footer_reflects_errors_tokens_queue_questions_and_todos() {
    let mut state = TuiState::default();
    state.status.last_error = Some("network is unavailable for a long time".to_string());
    state.prompt.queue_command(QueuedPromptCommand {
        raw: "queued".to_string(),
        kind: QueuedPromptCommandKind::Submit,
        enqueued_ms: None,
    });
    reduce_tui_state(
        &mut state,
        TuiAction::MessagePushed(UiMessage::User(UiUserMessage {
            base: UiMessageBase::new(UiMessageId::new("u1")),
            text: "hello".to_string(),
        })),
    );
    state.tasks.pending_questions.push(crate::cli::tui_v2::model::UiQuestionOverlay {
        request_id: "req".to_string(),
        session_id: "session".to_string(),
        prompts: Vec::new(),
        answers: Vec::new(),
        tool: None,
        selected_index: 0,
    });
    state.tasks.todo_overlay = Some(crate::cli::tui_v2::model::UiTodoOverlay {
        session_id: None,
        items: vec![crate::cli::tui_v2::model::UiTodoItem {
            id: "1".to_string(),
            content: "todo".to_string(),
            status: "pending".to_string(),
            priority: "medium".to_string(),
        }],
        selected_index: 0,
        dirty: false,
    });

    let mut status = select_status_summary(&state);
    status.token_usage.input_tokens = 3;
    status.token_usage.output_tokens = 4;
    let window = select_visible_grouped_transcript_window(&state);
    let footer = build_status_footer(&state, &status, &window);

    assert!(footer.pills.iter().any(|pill| pill.label.starts_with("错误 ")));
    assert!(footer.pills.iter().any(|pill| pill.label == "令牌 3/4"));
    assert!(footer.pills.iter().any(|pill| pill.label == "队列 1"));
    assert!(footer.pills.iter().any(|pill| pill.label == "问题 1"));
    assert!(footer.pills.iter().any(|pill| pill.label == "待办 1"));
}

#[test]
fn prompt_host_reports_suggestions_queue_footer_and_last_submission() {
    let mut state = TuiState::default();
    reduce_tui_state(&mut state, TuiAction::StatusModelSet(Some("gpt-5.4".to_string())));
    reduce_tui_state(&mut state, TuiAction::PromptValueSet("/".to_string()));
    state.prompt.set_selected_suggestion_index(Some(2));
    for index in 0..5 {
        state.prompt.queue_command(QueuedPromptCommand {
            raw: format!("cmd-{index}"),
            kind: QueuedPromptCommandKind::Submit,
            enqueued_ms: Some(index),
        });
    }
    state.prompt.last_submission = Some(PromptSubmission {
        stream_id: None,
        session_id: None,
        text: "old".to_string(),
        root: None,
        model: None,
        history_len: 0,
        status: PromptSubmissionStatus::Error { message: "failed".to_string() },
    });

    let host = build_prompt_host(&state);
    assert_eq!(host.placeholder, "输入 / 命令…");
    assert_eq!(host.suggestion_rows.len(), 3);
    assert!(host.suggestion_rows.iter().any(|row| row.selected));
    assert_eq!(host.queued_commands.last().map(|pill| pill.label.as_str()), Some("+2 more"));
    assert!(host.footer_pills.iter().any(|pill| pill.label == "上次失败"));
}

#[test]
fn project_modified_modal_and_prompt_viewport_helpers_are_stable() {
    let mut state = TuiState::default();
    state.session.session_id = Some("session-1".to_string());
    state.session.scope = Some("project".to_string());
    state.project.info = "Demo project".to_string();
    state.project.workspace_root = Some(PathBuf::from("/tmp/workspace"));
    state.project.git_status = GitWorkspaceStatus::ReadyDirty(vec!["src/lib.rs".to_string()]);
    let status = select_status_summary(&state);

    let project = build_project_context_host(&state, &status);
    assert!(project.pills.iter().any(|pill| pill.label == "会话已绑定"));
    assert!(project.body_lines.iter().any(|line| line.contains("Demo project")));
    let files = build_modified_files_host(&state);
    assert_eq!(files.pills[0].label, "数量 1");

    state.overlays.push(UiOverlay::Confirm(UiConfirmOverlay {
        title: String::new(),
        body: "Clear everything?".to_string(),
        confirm_label: "Clear".to_string(),
        cancel_label: "Cancel".to_string(),
        destructive: true,
    }));
    let modal = build_modal_host(&state).expect("modal host");
    assert_eq!(modal.title, "确认");
    assert!(modal.body_lines.iter().any(|line| line.contains("危险操作=true")));

    let mut prompt = build_prompt_host(&TuiState::default());
    prompt.value = "first\n第二\nthird".to_string();
    prompt.cursor_char_index = prompt.value.chars().count();
    let viewport = render_prompt_value_lines(&prompt, super::TuiTheme::default(), 2);
    assert_eq!(viewport.hidden_above, 1);
    assert_eq!(viewport.cursor.y, 1);
}

#[test]
fn small_helpers_cover_labels_truncation_windows_and_sidebar_clipping() {
    assert_eq!(visible_prompt_suggestion_window(2, 1, 3), (0, 2));
    assert_eq!(visible_prompt_suggestion_window(8, 7, 3), (5, 8));
    assert_eq!(truncate_label("abcdef", 4), "a...");
    assert_eq!(truncate_start("/very/long/path", 8), ".../path");
    assert_eq!(compact_path_label(None), "-");
    assert_eq!(prompt_mode_label(&PromptMode::Search), "搜索");
    assert_eq!(step_state_label(&UiStepState::Failed), "失败");
    assert_eq!(overlay_kind_label(crate::cli::tui_v2::model::UiOverlayKind::Mcp), "MCP");
    assert_eq!(
        terminal_tone(&UiTurnTerminal::TimedOut { message: "slow".into() }),
        TuiTone::Danger
    );

    let queued = queue_command_pill(&QueuedPromptCommand {
        raw: "/really-long-command-name".to_string(),
        kind: QueuedPromptCommandKind::SlashCommand,
        enqueued_ms: None,
    });
    assert!(queued.label.starts_with("命令 "));
    assert_eq!(queued.tone, TuiTone::Warning);

    let lines = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
    assert_eq!(clip_sidebar_body_lines(&lines, 0), Vec::<String>::new());
    assert_eq!(clip_sidebar_body_lines(&lines, 1), vec!["... +4 more".to_string()]);
    assert_eq!(clip_sidebar_body_lines(&lines, 3), vec!["a", "b", "... +2 more"]);
    assert_eq!(sidebar_lines(ratatui::text::Line::raw("head"), &lines, 0).len(), 0);
    assert_eq!(sidebar_lines(ratatui::text::Line::raw("head"), &lines, 1).len(), 1);
}

#[test]
fn last_submission_and_step_timing_helpers_cover_terminal_variants() {
    let mut state = TuiState::default();
    assert_eq!(last_submission_pill(&state).label, "上次空闲");
    state.prompt.last_submission = Some(PromptSubmission {
        stream_id: None,
        session_id: None,
        text: "text".to_string(),
        root: None,
        model: None,
        history_len: 0,
        status: PromptSubmissionStatus::TimedOut { message: "slow".to_string() },
    });
    assert_eq!(last_submission_pill(&state).label, "上次失败");

    let running = UiTaskStepItem {
        message_id: UiMessageId::new("step"),
        step_index: 1,
        state: UiStepState::Running,
        started_ms: 10,
        finished_ms: None,
        model: None,
        finish_reason: None,
        usage: UiTokenUsage::default(),
    };
    assert_eq!(step_timing_line(&running), "耗时: 开始=10 结束=进行中");
    let finished = UiTaskStepItem { finished_ms: Some(25), ..running };
    assert_eq!(step_timing_line(&finished), "耗时: 开始=10 结束=25 总计=15ms");
}
