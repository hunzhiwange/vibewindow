//! 树形布局计算模块
//!
//! 本模块提供了思维导图的树形布局算法实现，支持多种布局样式。
//! 根据不同的布局格式，计算每个节点在画布上的位置坐标，以及节点之间的连接关系。
//!
//! # 支持的布局格式
//!
//! - **FanDown（扇形向下）**: 子节点呈扇形向下展开，所有子树水平居中对齐
//! - **SymmetricSplit（对称分割）**: 根节点居中，子节点交替分布在左右两侧
//! - **LeftAligned（左对齐）**: 所有节点向左对齐，呈垂直脊柱状
//! - **RightAligned（右对齐）**: 所有节点向右对齐，呈垂直脊柱状
//!
//! # 布局算法特点
//!
//! - 自动计算子树宽度，避免节点重叠
//! - 支持节点折叠状态，折叠的子树不参与布局计算
//! - 支持自定义节点位置（拖拽后的位置会被保留）
//! - 根据节点内容动态计算节点大小
//!
//! # 示例
//!
//! ```ignore
//! let layout = compute_tree_layout(
//!     &root_node,
//!     &node_positions,
//!     &node_priorities,
//!     &node_urls,
//!     &collapsed_paths,
//!     TreeLayoutFormat::FanDown,
//! );
//! ```

use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::TreeLayoutFormat;
use iced::Point;
use std::collections::{HashMap, HashSet};

use super::helpers::{has_priority, has_url, node_size};
use super::{EdgeLayout, Layout, NodeLayout};

/// 计算树形布局
///
/// 根据指定的布局格式，计算思维导图树中所有节点的位置和边连接关系。
/// 这是布局计算的主入口函数，会根据不同的布局格式调用相应的布局算法。
///
/// # 参数
///
/// - `root`: 根节点，包含完整的节点树结构
/// - `node_positions`: 用户手动调整过的节点位置映射（路径 -> 坐标点）
/// - `node_priorities`: 节点优先级映射（路径 -> 优先级值），影响节点样式
/// - `node_urls`: 节点URL映射（路径 -> URL字符串），影响节点样式
/// - `collapsed_paths`: 已折叠的节点路径集合，这些节点的子树不参与布局
/// - `tree_layout_format`: 树形布局格式，决定节点的排列方式
///
/// # 返回值
///
/// 返回 `Layout` 结构体，包含：
/// - `nodes`: 所有节点的布局信息（位置、大小、文本等）
/// - `edges`: 所有边的连接信息（从哪个节点到哪个节点）
///
/// # 布局算法流程
///
/// 1. 计算每层的最大高度（用于确定Y轴偏移量）
/// 2. 根据布局格式选择相应的布局策略：
///    - FanDown: 扇形向下展开
///    - SymmetricSplit: 根节点居中，子节点左右对称分布
///    - LeftAligned/RightAligned: 垂直脊柱式布局
/// 3. 递归遍历节点树，计算每个节点的位置
/// 4. 生成边连接信息
///
/// # 示例
///
/// ```ignore
/// let layout = compute_tree_layout(
///     &root,
///     &HashMap::new(),
///     &HashMap::new(),
///     &HashMap::new(),
///     &HashSet::new(),
///     TreeLayoutFormat::FanDown,
/// );
/// ```
pub(crate) fn compute_tree_layout(
    root: &MindNode,
    node_positions: &HashMap<Vec<usize>, Point>,
    node_priorities: &HashMap<Vec<usize>, u8>,
    node_urls: &HashMap<Vec<usize>, String>,
    collapsed_paths: &HashSet<Vec<usize>>,
    tree_layout_format: TreeLayoutFormat,
) -> Layout {
    // 存储所有节点的布局信息
    let mut nodes = Vec::new();
    // 存储所有边的连接信息
    let mut edges = Vec::new();

    // 水平方向节点间距（相邻节点之间的水平距离）
    let x_gap = 70.0f32;
    // 垂直方向层级间距（相邻层级之间的垂直距离）
    let y_gap = 90.0f32;

    /// 计算每个深度层级的最大高度
    ///
    /// 递归遍历整个节点树，统计每个深度层上所有节点的最大高度。
    /// 这用于在垂直方向上对齐不同层级的节点，确保同一层级的节点有统一的Y坐标。
    ///
    /// # 参数
    ///
    /// - `node`: 当前处理的节点
    /// - `path`: 当前节点在树中的路径（索引序列）
    /// - `depth`: 当前节点的深度（根节点为0）
    /// - `max_heights`: 输出参数，存储每个深度的最大高度
    /// - `node_priorities`: 节点优先级映射
    /// - `node_urls`: 节点URL映射
    /// - `collapsed_paths`: 折叠节点路径集合
    ///
    /// # 算法说明
    ///
    /// 1. 计算当前节点的尺寸（考虑文本、优先级、URL等因素）
    /// 2. 更新当前深度的最大高度记录
    /// 3. 如果节点已折叠，直接返回（不处理子节点）
    /// 4. 递归处理所有子节点
    fn depth_max_heights(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        max_heights: &mut Vec<f32>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) {
        // 计算当前节点的尺寸，根节点(depth==0)可能有特殊样式
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 更新当前深度的最大高度
        if max_heights.len() <= depth {
            // 如果是第一次遇到这个深度，直接添加
            max_heights.push(size.height);
        } else {
            // 否则，取当前高度和已有最大高度的最大值
            max_heights[depth] = max_heights[depth].max(size.height);
        }

        // 如果节点已折叠，跳过子节点的处理
        if collapsed_paths.contains(path) {
            return;
        }

        // 递归处理所有子节点
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            depth_max_heights(
                child,
                path,
                depth + 1,
                max_heights,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            path.pop();
        }
    }

    /// 估算子树的总宽度
    ///
    /// 递归计算以指定节点为根的子树所需的总水平宽度。
    /// 这个宽度用于在水平方向上合理分配子节点的位置，避免子树之间的重叠。
    ///
    /// # 参数
    ///
    /// - `node`: 当前处理的节点
    /// - `path`: 当前节点在树中的路径
    /// - `depth`: 当前节点的深度
    /// - `x_gap`: 水平方向节点间距
    /// - `node_priorities`: 节点优先级映射
    /// - `node_urls`: 节点URL映射
    /// - `collapsed_paths`: 折叠节点路径集合
    ///
    /// # 返回值
    ///
    /// 返回子树的总宽度（包括节点之间的间距）
    ///
    /// # 算法说明
    ///
    /// 1. 如果节点已折叠或没有子节点，返回节点自身的宽度
    /// 2. 否则，累加所有子树的宽度 + 子树之间的间距
    /// 3. 最终宽度取（子树总宽度，当前节点宽度）的最大值
    fn estimate_subtree_width(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        x_gap: f32,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) -> f32 {
        // 计算当前节点的尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 如果节点已折叠或没有子节点，直接返回节点宽度
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return size.width;
        }

        // 累加所有子树的宽度和间距
        let mut total = 0.0f32;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            let w = estimate_subtree_width(
                child,
                path,
                depth + 1,
                x_gap,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            path.pop();
            // 从第二个子节点开始，需要加上间距
            if i > 0 {
                total += x_gap;
            }
            total += w;
        }

        // 返回子树总宽度和当前节点宽度的最大值
        total.max(size.width)
    }

    // 计算每个深度的最大高度
    let mut max_heights = Vec::<f32>::new();
    let mut root_path = Vec::new();
    depth_max_heights(
        root,
        &mut root_path,
        0,
        &mut max_heights,
        node_priorities,
        node_urls,
        collapsed_paths,
    );

    // 根据每层的最大高度，计算每层的Y轴偏移量
    // y_offsets[i] 表示第 i 层中心点的 Y 坐标
    let mut y_offsets = Vec::<f32>::new();
    for (i, h) in max_heights.iter().copied().enumerate() {
        if i == 0 {
            // 根节点的 Y 坐标为 0
            y_offsets.push(0.0);
        } else {
            // 后续层级的 Y 坐标 = 上一层的 Y 坐标 + 上一层高度的一半 + 当前层高度的一半 + 层级间距
            let prev_h = max_heights[i - 1];
            let prev = *y_offsets.get(i - 1).unwrap_or(&0.0);
            y_offsets.push(prev + prev_h / 2.0 + h / 2.0 + y_gap);
        }
    }

    /// 扇形布局放置算法（FanDown 布局）
    ///
    /// 以扇形方式递归放置节点及其子树。子节点水平居中对齐，向下展开。
    /// 每个子树的宽度会自动计算，确保不与其他子树重叠。
    ///
    /// # 参数
    ///
    /// - `node`: 当前要放置的节点
    /// - `path`: 当前节点在树中的路径
    /// - `depth`: 当前节点的深度
    /// - `center_x`: 当前节点中心的X坐标
    /// - `nodes`: 输出参数，存储所有节点的布局信息
    /// - `edges`: 输出参数，存储所有边的连接信息
    /// - `y_offsets`: 每个深度层级的Y轴偏移量
    /// - `x_gap`: 水平方向节点间距
    /// - `node_positions`: 用户自定义的节点位置
    /// - `node_priorities`: 节点优先级映射
    /// - `node_urls`: 节点URL映射
    /// - `collapsed_paths`: 折叠节点路径集合
    ///
    /// # 布局算法
    ///
    /// 1. 确定当前节点的位置（优先使用用户自定义位置，否则使用自动计算的位置）
    /// 2. 将节点信息添加到布局结果中
    /// 3. 如果节点已折叠或无子节点，直接返回
    /// 4. 计算每个子树的宽度
    /// 5. 计算子树的总宽度和起始位置（确保整体居中对齐）
    /// 6. 为每个子节点递归调用布局算法
    fn place_fan(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        center_x: f32,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        y_offsets: &[f32],
        x_gap: f32,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) {
        // 获取当前层级的Y坐标
        let y = *y_offsets.get(depth).unwrap_or(&0.0);

        // 自动计算的位置（水平居中）
        let auto_pos = Point::new(center_x, y);

        // 如果用户拖动过该节点，使用用户指定的位置，否则使用自动计算的位置
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 计算节点尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 将节点添加到布局结果中
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        // 如果节点已折叠或没有子节点，直接返回
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return;
        }

        // 计算每个子树的宽度，用于水平分布子节点
        let mut widths = Vec::<f32>::with_capacity(node.children.len());
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            widths.push(estimate_subtree_width(
                child,
                path,
                depth + 1,
                x_gap,
                node_priorities,
                node_urls,
                collapsed_paths,
            ));
            path.pop();
        }

        // 计算所有子树的总宽度（包括子树之间的间距）
        let total =
            widths.iter().copied().sum::<f32>() + x_gap * (widths.len().saturating_sub(1) as f32);

        // 计算第一个子树的起始X坐标（整体居中对齐）
        let mut cursor = center_x - total / 2.0;

        // 递归放置每个子节点
        for (i, child) in node.children.iter().enumerate() {
            let w = widths[i];
            // 计算子节点的中心X坐标
            let child_center_x = cursor + w / 2.0;
            // 更新下一个子树的起始位置
            cursor += w + x_gap;

            path.push(i);
            // 添加从父节点到子节点的边
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });
            // 递归放置子节点
            place_fan(
                child,
                path,
                depth + 1,
                child_center_x,
                nodes,
                edges,
                y_offsets,
                x_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            path.pop();
        }
    }

    /// 对称分割布局算法（SymmetricSplit 布局）
    ///
    /// 根节点居中显示，子节点交替分布在左右两侧，形成对称的树形结构。
    /// 偶数索引的子节点在右侧，奇数索引的子节点在左侧。
    ///
    /// # 参数
    ///
    /// - `root`: 根节点
    /// - `nodes`: 输出参数，存储所有节点的布局信息
    /// - `edges`: 输出参数，存储所有边的连接信息
    /// - `y_offsets`: 每个深度层级的Y轴偏移量
    /// - `x_gap`: 水平方向节点间距
    /// - `node_positions`: 用户自定义的节点位置
    /// - `node_priorities`: 节点优先级映射
    /// - `node_urls`: 节点URL映射
    /// - `collapsed_paths`: 折叠节点路径集合
    ///
    /// # 布局算法
    ///
    /// 1. 先放置根节点（居中显示）
    /// 2. 将子节点分为左右两组（偶数索引 -> 右，奇数索引 -> 左）
    /// 3. 计算左右两组的总宽度
    /// 4. 从中心向左右两侧依次放置子节点
    /// 5. 每个子节点内部使用扇形布局算法（place_fan）
    fn place_split_root(
        root: &MindNode,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        y_offsets: &[f32],
        x_gap: f32,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) {
        // 根节点路径为空
        let root_path = Vec::<usize>::new();

        // 计算根节点尺寸
        let root_size = node_size(
            &root.text,
            has_priority(node_priorities, &root_path),
            has_url(node_urls, &root_path),
            true,
        );

        // 根节点位置（默认在原点，可被用户自定义位置覆盖）
        let root_auto = Point::new(0.0, *y_offsets.first().unwrap_or(&0.0));
        let root_pos = node_positions.get(&root_path).copied().unwrap_or(root_auto);

        // 添加根节点到布局结果
        nodes.push(NodeLayout {
            path: root_path.clone(),
            text: root.text.clone(),
            pos: root_pos,
            size: root_size,
        });

        // 如果根节点已折叠或没有子节点，直接返回
        if collapsed_paths.contains(&root_path) || root.children.is_empty() {
            return;
        }

        // 将子节点分为左右两组
        // 偶数索引的子节点放在右侧，奇数索引的子节点放在左侧
        let mut right = Vec::<(usize, f32)>::new();
        let mut left = Vec::<(usize, f32)>::new();
        for (i, child) in root.children.iter().enumerate() {
            let mut path = vec![i];
            let w = estimate_subtree_width(
                child,
                &mut path,
                1,
                x_gap,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
            if i % 2 == 0 {
                right.push((i, w));
            } else {
                left.push((i, w));
            }
        }

        // 计算左右两侧的总宽度（包括间距）
        let _total_right = right.iter().map(|(_, w)| *w).sum::<f32>()
            + x_gap * (right.len().saturating_sub(1) as f32);
        let total_left = left.iter().map(|(_, w)| *w).sum::<f32>()
            + x_gap * (left.len().saturating_sub(1) as f32);

        // 根节点与两侧子节点之间的水平距离
        let gap_from_center = root_size.width / 2.0 + 160.0;

        // 放置左侧的子节点（从左到右依次排列）
        let mut cursor_left = root_pos.x - gap_from_center - total_left;
        for (idx, w) in left {
            let child_center_x = cursor_left + w / 2.0;
            cursor_left += w + x_gap;

            let child = &root.children[idx];
            let mut path = vec![idx];
            // 添加从根节点到子节点的边
            edges.push(EdgeLayout { from: Vec::new(), to: path.clone() });
            // 使用扇形布局放置子树
            place_fan(
                child,
                &mut path,
                1,
                child_center_x,
                nodes,
                edges,
                y_offsets,
                x_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
        }

        // 放置右侧的子节点（从左到右依次排列）
        let mut cursor_right = root_pos.x + gap_from_center;
        for (idx, w) in right {
            let child_center_x = cursor_right + w / 2.0;
            cursor_right += w + x_gap;

            let child = &root.children[idx];
            let mut path = vec![idx];
            // 添加从根节点到子节点的边
            edges.push(EdgeLayout { from: Vec::new(), to: path.clone() });
            // 使用扇形布局放置子树
            place_fan(
                child,
                &mut path,
                1,
                child_center_x,
                nodes,
                edges,
                y_offsets,
                x_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
        }
    }

    /// 脊柱式布局算法（LeftAligned / RightAligned 布局）
    ///
    /// 节点呈垂直脊柱状排列，所有子节点在水平方向上向左或向右对齐，
    /// 垂直方向上依次向下排列。这种布局适合展示层级分明的结构。
    ///
    /// # 参数
    ///
    /// - `node`: 当前要放置的节点
    /// - `path`: 当前节点在树中的路径
    /// - `depth`: 当前节点的深度
    /// - `x`: 当前节点的X坐标
    /// - `y`: 当前节点的Y坐标
    /// - `dir`: 水平方向（-1.0 表示左对齐，1.0 表示右对齐）
    /// - `nodes`: 输出参数，存储所有节点的布局信息
    /// - `edges`: 输出参数，存储所有边的连接信息
    /// - `x_gap`: 水平方向节点间距
    /// - `y_gap`: 垂直方向层级间距
    /// - `node_positions`: 用户自定义的节点位置
    /// - `node_priorities`: 节点优先级映射
    /// - `node_urls`: 节点URL映射
    /// - `collapsed_paths`: 折叠节点路径集合
    ///
    /// # 返回值
    ///
    /// 返回以当前节点为根的子树的底部Y坐标（用于计算下一个兄弟节点的位置）
    ///
    /// # 布局算法
    ///
    /// 1. 放置当前节点
    /// 2. 计算子节点的水平位置（根据方向和深度）
    /// 3. 垂直方向上依次放置所有子节点
    /// 4. 递归处理每个子节点
    /// 5. 返回整个子树的底部位置
    fn place_spine(
        node: &MindNode,
        path: &mut Vec<usize>,
        depth: usize,
        x: f32,
        y: f32,
        dir: f32,
        nodes: &mut Vec<NodeLayout>,
        edges: &mut Vec<EdgeLayout>,
        x_gap: f32,
        y_gap: f32,
        node_positions: &HashMap<Vec<usize>, Point>,
        node_priorities: &HashMap<Vec<usize>, u8>,
        node_urls: &HashMap<Vec<usize>, String>,
        collapsed_paths: &HashSet<Vec<usize>>,
    ) -> f32 {
        // 确定节点位置（优先使用用户自定义位置）
        let auto_pos = Point::new(x, y);
        let pos = node_positions.get(path).copied().unwrap_or(auto_pos);

        // 计算节点尺寸
        let size = node_size(
            &node.text,
            has_priority(node_priorities, path),
            has_url(node_urls, path),
            depth == 0,
        );

        // 添加节点到布局结果
        nodes.push(NodeLayout { path: path.clone(), text: node.text.clone(), pos, size });

        // 记录当前子树的底部位置（初始为当前节点的底部）
        let mut bottom = pos.y + size.height / 2.0;

        // 如果节点已折叠或没有子节点，直接返回底部位置
        if collapsed_paths.contains(path) || node.children.is_empty() {
            return bottom;
        }

        // 从当前节点底部开始，垂直向下排列子节点
        let mut cursor_y = pos.y + size.height / 2.0 + y_gap;
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            // 添加边连接信息
            edges.push(EdgeLayout { from: path[..path.len() - 1].to_vec(), to: path.clone() });

            // 计算子节点尺寸
            let child_size = node_size(
                &child.text,
                has_priority(node_priorities, path),
                has_url(node_urls, path),
                false,
            );

            // 计算子节点的Y坐标（节点的Y坐标是其中心点）
            let child_y = cursor_y + child_size.height / 2.0;

            // 计算子节点的X坐标（根据方向和深度确定水平位置）
            // 深度越深，X坐标的偏移量越大
            let child_x = dir * (depth as f32 + 1.0) * x_gap;

            // 递归放置子节点，并获取子树的底部位置
            let child_bottom = place_spine(
                child,
                path,
                depth + 1,
                child_x,
                child_y,
                dir,
                nodes,
                edges,
                x_gap,
                y_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );

            // 更新整个子树的底部位置
            bottom = bottom.max(child_bottom);
            // 下一个子节点的起始Y位置
            cursor_y = child_bottom + y_gap;
            path.pop();
        }

        bottom
    }

    // 根据布局格式选择相应的布局算法
    match tree_layout_format {
        // 扇形向下布局：所有节点从根节点向下扇形展开
        TreeLayoutFormat::FanDown => {
            let mut path = Vec::new();
            place_fan(
                root,
                &mut path,
                0,
                0.0, // 根节点X坐标为0（居中）
                &mut nodes,
                &mut edges,
                &y_offsets,
                x_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
        }
        // 对称分割布局：根节点居中，子节点左右对称分布
        TreeLayoutFormat::SymmetricSplit => {
            place_split_root(
                root,
                &mut nodes,
                &mut edges,
                &y_offsets,
                x_gap,
                node_positions,
                node_priorities,
                node_urls,
                collapsed_paths,
            );
        }
        // 左对齐或右对齐布局：节点呈垂直脊柱状排列
        TreeLayoutFormat::LeftAligned | TreeLayoutFormat::RightAligned => {
            // 确定方向：左对齐为-1.0，右对齐为1.0
            let dir = if tree_layout_format == TreeLayoutFormat::LeftAligned { -1.0 } else { 1.0 };
            let mut path = Vec::new();

            // 计算根节点尺寸
            let root_size = node_size(
                &root.text,
                has_priority(node_priorities, &path),
                has_url(node_urls, &path),
                true,
            );

            // 确定根节点位置
            let root_auto = Point::new(0.0, 0.0);
            let root_pos = node_positions.get(&path).copied().unwrap_or(root_auto);
            let root_y = root_pos.y;

            // 添加根节点到布局结果
            nodes.push(NodeLayout {
                path: path.clone(),
                text: root.text.clone(),
                pos: root_pos,
                size: root_size,
            });

            // 如果根节点未折叠且有子节点，处理所有子节点
            if !collapsed_paths.contains(&path) && !root.children.is_empty() {
                // 从根节点底部开始，垂直向下排列子节点
                let mut cursor_y = root_pos.y + root_size.height / 2.0 + y_gap;
                for (i, child) in root.children.iter().enumerate() {
                    path.push(i);
                    // 添加边连接
                    edges.push(EdgeLayout { from: Vec::new(), to: path.clone() });

                    // 计算子节点尺寸
                    let child_size = node_size(
                        &child.text,
                        has_priority(node_priorities, &path),
                        has_url(node_urls, &path),
                        false,
                    );

                    // 计算子节点位置
                    let child_y = cursor_y + child_size.height / 2.0;
                    let child_x = dir * x_gap; // 第一层子节点的X坐标

                    // 递归放置子节点
                    let child_bottom = place_spine(
                        child,
                        &mut path,
                        1,
                        child_x,
                        child_y,
                        dir,
                        &mut nodes,
                        &mut edges,
                        x_gap,
                        y_gap,
                        node_positions,
                        node_priorities,
                        node_urls,
                        collapsed_paths,
                    );

                    // 更新下一个子节点的起始位置
                    cursor_y = child_bottom + y_gap;
                    path.pop();
                }
            }
            let _ = root_y;
        }
    }

    // 返回完整的布局结果
    Layout { nodes, edges }
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tree_tests;
