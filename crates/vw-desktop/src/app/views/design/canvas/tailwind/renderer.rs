//! Tailwind CSS 样式渲染器模块。
//!
//! 本模块保留对外渲染、布局导出与命中测试入口，
//! 具体实现按职责拆分到同级子模块中。

use iced::alignment::Horizontal;
use iced::widget::canvas::Frame;
use iced::widget::image::Handle;
use iced::{Point, Rectangle};
use std::collections::HashMap;

use super::dom::TailwindNode;
use super::parser::ParsedStyle;
use crate::app::views::design::canvas::rendering::utils::{
    compute_line_width, draw_text_decoration,
};

mod flex;
mod frame;
mod hit_test;
mod layout;
mod style;
mod text;
mod tree;

use self::hit_test::{bounds_for_node_path, hit_test_node};
pub use self::tree::TailwindNodeLayout;
use self::{
    flex::{effective_divide_x_reverse, effective_divide_y_reverse},
    frame::resolve_node_frame,
    style::{
        inherit_text_style, resolve_node_style, resolve_svg_render_origin, resolve_svg_view_box,
        resolve_visual_style,
    },
    text::{fill_text_with_spacing, resolve_text_layout},
    tree::{build_node_layout_tree, export_node_layout, ComputedNodeLayout},
};
#[cfg(test)]
use self::{text::text_layout_size, tree::child_layouts};

pub fn render(
    frame: &mut Frame,
    roots: &[TailwindNode],
    bounds: Rectangle,
    zoom: f32,
    images: &HashMap<String, Handle>,
) {
    let mut y_offset = bounds.y;

    for node in roots {
        let layout = build_node_layout_tree(
            node,
            Rectangle {
                x: bounds.x,
                y: y_offset,
                width: bounds.width,
                height: bounds.height - (y_offset - bounds.y),
            },
            zoom,
            None,
        );

        render_node(frame, node, &layout, zoom, None, images);
        y_offset += layout.size.height;
    }
}

pub fn layout_roots(
    roots: &[TailwindNode],
    bounds: Rectangle,
    zoom: f32,
) -> Vec<TailwindNodeLayout> {
    let mut y_offset = bounds.y;
    let mut layouts = Vec::with_capacity(roots.len());

    for node in roots {
        let layout = build_node_layout_tree(
            node,
            Rectangle {
                x: bounds.x,
                y: y_offset,
                width: bounds.width,
                height: bounds.height - (y_offset - bounds.y),
            },
            zoom,
            None,
        );
        y_offset += layout.size.height;
        layouts.push(export_node_layout(&layout));
    }

    layouts
}

pub fn hit_test_path(
    roots: &[TailwindNode],
    bounds: Rectangle,
    zoom: f32,
    point: Point,
) -> Option<Vec<usize>> {
    let mut y_offset = bounds.y;

    for (i, node) in roots.iter().enumerate() {
        let node_bounds = Rectangle {
            x: bounds.x,
            y: y_offset,
            width: bounds.width,
            height: bounds.height - (y_offset - bounds.y),
        };

        let layout = build_node_layout_tree(node, node_bounds, zoom, None);
        if let Some((path, _size)) = hit_test_node(node, &layout, point, vec![i]) {
            return Some(path);
        }

        y_offset += layout.size.height;
    }

    None
}

pub fn bounds_for_path(
    roots: &[TailwindNode],
    bounds: Rectangle,
    zoom: f32,
    path: &[usize],
) -> Option<Rectangle> {
    if path.is_empty() {
        return None;
    }

    let mut y_offset = bounds.y;

    for (i, node) in roots.iter().enumerate() {
        let node_bounds = Rectangle {
            x: bounds.x,
            y: y_offset,
            width: bounds.width,
            height: bounds.height - (y_offset - bounds.y),
        };
        let layout = build_node_layout_tree(node, node_bounds, zoom, None);

        if i == path[0] {
            let (rect, _) = bounds_for_node_path(node, &layout, &path[1..])?;
            return Some(rect);
        }

        y_offset += layout.size.height;
    }

    None
}

fn render_node(
    frame: &mut Frame,
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    zoom: f32,
    inherited_text_style: Option<&ParsedStyle>,
    images: &HashMap<String, Handle>,
) {
    if let Some(text_content) = &node.text {
        let text_style = inherited_text_style.cloned().unwrap_or_default();
        let text_layout = resolve_text_layout(text_content, layout.rect, zoom, &text_style);

        for (index, line) in text_layout.lines.iter().enumerate() {
            let line_width =
                compute_line_width(line, text_layout.font_size, text_layout.letter_spacing);
            let x = match text_layout.align {
                Horizontal::Center => {
                    layout.rect.x + (layout.rect.width - line_width).max(0.0) / 2.0
                }
                Horizontal::Right => layout.rect.x + (layout.rect.width - line_width).max(0.0),
                _ => layout.rect.x,
            };
            let y = layout.rect.y + index as f32 * text_layout.line_height;

            fill_text_with_spacing(frame, line, x, y, &text_layout);

            if let Some(decoration) = text_layout.decoration.as_deref() {
                draw_text_decoration(
                    frame,
                    decoration,
                    x,
                    y,
                    line_width,
                    text_layout.font_size,
                    text_layout.color,
                    zoom,
                );
            }
        }

        return;
    }

    let style = resolve_node_style(node);
    let effective_text_style = inherit_text_style(&style, inherited_text_style);

    render_node_with_style(frame, node, layout, zoom, &effective_text_style, images, style);
}

fn render_node_with_style(
    frame: &mut Frame,
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    zoom: f32,
    inherited_text_style: &ParsedStyle,
    images: &HashMap<String, Handle>,
    style: ParsedStyle,
) {
    layout::render_node_with_style(frame, node, layout, zoom, inherited_text_style, images, style);
}

fn render_node_at(
    frame: &mut Frame,
    node: &TailwindNode,
    layout: &ComputedNodeLayout,
    zoom: f32,
    parent_style: &ParsedStyle,
    images: &HashMap<String, Handle>,
) {
    if let Some(text_content) = &node.text {
        let text_layout = resolve_text_layout(text_content, layout.rect, zoom, parent_style);

        for (index, line) in text_layout.lines.iter().enumerate() {
            let y = layout.rect.y + index as f32 * text_layout.line_height;
            let mut start_x = match text_layout.align {
                Horizontal::Center => {
                    let line_width =
                        compute_line_width(line, text_layout.font_size, text_layout.letter_spacing);
                    layout.rect.x + (layout.rect.width - line_width).max(0.0) / 2.0
                }
                Horizontal::Right => {
                    let line_width =
                        compute_line_width(line, text_layout.font_size, text_layout.letter_spacing);
                    layout.rect.x + (layout.rect.width - line_width).max(0.0)
                }
                _ => layout.rect.x,
            };
            let mut drawn_width =
                compute_line_width(line, text_layout.font_size, text_layout.letter_spacing);

            if text_layout.is_justify && index < text_layout.lines.len() - 1 {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() > 1 {
                    let word_width: f32 = words
                        .iter()
                        .map(|word| {
                            compute_line_width(
                                word,
                                text_layout.font_size,
                                text_layout.letter_spacing,
                            )
                        })
                        .sum();
                    let spaces = (words.len() - 1) as f32;
                    let extra = ((layout.rect.width - word_width).max(0.0) / spaces).max(0.0);
                    let base_space = compute_line_width(" ", text_layout.font_size, 0.0);
                    let mut cursor_x = layout.rect.x;

                    for (word_index, word) in words.iter().enumerate() {
                        fill_text_with_spacing(frame, word, cursor_x, y, &text_layout);
                        cursor_x += compute_line_width(
                            word,
                            text_layout.font_size,
                            text_layout.letter_spacing,
                        );
                        if word_index < words.len() - 1 {
                            cursor_x += base_space + extra;
                        }
                    }

                    start_x = layout.rect.x;
                    drawn_width = layout.rect.width;
                } else {
                    fill_text_with_spacing(frame, line, start_x, y, &text_layout);
                }
            } else {
                fill_text_with_spacing(frame, line, start_x, y, &text_layout);
            }

            if let Some(decoration) = text_layout.decoration.as_deref() {
                draw_text_decoration(
                    frame,
                    decoration,
                    start_x,
                    y,
                    drawn_width,
                    text_layout.font_size,
                    text_layout.color,
                    zoom,
                );
            }
        }
        return;
    }

    render_node(frame, node, layout, zoom, Some(parent_style), images);
}

#[cfg(test)]
#[path = "renderer_tests.rs"]
mod renderer_tests;

#[cfg(test)]
#[path = "renderer_text_tests.rs"]
mod renderer_text_tests;

#[cfg(test)]
#[path = "renderer_media_tests.rs"]
mod renderer_media_tests;

#[cfg(test)]
#[path = "renderer_transform_tests.rs"]
mod renderer_transform_tests;

#[cfg(test)]
#[path = "renderer_flex_tests.rs"]
mod renderer_flex_tests;

#[cfg(test)]
#[path = "renderer_flex_align_tests.rs"]
mod renderer_flex_align_tests;

#[cfg(test)]
#[path = "renderer_flex_direction_tests.rs"]
mod renderer_flex_direction_tests;

#[cfg(test)]
#[path = "renderer_flex_item_tests.rs"]
mod renderer_flex_item_tests;

#[cfg(test)]
#[path = "renderer_grid_tests.rs"]
mod renderer_grid_tests;

#[cfg(test)]
#[path = "renderer_size_constraint_tests.rs"]
mod renderer_size_constraint_tests;

#[cfg(test)]
#[path = "renderer_effect_tests.rs"]
mod renderer_effect_tests;

#[cfg(test)]
#[path = "renderer_layout_regression_tests.rs"]
mod renderer_layout_regression_tests;
