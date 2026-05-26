//! 思维导图布局计算模块
//!
//! 本模块负责计算思维导图的节点位置和连线布局。采用经典的思维导图布局算法，
//! 支持多种布局格式（右对齐、左对齐、双向）。
//!
//! # 布局算法
//!
//! 算法分为两个阶段：
//! 1. **预处理阶段**：计算每个深度层级的最大节点宽度，用于确定 X 轴偏移量
//! 2. **布局阶段**：递归遍历节点树，计算每个节点的 Y 坐标（基于子节点居中）和 X 坐标
//!
//! # 布局格式
//!
//! - `RightAligned`：所有节点在根节点右侧，从左到右展开
//! - `LeftAligned`：所有节点在根节点左侧，从右到左展开
//! - `Bidirectional`：子节点交替分布在根节点两侧

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

use super::helpers::{has_priority, has_url, node_size};
use super::{EdgeLayout, Layout, NodeLayout};

/// 计算思维导图的布局
///
/// 根据根节点和配置信息，计算所有节点的位置和边的连接关系。
///
/// # 参数
///
/// * `root` - 思维导图的根节点，包含整个节点树结构
/// * `node_positions` - 用户手动调整过的节点位置映射（路径 -> 坐标点）
/// * `node_priorities` - 节点优先级映射（路径 -> 优先级值），用于显示优先级标记
/// * `node_urls` - 节点关联的 URL 映射（路径 -> URL），用于显示链接标记
/// * `collapsed_paths` - 折叠状态的节点路径集合，折叠的节点不会展开其子节点
/// * `layout_format` - 布局格式，决定节点的展开方向
///
/// # 返回值
///
/// 返回 `Layout` 结构体，包含：
/// - `nodes`: 所有节点的布局信息（位置、大小、文本等）
/// - `edges`: 所有边的连接信息（从父节点到子节点的路径）
///
/// # 示例
///
/// ```ignore
/// let layout = compute_layout(
///     &root_node,
///     &manual_positions,
///     &priorities,
///     &urls,
///     &collapsed,
///     MindMapLayoutFormat::RightAligned,
/// );
/// // 使用 layout.nodes 和 layout.edges 进行渲染
/// ```
pub(crate) fn compute_layout(
    root: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    layout_format: MindMapLayoutFormat,
) -> Layout {
    // 存储计算后的节点布局列表
    let mut nodes = Vec::new();
    // 存储计算后的边布局列表
    let mut edges = Vec::new();
    // 下一个叶子节点的 Y 坐标，用于从上到下布局叶子节点
    let mut next_leaf_y = 0.0f32;
    // 水平方向节点间距（像素）
    let x_gap = 80.0f32;
    // 垂直方向节点间距（像素）
    let y_gap = 70.0f32;

    /// 递归计算每个深度层级的最大节点宽度
    ///
    /// 此函数遍历整个节点树，统计每一层深度的最大节点宽度。
    /// 这些宽度信息用于计算 X 轴的偏移量，确保不同深度的节点不会重叠。
    ///
    /// # 参数
    ///
    /// * `node` - 当前处理的节点
    /// * `path` - 从根节点到当前节点的路径（索引序列）
    /// * `depth` - 当前节点的深度（根节点为 0）
    /// * `acc` - 累加器，存储每个深度的最大宽度
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点 URL 映射
    /// * `collapsed_paths` - 折叠节点路径集合
    fn depth_max_widths(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        acc: &mut Vec<f32>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) {
        // 计算当前节点的宽度（考虑文本长度、优先级标记、URL标记等）
        let w = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0, // 是否为根节点
        )
        .width;

        // 确保累加器有足够的空间存储当前深度的宽度
        if depth >= acc.len() {
            acc.resize(depth + 1, 0.0);
        }
        // 更新当前深度的最大宽度
        acc[depth] = acc[depth].max(w);

        // 如果当前节点被折叠，则不处理其子节点
        if collapsed_paths.contains(path) {
            return;
        }

        // 递归处理所有子节点
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            depth_max_widths(
                child,
                path,
                depth + 1,
                acc,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            path.pop();
        }
    }

    // 第一步：计算每个深度的最大节点宽度
    let mut max_widths = Vec::<f32>::new();
    let mut path = Vec::new();
    depth_max_widths(
        root,
        &mut path,
        0,
        &mut max_widths,
        node_priorities,
        node_urls,
        collapsed_paths,
    );

    // 第二步：根据最大宽度计算每个深度的 X 轴偏移量
    // X 偏移量是累积的，每个深度的 X 位置 = 前一深度的位置 + 前一深度宽度/2 + 当前宽度/2 + 间距
    let mut x_offsets = Vec::<f32>::new();
    for (i, w) in max_widths.iter().copied().enumerate() {
        if i == 0 {
            // 根节点的 X 偏移量为 0
            x_offsets.push(0.0);
        } else {
            // 计算当前深度的 X 偏移量
            let prev_w = max_widths[i - 1];
            let prev = *x_offsets.get(i - 1).unwrap_or(&0.0);
            // 偏移量 = 前一位置 + 前一节点半宽 + 当前节点半宽 + 间距
            x_offsets.push(prev + prev_w / 2.0 + w / 2.0 + x_gap);
        }
    }

    /// 递归遍历节点树并计算每个节点的位置
    ///
    /// 采用"叶子优先"的布局策略：
    /// - 叶子节点从上到下依次排列
    /// - 非叶子节点的 Y 坐标是其所有子节点 Y 坐标的中点
    ///
    /// # 参数
    ///
    /// * `node` - 当前处理的节点
    /// * `path` - 从根节点到当前节点的路径
    /// * `depth` - 当前节点的深度
    /// * `nodes` - 输出：节点布局列表
    /// * `edges` - 输出：边布局列表
    /// * `next_leaf_y` - 下一个叶子节点的 Y 坐标（可变引用）
    /// * `y_gap` - 垂直间距
    /// * `x_offsets` - 每个深度的 X 轴偏移量数组
    /// * `node_positions` - 用户手动调整的位置
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点 URL 映射
    /// * `collapsed_paths` - 折叠节点路径集合
    /// * `layout_format` - 布局格式
    ///
    /// # 返回值
    ///
    /// 返回当前节点的 Y 坐标
    fn walk(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        next_leaf_y: &mut f32,
        y_gap: f32,
        x_offsets: &[f32],
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        layout_format: MindMapLayoutFormat,
    ) -> f32 {
        // 存储所有子节点的 Y 坐标，用于计算当前节点的居中位置
        let mut child_ys = Vec::new();

        // 如果节点未被折叠，则处理其子节点
        if !collapsed_paths.contains(path) {
            for (i, child) in node.children.iter().enumerate() {
                path.push(i);
                // 递归处理子节点，获取子节点的 Y 坐标
                let y = walk(
                    child,
                    path,
                    depth + 1,
                    nodes,
                    edges,
                    next_leaf_y,
                    y_gap,
                    x_offsets,
                    node_positions,
                    node_priorities,
                    node_urls,
                    collapsed_paths,
                    layout_format,
                );
                // 添加从父节点到子节点的边
                edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
                child_ys.push(y);
                path.pop();
            }
        }

        // 计算当前节点的大小（宽度和高度）
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0, // 根节点有特殊样式
        );

        // 计算当前节点的 Y 坐标
        let y = if child_ys.is_empty() {
            // 叶子节点：使用 next_leaf_y 并递增
            let y = *next_leaf_y + size.height / 2.0;
            *next_leaf_y += size.height + y_gap;
            y
        } else {
            // 非叶子节点：Y 坐标为所有子节点 Y 坐标的中点
            let min = child_ys.iter().copied().reduce(f32::min).unwrap_or(0.0);
            let max = child_ys.iter().copied().reduce(f32::max).unwrap_or(0.0);
            (min + max) / 2.0
        };

        // 计算自动布局的 X 坐标
        let auto_x = if depth == 0 {
            // 根节点固定在 X = 0
            0.0
        } else {
            // 根据深度获取 X 偏移量
            let x = *x_offsets.get(depth).unwrap_or(&0.0);
            // 根据布局格式确定方向（正或负）
            let dir = match layout_format {
                MindMapLayoutFormat::RightAligned => 1.0, // 向右展开
                MindMapLayoutFormat::LeftAligned => -1.0, // 向左展开
                MindMapLayoutFormat::Bidirectional => {
                    // 双向布局：根据子节点索引的奇偶性决定方向
                    let on_right = path.first().copied().unwrap_or(0) % 2 == 0;
                    if on_right { 1.0 } else { -1.0 }
                }
            };
            x * dir
        };

        // 生成自动计算的位置
        let auto_pos = Point::new(auto_x, y);
        // 优先使用用户手动调整的位置，如果没有则使用自动计算的位置
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 将当前节点添加到布局列表
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        // 返回当前节点的 Y 坐标，供父节点计算居中位置使用
        y
    }

    // 执行布局计算
    let mut path = Vec::new();
    walk(
        root,
        &mut path,
        0,
        &mut nodes,
        &mut edges,
        &mut next_leaf_y,
        y_gap,
        &x_offsets,
        node_positions,
        node_priorities,
        node_urls,
        collapsed_paths,
        layout_format,
    );

    // 返回最终的布局结果
    Layout { nodes, edges }
}

#[cfg(test)]
#[path = "mindmap_tests.rs"]
mod mindmap_tests;
