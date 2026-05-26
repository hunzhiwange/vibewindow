//! 思维导图布局模块入口，按图形类型分发到具体布局算法。

mod base;
mod fishbone;
mod helpers;
mod mindmap;
mod org_chart;
mod timeline;
mod tree;

pub(crate) use base::{layout_bounds_world, layout_node_rect, selected_node_rect_screen};
pub(crate) use mindmap::compute_layout;

pub use base::{EdgeLayout, Layout, NodeLayout};

use crate::apps::mindmap::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use std::collections::{HashMap, HashSet};

/// 构建或更新 compute layout for diagram 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(crate) fn compute_layout_for_diagram(
    root: &crate::app::components::mind_map::MindNode,
    node_positions: &HashMap<Vec<usize>, iced::Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    diagram_type: MindMapDiagramType,
    mindmap_layout_format: MindMapLayoutFormat,
    org_chart_layout_format: OrgChartLayoutFormat,
    fishbone_layout_format: FishboneLayoutFormat,
    timeline_layout_format: TimelineLayoutFormat,
    bracket_layout_format: BracketLayoutFormat,
    tree_layout_format: TreeLayoutFormat,
) -> Layout {
    match diagram_type {
        MindMapDiagramType::OrgChart => org_chart::compute_org_chart_layout(
            root,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            org_chart_layout_format,
        ),
        MindMapDiagramType::Fishbone => fishbone::compute_fishbone_layout(
            root,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            fishbone_layout_format,
        ),
        MindMapDiagramType::Bracket => {
            let bracket_layout = match bracket_layout_format {
                BracketLayoutFormat::BraceRight => MindMapLayoutFormat::RightAligned,
                BracketLayoutFormat::BraceLeft => MindMapLayoutFormat::LeftAligned,
            };
            mindmap::compute_layout(
                root,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
                bracket_layout,
            )
        }
        MindMapDiagramType::Tree => tree::compute_tree_layout(
            root,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            tree_layout_format,
        ),
        MindMapDiagramType::Timeline => timeline::compute_timeline_layout(
            root,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            timeline_layout_format,
        ),
        MindMapDiagramType::MindMap => mindmap::compute_layout(
            root,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            mindmap_layout_format,
        ),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
