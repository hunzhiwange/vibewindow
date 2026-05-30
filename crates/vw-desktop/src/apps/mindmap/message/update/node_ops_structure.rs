use crate::app::components::mind_map;
use crate::app::{App, Message};
use crate::apps::mindmap::state::MindMapDiagramType;
use iced::widget::text_editor;
use iced::{Point, Task, Vector};

use super::super::persist::persist;
use super::node_ops_helpers::{
    close_context_menu_tab, push_undo, relayout_keep_root, shift_on_insert,
    shift_positions_on_insert, shift_set_on_insert,
};

/// 为当前选中的节点添加子节点。
pub(super) fn add_child(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let parent_path = tab.selected_path.clone().unwrap_or_default();
        if tab.collapsed_paths.contains(&parent_path) {
            return Task::none();
        }

        push_undo(tab);

        if let Some(new_path) =
            mind_map::add_child(&mut tab.doc, &parent_path, "新节点".to_string())
        {
            tab.selected_path = Some(new_path);
            relayout_keep_root(tab);
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 为当前选中的节点添加兄弟节点。
pub(super) fn add_sibling(app: &mut App) -> Task<Message> {
    let path = app.active_mindmap_tab_mut().and_then(|tab| tab.selected_path.clone());
    add_sibling_inner(app, path)
}

/// 为指定父节点添加子节点。
pub(super) fn add_child_at(app: &mut App, parent_path: Vec<usize>) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.selected_path = Some(parent_path);
    }
    add_child(app)
}

/// 为指定节点添加兄弟节点。
pub(super) fn add_sibling_at(app: &mut App, path: Vec<usize>) -> Task<Message> {
    add_sibling_inner(app, Some(path))
}

fn add_sibling_inner(app: &mut App, path: Option<Vec<usize>>) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let Some(path) = path.or_else(|| tab.selected_path.clone()) else {
            return Task::none();
        };
        if path.is_empty() {
            return Task::none();
        }
        if tab.collapsed_paths.contains(&path) {
            return Task::none();
        }

        let parent_path = path[..path.len() - 1].to_vec();
        let src_idx = *path.last().unwrap_or(&0);
        let parent_children_len =
            mind_map::node(&tab.doc, &parent_path).map(|n| n.children.len()).unwrap_or(0);
        let insert_at = (src_idx + 1).min(parent_children_len);

        push_undo(tab);

        let shift = match tab.diagram_type {
            MindMapDiagramType::OrgChart => Vector::new(220.0, 0.0),
            MindMapDiagramType::Fishbone => Vector::new(0.0, 0.0),
            MindMapDiagramType::MindMap
            | MindMapDiagramType::Timeline
            | MindMapDiagramType::Tree
            | MindMapDiagramType::Bracket => Vector::new(0.0, 70.0),
        };
        if shift != Vector::new(0.0, 0.0) {
            shift_positions_on_insert(&mut tab.node_positions, &parent_path, insert_at, shift);
        }

        shift_on_insert(&mut tab.node_positions, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_fills, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_text_colors, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_border_colors, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_priorities, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_urls, &parent_path, insert_at);
        shift_on_insert(&mut tab.edge_styles, &parent_path, insert_at);
        shift_on_insert(&mut tab.edge_colors, &parent_path, insert_at);
        shift_set_on_insert(&mut tab.collapsed_paths, &parent_path, insert_at);

        if let Some(new_path) = mind_map::add_sibling(&mut tab.doc, &path, "新节点".to_string())
        {
            tab.selected_path = Some(new_path);
            relayout_keep_root(tab);
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 切换指定节点的折叠状态。
pub(super) fn toggle_collapse_at(app: &mut App, path: Vec<usize>) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);

        let children_len = mind_map::node(&tab.doc, &path).map(|n| n.children.len()).unwrap_or(0);
        if children_len == 0 {
            return Task::none();
        }

        if tab.collapsed_paths.contains(&path) {
            tab.collapsed_paths.remove(&path);
        } else {
            tab.collapsed_paths.insert(path.clone());
            if tab
                .selected_path
                .as_ref()
                .is_some_and(|sel| sel.len() > path.len() && sel.as_slice().starts_with(&path))
            {
                tab.selected_path = Some(path.clone());
            }
        }

        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 打开节点的上下文菜单。
pub(super) fn open_context_menu(app: &mut App, path: Vec<usize>, anchor: Point) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        tab.selected_path = Some(path);
        tab.active_color_picker = None;
        tab.show_markdown_import = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_url_editor = false;
        tab.show_text_editor = false;
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::new();
        tab.show_context_menu = true;
        tab.context_menu_anchor = Some(anchor);
        tab.canvas_cache.clear();
    }
    Task::none()
}

/// 关闭上下文菜单。
pub(super) fn close_context_menu(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        close_context_menu_tab(tab);
        tab.canvas_cache.clear();
    }
    Task::none()
}
