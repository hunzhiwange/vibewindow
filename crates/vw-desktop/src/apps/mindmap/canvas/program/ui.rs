//! UI 区域检测模块
//!
//! 本模块提供思维导图画布中 UI 元素区域检测功能，
//! 主要用于判断鼠标光标是否位于阻止画布交互的 UI 区域内。
//!
//! # 功能
//!
//! - 检测光标是否在工具栏区域内
//! - 检测光标是否在画笔面板区域内
//! - 检测光标是否在自定义阻止区域内
//!
//! # 使用场景
//!
//! 当用户在 UI 控件上操作时，需要阻止画布的拖拽、缩放等交互行为，
//! 本模块提供统一的区域检测逻辑。

use crate::apps::mindmap::state::MindMapCanvasTool;
use iced::{Point, Rectangle, Size};

use super::{
    DragMode, PEN_PANEL_GAP, PEN_PANEL_H, PEN_PANEL_W, TOOLBAR_H, TOOLBAR_MARGIN, TOOLBAR_W,
};

/// 检测光标是否位于阻止画布交互的 UI 区域内
///
/// 此函数用于判断鼠标光标是否位于需要阻止画布交互的区域，
/// 例如工具栏、画笔面板或其他自定义 UI 控件区域。
///
/// # 参数
///
/// * `bounds` - 画布的边界矩形，用于计算 UI 元素的位置约束
/// * `cursor_pos_in_bounds` - 光标在画布坐标系中的位置，若光标不在画布内则为 `None`
/// * `drag_mode` - 当前的拖拽模式，非 `DragMode::None` 时直接返回 `false`
/// * `canvas_tool` - 当前激活的画布工具类型
/// * `ui_blocked_rects` - 自定义阻止区域列表，这些区域会被进行边界约束后检测
///
/// # 返回值
///
/// 返回 `true` 表示光标位于阻止区域内，需要阻止画布交互；
/// 返回 `false` 表示光标不在阻止区域，可以正常进行画布交互。
///
/// # 检测逻辑
///
/// 1. 如果当前处于拖拽模式（`DragMode::None` 之外），直接返回 `false`
/// 2. 如果光标不在画布内，直接返回 `false`
/// 3. 检测光标是否在工具栏区域内
/// 4. 如果当前工具是画笔，额外检测光标是否在画笔面板区域内
/// 5. 检测光标是否在任何自定义阻止区域内（会对这些区域进行边界约束）
///
/// # 示例
///
/// ```ignore
/// let is_blocked = cursor_in_blocked_ui(
///     canvas_bounds,
///     cursor_position,
///     &drag_mode,
///     current_tool,
///     &custom_blocked_areas,
/// );
///
/// if is_blocked {
///     // 不处理画布拖拽/缩放等交互
/// } else {
///     // 正常处理画布交互
/// }
/// ```
pub(super) fn cursor_in_blocked_ui(
    bounds: Rectangle,
    cursor_pos_in_bounds: Option<Point>,
    drag_mode: &DragMode,
    canvas_tool: MindMapCanvasTool,
    ui_blocked_rects: &[Rectangle],
) -> bool {
    // 如果当前处于拖拽模式，不阻止画布交互
    // 拖拽时用户可能需要跨 UI 元素移动，因此不进行阻止
    if !matches!(drag_mode, DragMode::None) {
        return false;
    }

    // 如果光标不在画布内，不阻止交互
    let Some(p) = cursor_pos_in_bounds else {
        return false;
    };

    // 计算工具栏的位置矩形
    // 工具栏水平居中，距离顶部有固定的边距
    let toolbar_rect = Rectangle::new(
        Point::new((bounds.width - TOOLBAR_W).max(0.0) / 2.0, TOOLBAR_MARGIN),
        Size::new(TOOLBAR_W, TOOLBAR_H),
    );

    // 计算画笔面板的位置矩形
    // 画笔面板水平居中，位于工具栏下方，之间有固定间距
    let pen_panel_rect = Rectangle::new(
        Point::new(
            (bounds.width - PEN_PANEL_W).max(0.0) / 2.0,
            TOOLBAR_MARGIN + TOOLBAR_H + PEN_PANEL_GAP,
        ),
        Size::new(PEN_PANEL_W, PEN_PANEL_H),
    );

    // 创建带边界约束的区域检测闭包
    // 此闭包会将给定矩形约束在画布边界内，然后检测点是否在约束后的区域内
    //
    // 边界约束逻辑：
    // - 计算矩形在 x 和 y 方向上的最大可移动距离
    // - 将矩形的 x 和 y 坐标限制在有效范围内
    // - 使用约束后的矩形进行包含检测
    let snapped_rect_contains = |rect: &Rectangle, p: Point| {
        // 计算矩形在画布内的最大 x 坐标（确保矩形右边缘不超过画布右边缘）
        let max_x = (bounds.width - rect.width).max(0.0);
        // 计算矩形在画布内的最大 y 坐标（确保矩形下边缘不超过画布下边缘）
        let max_y = (bounds.height - rect.height).max(0.0);
        // 将矩形的 x 坐标约束在 [0, max_x] 范围内
        let x = rect.x.clamp(0.0, max_x);
        // 将矩形的 y 坐标约束在 [0, max_y] 范围内
        let y = rect.y.clamp(0.0, max_y);
        // 创建约束后的矩形并检测点是否在内部
        Rectangle::new(Point::new(x, y), Size::new(rect.width, rect.height)).contains(p)
    };

    // 检测光标是否在任何阻止区域内：
    // 1. 工具栏区域（始终检测）
    // 2. 画笔面板区域（仅当工具为画笔时检测）
    // 3. 自定义阻止区域（带边界约束）
    toolbar_rect.contains(p)
        || (canvas_tool == MindMapCanvasTool::Pen && pen_panel_rect.contains(p))
        || ui_blocked_rects.iter().any(|r| snapped_rect_contains(r, p))
}
