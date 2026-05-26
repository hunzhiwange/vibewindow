//! # Mesh Fill Actions 模块
//!
//! 本模块提供网格填充（Mesh Fill）属性编辑的各种操作函数。
//!
//! ## 主要功能
//!
//! - **网格结构调整**：更新网格的行列数，智能保留已有数据
//! - **网格属性设置**：配置轮廓、镜像等视觉属性
//! - **颜色操作**：随机打乱、重新生成、统一设置网格颜色
//! - **网格点操作**：选中、重置位置、重置曲线控制柄
//!
//! ## 设计原则
//!
//! 所有函数均返回 `Message` 类型，用于在 Iced 框架中触发 UI 更新。
//! 函数参数遵循不可变借用优先原则，内部通过克隆或可变引用进行修改。

use web_time::{SystemTime, UNIX_EPOCH};

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject, MeshFill};

use super::utils::next_seed;

/// 更新网格的行列数配置
///
/// 该函数用于调整网格填充的行列数，并智能处理数据的保留与扩展：
/// - 当网格扩大时，新增的单元格会使用随机生成的颜色和默认位置
/// - 当网格缩小时，超出的数据会被丢弃
/// - 原有单元格的颜色、位置和控制柄会尽可能保留
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表，包含所有填充配置
/// - `index`: 目标网格填充在列表中的索引位置
/// - `cols`: 新的列数（会被 clamp 到 2-6 的范围）
/// - `rows`: 新的行数（会被 clamp 到 2-6 的范围）
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`，用于触发属性更新
///
/// # 示例
///
/// ```ignore
/// let msg = update_mesh_grid_cells(element_id, fills, 0, 3, 4);
/// // 将第0个填充项的网格调整为4列5行
/// ```
pub(super) fn update_mesh_grid_cells(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    cols: usize,
    rows: usize,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    // 尝试获取目标索引处的网格填充对象
    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 将行列数限制在有效范围内 [2, 6]
        let new_columns = (cols + 1).clamp(2, 6);
        let new_rows = (rows + 1).clamp(2, 6);

        // 保存旧的行列数，用于后续的数据迁移
        let old_columns = m.columns;
        let old_rows = m.rows;

        // 更新网格尺寸
        m.columns = new_columns;
        m.rows = new_rows;

        // 计算新的单元格总数
        let new_count = new_columns.saturating_mul(new_rows);

        // 生成新的颜色数组和位置/控制柄数据
        let mut new_colors = MeshFill::random_colors(new_count);
        let (mut new_points, mut new_handles) =
            MeshFill::default_points_and_handles(new_columns, new_rows);

        // 计算需要复制的行列数（取旧尺寸和新尺寸的较小值）
        let copy_cols = old_columns.min(new_columns);
        let copy_rows = old_rows.min(new_rows);

        // 遍历需要保留的单元格，将旧数据迁移到新数组中
        for r in 0..copy_rows {
            for c in 0..copy_cols {
                // 计算旧数组和新数组中的索引位置
                let old_idx = r * old_columns + c;
                let new_idx = r * new_columns + c;

                // 复制颜色数据
                if let Some(src) = m.colors.get(old_idx).cloned()
                    && let Some(dst) = new_colors.get_mut(new_idx) {
                        *dst = src;
                    }
                // 复制点位置数据
                if let Some(src) = m.points.get(old_idx).cloned()
                    && let Some(dst) = new_points.get_mut(new_idx) {
                        *dst = src;
                    }
                // 复制控制柄数据
                if let Some(src) = m.handles.get(old_idx).cloned()
                    && let Some(dst) = new_handles.get_mut(new_idx) {
                        *dst = src;
                    }
            }
        }

        // 应用新的颜色、位置和控制柄数据
        m.colors = new_colors;
        m.points = new_points;
        m.handles = new_handles;

        // 如果当前选中的点索引超出新的范围，则清除选择
        if m.selected_point_index.is_some_and(|idx| idx >= new_count) {
            m.selected_point_index = None;
        }
    }

    // 构造并发送属性更新消息
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 更新网格的轮廓显示设置
///
/// 启用或禁用网格填充的轮廓边框显示。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
/// - `val`: 是否显示轮廓（true = 显示，false = 隐藏）
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn update_mesh_outline(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: bool,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        m.outline = val;
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod actions_tests;

/// 更新网格的镜像模式设置
///
/// 配置网格填充的镜像对称模式，支持水平、垂直或无镜像。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
/// - `val`: 镜像模式，`None` 表示无镜像，`Some(String)` 表示具体的镜像方向
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn update_mesh_mirroring(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: Option<String>,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        m.mirroring = val;
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 设置网格中选中点的索引
///
/// 当用户点击网格中的某个控制点时，记录该点的索引以便后续操作。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
/// - `pt_idx`: 被选中的点的索引
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn update_mesh_selection(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    pt_idx: usize,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        m.selected_point_index = Some(pt_idx);
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 清除网格的选中状态
///
/// 取消当前网格中所有点的选中状态。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn clear_mesh_selection(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        m.selected_point_index = None;
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 随机打乱网格颜色分布
///
/// 使用 Fisher-Yates 洗牌算法对网格中的颜色进行随机重排，
/// 实现颜色在网格单元之间的交换效果。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
///
/// # 注意
///
/// 如果当前有选中的点，则不会执行打乱操作，直接返回原数据
pub(super) fn shuffle_mesh_colors(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 如果有选中的点，则不打乱颜色
        if m.selected_point_index.is_some() {
            return Message::Design(DesignMessage::PropertyUpdate(
                id,
                "fill".to_string(),
                json!(new_fills),
            ));
        }

        // 使用时间戳和颜色数量生成随机种子
        // 与魔数 0x9E37_79B9_7F4A_7C15 进行异或运算以增加随机性
        let mut seed =
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
                ^ (m.colors.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);

        // Fisher-Yates 洗牌算法：从后向前遍历并随机交换
        for i in (1..m.colors.len()).rev() {
            seed = next_seed(seed);
            let j = (seed as usize) % (i + 1);
            m.colors.swap(i, j);
        }
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 重新生成所有网格颜色
///
/// 使用随机算法为网格中的所有单元格生成全新的颜色，
/// 同时清除当前的选中状态。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
///
/// # 注意
///
/// 如果当前有选中的点，则不会执行重新生成操作
pub(super) fn regenerate_mesh_colors(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 如果有选中的点，则不重新生成颜色
        if m.selected_point_index.is_some() {
            return Message::Design(DesignMessage::PropertyUpdate(
                id,
                "fill".to_string(),
                json!(new_fills),
            ));
        }

        // 计算单元格总数并生成新的随机颜色
        let count = m.columns.saturating_mul(m.rows);
        m.colors = MeshFill::random_colors(count);
        m.selected_point_index = None;
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 更新指定网格点的颜色
///
/// 为指定索引的网格点随机生成一个新的颜色。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
/// - `point_index`: 需要更新的点的索引
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn update_mesh_selected_color(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    point_index: usize,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index)
        && let Some(dst) = m.colors.get_mut(point_index) {
            // 生成一个随机颜色，失败时使用白色作为默认值
            let next = MeshFill::random_colors(1).pop().unwrap_or_else(|| "#ffffff".to_string());
            *dst = next;
        }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 重置所有网格点的位置和控制柄
///
/// 将网格中所有点的位置和曲线控制柄恢复到默认的均匀分布状态，
/// 同时清除当前的选中状态。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn reset_mesh_positions(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 生成默认的点位置和控制柄配置
        let (points, handles) = MeshFill::default_points_and_handles(m.columns, m.rows);
        m.points = points;
        m.handles = handles;
        m.selected_point_index = None;
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 重置当前选中网格点的位置
///
/// 将当前选中点的位置恢复到默认的均匀分布位置，
/// 同时重置该点的曲线控制柄。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
///
/// # 注意
///
/// 如果没有选中任何点，则不会执行任何操作
pub(super) fn reset_selected_mesh_position(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 如果没有选中的点，直接返回
        let Some(pt_idx) = m.selected_point_index else {
            return Message::Design(DesignMessage::PropertyUpdate(
                id,
                "fill".to_string(),
                json!(new_fills),
            ));
        };

        // 获取默认的点位置和控制柄配置
        let (default_points, default_handles) =
            MeshFill::default_points_and_handles(m.columns, m.rows);

        // 将选中点的位置和控制柄重置为默认值
        if let (Some(p), Some(h)) =
            (default_points.get(pt_idx).cloned(), default_handles.get(pt_idx).cloned())
        {
            if let Some(dst) = m.points.get_mut(pt_idx) {
                *dst = p;
            }
            if let Some(dst) = m.handles.get_mut(pt_idx) {
                *dst = h;
            }
        }
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 重置当前选中网格点的曲线控制柄
///
/// 将选中点的贝塞尔曲线控制柄重置为默认状态，
/// 使该点的四个控制柄都指向点本身的位置（产生平滑效果）。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
///
/// # 控制柄格式
///
/// 每个点有8个控制柄值（4个控制点，每个2个坐标）：
/// `[x1, y1, x2, y2, x3, y3, x4, y4]`
/// 重置后所有控制点都位于点本身的位置
pub(super) fn reset_selected_mesh_curve(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 如果没有选中的点，直接返回
        let Some(pt_idx) = m.selected_point_index else {
            return Message::Design(DesignMessage::PropertyUpdate(
                id,
                "fill".to_string(),
                json!(new_fills),
            ));
        };

        // 获取选中点的位置坐标
        let (x, y) = m
            .points
            .get(pt_idx)
            .map(|p| (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0)))
            .unwrap_or((0.0, 0.0));

        // 将控制柄重置为点本身的位置
        // 格式：[x, y, x, y, x, y, x, y] 表示4个控制点
        if let Some(h) = m.handles.get_mut(pt_idx) {
            *h = vec![x, y, x, y, x, y, x, y];
        }
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 将指定颜色应用到网格的所有单元格
///
/// 统一设置网格中所有单元格的颜色为同一个值。
///
/// # 参数
///
/// - `id`: 设计元素的唯一标识符
/// - `fills`: 填充项列表
/// - `index`: 目标网格填充的索引位置
/// - `color`: 要应用的颜色字符串（如 "#FF5733"）
///
/// # 返回值
///
/// 返回包含更新后填充数据的 `Message`
pub(super) fn apply_mesh_color_to_all(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    color: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;

    if let Some(FillItem::Object(FillObject::Mesh(m))) = new_fills.get_mut(index) {
        // 遍历所有颜色单元格，设置为统一的颜色
        for c in &mut m.colors {
            *c = color.clone();
        }
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}
