//! 聊天输入处理模块。
//!
//! 保留 `chat::input` 作为外部入口，并将文件搜索、工具详情、任务模式、
//! 右键菜单和只读编辑器交互拆分到独立子模块，避免单文件继续膨胀。

mod clipboard;
mod context_menus;
mod editor_actions;
mod file_search;
mod shared;
mod task_mode;
mod tool_detail;

#[cfg(test)]
mod file_search_tests;
#[cfg(test)]
mod shared_tests;

use super::ChatMessage;
use super::ClipboardPastePayload;
pub(crate) use file_search::{
    FileSearchResult, build_ranked_file_search_entries, ranked_file_search_entries,
};
use crate::app::{App, Message, message};
use iced::Task;
use std::time::Duration;

pub fn update(app: &mut App, message: ChatMessage) -> Task<Message> {
    match message {
        ChatMessage::InputChanged(value) => {
            app.input_text = value;
            Task::none()
        }
        ChatMessage::OpenInputContextMenu { x, y } => {
            context_menus::handle_open_input_context_menu(app, x, y)
        }
        ChatMessage::CloseInputContextMenu => context_menus::handle_close_input_context_menu(app),
        ChatMessage::CopyInputSelection => context_menus::handle_copy_input_selection(app),
        ChatMessage::CutInputSelection => context_menus::handle_cut_input_selection(app),
        ChatMessage::PasteIntoInput => context_menus::handle_paste_into_input(app),
        ChatMessage::ClipboardPasteResolved(payload) => {
            context_menus::handle_clipboard_paste_resolved(app, payload)
        }
        ChatMessage::SelectAllInput => context_menus::handle_select_all_input(app),
        ChatMessage::InputEditorAction(action) => file_search::handle_input_editor_action(app, action),
        ChatMessage::MessageEditorAction(idx, action) => {
            editor_actions::handle_message_editor_action(app, idx, action)
        }
        ChatMessage::SpecialTextEditorAction(msg_idx, text_idx, action) => {
            editor_actions::handle_special_text_editor_action(app, msg_idx, text_idx, action)
        }
        ChatMessage::ToolTextEditorAction(msg_idx, tool_idx, text_idx, action) => {
            editor_actions::handle_tool_text_editor_action(app, msg_idx, tool_idx, text_idx, action)
        }
        ChatMessage::ThinkEditorAction(msg_idx, think_idx, action) => {
            editor_actions::handle_think_editor_action(app, msg_idx, think_idx, action)
        }
        ChatMessage::ScrollChanged { offset_y: rel_y, viewport_h } => {
            let next_offset_y = rel_y.clamp(0.0, 1.0);
            let next_viewport_h = viewport_h.max(0.0);
            let offset_changed = (app.chat_scroll_offset_y - next_offset_y).abs() > 0.0005;
            let viewport_changed = (app.chat_scroll_viewport_h - next_viewport_h).abs() > 0.5;
            let within_hold_window = super::now_ms() < app.chat_autoscroll_hold_until_ms;
            let next_auto_scroll = if within_hold_window && app.chat_auto_scroll {
                next_offset_y >= 0.82
            } else if app.chat_auto_scroll {
                next_offset_y >= 0.90
            } else {
                next_offset_y >= 0.975
            };

            app.chat_scroll_offset_y = next_offset_y;
            app.chat_scroll_viewport_h = next_viewport_h;
            app.chat_auto_scroll = next_auto_scroll;

            if !offset_changed && !viewport_changed {
                return Task::none();
            }

            let (visible_start_idx, visible_end_idx) = app.visible_chat_message_window();
            app.prune_chat_heavy_editor_caches(visible_start_idx, visible_end_idx);
            let pending_chunk_starts =
                app.pending_chat_ui_chunk_starts(visible_start_idx, visible_end_idx, true);

            if !pending_chunk_starts.is_empty()
                && let Some(session_id) = app.active_session_id.clone()
            {
                app.mark_chat_ui_chunks_preparing(&pending_chunk_starts);
                return crate::app::message::project::prepare_session_ui_chunks_task(
                    session_id,
                    app.active_shared_chat_messages(),
                    pending_chunk_starts,
                    None,
                );
            }

            Task::none()
        }
        ChatMessage::ScrollToBottom => super::scroll_chat_to_bottom_task(app),
        ChatMessage::LocateChatMessage(message_id) => shared::locate_chat_message(app, &message_id),
        ChatMessage::LocateChatMessageIndex(target_idx) => {
            shared::locate_chat_message_index(app, target_idx)
        }
        ChatMessage::ToggleFullscreen => {
            app.chat_panel_fullscreen = !app.chat_panel_fullscreen;
            app.chat_panel_half_fullscreen = false;
            app.fullscreen_layout_settling = true;
            if app.chat_panel_fullscreen {
                app.git_diff_fullscreen = false;
                app.git_diff_half_fullscreen = false;
            }
            crate::app::message::after(
                Duration::from_millis(180),
                Message::View(message::ViewMessage::FullscreenLayoutSettled),
            )
        }
        ChatMessage::ToggleHalfFullscreen => {
            app.chat_panel_half_fullscreen = !app.chat_panel_half_fullscreen;
            app.chat_panel_fullscreen = false;
            app.fullscreen_layout_settling = true;
            if app.chat_panel_half_fullscreen {
                app.git_diff_fullscreen = false;
                app.git_diff_half_fullscreen = false;
            }
            crate::app::message::after(
                Duration::from_millis(180),
                Message::View(message::ViewMessage::FullscreenLayoutSettled),
            )
        }
        ChatMessage::FullscreenOverlayEntered => {
            app.show_chat_fullscreen_overlay = true;
            Task::none()
        }
        ChatMessage::FullscreenOverlayExited => {
            app.show_chat_fullscreen_overlay = false;
            Task::none()
        }
        ChatMessage::ToggleThink(msg_idx, think_idx, default_expanded) => {
            let key = ((msg_idx as u64) << 32) | (think_idx as u64);
            let had_expanded_override = app.chat_think_expanded.remove(&key);
            let had_collapsed_override = app.chat_think_collapsed.remove(&key);

            let next_expanded = if had_expanded_override || had_collapsed_override {
                default_expanded
            } else if default_expanded {
                app.chat_think_collapsed.insert(key);
                false
            } else {
                app.chat_think_expanded.insert(key);
                true
            };

            if next_expanded {
                return super::scroll_chat_to_bottom_with_followups(app);
            }

            Task::none()
        }
        ChatMessage::ThinkHover(msg_idx, think_idx) => {
            let key = ((msg_idx as u64) << 32) | (think_idx as u64);
            app.chat_think_hovered_idx = Some(key);
            Task::none()
        }
        ChatMessage::ThinkHoverLeave => {
            app.chat_think_hovered_idx = None;
            Task::none()
        }
        ChatMessage::ToggleToolFile(msg_idx, tool_idx, path) => {
            let key = format!("{msg_idx}:{tool_idx}:{path}");
            if !app.chat_tool_file_expanded.remove(&key) {
                app.chat_tool_file_expanded.insert(key);
            }
            Task::none()
        }
        ChatMessage::ToggleTool(msg_idx, tool_idx) => {
            let key = ((msg_idx as u64) << 32) | (tool_idx as u64);
            if !app.chat_tool_expanded.remove(&key) {
                app.chat_tool_expanded.insert(key);
            }
            Task::none()
        }
        ChatMessage::ToolFileHover(key) => {
            app.chat_tool_file_hovered = Some(key);
            Task::none()
        }
        ChatMessage::ToolFileHoverLeave => {
            app.chat_tool_file_hovered = None;
            Task::none()
        }
        ChatMessage::ToolHover(msg_idx, tool_idx) => {
            let key = ((msg_idx as u64) << 32) | (tool_idx as u64);
            app.chat_tool_hovered_idx = Some(key);
            Task::none()
        }
        ChatMessage::ToolHoverLeave => {
            app.chat_tool_hovered_idx = None;
            app.chat_tool_file_hovered = None;
            Task::none()
        }
        ChatMessage::ToggleExploreSummary(msg_idx, tool_idx) => {
            let key = ((msg_idx as u64) << 32) | (tool_idx as u64);
            if !app.chat_explore_expanded.remove(&key) {
                app.chat_explore_expanded.insert(key);
            }
            Task::none()
        }
        ChatMessage::OpenToolDetail(msg_idx, tool_idx, raw) => {
            tool_detail::handle_open_tool_detail(app, msg_idx, tool_idx, raw)
        }
        ChatMessage::ToolDetailEditorAction(action) => {
            tool_detail::handle_tool_detail_editor_action(app, action)
        }
        ChatMessage::ToolDetailOpenContextMenu { x, y } => {
            tool_detail::handle_tool_detail_open_context_menu(app, x, y)
        }
        ChatMessage::ToolDetailCloseContextMenu => {
            tool_detail::handle_tool_detail_close_context_menu(app)
        }
        ChatMessage::ToolDetailContextMenuCopy => {
            tool_detail::handle_tool_detail_context_menu_copy(app)
        }
        ChatMessage::ToolDetailContextMenuCut => {
            tool_detail::handle_tool_detail_context_menu_cut(app)
        }
        ChatMessage::ToolDetailContextMenuPaste => {
            tool_detail::handle_tool_detail_context_menu_paste(app)
        }
        ChatMessage::ToolDetailContextMenuDelete => {
            tool_detail::handle_tool_detail_context_menu_delete(app)
        }
        ChatMessage::ToolDetailEditorWheelScrolled { delta, viewport_height } => {
            tool_detail::handle_tool_detail_editor_wheel_scrolled(app, delta, viewport_height)
        }
        ChatMessage::ToolDetailScrollbarChanged { top_line, viewport_height } => {
            tool_detail::handle_tool_detail_scrollbar_changed(app, top_line, viewport_height)
        }
        ChatMessage::CloseToolDetail => tool_detail::handle_close_tool_detail(app),
        ChatMessage::ToggleTodoPanel => {
            app.chat_todo_expanded = !app.chat_todo_expanded;
            if (app.chat_todo_anim - if app.chat_todo_expanded { 1.0 } else { 0.0 }).abs() < 0.001 {
                app.chat_todo_anim = if app.chat_todo_expanded { 1.0 } else { 0.0 };
                Task::none()
            } else {
                message::after(Duration::from_millis(16), Message::Chat(ChatMessage::TodoAnimTick))
            }
        }
        ChatMessage::TodoAnimTick => {
            let target = if app.chat_todo_expanded { 1.0 } else { 0.0 };
            let current = app.chat_todo_anim;
            let next = current + (target - current) * 0.22;
            let done = (next - target).abs() < 0.005;
            app.chat_todo_anim = if done { target } else { next.clamp(0.0, 1.0) };

            if done {
                Task::none()
            } else {
                message::after(Duration::from_millis(16), Message::Chat(ChatMessage::TodoAnimTick))
            }
        }
        ChatMessage::FileSearchInputChanged(value) => {
            file_search::handle_file_search_input_changed(app, value)
        }
        ChatMessage::FileSearchNavigateUp => file_search::handle_file_search_navigate_up(app),
        ChatMessage::FileSearchNavigateDown => file_search::handle_file_search_navigate_down(app),
        ChatMessage::FileSearchSelectCurrent => file_search::handle_file_search_select_current(app),
        ChatMessage::FileSearchSelect(path) => file_search::handle_file_search_select(app, path),
        ChatMessage::RemoveFileReference(file_path) => {
            file_search::handle_remove_file_reference(app, file_path)
        }
        ChatMessage::FileReferenceHoverChanged(value) => {
            app.file_ref_hovered_index = value;
            Task::none()
        }
        ChatMessage::ToolFilesFilterChanged(value) => {
            app.tool_files_filter = value;
            Task::none()
        }
        ChatMessage::TaskModeToggled(value) => task_mode::handle_task_mode_toggled(app, value),
        ChatMessage::TaskModePriorityChanged(value) => {
            task_mode::handle_task_mode_priority_changed(app, value)
        }
        ChatMessage::TaskModeModelChanged(model) => {
            task_mode::handle_task_mode_model_changed(app, model)
        }
        ChatMessage::TaskModeExecutorChanged(executor) => {
            task_mode::handle_task_mode_executor_changed(app, executor)
        }
        ChatMessage::TaskModeSubtaskChanged { index, value } => {
            task_mode::handle_task_mode_subtask_changed(app, index, value)
        }
        ChatMessage::TaskModeSubtaskEditorAction { index, action } => {
            task_mode::handle_task_mode_subtask_editor_action(app, index, action)
        }
        ChatMessage::TaskModeAddSubtask => task_mode::handle_task_mode_add_subtask(app),
        ChatMessage::TaskModeRemoveSubtask(index) => {
            task_mode::handle_task_mode_remove_subtask(app, index)
        }
        ChatMessage::TaskModeMoveSubtaskUp(index) => {
            task_mode::handle_task_mode_move_subtask_up(app, index)
        }
        ChatMessage::TaskModeMoveSubtaskDown(index) => {
            task_mode::handle_task_mode_move_subtask_down(app, index)
        }
        ChatMessage::InputAreaDragDrop => file_search::handle_input_area_drag_drop(app),
        ChatMessage::InputAreaDragHoverChanged(value) => {
            app.input_drop_hovered = value;
            Task::none()
        }
        ChatMessage::OpenMessageContextMenu { target, x, y, text } => {
            context_menus::handle_open_message_context_menu(app, target, x, y, text)
        }
        ChatMessage::CloseMessageContextMenu => {
            context_menus::handle_close_message_context_menu(app)
        }
        ChatMessage::ToggleResetMenu(msg_idx) => context_menus::handle_toggle_reset_menu(app, msg_idx),
        ChatMessage::CloseResetMenu => context_menus::handle_close_reset_menu(app),
        ChatMessage::CopyContextMenuText => context_menus::handle_copy_context_menu_text(app),
        ChatMessage::AppendContextMenuText => context_menus::handle_append_context_menu_text(app),
        ChatMessage::SearchContextMenuWithBaidu => {
            context_menus::handle_search_context_menu_with_baidu(app)
        }
        ChatMessage::SearchContextMenuWithGoogle => {
            context_menus::handle_search_context_menu_with_google(app)
        }
        ChatMessage::SearchContextMenuWithBing => {
            context_menus::handle_search_context_menu_with_bing(app)
        }
        _ => Task::none(),
    }
}
