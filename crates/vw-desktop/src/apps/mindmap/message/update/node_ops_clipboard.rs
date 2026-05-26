use crate::app::components::mind_map;
use crate::app::{App, Message};
use iced::{Point, Task};
use std::collections::HashMap;

use super::super::persist::persist;
use super::node_ops_helpers::{
    close_context_menu_tab, push_undo, relayout_keep_root, remove_prefix, remove_prefix_set,
    shift_on_delete, shift_on_insert, shift_set_on_delete, shift_set_on_insert,
};

/// 复制当前选中的节点到剪贴板。
pub(super) fn copy_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        if let Some(path) = tab.selected_path.clone()
            && let Some(node) = mind_map::node(&tab.doc, &path).cloned()
        {
            tab.clipboard_node = Some(node);
        }
    }
    Task::none()
}

/// 剪切当前选中的节点。
pub(super) fn cut_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let Some(path) = tab.selected_path.clone() else {
            return Task::none();
        };
        if path.is_empty() {
            return Task::none();
        }

        let parent_path = path[..path.len() - 1].to_vec();
        let removed_idx = *path.last().unwrap_or(&0);

        push_undo(tab);

        let removed_path = path.clone();
        remove_prefix(&mut tab.node_positions, &removed_path);
        remove_prefix(&mut tab.node_fills, &removed_path);
        remove_prefix(&mut tab.node_text_colors, &removed_path);
        remove_prefix(&mut tab.node_border_colors, &removed_path);
        remove_prefix(&mut tab.node_priorities, &removed_path);
        remove_prefix(&mut tab.node_urls, &removed_path);
        remove_prefix(&mut tab.edge_styles, &removed_path);
        remove_prefix(&mut tab.edge_colors, &removed_path);
        remove_prefix_set(&mut tab.collapsed_paths, &removed_path);

        shift_on_delete(&mut tab.node_positions, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_fills, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_text_colors, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_border_colors, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_priorities, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_urls, &parent_path, removed_idx);
        shift_on_delete(&mut tab.edge_styles, &parent_path, removed_idx);
        shift_on_delete(&mut tab.edge_colors, &parent_path, removed_idx);
        shift_set_on_delete(&mut tab.collapsed_paths, &parent_path, removed_idx);

        if let Some(node) = mind_map::take_node(&mut tab.doc, &path) {
            tab.clipboard_node = Some(node);
            tab.selected_path = Some(parent_path);
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 删除当前选中的节点。
pub(super) fn delete_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let Some(path) = tab.selected_path.clone() else {
            return Task::none();
        };
        if path.is_empty() {
            return Task::none();
        }

        let parent_path = path[..path.len() - 1].to_vec();
        let removed_idx = *path.last().unwrap_or(&0);

        push_undo(tab);

        let removed_path = path.clone();
        remove_prefix(&mut tab.node_positions, &removed_path);
        remove_prefix(&mut tab.node_fills, &removed_path);
        remove_prefix(&mut tab.node_text_colors, &removed_path);
        remove_prefix(&mut tab.node_border_colors, &removed_path);
        remove_prefix(&mut tab.node_priorities, &removed_path);
        remove_prefix(&mut tab.node_urls, &removed_path);
        remove_prefix(&mut tab.edge_styles, &removed_path);
        remove_prefix(&mut tab.edge_colors, &removed_path);
        remove_prefix_set(&mut tab.collapsed_paths, &removed_path);

        shift_on_delete(&mut tab.node_positions, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_fills, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_text_colors, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_border_colors, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_priorities, &parent_path, removed_idx);
        shift_on_delete(&mut tab.node_urls, &parent_path, removed_idx);
        shift_on_delete(&mut tab.edge_styles, &parent_path, removed_idx);
        shift_on_delete(&mut tab.edge_colors, &parent_path, removed_idx);
        shift_set_on_delete(&mut tab.collapsed_paths, &parent_path, removed_idx);

        if let Some(parent) = mind_map::delete_node(&mut tab.doc, &path) {
            tab.selected_path = Some(parent);
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 粘贴剪贴板中的节点。
pub(super) fn paste_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let Some(parent_path) = tab.selected_path.clone() else {
            return Task::none();
        };
        let Some(node) = tab.clipboard_node.clone() else {
            return Task::none();
        };

        push_undo(tab);
        if let Some(new_path) = mind_map::insert_child_node(&mut tab.doc, &parent_path, node) {
            tab.selected_path = Some(new_path);
            relayout_keep_root(tab);
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}

/// 复制当前选中的节点。
pub(super) fn duplicate_node(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        close_context_menu_tab(tab);
        let Some(src_path) = tab.selected_path.clone() else {
            return Task::none();
        };
        if src_path.is_empty() {
            return Task::none();
        }
        let Some(src_node) = mind_map::node(&tab.doc, &src_path).cloned() else {
            return Task::none();
        };

        let parent_path = src_path[..src_path.len() - 1].to_vec();
        let src_idx = *src_path.last().unwrap_or(&0);
        let parent_children_len =
            mind_map::node(&tab.doc, &parent_path).map(|n| n.children.len()).unwrap_or(0);
        let insert_at = (src_idx + 1).min(parent_children_len);

        push_undo(tab);
        shift_on_insert(&mut tab.node_positions, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_fills, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_text_colors, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_border_colors, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_priorities, &parent_path, insert_at);
        shift_on_insert(&mut tab.node_urls, &parent_path, insert_at);
        shift_on_insert(&mut tab.edge_styles, &parent_path, insert_at);
        shift_on_insert(&mut tab.edge_colors, &parent_path, insert_at);
        shift_set_on_insert(&mut tab.collapsed_paths, &parent_path, insert_at);

        let Some(new_path) = mind_map::insert_sibling_node(&mut tab.doc, &src_path, src_node) else {
            return Task::none();
        };

        let offset = Point::new(40.0, 40.0);
        let src_root_pos = tab.node_positions.get(&src_path).copied();
        if let Some(pos) = src_root_pos {
            tab.node_positions
                .insert(new_path.clone(), Point::new(pos.x + offset.x, pos.y + offset.y));
        }

        let mut copy_subtree_pos = Vec::new();
        for (path, pos) in &tab.node_positions {
            if path.as_slice().starts_with(&src_path) && *path != src_path {
                let suffix = &path[src_path.len()..];
                let mut new_key = new_path.clone();
                new_key.extend_from_slice(suffix);
                copy_subtree_pos.push((new_key, *pos));
            }
        }
        for (path, pos) in copy_subtree_pos {
            tab.node_positions
                .entry(path)
                .or_insert(Point::new(pos.x + offset.x, pos.y + offset.y));
        }

        let copy_meta_u32 = |src: &HashMap<Vec<usize>, u32>, dst: &mut HashMap<Vec<usize>, u32>| {
            for (path, value) in src {
                if path.as_slice().starts_with(&src_path) {
                    let suffix = &path[src_path.len()..];
                    let mut new_key = new_path.clone();
                    new_key.extend_from_slice(suffix);
                    dst.insert(new_key, *value);
                }
            }
        };
        let copy_meta_u8 = |src: &HashMap<Vec<usize>, u8>, dst: &mut HashMap<Vec<usize>, u8>| {
            for (path, value) in src {
                if path.as_slice().starts_with(&src_path) {
                    let suffix = &path[src_path.len()..];
                    let mut new_key = new_path.clone();
                    new_key.extend_from_slice(suffix);
                    dst.insert(new_key, *value);
                }
            }
        };
        let copy_meta_string =
            |src: &HashMap<Vec<usize>, String>, dst: &mut HashMap<Vec<usize>, String>| {
                for (path, value) in src {
                    if path.as_slice().starts_with(&src_path) {
                        let suffix = &path[src_path.len()..];
                        let mut new_key = new_path.clone();
                        new_key.extend_from_slice(suffix);
                        dst.insert(new_key, value.clone());
                    }
                }
            };
        let copy_meta_edge_style = |src: &HashMap<
            Vec<usize>,
            crate::apps::mindmap::state::EdgeStyle,
        >,
                                    dst: &mut HashMap<
            Vec<usize>,
            crate::apps::mindmap::state::EdgeStyle,
        >| {
            for (path, value) in src {
                if path.as_slice().starts_with(&src_path) {
                    let suffix = &path[src_path.len()..];
                    let mut new_key = new_path.clone();
                    new_key.extend_from_slice(suffix);
                    dst.insert(new_key, *value);
                }
            }
        };

        copy_meta_u32(&tab.node_fills.clone(), &mut tab.node_fills);
        copy_meta_u32(&tab.node_text_colors.clone(), &mut tab.node_text_colors);
        copy_meta_u32(&tab.node_border_colors.clone(), &mut tab.node_border_colors);
        copy_meta_u8(&tab.node_priorities.clone(), &mut tab.node_priorities);
        copy_meta_string(&tab.node_urls.clone(), &mut tab.node_urls);
        copy_meta_edge_style(&tab.edge_styles.clone(), &mut tab.edge_styles);
        copy_meta_u32(&tab.edge_colors.clone(), &mut tab.edge_colors);

        let mut copy_collapsed = Vec::new();
        for path in &tab.collapsed_paths {
            if path.as_slice().starts_with(&src_path) {
                let suffix = &path[src_path.len()..];
                let mut new_key = new_path.clone();
                new_key.extend_from_slice(suffix);
                copy_collapsed.push(new_key);
            }
        }
        for path in copy_collapsed {
            tab.collapsed_paths.insert(path);
        }

        tab.selected_path = Some(new_path);
        relayout_keep_root(tab);
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}
