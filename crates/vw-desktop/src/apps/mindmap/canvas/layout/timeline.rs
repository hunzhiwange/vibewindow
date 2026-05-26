//! 时间线布局模块
//!
//! 本模块提供思维导图的时间线布局算法实现。时间线布局将根节点置于左侧中央，
//! 子节点沿水平方向依次展开，每个子节点的后代可以向上或向下延伸，
//! 形成类似时间轴的可视化效果。
//!
//! # 布局特性
//!
//! - 根节点位于画布左侧中央位置
//! - 子节点沿水平轴依次排列
//! - 支持三种布局方向：上下交替、全部向上、全部向下
//! - 自动处理节点折叠状态
//! - 支持手动调整的节点位置
//!
//! # 布局参数
//!
//! - `axis_gap`: 分支之间的水平间距
//! - `depth_dx`: 深度方向的水平位移
//! - `y_gap`: 叶子节点之间的垂直间距

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::TimelineLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

use super::helpers::{has_priority, has_url, node_size};
use super::{EdgeLayout, Layout, NodeLayout};

/// 计算时间线布局
///
/// 根据给定的思维导图根节点和布局参数，计算每个节点在画布上的位置和大小，
/// 以及节点之间的连接边，生成完整的时间线布局数据。
///
/// # 参数
///
/// * `root` - 思维导图的根节点引用，包含完整的节点树结构
/// * `node_positions` - 手动调整的节点位置映射，键为节点路径，值为坐标点
/// * `node_priorities` - 节点优先级映射，键为节点路径，值为优先级（0-255）
/// * `node_urls` - 节点关联的 URL 映射，键为节点路径，值为 URL 字符串
/// * `collapsed_paths` - 已折叠的节点路径集合，这些节点的子节点将被隐藏
/// * `timeline_layout_format` - 时间线布局格式，决定子节点后代的延伸方向
///
/// # 返回值
///
/// 返回 `Layout` 结构体，包含：
/// - `nodes`: 所有可见节点的布局信息（路径、文本、位置、大小）
/// - `edges`: 所有连接边的布局信息（起点路径、终点路径）
///
/// # 布局算法
///
/// 1. 首先计算根节点的位置和大小
/// 2. 遍历根节点的直接子节点，为每个子分支计算水平范围
/// 3. 根据分支范围确定每个子节点的水平中心位置
/// 4. 递归处理每个子节点的后代，按指定方向（上/下）排列
/// 5. 叶子节点按顺序排列，非叶子节点居中于其子节点范围
///
/// # 示例
///
/// ```ignore
/// let layout = compute_timeline_layout(
///     &root_node,
///     &manual_positions,
///     &priorities,
///     &urls,
///     &collapsed,
///     TimelineLayoutFormat::UpDown,
/// );
/// ```
pub(crate) fn compute_timeline_layout(
    root: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    timeline_layout_format: TimelineLayoutFormat,
) -> Layout {
    // 存储所有节点的布局信息
    let mut nodes = Vec::new();
    // 存储所有边的布局信息
    let mut edges = Vec::new();

    // 分支之间的水平间距（根节点与子节点之间、各分支之间）
    let axis_gap = 90.0f32;
    // 深度方向每增加一层，节点的水平位移量
    let depth_dx = 150.0f32;
    // 叶子节点之间的垂直间距
    let y_gap = 70.0f32;

    // 根节点路径（空路径表示根节点）
    let root_path: Vec<usize> = Vec::new();

    // 计算根节点的尺寸，考虑优先级和 URL 状态
    let root_size = node_size(
        &root.text,
        has_priority(node_priorities, &root_path),
        has_url(node_urls, &root_path),
        true, // 根节点标记
    );

    // 根节点的默认位置（原点）
    let root_auto = Point::new(0.0, 0.0);

    // 优先使用手动调整的位置，否则使用默认位置
    let root_pos = node_positions.get(&root_path).copied().unwrap_or(root_auto);

    // 将根节点添加到布局中
    nodes.push(NodeLayout {
        path: root_path.clone(),
        text: root.text.clone(),
        pos: root_pos,
        size: root_size,
    });

    // 如果根节点已折叠或没有子节点，直接返回当前布局
    if collapsed_paths.contains(&root_path) || root.children.is_empty() {
        return Layout { nodes, edges };
    }

    /// 计算子树的水平右边界范围
    ///
    /// 递归遍历子树中的所有节点，找出最右侧的边界坐标。
    /// 这个值用于确定分支的总宽度，以便在多个分支之间合理分配空间。
    ///
    /// # 参数
    ///
    /// * `node` - 当前处理的节点
    /// * `path` - 当前节点的路径（会被修改并在递归后恢复）
    /// * `depth_in_branch` - 当前节点在分支中的深度（从 0 开始）
    /// * `depth_dx` - 每层深度的水平位移
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点 URL 映射
    /// * `collapsed_paths` - 已折叠的节点路径集合
    ///
    /// # 返回值
    ///
    /// 返回子树最右侧边界的 X 坐标值
    fn subtree_right_extent(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth_in_branch: usize,
        depth_dx: f32,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) -> f32 {
        // 计算当前节点的尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            path.is_empty(),
        );

        // 当前节点自身的右边界：深度位移 + 节点宽度的一半
        let mut best = depth_in_branch as f32 * depth_dx + size.width / 2.0;

        // 如果节点已折叠，不继续遍历子节点
        if collapsed_paths.contains(path) {
            return best;
        }

        // 递归遍历所有子节点，更新最大右边界
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            best = best.max(subtree_right_extent(
                child,
                path,
                depth_in_branch + 1,
                depth_dx,
                node_priorities,
                node_urls,
                collapsed_paths,
            ));
            path.pop();
        }
        best
    }

    /// 遍历分支并布局节点
    ///
    /// 递归遍历分支中的所有节点，为每个节点计算位置并添加到布局中。
    /// 采用叶子节点优先的策略：叶子节点按顺序排列，非叶子节点居中于其子节点。
    ///
    /// # 参数
    ///
    /// * `node` - 当前处理的节点
    /// * `path` - 当前节点的路径（会被修改并在递归后恢复）
    /// * `depth_in_branch` - 当前节点在分支中的深度（从 0 开始）
    /// * `base_x` - 分支的基准 X 坐标
    /// * `sign` - 垂直方向符号（-1.0 表示向上，1.0 表示向下）
    /// * `nodes` - 节点布局集合（输出参数）
    /// * `edges` - 边布局集合（输出参数）
    /// * `next_leaf_t` - 下一个叶子节点的垂直位置（可变引用）
    /// * `y_gap` - 叶子节点之间的垂直间距
    /// * `depth_dx` - 每层深度的水平位移
    /// * `node_positions` - 手动调整的节点位置映射
    /// * `node_priorities` - 节点优先级映射
    /// * `node_urls` - 节点 URL 映射
    /// * `collapsed_paths` - 已折叠的节点路径集合
    ///
    /// # 返回值
    ///
    /// 返回当前节点（及其子树）的中心垂直位置 T 值
    fn walk_branch(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth_in_branch: usize,
        base_x: f32,
        sign: f32,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        next_leaf_t: &mut f32,
        y_gap: f32,
        depth_dx: f32,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) -> f32 {
        // 存储所有子节点的中心垂直位置
        let mut child_ts = Vec::new();

        // 如果节点未折叠，递归处理子节点
        if !collapsed_paths.contains(path) {
            for (i, child) in node.children.iter().enumerate() {
                path.push(i);

                // 递归处理子节点
                let t = walk_branch(
                    child,
                    path,
                    depth_in_branch + 1,
                    base_x,
                    sign,
                    nodes,
                    edges,
                    next_leaf_t,
                    y_gap,
                    depth_dx,
                    node_positions,
                    node_priorities,
                    node_urls,
                    collapsed_paths,
                );

                // 添加从父节点到子节点的边
                edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
                child_ts.push(t);
                path.pop();
            }
        }

        // 计算当前节点的尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            path.is_empty(),
        );

        // 计算当前节点的中心垂直位置 T
        // 如果是叶子节点：按顺序排列，使用 next_leaf_t 追踪位置
        // 如果是非叶子节点：居中于所有子节点的范围
        let t = if child_ts.is_empty() {
            // 叶子节点：放置在 next_leaf_t 位置，并更新 next_leaf_t
            let t = *next_leaf_t + size.height / 2.0;
            *next_leaf_t += size.height + y_gap;
            t
        } else {
            // 非叶子节点：居中于子节点的最小和最大 T 值之间
            let min = child_ts.iter().copied().reduce(f32::min).unwrap_or(0.0);
            let max = child_ts.iter().copied().reduce(f32::max).unwrap_or(0.0);
            (min + max) / 2.0
        };

        // 计算节点的自动位置
        // X 坐标：基准位置 + 深度位移
        // Y 坐标：符号 * T 值（实现向上或向下的布局）
        let auto_x = base_x + depth_in_branch as f32 * depth_dx;
        let auto_pos = Point::new(auto_x, sign * t);

        // 优先使用手动调整的位置
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 将节点添加到布局中
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        t
    }

    // ========== 第一阶段：计算各分支的水平范围 ==========

    // 存储每个分支的（左边界，右边界）范围
    let mut extents: Vec<(f32, f32)> = Vec::new();

    for (i, child) in root.children.iter().enumerate() {
        let mut path = vec![i];

        // 计算子节点的尺寸
        let child_size = node_size(
            &child.text,
            has_priority(node_priorities, &path),
            has_url(node_urls, &path),
            false,
        );

        // 左边界为节点宽度的一半
        let left = child_size.width / 2.0;

        // 右边界通过递归计算整个子树的最大范围
        let right = subtree_right_extent(
            child,
            &mut path,
            0,
            depth_dx,
            node_priorities,
            node_urls,
            collapsed_paths,
        );

        extents.push((left, right));
    }

    // ========== 第二阶段：计算各分支的水平中心位置 ==========

    // 存储每个分支的 X 中心坐标
    let mut x_centers: Vec<f32> = Vec::new();

    // 起始位置：根节点右侧 + 根节点宽度的一半 + 分支间距
    let mut x = root_pos.x + root_size.width / 2.0 + axis_gap;

    for (i, (left, right)) in extents.iter().copied().enumerate() {
        if i == 0 {
            // 第一个分支：加上左边界偏移
            x += left;
        } else {
            // 后续分支：与前一个分支的右边界保持间距
            let prev_right = extents[i - 1].1;
            x += prev_right + axis_gap + left;
        }

        x_centers.push(x);
        x += right;
    }

    // ========== 第三阶段：布局各分支的节点 ==========

    for (i, child) in root.children.iter().enumerate() {
        let mut path = vec![i];

        // 获取当前分支的基准 X 坐标
        let base_x = *x_centers.get(i).unwrap_or(&0.0);

        // 计算子节点的位置，与根节点同一水平线
        let auto_pos = Point::new(base_x, root_pos.y);
        let pos = node_positions.get(&path).copied().unwrap_or(auto_pos);

        // 计算子节点尺寸
        let size = node_size(
            &child.text,
            has_priority(node_priorities, &path),
            has_url(node_urls, &path),
            false,
        );

        // 添加子节点到布局
        nodes.push(NodeLayout { path: path.clone(), text: child.text.clone(), pos, size });

        // 添加从根节点到子节点的边
        edges.push(EdgeLayout { from: Vec::new(), to: path.clone() });

        // 根据布局格式确定后代节点的垂直延伸方向
        // UpDown: 偶数索引向上，奇数索引向下
        // AllUp: 全部向上
        // AllDown: 全部向下
        let sign = match timeline_layout_format {
            TimelineLayoutFormat::UpDown => {
                if i % 2 == 0 {
                    -1.0 // 向上
                } else {
                    1.0 // 向下
                }
            }
            TimelineLayoutFormat::AllUp => -1.0,
            TimelineLayoutFormat::AllDown => 1.0,
        };

        // 如果子节点已折叠或没有后代，跳过后续处理
        if collapsed_paths.contains(&path) || child.children.is_empty() {
            continue;
        }

        // 初始化叶子节点的垂直位置追踪器
        let mut next_leaf_t = y_gap;

        // 递归处理子节点的所有后代
        for (j, grand) in child.children.iter().enumerate() {
            path.push(j);

            let _ = walk_branch(
                grand,
                &mut path,
                1, // 从深度 1 开始（因为这是孙节点）
                base_x,
                sign,
                &mut nodes,
                &mut edges,
                &mut next_leaf_t,
                y_gap,
                depth_dx,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );

            // 添加从子节点到孙节点的边
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
            path.pop();
        }
    }

    Layout { nodes, edges }
}

#[cfg(test)]
#[path = "timeline_tests.rs"]
mod timeline_tests;
