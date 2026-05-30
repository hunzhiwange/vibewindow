//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::alignment::Horizontal;
use iced::font::{Font as IcedFont, Weight as IcedWeight};
use iced::{Color, Point, Rectangle, Size};

use super::super::dom::TailwindNode;
use super::super::parser::{ParsedStyle, TailwindParser};

const DEFAULT_SVG_SIZE: f32 = 20.0;
const DEFAULT_IMAGE_SIZE: f32 = 100.0;

#[derive(Debug, Clone, Copy)]
struct SvgViewBox {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct VisualOutset {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl VisualOutset {
    fn union(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            right: self.right.max(other.right),
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// 执行 resolve_node_style 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_node_style(node: &TailwindNode) -> ParsedStyle {
    let class_string = node.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
    let mut style = TailwindParser::parse(class_string);

    if let Some(dir) = node.attributes.get("dir")
        && !dir.is_empty()
    {
        style.text_direction = Some(dir.clone());
    }

    style
}

/// 执行 clamp_explicit_size_to_bounds 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn clamp_explicit_size_to_bounds(size: f32, bound: f32) -> f32 {
    if bound.is_finite() { size.min(bound.max(0.0)) } else { size }
}

/// 执行 resolve_svg_view_box 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_svg_view_box(node: &TailwindNode) -> (f32, f32, f32, f32) {
    let raw_view_box = node.attributes.get("viewBox").map(|s| s.as_str()).unwrap_or("");
    let parts: Vec<f32> =
        raw_view_box.split_whitespace().filter_map(|part| part.parse().ok()).collect();

    let view_box = if parts.len() == 4
        && parts[2].is_finite()
        && parts[3].is_finite()
        && parts[2] > 0.0
        && parts[3] > 0.0
    {
        SvgViewBox { min_x: parts[0], min_y: parts[1], width: parts[2], height: parts[3] }
    } else {
        SvgViewBox { min_x: 0.0, min_y: 0.0, width: DEFAULT_SVG_SIZE, height: DEFAULT_SVG_SIZE }
    };

    (view_box.min_x, view_box.min_y, view_box.width, view_box.height)
}

/// 执行 resolve_svg_rect 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_svg_rect(
    style: &ParsedStyle,
    bounds: Rectangle,
    zoom: f32,
    view_box: (f32, f32, f32, f32),
) -> Rectangle {
    let (_, _, view_box_width, view_box_height) = view_box;
    let aspect_ratio =
        if view_box_height.abs() > f32::EPSILON { view_box_width / view_box_height } else { 1.0 };

    let width = style.width.map(|value| value * zoom);
    let height = style.height.map(|value| value * zoom);

    let resolved = match (width, height) {
        (Some(width), Some(height)) => Size::new(width.max(0.0), height.max(0.0)),
        (Some(width), None) => Size::new(width.max(0.0), (width / aspect_ratio).max(0.0)),
        (None, Some(height)) => Size::new((height * aspect_ratio).max(0.0), height.max(0.0)),
        (None, None) => Size::new(view_box_width * zoom, view_box_height * zoom),
    };

    Rectangle { x: bounds.x, y: bounds.y, width: resolved.width, height: resolved.height }
}

/// 执行 resolve_img_rect 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_img_rect(style: &ParsedStyle, bounds: Rectangle, zoom: f32) -> Rectangle {
    let width = style.width.map(|value| value * zoom);
    let height = style.height.map(|value| value * zoom);

    let resolved = match (width, height) {
        (Some(width), Some(height)) => Size::new(width.max(0.0), height.max(0.0)),
        (Some(width), None) => Size::new(width.max(0.0), width.max(0.0)),
        (None, Some(height)) => Size::new(height.max(0.0), height.max(0.0)),
        (None, None) => Size::new(DEFAULT_IMAGE_SIZE * zoom, DEFAULT_IMAGE_SIZE * zoom),
    };

    Rectangle { x: bounds.x, y: bounds.y, width: resolved.width, height: resolved.height }
}

/// 执行 resolve_svg_render_origin 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_svg_render_origin(
    draw_bounds: Rectangle,
    view_box: (f32, f32, f32, f32),
) -> (Point, f32) {
    let (min_x, min_y, view_box_width, view_box_height) = view_box;
    let scale_x =
        if view_box_width.abs() > f32::EPSILON { draw_bounds.width / view_box_width } else { 1.0 };
    let scale_y = if view_box_height.abs() > f32::EPSILON {
        draw_bounds.height / view_box_height
    } else {
        1.0
    };
    let scale = scale_x.min(scale_y).max(0.0);

    let content_width = view_box_width * scale;
    let content_height = view_box_height * scale;
    let origin_x = draw_bounds.x + (draw_bounds.width - content_width) / 2.0 - min_x * scale;
    let origin_y = draw_bounds.y + (draw_bounds.height - content_height) / 2.0 - min_y * scale;

    (Point::new(origin_x, origin_y), scale)
}

/// 执行 inherit_text_style 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn inherit_text_style(
    style: &ParsedStyle,
    inherited: Option<&ParsedStyle>,
) -> ParsedStyle {
    let mut effective = style.clone();

    if let Some(parent) = inherited {
        if effective.text_color.is_none() {
            effective.text_color = parent.text_color;
        }
        if effective.font_size.is_none() {
            effective.font_size = parent.font_size;
        }
        if effective.font_weight.is_none() {
            effective.font_weight = parent.font_weight;
        }
        if effective.text_align.is_none() {
            effective.text_align = parent.text_align.clone();
        }
        if effective.text_direction.is_none() {
            effective.text_direction = parent.text_direction.clone();
        }
        if effective.text_decoration.is_none() {
            effective.text_decoration = parent.text_decoration.clone();
        }
        if effective.font_style.is_none() {
            effective.font_style = parent.font_style.clone();
        }
        if effective.letter_spacing.is_none() {
            effective.letter_spacing = parent.letter_spacing;
        }
        if effective.line_height.is_none() {
            effective.line_height = parent.line_height;
        }
        if effective.text_transform.is_none() {
            effective.text_transform = parent.text_transform.clone();
        }
        effective.opacity = match (parent.opacity, effective.opacity) {
            (Some(parent_opacity), Some(current_opacity)) => Some(parent_opacity * current_opacity),
            (Some(parent_opacity), None) => Some(parent_opacity),
            _ => effective.opacity,
        };
    }

    effective
}

/// 执行 resolve_text_align 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_text_align(style: &ParsedStyle) -> Horizontal {
    let mut align = match style.text_align.as_deref() {
        Some("center") => Horizontal::Center,
        Some("right") => Horizontal::Right,
        Some("start") => Horizontal::Left,
        Some("end") => Horizontal::Right,
        _ => Horizontal::Left,
    };

    if matches!(style.text_align.as_deref(), Some("start") | Some("end"))
        && style.text_direction.as_deref().is_some_and(|dir| dir.eq_ignore_ascii_case("rtl"))
    {
        align = match style.text_align.as_deref() {
            Some("start") => Horizontal::Right,
            Some("end") => Horizontal::Left,
            _ => align,
        };
    }

    align
}

/// 执行 resolve_font 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_font(style: &ParsedStyle) -> IcedFont {
    let mut font = IcedFont::default();
    font.weight = match style.font_weight.unwrap_or(400) {
        300 => IcedWeight::Light,
        500 => IcedWeight::Medium,
        600 => IcedWeight::Semibold,
        700 => IcedWeight::Bold,
        800 => IcedWeight::ExtraBold,
        _ => IcedWeight::Normal,
    };
    font.style = if style.font_style.as_deref() == Some("italic") {
        iced::font::Style::Italic
    } else {
        iced::font::Style::Normal
    };
    font
}

/// 执行 apply_opacity 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn apply_opacity(color: Color, opacity: Option<f32>) -> Color {
    if let Some(opacity) = opacity {
        Color { a: (color.a * opacity).clamp(0.0, 1.0), ..color }
    } else {
        color
    }
}

/// 执行 resolve_visual_style 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_visual_style(
    style: &ParsedStyle,
    inherited_text_style: &ParsedStyle,
) -> ParsedStyle {
    let mut visual_style = style.clone();
    let opacity = inherited_text_style.opacity.or(style.opacity);

    if let Some(color) = visual_style.background_color {
        visual_style.background_color = Some(apply_opacity(color, opacity));
    }
    if let Some(color) = visual_style.border_color {
        visual_style.border_color = Some(apply_opacity(color, opacity));
    }
    if let Some(color) = visual_style.text_color {
        visual_style.text_color = Some(apply_opacity(color, opacity));
    }

    visual_style.opacity = opacity;
    visual_style
}

/// 执行 visual_bounds_for_style 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn visual_bounds_for_style(
    style: &ParsedStyle,
    rect: Rectangle,
    zoom: f32,
) -> Rectangle {
    let outset = shadow_visual_outset(style, zoom).union(outline_visual_outset(style, zoom));

    Rectangle {
        x: rect.x - outset.left,
        y: rect.y - outset.top,
        width: rect.width + outset.left + outset.right,
        height: rect.height + outset.top + outset.bottom,
    }
}

fn shadow_visual_outset(style: &ParsedStyle, zoom: f32) -> VisualOutset {
    let Some(color) = style.shadow_color else {
        return VisualOutset::default();
    };

    if color.a <= 0.0 {
        return VisualOutset::default();
    }

    let offset_x = style.shadow_offset_x.unwrap_or(0.0) * zoom;
    let offset_y = style.shadow_offset_y.unwrap_or(0.0) * zoom;
    let spread = style.shadow_spread.unwrap_or(0.0).max(0.0) * zoom;

    VisualOutset {
        left: (spread - offset_x).max(0.0),
        right: (spread + offset_x).max(0.0),
        top: (spread - offset_y).max(0.0),
        bottom: (spread + offset_y).max(0.0),
    }
}

fn outline_visual_outset(style: &ParsedStyle, zoom: f32) -> VisualOutset {
    let outline_width = style.outline_width.unwrap_or(0.0) * zoom;
    let outline_style = style.outline_style.as_deref().unwrap_or("solid");

    if outline_width <= 0.0 || outline_style == "none" {
        return VisualOutset::default();
    }

    let outward = (style.outline_offset.unwrap_or(0.0) * zoom + outline_width).max(0.0);

    VisualOutset { left: outward, right: outward, top: outward, bottom: outward }
}

#[cfg(test)]
#[path = "style_tests.rs"]
mod style_tests;
