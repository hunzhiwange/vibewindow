//! 思维导图画布覆盖层绘制逻辑，负责工具栏、悬浮控件和辅助提示。

use crate::apps::mindmap::state::MindMapCanvasTool;
use iced::widget::canvas::{Frame, LineCap, Path, Stroke};
use iced::{Color, Point, Rectangle, mouse};

use super::super::{
    DragMode, ERASER_RADIUS_PX, HoverButtonKind, MindMapCanvas, MindMapCanvasState,
};
use super::doodles::{draw_active_doodle, draw_eraser};
use super::layout_for_canvas;

/// 构建或更新 draw overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_overlay(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    state: &MindMapCanvasState,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) {
    let Some(cursor_pos) = cursor.position_in(bounds) else {
        return;
    };

    match state.drag_mode {
        DragMode::DoodlePen => draw_active_doodle(frame, canvas, state),
        DragMode::DoodleErase => draw_eraser(frame, cursor_pos, ERASER_RADIUS_PX),
        DragMode::None => draw_hover_buttons(frame, canvas, state, cursor_pos),
        _ => {}
    }
}

fn draw_hover_buttons(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    state: &MindMapCanvasState,
    cursor_pos: Point,
) {
    if canvas.canvas_tool != MindMapCanvasTool::Select {
        return;
    }

    let Some(node_path) = state.hovered_node.as_ref() else {
        return;
    };

    let layout = layout_for_canvas(canvas);
    let Some(node) = layout.nodes.iter().find(|node| &node.path == node_path) else {
        return;
    };
    let rect = canvas.node_screen_rect(node);

    for (kind, center, r) in canvas.node_button_specs(node_path, rect) {
        let Some(fill) = hover_button_fill(kind) else { continue };
        let hovered = point_in_circle(cursor_pos, center, r);
        let circle = Path::circle(center, r);
        frame.fill(&circle, if hovered { fill } else { fill.scale_alpha(0.92) });
        frame.stroke(
            &circle,
            Stroke::default()
                .with_color(Color::from_rgba8(0, 0, 0, 0.14))
                .with_width((1.0 * canvas.zoom).clamp(0.9, 1.6)),
        );

        let icon_stroke = Stroke {
            width: (2.0 * canvas.zoom).clamp(1.2, 2.6),
            style: Color::WHITE.into(),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        match kind {
            HoverButtonKind::AddChild => {
                let size = r * 0.36;
                frame.stroke(
                    &Path::line(
                        Point::new(center.x - size, center.y),
                        Point::new(center.x + size, center.y),
                    ),
                    icon_stroke,
                );
                frame.stroke(
                    &Path::line(
                        Point::new(center.x, center.y - size),
                        Point::new(center.x, center.y + size),
                    ),
                    icon_stroke,
                );
            }
            HoverButtonKind::AddSibling => {
                let x0 = center.x - r * 0.48;
                let x1 = center.x + r * 0.08;
                let dy = r * 0.22;
                for y in [center.y - dy, center.y, center.y + dy] {
                    frame.stroke(&Path::line(Point::new(x0, y), Point::new(x1, y)), icon_stroke);
                }

                let px = center.x + r * 0.30;
                let ps = r * 0.18;
                frame.stroke(
                    &Path::line(Point::new(px - ps, center.y), Point::new(px + ps, center.y)),
                    icon_stroke,
                );
                frame.stroke(
                    &Path::line(Point::new(px, center.y - ps), Point::new(px, center.y + ps)),
                    icon_stroke,
                );
            }
            HoverButtonKind::ToggleCollapse => {}
        }
    }
}

pub(super) fn hover_button_fill(kind: HoverButtonKind) -> Option<Color> {
    match kind {
        HoverButtonKind::AddChild => Some(Color::from_rgba8(34, 197, 94, 1.0)),
        HoverButtonKind::AddSibling => Some(Color::from_rgba8(59, 130, 246, 1.0)),
        HoverButtonKind::ToggleCollapse => None,
    }
}

pub(super) fn point_in_circle(point: Point, center: Point, r: f32) -> bool {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    dx * dx + dy * dy <= r * r
}
