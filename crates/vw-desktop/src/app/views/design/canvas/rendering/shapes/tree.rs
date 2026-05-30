//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

use iced::{
    Color, Pixels, Point, Rectangle, Size,
    widget::canvas::{Frame, Image, Path, Stroke, Text},
};

use super::{
    fills::{
        draw_image_fill, draw_mesh_fill, extract_first_enabled_image_fill,
        extract_first_enabled_mesh_fill,
    },
    helpers::{
        clamp_child_size_to_content, draw_slot_hatch, expand_slot_children, is_brush_path_class,
    },
    shadow::draw_shadow,
    stroke::{
        DeferredStroke, StrokeAlign, StrokeSides, draw_deferred_stroke, parse_stroke_paint,
        parse_stroke_sides,
    },
};
use crate::app::assets;
use crate::app::views::design::canvas::tailwind::parser::TailwindParser;
use crate::app::views::design::{
    canvas::{
        layout::{compute_layout, parse_layout, parse_padding, resolve_element_size},
        parse::{parse_corner_radii, parse_fills, parse_shadow},
        tailwind::{parse_html, render},
        utils::{resolve_ref_instance, theme_mode_for_element},
    },
    models::{DesignDoc, DesignElement},
};

use super::super::svg::{build_svg_path, build_svg_path_fit};
use super::super::utils::{draw_image_from_cache, element_path, element_path_radius};

/// 公开的 draw_shapes_tree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_shapes_tree(
    frame: &mut Frame,
    element: &DesignElement,
    parent_pos: Point,
    zoom: f32,
    doc: &DesignDoc,
    overrides: Option<&serde_json::Map<String, serde_json::Value>>,
    parent_size: Option<Size>,
    size_override: Option<Size>,
    _is_root: bool,
    inherited_theme_mode: Option<&str>,
    show_slot_content: bool,
    show_slot_overflow: bool,
) {
    if element.kind == "ref" {
        if let Some(inst) = resolve_ref_instance(element, &doc.children, overrides) {
            let base_pos =
                Point::new(parent_pos.x + (element.x * zoom), parent_pos.y + (element.y * zoom));
            let override_size = size_override.unwrap_or_else(|| {
                resolve_element_size(element, parent_size, doc, inherited_theme_mode)
            });
            draw_shapes_tree(
                frame,
                &inst,
                base_pos,
                zoom,
                doc,
                overrides,
                parent_size,
                Some(override_size),
                false,
                inherited_theme_mode,
                show_slot_content,
                show_slot_overflow,
            );
        }
        return;
    }

    if element.enabled == Some(false) || element.visible == Some(false) {
        return;
    }

    let x = parent_pos.x + (element.x * zoom);
    let y = parent_pos.y + (element.y * zoom);
    let resolved = size_override
        .unwrap_or_else(|| resolve_element_size(element, parent_size, doc, inherited_theme_mode));
    let w = resolved.width * zoom;
    let h = resolved.height * zoom;

    let theme_mode = theme_mode_for_element(doc, element, inherited_theme_mode);
    let padding = parse_padding(&element.padding, &doc.variables, theme_mode);
    let mut corner_radii = parse_corner_radii(
        &element.corner_radius,
        resolved.width,
        resolved.height,
        &doc.variables,
        theme_mode,
    );
    let mut radius = corner_radii
        .top_left
        .max(corner_radii.top_right)
        .max(corner_radii.bottom_right)
        .max(corner_radii.bottom_left);
    let fill_colors = parse_fills(&element.fill, &doc.variables, theme_mode);
    let stroke_align = element.stroke.as_ref().and_then(|s| s.align.as_deref());
    let (stroke_color, stroke_dash_segments) = parse_stroke_paint(
        element.stroke.as_ref().and_then(|s| s.fill.as_deref()),
        &doc.variables,
        theme_mode,
    );
    let stroke_sides = parse_stroke_sides(
        element.stroke.as_ref().and_then(|s| s.thickness.as_ref()),
        &doc.variables,
        theme_mode,
    );
    let layout = parse_layout(&element.layout).or_else(|| {
        if let Some(l) = &element.layout
            && l == "none"
        {
            return None;
        }
        if element.justify_content.is_some()
            || element.align_items.is_some()
            || element.gap.is_some()
        {
            Some(crate::app::views::design::canvas::types::LayoutDirection::Horizontal)
        } else {
            None
        }
    });

    let mut deferred_stroke: Option<DeferredStroke> = None;

    if element.kind != "Typography" && element.kind != "text" {
        if element.kind == "tailwind"
            && let Some(content) = &element.content
        {
            let nodes = parse_html(content);
            render(frame, &nodes, Rectangle { x, y, width: w, height: h }, zoom, &doc.images);
        }

        if let Some(shadow) = parse_shadow(&element.effect, &doc.variables, theme_mode) {
            draw_shadow(
                frame,
                x + padding.left * zoom,
                y + padding.top * zoom,
                w - (padding.left + padding.right) * zoom,
                h - (padding.top + padding.bottom) * zoom,
                radius * zoom,
                shadow,
                element.kind.as_str(),
            );
        }

        let stroke_align_mode = StrokeAlign::from_str(stroke_align);
        let stroke_width_uniform = stroke_sides
            .is_uniform()
            .unwrap_or_else(|| if element.kind == "path" { stroke_sides.max() } else { 0.0 })
            * zoom;
        let align_inset = match stroke_align_mode {
            StrokeAlign::Inside => stroke_width_uniform / 2.0,
            StrokeAlign::Outside => -stroke_width_uniform / 2.0,
            StrokeAlign::Center => 0.0,
        };
        let tailwind_style =
            element.class.as_deref().map(TailwindParser::parse).unwrap_or_default();
        if let Some(r) = tailwind_style.border_radius {
            if element.class.as_deref().map(|s| s.contains("rounded-full")).unwrap_or(false) {
                let full = (resolved.width.min(resolved.height)) / 2.0;
                corner_radii = full.into();
                radius = full;
            } else {
                corner_radii = r.into();
                radius = r;
            }
        }

        let (path_x, path_y, path_w, path_h, path_radius, path_radii) = if element.kind == "path" {
            (x, y, w, h, radius * zoom, iced::border::Radius::from(0.0))
        } else {
            let w_adj = (w - align_inset * 2.0).max(0.0);
            let h_adj = (h - align_inset * 2.0).max(0.0);
            let r_adj = (radius * zoom - align_inset).max(0.0);
            let max_r = (w_adj.min(h_adj) / 2.0).max(0.0);
            let clamp = |v: f32| v.clamp(0.0, max_r);
            let r_px = iced::border::Radius {
                top_left: corner_radii.top_left * zoom,
                top_right: corner_radii.top_right * zoom,
                bottom_right: corner_radii.bottom_right * zoom,
                bottom_left: corner_radii.bottom_left * zoom,
            };
            let r_adj_px = iced::border::Radius {
                top_left: clamp(r_px.top_left - align_inset),
                top_right: clamp(r_px.top_right - align_inset),
                bottom_right: clamp(r_px.bottom_right - align_inset),
                bottom_left: clamp(r_px.bottom_left - align_inset),
            };
            (x + align_inset, y + align_inset, w_adj, h_adj, r_adj, r_adj_px)
        };
        let path = if element.kind == "path" {
            element
                .geometry
                .as_deref()
                .and_then(|geometry| {
                    if path_w <= f32::EPSILON || path_h <= f32::EPSILON {
                        build_svg_path(geometry, Point::new(path_x, path_y), zoom)
                    } else {
                        build_svg_path_fit(
                            geometry,
                            Point::new(path_x, path_y),
                            Size::new(path_w, path_h),
                        )
                        .or_else(|| build_svg_path(geometry, Point::new(path_x, path_y), zoom))
                    }
                })
                .unwrap_or_else(|| {
                    element_path(element.kind.as_str(), path_x, path_y, path_w, path_h, path_radius)
                })
        } else {
            element_path_radius(element.kind.as_str(), path_x, path_y, path_w, path_h, path_radii)
        };
        let bounds = Rectangle { x: path_x, y: path_y, width: path_w, height: path_h };

        if element.kind.eq_ignore_ascii_case("icon_font") {
            let icon_color =
                fill_colors.first().copied().unwrap_or_else(|| Color::from_rgba8(17, 24, 39, 1.0));
            if let (Some(family), Some(name)) =
                (element.icon_font_family.as_deref(), element.icon_font_name.as_deref())
                && let Some(handle) = assets::get_named_icon_image_with_weight(
                    family,
                    name,
                    element.weight.as_ref(),
                    icon_color,
                )
            {
                frame.draw_image(
                    Rectangle { x: path_x, y: path_y, width: path_w, height: path_h },
                    Image::new(handle),
                );
            } else {
                frame.fill(&path, icon_color);
            }
        } else if element.kind == "image" {
            let mut url = "";
            if let Some(serde_json::Value::Object(map)) = &element.fill
                && let Some(serde_json::Value::String(u)) = map.get("url")
            {
                url = u;
            }

            let mut drawn = false;
            if !url.is_empty() {
                drawn = draw_image_from_cache(
                    frame,
                    Rectangle { x: path_x, y: path_y, width: path_w, height: path_h },
                    &doc.images,
                    url,
                );
            }

            if !drawn {
                frame.fill(&path, Color::from_rgb(0.9, 0.9, 0.95));
                frame.stroke(
                    &path,
                    Stroke::default().with_color(Color::from_rgb(0.8, 0.8, 0.8)).with_width(1.0),
                );

                if !url.is_empty() {
                    let short_url =
                        if url.len() > 30 { format!("{}...", &url[..27]) } else { url.to_string() };
                    frame.fill_text(Text {
                        content: short_url,
                        position: Point::new(
                            path_x + 5.0 * zoom,
                            path_y + path_h / 2.0 - 6.0 * zoom,
                        ),
                        size: Pixels(12.0 * zoom),
                        color: Color::from_rgb(0.4, 0.4, 0.4),
                        ..Default::default()
                    });
                }
            }
        } else if let Some(img) = extract_first_enabled_image_fill(&element.fill) {
            let drawn = draw_image_fill(frame, bounds, doc, &img);
            if !drawn {
                frame.fill(&path, Color::from_rgb8(0xE6, 0xE6, 0xE6));
            }
        } else if let Some(mesh) = extract_first_enabled_mesh_fill(&element.fill) {
            draw_mesh_fill(frame, bounds, &mesh, &doc.variables, theme_mode, mesh.outline);
        } else if !fill_colors.is_empty() {
            for color in &fill_colors {
                frame.fill(&path, *color);
            }
        } else if let Some(bg) = tailwind_style.background_color {
            frame.fill(&path, bg);
        }

        if element.children.is_empty()
            && let Some(serde_json::Value::Array(arr)) = element.slot.as_ref()
            && (arr.is_empty() || !show_slot_content)
        {
            let hatch_color = Color::from_rgba8(196, 131, 255, 0.55);
            let border_color = Color::from_rgba8(196, 131, 255, 0.9);
            let inset = (10.0 * zoom).clamp(6.0, 24.0);
            let hatch_bounds = Rectangle {
                x: bounds.x + inset,
                y: bounds.y + inset,
                width: (bounds.width - inset * 2.0).max(0.0),
                height: (bounds.height - inset * 2.0).max(0.0),
            };
            draw_slot_hatch(frame, hatch_bounds, zoom, hatch_color);
            let border_w = (1.0 * zoom).clamp(0.8, 2.0);
            frame.stroke(
                &Path::rectangle(
                    Point::new(hatch_bounds.x, hatch_bounds.y),
                    Size::new(hatch_bounds.width, hatch_bounds.height),
                ),
                Stroke::default().with_color(border_color).with_width(border_w),
            );
        }

        if element.stroke.is_some() && stroke_color.a > 0.0 {
            if element.kind == "path" {
                if stroke_width_uniform > 0.0 {
                    deferred_stroke = Some(DeferredStroke::Path {
                        path,
                        color: stroke_color,
                        width: stroke_width_uniform,
                        dash_segments: stroke_dash_segments.clone(),
                        round_cap: is_brush_path_class(element.class.as_deref()),
                    });
                }
            } else if let Some(uniform) = stroke_sides.is_uniform() {
                let stroke_width = uniform * zoom;
                if stroke_width > 0.0 {
                    deferred_stroke = Some(DeferredStroke::Path {
                        path,
                        color: stroke_color,
                        width: stroke_width,
                        dash_segments: stroke_dash_segments,
                        round_cap: false,
                    });
                }
            } else {
                let sides_px = StrokeSides {
                    top: stroke_sides.top * zoom,
                    right: stroke_sides.right * zoom,
                    bottom: stroke_sides.bottom * zoom,
                    left: stroke_sides.left * zoom,
                };
                if sides_px.any_positive() {
                    deferred_stroke = Some(DeferredStroke::Sides {
                        x,
                        y,
                        w,
                        h,
                        sides_px,
                        align: stroke_align_mode,
                        color: stroke_color,
                    });
                }
            }
        }
    }

    let content_size = Size::new(
        (resolved.width - (padding.left + padding.right)).max(0.0),
        (resolved.height - (padding.top + padding.bottom)).max(0.0),
    );

    let slot_children_buf = if element.children.is_empty() && show_slot_content {
        expand_slot_children(element)
    } else {
        None
    };
    let children: &[DesignElement] =
        slot_children_buf.as_deref().unwrap_or_else(|| element.children.as_slice());

    let has_slot = element
        .slot
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    let clip_children = (element.clip == Some(true) || element.clip_content == Some(true))
        && !(show_slot_content && show_slot_overflow && has_slot);
    let propagate_corner_radius =
        element.kind == "frame" && clip_children && element.corner_radius.is_some();

    let draw_children = |frame: &mut Frame| {
        if let Some(direction) = layout {
            let layouts =
                compute_layout(direction, children, content_size, element, doc, theme_mode);
            for (child, layout) in children.iter().zip(layouts.into_iter()) {
                if let Some(map) = overrides
                    && let Some(serde_json::Value::Object(ov)) = map.get(&child.id)
                    && let Some(serde_json::Value::Bool(enabled)) = ov.get("enabled")
                    && !enabled
                {
                    continue;
                }

                if clip_children {
                    let cx = layout.offset.x;
                    let cy = layout.offset.y;
                    let cw = layout.size.width;
                    let ch = layout.size.height;
                    if cx >= content_size.width
                        || cy >= content_size.height
                        || (cx + cw) <= 0.0
                        || (cy + ch) <= 0.0
                    {
                        match direction {
                            crate::app::views::design::canvas::types::LayoutDirection::Vertical => {
                                if cy >= content_size.height {
                                    break;
                                }
                            }
                            crate::app::views::design::canvas::types::LayoutDirection::Horizontal => {
                                if cx >= content_size.width {
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                }

                let child_parent = Point::new(
                    x + (padding.left + layout.offset.x - child.x) * zoom,
                    y + (padding.top + layout.offset.y - child.y) * zoom,
                );
                if propagate_corner_radius
                    && child.corner_radius.is_none()
                    && (child.kind == "rectangle" || child.kind == "frame")
                    && child.x.abs() <= 0.01
                    && child.y.abs() <= 0.01
                    && (layout.size.height - content_size.height).abs() <= 0.5
                {
                    let mut inst = child.clone();
                    inst.corner_radius = element.corner_radius.clone();
                    draw_shapes_tree(
                        frame,
                        &inst,
                        child_parent,
                        zoom,
                        doc,
                        overrides,
                        Some(content_size),
                        Some(if clip_children {
                            clamp_child_size_to_content(
                                content_size,
                                layout.offset.x,
                                layout.offset.y,
                                layout.size,
                            )
                        } else {
                            layout.size
                        }),
                        false,
                        theme_mode,
                        show_slot_content,
                        show_slot_overflow,
                    );
                } else {
                    draw_shapes_tree(
                        frame,
                        child,
                        child_parent,
                        zoom,
                        doc,
                        overrides,
                        Some(content_size),
                        Some(if clip_children {
                            clamp_child_size_to_content(
                                content_size,
                                layout.offset.x,
                                layout.offset.y,
                                layout.size,
                            )
                        } else {
                            layout.size
                        }),
                        false,
                        theme_mode,
                        show_slot_content,
                        show_slot_overflow,
                    );
                }
            }
        } else {
            for child in children {
                if let Some(map) = overrides
                    && let Some(serde_json::Value::Object(ov)) = map.get(&child.id)
                    && let Some(serde_json::Value::Bool(enabled)) = ov.get("enabled")
                    && !enabled
                {
                    continue;
                }

                let child_size = resolve_element_size(child, Some(content_size), doc, theme_mode);
                let child_size_override = if clip_children {
                    Some(clamp_child_size_to_content(content_size, child.x, child.y, child_size))
                } else {
                    None
                };
                if propagate_corner_radius
                    && child.corner_radius.is_none()
                    && (child.kind == "rectangle" || child.kind == "frame")
                    && child.x.abs() <= 0.01
                    && child.y.abs() <= 0.01
                    && (child_size.height - content_size.height).abs() <= 0.5
                {
                    let mut inst = child.clone();
                    inst.corner_radius = element.corner_radius.clone();
                    draw_shapes_tree(
                        frame,
                        &inst,
                        Point::new(x + padding.left * zoom, y + padding.top * zoom),
                        zoom,
                        doc,
                        overrides,
                        Some(content_size),
                        child_size_override,
                        false,
                        theme_mode,
                        show_slot_content,
                        show_slot_overflow,
                    );
                } else {
                    draw_shapes_tree(
                        frame,
                        child,
                        Point::new(x + padding.left * zoom, y + padding.top * zoom),
                        zoom,
                        doc,
                        overrides,
                        Some(content_size),
                        child_size_override,
                        false,
                        theme_mode,
                        show_slot_content,
                        show_slot_overflow,
                    );
                }
            }
        }
    };
    draw_children(frame);

    if let Some(deferred) = deferred_stroke {
        draw_deferred_stroke(frame, deferred);
    }
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tree_tests;
