//! 设计画布文本渲染模块。
//!
//! 该模块处理文本节点的排版、网格、树结构或便签绘制逻辑，确保 DOM 风格输入能够稳定映射到画布中的可见文本。

use iced::font::{Font as IcedFont, Weight as IcedWeight};
use iced::{
    Color, Point, Rectangle, Size, Vector,
    widget::canvas::{Frame, Text},
};

use crate::app::views::design::{
    canvas::{
        parse::{
            intern_font_family_name, measure_font_vertical_metrics_with_font,
            measure_text_width_with_font, parse_color, parse_fills, parse_font_size,
            parse_line_height, resolve_font_family, wrap_text_lines_with_font,
        },
        tailwind::ParsedStyle,
        types::Padding,
    },
    models::{DesignDoc, DesignElement},
};

use super::{
    super::utils::{apply_text_transform, draw_tailwind_box, draw_text_decoration},
    mesh::{draw_char_outline, extract_mesh_fill, measure_char_advance, sample_mesh_color},
};

/// 模块内部可见的 draw_typography_text 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_typography_text(
    frame: &mut Frame,
    element: &DesignElement,
    bounds: Rectangle,
    resolved: Size,
    zoom: f32,
    doc: &DesignDoc,
    theme_mode: Option<&str>,
    tailwind_style: &ParsedStyle,
    padding: Padding,
) {
    let x = bounds.x;
    let y = bounds.y;
    let fill_colors = if let Some(c) = tailwind_style.text_color {
        vec![c]
    } else if let Some(c_str) = &element.color {
        vec![parse_color(c_str, &doc.variables, theme_mode)]
    } else {
        let fills = parse_fills(&element.fill, &doc.variables, theme_mode);
        if fills.is_empty() { vec![Color::BLACK] } else { fills }
    };
    let base_color = fill_colors.first().copied().unwrap_or(Color::TRANSPARENT);

    let mesh_fill = if element.color.is_none() && element.kind != "tailwind" {
        extract_mesh_fill(&element.fill, &doc.variables, theme_mode)
    } else {
        None
    };

    let font_size = tailwind_style
        .font_size
        .unwrap_or_else(|| parse_font_size(&element.font_size, &doc.variables, theme_mode));
    let line_height_val =
        tailwind_style.line_height.map(|lh| lh * font_size).unwrap_or_else(|| {
            parse_line_height(&element.line_height, font_size, &doc.variables, theme_mode)
        });
    let scaled_font_size = font_size * zoom;
    let content_w = (resolved.width - (padding.left + padding.right)).max(0.0) * zoom;
    let content_h = (resolved.height - (padding.top + padding.bottom)).max(0.0) * zoom;
    let rotation = element.rotation.unwrap_or(0.0);
    let rotation_rad = rotation.to_radians();
    let center_x = x + (resolved.width * zoom) / 2.0;
    let center_y = y + (resolved.height * zoom) / 2.0;
    let effective_content = element
        .content
        .as_ref()
        .map(|s| apply_text_transform(s, tailwind_style.text_transform.as_deref()));

    if let Some(content) = effective_content.as_ref() {
        let font_family_name =
            resolve_font_family(&element.font_family, &doc.variables, theme_mode);
        let font_family_static = intern_font_family_name(&font_family_name);
        let (ascent_px, descent_px) =
            measure_font_vertical_metrics_with_font(&font_family_name, font_size)
                .unwrap_or((font_size * 0.8, font_size * 0.2));
        let font_box_h_px = (ascent_px + descent_px).max(0.0);
        let letter_spacing = tailwind_style.letter_spacing.unwrap_or(0.0) * zoom;
        let curve_text_enabled = false;
        let lines = wrap_text_lines_with_font(
            content,
            content_w,
            &font_family_name,
            scaled_font_size,
            letter_spacing,
        );
        let total_text_h = (lines.len() as f32) * line_height_val * zoom;
        let start_y = y
            + padding.top * zoom
            + match element.text_align_vertical.as_deref() {
                Some("middle") | Some("center") => (content_h - total_text_h).max(0.0) / 2.0,
                Some("end") | Some("bottom") => (content_h - total_text_h).max(0.0),
                _ => 0.0,
            };
        let horizontal = tailwind_style.text_align.as_deref().or(element.text_align.as_deref());

        let draw_lines = |frame: &mut Frame| {
            for (i, line) in lines.iter().enumerate() {
                let mut line_w = measure_text_width_with_font(
                    line,
                    &font_family_name,
                    scaled_font_size,
                    letter_spacing,
                );
                let mut start_x = x + padding.left * zoom;
                let mut did_justify = false;
                let line_box_y = start_y + (i as f32) * line_height_val * zoom;
                let extra_leading = ((line_height_val - font_box_h_px).max(0.0) * zoom) / 2.0;
                let line_y = line_box_y + extra_leading;

                let weight_val = tailwind_style.font_weight.map(|w| match w {
                    300 => "300",
                    400 => "400",
                    500 => "500",
                    600 => "600",
                    700 => "700",
                    800 => "800",
                    _ => "400",
                });
                let weight_str =
                    weight_val.or_else(|| element.font_weight.as_ref().and_then(|v| v.as_str()));

                let weight = match weight_str {
                    Some("300") => IcedWeight::Light,
                    Some("400") | None => IcedWeight::Normal,
                    Some("500") => IcedWeight::Medium,
                    Some("600") => IcedWeight::Semibold,
                    Some("700") => IcedWeight::Bold,
                    Some("800") => IcedWeight::ExtraBold,
                    Some(_) => IcedWeight::Normal,
                };

                let font_style = tailwind_style
                    .font_style
                    .as_deref()
                    .or(element.font_style.as_deref())
                    .unwrap_or("normal");
                let style = if font_style == "italic" {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                };

                let mut font = IcedFont::with_name(font_family_static);
                font.weight = weight;
                font.style = style;

                if horizontal == Some("justify") && i < lines.len() - 1 {
                    let words: Vec<&str> = line.split_whitespace().collect();
                    if words.len() > 1 {
                        let words_width: f32 = words
                            .iter()
                            .map(|w| {
                                measure_text_width_with_font(
                                    w,
                                    &font_family_name,
                                    scaled_font_size,
                                    letter_spacing,
                                )
                            })
                            .sum();
                        let spaces = (words.len() - 1) as f32;
                        let base_space = measure_text_width_with_font(
                            " ",
                            &font_family_name,
                            scaled_font_size,
                            0.0,
                        );
                        let extra = ((content_w - words_width).max(0.0) / spaces).max(0.0);

                        let mut cx = start_x;
                        for (wi, w) in words.iter().enumerate() {
                            if curve_text_enabled {
                                let mut cursor_x = 0.0;
                                let chars: Vec<char> = w.chars().collect();
                                for (ci, ch) in chars.iter().enumerate() {
                                    let pen = Point::new(cx + cursor_x, line_y);
                                    let adv = draw_char_outline(
                                        frame,
                                        *ch,
                                        pen,
                                        &font_family_name,
                                        scaled_font_size,
                                        base_color,
                                    );
                                    cursor_x += adv;
                                    if ci < chars.len().saturating_sub(1) {
                                        cursor_x += letter_spacing;
                                    }
                                }
                                cx += measure_text_width_with_font(
                                    w,
                                    &font_family_name,
                                    scaled_font_size,
                                    letter_spacing,
                                );
                            } else {
                                frame.fill_text(Text {
                                    content: w.to_string(),
                                    position: Point::new(cx, line_y),
                                    color: base_color,
                                    size: scaled_font_size.into(),
                                    font,
                                    ..Default::default()
                                });
                                cx += measure_text_width_with_font(
                                    w,
                                    &font_family_name,
                                    scaled_font_size,
                                    letter_spacing,
                                );
                            }
                            if wi < words.len() - 1 {
                                cx += base_space + extra;
                            }
                        }
                        line_w = content_w;
                        did_justify = true;
                    }
                } else {
                    start_x += match horizontal {
                        Some("center") => (content_w - line_w).max(0.0) / 2.0,
                        Some("right") => (content_w - line_w).max(0.0),
                        _ => 0.0,
                    };
                }

                if !did_justify {
                    if let Some(mesh) = &mesh_fill {
                        let mut cursor_x = 0.0;
                        let chars: Vec<char> = line.chars().collect();
                        for (idx, ch) in chars.iter().enumerate() {
                            let ch_width = measure_text_width_with_font(
                                &ch.to_string(),
                                &font_family_name,
                                scaled_font_size,
                                0.0,
                            );
                            let ch_x = start_x + cursor_x;
                            let char_center_x = ch_x + ch_width / 2.0;
                            let char_center_y = line_box_y + (line_height_val * zoom) / 2.0;
                            let u = if content_w > 0.0 {
                                (char_center_x - (x + padding.left * zoom)) / content_w
                            } else {
                                0.0
                            };
                            let v = if content_h > 0.0 {
                                (char_center_y - (y + padding.top * zoom)) / content_h
                            } else {
                                0.0
                            };
                            let color = sample_mesh_color(mesh, u, v, base_color);

                            if curve_text_enabled {
                                draw_char_outline(
                                    frame,
                                    *ch,
                                    Point::new(ch_x, line_y),
                                    &font_family_name,
                                    scaled_font_size,
                                    color,
                                );
                            } else {
                                frame.fill_text(Text {
                                    content: ch.to_string(),
                                    position: Point::new(ch_x, line_y),
                                    color,
                                    size: scaled_font_size.into(),
                                    font,
                                    ..Default::default()
                                });
                            }

                            cursor_x += ch_width;
                            if idx < chars.len() - 1 {
                                cursor_x += letter_spacing;
                            }
                        }
                    } else {
                        for color in &fill_colors {
                            if curve_text_enabled {
                                let mut cursor_x = 0.0;
                                let chars: Vec<char> = line.chars().collect();
                                for (idx, ch) in chars.iter().enumerate() {
                                    let ch_width = measure_char_advance(
                                        *ch,
                                        &font_family_name,
                                        scaled_font_size,
                                    );
                                    let ch_x = start_x + cursor_x;
                                    draw_char_outline(
                                        frame,
                                        *ch,
                                        Point::new(ch_x, line_y),
                                        &font_family_name,
                                        scaled_font_size,
                                        *color,
                                    );
                                    cursor_x += ch_width;
                                    if idx < chars.len() - 1 {
                                        cursor_x += letter_spacing;
                                    }
                                }
                            } else if letter_spacing != 0.0 {
                                let mut cursor_x = 0.0;
                                let chars: Vec<char> = line.chars().collect();
                                for (idx, ch) in chars.iter().enumerate() {
                                    let ch_width = measure_text_width_with_font(
                                        &ch.to_string(),
                                        &font_family_name,
                                        scaled_font_size,
                                        0.0,
                                    );
                                    frame.fill_text(Text {
                                        content: ch.to_string(),
                                        position: Point::new(start_x + cursor_x, line_y),
                                        color: *color,
                                        size: scaled_font_size.into(),
                                        font,
                                        ..Default::default()
                                    });
                                    cursor_x += ch_width;
                                    if idx < chars.len() - 1 {
                                        cursor_x += letter_spacing;
                                    }
                                }
                            } else {
                                frame.fill_text(Text {
                                    content: line.clone(),
                                    position: Point::new(start_x, line_y),
                                    color: *color,
                                    size: scaled_font_size.into(),
                                    font,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                let decoration = tailwind_style
                    .text_decoration
                    .as_deref()
                    .or(element.text_decoration.as_deref());

                if let Some(dec) = decoration {
                    let color = tailwind_style
                        .text_color
                        .or_else(|| {
                            element
                                .color
                                .as_ref()
                                .map(|s| parse_color(s, &doc.variables, theme_mode))
                        })
                        .unwrap_or(Color::BLACK);

                    draw_text_decoration(
                        frame,
                        dec,
                        start_x,
                        line_y,
                        line_w,
                        scaled_font_size,
                        color,
                        zoom,
                    );
                }
            }
        };

        if rotation != 0.0 {
            frame.with_save(|frame| {
                frame.translate(Vector::new(center_x, center_y));
                frame.rotate(rotation_rad);
                frame.translate(Vector::new(-center_x, -center_y));
                draw_tailwind_box(frame, bounds, zoom, tailwind_style);
                draw_lines(frame);
            });
        } else {
            draw_tailwind_box(frame, bounds, zoom, tailwind_style);
            draw_lines(frame);
        }
    }
}

#[cfg(test)]
#[path = "typography_tests.rs"]
mod typography_tests;
