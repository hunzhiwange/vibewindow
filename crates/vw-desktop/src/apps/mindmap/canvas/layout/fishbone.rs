//! 思维导图鱼骨布局算法，负责鱼骨图分支与节点坐标的稳定排列。

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::FishboneLayoutFormat;
use iced::{Point, Size};
use std::collections::{HashMap, HashSet};

use super::helpers::{has_priority, has_url, node_size};
use super::{EdgeLayout, Layout, NodeLayout};

/// 构建或更新 compute fishbone layout 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(crate) fn compute_fishbone_layout(
    root: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    layout_format: FishboneLayoutFormat,
) -> Layout {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let spine_dir = match layout_format {
        FishboneLayoutFormat::HeadRight => -1.0,
        FishboneLayoutFormat::HeadLeft => 1.0,
    };

    let spine_start = 240.0f32;
    let base_branch_gap = 120.0f32;
    let base_branch_step = 240.0f32;
    let base_branch_dx = 160.0f32;
    let base_branch_dy = 130.0f32;
    let base_deep_x_gap = 190.0f32;
    let base_stack_gap = 18.0f32;
    let spine_clearance = 18.0f32;

    fn estimate_subtree_width(
        node: &MindNode,
        path: &mut Vec<usize>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        base_deep_x_gap: f32,
    ) -> f32 {
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            path.is_empty(),
        );

        if collapsed_paths.contains(path) || node.children.is_empty() {
            return size.width;
        }

        let mut max_child = 0.0f32;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            max_child = max_child.max(estimate_subtree_width(
                child,
                path,
                node_priorities,
                node_urls,
                collapsed_paths,
                base_deep_x_gap,
            ));
            path.pop();
        }
        size.width + base_deep_x_gap + max_child
    }

    fn stack_height_from_sizes(sizes: &[Size], gap: f32) -> f32 {
        if sizes.is_empty() {
            return 0.0;
        }
        let mut h = 0.0f32;
        for s in sizes {
            h += s.height;
        }
        h + gap * (sizes.len().saturating_sub(1) as f32)
    }

    fn layout_children_stack(
        node: &MindNode,
        path: &mut Vec<usize>,
        parent_pos: Point,
        parent_size: Size,
        side_sign: f32,
        spine_y: f32,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        spine_dir: f32,
        base_deep_x_gap: f32,
        base_stack_gap: f32,
        spine_clearance: f32,
    ) {
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return;
        }

        let mut child_sizes: Vec<Size> = Vec::with_capacity(node.children.len());
        let mut max_child_w = 0.0f32;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let s = node_size(
                &child.text,
                has_priority(node_priorities, path),
                has_url(node_urls, path),
                false,
            );
            max_child_w = max_child_w.max(s.width);
            child_sizes.push(s);
            path.pop();
        }

        let x_gap = base_deep_x_gap.max(parent_size.width / 2.0 + max_child_w / 2.0 + 80.0);
        let stack_h = stack_height_from_sizes(&child_sizes, base_stack_gap);

        let mut start_y = parent_pos.y - stack_h / 2.0;
        if side_sign < 0.0 {
            let stack_max = start_y + stack_h;
            let limit = spine_y - spine_clearance;
            if stack_max > limit {
                start_y -= stack_max - limit;
            }
        } else {
            let stack_min = start_y;
            let limit = spine_y + spine_clearance;
            if stack_min < limit {
                start_y += limit - stack_min;
            }
        }

        let child_x = parent_pos.x + spine_dir * x_gap;
        let mut cursor_y = start_y;
        for (i, child) in node.children.iter().enumerate() {
            let s = child_sizes[i];
            let child_center_y = cursor_y + s.height / 2.0;
            cursor_y += s.height + base_stack_gap;

            path.push(i);
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });

            let auto_pos = Point::new(child_x, child_center_y);
            let pos = node_positions.get(path).copied().unwrap_or(auto_pos);
            nodes.push(NodeLayout { path: path.clone(), text: child.text.clone(), pos, size: s });

            layout_children_stack(
                child,
                path,
                pos,
                s,
                side_sign,
                spine_y,
                nodes,
                edges,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
                spine_dir,
                base_deep_x_gap,
                base_stack_gap,
                spine_clearance,
            );
            path.pop();
        }
    }

    fn layout_branch_children_along_rib(
        branch: &MindNode,
        branch_path: &mut Vec<usize>,
        branch_pos: Point,
        branch_size: Size,
        side_sign: f32,
        spine_x: f32,
        spine_y: f32,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        spine_dir: f32,
        base_deep_x_gap: f32,
        base_stack_gap: f32,
        spine_clearance: f32,
    ) {
        if collapsed_paths.contains(branch_path) || branch.children.is_empty() {
            return;
        }

        let mut child_sizes: Vec<Size> = Vec::with_capacity(branch.children.len());
        for (i, child) in branch.children.iter().enumerate() {
            branch_path.push(i);
            child_sizes.push(node_size(
                &child.text,
                has_priority(node_priorities, branch_path),
                has_url(node_urls, branch_path),
                false,
            ));
            branch_path.pop();
        }

        let stack_h = stack_height_from_sizes(&child_sizes, base_stack_gap);
        let mut start_y = branch_pos.y - stack_h / 2.0;
        if side_sign < 0.0 {
            let stack_max = start_y + stack_h;
            let limit = spine_y - spine_clearance;
            if stack_max > limit {
                start_y -= stack_max - limit;
            }
        } else {
            let stack_min = start_y;
            let limit = spine_y + spine_clearance;
            if stack_min < limit {
                start_y += limit - stack_min;
            }
        }

        let denom = branch_pos.y - spine_y;
        let choose_farther_x =
            |a: f32, b: f32| -> f32 { if spine_dir > 0.0 { a.max(b) } else { a.min(b) } };

        let mut cursor_y = start_y;
        for (i, child) in branch.children.iter().enumerate() {
            let s = child_sizes[i];
            let y = cursor_y + s.height / 2.0;
            cursor_y += s.height + base_stack_gap;

            branch_path.push(i);
            edges.push(EdgeLayout {
                from: branch_path[..branch_path.len() - 1].to_vec(),
                to: branch_path.clone(),
            });

            let t = if denom.abs() < 1.0 { 1.0 } else { ((y - spine_y) / denom).clamp(0.0, 1.0) };
            let rib_x = spine_x + t * (branch_pos.x - spine_x);

            let rib_offset = 140.0f32.max(s.width / 2.0 + 60.0);
            let child_from_rib_x = rib_x + spine_dir * rib_offset;
            let child_from_branch_x =
                branch_pos.x + spine_dir * (branch_size.width / 2.0 + s.width / 2.0 + 120.0);
            let auto_x = choose_farther_x(child_from_rib_x, child_from_branch_x);

            let auto_pos = Point::new(auto_x, y);
            let pos = node_positions.get(branch_path).copied().unwrap_or(auto_pos);
            nodes.push(NodeLayout {
                path: branch_path.clone(),
                text: child.text.clone(),
                pos,
                size: s,
            });

            layout_children_stack(
                child,
                branch_path,
                pos,
                s,
                side_sign,
                spine_y,
                nodes,
                edges,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
                spine_dir,
                base_deep_x_gap,
                base_stack_gap,
                spine_clearance,
            );
            branch_path.pop();
        }
    }

    let root_path = Vec::new();
    let root_size = node_size(
        &root.text,
        has_priority(node_priorities, &root_path),
        has_url(node_urls, &root_path),
        true,
    );
    let root_pos = node_positions.get(&root_path).copied().unwrap_or(Point::new(0.0, 0.0));
    nodes.push(NodeLayout {
        path: root_path.clone(),
        text: root.text.clone(),
        pos: root_pos,
        size: root_size,
    });

    if collapsed_paths.contains(&root_path) {
        return Layout { nodes, edges };
    }

    let spine_y = root_pos.y;
    let mut cursor = spine_start;
    let mut prev_w = 0.0f32;

    for (i, child) in root.children.iter().enumerate() {
        let up = i % 2 == 0;
        let sign = if up { -1.0 } else { 1.0 };

        let mut path = vec![i];
        edges.push(EdgeLayout { from: Vec::new(), to: path.clone() });

        let child_size = node_size(
            &child.text,
            has_priority(node_priorities, &path),
            has_url(node_urls, &path),
            false,
        );

        let subtree_w = estimate_subtree_width(
            child,
            &mut path.clone(),
            node_priorities,
            node_urls,
            collapsed_paths,
            base_deep_x_gap,
        );
        if i > 0 {
            cursor += (base_branch_step).max(prev_w / 2.0 + subtree_w / 2.0 + base_branch_gap);
        }
        prev_w = subtree_w;

        let mut child_child_sizes: Vec<Size> = Vec::with_capacity(child.children.len());
        for (j, grand) in child.children.iter().enumerate() {
            path.push(j);
            child_child_sizes.push(node_size(
                &grand.text,
                has_priority(node_priorities, &path),
                has_url(node_urls, &path),
                false,
            ));
            path.pop();
        }
        let child_stack_h = stack_height_from_sizes(&child_child_sizes, base_stack_gap);

        let branch_dx = base_branch_dx.max(root_size.width / 2.0 + child_size.width / 2.0 + 140.0);
        let branch_dy = base_branch_dy.max(root_size.height / 2.0 + child_size.height / 2.0 + 80.0);
        let y_offset = branch_dy + child_stack_h / 2.0 + spine_clearance;

        let spine_x = root_pos.x + spine_dir * cursor;
        let branch_end = Point::new(spine_x + spine_dir * branch_dx, spine_y + sign * y_offset);

        let child_pos = node_positions.get(&path).copied().unwrap_or(branch_end);
        nodes.push(NodeLayout {
            path: path.clone(),
            text: child.text.clone(),
            pos: child_pos,
            size: child_size,
        });

        if collapsed_paths.contains(&path) {
            continue;
        }

        layout_branch_children_along_rib(
            child,
            &mut path,
            child_pos,
            child_size,
            sign,
            spine_x,
            spine_y,
            &mut nodes,
            &mut edges,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            spine_dir,
            base_deep_x_gap,
            base_stack_gap,
            spine_clearance,
        );
    }

    Layout { nodes, edges }
}

#[cfg(test)]
#[path = "fishbone_tests.rs"]
mod fishbone_tests;
