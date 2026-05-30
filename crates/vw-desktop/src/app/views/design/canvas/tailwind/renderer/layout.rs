use iced::widget::canvas::fill::Rule as FillRule;
use iced::widget::canvas::{Fill, Frame, Style as CanvasStyle};
use iced::widget::image::Handle;
use iced::{Color, Point, Size};
use std::collections::HashMap;

use crate::app::views::design::canvas::rendering::svg::build_svg_path;
use crate::app::views::design::canvas::rendering::utils::{
    draw_image_from_cache, draw_tailwind_box, draw_tailwind_outline,
};

use super::super::dom::TailwindNode;
use super::super::parser::ParsedStyle;

use super::{
    ComputedNodeLayout, effective_divide_x_reverse, effective_divide_y_reverse, render_node_at,
    resolve_node_frame, resolve_svg_render_origin, resolve_svg_view_box, resolve_visual_style,
};

pub(super) fn render_node_with_style(
    frame: &mut Frame,
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    zoom: f32,
    inherited_text_style: &ParsedStyle,
    images: &HashMap<String, Handle>,
    style: ParsedStyle,
) {
    let visual_style = resolve_visual_style(&style, inherited_text_style);
    let frame_layout = resolve_node_frame(&style, layout.rect, zoom);
    let draw_bounds = layout.rect;

    // Draw background
    if let Some(_bg_color) = style.background_color {
        let _radius = style.border_radius.unwrap_or(0.0) * zoom;
        // If height is not fixed, we need to calculate it from children first.
        // But to draw background behind children, we need size.
        // So we must measure first.

        // Since we are doing immediate mode rendering here, we can't easily measure then draw without 2 passes.
        // However, we can just render children, calculate total height, then draw background?
        // No, background must be drawn first (painters algorithm).

        // Two pass approach:
        // 1. Measure (layout)
        // 2. Draw

        // OR: Draw background later? No, it will cover children.

        // Workaround:
        // If fixed height is known, draw now.
        // If auto height, we are in trouble.
        // We can just draw background *after* measuring, but *before* children?
        // Impossible in immediate mode unless we buffer commands.

        // Solution: Recursion returns the size.
        // We can traverse twice.
        // Or simpler: Draw background assuming a size?

        // Let's implement a simple 2-pass for this node if it has background.
        // Or actually, `iced` Canvas Frame is a list of commands.
        // If we push background rect first, it's at the bottom.
        // So we can compute layout, then push background, then push children?
        // Wait, `Frame` operations are executed in order.

        // Actually, we can use `frame.with_save` or similar? No.

        // Let's do a simplified layout pass first.
    }

    if node.tag == "svg" {
        let view_box = resolve_svg_view_box(node);
        let (origin, scale) = resolve_svg_render_origin(draw_bounds, view_box);

        for child in &node.children {
            if child.tag == "path"
                && let Some(d) = child.attributes.get("d")
            {
                let color = visual_style
                    .text_color
                    .or(inherited_text_style.text_color)
                    .unwrap_or(Color::BLACK);

                let rule = if let Some(r) = child.attributes.get("fill-rule") {
                    match r.as_str() {
                        "evenodd" => FillRule::EvenOdd,
                        _ => FillRule::NonZero,
                    }
                } else {
                    FillRule::NonZero
                };

                if let Some(path) = build_svg_path(d, origin, scale) {
                    frame.fill(&path, Fill { style: CanvasStyle::Solid(color), rule });
                }
            }
        }
        return;
    }

    if node.tag == "img" {
        let src = node.attributes.get("src").map(|s| s.as_str()).unwrap_or("");
        if !src.is_empty() && draw_image_from_cache(frame, draw_bounds, images, src) {
            return;
        }

        if style.background_color.is_none() {
            frame.fill_rectangle(
                Point::new(draw_bounds.x, draw_bounds.y),
                Size::new(draw_bounds.width, draw_bounds.height),
                visual_style.background_color.unwrap_or(Color::from_rgba(
                    0.9,
                    0.9,
                    0.9,
                    visual_style.opacity.unwrap_or(1.0),
                )),
            );
        }

        return;
    }

    draw_tailwind_box(frame, draw_bounds, zoom, &visual_style);

    for (node_idx, child_layout) in &layout.children {
        let child = &node.children[*node_idx];
        render_node_at(frame, child, child_layout, zoom, inherited_text_style, images);
    }

    // Divide between children
    if let Some(w) = style.divide_x_width {
        let color = visual_style.border_color.unwrap_or(Color::from_rgb(0.8, 0.8, 0.8));
        let sep_w = w * zoom;
        let divide_x_reverse = effective_divide_x_reverse(&style);
        for i in 0..layout.children.len().saturating_sub(1) {
            let a = &layout.children[i].1.rect;
            let x = if divide_x_reverse { a.x } else { a.x + a.width };
            frame.fill_rectangle(
                Point::new(x, draw_bounds.y + frame_layout.pt),
                Size::new(sep_w, draw_bounds.height - frame_layout.pt - frame_layout.pb),
                color,
            );
        }
    }
    if let Some(w) = style.divide_y_width {
        let color = visual_style.border_color.unwrap_or(Color::from_rgb(0.8, 0.8, 0.8));
        let sep_h = w * zoom;
        let divide_y_reverse = effective_divide_y_reverse(&style);
        for i in 0..layout.children.len().saturating_sub(1) {
            let a = &layout.children[i].1.rect;
            let y = if divide_y_reverse { a.y } else { a.y + a.height };
            frame.fill_rectangle(
                Point::new(draw_bounds.x + frame_layout.pl, y),
                Size::new(draw_bounds.width - frame_layout.pl - frame_layout.pr, sep_h),
                color,
            );
        }
    }

    draw_tailwind_outline(frame, draw_bounds, zoom, &visual_style);
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;
