//! 处理交互式 CLI 的终端事件。
//! 模块将键盘输入、滚动和提交行为集中在事件层，避免污染渲染状态。

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use super::super::interactive_input::{InputEditResult, apply_key_to_input, insert_newline};
use super::super::stats::{CliStats, build_session_title};
use super::super::transcript::{TranscriptEntry, TranscriptRole};
use super::super::tui::{CliTui, MouseAction};

/// 执行 poll_and_tick 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn poll_and_tick(
    tui: &mut CliTui,
    busy: bool,
    transcript: &[TranscriptEntry],
    input: &str,
    cursor_idx: usize,
    awaiting_clear_confirm: bool,
    stats: &CliStats,
    workspace: &str,
    draft: &str,
    provider_name: &str,
    model_name: &str,
    modified_files: &[String],
    files_collapsed: bool,
    scroll_back: u16,
    show_menu: bool,
) -> Result<bool> {
    if !crossterm::event::poll(std::time::Duration::from_millis(100))? {
        if busy {
            tui.tick();
        }
        let session_title = build_session_title(stats, provider_name, model_name);
        tui.draw(
            transcript,
            input,
            cursor_idx,
            busy,
            awaiting_clear_confirm,
            provider_name,
            model_name,
            stats,
            workspace,
            draft,
            &session_title,
            modified_files,
            files_collapsed,
            scroll_back,
            show_menu,
        )?;
        return Ok(false);
    }
    Ok(true)
}

/// 执行 handle_resize_event 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn handle_resize_event(
    tui: &mut CliTui,
    transcript: &[TranscriptEntry],
    input: &str,
    cursor_idx: usize,
    busy: bool,
    awaiting_clear_confirm: bool,
    stats: &CliStats,
    workspace: &str,
    draft: &str,
    provider_name: &str,
    model_name: &str,
    modified_files: &[String],
    files_collapsed: bool,
    scroll_back: u16,
    show_menu: bool,
) -> Result<()> {
    tui.invalidate_render_cache();
    let session_title = build_session_title(stats, provider_name, model_name);
    tui.draw(
        transcript,
        input,
        cursor_idx,
        busy,
        awaiting_clear_confirm,
        provider_name,
        model_name,
        stats,
        workspace,
        draft,
        &session_title,
        modified_files,
        files_collapsed,
        scroll_back,
        show_menu,
    )?;
    Ok(())
}

/// 执行 handle_mouse_event 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn handle_mouse_event(
    tui: &mut CliTui,
    mouse: MouseEvent,
    scroll_back: &mut u16,
    transcript: &[TranscriptEntry],
    input: &str,
    cursor_idx: usize,
    busy: bool,
    awaiting_clear_confirm: bool,
    stats: &CliStats,
    workspace: &str,
    draft: &str,
    provider_name: &str,
    model_name: &str,
    modified_files: &[String],
    files_collapsed: bool,
    show_menu: bool,
) -> Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            *scroll_back = scroll_back.saturating_add(3);
            tui.invalidate_render_cache();
        }
        MouseEventKind::ScrollDown => {
            *scroll_back = scroll_back.saturating_sub(3);
            tui.invalidate_render_cache();
        }
        MouseEventKind::Down(crossterm::event::MouseButton::Left)
        | MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            match tui.resolve_mouse_action(mouse.column, mouse.row, *scroll_back) {
                MouseAction::ToggleToolDetails | MouseAction::ToggleThinkDetails => {
                    tui.invalidate_render_cache();
                }
                MouseAction::SetScrollBack(new_back) => {
                    *scroll_back = new_back;
                    tui.invalidate_render_cache();
                }
                MouseAction::None => {}
            }
        }
        _ => {}
    }
    if tui.last_render_hash.is_none() {
        let session_title = build_session_title(stats, provider_name, model_name);
        tui.draw(
            transcript,
            input,
            cursor_idx,
            busy,
            awaiting_clear_confirm,
            provider_name,
            model_name,
            stats,
            workspace,
            draft,
            &session_title,
            modified_files,
            files_collapsed,
            *scroll_back,
            show_menu,
        )?;
    }
    Ok(())
}

/// 执行 handle_key_press 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) fn handle_key_press(
    tui: &mut CliTui,
    key: KeyEvent,
    transcript: &mut Vec<TranscriptEntry>,
    input: &mut String,
    cursor_idx: &mut usize,
    scroll_back: &mut u16,
    show_menu: &mut bool,
    exit_confirm_armed: &mut bool,
) -> Option<String> {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c' | 'd'))
    {
        if *exit_confirm_armed {
            return Some("/exit".to_string());
        }
        *exit_confirm_armed = true;
        transcript.push(TranscriptEntry::new(
            TranscriptRole::System,
            "再按一次 Ctrl+C 退出（按任意其他键取消）",
        ));
        tui.invalidate_render_cache();
        return None;
    }

    if *exit_confirm_armed {
        *exit_confirm_armed = false;
        tui.invalidate_render_cache();
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('p')) {
        *show_menu = !*show_menu;
        tui.invalidate_render_cache();
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('t')) {
        tui.expand_tool_blocks = !tui.expand_tool_blocks;
        tui.invalidate_render_cache();
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('y')) {
        tui.expand_think_all = !tui.expand_think_all;
        tui.invalidate_render_cache();
        return None;
    }

    match key.code {
        KeyCode::Up => {
            *scroll_back = scroll_back.saturating_add(1);
        }
        KeyCode::Down => {
            *scroll_back = scroll_back.saturating_sub(1);
        }
        KeyCode::PageUp => {
            *scroll_back = scroll_back.saturating_add(5);
        }
        KeyCode::PageDown => {
            *scroll_back = scroll_back.saturating_sub(5);
        }
        KeyCode::Char('j')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::ALT) =>
        {
            insert_newline(input, cursor_idx);
        }
        KeyCode::Enter => {
            if key
                .modifiers
                .intersects(KeyModifiers::SHIFT | KeyModifiers::ALT | KeyModifiers::CONTROL)
            {
                insert_newline(input, cursor_idx);
                return None;
            }
            if !key.modifiers.is_empty() {
                return None;
            }
            let user_input = input.trim().to_string();
            input.clear();
            *cursor_idx = 0;
            return Some(user_input);
        }
        KeyCode::Tab => {}
        _ => {
            if matches!(apply_key_to_input(input, cursor_idx, key.code), InputEditResult::Updated) {
                return None;
            }
        }
    }
    None
}
