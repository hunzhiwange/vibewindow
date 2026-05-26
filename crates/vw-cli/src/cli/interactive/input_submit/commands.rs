//! CLI 交互式模式命令处理模块
//!
//! 本模块提供了 CLI 交互式模式下的命令解析与处理功能，包括：
//! - 内联命令（如 `/help`、`/quit`、`/clear` 等）的识别与执行
//! - 会话清除确认流程的处理
//!
//! # 架构位置
//!
//! 本模块位于 CLI 交互式输入提交流程的命令处理层，负责：
//! 1. 解析用户输入的内联命令（以 `/` 开头）
//! 2. 执行相应的命令逻辑
//! 3. 更新会话状态和 UI 显示
//!
//! # 支持的命令
//!
//! | 命令      | 别名    | 功能说明                     |
//! |-----------|---------|------------------------------|
//! | `/help`   | -       | 显示可用命令的帮助信息       |
//! | `/quit`   | `/exit` | 退出交互式模式               |
//! | `/clear`  | `/new`  | 清除当前会话历史和内存       |
//! | `/files`  | -       | 切换文件面板的显示/隐藏状态  |

use crate::app::agent::config::Config;
use crate::app::agent::memory::MemoryCategory;
use anyhow::Result;

use super::super::super::session::{collect_modified_files, create_cli_session};
use super::super::super::setup::CliSetup;
use super::super::super::stats::build_session_title;
use super::super::super::transcript::{TranscriptEntry, TranscriptRole};
use super::super::super::tui::CliTui;
use super::flow::SubmitOutcome;

/// 处理等待中的会话清除确认
///
/// 当用户执行 `/clear` 或 `/new` 命令后，系统会进入等待确认状态。
/// 本函数处理用户的确认或取消响应，并在确认时执行完整的会话清除操作。
///
/// # 参数
///
/// * `config` - 代理配置，包含工作区目录等设置
/// * `setup` - CLI 设置，包含内存存储和模型信息
/// * `user_input` - 用户输入的确认响应（期望 "y" 或 "yes" 进行确认）
/// * `tui` - TUI 渲染器，用于更新界面显示
/// * `transcript` - 会话记录，用于添加系统消息
/// * `session_history` - 聊天消息历史，在确认时会被清空
/// * `session_id` - 当前会话 ID，在确认时会被重置为新 ID
/// * `session_title_refreshed` - 会话标题刷新标记，重置后需要重新生成标题
/// * `cursor_idx` - 光标索引，用于 UI 渲染
/// * `busy` - 忙碌状态标记，用于 UI 渲染
/// * `awaiting_clear_confirm` - 等待清除确认的标记，处理后会被重置为 false
/// * `stats` - CLI 统计信息，用于生成会话标题
/// * `workspace` - 工作区路径，用于 UI 显示
/// * `modified_files` - 已修改文件列表，清除后重新收集
/// * `files_collapsed` - 文件面板折叠状态，用于 UI 渲染
/// * `draft` - 草稿内容，用于 UI 渲染
/// * `scroll_back` - 滚动回退量，用于 UI 渲染
/// * `show_menu` - 菜单显示状态，用于 UI 渲染
///
/// # 返回值
///
/// 返回 `Result<SubmitOutcome>`，始终返回 `SubmitOutcome::Continue` 以继续交互循环。
/// 错误情况可能源于 TUI 渲染或内存操作失败。
///
/// # 清除操作详情
///
/// 确认时（用户输入 "y" 或 "yes"，不区分大小写）会执行：
/// 1. 清空聊天消息历史
/// 2. 创建新的会话 ID
/// 3. 从内存中删除 `Conversation` 和 `Daily` 类别的所有条目
/// 4. 重新收集工作区中的修改文件列表
///
/// # 示例
///
/// ```ignore
/// // 用户在清除确认提示后输入 "y"
/// let outcome = handle_pending_clear(
///     &config,
///     &setup,
///     "y",
///     &mut tui,
///     &mut transcript,
///     &mut session_history,
///     &mut session_id,
///     &mut session_title_refreshed,
///     cursor_idx,
///     busy,
///     &mut awaiting_clear_confirm,
///     &mut stats,
///     workspace,
///     &mut modified_files,
///     &mut files_collapsed,
///     draft,
///     scroll_back,
///     show_menu,
/// ).await?;
/// ```
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_pending_clear(
    config: &Config,
    setup: &CliSetup,
    user_input: &str,
    tui: &mut CliTui,
    transcript: &mut Vec<TranscriptEntry>,
    session_history: &mut Vec<crate::session::ui_types::ChatMessage>,
    session_id: &mut String,
    session_title_refreshed: &mut bool,
    cursor_idx: usize,
    busy: bool,
    awaiting_clear_confirm: &mut bool,
    stats: &mut super::super::super::stats::CliStats,
    workspace: &str,
    modified_files: &mut Vec<String>,
    files_collapsed: &mut bool,
    draft: &str,
    scroll_back: u16,
    show_menu: bool,
) -> Result<SubmitOutcome> {
    // 重置等待确认状态，无论用户如何响应
    *awaiting_clear_confirm = false;

    // 检查用户是否确认清除（接受 "y" 或 "yes"，不区分大小写）
    if matches!(user_input.to_lowercase().as_str(), "y" | "yes") {
        // === 执行会话清除操作 ===

        // 清空聊天消息历史
        session_history.clear();

        // 创建新的会话 ID，使用当前工作目录作为会话上下文
        *session_id = create_cli_session(
            &std::env::current_dir().unwrap_or_else(|_| config.workspace_dir.clone()),
            None,
        )
        .await;

        // 标记会话标题需要重新生成
        *session_title_refreshed = false;

        // === 清除内存中的会话相关条目 ===
        // 遍历 Conversation 和 Daily 两个类别的内存条目并删除
        // 注意：Core（核心）类别的内存会被保留
        let mut cleared = 0;
        for category in [MemoryCategory::Conversation, MemoryCategory::Daily] {
            // 获取该类别下的所有内存条目
            let entries = setup.mem.list(Some(&category), None).await.unwrap_or_default();

            // 逐个删除条目，统计成功删除的数量
            for entry in entries {
                if setup.mem.forget(&entry.key).await.unwrap_or(false) {
                    cleared += 1;
                }
            }
        }

        // 向会话记录添加清除结果反馈
        if cleared > 0 {
            transcript.push(TranscriptEntry::new(
                TranscriptRole::System,
                format!("会话已清除 ({cleared} 条记忆条目已移除)"),
            ));
        } else {
            transcript.push(TranscriptEntry::new(TranscriptRole::System, "Conversation cleared"));
        }
    } else {
        // 用户取消清除操作
        transcript.push(TranscriptEntry::new(TranscriptRole::System, "Cancelled"));
    }

    // === 更新 UI 状态 ===

    // 重新收集工作区中的修改文件列表
    *modified_files = collect_modified_files(&config.workspace_dir);

    // 生成当前会话标题
    let session_title = build_session_title(stats, &setup.provider_name, &setup.model_name);

    // 重绘 TUI 界面以反映状态变化
    tui.draw(
        transcript,
        "",
        cursor_idx,
        busy,
        *awaiting_clear_confirm,
        &setup.provider_name,
        &setup.model_name,
        stats,
        workspace,
        draft,
        &session_title,
        modified_files,
        *files_collapsed,
        scroll_back,
        show_menu,
    )?;

    Ok(SubmitOutcome::Continue)
}

/// 处理内联命令
///
/// 解析并执行以 `/` 开头的内联命令。支持的命令包括：
/// - `/quit` / `/exit`：退出交互式模式
/// - `/help`：显示可用命令的帮助信息
/// - `/files`：切换文件面板的显示/隐藏
/// - `/clear` / `/new`：启动会话清除确认流程
///
/// # 参数
///
/// * `user_input` - 用户输入的命令字符串（应已去除首尾空白）
/// * `tui` - TUI 渲染器，用于更新界面显示
/// * `transcript` - 会话记录，用于添加系统消息（如帮助文本）
/// * `cursor_idx` - 光标索引，用于 UI 渲染
/// * `busy` - 忙碌状态标记，用于 UI 渲染
/// * `awaiting_clear_confirm` - 等待清除确认的标记，`/clear` 命令会将其设为 true
/// * `stats` - CLI 统计信息，用于生成会话标题
/// * `workspace` - 工作区路径，用于 UI 显示
/// * `modified_files` - 已修改文件列表，用于 UI 渲染
/// * `files_collapsed` - 文件面板折叠状态，`/files` 命令会切换此状态
/// * `draft` - 草稿内容，用于 UI 渲染
/// * `scroll_back` - 滚动回退量，用于 UI 渲染
/// * `show_menu` - 菜单显示状态，用于 UI 渲染
/// * `provider_name` - 当前使用的提供者名称，用于 UI 显示
/// * `model_name` - 当前使用的模型名称，用于 UI 显示
///
/// # 返回值
///
/// 返回 `Result<Option<SubmitOutcome>>`：
/// - `Ok(Some(SubmitOutcome::Exit))` - 用户执行了退出命令
/// - `Ok(Some(SubmitOutcome::Continue))` - 命令已处理，继续交互循环
/// - `Ok(None)` - 输入不是已知命令，应由调用者继续处理
/// - `Err(...)` - TUI 渲染或其他操作失败
///
/// # 命令处理流程
///
/// 1. 匹配命令字符串
/// 2. 执行对应的命令逻辑（如切换状态、添加消息等）
/// 3. 更新 TUI 显示
/// 4. 返回适当的 `SubmitOutcome`
///
/// # 示例
///
/// ```ignore
/// // 处理 /help 命令
/// let result = handle_inline_command(
///     "/help",
///     &mut tui,
///     &mut transcript,
///     cursor_idx,
///     busy,
///     &mut awaiting_clear_confirm,
///     &mut stats,
///     workspace,
///     &mut modified_files,
///     &mut files_collapsed,
///     draft,
///     scroll_back,
///     show_menu,
///     "openai",
///     "gpt-4",
/// )?;
///
/// match result {
///     Some(SubmitOutcome::Continue) => println!("命令已处理"),
///     Some(SubmitOutcome::Exit) => println!("用户请求退出"),
///     None => println!("不是已知命令"),
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_inline_command(
    user_input: &str,
    tui: &mut CliTui,
    transcript: &mut Vec<TranscriptEntry>,
    cursor_idx: usize,
    busy: bool,
    awaiting_clear_confirm: &mut bool,
    stats: &mut super::super::super::stats::CliStats,
    workspace: &str,
    modified_files: &[String],
    files_collapsed: &mut bool,
    draft: &str,
    scroll_back: u16,
    show_menu: bool,
    provider_name: &str,
    model_name: &str,
) -> Result<Option<SubmitOutcome>> {
    match user_input {
        // 退出命令：立即返回退出信号
        "/quit" | "/exit" => return Ok(Some(SubmitOutcome::Exit)),

        // 帮助命令：向会话记录添加帮助文本
        "/help" => {
            transcript.push(TranscriptEntry::new(
                TranscriptRole::System,
                "命令：\n  /help         显示此帮助消息\n  /files        切换文件面板\n  /clear /new   清除会话历史\n  /quit /exit   退出交互式模式",
            ));
        }

        // 文件面板切换：反转折叠状态
        "/files" => {
            *files_collapsed = !*files_collapsed;
        }

        // 清除会话：进入确认流程
        "/clear" | "/new" => {
            *awaiting_clear_confirm = true;
            // 添加确认提示消息
            transcript.push(TranscriptEntry::new(
                TranscriptRole::System,
                "确认清除当前会话和会话内存（核心记忆将保留）？",
            ));
            transcript.push(TranscriptEntry::new(TranscriptRole::System, "确认请输入 y/yes"));
        }

        // 非命令输入：返回 None 让调用者继续处理
        _ => return Ok(None),
    }

    // 生成会话标题并更新 TUI 显示
    let session_title = build_session_title(stats, provider_name, model_name);
    tui.draw(
        transcript,
        "",
        cursor_idx,
        busy,
        *awaiting_clear_confirm,
        provider_name,
        model_name,
        stats,
        workspace,
        draft,
        &session_title,
        modified_files,
        *files_collapsed,
        scroll_back,
        show_menu,
    )?;

    // 命令已处理，返回 Continue 以继续交互循环
    Ok(Some(SubmitOutcome::Continue))
}
