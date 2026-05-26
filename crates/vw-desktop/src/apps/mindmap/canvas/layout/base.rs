//! 思维导图基础布局算法，提供不同布局格式共享的节点测量和坐标计算。

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use iced::{Point, Rectangle, Size, Vector};
use std::collections::{HashMap, HashSet};

/// NodeLayout 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Clone)]
pub struct NodeLayout {
    /// path 字段，保存渲染或状态更新所需的输入数据。
    pub path: Vec<usize>,
    /// text 字段，保存渲染或状态更新所需的输入数据。
    pub text: String,
    /// pos 字段，保存渲染或状态更新所需的输入数据。
    pub pos: Point,
    /// size 字段，保存渲染或状态更新所需的输入数据。
    pub size: Size,
}

/// EdgeLayout 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Clone)]
pub struct EdgeLayout {
    /// from 字段，保存渲染或状态更新所需的输入数据。
    pub from: Vec<usize>,
    /// to 字段，保存渲染或状态更新所需的输入数据。
    pub to: Vec<usize>,
}

/// Layout 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Clone)]
pub struct Layout {
    /// nodes 字段，保存渲染或状态更新所需的输入数据。
    pub nodes: Vec<NodeLayout>,
    /// edges 字段，保存渲染或状态更新所需的输入数据。
    pub edges: Vec<EdgeLayout>,
}

/// 构建或更新 layout node rect 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(crate) fn layout_node_rect(n: &NodeLayout) -> Rectangle {
    Rectangle::new(Point::new(n.pos.x - n.size.width / 2.0, n.pos.y - n.size.height / 2.0), n.size)
}

/// 构建或更新 layout bounds world 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(crate) fn layout_bounds_world(layout: &Layout) -> Rectangle {
    let mut min_x = 0.0f32;
    let mut min_y = 0.0f32;
    let mut max_x = 0.0f32;
    let mut max_y = 0.0f32;
    let mut first = true;
    for n in &layout.nodes {
        let r = layout_node_rect(n);
        if first {
            min_x = r.x;
            min_y = r.y;
            max_x = r.x + r.width;
            max_y = r.y + r.height;
            first = false;
        } else {
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.width);
            max_y = max_y.max(r.y + r.height);
        }
    }
    Rectangle::new(Point::new(min_x, min_y), Size::new(max_x - min_x, max_y - min_y))
}

/// 构建或更新 selected node top center screen 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
#[allow(dead_code)]
pub fn selected_node_top_center_screen(
    doc: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    collapsed_paths: &HashSet<Vec<usize>>,
    pan: Vector,
    zoom: f32,
    selected_path: &[usize],
    diagram_type: MindMapDiagramType,
    layout_format: MindMapLayoutFormat,
    org_chart_layout_format: OrgChartLayoutFormat,
    fishbone_layout_format: FishboneLayoutFormat,
    timeline_layout_format: TimelineLayoutFormat,
    bracket_layout_format: BracketLayoutFormat,
    tree_layout_format: TreeLayoutFormat,
) -> Option<Point> {
    let empty_priorities = HashMap::new();
    let empty_urls = HashMap::new();
    let layout = super::compute_layout_for_diagram(
        doc,
        node_positions,
        &empty_priorities,
        &empty_urls,
        collapsed_paths,
        diagram_type,
        layout_format,
        org_chart_layout_format,
        fishbone_layout_format,
        timeline_layout_format,
        bracket_layout_format,
        tree_layout_format,
    );
    let node = layout.nodes.iter().find(|n| n.path == selected_path)?;
    let world_rect = layout_node_rect(node);
    let top_center_world = Point::new(world_rect.x + world_rect.width / 2.0, world_rect.y);
    Some(super::super::transform::screen_from_world(top_center_world, pan, zoom))
}

/// 构建或更新 selected node rect screen 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn selected_node_rect_screen(
    doc: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    collapsed_paths: &HashSet<Vec<usize>>,
    pan: Vector,
    zoom: f32,
    selected_path: &[usize],
    diagram_type: MindMapDiagramType,
    layout_format: MindMapLayoutFormat,
    org_chart_layout_format: OrgChartLayoutFormat,
    fishbone_layout_format: FishboneLayoutFormat,
    timeline_layout_format: TimelineLayoutFormat,
    bracket_layout_format: BracketLayoutFormat,
    tree_layout_format: TreeLayoutFormat,
) -> Option<Rectangle> {
    let empty_priorities = HashMap::new();
    let empty_urls = HashMap::new();
    let layout = super::compute_layout_for_diagram(
        doc,
        node_positions,
        &empty_priorities,
        &empty_urls,
        collapsed_paths,
        diagram_type,
        layout_format,
        org_chart_layout_format,
        fishbone_layout_format,
        timeline_layout_format,
        bracket_layout_format,
        tree_layout_format,
    );
    let node = layout.nodes.iter().find(|n| n.path == selected_path)?;
    let world_rect = layout_node_rect(node);
    let top_left_screen = super::super::transform::screen_from_world(
        Point::new(world_rect.x, world_rect.y),
        pan,
        zoom,
    );
    Some(Rectangle::new(
        top_left_screen,
        Size::new(world_rect.width * zoom, world_rect.height * zoom),
    ))
}

#[cfg(test)]
#[path = "base_tests.rs"]
mod base_tests;
