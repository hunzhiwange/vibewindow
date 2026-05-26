use crate::apps::mindmap::canvas::layout::compute_layout_for_diagram;
use crate::apps::mindmap::state::{MindMapLayoutFormat, MindMapTab};
use iced::{Point, Vector};
use std::collections::{HashMap, HashSet};

/// 从 HashMap 中移除所有以指定前缀开头的键。
pub(super) fn remove_prefix<V>(map: &mut HashMap<Vec<usize>, V>, prefix: &[usize]) {
    map.retain(|k, _| !k.as_slice().starts_with(prefix));
}

/// 从 HashSet 中移除所有以指定前缀开头的元素。
pub(super) fn remove_prefix_set(set: &mut HashSet<Vec<usize>>, prefix: &[usize]) {
    set.retain(|k| !k.as_slice().starts_with(prefix));
}

/// 在指定位置插入节点时，对 HashMap 中的路径进行移位。
pub(super) fn shift_on_insert<V>(
    map: &mut HashMap<Vec<usize>, V>,
    parent_path: &[usize],
    insert_idx: usize,
) {
    let mut next = HashMap::with_capacity(map.len());
    for (mut path, value) in map.drain() {
        if path.len() > parent_path.len() && path.as_slice().starts_with(parent_path) {
            let idx = path[parent_path.len()];
            if idx >= insert_idx {
                path[parent_path.len()] = idx + 1;
            }
        }
        next.insert(path, value);
    }
    *map = next;
}

/// 在指定位置插入节点时，对 HashSet 中的路径进行移位。
pub(super) fn shift_set_on_insert(
    set: &mut HashSet<Vec<usize>>,
    parent_path: &[usize],
    insert_idx: usize,
) {
    let mut next = HashSet::with_capacity(set.len());
    for mut path in set.drain() {
        if path.len() > parent_path.len() && path.as_slice().starts_with(parent_path) {
            let idx = path[parent_path.len()];
            if idx >= insert_idx {
                path[parent_path.len()] = idx + 1;
            }
        }
        next.insert(path);
    }
    *set = next;
}

/// 在指定位置插入节点时，对节点位置进行偏移。
pub(super) fn shift_positions_on_insert(
    map: &mut HashMap<Vec<usize>, Point>,
    parent_path: &[usize],
    insert_idx: usize,
    delta: Vector,
) {
    let parent_len = parent_path.len();
    for (path, pos) in map.iter_mut() {
        if path.len() > parent_len
            && path.as_slice().starts_with(parent_path)
            && path[parent_len] >= insert_idx
        {
            pos.x += delta.x;
            pos.y += delta.y;
        }
    }
}

/// 删除节点后，对 HashMap 中的路径进行移位。
pub(super) fn shift_on_delete<V>(
    map: &mut HashMap<Vec<usize>, V>,
    parent_path: &[usize],
    removed_idx: usize,
) {
    let mut next = HashMap::with_capacity(map.len());
    for (mut path, value) in map.drain() {
        if path.len() > parent_path.len() && path.as_slice().starts_with(parent_path) {
            let idx = path[parent_path.len()];
            if idx > removed_idx {
                path[parent_path.len()] = idx - 1;
            }
        }
        next.insert(path, value);
    }
    *map = next;
}

/// 删除节点后，对 HashSet 中的路径进行移位。
pub(super) fn shift_set_on_delete(
    set: &mut HashSet<Vec<usize>>,
    parent_path: &[usize],
    removed_idx: usize,
) {
    let mut next = HashSet::with_capacity(set.len());
    for mut path in set.drain() {
        if path.len() > parent_path.len() && path.as_slice().starts_with(parent_path) {
            let idx = path[parent_path.len()];
            if idx > removed_idx {
                path[parent_path.len()] = idx - 1;
            }
        }
        next.insert(path);
    }
    *set = next;
}

/// 关闭标签页的上下文菜单。
pub(super) fn close_context_menu_tab(tab: &mut MindMapTab) {
    tab.show_context_menu = false;
    tab.context_menu_anchor = None;
}

/// 将当前文档状态推入撤销栈。
pub(super) fn push_undo(tab: &mut MindMapTab) {
    tab.undo_stack.push(tab.doc.clone());
    tab.redo_stack.clear();
    if tab.undo_stack.len() > 50 {
        tab.undo_stack.remove(0);
    }
}

#[allow(dead_code)]
pub(super) fn dir_for_node(layout_format: MindMapLayoutFormat, node_path: &[usize]) -> f32 {
    match layout_format {
        MindMapLayoutFormat::RightAligned => 1.0,
        MindMapLayoutFormat::LeftAligned => -1.0,
        MindMapLayoutFormat::Bidirectional => {
            if node_path.is_empty() || node_path[0].is_multiple_of(2) {
                1.0
            } else {
                -1.0
            }
        }
    }
}

/// 重新布局画布但保持根节点位置。
pub(super) fn relayout_keep_root(tab: &mut MindMapTab) {
    let current_layout = compute_layout_for_diagram(
        &tab.doc,
        &tab.node_positions,
        &tab.node_priorities,
        &tab.node_urls,
        &tab.collapsed_paths,
        tab.diagram_type,
        tab.layout_format,
        tab.org_chart_layout_format,
        tab.fishbone_layout_format,
        tab.timeline_layout_format,
        tab.bracket_layout_format,
        tab.tree_layout_format,
    );
    let current_root_pos = current_layout
        .nodes
        .iter()
        .find(|n| n.path.is_empty())
        .map(|n| n.pos)
        .unwrap_or(Point::new(0.0, 0.0));

    let empty_positions: HashMap<Vec<usize>, Point> = HashMap::new();
    let auto_layout = compute_layout_for_diagram(
        &tab.doc,
        &empty_positions,
        &tab.node_priorities,
        &tab.node_urls,
        &tab.collapsed_paths,
        tab.diagram_type,
        tab.layout_format,
        tab.org_chart_layout_format,
        tab.fishbone_layout_format,
        tab.timeline_layout_format,
        tab.bracket_layout_format,
        tab.tree_layout_format,
    );
    let auto_root_pos = auto_layout
        .nodes
        .iter()
        .find(|n| n.path.is_empty())
        .map(|n| n.pos)
        .unwrap_or(Point::new(0.0, 0.0));

    let delta =
        Vector::new(current_root_pos.x - auto_root_pos.x, current_root_pos.y - auto_root_pos.y);

    tab.node_positions.clear();
    for node in auto_layout.nodes {
        tab.node_positions.insert(
            node.path,
            Point::new(node.pos.x + delta.x, node.pos.y + delta.y),
        );
    }
    tab.canvas_cache.clear();
}
