//! 设计画布渲染工具模块。
//!
//! 该模块提供路径、图片、文本和 Tailwind 样式转换等底层辅助函数，减少渲染主流程中的重复样板逻辑。

use iced::{
    Color, Rectangle, Size,
    widget::canvas::{Frame, LineCap, LineDash, Stroke},
};

use crate::app::views::design::canvas::tailwind::parser::ParsedStyle;

use super::path::element_path;

const SQUARE_EPS_PX: f32 = 0.5;

fn resolved_shape_kind(bounds: Rectangle, radius: f32) -> (&'static str, f32) {
    let max_r = (bounds.width.min(bounds.height)) / 2.0;
    let is_square = (bounds.width - bounds.height).abs() <= SQUARE_EPS_PX;

    if is_square && radius >= max_r - SQUARE_EPS_PX {
        ("circle", max_r)
    } else {
        ("rect", radius.max(0.0))
    }
}

fn draw_tailwind_shadow(frame: &mut Frame, bounds: Rectangle, zoom: f32, style: &ParsedStyle) {
    let Some(color) = style.shadow_color else {
        return;
    };

    if color.a <= 0.0 {
        return;
    }

    let offset_x = style.shadow_offset_x.unwrap_or(0.0) * zoom;
    let offset_y = style.shadow_offset_y.unwrap_or(0.0) * zoom;
    let spread = style.shadow_spread.unwrap_or(0.0).max(0.0) * zoom;

    if offset_x == 0.0 && offset_y == 0.0 && spread == 0.0 {
        return;
    }

    let shadow_bounds = Rectangle {
        x: bounds.x + offset_x - spread,
        y: bounds.y + offset_y - spread,
        width: (bounds.width + spread * 2.0).max(0.0),
        height: (bounds.height + spread * 2.0).max(0.0),
    };
    let radius = style.border_radius.unwrap_or(0.0) * zoom + spread;
    let (kind, resolved_radius) = resolved_shape_kind(shadow_bounds, radius);
    let path = element_path(
        kind,
        shadow_bounds.x,
        shadow_bounds.y,
        shadow_bounds.width,
        shadow_bounds.height,
        resolved_radius,
    );

    frame.fill(&path, color);
}

/// 公开的 draw_tailwind_outline 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_tailwind_outline(frame: &mut Frame, bounds: Rectangle, zoom: f32, style: &ParsedStyle) {
    let outline_width = style.outline_width.unwrap_or(0.0) * zoom;
    let outline_style = style.outline_style.as_deref().unwrap_or("solid");

    if outline_width <= 0.0 || outline_style == "none" {
        return;
    }

    let outline_offset = style.outline_offset.unwrap_or(0.0) * zoom;
    let center_distance = outline_offset + outline_width / 2.0;
    let outline_bounds = Rectangle {
        x: bounds.x - center_distance,
        y: bounds.y - center_distance,
        width: (bounds.width + center_distance * 2.0).max(0.0),
        height: (bounds.height + center_distance * 2.0).max(0.0),
    };
    let outline_color = style.outline_color.unwrap_or(style.border_color.unwrap_or(Color::BLACK));
    let outline_radius = style.border_radius.unwrap_or(0.0) * zoom + center_distance.max(0.0);
    let (kind, resolved_radius) = resolved_shape_kind(outline_bounds, outline_radius);
    let path = element_path(
        kind,
        outline_bounds.x,
        outline_bounds.y,
        outline_bounds.width,
        outline_bounds.height,
        resolved_radius,
    );
    let mut stroke = Stroke::default().with_color(outline_color).with_width(outline_width);

    let dash_segments = [outline_width * 3.0, outline_width * 3.0];
    let dot_segments = [outline_width, outline_width];

    if outline_style == "dashed" {
        stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
    } else if outline_style == "dotted" {
        stroke.line_dash = LineDash { segments: &dot_segments, offset: 0 };
        stroke.line_cap = LineCap::Round;
    } else if outline_style == "double" {
        let third = outline_width / 3.0;
        stroke.width = third;
        frame.stroke(&path, stroke);

        let inner_distance = outline_offset + outline_width / 6.0;
        let inner_bounds = Rectangle {
            x: bounds.x - inner_distance,
            y: bounds.y - inner_distance,
            width: (bounds.width + inner_distance * 2.0).max(0.0),
            height: (bounds.height + inner_distance * 2.0).max(0.0),
        };
        let inner_radius = style.border_radius.unwrap_or(0.0) * zoom + inner_distance.max(0.0);
        let (inner_kind, inner_resolved_radius) = resolved_shape_kind(inner_bounds, inner_radius);
        let inner_path = element_path(
            inner_kind,
            inner_bounds.x,
            inner_bounds.y,
            inner_bounds.width,
            inner_bounds.height,
            inner_resolved_radius,
        );
        frame.stroke(&inner_path, stroke);
        return;
    }

    frame.stroke(&path, stroke);
}

/// 公开的 draw_tailwind_box 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_tailwind_box(frame: &mut Frame, bounds: Rectangle, zoom: f32, style: &ParsedStyle) {
    draw_tailwind_shadow(frame, bounds, zoom, style);

    if let Some(bg_color) = style.background_color {
        let radius = style.border_radius.unwrap_or(0.0) * zoom;
        let (kind, resolved_radius) = resolved_shape_kind(bounds, radius);
        let path =
            element_path(kind, bounds.x, bounds.y, bounds.width, bounds.height, resolved_radius);
        frame.fill(&path, bg_color);
    }

    let border_style = style.border_style.as_deref().unwrap_or("solid");
    let border_width = style.border_width.unwrap_or(0.0) * zoom;
    let border_color = style.border_color.unwrap_or(Color::BLACK);

    if border_style == "none" || border_style == "hidden" {
        return;
    }

    let uniform = style.border_width.is_some()
        && style.border_top_width.is_none()
        && style.border_right_width.is_none()
        && style.border_bottom_width.is_none()
        && style.border_left_width.is_none()
        && style.border_inline_start_width.is_none()
        && style.border_inline_end_width.is_none();

    if uniform && border_width > 0.0 {
        let radius = style.border_radius.unwrap_or(0.0) * zoom;
        let stroke_bounds = Rectangle {
            x: bounds.x + border_width / 2.0,
            y: bounds.y + border_width / 2.0,
            width: bounds.width - border_width,
            height: bounds.height - border_width,
        };
        let (kind, resolved_radius) = resolved_shape_kind(stroke_bounds, radius);
        let path = element_path(
            kind,
            stroke_bounds.x,
            stroke_bounds.y,
            stroke_bounds.width,
            stroke_bounds.height,
            resolved_radius,
        );

        let mut stroke = Stroke::default().with_color(border_color).with_width(border_width);

        let dash_segments = [border_width * 3.0, border_width * 3.0];
        let dot_segments = [border_width, border_width];

        if border_style == "dashed" {
            stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
        } else if border_style == "dotted" {
            stroke.line_dash = LineDash { segments: &dot_segments, offset: 0 };
            stroke.line_cap = LineCap::Round;
        } else if border_style == "double" {
            let third = border_width / 3.0;
            stroke.width = third;
            frame.stroke(&path, stroke);

            let inner_bounds = Rectangle {
                x: bounds.x + border_width - third / 2.0,
                y: bounds.y + border_width - third / 2.0,
                width: bounds.width - (border_width * 2.0) + third,
                height: bounds.height - (border_width * 2.0) + third,
            };
            let (inner_kind, inner_resolved_radius) = resolved_shape_kind(inner_bounds, radius);
            let inner_path = element_path(
                inner_kind,
                inner_bounds.x,
                inner_bounds.y,
                inner_bounds.width,
                inner_bounds.height,
                inner_resolved_radius,
            );
            frame.stroke(&inner_path, stroke);
            return;
        }

        frame.stroke(&path, stroke);
        return;
    }

    if border_style != "solid" {
        return;
    }

    if style.border_width.unwrap_or(0.0) <= 0.0
        && style.border_top_width.unwrap_or(0.0) <= 0.0
        && style.border_right_width.unwrap_or(0.0) <= 0.0
        && style.border_bottom_width.unwrap_or(0.0) <= 0.0
        && style.border_left_width.unwrap_or(0.0) <= 0.0
        && style.border_inline_start_width.unwrap_or(0.0) <= 0.0
        && style.border_inline_end_width.unwrap_or(0.0) <= 0.0
    {
        return;
    }

    let top_w = style.border_top_width.or(style.border_width).unwrap_or(0.0) * zoom;
    let mut right_w = style.border_right_width.or(style.border_width).unwrap_or(0.0) * zoom;
    let bottom_w = style.border_bottom_width.or(style.border_width).unwrap_or(0.0) * zoom;
    let mut left_w = style.border_left_width.or(style.border_width).unwrap_or(0.0) * zoom;

    if let Some(s) = style.border_inline_start_width {
        if style.text_direction.as_deref() == Some("rtl") {
            right_w = s * zoom;
        } else {
            left_w = s * zoom;
        }
    }
    if let Some(e) = style.border_inline_end_width {
        if style.text_direction.as_deref() == Some("rtl") {
            left_w = e * zoom;
        } else {
            right_w = e * zoom;
        }
    }

    if top_w > 0.0 {
        frame.fill_rectangle(
            iced::Point::new(bounds.x, bounds.y),
            Size::new(bounds.width, top_w),
            border_color,
        );
    }
    if bottom_w > 0.0 {
        frame.fill_rectangle(
            iced::Point::new(bounds.x, bounds.y + bounds.height - bottom_w),
            Size::new(bounds.width, bottom_w),
            border_color,
        );
    }
    if left_w > 0.0 {
        frame.fill_rectangle(
            iced::Point::new(bounds.x, bounds.y),
            Size::new(left_w, bounds.height),
            border_color,
        );
    }
    if right_w > 0.0 {
        frame.fill_rectangle(
            iced::Point::new(bounds.x + bounds.width - right_w, bounds.y),
            Size::new(right_w, bounds.height),
            border_color,
        );
    }
}

#[cfg(test)]
#[path = "tailwind_tests.rs"]
mod tailwind_tests;
