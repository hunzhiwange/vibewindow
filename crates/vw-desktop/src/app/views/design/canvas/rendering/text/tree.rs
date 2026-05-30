//! 设计画布文本渲染模块。
//!
//! 该模块处理文本节点的排版、网格、树结构或便签绘制逻辑，确保 DOM 风格输入能够稳定映射到画布中的可见文本。

use iced::{Point, Rectangle, Size};

use crate::app::views::design::{
    canvas::{
        layout::{compute_layout, parse_layout, parse_padding, resolve_element_size},
        parse::parse_radius,
        tailwind::TailwindParser,
        types::LayoutDirection,
        utils::{resolve_ref_instance, theme_mode_for_element},
    },
    models::{DesignDoc, DesignElement},
};

use super::{sticky_note::draw_sticky_note_text, typography::draw_typography_text};

fn expand_slot_children(element: &DesignElement) -> Option<Vec<DesignElement>> {
    let Some(serde_json::Value::Array(arr)) = element.slot.as_ref() else {
        return None;
    };

    if arr.is_empty() {
        return None;
    }

    let ids: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
    if ids.is_empty() {
        return None;
    }

    Some(
        ids.into_iter()
            .enumerate()
            .map(|(i, slot_id)| DesignElement {
                kind: "ref".to_string(),
                id: format!("{}__slot__{i}", element.id),
                reference: Some(slot_id.to_string()),
                ..Default::default()
            })
            .collect(),
    )
}

/// 公开的 clamp_child_size_to_content 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(crate) fn clamp_child_size_to_content(
    content_size: Size,
    child_x: f32,
    child_y: f32,
    child_size: Size,
) -> Size {
    let clipped_w = (content_size.width - child_x).max(0.0).min(child_size.width);
    let clipped_h = (content_size.height - child_y).max(0.0).min(child_size.height);
    Size::new(clipped_w, clipped_h)
}

/// 公开的 draw_texts_tree 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_texts_tree(
    frame: &mut iced::widget::canvas::Frame,
    element: &DesignElement,
    parent_pos: Point,
    zoom: f32,
    doc: &DesignDoc,
    overrides: Option<&serde_json::Map<String, serde_json::Value>>,
    parent_size: Option<Size>,
    size_override: Option<Size>,
    editing_id: Option<&str>,
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

            draw_texts_tree(
                frame,
                &inst,
                base_pos,
                zoom,
                doc,
                overrides,
                parent_size,
                Some(override_size),
                editing_id,
                inherited_theme_mode,
                show_slot_content,
                show_slot_overflow,
            );
        }
        return;
    }

    if element.enabled == Some(false) {
        return;
    }

    let x = parent_pos.x + (element.x * zoom);
    let y = parent_pos.y + (element.y * zoom);
    let resolved = size_override
        .unwrap_or_else(|| resolve_element_size(element, parent_size, doc, inherited_theme_mode));
    let w = resolved.width * zoom;
    let h = resolved.height * zoom;
    let theme_mode = theme_mode_for_element(doc, element, inherited_theme_mode);
    let _radius = parse_radius(&element.corner_radius, h, &doc.variables, theme_mode);
    let tailwind_style = element.class.as_deref().map(TailwindParser::parse).unwrap_or_default();

    let mut padding = parse_padding(&element.padding, &doc.variables, theme_mode);
    if let Some(p) = tailwind_style.padding {
        padding.top = p;
        padding.bottom = p;
        padding.left = p;
        padding.right = p;
    }
    if let Some(p) = tailwind_style.padding_top {
        padding.top = p;
    }
    if let Some(p) = tailwind_style.padding_bottom {
        padding.bottom = p;
    }
    if let Some(p) = tailwind_style.padding_left {
        padding.left = p;
    }
    if let Some(p) = tailwind_style.padding_right {
        padding.right = p;
    }

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
            Some(LayoutDirection::Horizontal)
        } else if let Some(display) = &tailwind_style.display {
            if display == "flex" {
                match tailwind_style.flex_direction.as_deref() {
                    Some("column") => Some(LayoutDirection::Vertical),
                    _ => Some(LayoutDirection::Horizontal),
                }
            } else {
                None
            }
        } else {
            None
        }
    });

    let is_editing = editing_id == Some(element.id.as_str());
    if !is_editing && element.kind.eq_ignore_ascii_case("sticky_note") {
        draw_sticky_note_text(frame, element, x, y, w, h, zoom, doc, theme_mode);
    } else if !is_editing && (element.kind == "Typography" || element.kind == "text") {
        draw_typography_text(
            frame,
            element,
            Rectangle { x, y, width: w, height: h },
            resolved,
            zoom,
            doc,
            theme_mode,
            &tailwind_style,
            padding,
        );
    }

    let content_size = Size::new(
        (resolved.width - (padding.left + padding.right)).max(0.0),
        (resolved.height - (padding.top + padding.bottom)).max(0.0),
    );
    let has_slot = element
        .slot
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    let clip_children = (element.clip == Some(true) || element.clip_content == Some(true))
        && !(show_slot_content && show_slot_overflow && has_slot);

    let slot_children_buf = if element.children.is_empty() && show_slot_content {
        expand_slot_children(element)
    } else {
        None
    };
    let children: &[DesignElement] =
        slot_children_buf.as_deref().unwrap_or_else(|| element.children.as_slice());

    let draw_children = |frame: &mut iced::widget::canvas::Frame| {
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
                            LayoutDirection::Vertical => {
                                if cy >= content_size.height {
                                    break;
                                }
                            }
                            LayoutDirection::Horizontal => {
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

                draw_texts_tree(
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
                    editing_id,
                    theme_mode,
                    show_slot_content,
                    show_slot_overflow,
                );
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

                draw_texts_tree(
                    frame,
                    child,
                    Point::new(x + padding.left * zoom, y + padding.top * zoom),
                    zoom,
                    doc,
                    overrides,
                    Some(content_size),
                    child_size_override,
                    editing_id,
                    theme_mode,
                    show_slot_content,
                    show_slot_overflow,
                );
            }
        }
    };

    draw_children(frame);
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tree_tests;
