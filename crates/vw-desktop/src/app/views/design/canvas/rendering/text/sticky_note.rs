//! 设计画布文本渲染模块。
//!
//! 该模块处理文本节点的排版、网格、树结构或便签绘制逻辑，确保 DOM 风格输入能够稳定映射到画布中的可见文本。

use iced::font::{Font as IcedFont, Weight as IcedWeight};
use iced::{
    Point,
    widget::canvas::{Frame, LineDash, Path, Stroke, Text},
};

use crate::app::views::design::{
    canvas::parse::{
        intern_font_family_name, measure_font_vertical_metrics_with_font, parse_color,
        parse_font_size, parse_line_height, resolve_font_family, wrap_text_lines_with_font,
    },
    models::{DesignDoc, DesignElement},
};

/// 模块内部可见的 draw_sticky_note_text 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_sticky_note_text(
    frame: &mut Frame,
    element: &DesignElement,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    zoom: f32,
    doc: &DesignDoc,
    theme_mode: Option<&str>,
) {
    let note_kind = element.sticky_note_kind();
    let font_family_name = resolve_font_family(&element.font_family, &doc.variables, theme_mode);
    let font_family_static = intern_font_family_name(&font_family_name);
    let raw_body_font_size = parse_font_size(&element.font_size, &doc.variables, theme_mode);
    let header_font_size = 15.0 * zoom;
    let body_font_size = raw_body_font_size * zoom;
    let body_line_height =
        parse_line_height(&element.line_height, raw_body_font_size, &doc.variables, theme_mode)
            * zoom;
    let text_color = element
        .color
        .as_ref()
        .map(|value| parse_color(value, &doc.variables, theme_mode))
        .unwrap_or_else(|| parse_color(note_kind.text_color(), &doc.variables, theme_mode));
    let (header_ascent, header_descent) =
        measure_font_vertical_metrics_with_font(&font_family_name, header_font_size)
            .unwrap_or((header_font_size * 0.8, header_font_size * 0.2));
    let header_x = x + 18.0 * zoom;
    let header_band_top = y + 6.0 * zoom;
    let divider_y = y + 42.0 * zoom;
    let header_band_center = header_band_top + (divider_y - header_band_top) * 0.5;
    let header_y = header_band_center + (header_ascent - header_descent) * 0.5;
    let body_x = x + 18.0 * zoom;
    let body_y = y + 64.0 * zoom;
    let body_width = (w - 36.0 * zoom).max(0.0);
    let body_limit = y + h - 18.0 * zoom;
    let label = note_kind.bilingual_label();
    let lines = wrap_text_lines_with_font(
        element.content.as_deref().unwrap_or_default(),
        body_width,
        &font_family_name,
        body_font_size,
        0.0,
    );

    let mut header_font = IcedFont::with_name(font_family_static);
    header_font.weight = IcedWeight::Medium;
    frame.fill_text(Text {
        content: label.clone(),
        position: Point::new(header_x, header_y),
        color: text_color,
        size: header_font_size.into(),
        font: header_font,
        ..Default::default()
    });

    let dash_segments = [6.0 * zoom, 5.0 * zoom];
    let divider = Path::line(
        Point::new(x + 1.5 * zoom, divider_y),
        Point::new(x + w - 1.5 * zoom, divider_y),
    );
    let mut divider_stroke = Stroke::default()
        .with_color(text_color.scale_alpha(0.8))
        .with_width((1.2 * zoom).clamp(1.0, 2.0));
    divider_stroke.line_dash = LineDash { segments: &dash_segments, offset: 0 };
    frame.stroke(&divider, divider_stroke);

    let body_font = IcedFont::with_name(font_family_static);
    for (index, line) in lines.iter().enumerate() {
        let line_y = body_y + (index as f32) * body_line_height;
        if line_y > body_limit {
            break;
        }
        frame.fill_text(Text {
            content: line.clone(),
            position: Point::new(body_x, line_y),
            color: text_color,
            size: body_font_size.into(),
            font: body_font,
            ..Default::default()
        });
    }
}

#[cfg(test)]
#[path = "sticky_note_tests.rs"]
mod sticky_note_tests;
