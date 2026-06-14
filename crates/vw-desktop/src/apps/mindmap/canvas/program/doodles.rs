//! 思维导图画布涂鸦绘制逻辑，负责手写笔迹和橡皮擦视觉反馈。

use crate::apps::mindmap::canvas::style::rgba_u32_to_color;
use crate::apps::mindmap::canvas::transform::screen_from_world;
use iced::Color;
use iced::widget::canvas::{Frame, LineCap, Path, Stroke};

use super::{MindMapCanvas, MindMapCanvasState};

/// 构建或更新 draw committed doodles 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_committed_doodles(frame: &mut Frame, canvas: &MindMapCanvas<'_>) {
    for stroke_data in canvas.doodles {
        if stroke_data.points_world.len() < 2 {
            continue;
        }

        let first = screen_from_world(stroke_data.points_world[0], canvas.pan, canvas.zoom);
        let path = Path::new(|builder| {
            builder.move_to(first);
            for point in &stroke_data.points_world[1..] {
                builder.line_to(screen_from_world(*point, canvas.pan, canvas.zoom));
            }
        });

        let stroke = Stroke {
            style: rgba_u32_to_color(stroke_data.rgba).into(),
            width: doodle_stroke_width(stroke_data.width_px),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        frame.stroke(&path, stroke);
    }
}

/// 构建或更新 draw active doodle 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_active_doodle(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    state: &MindMapCanvasState,
) {
    if state.doodle_points_world.len() < 2 {
        return;
    }

    let first = screen_from_world(state.doodle_points_world[0], canvas.pan, canvas.zoom);
    let path = Path::new(|builder| {
        builder.move_to(first);
        for point in &state.doodle_points_world[1..] {
            builder.line_to(screen_from_world(*point, canvas.pan, canvas.zoom));
        }
    });

    let stroke = Stroke {
        style: rgba_u32_to_color(canvas.doodle_rgba).into(),
        width: doodle_stroke_width(canvas.doodle_width_px),
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    frame.stroke(&path, stroke);
}

/// 构建或更新 draw eraser 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_eraser(frame: &mut Frame, cursor_pos: iced::Point, radius: f32) {
    let circle = Path::circle(cursor_pos, radius);
    frame.fill(&circle, Color::from_rgba8(17, 24, 39, 0.08));
    frame.stroke(
        &circle,
        Stroke::default().with_color(Color::from_rgba8(17, 24, 39, 0.35)).with_width(1.0),
    );
}

pub(super) fn doodle_stroke_width(width_px: f32) -> f32 {
    width_px.clamp(1.0, 18.0)
}
