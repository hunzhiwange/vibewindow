use crate::app::components::mind_map;
use crate::app::{App, Message};
use crate::apps::mindmap::state::MindMapTab;
use iced::Task;
use iced::widget::text_editor;

use super::super::persist::persist;
use super::node_ops_helpers::push_undo;

/// 提交文本编辑器的内容（如果需要）。
pub(super) fn commit_text_editor_if_needed(tab: &mut MindMapTab) {
    if !tab.show_text_editor {
        return;
    }

    let Some(path) = tab.selected_path.clone() else {
        return;
    };

    let new_text = tab.node_text_editor.text().to_string();
    let Some(cur_text) = mind_map::node_text(&tab.doc, &path).map(|s| s.to_string()) else {
        return;
    };
    if new_text == cur_text {
        return;
    }

    push_undo(tab);
    if let Some(text) = mind_map::node_text_mut(&mut tab.doc, &path) {
        *text = new_text;
    }
}

/// 切换节点文本编辑器的显示状态。
pub(super) fn toggle_node_text_editor(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if tab.show_text_editor {
            commit_text_editor_if_needed(tab);
            tab.show_text_editor = false;
            tab.node_text_editor = text_editor::Content::new();
            tab.canvas_cache.clear();
            let _ = persist(app);
            return Task::none();
        }

        super::node_meta_ops::commit_url_editor_if_needed(tab);
        tab.show_text_editor = true;
        tab.active_color_picker = None;
        tab.show_diagram_type_picker = false;
        tab.show_markdown_import = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_url_editor = false;
        tab.show_action_menu = false;
        tab.show_theme_panel = false;
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::with_text(
            tab.selected_path
                .as_deref()
                .and_then(|path| mind_map::node_text(&tab.doc, path))
                .unwrap_or(""),
        );
    }
    Task::none()
}

/// 处理节点文本变化事件。
pub(super) fn node_text_changed(app: &mut App, value: String) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.node_text_editor = text_editor::Content::with_text(&value);
    }
    Task::none()
}

/// 处理文本编辑器的动作。
pub(super) fn node_text_editor_action(app: &mut App, action: text_editor::Action) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab_mut() else {
        return Task::none();
    };
    if !tab.show_text_editor {
        return Task::none();
    }

    tab.node_text_editor.perform(action);
    Task::none()
}

/// 处理文本编辑器中的回车键。
pub(super) fn node_text_editor_enter(app: &mut App, shift: bool) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab_mut() else {
        return Task::none();
    };
    if !tab.show_text_editor {
        return Task::none();
    }

    if shift {
        tab.node_text_editor.perform(text_editor::Action::Edit(text_editor::Edit::Insert('\n')));
        return Task::none();
    }

    commit_text_editor_if_needed(tab);
    tab.show_text_editor = false;
    tab.node_text_editor = text_editor::Content::new();
    tab.canvas_cache.clear();
    let _ = persist(app);
    Task::none()
}

/// 保存节点文本。
pub(super) fn save_node_text(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        commit_text_editor_if_needed(tab);
        tab.show_text_editor = false;
        tab.node_text_editor = text_editor::Content::new();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}
