use crate::app::{App, Message};
use iced::Task;

use super::super::persist::persist;
use super::node_ops_helpers::close_context_menu_tab;

/// 执行撤销操作。
pub(super) fn undo_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        close_context_menu_tab(tab);
        if let Some(prev) = tab.undo_stack.pop() {
            tab.redo_stack.push(tab.doc.clone());
            tab.doc = prev;
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 执行重做操作。
pub(super) fn redo_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        close_context_menu_tab(tab);
        if let Some(next) = tab.redo_stack.pop() {
            tab.undo_stack.push(tab.doc.clone());
            tab.doc = next;
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}
