//! tui_v2 controller 骨架。
//!
//! 该模块只负责把 terminal 输入归一为状态动作，不持有 renderer，也不直接访问
//! gateway 网络逻辑。当前阶段控制器只承担输入归一化职责：
//! - tick / resize 轮询
//! - prompt 输入与退出键
//! - 基础滚动控制
//! - 提交意图生成与 modal 开关

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};

use super::input::{
    TuiPromptSuggestionMotion, TuiSlashCommandInvocation, apply_selected_suggestion,
    move_prompt_suggestion_selection, parse_slash_command, selected_prompt_suggestion,
};
use super::model::{
    PromptMode, PromptMotion, PromptSubmission, PromptSubmissionStatus, QueuedPromptCommand,
    QueuedPromptCommandKind, UiConfirmOverlay, UiErrorOverlay, UiOverlay, UiOverlayKind,
    UiQuestionOverlay, UiTaskOverlay, UiTodoOverlay,
};
use super::render::layout::FullscreenLayoutSlots;
use super::state::{
    TuiAction, TuiScrollState, TuiState, reduce_tui_state, select_transcript_message_anchors,
    select_visible_grouped_transcript_window,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum KeymapLayerResult {
    Handled(TuiControllerCommand),
    Pass,
}

/// controller 轮询得到的归一化事件。
#[derive(Debug)]
pub(crate) enum TuiControllerEvent {
    Tick,
    Terminal(CrosstermEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiOverlayCommand {
    OpenSearchOverlay,
    OpenPendingQuestions,
    OpenTodoPanel,
    OpenTaskPanel,
    OpenMcpPanel,
    OpenMemoryPanel,
    ConfirmExit,
    ConfirmAccepted(UiConfirmOverlay),
    QuestionSubmitted(UiQuestionOverlay),
    QuestionRejected(UiQuestionOverlay),
    TodoRefresh(UiTodoOverlay),
    TodoSave(UiTodoOverlay),
}

/// controller 每次处理事件后给 app 的控制指令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiControllerCommand {
    Continue,
    Quit,
    CancelActiveSubmission,
    Overlay(TuiOverlayCommand),
    SubmitPrompt(PromptSubmission),
    ExecuteSlashCommand(TuiSlashCommandInvocation),
}

/// fullscreen skeleton 的 terminal controller。
#[derive(Debug, Clone)]
pub(crate) struct TuiController {
    tick_rate: Duration,
    spinner_frame: usize,
}

impl Default for TuiController {
    fn default() -> Self {
        Self::new(Duration::from_millis(180))
    }
}

impl TuiController {
    /// 基于给定 tick 速率创建 controller。
    pub(crate) fn new(tick_rate: Duration) -> Self {
        Self { tick_rate, spinner_frame: 0 }
    }

    /// 返回当前动画帧索引，供 renderer 在 header 中绘制轻量状态指示。
    pub(crate) fn spinner_frame(&self) -> usize {
        self.spinner_frame
    }

    /// 轮询下一个终端事件；超时后返回 `Tick`。
    pub(crate) fn next_event(&mut self) -> Result<TuiControllerEvent> {
        self.next_event_with_timeout(self.tick_rate)
    }

    /// 使用显式超时轮询一个终端事件；超时后返回 `Tick`。
    pub(crate) fn next_event_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<TuiControllerEvent> {
        if event::poll(timeout)? {
            Ok(TuiControllerEvent::Terminal(event::read()?))
        } else {
            self.spinner_frame = (self.spinner_frame + 1) % 4;
            Ok(TuiControllerEvent::Tick)
        }
    }

    /// 将 renderer 反馈的 viewport 能力同步回状态层。
    pub(crate) fn sync_layout(&mut self, state: &mut TuiState, slots: &impl TuiRenderFeedbackLike) {
        let layout = slots.layout();
        let next_viewport_height = layout.scrollable.height.saturating_sub(2);
        let next_viewport_width = layout.scrollable.width.saturating_sub(2);
        if state.scroll.viewport_height == next_viewport_height
            && state.scroll.viewport_width == next_viewport_width
        {
            return;
        }

        let mut scroll = state.scroll.clone();
        scroll.sync_viewport(next_viewport_height, next_viewport_width);
        reduce_tui_state(state, TuiAction::ScrollSet(scroll));
        state.refresh_transcript_layout_for_current_width();
    }

    /// 处理一个 controller 事件。
    pub(crate) fn handle_event(
        &mut self,
        state: &mut TuiState,
        event: TuiControllerEvent,
    ) -> TuiControllerCommand {
        match event {
            TuiControllerEvent::Tick => TuiControllerCommand::Continue,
            TuiControllerEvent::Terminal(event) => self.handle_terminal_event(state, event),
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut TuiState,
        event: CrosstermEvent,
    ) -> TuiControllerCommand {
        match event {
            CrosstermEvent::Key(key)
                if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                self.handle_key_event(state, key)
            }
            _ => TuiControllerCommand::Continue,
        }
    }

    fn handle_key_event(&mut self, state: &mut TuiState, key: KeyEvent) -> TuiControllerCommand {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'd'))
        {
            // 若会话中已有消息，弹二次确认；无消息则直接退出
            if state.messages.is_empty() && state.session.session_id.is_none() {
                return TuiControllerCommand::Quit;
            }
            return TuiControllerCommand::Overlay(TuiOverlayCommand::ConfirmExit);
        }

        if let KeymapLayerResult::Handled(command) = handle_overlay_layer_key(state, key) {
            return command;
        }

        if let KeymapLayerResult::Handled(command) = handle_prompt_layer_key(state, key) {
            return command;
        }

        if let KeymapLayerResult::Handled(command) = handle_scroll_layer_key(state, key) {
            return command;
        }

        TuiControllerCommand::Continue
    }

    fn handle_escape(&mut self, state: &mut TuiState) -> TuiControllerCommand {
        if state.overlays.active().is_some() {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            return TuiControllerCommand::Continue;
        }

        if !state.prompt.value.is_empty() {
            reduce_tui_state(state, TuiAction::PromptValueSet(String::new()));
            return TuiControllerCommand::Continue;
        }

        TuiControllerCommand::Quit
    }
}

/// 为了避免 controller 依赖 renderer 具体实现，这里只约定同步布局时所需的最小视图。
pub(crate) trait TuiRenderFeedbackLike {
    fn layout(&self) -> &FullscreenLayoutSlots;
}

impl TuiRenderFeedbackLike for super::render::TuiRenderFeedback {
    fn layout(&self) -> &FullscreenLayoutSlots {
        &self.layout
    }
}

fn can_edit_prompt(modifiers: KeyModifiers, state: &TuiState) -> bool {
    state.overlays.active().is_none()
        && matches!(modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT)
}

const QUESTION_CUSTOM_ANSWER_PREFIX: &str = "__custom__:";

fn append_prompt_char(state: &mut TuiState, ch: char) {
    reduce_tui_state(state, TuiAction::PromptInsert(ch.to_string()));
}

fn backspace_prompt(state: &mut TuiState) {
    reduce_tui_state(state, TuiAction::PromptBackspace);
}

fn insert_prompt_newline(state: &mut TuiState) {
    reduce_tui_state(state, TuiAction::PromptInsert("\n".to_string()));
}

fn delete_prompt(state: &mut TuiState) {
    reduce_tui_state(state, TuiAction::PromptDelete);
}

fn move_prompt_cursor(state: &mut TuiState, motion: PromptMotion) {
    reduce_tui_state(state, TuiAction::PromptCursorMove(motion));
}

fn prompt_history_previous(state: &mut TuiState) {
    reduce_tui_state(state, TuiAction::PromptHistoryPrevious);
}

fn prompt_history_next(state: &mut TuiState) {
    reduce_tui_state(state, TuiAction::PromptHistoryNext);
}

fn accept_prompt_suggestion(state: &mut TuiState) {
    if let Some(replacement) = apply_selected_suggestion(state) {
        reduce_tui_state(state, TuiAction::PromptValueSet(replacement));
    }
}

fn try_accept_prompt_suggestion(state: &mut TuiState) -> bool {
    if state.prompt.is_busy() || state.prompt.selected_suggestion_index.is_none() {
        return false;
    }

    let Some(replacement) = apply_selected_suggestion(state) else {
        return false;
    };
    reduce_tui_state(state, TuiAction::PromptValueSet(replacement));
    true
}

fn can_navigate_prompt_suggestions(state: &TuiState) -> bool {
    matches!(state.prompt.mode, PromptMode::SlashCommand)
        && selected_prompt_suggestion(state).is_some()
}

fn move_prompt_suggestion_cursor(state: &mut TuiState, motion: TuiPromptSuggestionMotion) {
    if let Some(next_index) = move_prompt_suggestion_selection(state, motion) {
        reduce_tui_state(state, TuiAction::PromptSuggestionSelectionSet(Some(next_index)));
    }
}

pub(crate) fn build_prompt_submission(state: &TuiState, text: &str) -> Option<PromptSubmission> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return None;
    }

    let mut submission = PromptSubmission::new(text.clone())
        .with_stream_id(now_ms())
        .with_history_len(state.prompt.history.entries.len());

    if let Some(session_id) = state.session.session_id.as_ref() {
        submission = submission.with_session_id(session_id.clone());
    }

    if let Some(root) = state.project.workspace_root.as_ref() {
        submission = submission.with_root(root.display().to_string());
    }

    if let Some(model) = state.status.model_name.as_ref() {
        submission = submission.with_model(model.clone());
    }

    Some(submission)
}

fn submit_prompt_command(state: &TuiState) -> TuiControllerCommand {
    build_prompt_submission(state, state.prompt.value.as_str())
        .map(TuiControllerCommand::SubmitPrompt)
        .unwrap_or(TuiControllerCommand::Continue)
}

fn queue_prompt_command(state: &mut TuiState) {
    let text = state.prompt.value.trim().to_string();
    if text.is_empty() {
        return;
    }

    let kind = if text.starts_with('/') {
        QueuedPromptCommandKind::SlashCommand
    } else {
        QueuedPromptCommandKind::Submit
    };

    reduce_tui_state(
        state,
        TuiAction::PromptCommandQueued(QueuedPromptCommand {
            raw: text,
            kind,
            enqueued_ms: Some(now_ms()),
        }),
    );
    reduce_tui_state(state, TuiAction::PromptValueSet(String::new()));
}

fn submit_or_execute_prompt_command(state: &mut TuiState) -> TuiControllerCommand {
    let text = state.prompt.value.trim().to_string();
    if text.is_empty() {
        return TuiControllerCommand::Continue;
    }

    if state.prompt.is_busy() {
        queue_prompt_command(state);
        return TuiControllerCommand::Continue;
    }

    if matches!(state.prompt.mode, PromptMode::SlashCommand) || text.starts_with('/') {
        let invocation = parse_slash_command(text.as_str()).unwrap_or(TuiSlashCommandInvocation {
            raw: text.clone(),
            token: text.trim_start_matches('/').to_string(),
            argument: None,
            kind: None,
        });
        reduce_tui_state(state, TuiAction::PromptValueSet(String::new()));
        return TuiControllerCommand::ExecuteSlashCommand(invocation);
    }

    submit_prompt_command(state)
}

fn toggle_help_overlay(state: &mut TuiState) {
    if state.overlays.active().is_some() {
        reduce_tui_state(state, TuiAction::OverlayPopped);
        return;
    }

    reduce_tui_state(
        state,
        TuiAction::OverlayPushed(UiOverlay::Error(UiErrorOverlay {
            title: "TUI 输入帮助".to_string(),
            message: "区域：顶部会话栏 / 对话流 / 状态栏 / 输入区 / 弹层。\n按键：直接输入即可编辑，Shift+Enter 换行，Enter 发送；斜杠建议面板出现时可用 Up/Down 切换、Tab 或 Enter 接受当前项，接受后再按 Enter 执行命令；Ctrl+F 打开搜索，F2 打开问题或授权面板，F3 打开待办面板，F4 打开任务面板，F5 打开 MCP 服务器面板，F6 打开内存文件面板，PageUp/PageDown 滚动会话，Esc 关闭弹层或清空输入，Ctrl+C 退出。\n弹层按键：确认弹层用 Enter/Esc，搜索弹层支持文字输入 + Backspace/Delete + Tab/Shift+Tab + Enter + End/u + Ctrl+S，提问弹层支持 1-9 + Tab + Enter + Ctrl+R，待办弹层支持 Up/Down + Space + s/r + Esc，任务弹层支持 Up/Down + Enter + Esc，MCP/内存弹层支持 Up/Down + Esc。".to_string(),
            recoverable: true,
        })),
    );
}

fn handle_overlay_layer_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    let Some(kind) = state.overlays.active().map(UiOverlay::kind) else {
        return KeymapLayerResult::Pass;
    };

    match kind {
        UiOverlayKind::Confirm => handle_confirm_overlay_key(state, key),
        UiOverlayKind::Search => handle_search_overlay_key(state, key),
        UiOverlayKind::Question => handle_question_overlay_key(state, key),
        UiOverlayKind::Todo => handle_todo_overlay_key(state, key),
        UiOverlayKind::Task => handle_task_overlay_key(state, key),
        UiOverlayKind::Error => handle_error_overlay_key(state, key),
        UiOverlayKind::CommandPalette => handle_generic_overlay_key(state, key),
        UiOverlayKind::Mcp => handle_mcp_overlay_key(state, key),
        UiOverlayKind::Memory => handle_memory_overlay_key(state, key),
    }
}

fn handle_confirm_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Esc => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Enter => {
            let Some(UiOverlay::Confirm(overlay)) = state.overlays.active().cloned() else {
                return KeymapLayerResult::Pass;
            };
            KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
                TuiOverlayCommand::ConfirmAccepted(overlay),
            ))
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn handle_error_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Esc => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Enter => {
            if let Some(UiOverlay::Error(overlay)) = state.overlays.active()
                && overlay.recoverable
            {
                reduce_tui_state(state, TuiAction::OverlayPopped);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn handle_generic_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Esc => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn handle_search_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Backspace) => {
            backspace_search_query(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Delete) => {
            clear_search_query(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Up | KeyCode::BackTab) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            move_search_selection(state, -1);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down | KeyCode::Tab) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            move_search_selection(state, 1);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Enter) => {
            if jump_to_selected_search_match(state) {
                reduce_tui_state(state, TuiAction::OverlayPopped);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::End) => {
            scroll_to_tail(state);
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char('u')) => {
            if jump_to_unread_message(state) {
                reduce_tui_state(state, TuiAction::OverlayPopped);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            toggle_search_case_sensitivity(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char(ch))
            if matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT) =>
        {
            append_search_query(state, ch);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn search_overlay_query(state: &TuiState) -> Option<String> {
    let Some(UiOverlay::Search(overlay)) = state.overlays.active() else {
        return None;
    };
    Some(overlay.query.clone())
}

fn append_search_query(state: &mut TuiState, ch: char) {
    let mut query = search_overlay_query(state).unwrap_or_default();
    query.push(ch);
    reduce_tui_state(state, TuiAction::SearchQuerySet(query));
}

fn backspace_search_query(state: &mut TuiState) {
    let Some(mut query) = search_overlay_query(state) else {
        return;
    };
    query.pop();
    reduce_tui_state(state, TuiAction::SearchQuerySet(query));
}

fn clear_search_query(state: &mut TuiState) {
    if search_overlay_query(state).is_none() {
        return;
    }
    reduce_tui_state(state, TuiAction::SearchQuerySet(String::new()));
}

fn toggle_search_case_sensitivity(state: &mut TuiState) {
    let Some(query) = search_overlay_query(state) else {
        return;
    };

    if let Some(UiOverlay::Search(overlay)) = state.overlays.stack.last_mut() {
        overlay.case_sensitive = !overlay.case_sensitive;
    }

    reduce_tui_state(state, TuiAction::SearchQuerySet(query));
}

fn move_search_selection(state: &mut TuiState, delta: isize) {
    let Some(UiOverlay::Search(overlay)) = state.overlays.stack.last_mut() else {
        return;
    };

    if overlay.matches.is_empty() {
        overlay.selected_index = None;
        return;
    }

    let len = overlay.matches.len();
    let current = overlay.selected_index.unwrap_or_default();
    let next = if delta.is_negative() {
        if current == 0 {
            len.saturating_sub(1)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        }
    } else {
        current.saturating_add(delta.cast_unsigned()) % len
    };
    overlay.selected_index = Some(next);
}

fn jump_to_selected_search_match(state: &mut TuiState) -> bool {
    let Some(UiOverlay::Search(overlay)) = state.overlays.active() else {
        return false;
    };
    let Some(selected_index) = overlay.selected_index else {
        return false;
    };
    let Some(message_id) =
        overlay.matches.get(selected_index).and_then(|item| item.message_id.as_ref())
    else {
        return false;
    };
    let Some(message_index) = state.messages.iter().position(|message| message.id() == message_id)
    else {
        return false;
    };

    jump_to_message(state, message_index)
}

fn jump_to_unread_message(state: &mut TuiState) -> bool {
    let Some(first_unread_message) = state
        .scroll
        .last_seen_message
        .map(|last_seen| last_seen.saturating_add(1))
        .filter(|index| *index < state.messages.len())
        .or_else(|| (!state.messages.is_empty()).then_some(0))
    else {
        return false;
    };

    jump_to_message(state, first_unread_message)
}

fn jump_to_message(state: &mut TuiState, message_index: usize) -> bool {
    let anchors = select_transcript_message_anchors(state);
    let Some(target_anchor) = anchors
        .iter()
        .copied()
        .rfind(|anchor| *anchor <= message_index)
        .or_else(|| anchors.first().copied())
    else {
        return false;
    };

    let mut scroll = state.scroll.clone();
    scroll.follow_tail = false;
    scroll.top_message = target_anchor;
    reduce_tui_state(state, TuiAction::ScrollSet(scroll));
    true
}

fn handle_question_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            let Some(UiOverlay::Question(overlay)) = state.overlays.active().cloned() else {
                return KeymapLayerResult::Pass;
            };
            KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
                TuiOverlayCommand::QuestionRejected(overlay),
            ))
        }
        (_, KeyCode::Enter) => {
            let Some(UiOverlay::Question(overlay)) = state.overlays.active().cloned() else {
                return KeymapLayerResult::Pass;
            };
            KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
                TuiOverlayCommand::QuestionSubmitted(overlay),
            ))
        }
        (_, KeyCode::Tab | KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                move_question_prompt_selection(overlay, 1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::BackTab | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                move_question_prompt_selection(overlay, -1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Backspace) => {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                backspace_question_custom_answer(overlay);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Delete) => {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                clear_question_custom_answer(overlay);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char(ch)) if ch.is_ascii_digit() && ch != '0' => {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                toggle_question_option_by_digit(overlay, ch);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char(ch))
            if question_overlay_allows_custom_input(state)
                && matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT) =>
        {
            if let Some(UiOverlay::Question(overlay)) = state.overlays.stack.last_mut() {
                append_question_custom_answer(overlay, ch);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn question_overlay_allows_custom_input(state: &TuiState) -> bool {
    let Some(UiOverlay::Question(overlay)) = state.overlays.active() else {
        return false;
    };
    overlay.prompts.get(overlay.selected_index).is_some_and(|prompt| prompt.allow_custom_input)
}

fn move_question_prompt_selection(overlay: &mut UiQuestionOverlay, delta: isize) {
    if overlay.prompts.is_empty() {
        overlay.selected_index = 0;
        return;
    }

    let max_index = overlay.prompts.len().saturating_sub(1);
    overlay.selected_index = if delta.is_negative() {
        overlay.selected_index.saturating_sub(delta.unsigned_abs())
    } else {
        overlay.selected_index.saturating_add(delta.cast_unsigned()).min(max_index)
    };
}

fn toggle_question_option_by_digit(overlay: &mut UiQuestionOverlay, digit: char) {
    let Some(prompt) = overlay.prompts.get(overlay.selected_index) else {
        return;
    };
    let Some(option_index) = digit
        .to_digit(10)
        .map(|value| usize::try_from(value.saturating_sub(1)).unwrap_or_default())
    else {
        return;
    };
    let Some(option) = prompt.options.get(option_index) else {
        return;
    };

    let answers = &mut overlay.answers[overlay.selected_index];
    let selected_label = option.label.clone();

    if prompt.multiple {
        if let Some(index) = answers.iter().position(|answer| answer == &selected_label) {
            answers.remove(index);
        } else {
            answers.push(selected_label);
        }
        return;
    }

    answers.retain(|answer| !prompt.options.iter().any(|candidate| candidate.label == *answer));
    answers.push(selected_label);
}

fn append_question_custom_answer(overlay: &mut UiQuestionOverlay, ch: char) {
    let answers = &mut overlay.answers[overlay.selected_index];
    let mut custom = question_custom_answer(answers);
    custom.push(ch);
    set_question_custom_answer(answers, custom);
}

fn backspace_question_custom_answer(overlay: &mut UiQuestionOverlay) {
    let answers = &mut overlay.answers[overlay.selected_index];
    let mut custom = question_custom_answer(answers);
    custom.pop();
    set_question_custom_answer(answers, custom);
}

fn clear_question_custom_answer(overlay: &mut UiQuestionOverlay) {
    let answers = &mut overlay.answers[overlay.selected_index];
    set_question_custom_answer(answers, String::new());
}

fn question_custom_answer(answers: &[String]) -> String {
    answers
        .iter()
        .find_map(|answer| answer.strip_prefix(QUESTION_CUSTOM_ANSWER_PREFIX))
        .map(ToOwned::to_owned)
        .unwrap_or_default()
}

fn set_question_custom_answer(answers: &mut Vec<String>, value: String) {
    answers.retain(|answer| !answer.starts_with(QUESTION_CUSTOM_ANSWER_PREFIX));
    if !value.is_empty() {
        answers.push(format!("{QUESTION_CUSTOM_ANSWER_PREFIX}{value}"));
    }
}

fn handle_todo_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            if let Some(UiOverlay::Todo(overlay)) = state.overlays.stack.last_mut() {
                move_todo_selection(overlay, -1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            if let Some(UiOverlay::Todo(overlay)) = state.overlays.stack.last_mut() {
                move_todo_selection(overlay, 1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char(' ') | KeyCode::Enter) => {
            if let Some(UiOverlay::Todo(overlay)) = state.overlays.stack.last_mut() {
                toggle_selected_todo_item(overlay);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char('r')) => {
            let Some(UiOverlay::Todo(overlay)) = state.overlays.active().cloned() else {
                return KeymapLayerResult::Pass;
            };
            KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
                TuiOverlayCommand::TodoRefresh(overlay),
            ))
        }
        (_, KeyCode::Char('s')) => {
            let Some(UiOverlay::Todo(overlay)) = state.overlays.active().cloned() else {
                return KeymapLayerResult::Pass;
            };
            KeymapLayerResult::Handled(TuiControllerCommand::Overlay(TuiOverlayCommand::TodoSave(
                overlay,
            )))
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn move_todo_selection(overlay: &mut UiTodoOverlay, delta: isize) {
    if overlay.items.is_empty() {
        overlay.selected_index = 0;
        return;
    }

    let max_index = overlay.items.len().saturating_sub(1);
    overlay.selected_index = if delta.is_negative() {
        overlay.selected_index.saturating_sub(delta.unsigned_abs())
    } else {
        overlay.selected_index.saturating_add(delta.cast_unsigned()).min(max_index)
    };
}

fn toggle_selected_todo_item(overlay: &mut UiTodoOverlay) {
    let Some(item) = overlay.items.get_mut(overlay.selected_index) else {
        return;
    };

    item.status = if item.status.eq_ignore_ascii_case("completed") {
        "pending".to_string()
    } else {
        "completed".to_string()
    };
    overlay.dirty = true;
}

fn handle_task_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Up | KeyCode::BackTab) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            if let Some(UiOverlay::Task(overlay)) = state.overlays.stack.last_mut() {
                move_task_selection(overlay, -1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down | KeyCode::Tab) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            if let Some(UiOverlay::Task(overlay)) = state.overlays.stack.last_mut() {
                move_task_selection(overlay, 1);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Enter) => {
            if jump_to_selected_task_step(state) {
                reduce_tui_state(state, TuiAction::OverlayPopped);
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn move_task_selection(overlay: &mut UiTaskOverlay, delta: isize) {
    if overlay.steps.is_empty() {
        overlay.selected_index = 0;
        return;
    }

    let len = overlay.steps.len();
    let current = overlay.selected_index.min(len.saturating_sub(1));
    overlay.selected_index = if delta.is_negative() {
        if current == 0 {
            len.saturating_sub(1)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        }
    } else {
        current.saturating_add(delta.cast_unsigned()) % len
    };
}

fn jump_to_selected_task_step(state: &mut TuiState) -> bool {
    let Some(UiOverlay::Task(overlay)) = state.overlays.active() else {
        return false;
    };
    let Some(message_id) = overlay.steps.get(overlay.selected_index).map(|step| &step.message_id)
    else {
        return false;
    };
    let Some(message_index) = state.messages.iter().position(|message| message.id() == message_id)
    else {
        return false;
    };

    jump_to_message(state, message_index)
}

fn handle_mcp_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Esc => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Up | KeyCode::BackTab => {
            if let Some(UiOverlay::Mcp(overlay)) = state.overlays.stack.last_mut() {
                let len = overlay.servers.len();
                if len > 0 {
                    overlay.selected_index = (overlay.selected_index + len.saturating_sub(1)) % len;
                }
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Down | KeyCode::Tab => {
            if let Some(UiOverlay::Mcp(overlay)) = state.overlays.stack.last_mut() {
                let len = overlay.servers.len();
                if len > 0 {
                    overlay.selected_index = (overlay.selected_index + 1) % len;
                }
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn handle_memory_overlay_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Esc => {
            reduce_tui_state(state, TuiAction::OverlayPopped);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Up | KeyCode::BackTab => {
            if let Some(UiOverlay::Memory(overlay)) = state.overlays.stack.last_mut() {
                let len = overlay.entries.len();
                if len > 0 {
                    overlay.selected_index = (overlay.selected_index + len.saturating_sub(1)) % len;
                }
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Down | KeyCode::Tab => {
            if let Some(UiOverlay::Memory(overlay)) = state.overlays.stack.last_mut() {
                let len = overlay.entries.len();
                if len > 0 {
                    overlay.selected_index = (overlay.selected_index + 1) % len;
                }
            }
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Handled(TuiControllerCommand::Continue),
    }
}

fn handle_prompt_layer_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    if state.overlays.active().is_some() {
        return KeymapLayerResult::Pass;
    }

    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => KeymapLayerResult::Handled(handle_prompt_escape(state)),
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => KeymapLayerResult::Handled(
            TuiControllerCommand::Overlay(TuiOverlayCommand::OpenSearchOverlay),
        ),
        (_, KeyCode::F(2)) => KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
            TuiOverlayCommand::OpenPendingQuestions,
        )),
        (_, KeyCode::F(3)) => KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
            TuiOverlayCommand::OpenTodoPanel,
        )),
        (_, KeyCode::F(4)) => KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
            TuiOverlayCommand::OpenTaskPanel,
        )),
        (_, KeyCode::F(5)) => KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
            TuiOverlayCommand::OpenMcpPanel,
        )),
        (_, KeyCode::F(6)) => KeymapLayerResult::Handled(TuiControllerCommand::Overlay(
            TuiOverlayCommand::OpenMemoryPanel,
        )),
        (_, KeyCode::F(1) | KeyCode::Char('?')) => {
            toggle_help_overlay(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::BackTab | KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p'))
            if can_navigate_prompt_suggestions(state) =>
        {
            move_prompt_suggestion_cursor(state, TuiPromptSuggestionMotion::Previous);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (KeyModifiers::SHIFT, KeyCode::Enter) => {
            insert_prompt_newline(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Tab) => {
            accept_prompt_suggestion(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Enter) if try_accept_prompt_suggestion(state) => {
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Enter) => KeymapLayerResult::Handled(submit_or_execute_prompt_command(state)),
        (_, KeyCode::Backspace) => {
            backspace_prompt(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Delete) => {
            delete_prompt(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Left) if state.prompt.can_move_cursor(PromptMotion::Left) => {
            move_prompt_cursor(state, PromptMotion::Left);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Right) if state.prompt.can_move_cursor(PromptMotion::Right) => {
            move_prompt_cursor(state, PromptMotion::Right);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Home) if !state.prompt.value.is_empty() => {
            move_prompt_cursor(state, PromptMotion::Home);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::End) if !state.prompt.value.is_empty() => {
            move_prompt_cursor(state, PromptMotion::End);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n'))
            if can_navigate_prompt_suggestions(state) =>
        {
            move_prompt_suggestion_cursor(state, TuiPromptSuggestionMotion::Next);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Up) if state.prompt.can_move_cursor(PromptMotion::Up) => {
            move_prompt_cursor(state, PromptMotion::Up);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down) if state.prompt.can_move_cursor(PromptMotion::Down) => {
            move_prompt_cursor(state, PromptMotion::Down);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p'))
            if !state.prompt.history.entries.is_empty() =>
        {
            prompt_history_previous(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n'))
            if state.prompt.history.selected_index.is_some() =>
        {
            prompt_history_next(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        (_, KeyCode::Char(ch)) if can_edit_prompt(key.modifiers, state) => {
            append_prompt_char(state, ch);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Pass,
    }
}

fn handle_scroll_layer_key(state: &mut TuiState, key: KeyEvent) -> KeymapLayerResult {
    match key.code {
        KeyCode::Up => {
            scroll_relative(state, -1);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Down => {
            scroll_relative(state, 1);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::PageUp => {
            scroll_relative(state, -page_scroll_amount(state));
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::PageDown => {
            scroll_relative(state, page_scroll_amount(state));
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::Home => {
            scroll_to_top(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        KeyCode::End => {
            scroll_to_tail(state);
            KeymapLayerResult::Handled(TuiControllerCommand::Continue)
        }
        _ => KeymapLayerResult::Pass,
    }
}

fn handle_prompt_escape(state: &mut TuiState) -> TuiControllerCommand {
    if !state.prompt.value.is_empty() {
        reduce_tui_state(state, TuiAction::PromptValueSet(String::new()));
        return TuiControllerCommand::Continue;
    }

    if state.prompt.is_busy() {
        return TuiControllerCommand::CancelActiveSubmission;
    }

    // 有会话内容时弹二次确认，否则直接退出
    if state.messages.is_empty() && state.session.session_id.is_none() {
        return TuiControllerCommand::Quit;
    }
    TuiControllerCommand::Overlay(TuiOverlayCommand::ConfirmExit)
}

fn page_scroll_amount(state: &TuiState) -> isize {
    select_visible_grouped_transcript_window(state).viewport_summary().message_capacity.max(1)
        as isize
}

fn scroll_relative(state: &mut TuiState, delta: isize) {
    let anchors = select_transcript_message_anchors(state);
    if anchors.is_empty() {
        return;
    }

    let current_anchor_index =
        anchors.iter().rposition(|anchor| *anchor <= state.scroll.top_message).unwrap_or_default();
    let max_top = anchors.len().saturating_sub(1);
    let mut scroll = state.scroll.clone();
    let next_top = offset_index(current_anchor_index, delta).min(max_top);
    scroll.top_message = anchors[next_top];
    scroll.follow_tail = next_top >= max_top;
    reduce_tui_state(state, TuiAction::ScrollSet(scroll));
}

fn scroll_to_top(state: &mut TuiState) {
    let mut scroll = TuiScrollState { top_message: 0, follow_tail: false, ..state.scroll.clone() };
    scroll.follow_tail = false;
    reduce_tui_state(state, TuiAction::ScrollSet(scroll));
}

fn scroll_to_tail(state: &mut TuiState) {
    let anchors = select_transcript_message_anchors(state);
    let mut scroll = state.scroll.clone();
    scroll.follow_tail = true;
    scroll.top_message = anchors.last().copied().unwrap_or_default();
    reduce_tui_state(state, TuiAction::ScrollSet(scroll));
}

fn offset_index(current: usize, delta: isize) -> usize {
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta.cast_unsigned())
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

#[allow(dead_code)]
fn _assert_prompt_status(_: PromptSubmissionStatus) {}
