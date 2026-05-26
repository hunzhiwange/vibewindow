//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::{Rectangle, Size};

use super::super::dom::TailwindNode;
use super::super::parser::ParsedStyle;
use super::flex::{
    FlexItemConstraints, apply_flex_alignment, apply_flex_item_constraints,
    is_reverse_flex_direction, is_row_layout,
};
use super::frame::resolve_node_frame;
use super::style::{
    inherit_text_style, resolve_img_rect, resolve_node_style, resolve_svg_rect,
    resolve_svg_view_box, visual_bounds_for_style,
};
use super::text::resolve_text_layout;

#[derive(Debug, Clone, Copy)]
struct NodeLayoutResult {
    rect: Rectangle,
    size: Size,
}

#[derive(Debug, Clone)]
/// ComputedNodeLayout 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct ComputedNodeLayout {
    pub(super) rect: Rectangle,
    pub(super) visual_rect: Rectangle,
    pub(super) size: Size,
    pub(super) children: Vec<(usize, ComputedNodeLayout)>,
}

#[derive(Debug, Clone)]
/// TailwindNodeLayout 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub struct TailwindNodeLayout {
    pub rect: Rectangle,
    pub size: Size,
    pub children: Vec<TailwindNodeLayout>,
}

/// 根据设计文档或元素生成外部表示。
///
/// 返回生成后的内容；找不到指定元素时返回 `None`，避免导出不完整结果。
pub(super) fn export_node_layout(layout: &ComputedNodeLayout) -> TailwindNodeLayout {
    TailwindNodeLayout {
        rect: layout.rect,
        size: layout.size,
        children: layout
            .children
            .iter()
            .map(|(_, child_layout)| export_node_layout(child_layout))
            .collect(),
    }
}

/// 执行 build_node_layout_tree 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn build_node_layout_tree(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
    inherited_text_style: Option<&ParsedStyle>,
) -> ComputedNodeLayout {
    let style = if node.text.is_some() { ParsedStyle::default() } else { resolve_node_style(node) };
    let effective_text_style = if node.text.is_some() {
        inherited_text_style.cloned().unwrap_or_default()
    } else {
        inherit_text_style(&style, inherited_text_style)
    };

    let NodeLayoutResult { rect, size } =
        node_layout_with_style(node, bounds, zoom, &style, &effective_text_style);

    let children = child_layouts_with_style(node, bounds, zoom, &style, &effective_text_style)
        .map(|(_, child_rects)| {
            child_rects
                .into_iter()
                .map(|(child_index, child_rect)| {
                    let child_style = resolve_node_style(&node.children[child_index]);
                    let mut child_layout = build_node_layout_tree(
                        &node.children[child_index],
                        child_rect,
                        zoom,
                        Some(&effective_text_style),
                    );

                    if child_layout.rect.width == 0.0
                        && child_rect.width.is_finite()
                        && child_rect.width > 0.0
                        && child_style.width.is_none()
                    {
                        child_layout.rect.width = child_rect.width;
                        child_layout.size.width = child_layout.size.width.max(child_rect.width);
                    }

                    if child_layout.rect.height == 0.0
                        && child_rect.height.is_finite()
                        && child_rect.height > 0.0
                        && child_style.height.is_none()
                    {
                        child_layout.rect.height = child_rect.height;
                        child_layout.size.height = child_layout.size.height.max(child_rect.height);
                    }

                    child_layout.visual_rect = visual_bounds_for_style(&child_style, child_layout.rect, zoom);
                    (
                        child_index,
                        child_layout,
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let mut layout = ComputedNodeLayout {
        rect,
        visual_rect: visual_bounds_for_style(&style, rect, zoom),
        size,
        children,
    };
    let translate_x = style.translate_x.unwrap_or(0.0) * zoom;
    let translate_y = style.translate_y.unwrap_or(0.0) * zoom;

    if translate_x != 0.0 || translate_y != 0.0 {
        offset_layout_tree(&mut layout, translate_x, translate_y);
    }

    layout
}

#[cfg(test)]
/// 执行 child_layouts 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn child_layouts(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
) -> Option<(Rectangle, Vec<(usize, Rectangle)>)> {
    let layout = build_node_layout_tree(node, bounds, zoom, None);

    if layout.children.is_empty() && (node.text.is_some() || node.tag == "svg" || node.tag == "img")
    {
        return None;
    }

    Some((
        layout.rect,
        layout.children.into_iter().map(|(index, child)| (index, child.rect)).collect(),
    ))
}

fn measure_node(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
    inherited_text_style: Option<&ParsedStyle>,
) -> Size {
    if let Some(text) = &node.text {
        let layout = resolve_text_layout(
            text,
            bounds,
            zoom,
            &inherited_text_style.cloned().unwrap_or_default(),
        );
        return text_layout_size(&layout);
    }

    let style = resolve_node_style(node);
    let effective_text_style = inherit_text_style(&style, inherited_text_style);
    let frame = resolve_node_frame(&style, bounds, zoom);

    if let Some(h) = frame.height_fixed {
        return Size::new(frame.width, h);
    }

    let is_grid = style.display.as_deref() == Some("grid");
    let is_row = is_row_layout(&style);
    let grid_cols = style.grid_cols.unwrap_or(1).max(1);

    let mut current_w: f32 = 0.0;
    let mut current_h: f32 = 0.0;
    let mut grid_col_idx = 0;
    let mut grid_max_row_height: f32 = 0.0;

    let content_bounds = frame.content_bounds();
    let grid_col_width = if is_grid {
        (content_bounds.width - (frame.gap_x * (grid_cols as f32 - 1.0))).max(0.0)
            / (grid_cols as f32)
    } else {
        0.0
    };

    for child in &node.children {
        let child_style = resolve_node_style(child);
        if child_style.position.as_deref() == Some("absolute") {
            continue;
        }

        let available_width = if is_grid { grid_col_width } else { content_bounds.width };
        let child_size = measure_node(
            child,
            Rectangle { width: available_width, ..bounds },
            zoom,
            Some(&effective_text_style),
        );

        if is_grid {
            grid_max_row_height = grid_max_row_height.max(child_size.height);
            grid_col_idx += 1;

            if grid_col_idx >= grid_cols {
                grid_col_idx = 0;
                current_h += grid_max_row_height + frame.gap_y;
                grid_max_row_height = 0.0;
            }

            current_w = frame.width;
        } else if is_row {
            current_w += child_size.width + frame.gap_x;
            current_h = current_h.max(child_size.height);
        } else {
            current_h += child_size.height + frame.gap_y;
            current_w = current_w.max(child_size.width);
        }
    }

    if is_grid {
        if grid_col_idx > 0 {
            current_h += grid_max_row_height;
        } else if current_h > 0.0 {
            current_h -= frame.gap_y;
        }
    } else if current_h > 0.0 {
        current_h -= frame.gap_y;
    }

    Size::new(frame.width, current_h + frame.pt + frame.pb)
}

fn node_layout_with_style(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
    style: &ParsedStyle,
    effective_text_style: &ParsedStyle,
) -> NodeLayoutResult {
    if node.text.is_some() {
        let size = measure_node(node, bounds, zoom, Some(effective_text_style));
        return NodeLayoutResult { rect: bounds, size };
    }

    if let Some(layout) = resolve_replaced_node_layout(node, bounds, zoom, style) {
        return layout;
    }

    let frame = resolve_node_frame(style, bounds, zoom);
    let mut draw_bounds = frame.draw_bounds;

    let content_bounds = frame.content_bounds();
    let content_x = content_bounds.x;
    let mut current_y = content_bounds.y;
    let mut max_row_height: f32 = 0.0;
    let mut current_row_x = content_x;

    let is_grid = style.display.as_deref() == Some("grid");
    let is_row = is_row_layout(style);
    let grid_cols = style.grid_cols.unwrap_or(1).max(1);

    let mut grid_col_idx = 0;
    let grid_col_width = if is_grid {
        (content_bounds.width - (frame.gap_x * (grid_cols as f32 - 1.0))).max(0.0)
            / (grid_cols as f32)
    } else {
        0.0
    };

    for child in &node.children {
        let child_style = resolve_node_style(child);
        if child_style.position.as_deref() == Some("absolute") {
            continue;
        }

        let child_bounds = if is_grid {
            Rectangle {
                x: current_row_x,
                y: current_y,
                width: grid_col_width,
                height: f32::INFINITY,
            }
        } else {
            Rectangle {
                x: current_row_x,
                y: current_y,
                width: content_bounds.width,
                height: f32::INFINITY,
            }
        };

        let child_size = measure_node(child, child_bounds, zoom, Some(effective_text_style));

        if is_grid {
            max_row_height = max_row_height.max(child_size.height);
            grid_col_idx += 1;
            if grid_col_idx >= grid_cols {
                grid_col_idx = 0;
                current_y += max_row_height + frame.gap_y;
                max_row_height = 0.0;
            }
        } else if is_row {
            current_row_x += child_size.width + frame.gap_x;
            max_row_height = max_row_height.max(child_size.height);
        } else {
            current_y += child_size.height + frame.gap_y;
        }
    }

    if is_grid && grid_col_idx > 0 {
        current_y += max_row_height;
    }

    let content_width = if is_row {
        (current_row_x - content_x - frame.gap_x).max(0.0)
    } else {
        content_bounds.width
    };
    let content_height = if is_grid {
        if grid_col_idx > 0 {
            (current_y - draw_bounds.y - frame.pt).max(0.0)
        } else {
            (current_y - draw_bounds.y - frame.pt - frame.gap_y).max(0.0)
        }
    } else if is_row {
        max_row_height
    } else {
        (current_y - draw_bounds.y - frame.pt - frame.gap_y).max(0.0)
    };

    let final_width = if style.width.is_some() || style.max_width.is_some() {
        frame.width
    } else {
        content_width + frame.pl + frame.pr + frame.ml + frame.mr
    };
    let final_height =
        if let Some(h) = frame.height_fixed { h } else { content_height + frame.pt + frame.pb };

    draw_bounds.height = final_height;

    NodeLayoutResult {
        rect: draw_bounds,
        size: Size::new(final_width, final_height + frame.mt + frame.mb),
    }
}

fn child_layouts_with_style(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
    style: &ParsedStyle,
    effective_text_style: &ParsedStyle,
) -> Option<(Rectangle, Vec<(usize, Rectangle)>)> {
    if node.text.is_some() || node.tag == "svg" || node.tag == "img" {
        return None;
    }

    let frame = resolve_node_frame(style, bounds, zoom);
    let mut draw_bounds = frame.draw_bounds;

    let content_bounds = frame.content_bounds();
    let content_x = content_bounds.x;
    let mut current_y = content_bounds.y;
    let mut max_row_height: f32 = 0.0;
    let mut current_row_x = content_x;

    let is_grid = style.display.as_deref() == Some("grid");
    let is_flex = style.display.as_deref().map(|s| s.contains("flex")).unwrap_or(false);
    let is_row = is_row_layout(style);
    let is_reverse = is_reverse_flex_direction(style);
    let is_flex_layout = !is_grid && (is_flex || style.flex_direction.is_some());
    let grid_cols = style.grid_cols.unwrap_or(1).max(1);

    let align_items = style.align_items.as_deref().unwrap_or("stretch");
    let justify_content = style.justify_content.as_deref().unwrap_or("flex-start");

    let mut grid_col_idx = 0;
    let grid_col_width = if is_grid {
        (content_bounds.width - (frame.gap_x * (grid_cols as f32 - 1.0))).max(0.0)
            / (grid_cols as f32)
    } else {
        0.0
    };

    let mut out: Vec<(usize, Rectangle)> = Vec::new();
    let mut flow_child_indices: Vec<usize> = Vec::new();
    let mut flow_child_auto_cross_sizes: Vec<bool> = Vec::new();
    let mut flow_child_constraints: Vec<FlexItemConstraints> = Vec::new();

    for (i, child) in node.children.iter().enumerate() {
        let child_style = resolve_node_style(child);

        if child_style.position.as_deref() == Some("absolute") {
            let mut abs_rect =
                Rectangle { x: content_bounds.x, y: content_bounds.y, width: 0.0, height: 0.0 };

            let child_size = measure_node(
                child,
                Rectangle {
                    x: abs_rect.x,
                    y: abs_rect.y,
                    width: draw_bounds.width,
                    height: f32::INFINITY,
                },
                zoom,
                Some(effective_text_style),
            );
            abs_rect.width = child_size.width;
            abs_rect.height = child_size.height;

            if let Some(top) = child_style.top {
                abs_rect.y = draw_bounds.y + top * zoom;
            } else if let Some(bottom) = child_style.bottom {
                abs_rect.y = draw_bounds.y + draw_bounds.height - child_size.height - bottom * zoom;
            }
            if let Some(left) = child_style.left {
                abs_rect.x = draw_bounds.x + left * zoom;
            } else if let Some(right) = child_style.right {
                abs_rect.x = draw_bounds.x + draw_bounds.width - child_size.width - right * zoom;
            }

            out.push((i, abs_rect));
            continue;
        }

        if is_grid {
            let child_size = measure_node(
                child,
                Rectangle {
                    x: current_row_x,
                    y: current_y,
                    width: grid_col_width,
                    height: f32::INFINITY,
                },
                zoom,
                Some(effective_text_style),
            );

            let x_pos = content_x + (grid_col_idx as f32) * (grid_col_width + frame.gap_x);
            out.push((
                i,
                Rectangle {
                    x: x_pos,
                    y: current_y,
                    width: grid_col_width,
                    height: child_size.height,
                },
            ));
            flow_child_indices.push(out.len() - 1);
            flow_child_auto_cross_sizes.push(if is_row {
                child_style.height.is_none()
            } else {
                child_style.width.is_none()
            });
            flow_child_constraints.push(FlexItemConstraints {
                grow: child_style.flex_grow.unwrap_or(0.0),
                shrink: child_style.flex_shrink.unwrap_or(1.0),
                basis: child_style.flex_basis.map(|basis| basis * zoom),
            });

            max_row_height = max_row_height.max(child_size.height);
            grid_col_idx += 1;
            if grid_col_idx >= grid_cols {
                grid_col_idx = 0;
                current_y += max_row_height + frame.gap_y;
                max_row_height = 0.0;
            }
        } else if is_row {
            let child_size = measure_node(
                child,
                Rectangle {
                    x: current_row_x,
                    y: current_y,
                    width: content_bounds.width,
                    height: f32::INFINITY,
                },
                zoom,
                Some(effective_text_style),
            );

            out.push((
                i,
                Rectangle {
                    x: current_row_x,
                    y: current_y,
                    width: child_size.width,
                    height: child_size.height,
                },
            ));
            flow_child_indices.push(out.len() - 1);
            flow_child_auto_cross_sizes.push(child_style.height.is_none());
            flow_child_constraints.push(FlexItemConstraints {
                grow: child_style.flex_grow.unwrap_or(0.0),
                shrink: child_style.flex_shrink.unwrap_or(1.0),
                basis: child_style.flex_basis.map(|basis| basis * zoom),
            });
            current_row_x += child_size.width + frame.gap_x;
            max_row_height = max_row_height.max(child_size.height);
        } else {
            let child_size = measure_node(
                child,
                Rectangle {
                    x: current_row_x,
                    y: current_y,
                    width: content_bounds.width,
                    height: f32::INFINITY,
                },
                zoom,
                Some(effective_text_style),
            );

            out.push((
                i,
                Rectangle {
                    x: content_x,
                    y: current_y,
                    width: child_size.width,
                    height: child_size.height,
                },
            ));
            flow_child_indices.push(out.len() - 1);
            flow_child_auto_cross_sizes.push(child_style.width.is_none());
            flow_child_constraints.push(FlexItemConstraints {
                grow: child_style.flex_grow.unwrap_or(0.0),
                shrink: child_style.flex_shrink.unwrap_or(1.0),
                basis: child_style.flex_basis.map(|basis| basis * zoom),
            });
            current_y += child_size.height + frame.gap_y;
        }
    }

    if is_grid && grid_col_idx > 0 {
        current_y += max_row_height;
    }

    let content_height = if is_grid {
        if grid_col_idx > 0 {
            (current_y - draw_bounds.y - frame.pt).max(0.0)
        } else {
            (current_y - draw_bounds.y - frame.pt - frame.gap_y).max(0.0)
        }
    } else if is_row {
        max_row_height
    } else {
        (current_y - draw_bounds.y - frame.pt - frame.gap_y).max(0.0)
    };

    let final_height = frame.height_fixed.unwrap_or(content_height + frame.pt + frame.pb);
    draw_bounds.height = final_height;

    if is_flex_layout {
        let available_main = if is_row {
            (draw_bounds.width - frame.pl - frame.pr).max(0.0)
        } else {
            (draw_bounds.height - frame.pt - frame.pb).max(0.0)
        };
        let base_gap = if is_row { frame.gap_x } else { frame.gap_y };
        let content_main = apply_flex_item_constraints(
            &mut out,
            &flow_child_indices,
            &flow_child_constraints,
            is_row,
            available_main,
            base_gap,
        );
        apply_flex_alignment(
            &mut out,
            &flow_child_indices,
            &flow_child_auto_cross_sizes,
            draw_bounds,
            &frame,
            is_row,
            is_reverse,
            align_items,
            justify_content,
            content_main,
        );
    } else if !is_grid {
        let available_width = (draw_bounds.width - frame.pl - frame.pr).max(0.0);

        for out_idx in flow_child_indices {
            let rect = &mut out[out_idx].1;

            if align_items == "center" {
                rect.x = draw_bounds.x + frame.pl + (available_width - rect.width) / 2.0;
            } else if align_items == "flex-end" {
                rect.x = draw_bounds.x + frame.pl + available_width - rect.width;
            }
        }
    }

    Some((draw_bounds, out))
}

fn resolve_replaced_node_layout(
    node: &TailwindNode,
    bounds: Rectangle,
    zoom: f32,
    style: &ParsedStyle,
) -> Option<NodeLayoutResult> {
    if node.tag == "svg" {
        let rect = resolve_svg_rect(style, bounds, zoom, resolve_svg_view_box(node));
        return Some(NodeLayoutResult { rect, size: Size::new(rect.width, rect.height) });
    }

    if node.tag == "img" {
        let rect = resolve_img_rect(style, bounds, zoom);
        return Some(NodeLayoutResult { rect, size: Size::new(rect.width, rect.height) });
    }

    None
}

fn offset_layout_tree(layout: &mut ComputedNodeLayout, dx: f32, dy: f32) {
    layout.rect.x += dx;
    layout.rect.y += dy;
    layout.visual_rect.x += dx;
    layout.visual_rect.y += dy;

    for (_, child) in &mut layout.children {
        offset_layout_tree(child, dx, dy);
    }
}

fn text_layout_size(layout: &super::text::TailwindTextLayout) -> Size {
    if layout.lines.is_empty() {
        return Size::new(0.0, 0.0);
    }

    let width = layout
        .lines
        .iter()
        .map(|line| {
            crate::app::views::design::canvas::rendering::utils::compute_line_width(
                line,
                layout.font_size,
                layout.letter_spacing,
            )
        })
        .fold(0.0, f32::max);

    Size::new(width, layout.lines.len() as f32 * layout.line_height)
}
