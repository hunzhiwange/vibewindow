//! 组织结构图布局模块
//!
//! 本模块提供组织结构图（Org Chart）的自动布局算法实现。支持两种布局格式：
//! - **自上而下（TopDown）**：根节点在顶部，子节点向下延伸
//! - **自左向右（LeftRight）**：根节点在左侧，子节点向右延伸
//!
//! # 核心功能
//!
//! - 计算树形结构中每个节点的最优位置
//! - 支持节点优先级和URL标记的显示
//! - 支持折叠节点，折叠后的子节点不参与布局
//! - 支持手动定位与自动定位的混合使用
//!
//! # 布局算法
//!
//! 采用经典的树形布局算法，通过以下步骤完成布局：
//! 1. **尺寸计算**：遍历整棵树，计算每层深度的最大节点尺寸
//! 2. **跨度计算**：递归计算每个子树的占用空间
//! 3. **位置分配**：根据子树跨度，居中对齐排列子节点
//! 4. **边生成**：为父子节点之间创建连接边

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::OrgChartLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

use super::helpers::{has_priority, has_url, node_size};
use super::{EdgeLayout, Layout, NodeLayout};

/// 计算组织结构图的布局
///
/// 根据给定的思维导图根节点和各种参数，计算组织结构图的节点位置和连接边。
/// 该函数支持两种布局格式，通过 `layout_format` 参数控制。
///
/// # 参数
///
/// * `root` - 思维导图的根节点，包含完整的树形结构
/// * `node_positions` - 节点路径到手动位置的映射，用于支持手动定位
/// * `node_priorities` - 节点路径到优先级的映射，影响节点尺寸计算
/// * `node_urls` - 节点路径到URL的映射，影响节点尺寸计算
/// * `collapsed_paths` - 被折叠的节点路径集合，折叠节点的子树不参与布局
/// * `layout_format` - 布局格式，决定是自上而下还是自左向右
///
/// # 返回值
///
/// 返回包含所有节点布局信息和边信息的 `Layout` 结构体
///
/// # 示例
///
/// ```ignore
/// let layout = compute_org_chart_layout(
///     &root_node,
///     &manual_positions,
///     &priorities,
///     &urls,
///     &collapsed,
///     OrgChartLayoutFormat::TopDown,
/// );
/// // layout.nodes 包含所有节点的位置和尺寸
/// // layout.edges 包含所有父子连接边
/// ```
pub(crate) fn compute_org_chart_layout(
    root: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    layout_format: OrgChartLayoutFormat,
) -> Layout {
    // 存储所有节点的布局信息
    let mut nodes = Vec::new();
    // 存储所有边的布局信息
    let mut edges = Vec::new();

    // 层级之间的水平间距（用于左右布局）
    let x_gap = 80.0f32;
    // 层级之间的垂直间距（用于上下布局）
    let y_gap = 90.0f32;
    // 同级兄弟节点之间的间距
    let sibling_gap = 70.0f32;

    /// 递归计算每一层的最大节点宽度和高度
    ///
    /// 遍历整个树形结构，统计每一层深度中所有节点的最大尺寸，
    /// 用于后续计算层级之间的偏移量。
    ///
    /// # 参数
    ///
    /// * `node` - 当前访问的节点
    /// * `path` - 当前节点在树中的路径（索引序列）
    /// * `depth` - 当前节点的深度（根节点为0）
    /// * `max_w` - 存储每层最大宽度的向量
    /// * `max_h` - 存储每层最大高度的向量
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点URL映射
    /// * `collapsed_paths` - 折叠节点路径集合
    fn depth_max_sizes(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        max_w: &mut Vec<f32>,
        max_h: &mut Vec<f32>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) {
        // 计算当前节点的尺寸，考虑优先级、URL和是否为根节点
        let s = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 确保尺寸向量足够大以容纳当前深度
        if depth >= max_w.len() {
            max_w.resize(depth + 1, 0.0);
            max_h.resize(depth + 1, 0.0);
        }

        // 更新当前深度的最大宽度和高度
        max_w[depth] = max_w[depth].max(s.width);
        max_h[depth] = max_h[depth].max(s.height);

        // 如果节点被折叠，不继续遍历其子节点
        if collapsed_paths.contains(path) {
            return;
        }

        // 递归遍历所有子节点
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            depth_max_sizes(
                child,
                path,
                depth + 1,
                max_w,
                max_h,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            path.pop();
        }
    }

    /// 计算子树在自上而下布局中的水平跨度
    ///
    /// 递归计算以给定节点为根的子树在水平方向上占据的总宽度。
    /// 这包括所有子孙节点的宽度以及它们之间的间距。
    ///
    /// # 参数
    ///
    /// * `node` - 当前子树的根节点
    /// * `path` - 当前节点在树中的路径
    /// * `depth` - 当前节点的深度
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点URL映射
    /// * `collapsed_paths` - 折叠节点路径集合
    /// * `sibling_gap` - 兄弟节点之间的间距
    ///
    /// # 返回值
    ///
    /// 返回子树的总水平跨度（宽度）
    fn subtree_span_topdown(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        sibling_gap: f32,
    ) -> f32 {
        // 获取当前节点的宽度
        let self_w = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        )
        .width;

        // 如果节点被折叠或没有子节点，返回自身宽度
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return self_w;
        }

        // 计算所有子节点的总宽度（包括间距）
        let mut total = 0.0f32;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let w = subtree_span_topdown(
                child,
                path,
                depth + 1,
                node_priorities,
                node_urls,
                collapsed_paths,
                sibling_gap,
            );
            path.pop();
            // 第一个子节点不加间距，后续子节点之间添加间距
            if i > 0 {
                total += sibling_gap;
            }
            total += w;
        }

        // 返回子树总宽度和自身宽度中的较大值
        total.max(self_w)
    }

    /// 计算子树在自左向右布局中的垂直跨度
    ///
    /// 递归计算以给定节点为根的子树在垂直方向上占据的总高度。
    /// 这包括所有子孙节点的高度以及它们之间的间距。
    ///
    /// # 参数
    ///
    /// * `node` - 当前子树的根节点
    /// * `path` - 当前节点在树中的路径
    /// * `depth` - 当前节点的深度
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点URL映射
    /// * `collapsed_paths` - 折叠节点路径集合
    /// * `sibling_gap` - 兄弟节点之间的间距
    ///
    /// # 返回值
    ///
    /// 返回子树的总垂直跨度（高度）
    #[allow(dead_code)]
    fn subtree_span_leftright(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        sibling_gap: f32,
    ) -> f32 {
        // 获取当前节点的高度
        let self_h = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        )
        .height;

        // 如果节点被折叠或没有子节点，返回自身高度
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return self_h;
        }

        // 计算所有子节点的总高度（包括间距）
        let mut total = 0.0f32;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let h = subtree_span_leftright(
                child,
                path,
                depth + 1,
                node_priorities,
                node_urls,
                collapsed_paths,
                sibling_gap,
            );
            path.pop();
            // 第一个子节点不加间距，后续子节点之间添加间距
            if i > 0 {
                total += sibling_gap;
            }
            total += h;
        }

        // 返回子树总高度和自身高度中的较大值
        total.max(self_h)
    }

    // 计算每层的最大尺寸
    let mut max_w = Vec::<f32>::new();
    let mut max_h = Vec::<f32>::new();
    let mut path = Vec::new();
    depth_max_sizes(
        root,
        &mut path,
        0,
        &mut max_w,
        &mut max_h,
        node_priorities,
        node_urls,
        collapsed_paths,
    );

    // 计算每一层相对于前一层的偏移量
    // x_offsets 用于左右布局，y_offsets 用于上下布局
    let mut x_offsets = Vec::<f32>::new();
    let mut y_offsets = Vec::<f32>::new();
    for (i, (w, h)) in max_w.iter().copied().zip(max_h.iter().copied()).enumerate() {
        if i == 0 {
            // 第0层（根节点层）的偏移量为0
            x_offsets.push(0.0);
            y_offsets.push(0.0);
        } else {
            // 计算第i层相对于第i-1层的偏移
            // 偏移 = 前一层偏移 + 前一层半宽/高 + 当前层半宽/高 + 层间距
            let prev_w = max_w[i - 1];
            let prev_x = *x_offsets.get(i - 1).unwrap_or(&0.0);
            x_offsets.push(prev_x + prev_w / 2.0 + w / 2.0 + x_gap);

            let prev_h = max_h[i - 1];
            let prev_y = *y_offsets.get(i - 1).unwrap_or(&0.0);
            y_offsets.push(prev_y + prev_h / 2.0 + h / 2.0 + y_gap);
        }
    }

    /// 递归遍历并布局自上而下的组织结构图
    ///
    /// 从给定节点开始，递归地为整个子树计算布局。子节点在水平方向上居中对齐排列，
    /// 在垂直方向上按层级分布。
    ///
    /// # 参数
    ///
    /// * `node` - 当前要布局的节点
    /// * `path` - 当前节点在树中的路径
    /// * `depth` - 当前节点的深度
    /// * `auto_center` - 自动计算的中心位置
    /// * `nodes` - 存储所有节点布局信息的向量
    /// * `edges` - 存储所有边布局信息的向量
    /// * `node_positions` - 手动指定的节点位置映射
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点URL映射
    /// * `collapsed_paths` - 折叠节点路径集合
    /// * `y_offsets` - 每层在Y轴上的偏移量
    /// * `sibling_gap` - 兄弟节点之间的间距
    fn walk_topdown(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        auto_center: Point,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        y_offsets: &[f32],
        sibling_gap: f32,
    ) {
        // 确定节点位置：优先使用手动指定的位置，否则使用自动计算的位置
        let auto_pos = auto_center;
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 计算节点尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 将节点信息添加到布局中
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        // 如果节点被折叠或没有子节点，不继续处理子节点
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return;
        }

        // 预先计算每个子树的跨度
        let mut child_spans = Vec::<f32>::with_capacity(node.children.len());
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let span = subtree_span_topdown(
                child,
                path,
                depth + 1,
                node_priorities,
                node_urls,
                collapsed_paths,
                sibling_gap,
            );
            path.pop();
            child_spans.push(span);
        }

        // 计算所有子节点的总跨度（包括间距）
        let total = child_spans.iter().copied().sum::<f32>()
            + sibling_gap * (child_spans.len().saturating_sub(1) as f32);

        // 计算起始X坐标，使子节点居中对齐
        let mut cursor_x = auto_center.x - total / 2.0;

        // 获取子节点所在层的Y坐标
        let child_y = *y_offsets.get(depth + 1).unwrap_or(&auto_center.y);

        // 遍历每个子节点，计算其位置并递归处理
        for (i, child) in node.children.iter().enumerate() {
            let span = child_spans.get(i).copied().unwrap_or(0.0);
            // 计算子节点的中心X坐标
            let child_center_x = cursor_x + span / 2.0;
            // 移动光标到下一个子节点的起始位置
            cursor_x += span + sibling_gap;

            path.push(i);
            // 添加从父节点到当前子节点的边
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
            // 递归处理子节点
            walk_topdown(
                child,
                path,
                depth + 1,
                Point::new(child_center_x, child_y),
                nodes,
                edges,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
                y_offsets,
                sibling_gap,
            );
            path.pop();
        }
    }

    /// 递归遍历并布局自左向右的组织结构图
    ///
    /// 从给定节点开始，递归地为整个子树计算布局。子节点在垂直方向上居中对齐排列，
    /// 在水平方向上按层级分布。
    ///
    /// # 参数
    ///
    /// * `node` - 当前要布局的节点
    /// * `path` - 当前节点在树中的路径
    /// * `depth` - 当前节点的深度
    /// * `auto_center` - 自动计算的中心位置
    /// * `nodes` - 存储所有节点布局信息的向量
    /// * `edges` - 存储所有边布局信息的向量
    /// * `node_positions` - 手动指定的节点位置映射
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点URL映射
    /// * `collapsed_paths` - 折叠节点路径集合
    /// * `x_offsets` - 每层在X轴上的偏移量
    /// * `sibling_gap` - 兄弟节点之间的间距
    #[allow(dead_code)]
    fn walk_leftright(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        auto_center: Point,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
        x_offsets: &[f32],
        sibling_gap: f32,
    ) {
        // 确定节点位置：优先使用手动指定的位置，否则使用自动计算的位置
        let auto_pos = auto_center;
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 计算节点尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 将节点信息添加到布局中
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        // 如果节点被折叠或没有子节点，不继续处理子节点
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return;
        }

        // 预先计算每个子树的跨度
        let mut child_spans = Vec::<f32>::with_capacity(node.children.len());
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let span = subtree_span_leftright(
                child,
                path,
                depth + 1,
                node_priorities,
                node_urls,
                collapsed_paths,
                sibling_gap,
            );
            path.pop();
            child_spans.push(span);
        }

        // 计算所有子节点的总跨度（包括间距）
        let total = child_spans.iter().copied().sum::<f32>()
            + sibling_gap * (child_spans.len().saturating_sub(1) as f32);

        // 计算起始Y坐标，使子节点居中对齐
        let mut cursor_y = auto_center.y - total / 2.0;

        // 获取子节点所在层的X坐标
        let child_x = *x_offsets.get(depth + 1).unwrap_or(&auto_center.x);

        // 遍历每个子节点，计算其位置并递归处理
        for (i, child) in node.children.iter().enumerate() {
            let span = child_spans.get(i).copied().unwrap_or(0.0);
            // 计算子节点的中心Y坐标
            let child_center_y = cursor_y + span / 2.0;
            // 移动光标到下一个子节点的起始位置
            cursor_y += span + sibling_gap;

            path.push(i);
            // 添加从父节点到当前子节点的边
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
            // 递归处理子节点
            walk_leftright(
                child,
                path,
                depth + 1,
                Point::new(child_x, child_center_y),
                nodes,
                edges,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
                x_offsets,
                sibling_gap,
            );
            path.pop();
        }
    }

    // 根据布局格式选择相应的遍历函数
    let mut path = Vec::new();
    match layout_format {
        // 自上而下布局
        OrgChartLayoutFormat::TopDown => walk_topdown(
            root,
            &mut path,
            0,
            Point::new(0.0, 0.0),
            &mut nodes,
            &mut edges,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            &y_offsets,
            sibling_gap,
        ),
        // 自左向右布局（注：当前实现使用了 walk_topdown，可能需要修正为 walk_leftright）
        OrgChartLayoutFormat::LeftRight => walk_topdown(
            root,
            &mut path,
            0,
            Point::new(0.0, 0.0),
            &mut nodes,
            &mut edges,
            node_positions,
            node_priorities,
            node_urls,
            collapsed_paths,
            &y_offsets,
            sibling_gap,
        ),
    }

    Layout { nodes, edges }
}

#[cfg(test)]
#[path = "org_chart_tests.rs"]
mod org_chart_tests;
