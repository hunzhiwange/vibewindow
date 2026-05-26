//! 网格渐变填充（Mesh Gradient）辅助模块
//!
//! 本模块提供网格渐变填充相关的辅助函数，用于处理设计画布中的网格渐变效果。
//! 网格渐变是一种复杂的渐变类型，通过二维网格点及其控制句柄来定义颜色过渡。
//!
//! # 主要功能
//!
//! - **解析填充项**：将 JSON 数据解析为 `FillItem` 集合，支持网格填充类型的标准化
//! - **填充索引选择**：根据启用状态和选中状态选择合适的网格填充索引
//! - **坐标转换**：将屏幕光标坐标转换为归一化的 UV 纹理坐标
//! - **句柄管理**：获取和管理网格点的控制句柄
//! - **命中测试**：判断鼠标点击是否命中网格点或其控制句柄
//! - **拖拽更新**：处理网格点和句柄的拖拽操作，实时更新位置
//! - **曲线载荷生成**：生成用于前端渲染的贝塞尔曲线数据

use crate::app::views::design::properties::fill::types::{FillItem, FillObject, MeshFill};
use iced::{Point, Rectangle};

use super::super::types::{MeshDragKind, MeshDragState};

/// 解析填充项列表
///
/// 从可选的 JSON 值中解析填充项，支持数组和单个对象两种格式。
/// 如果解析出的填充项是网格类型，会自动调用 `normalize()` 方法进行标准化处理。
///
/// # 参数
///
/// - `v`: 可选的 JSON 值，可以是填充项数组或单个填充项对象
///
/// # 返回值
///
/// 返回解析后的 `FillItem` 向量。如果输入为 `None` 或解析失败，返回空向量。
///
/// # 示例
///
/// ```ignore
/// let json = serde_json::json!([{"type": "mesh", ...}]);
/// let items = parse_fill_items(&Some(json));
/// ```
pub(super) fn parse_fill_items(v: &Option<serde_json::Value>) -> Vec<FillItem> {
    // 如果输入为 None，直接返回空向量
    let Some(v) = v else {
        return vec![];
    };

    // 尝试解析为数组格式
    if let Ok(mut fills) = serde_json::from_value::<Vec<FillItem>>(v.clone()) {
        // 遍历所有填充项，对网格类型进行标准化
        for item in &mut fills {
            if let FillItem::Object(FillObject::Mesh(m)) = item {
                m.normalize();
            }
        }
        return fills;
    }

    // 尝试解析为单个对象格式
    if let Ok(mut item) = serde_json::from_value::<FillItem>(v.clone()) {
        if let FillItem::Object(FillObject::Mesh(m)) = &mut item {
            m.normalize();
        }
        return vec![item];
    }

    // 所有格式都解析失败，返回空向量
    vec![]
}

/// 选择网格填充的索引
///
/// 根据当前选中的填充索引和启用状态，从填充列表中选择一个合适的网格填充。
/// 选择逻辑优先使用用户选中的索引，如果不可用则选择第一个启用的网格填充。
///
/// # 参数
///
/// - `fills`: 填充项列表的切片引用
/// - `selected_fill_index`: 用户当前选中的填充索引（可选）
///
/// # 返回值
///
/// 返回选中的网格填充索引。如果没有找到合适的网格填充，返回 `None`。
///
/// # 选择规则
///
/// 1. 如果选中的索引有效且对应的网格填充已启用，直接返回该索引
/// 2. 否则，从列表开头开始查找第一个启用的网格填充
/// 3. 如果没有找到任何启用的网格填充，返回 `None`
pub(super) fn choose_mesh_fill_index(
    fills: &[FillItem],
    selected_fill_index: Option<usize>,
) -> Option<usize> {
    // 检查用户选中的索引是否指向一个启用的网格填充
    if let Some(i) = selected_fill_index
        && let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get(i)
        && m.enabled
    {
        return Some(i);
    }

    // 遍历所有填充项，查找第一个启用的网格填充
    for (i, item) in fills.iter().enumerate() {
        if let FillItem::Object(FillObject::Mesh(m)) = item
            && m.enabled
        {
            return Some(i);
        }
    }

    // 没有找到任何合适的网格填充
    None
}

/// 将光标坐标转换为归一化 UV 坐标（带边界限制）
///
/// 将屏幕上的光标坐标转换为相对于给定边界框的归一化 UV 坐标，
/// 并将结果限制在 [0.0, 1.0] 范围内。
///
/// # 参数
///
/// - `x`: 光标的 X 屏幕坐标
/// - `y`: 光标的 Y 屏幕坐标
/// - `bounds`: 参考边界框（通常是网格填充的可视区域）
///
/// # 返回值
///
/// 返回归一化的 (u, v) 坐标元组，其中 u 和 v 都在 [0.0, 1.0] 范围内。
///
/// # 注意
///
/// 此函数会将超出边界范围的值截断到 [0.0, 1.0]，适用于大多数交互场景。
/// 如果需要原始值（不截断），请使用 `cursor_to_uv_raw` 函数。
pub(super) fn cursor_to_uv(x: f32, y: f32, bounds: Rectangle) -> (f64, f64) {
    // 获取原始 UV 坐标
    let (u, v) = cursor_to_uv_raw(x, y, bounds);
    // 将结果限制在 [0.0, 1.0] 范围内
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// 将光标坐标转换为归一化 UV 坐标（原始值，无边界限制）
///
/// 将屏幕上的光标坐标转换为相对于给定边界框的归一化 UV 坐标。
/// 不对结果进行边界限制，允许值超出 [0.0, 1.0] 范围。
///
/// # 参数
///
/// - `x`: 光标的 X 屏幕坐标
/// - `y`: 光标的 Y 屏幕坐标
/// - `bounds`: 参考边界框（通常是网格填充的可视区域）
///
/// # 返回值
///
/// 返回归一化的 (u, v) 坐标元组，值可能超出 [0.0, 1.0] 范围。
///
/// # 边界情况处理
///
/// - 如果边界框的宽度或高度为零（或接近零），对应的坐标分量返回 0.0
/// - 使用 `f32::EPSILON` 进行浮点数零值判断，避免除以零错误
pub(super) fn cursor_to_uv_raw(x: f32, y: f32, bounds: Rectangle) -> (f64, f64) {
    // 计算 U 坐标：相对于边界框左侧的归一化位置
    // 如果宽度为零或接近零，返回 0.0 避免除零错误
    let u = if bounds.width.abs() <= f32::EPSILON { 0.0 } else { (x - bounds.x) / bounds.width };

    // 计算 V 坐标：相对于边界框顶部的归一化位置
    // 如果高度为零或接近零，返回 0.0 避免除零错误
    let v = if bounds.height.abs() <= f32::EPSILON { 0.0 } else { (y - bounds.y) / bounds.height };

    (u as f64, v as f64)
}

/// 获取网格点的句柄坐标数组
///
/// 返回指定索引处的网格点的所有控制句柄坐标。
/// 网格点有 4 个控制句柄（左、上、右、下），每个句柄由 2 个坐标值组成，
/// 因此总共返回 8 个 f64 值。
///
/// # 参数
///
/// - `mesh`: 网格填充对象的引用
/// - `point_index`: 网格点的索引
///
/// # 返回值
///
/// 返回包含 8 个 f64 值的数组，格式为：
/// - `[h0_x, h0_y, h1_x, h1_y, h2_x, h2_y, h3_x, h3_y]`
/// - 其中 h0-h3 分别代表左、上、右、下四个方向的控制句柄坐标
///
/// # 默认值
///
/// 如果句柄数据不存在或长度不足，返回网格点自身的坐标作为所有句柄的默认值，
/// 这意味着句柄与网格点重合，曲线将表现为直线段。
pub(super) fn mesh_point_handles(mesh: &MeshFill, point_index: usize) -> [f64; 8] {
    // 获取网格点的坐标 (x, y)
    let (x, y) = mesh
        .points
        .get(point_index)
        .map(|p| (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0)))
        .unwrap_or((0.0, 0.0));

    // 获取句柄数据
    let h = mesh.handles.get(point_index);

    // 如果句柄数据存在且长度足够，返回句柄坐标
    if let Some(h) = h
        && h.len() >= 8
    {
        [h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]]
    } else {
        // 否则返回默认值：所有句柄与网格点重合
        [x, y, x, y, x, y, x, y]
    }
}

/// 网格命中测试
///
/// 检测屏幕坐标 (x, y) 是否命中了网格中的某个点或控制句柄。
/// 支持检测网格点本身及其 4 个方向的控制句柄。
///
/// # 参数
///
/// - `mesh`: 网格填充对象的引用
/// - `bounds`: 网格填充的边界框（屏幕坐标）
/// - `x`: 光标的 X 屏幕坐标
/// - `y`: 光标的 Y 屏幕坐标
///
/// # 返回值
///
/// 如果命中，返回 `Some((point_index, drag_kind))`，其中：
/// - `point_index`: 被命中的网格点索引
/// - `drag_kind`: 命中类型（点本身或某个方向的控制句柄）
///
/// 如果未命中任何元素，返回 `None`。
///
/// # 命中检测优先级
///
/// 1. 首先检测当前选中点的控制句柄（如果有点被选中）
/// 2. 然后检测当前选中点本身
/// 3. 最后检测其他网格点（选择最近的）
///
/// # 检测半径
///
/// - 网格点检测半径：7.0 像素
/// - 控制句柄检测半径：7.0 像素
pub(super) fn hit_test_mesh(
    mesh: &MeshFill,
    bounds: Rectangle,
    x: f32,
    y: f32,
) -> Option<(usize, MeshDragKind)> {
    // 获取网格的行列数，确保最小值为 2
    let columns = mesh.columns.max(2);
    let rows = mesh.rows.max(2);
    let expected = columns.saturating_mul(rows);

    // 如果网格点数量为零，直接返回 None
    if expected == 0 {
        return None;
    }

    // 初始化点和句柄数组为默认网格
    let (mut points, mut handles) = MeshFill::default_points_and_handles(columns, rows);

    // 复制实际数据到点和句柄数组（带边界限制）
    let copy = mesh.points.len().min(expected);
    for i in 0..copy {
        if let Some(p) = mesh.points.get(i) {
            // 将点坐标限制在 [0.0, 1.0] 范围内
            let px = p.first().copied().unwrap_or(points[i][0]).clamp(0.0, 1.0);
            let py = p.get(1).copied().unwrap_or(points[i][1]).clamp(0.0, 1.0);
            points[i] = vec![px, py];
        }
    }

    // 复制句柄数据（确保每个句柄有 8 个值）
    let copy_h = mesh.handles.len().min(expected);
    for i in 0..copy_h {
        if let Some(h) = mesh.handles.get(i)
            && h.len() >= 8
        {
            handles[i] = vec![h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
        }
    }

    // 定义命中检测半径（像素）
    let point_radius = 7.0;
    let handle_radius = 7.0;

    // 检测当前选中点的句柄和点本身
    if let Some(sel_idx) = mesh.selected_point_index
        && sel_idx < expected
    {
        // 获取选中点的有效句柄
        let h = mesh.effective_handles(sel_idx);

        // 获取选中点的屏幕坐标
        let p = points.get(sel_idx).cloned().unwrap_or_else(|| vec![0.0, 0.0]);
        let px = bounds.x + (p.first().copied().unwrap_or(0.0) as f32) * bounds.width;
        let py = bounds.y + (p.get(1).copied().unwrap_or(0.0) as f32) * bounds.height;
        let p_screen = Point::new(px, py);

        // 计算选中点在网格中的行列位置
        let sel_r = sel_idx / columns.max(1);
        let sel_c = sel_idx % columns.max(1);

        // 判断选中点在各方向是否有相邻点（边界检测）
        let has_left = sel_c > 0;
        let has_top = sel_r > 0;
        let has_right = sel_c + 1 < columns;
        let has_bottom = sel_r + 1 < rows;

        // 定义 4 个方向句柄的检测候选
        // (句柄索引, 是否启用)
        let candidates = [(0, has_left), (1, has_top), (2, has_right), (3, has_bottom)];

        // 检测各个方向的控制句柄
        for (hi, enabled) in candidates {
            if !enabled {
                continue;
            }
            // 计算句柄的屏幕坐标
            let hx = h[hi * 2] as f32;
            let hy = h[hi * 2 + 1] as f32;
            let hp = Point::new(bounds.x + hx * bounds.width, bounds.y + hy * bounds.height);

            // 检测是否命中句柄
            if hp.distance(Point::new(x, y)) <= handle_radius {
                return Some((sel_idx, MeshDragKind::Handle(hi)));
            }
        }

        // 检测是否命中选中点本身
        if p_screen.distance(Point::new(x, y)) <= point_radius {
            return Some((sel_idx, MeshDragKind::Point));
        }
    }

    // 检测其他网格点（选择最近的命中点）
    let mut best: Option<(usize, f32)> = None;
    let cursor = Point::new(x, y);

    for i in 0..expected {
        // 计算网格点的屏幕坐标
        let p = points.get(i).cloned().unwrap_or_else(|| vec![0.0, 0.0]);
        let px = bounds.x + (p.first().copied().unwrap_or(0.0) as f32) * bounds.width;
        let py = bounds.y + (p.get(1).copied().unwrap_or(0.0) as f32) * bounds.height;
        let d = cursor.distance(Point::new(px, py));

        // 如果在检测半径内，且比当前最佳候选更近，则更新最佳候选
        if d <= point_radius {
            best = match best {
                Some((_, best_d)) if best_d <= d => best,
                _ => Some((i, d)),
            };
        }
    }

    // 返回最佳候选点
    best.map(|(i, _)| (i, MeshDragKind::Point))
}

/// 更新网格拖拽状态
///
/// 根据拖拽操作的当前光标位置，更新网格点或控制句柄的位置。
/// 支持拖拽网格点本身和拖拽控制句柄两种模式。
///
/// # 参数
///
/// - `mesh`: 可变引用的网格填充对象
/// - `drag`: 当前拖拽状态的引用
/// - `bounds`: 网格填充的边界框（屏幕坐标）
/// - `x`: 当前光标的 X 屏幕坐标
/// - `y`: 当前光标的 Y 屏幕坐标
///
/// # 返回值
///
/// 如果网格数据发生了变化，返回 `true`；否则返回 `false`。
///
/// # 拖拽行为
///
/// ## 拖拽网格点
/// - 网格点坐标限制在 [0.0, 1.0] 范围内
/// - 所有 4 个控制句柄随网格点同步移动
/// - 句柄坐标限制在 [-0.5, 1.5] 范围内（允许一定程度的超出）
///
/// ## 拖拽控制句柄
/// - 仅更新指定方向的控制句柄坐标
/// - 句柄坐标限制在 [-0.5, 1.5] 范围内
///
/// # 性能优化
///
/// 函数会比较新旧值，仅在值真正发生变化时才更新数据，
/// 避免不必要的内存写入和后续的重绘操作。
pub(super) fn update_mesh_drag(
    mesh: &mut MeshFill,
    drag: &MeshDragState,
    bounds: Rectangle,
    x: f32,
    y: f32,
) -> bool {
    // 计算当前光标的 UV 坐标（原始值，不截断）
    let (u, v) = cursor_to_uv_raw(x, y, bounds);

    // 计算从拖拽开始到当前的 UV 坐标偏移量
    let du = u - drag.start_cursor_u;
    let dv = v - drag.start_cursor_v;

    let idx = drag.point_index;

    // 边界检查：确保索引有效
    if idx >= mesh.points.len() || idx >= mesh.handles.len() {
        return false;
    }

    // 判断两个浮点数是否不同的辅助闭包
    let differs = |a: f64, b: f64| (a - b).abs() > 1e-9;

    // 定义句柄坐标的边界限制
    let handle_min = -0.5;
    let handle_max = 1.5;

    match drag.kind {
        MeshDragKind::Point => {
            // 计算网格点的新位置（限制在 [0.0, 1.0] 范围内）
            let nx = (drag.start_point_x + du).clamp(0.0, 1.0);
            let ny = (drag.start_point_y + dv).clamp(0.0, 1.0);

            // 计算实际的有效偏移量（考虑边界限制后）
            let eff_du = nx - drag.start_point_x;
            let eff_dv = ny - drag.start_point_y;

            // 计算所有句柄的新位置（随网格点同步移动）
            let next_handles = [
                (drag.start_handles[0] + eff_du).clamp(handle_min, handle_max),
                (drag.start_handles[1] + eff_dv).clamp(handle_min, handle_max),
                (drag.start_handles[2] + eff_du).clamp(handle_min, handle_max),
                (drag.start_handles[3] + eff_dv).clamp(handle_min, handle_max),
                (drag.start_handles[4] + eff_du).clamp(handle_min, handle_max),
                (drag.start_handles[5] + eff_dv).clamp(handle_min, handle_max),
                (drag.start_handles[6] + eff_du).clamp(handle_min, handle_max),
                (drag.start_handles[7] + eff_dv).clamp(handle_min, handle_max),
            ];

            // 获取当前的点和句柄数据
            let cur_p = mesh.points.get(idx);
            let cur_h = mesh.handles.get(idx);

            // 检查点坐标是否发生变化
            let point_changed = match cur_p {
                Some(p) => {
                    let px = p.first().copied().unwrap_or(0.0);
                    let py = p.get(1).copied().unwrap_or(0.0);
                    differs(px, nx) || differs(py, ny)
                }
                None => true,
            };

            // 检查句柄坐标是否发生变化
            let handles_changed = match cur_h {
                Some(h) => {
                    if h.len() < 8 {
                        true
                    } else {
                        (0..8).any(|i| differs(h[i], next_handles[i]))
                    }
                }
                None => true,
            };

            // 仅在值发生变化时更新数据
            if point_changed {
                mesh.points[idx] = vec![nx, ny];
            }
            if handles_changed {
                mesh.handles[idx] = next_handles.to_vec();
            }

            point_changed || handles_changed
        }
        MeshDragKind::Handle(hi) => {
            // 计算句柄在数组中的基础索引（每个句柄占用 2 个位置）
            let base = hi.saturating_mul(2);

            // 边界检查
            if base + 1 >= 8 {
                return false;
            }

            // 计算句柄的新位置
            let mut next = drag.start_handles;
            next[base] = (next[base] + du).clamp(handle_min, handle_max);
            next[base + 1] = (next[base + 1] + dv).clamp(handle_min, handle_max);

            // 检查句柄坐标是否发生变化
            let handles_changed = match mesh.handles.get(idx) {
                Some(h) => h.len() < 8 || (0..8).any(|i| differs(h[i], next[i])),
                None => true,
            };

            // 仅在值发生变化时更新数据
            if handles_changed {
                mesh.handles[idx] = next.to_vec();
            }

            handles_changed
        }
    }
}

/// 生成网格曲线变更载荷
///
/// 为前端渲染生成贝塞尔曲线的 JSON 数据载荷。
/// 根据网格点和控制句柄的位置，计算出从一个网格点到相邻网格点的贝塞尔曲线路径。
///
/// # 参数
///
/// - `mesh`: 网格填充对象的引用
/// - `point_index`: 网格点索引
/// - `kind`: 拖拽类型（点或某个方向的句柄）
///
/// # 返回值
///
/// 返回包含路径信息的 JSON 对象，格式如下：
///
/// ```json
/// {
///   "pointIndex": 5,
///   "kind": "point" | "handle",
///   "paths": [
///     {
///       "from": 5,
///       "to": 6,
///       "handle": 2,
///       "d": "M 0.5 0.5 C 0.6 0.5 0.7 0.5 0.8 0.5",
///       "start": [0.5, 0.5],
///       "c1": [0.6, 0.5],
///       "c2": [0.7, 0.5],
///       "end": [0.8, 0.5]
///     }
///   ]
/// }
/// ```
///
/// # 曲线计算
///
/// 对于每个方向（左、上、右、下），如果该方向有相邻网格点：
/// 1. 获取起始点和终止点的坐标
/// 2. 获取起始点在该方向的控制句柄坐标（控制点1）
/// 3. 获取终止点在相反方向的控制句柄坐标（控制点2）
/// 4. 生成 SVG 路径字符串（三次贝塞尔曲线）
///
/// # 注意
///
/// - 仅当拖拽类型为 `Point` 时，才生成所有 4 个方向的曲线数据
/// - 拖拽单个句柄时，仅生成该句柄方向的曲线数据
pub(super) fn mesh_curve_change_payload(
    mesh: &MeshFill,
    point_index: usize,
    kind: MeshDragKind,
) -> serde_json::Value {
    // 获取网格的行列数
    let columns = mesh.columns.max(2);
    let rows = mesh.rows.max(2);
    let expected = columns.saturating_mul(rows);

    // 边界检查
    if expected == 0 || point_index >= expected {
        return serde_json::Value::Null;
    }

    // 格式化浮点数为 4 位小数的辅助函数
    let fmt = |v: f64| -> String { format!("{:.4}", v) };

    // 获取指定索引的网格点坐标
    let point_xy = |idx: usize| -> (f64, f64) {
        mesh.points
            .get(idx)
            .map(|p| {
                (
                    p.first().copied().unwrap_or(0.0).clamp(0.0, 1.0),
                    p.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0),
                )
            })
            .unwrap_or((0.0, 0.0))
    };

    // 获取指定点和句柄方向的控制点坐标
    let handle_xy = |idx: usize, hi: usize| -> (f64, f64) {
        let (px, py) = point_xy(idx);
        // 如果句柄数据存在且有效，返回句柄坐标；否则返回点坐标作为默认值
        if let Some(h) = mesh.handles.get(idx)
            && h.len() >= 8
            && hi < 4
        {
            (h[hi * 2], h[hi * 2 + 1])
        } else {
            (px, py)
        }
    };

    // 获取相反方向的句柄索引
    // 0(左) <-> 2(右), 1(上) <-> 3(下)
    let opposite = |hi: usize| match hi {
        0 => 2,
        2 => 0,
        1 => 3,
        3 => 1,
        _ => 0,
    };

    // 获取指定方向的相邻网格点索引
    let neighbor = |idx: usize, hi: usize| -> Option<usize> {
        // 计算当前点的行列位置
        let r = idx / columns.max(1);
        let c = idx % columns.max(1);

        // 根据方向返回相邻点的索引
        match hi {
            0 if c > 0 => Some(idx - 1),              // 左：列号 > 0
            2 if c + 1 < columns => Some(idx + 1),    // 右：列号 < 最大列号
            1 if r > 0 => Some(idx - columns),        // 上：行号 > 0
            3 if r + 1 < rows => Some(idx + columns), // 下：行号 < 最大行号
            _ => None,                                // 边界处无相邻点
        }
    };

    // 判断是否需要包含某个方向的曲线
    let include = |hi: usize| -> bool {
        match kind {
            MeshDragKind::Handle(sel) => sel == hi, // 单个句柄：仅包含选中方向
            MeshDragKind::Point => true,            // 网格点：包含所有方向
        }
    };

    // 生成所有方向的曲线路径数据
    let mut paths: Vec<serde_json::Value> = Vec::new();
    for hi in 0..4 {
        // 跳过不需要的方向
        if !include(hi) {
            continue;
        }

        // 获取相邻点的索引，如果没有则跳过
        let Some(to_idx) = neighbor(point_index, hi) else {
            continue;
        };

        // 获取起始点和终止点坐标
        let (sx, sy) = point_xy(point_index);
        let (ex, ey) = point_xy(to_idx);

        // 获取控制点坐标
        let (c1x, c1y) = handle_xy(point_index, hi); // 起始点的句柄
        let (c2x, c2y) = handle_xy(to_idx, opposite(hi)); // 终止点的相反方向句柄

        // 生成 SVG 三次贝塞尔曲线路径字符串
        let d = format!(
            "M {} {} C {} {} {} {} {} {}",
            fmt(sx),
            fmt(sy),
            fmt(c1x),
            fmt(c1y),
            fmt(c2x),
            fmt(c2y),
            fmt(ex),
            fmt(ey)
        );

        // 构建路径 JSON 对象
        paths.push(serde_json::json!({
            "from": point_index,
            "to": to_idx,
            "handle": hi,
            "d": d,
            "start": [sx, sy],
            "c1": [c1x, c1y],
            "c2": [c2x, c2y],
            "end": [ex, ey],
        }));
    }

    // 构建并返回最终的载荷对象
    serde_json::json!({
        "pointIndex": point_index,
        "kind": match kind { MeshDragKind::Point => "point", MeshDragKind::Handle(_) => "handle" },
        "paths": paths,
    })
}

#[cfg(test)]
#[path = "mesh_tests.rs"]
mod mesh_tests;
