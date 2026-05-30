//! 思维导图画布命中测试逻辑，负责把鼠标坐标映射到节点和工具区域。

use super::super::layout::{Layout, NodeLayout, layout_node_rect};
use super::super::transform::screen_from_world;
use crate::app::components::mind_map::{self, MindNode};
use crate::apps::mindmap::message::MindMapMessage;
use iced::{Point, Rectangle, Size};

use super::{HoverButtonKind, MindMapCanvas};

impl<'a> MindMapCanvas<'a> {
    fn node_button_dir(&self, node_path: &[usize]) -> f32 {
        if self.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::OrgChart {
            return 1.0;
        }
        if self.diagram_type == crate::apps::mindmap::state::MindMapDiagramType::Bracket {
            return match self.layout_format {
                crate::apps::mindmap::state::MindMapLayoutFormat::LeftAligned => -1.0,
                crate::apps::mindmap::state::MindMapLayoutFormat::RightAligned
                | crate::apps::mindmap::state::MindMapLayoutFormat::Bidirectional => 1.0,
            };
        }
        match self.layout_format {
            crate::apps::mindmap::state::MindMapLayoutFormat::RightAligned => 1.0,
            crate::apps::mindmap::state::MindMapLayoutFormat::LeftAligned => -1.0,
            crate::apps::mindmap::state::MindMapLayoutFormat::Bidirectional => {
                if node_path.is_empty() {
                    1.0
                } else if node_path[0].is_multiple_of(2) {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }

    /// 构建或更新 node screen rect 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn node_screen_rect(&self, n: &NodeLayout) -> Rectangle {
        let world_rect = layout_node_rect(n);
        let top_left =
            screen_from_world(Point::new(world_rect.x, world_rect.y), self.pan, self.zoom);
        let size = Size::new(world_rect.width * self.zoom, world_rect.height * self.zoom);
        Rectangle::new(top_left, size)
    }

    /// 构建或更新 collapsed count badge rect 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn collapsed_count_badge_rect(
        &self,
        node_path: &[usize],
        toggle_center: Point,
        r: f32,
        child_count: usize,
    ) -> Rectangle {
        let text = format!("+{child_count}");
        let font_size = (11.0 * self.zoom).clamp(9.0, 16.0);
        let char_w = font_size * 0.62;
        let pad_x = (8.0 * self.zoom).clamp(5.0, 10.0);
        let pad_y = (5.0 * self.zoom).clamp(3.0, 8.0);
        let w = (text.len() as f32 * char_w + pad_x * 2.0).max(r * 2.0);
        let h = (font_size + pad_y * 2.0).max(r * 2.0);
        let gap = (6.0 * self.zoom).clamp(3.0, 10.0);
        let dir = self.node_button_dir(node_path);
        let x = if dir >= 0.0 { toggle_center.x + r + gap } else { toggle_center.x - r - gap - w };
        Rectangle::new(Point::new(x, toggle_center.y - h / 2.0), Size::new(w, h))
    }

    /// 构建或更新 node button specs 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn node_button_specs(
        &self,
        node_path: &[usize],
        node_rect: Rectangle,
    ) -> Vec<(HoverButtonKind, Point, f32)> {
        let r = (10.0 * self.zoom).clamp(6.0, 12.0);
        let gap = (6.0 * self.zoom).clamp(4.0, 8.0);
        let outside = (10.0 * self.zoom).clamp(6.0, 14.0);

        let children_len =
            mind_map::node(self.doc, node_path).map(|n| n.children.len()).unwrap_or(0);
        let is_collapsed = self.collapsed_paths.contains(node_path);
        let mut kinds = Vec::new();
        if children_len > 0 {
            kinds.push(HoverButtonKind::ToggleCollapse);
        }
        if !is_collapsed {
            if !node_path.is_empty() {
                kinds.push(HoverButtonKind::AddSibling);
            }
            kinds.push(HoverButtonKind::AddChild);
        }

        let n = kinds.len() as f32;
        let _w = n * (2.0 * r) + (n - 1.0) * gap;
        let dir = self.node_button_dir(node_path);
        let left = if dir >= 0.0 {
            node_rect.x + node_rect.width + outside + r
        } else {
            node_rect.x - outside - r
        };
        let y = node_rect.y + node_rect.height / 2.0;

        kinds
            .into_iter()
            .enumerate()
            .map(|(i, k)| {
                let cx = left + dir * i as f32 * (2.0 * r + gap);
                (k, Point::new(cx, y), r)
            })
            .collect()
    }

    /// 构建或更新 node button group rect 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn node_button_group_rect(
        &self,
        node_path: &[usize],
        node_rect: Rectangle,
    ) -> Option<Rectangle> {
        let specs = self.node_button_specs(node_path, node_rect);
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut toggle_center: Option<(Point, f32)> = None;

        for (kind, center, r) in &specs {
            min_x = min_x.min(center.x - *r);
            min_y = min_y.min(center.y - *r);
            max_x = max_x.max(center.x + *r);
            max_y = max_y.max(center.y + *r);
            if *kind == HoverButtonKind::ToggleCollapse {
                toggle_center = Some((*center, *r));
            }
        }

        if !min_x.is_finite() {
            return None;
        }

        if self.collapsed_paths.contains(node_path) {
            let child_count =
                mind_map::node(self.doc, node_path).map(descendant_count).unwrap_or(0);
            if child_count > 0
                && let Some((center, r)) = toggle_center
            {
                let badge_rect = self.collapsed_count_badge_rect(node_path, center, r, child_count);
                min_x = min_x.min(badge_rect.x);
                min_y = min_y.min(badge_rect.y);
                max_x = max_x.max(badge_rect.x + badge_rect.width);
                max_y = max_y.max(badge_rect.y + badge_rect.height);
            }
        }

        Some(Rectangle::new(Point::new(min_x, min_y), Size::new(max_x - min_x, max_y - min_y)))
    }

    /// 构建或更新 node hover rect 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn node_hover_rect(&self, node_path: &[usize], node_rect: Rectangle) -> Rectangle {
        let mut min_x = node_rect.x;
        let mut min_y = node_rect.y;
        let mut max_x = node_rect.x + node_rect.width;
        let mut max_y = node_rect.y + node_rect.height;

        if let Some(btn_rect) = self.node_button_group_rect(node_path, node_rect) {
            min_x = min_x.min(btn_rect.x);
            min_y = min_y.min(btn_rect.y);
            max_x = max_x.max(btn_rect.x + btn_rect.width);
            max_y = max_y.max(btn_rect.y + btn_rect.height);
        }

        let pad = (6.0 * self.zoom).clamp(3.0, 10.0);
        Rectangle::new(
            Point::new(min_x - pad, min_y - pad),
            Size::new((max_x - min_x) + pad * 2.0, (max_y - min_y) + pad * 2.0),
        )
    }

    /// 构建或更新 hit node buttons 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn hit_node_buttons(
        &self,
        layout: &Layout,
        cursor_pos: Point,
    ) -> Option<MindMapMessage> {
        for n in &layout.nodes {
            let rect = self.node_screen_rect(n);
            for (kind, center, r) in self.node_button_specs(&n.path, rect) {
                let dx = cursor_pos.x - center.x;
                let dy = cursor_pos.y - center.y;
                if dx * dx + dy * dy <= r * r {
                    return match kind {
                        HoverButtonKind::ToggleCollapse => {
                            Some(MindMapMessage::ToggleCollapseAt(n.path.clone()))
                        }
                        HoverButtonKind::AddChild => {
                            Some(MindMapMessage::AddChildAt(n.path.clone()))
                        }
                        HoverButtonKind::AddSibling => {
                            Some(MindMapMessage::AddSiblingAt(n.path.clone()))
                        }
                    };
                }
            }
        }
        None
    }

    /// 构建或更新 hovered node path 相关行为。
    ///
    /// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
    pub(super) fn hovered_node_path(
        &self,
        layout: &Layout,
        cursor_pos: Point,
    ) -> Option<Vec<usize>> {
        for n in &layout.nodes {
            let node_rect = self.node_screen_rect(n);
            if self.node_hover_rect(&n.path, node_rect).contains(cursor_pos) {
                return Some(n.path.clone());
            }
        }
        None
    }
}

/// 构建或更新 descendant count 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn descendant_count(node: &MindNode) -> usize {
    node.children.iter().map(|c| 1usize + descendant_count(c)).sum()
}
