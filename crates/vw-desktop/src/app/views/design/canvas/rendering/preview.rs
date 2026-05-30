//! 设计画布渲染入口模块。
//!
//! 该模块组织预览、形状、文本和工具函数等渲染子能力，是画布视觉输出路径的组合层。

use iced::widget::canvas::{Frame, LineCap, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, Vector};

use crate::app::views::design::models::DesignTool;

/// 公开的 draw_tool_preview_overlay 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_tool_preview_overlay(
    frame: &mut Frame,
    active_tool: DesignTool,
    start: Option<Point>,
    current: Option<Point>,
    parent_rect: Option<Rectangle>,
) {
    let Some(start) = start else {
        return;
    };

    let current = current.unwrap_or(start);
    let preview = normalized_rect(start, current);
    let preview_rect = if preview.width < 1.0 || preview.height < 1.0 {
        default_preview_rect(active_tool, start)
    } else {
        preview
    };

    let fill = Color::from_rgba8(24, 160, 251, 0.14);
    let stroke_color = Color::from_rgba8(24, 160, 251, 0.92);
    let stroke = Stroke::default().with_color(stroke_color).with_width(1.5);

    let path = match active_tool {
        DesignTool::Line => Path::line(
            Point::new(preview_rect.x, preview_rect.y),
            Point::new(
                preview_rect.x + preview_rect.width,
                preview_rect.y + preview_rect.height.max(1.0),
            ),
        ),
        DesignTool::Ellipse => {
            let center = Point::new(
                preview_rect.x + preview_rect.width / 2.0,
                preview_rect.y + preview_rect.height / 2.0,
            );
            Path::circle(center, preview_rect.width.min(preview_rect.height) / 2.0)
        }
        _ => Path::rounded_rectangle(
            Point::new(preview_rect.x, preview_rect.y),
            Size::new(preview_rect.width, preview_rect.height),
            if active_tool == DesignTool::Frame { 14.0.into() } else { 10.0.into() },
        ),
    };

    if active_tool != DesignTool::Line {
        frame.fill(&path, fill);
    }
    frame.stroke(&path, stroke);

    if active_tool == DesignTool::Frame {
        let title_height = 26.0;
        let title_width = preview_rect.width.min(140.0).max(80.0);
        let title_path = Path::rounded_rectangle(
            Point::new(preview_rect.x, preview_rect.y - 34.0),
            Size::new(title_width, title_height),
            8.0.into(),
        );
        frame.fill(&title_path, Color::from_rgba8(255, 255, 255, 0.95));
        frame.stroke(&title_path, Stroke::default().with_color(stroke_color).with_width(1.0));
    }

    if let Some(parent_rect) = parent_rect {
        let parent_path = Path::rounded_rectangle(
            Point::new(parent_rect.x, parent_rect.y),
            Size::new(parent_rect.width, parent_rect.height),
            8.0.into(),
        );
        frame.stroke(
            &parent_path,
            Stroke::default().with_color(Color::from_rgba8(16, 185, 129, 0.85)).with_width(2.0),
        );
    }
}

/// 公开的 draw_brush_preview_overlay 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_brush_preview_overlay(
    frame: &mut Frame,
    points_world: &[Point],
    pan: Vector,
    zoom: f32,
    color_hex: &str,
    width_px: f32,
) {
    if points_world.len() < 2 {
        return;
    }

    let first = Point::new(points_world[0].x * zoom + pan.x, points_world[0].y * zoom + pan.y);
    let path = Path::new(|builder| {
        builder.move_to(first);
        for point in &points_world[1..] {
            builder.line_to(Point::new(point.x * zoom + pan.x, point.y * zoom + pan.y));
        }
    });

    let mut stroke = Stroke::default()
        .with_color(parse_hex_color(color_hex).unwrap_or(Color::from_rgba8(17, 24, 39, 0.92)))
        .with_width((width_px.max(1.0) * zoom).clamp(1.0, 18.0));
    stroke.line_cap = LineCap::Round;
    frame.stroke(&path, stroke);
}

/// 公开的 draw_eraser_overlay 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_eraser_overlay(frame: &mut Frame, cursor_pos: Point, radius_px: f32) {
    let circle = Path::circle(cursor_pos, radius_px.max(0.0));
    frame.fill(&circle, Color::from_rgba8(17, 24, 39, 0.08));
    frame.stroke(
        &circle,
        Stroke::default().with_color(Color::from_rgba8(17, 24, 39, 0.35)).with_width(1.0),
    );
}

fn normalized_rect(start: Point, current: Point) -> Rectangle {
    let x = start.x.min(current.x);
    let y = start.y.min(current.y);
    let width = (current.x - start.x).abs();
    let height = (current.y - start.y).abs();
    Rectangle::new(Point::new(x, y), Size::new(width, height))
}

fn default_preview_rect(tool: DesignTool, origin: Point) -> Rectangle {
    let (width, height) = match tool {
        DesignTool::Frame => (360.0, 240.0),
        DesignTool::Line => (160.0, 2.0),
        _ => (160.0, 160.0),
    };

    Rectangle::new(origin, Size::new(width, height))
}

fn parse_hex_color(input: &str) -> Option<Color> {
    let raw = input.trim().trim_start_matches('#');
    let parse = |start| u8::from_str_radix(&raw[start..start + 2], 16).ok();

    match raw.len() {
        6 => Some(Color::from_rgba8(parse(0)?, parse(2)?, parse(4)?, 1.0)),
        8 => Some(Color::from_rgba8(parse(0)?, parse(2)?, parse(4)?, f32::from(parse(6)?) / 255.0)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod preview_tests;
