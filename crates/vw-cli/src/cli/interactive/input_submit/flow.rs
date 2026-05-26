//! CLI 交互模式用户输入提交流程处理
//!
//! 本模块负责处理 CLI 交互模式下的用户输入提交流程，是用户与 Agent 交互的核心入口。
//! 主要功能包括：
//!
//! - 处理用户输入并路由到相应的处理器（内联命令、会话处理器等）
//! - 管理会话状态（历史消息、转录记录、统计信息等）
//! - 协调流式响应的渲染和超时控制
//! - 处理工具迭代限制和错误恢复
//!
//! ## 架构位置
//!
//! ```text
//! cli/interactive/
//! ├── input_submit/
//! │   ├── flow.rs      <-- 本文件：主流程编排
//! │   └── commands.rs  -- 内联命令处理
//! ```
//!
//! ## 处理流程
//!
//! 1. 空输入 → 仅重绘 UI
//! 2. 等待清除确认 → 委托给 `handle_pending_clear`
//! 3. 内联命令 → 委托给 `handle_inline_command`
//! 4. 普通消息 → 构建上下文 → 调用会话处理器 → 更新状态

use crate::app::agent::agent::loop_::progress;
use crate::app::agent::config::Config;
use crate::app::agent::memory::MemoryCategory;
use crate::session::ui_types as models;
use anyhow::Result;

use super::super::super::processor::run_session_processor_for_cli;
use super::super::super::session::{collect_modified_files, maybe_refresh_cli_session_title};
use super::super::super::setup::CliSetup;
use super::super::super::stats::{CliStats, build_session_title};
use super::super::super::transcript::{
    TranscriptEntry, TranscriptRole, build_streaming_transcript_view,
};
use super::super::super::tui::CliTui;
use super::commands::{handle_inline_command, handle_pending_clear};
use crate::app::agent::agent::loop_::context::build_context;
use crate::app::agent::agent::loop_::core::{
    AUTOSAVE_MIN_MESSAGE_CHARS, autosave_memory_key, effective_message_timeout_secs,
    is_tool_iteration_limit_error, message_timeout_budget_secs,
};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind};

/// 处理用户输入提交结果
///
/// 这是 CLI 交互模式的核心函数，负责编排用户输入的完整处理流程。
/// 根据输入类型和当前状态，将请求路由到适当的处理器。
///
/// # 参数
///
/// * `config` - Agent 配置，包含超时、内存设置等
/// * `setup` - CLI 启动设置，包含 provider、模型、内存后端等
/// * `user_input` - 用户输入的原始字符串
/// * `tui` - TUI 渲染器，用于绘制界面
/// * `transcript` - 会话转录记录，包含所有交互历史
/// * `session_history` - 模型会话历史，用于上下文传递
/// * `session_id` - 当前会话的唯一标识符
/// * `stream_id` - 流式响应的递增 ID，用于区分不同请求
/// * `session_title_refreshed` - 会话标题是否已刷新
/// * `input` - 当前输入缓冲区（可变引用）
/// * `cursor_idx` - 光标位置
/// * `busy` - 是否正在处理请求
/// * `awaiting_clear_confirm` - 是否等待清除确认
/// * `stats` - CLI 统计信息
/// * `workspace` - 工作区路径
/// * `modified_files` - 已修改文件列表
/// * `files_collapsed` - 文件列表是否折叠
/// * `draft` - 流式响应草稿缓冲区
/// * `scroll_back` - 滚动偏移量
/// * `show_menu` - 是否显示菜单
/// * `final_output` - 最终输出缓冲区
///
/// # 返回
///
/// 返回 `SubmitOutcome` 枚举，指示是否应继续循环或退出。
///
/// # 处理流程
///
/// 1. **空输入处理**：如果输入为空，仅重绘 UI 并返回继续
/// 2. **转录记录更新**：将用户输入添加到转录记录，更新统计
/// 3. **清除确认处理**：如果正在等待清除确认，委托给专门处理器
/// 4. **内联命令处理**：检查是否为内联命令（如 /help, /exit）
/// 5. **内存自动保存**：如果启用且满足条件，保存用户输入到记忆
/// 6. **上下文构建**：从记忆中检索相关上下文，丰富用户输入
/// 7. **会话处理器调用**：启动会话处理器，处理流式响应
/// 8. **超时和错误处理**：处理超时、工具迭代限制等异常
/// 9. **状态更新**：更新会话历史、统计信息、修改文件列表
///
/// # 示例
///
/// ```ignore
/// let outcome = handle_submit_result(
///     &config,
///     &setup,
///     user_input,
///     &mut tui,
///     &mut transcript,
///     &mut session_history,
///     &mut session_id,
///     &mut stream_id,
///     &mut session_title_refreshed,
///     &mut input,
///     &mut cursor_idx,
///     &mut busy,
///     &mut awaiting_clear_confirm,
///     &mut stats,
///     workspace,
///     &mut modified_files,
///     &mut files_collapsed,
///     &mut draft,
///     &mut scroll_back,
///     &mut show_menu,
///     &mut final_output,
/// ).await?;
///
/// match outcome {
///     SubmitOutcome::Continue => { /* 继续主循环 */ }
///     SubmitOutcome::Exit => { /* 退出程序 */ }
/// }
/// ```
///
/// # 错误
///
/// 函数可能返回以下错误：
/// - TUI 绘制错误
/// - 会话处理器错误
/// - 内存操作错误
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_submit_result(
    config: &Config,
    setup: &CliSetup,
    user_input: String,
    tui: &mut CliTui,
    transcript: &mut Vec<TranscriptEntry>,
    session_history: &mut Vec<models::ChatMessage>,
    session_id: &mut String,
    stream_id: &mut u64,
    session_title_refreshed: &mut bool,
    input: &str,
    cursor_idx: &mut usize,
    busy: &mut bool,
    awaiting_clear_confirm: &mut bool,
    stats: &mut CliStats,
    workspace: &str,
    modified_files: &mut Vec<String>,
    files_collapsed: &mut bool,
    draft: &mut String,
    scroll_back: &mut u16,
    show_menu: &mut bool,
    final_output: &mut String,
) -> Result<SubmitOutcome> {
    const ESC_INTERRUPT_ERROR: &str = "__cli_turn_interrupted_by_esc__";

    if user_input.is_empty() {
        let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
        tui.draw(
            transcript,
            input,
            *cursor_idx,
            *busy,
            *awaiting_clear_confirm,
            &setup.provider_name,
            &setup.model_name,
            stats,
            workspace,
            draft,
            &session_title,
            modified_files,
            *files_collapsed,
            *scroll_back,
            *show_menu,
        )?;
        return Ok(SubmitOutcome::Continue);
    }

    transcript.push(TranscriptEntry::new(TranscriptRole::User, user_input.clone()));
    stats.user_messages += 1;
    *scroll_back = 0;

    if *awaiting_clear_confirm {
        return handle_pending_clear(
            config,
            setup,
            &user_input,
            tui,
            transcript,
            session_history,
            session_id,
            session_title_refreshed,
            *cursor_idx,
            *busy,
            awaiting_clear_confirm,
            stats,
            workspace,
            modified_files,
            files_collapsed,
            draft,
            *scroll_back,
            *show_menu,
        )
        .await;
    }

    if let Some(outcome) = handle_inline_command(
        &user_input,
        tui,
        transcript,
        *cursor_idx,
        *busy,
        awaiting_clear_confirm,
        stats,
        workspace,
        modified_files,
        files_collapsed,
        draft,
        *scroll_back,
        *show_menu,
        &setup.provider_name,
        &setup.model_name,
    )? {
        return Ok(outcome);
    }

    if config.memory.auto_save && user_input.chars().count() >= AUTOSAVE_MIN_MESSAGE_CHARS {
        let user_key = autosave_memory_key("user_msg");
        let _ = setup.mem.store(&user_key, &user_input, MemoryCategory::Conversation, None).await;
    }

    let mem_context =
        build_context(setup.mem.as_ref(), &user_input, config.memory.min_relevance_score).await;
    let enriched = if mem_context.is_empty() {
        user_input.clone()
    } else {
        format!("{mem_context}{user_input}")
    };
    *busy = true;
    draft.clear();
    let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
    tui.draw(
        transcript,
        input,
        *cursor_idx,
        *busy,
        *awaiting_clear_confirm,
        &setup.provider_name,
        &setup.model_name,
        stats,
        workspace,
        draft,
        &session_title,
        modified_files,
        *files_collapsed,
        *scroll_back,
        *show_menu,
    )?;

    let response_result = {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
        *stream_id = stream_id.saturating_add(1);
        let message_timeout_secs =
            effective_message_timeout_secs(config.channels_config.message_timeout_secs);
        let timeout_budget_secs =
            message_timeout_budget_secs(message_timeout_secs, config.agent.max_tool_iterations);

        let req = crate::app::agent::session::processor::Request {
            stream: *stream_id,
            session: session_id.clone(),
            query: enriched.clone(),
            root: Some(
                std::env::current_dir()
                    .unwrap_or_else(|_| config.workspace_dir.clone())
                    .to_string_lossy()
                    .to_string(),
            ),
            model: Some(setup.model_name.to_string()),
            options: serde_json::Value::Object(serde_json::Map::new()),
            approval: None,
            channel_name: None,
            non_cli_approval_context: None,
            assistant_message_id: None,
            history: session_history.clone(),
            persist_app_session_artifacts: true,
        };

        let mut turn_future = std::pin::pin!(tokio::time::timeout(
            std::time::Duration::from_secs(timeout_budget_secs),
            run_session_processor_for_cli(req, Some(tx)),
        ));

        loop {
            tokio::select! {
                result = &mut turn_future => {
                    break result;
                }
                () = tokio::time::sleep(std::time::Duration::from_millis(80)) => {
                    tui.tick();
                    let (display_transcript, display_draft) =
                        build_streaming_transcript_view(transcript, draft, tui.expand_tool_blocks);
                    let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
                    tui.draw(
                        &display_transcript,
                        input,
                        *cursor_idx,
                        *busy,
                        *awaiting_clear_confirm,
                        &setup.provider_name,
                        &setup.model_name,
                        stats,
                        workspace,
                        &display_draft,
                        &session_title,
                        modified_files,
                        *files_collapsed,
                        *scroll_back,
                        *show_menu,
                    )?;
                    if consume_escape_keypress()? {
                        break Ok(Err(anyhow::anyhow!(ESC_INTERRUPT_ERROR)));
                    }
                }
                maybe_msg = rx.recv() => {
                    if let Some(msg) = maybe_msg {
                        if msg == progress::DRAFT_CLEAR_SENTINEL {
                            draft.clear();
                        } else if let Some(stripped) =
                            msg.strip_prefix(progress::DRAFT_PROGRESS_SENTINEL)
                        {
                            draft.push_str(stripped);
                        } else {
                            draft.push_str(&msg);
                        }
                        let (display_transcript, display_draft) =
                            build_streaming_transcript_view(transcript, draft, tui.expand_tool_blocks);
                        let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
                        tui.draw(
                            &display_transcript,
                            input,
                            *cursor_idx,
                            *busy,
                            *awaiting_clear_confirm,
                            &setup.provider_name,
                            &setup.model_name,
                            stats,
                            workspace,
                            &display_draft,
                            &session_title,
                            modified_files,
                            *files_collapsed,
                            *scroll_back,
                            *show_menu,
                        )?;
                    }
                }
            }
        }
    };

    let response = match response_result {
        Ok(Ok(resp)) => resp,
        Err(_) => {
            *busy = false;
            *modified_files = collect_modified_files(&config.workspace_dir);
            transcript.push(TranscriptEntry::new(
                TranscriptRole::System,
                "⚠️ 请求超时，等待模型响应超时。请稍后重试。",
            ));
            let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
            tui.draw(
                transcript,
                input,
                *cursor_idx,
                *busy,
                *awaiting_clear_confirm,
                &setup.provider_name,
                &setup.model_name,
                stats,
                workspace,
                draft,
                &session_title,
                modified_files,
                *files_collapsed,
                *scroll_back,
                *show_menu,
            )?;
            return Ok(SubmitOutcome::Continue);
        }
        Ok(Err(e)) => {
            *busy = false;
            if e.to_string() == ESC_INTERRUPT_ERROR {
                draft.clear();
                transcript.push(TranscriptEntry::new(TranscriptRole::System, "已中断当前对话"));
                let session_title =
                    build_session_title(stats, &setup.provider_name, &setup.model_name);
                tui.draw(
                    transcript,
                    input,
                    *cursor_idx,
                    *busy,
                    *awaiting_clear_confirm,
                    &setup.provider_name,
                    &setup.model_name,
                    stats,
                    workspace,
                    draft,
                    &session_title,
                    modified_files,
                    *files_collapsed,
                    *scroll_back,
                    *show_menu,
                )?;
                return Ok(SubmitOutcome::Continue);
            }
            *modified_files = collect_modified_files(&config.workspace_dir);
            if is_tool_iteration_limit_error(&e) {
                let limit = config.agent.max_tool_iterations.max(1);
                let pause_notice = format!(
                    "⚠️ 已达到工具迭代次数限制 ({limit})，上下文和进度已保留。回复 \"continue\" 继续，或增加 `agent.max_tool_iterations` 配置。"
                );
                transcript.push(TranscriptEntry::new(TranscriptRole::System, pause_notice));
                let session_title =
                    build_session_title(stats, &setup.provider_name, &setup.model_name);
                tui.draw(
                    transcript,
                    input,
                    *cursor_idx,
                    *busy,
                    *awaiting_clear_confirm,
                    &setup.provider_name,
                    &setup.model_name,
                    stats,
                    workspace,
                    draft,
                    &session_title,
                    modified_files,
                    *files_collapsed,
                    *scroll_back,
                    *show_menu,
                )?;
                return Ok(SubmitOutcome::Continue);
            }
            transcript.push(TranscriptEntry::new(TranscriptRole::Error, e.to_string()));
            let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);
            tui.draw(
                transcript,
                input,
                *cursor_idx,
                *busy,
                *awaiting_clear_confirm,
                &setup.provider_name,
                &setup.model_name,
                stats,
                workspace,
                draft,
                &session_title,
                modified_files,
                *files_collapsed,
                *scroll_back,
                *show_menu,
            )?;
            return Ok(SubmitOutcome::Continue);
        }
    };

    *busy = false;
    draft.clear();
    *final_output = response.output.clone();
    transcript.push(TranscriptEntry::new(TranscriptRole::Assistant, response.output.clone()));
    session_history.push(models::ChatMessage {
        role: models::ChatRole::User,
        content: enriched,
        think_timing: Vec::new(),
    });
    session_history.push(models::ChatMessage {
        role: models::ChatRole::Assistant,
        content: response.output,
        think_timing: Vec::new(),
    });
    stats.assistant_messages += 1;
    stats.input_tokens = stats
        .input_tokens
        .saturating_add(response.usage.input_tokens.max(0).cast_unsigned());
    stats.output_tokens = stats
        .output_tokens
        .saturating_add(response.usage.output_tokens.max(0).cast_unsigned());
    *modified_files = collect_modified_files(&config.workspace_dir);
    *scroll_back = 0;
    stats.tool_events = stats.tool_events.saturating_add(response.step_finishes);
    if !*session_title_refreshed {
        maybe_refresh_cli_session_title(
            session_id,
            &user_input,
            Some(setup.model_name.to_string()),
        )
        .await;
        *session_title_refreshed = true;
    }

    setup.observer.record_event(&crate::app::agent::observability::ObserverEvent::TurnComplete);

    Ok(SubmitOutcome::Continue)
}

fn consume_escape_keypress() -> Result<bool> {
    if !event::poll(std::time::Duration::from_millis(0))? {
        return Ok(false);
    }
    let evt = event::read()?;
    let CrosstermEvent::Key(key) = evt else {
        return Ok(false);
    };
    Ok(key.kind == KeyEventKind::Press && key.code == KeyCode::Esc)
}

/// 用户输入提交的处理结果
///
/// 指示主事件循环在处理完用户输入后应采取的后续动作。
/// 该枚举用于控制 CLI 主循环的控制流。
///
/// # 变体
///
/// - `Continue`: 继续主循环，等待下一次用户输入
/// - `Exit`: 退出主循环，结束 CLI 会话
///
/// # 示例
///
/// ```ignore
/// match outcome {
///     SubmitOutcome::Continue => {
///         // 继续等待用户输入
///     }
///     SubmitOutcome::Exit => {
///         // 执行清理并退出
///         break;
///     }
/// }
/// ```
#[derive(PartialEq, Eq)]
pub(crate) enum SubmitOutcome {
    /// 继续主循环
    ///
    /// 表示当前输入已处理完毕，应继续等待和处理下一用户输入。
    /// 这是最常见的返回值。
    Continue,

    /// 退出主循环
    ///
    /// 表示用户请求退出（如通过 /exit 命令），
    /// 主循环应执行清理操作并终止程序。
    Exit,
}
