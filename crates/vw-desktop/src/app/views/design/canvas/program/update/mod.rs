//! 设计画布更新模块
//!
//! 本模块负责处理设计画布中的所有用户交互事件，包括：
//! - 键盘事件（快捷键、删除、ESC 取消等）
//! - 鼠标指针事件（点击、移动、释放、滚轮等）
//! - 元素选择、移动、缩放、旋转等操作
//! - 网格（Mesh）变形控制点的拖拽操作
//! - 多选框选和批量操作
//!
//! # 主要功能
//!
//! 1. **键盘事件处理**：ESC 取消当前操作，Delete/Backspace 删除网格控制点
//! 2. **鼠标左键处理**：元素选择、控制点拖拽、多选框选
//! 3. **鼠标移动处理**：悬停检测、拖拽更新、平移画布
//! 4. **滚轮事件处理**：缩放或平移画布
//!
//! # 架构说明
//!
//! 所有更新方法都是 `DesignCanvas` 的方法，通过 `state` 参数管理画布状态，
//! 返回 `Action<Message>` 来触发重绘或发布消息。

use super::super::geometry::{get_element_screen_bounds, rotate_point};
use super::super::hit::{hit_test, hit_test_handle};
use super::super::layout::parse::parse_padding;
use super::super::types::{
    DesignCanvasState, Handle, MeshDragKind, MeshDragState, SelectedMeshHandle,
};
use super::super::utils::find_element_by_id;
use super::{DesignCanvas, FrameHeaderHit, mesh, selection};
use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::creation::{
    create_brush_path_element, create_capsule_element, create_chevron_element,
    create_diamond_element, create_ellipse_element, create_frame_element, create_hexagon_element,
    create_icon_element, create_line_element, create_parallelogram_element,
    create_pentagon_element, create_rectangle_element, create_star_element,
    create_sticky_note_element, create_text_element, create_trapezoid_element,
    create_triangle_element,
};
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
use iced::widget::canvas::{Action, Event};
use iced::{Point, Rectangle, Size, Vector, mouse};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use super::super::super::models::{DesignElement, DesignTool, StickyNoteKind};

mod keyboard;
mod moved;
mod pointer;
mod pressed;
mod released;
mod wheel;

const ERASER_RADIUS_PX: f32 = 30.0;

fn tool_supports_drag_preview(tool: DesignTool) -> bool {
    matches!(
        tool,
        DesignTool::Line
            | DesignTool::Rectangle
            | DesignTool::Ellipse
            | DesignTool::Frame
            | DesignTool::Triangle
            | DesignTool::Diamond
            | DesignTool::Star
            | DesignTool::Pentagon
            | DesignTool::Hexagon
            | DesignTool::Parallelogram
            | DesignTool::Trapezoid
            | DesignTool::Chevron
            | DesignTool::Capsule
            | DesignTool::StickyNote
    )
}

fn root_frame_at_cursor<'a>(canvas: &'a DesignCanvas<'a>, cursor_pos: Point) -> Option<&'a str> {
    for root in canvas.doc.children.iter().rev() {
        if root.kind != "frame" {
            continue;
        }

        if let Some(rect) =
            get_element_screen_bounds(canvas.doc.as_ref(), &root.id, canvas.pan, canvas.zoom)
            && rect.contains(cursor_pos)
        {
            return Some(root.id.as_str());
        }
    }

    None
}

fn frame_child_world_position<'a>(
    canvas: &DesignCanvas<'a>,
    parent_id: &str,
    cursor_pos: Point,
) -> Point {
    if let Some(parent) = canvas.doc.find_element(parent_id)
        && let Some(parent_rect) =
            get_element_screen_bounds(canvas.doc.as_ref(), parent_id, canvas.pan, canvas.zoom)
    {
        let theme_mode = canvas.doc.theme.as_ref().map(|theme| theme.mode.as_str());
        let padding = parse_padding(&parent.padding, &canvas.doc.variables, theme_mode);
        let local_x = ((cursor_pos.x - parent_rect.x) / canvas.zoom) - padding.left;
        let local_y = ((cursor_pos.y - parent_rect.y) / canvas.zoom) - padding.top;
        return Point::new(local_x.max(0.0), local_y.max(0.0));
    }

    Point::new(
        (cursor_pos.x - canvas.pan.x) / canvas.zoom,
        (cursor_pos.y - canvas.pan.y) / canvas.zoom,
    )
}

fn preview_world_rect<'a>(
    canvas: &DesignCanvas<'a>,
    state: &DesignCanvasState,
) -> Option<(Point, Point)> {
    let start = state.tool_preview_start?;
    let current = state.tool_preview_current.unwrap_or(start);

    let to_world = |point: Point| {
        if let Some(parent_id) = state.tool_preview_parent_id.as_deref() {
            frame_child_world_position(canvas, parent_id, point)
        } else {
            Point::new(
                (point.x - canvas.pan.x) / canvas.zoom,
                (point.y - canvas.pan.y) / canvas.zoom,
            )
        }
    };

    Some((to_world(start), to_world(current)))
}

fn build_created_element<'a>(
    canvas: &DesignCanvas<'a>,
    state: &DesignCanvasState,
    cursor_pos: Point,
) -> Option<(crate::app::views::design::models::DesignElement, Option<String>, bool)> {
    match canvas.active_tool {
        DesignTool::Text => {
            let parent_id = root_frame_at_cursor(canvas, cursor_pos).map(ToString::to_string);
            let position = if let Some(parent_id) = parent_id.as_deref() {
                frame_child_world_position(canvas, parent_id, cursor_pos)
            } else {
                Point::new(
                    (cursor_pos.x - canvas.pan.x) / canvas.zoom,
                    (cursor_pos.y - canvas.pan.y) / canvas.zoom,
                )
            };
            Some((create_text_element(position), parent_id, true))
        }
        DesignTool::Icon => {
            let parent_id = root_frame_at_cursor(canvas, cursor_pos).map(ToString::to_string);
            let position = if let Some(parent_id) = parent_id.as_deref() {
                frame_child_world_position(canvas, parent_id, cursor_pos)
            } else {
                Point::new(
                    (cursor_pos.x - canvas.pan.x) / canvas.zoom,
                    (cursor_pos.y - canvas.pan.y) / canvas.zoom,
                )
            };
            let mut element = create_icon_element(position);
            element.icon_font_family = Some(canvas.toolbar_icon_family.to_string());
            element.icon_font_name = Some(canvas.toolbar_icon_name.to_string());
            let element_name = canvas
                .toolbar_icon_name
                .split(['-', '_'])
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => {
                            first.to_ascii_uppercase().to_string()
                                + &chars.as_str().to_ascii_lowercase()
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            if !element_name.is_empty() {
                element.name = Some(element_name);
            }
            Some((element, parent_id, false))
        }
        DesignTool::Line
        | DesignTool::Rectangle
        | DesignTool::Ellipse
        | DesignTool::Frame
        | DesignTool::StickyNote
        | DesignTool::Triangle
        | DesignTool::Diamond
        | DesignTool::Star
        | DesignTool::Pentagon
        | DesignTool::Hexagon
        | DesignTool::Parallelogram
        | DesignTool::Trapezoid
        | DesignTool::Chevron
        | DesignTool::Capsule => {
            let (start, current) = preview_world_rect(canvas, state)?;
            let mut x = start.x.min(current.x);
            let mut y = start.y.min(current.y);
            let mut width = (current.x - start.x).abs();
            let mut height = (current.y - start.y).abs();

            let parent_id = root_frame_at_cursor(canvas, cursor_pos)
                .filter(|_| canvas.active_tool != DesignTool::Frame)
                .map(ToString::to_string);

            if width < 1.0 || height < 1.0 {
                match canvas.active_tool {
                    DesignTool::Frame => {
                        width = 360.0;
                        height = 240.0;
                    }
                    DesignTool::StickyNote => {
                        width = 320.0;
                        height = 220.0;
                    }
                    DesignTool::Line => {
                        width = 160.0;
                        height = 2.0;
                    }
                    _ => {
                        width = 160.0;
                        height = 160.0;
                    }
                }
                x = start.x;
                y = start.y;
            }

            let position = Point::new(x, y);
            let mut element = match canvas.active_tool {
                DesignTool::Rectangle => create_rectangle_element(position),
                DesignTool::Line => create_line_element(position),
                DesignTool::Ellipse => create_ellipse_element(position),
                DesignTool::Triangle => create_triangle_element(position),
                DesignTool::Diamond => create_diamond_element(position),
                DesignTool::Star => create_star_element(position),
                DesignTool::Pentagon => create_pentagon_element(position),
                DesignTool::Hexagon => create_hexagon_element(position),
                DesignTool::Parallelogram => create_parallelogram_element(position),
                DesignTool::Trapezoid => create_trapezoid_element(position),
                DesignTool::Chevron => create_chevron_element(position),
                DesignTool::Capsule => create_capsule_element(position),
                DesignTool::Frame => create_frame_element(position, canvas.doc.as_ref()),
                DesignTool::StickyNote => {
                    create_sticky_note_element(position, StickyNoteKind::Note)
                }
                _ => return None,
            };

            element.width = Some(serde_json::json!(width.max(1.0)));
            element.height = Some(serde_json::json!(height.max(1.0)));

            Some((element, parent_id, false))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
