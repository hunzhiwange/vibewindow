//! 鼠标交互模块
//!
//! 本模块负责计算画布上鼠标光标的交互样式（如指针、抓取、十字准星等），
//! 根据当前光标位置、画布状态和工具类型决定最合适的鼠标交互反馈。
//!
//! # 主要功能
//!
//! - 检测光标是否位于被阻止的 UI 区域
//! - 根据拖拽模式返回相应的交互样式
//! - 检测光标是否悬停在节点的 URL 图标上
//! - 检测光标是否悬停在折叠/展开按钮上
//! - 检测光标是否悬停在添加子节点/兄弟节点按钮上

use super::super::layout::compute_layout_for_diagram;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::MindMapCanvasTool;
use iced::{Point, Rectangle, mouse};

use super::hit_test::descendant_count;
use super::ui::cursor_in_blocked_ui;
use super::{DragMode, HoverButtonKind, MindMapCanvas, MindMapCanvasState};

/// 计算画布上鼠标光标的交互样式
///
/// 该函数根据当前光标位置、画布状态、工具类型等多个因素，决定鼠标光标应显示的交互样式。
/// 交互样式用于向用户提供视觉反馈，指示可执行的操作类型。
///
/// # 参数
///
/// - `canvas`: 思维导图画布的引用，包含节点数据、布局信息等
/// - `state`: 画布状态的引用，包含当前拖拽模式、悬停节点等信息
/// - `bounds`: 画布的边界矩形，用于坐标转换
/// - `cursor`: 鼠标光标状态，包含当前位置信息
///
/// # 返回值
///
/// 返回 [`mouse::Interaction`] 枚举值，表示鼠标光标的交互样式：
/// - `Idle`: 默认光标，表示无特殊交互
/// - `Grabbing`: 抓取光标，表示正在拖拽画布或节点
/// - `Crosshair`: 十字准星光标，表示绘制或擦除模式
/// - `Pointer`: 指针光标，表示可点击的交互元素
///
/// # 交互逻辑优先级
///
/// 1. **被阻止的 UI 区域**：如果光标位于被阻止的 UI 区域，返回 `Idle`
/// 2. **拖拽模式**：
///    - `Pan`（平移）或 `Node`（节点拖拽）时返回 `Grabbing`
///    - `DoodlePen`（涂鸦笔）或 `DoodleErase`（涂鸦擦除）时返回 `Crosshair`
/// 3. **工具模式**：
///    - `Pen`（画笔）或 `Eraser`（橡皮擦）时返回 `Crosshair`
///    - 非 `Select`（选择）工具时返回 `Idle`
/// 4. **可点击元素**（仅在 `Select` 工具模式下检测）：
///    - 节点的 URL 图标
///    - 折叠/展开按钮
///    - 添加子节点/兄弟节点按钮
///
/// # 示例
///
/// ```ignore
/// let interaction = mouse_interaction(&canvas, &state, bounds, cursor);
/// // 根据返回的交互样式更新光标外观
/// ```
pub(super) fn mouse_interaction(
    canvas: &MindMapCanvas<'_>,
    state: &MindMapCanvasState,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> mouse::Interaction {
    // 获取光标在画布边界内的位置（如果存在）
    let cursor_pos_in_bounds = cursor.position_in(bounds);

    // 检查光标是否位于被阻止的 UI 区域
    // 被阻止的区域通常包含模态对话框、菜单等不应与画布交互的元素
    let cursor_in_blocked_ui = cursor_in_blocked_ui(
        bounds,
        cursor_pos_in_bounds,
        &state.drag_mode,
        canvas.canvas_tool,
        &canvas.ui_blocked_rects,
    );

    // 如果光标在被阻止的 UI 区域，返回默认光标
    if cursor_in_blocked_ui {
        return mouse::Interaction::Idle;
    }

    // 检查是否处于拖拽模式（平移画布或拖拽节点）
    // 此时应显示抓取光标
    if matches!(state.drag_mode, DragMode::Pan | DragMode::Node(_)) {
        return mouse::Interaction::Grabbing;
    }

    // 检查是否处于涂鸦模式（绘制或擦除）
    // 此时应显示十字准星光标，方便精确定位
    if matches!(state.drag_mode, DragMode::DoodlePen | DragMode::DoodleErase) {
        return mouse::Interaction::Crosshair;
    }

    // 如果光标不在画布边界内，返回默认光标
    let Some(cursor_pos) = cursor_pos_in_bounds else {
        return mouse::Interaction::Idle;
    };

    // 检查当前工具是否为画笔或橡皮擦
    // 这些工具需要精确定位，因此显示十字准星光标
    if matches!(canvas.canvas_tool, MindMapCanvasTool::Pen | MindMapCanvasTool::Eraser) {
        return mouse::Interaction::Crosshair;
    }

    // 如果不是选择工具，返回默认光标（不进行后续的可点击元素检测）
    if canvas.canvas_tool != MindMapCanvasTool::Select {
        return mouse::Interaction::Idle;
    }

    // 计算当前画布的完整布局
    // 这包括所有节点的位置、大小等信息
    let layout = compute_layout_for_diagram(
        canvas.doc,
        canvas.node_positions,
        canvas.node_priorities,
        canvas.node_urls,
        canvas.collapsed_paths,
        canvas.diagram_type,
        canvas.layout_format,
        canvas.org_chart_layout_format,
        canvas.fishbone_layout_format,
        canvas.timeline_layout_format,
        canvas.bracket_layout_format,
        canvas.tree_layout_format,
    );

    // 检查光标是否悬停在节点的 URL 图标上
    // URL 图标显示在节点的右侧，点击可跳转到链接
    for n in &layout.nodes {
        // 获取节点的 URL，如果没有或为空则跳过
        let Some(url) = canvas.node_urls.get(&n.path) else {
            continue;
        };
        if url.trim().is_empty() {
            continue;
        }

        // 计算节点在屏幕上的矩形区域
        let rect = canvas.node_screen_rect(n);

        // 计算 URL 图标的半径（根据缩放级别调整，限制在 4-12 像素之间）
        let r = (8.0 * canvas.zoom).clamp(4.0, 12.0);

        // 计算 URL 图标的内边距（根据缩放级别调整，限制在 4-10 像素之间）
        let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);

        // 计算 URL 图标的中心位置（节点右侧）
        let center = Point::new(rect.x + rect.width - pad - r, rect.y + rect.height / 2.0);

        // 计算光标到 URL 图标中心的距离（使用距离平方避免开方运算）
        let dx = cursor_pos.x - center.x;
        let dy = cursor_pos.y - center.y;

        // 如果光标在 URL 图标的圆形区域内，返回指针光标
        if dx * dx + dy * dy <= r * r {
            return mouse::Interaction::Pointer;
        }
    }

    // 检查光标是否悬停在折叠/展开按钮上
    // 这些按钮用于控制节点子树的显示/隐藏
    for n in &layout.nodes {
        // 获取节点，如果没有子节点则跳过（没有折叠/展开的必要）
        let Some(node) = mind_map::node(canvas.doc, &n.path) else {
            continue;
        };
        if node.children.is_empty() {
            continue;
        }

        // 计算节点在屏幕上的矩形区域
        let rect = canvas.node_screen_rect(n);

        // 查找折叠/展开按钮的位置和大小
        let mut toggle: Option<(Point, f32)> = None;
        for (kind, center, r) in canvas.node_button_specs(&n.path, rect) {
            if kind == HoverButtonKind::ToggleCollapse {
                toggle = Some((center, r));
                break;
            }
        }

        // 如果没有找到折叠/展开按钮，跳过该节点
        let Some((center, r)) = toggle else {
            continue;
        };

        // 计算光标到折叠/展开按钮中心的距离
        let dx = cursor_pos.x - center.x;
        let dy = cursor_pos.y - center.y;

        // 如果光标在折叠/展开按钮的圆形区域内，返回指针光标
        if dx * dx + dy * dy <= r * r {
            return mouse::Interaction::Pointer;
        }

        // 如果节点已折叠，检查光标是否悬停在子节点数量徽章上
        // 徽章显示被隐藏的子节点总数
        if canvas.collapsed_paths.contains(&n.path) {
            // 计算所有后代节点的总数
            let child_count = descendant_count(node);

            // 如果有后代节点，检查光标是否在徽章区域内
            if child_count > 0 {
                let badge_rect = canvas.collapsed_count_badge_rect(&n.path, center, r, child_count);
                if badge_rect.contains(cursor_pos) {
                    return mouse::Interaction::Pointer;
                }
            }
        }
    }

    // 检查光标是否悬停在当前悬停节点的添加按钮上
    // 这些按钮用于快速添加子节点或兄弟节点
    if let Some(node_path) = state.hovered_node.as_ref()
        && let Some(node) = layout.nodes.iter().find(|n| &n.path == node_path)
    {
        // 计算悬停节点在屏幕上的矩形区域
        let rect = canvas.node_screen_rect(node);

        // 检查所有按钮，只关注添加子节点和添加兄弟节点按钮
        for (kind, center, r) in canvas.node_button_specs(node_path, rect) {
            if !matches!(kind, HoverButtonKind::AddChild | HoverButtonKind::AddSibling) {
                continue;
            }

            // 计算光标到按钮中心的距离
            let dx = cursor_pos.x - center.x;
            let dy = cursor_pos.y - center.y;

            // 如果光标在按钮的圆形区域内，返回指针光标
            if dx * dx + dy * dy <= r * r {
                return mouse::Interaction::Pointer;
            }
        }
    }

    // 默认返回空闲光标
    mouse::Interaction::Idle
}
