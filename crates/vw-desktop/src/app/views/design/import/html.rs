use crate::app::views::design::canvas::tailwind::dom::{TailwindNode, parse_html};
use crate::app::views::design::canvas::tailwind::parser::{ParsedStyle, TailwindParser};
use crate::app::views::design::canvas::tailwind::renderer::{TailwindNodeLayout, layout_roots};
use crate::app::views::design::canvas::utils::apply_tailwind_classes;
use crate::app::views::design::models::DesignElement;
use iced::{Color, Rectangle, Size};
use serde_json::{Value, json};

use super::shared::{TextStyle, generate_id, number_value, numeric_value};

pub fn import_html_as_elements(html: &str) -> Vec<DesignElement> {
    let nodes = parse_html(html);
    nodes.into_iter().map(|node| convert_node(node, &TextStyle::default())).collect()
}

/// 按当前 Tailwind 渲染器的矩形结果，将 HTML 转换为显式定位的图层树。
///
/// 该转换用于“转换为图层”操作：
/// - 复用 Tailwind 画布当前的布局计算结果，避免再次猜测 flex/grid/absolute 位置。
/// - 输出显式的 `x/y/width/height`，并关闭自动布局推断，避免二次布局导致偏移。
/// - 保留现有的背景、边框、文本、图片、SVG/path 等基础样式映射。
pub fn import_html_as_positioned_elements(html: &str, container_size: Size) -> Vec<DesignElement> {
    let nodes = parse_html(html);
    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: container_size.width.max(0.0),
        height: container_size.height.max(0.0),
    };
    let layouts = layout_roots(&nodes, bounds, 1.0);

    nodes
        .into_iter()
        .zip(layouts)
        .map(|(node, layout)| convert_positioned_node(node, &layout, None, &TextStyle::default()))
        .collect()
}

fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    let a = (c.a * 255.0) as u8;

    if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }
}

fn shadow_effect_value(style: &ParsedStyle) -> Option<Value> {
    let color = style.shadow_color?;
    if color.a <= 0.0 {
        return None;
    }

    Some(json!({
        "color": color_to_hex(color),
        "offset_x": style.shadow_offset_x.unwrap_or(0.0),
        "offset_y": style.shadow_offset_y.unwrap_or(0.0),
        "blur": style.shadow_spread.unwrap_or(0.0).max(0.0),
        "spread": style.shadow_spread.unwrap_or(0.0).max(0.0)
    }))
}

fn apply_positioned_geometry(
    element: &mut DesignElement,
    rect: Rectangle,
    parent_rect: Option<Rectangle>,
) {
    let (parent_x, parent_y) = parent_rect.map(|parent| (parent.x, parent.y)).unwrap_or((0.0, 0.0));

    element.x = rect.x - parent_x;
    element.y = rect.y - parent_y;
    element.width = Some(number_value(rect.width.max(0.0)));
    element.height = Some(number_value(rect.height.max(0.0)));
    element.fill_width = None;
    element.fill_height = None;
    element.hug_width = None;
    element.hug_height = None;
}

fn clear_layout_inference_fields(element: &mut DesignElement) {
    element.layout = if element.kind == "frame" { Some("none".to_string()) } else { None };
    element.padding = None;
    element.gap = None;
    element.align_items = None;
    element.justify_content = None;
}

fn convert_positioned_node(
    node: TailwindNode,
    layout: &TailwindNodeLayout,
    parent_rect: Option<Rectangle>,
    parent_style: &TextStyle,
) -> DesignElement {
    let mut element = DesignElement::default();
    element.id = generate_id();
    element.visible = Some(true);

    let mut current_style = parent_style.clone();
    let tag_name = node.tag.clone();

    if let Some(class) = node.attributes.get("class") {
        element.class = Some(class.clone());
        apply_tailwind_classes(&mut element);

        let style = TailwindParser::parse(class);
        if let Some(effect) = shadow_effect_value(&style) {
            element.effect = Some(effect);
        }
        if let Some(opacity) = style.opacity {
            element.opacity = Some(opacity);
        }
        if let Some(font_style) = style.font_style.clone() {
            element.font_style = Some(font_style.clone());
            current_style.font_style = Some(font_style);
        }
        if let Some(text_decoration) = style.text_decoration.clone() {
            element.text_decoration = Some(text_decoration.clone());
            current_style.text_decoration = Some(text_decoration);
        }
        if let Some(letter_spacing) = style.letter_spacing {
            let value = number_value(letter_spacing);
            element.letter_spacing = Some(value.clone());
            current_style.letter_spacing = Some(value);
        }
        if let Some(line_height) = style.line_height {
            let font_size = numeric_value(&element.font_size)
                .or_else(|| numeric_value(&current_style.font_size));
            if let Some(font_size) = font_size {
                let value = number_value(line_height * font_size);
                element.line_height = Some(value.clone());
                current_style.line_height = Some(value);
            }
        }

        if let Some(color) = element.color.clone() {
            current_style.color = Some(color);
        }
        if let Some(font_size) = element.font_size.clone() {
            current_style.font_size = Some(font_size);
        }
        if let Some(font_weight) = element.font_weight.clone() {
            current_style.font_weight = Some(font_weight);
        }
        if let Some(text_align) = element.text_align.clone() {
            current_style.text_align = Some(text_align);
        }
    }

    match tag_name.as_str() {
        "text" => {
            element.kind = "text".to_string();
            element.content = node.text.clone();
            if element.color.is_none() {
                element.color = current_style.color.clone();
            }
            if element.font_size.is_none() {
                element.font_size = current_style.font_size.clone();
            }
            if element.font_weight.is_none() {
                element.font_weight = current_style.font_weight.clone();
            }
            if element.text_align.is_none() {
                element.text_align = current_style.text_align.clone();
            }
            if element.font_style.is_none() {
                element.font_style = current_style.font_style.clone();
            }
            if element.text_decoration.is_none() {
                element.text_decoration = current_style.text_decoration.clone();
            }
            if element.line_height.is_none() {
                element.line_height = current_style.line_height.clone();
            }
            if element.letter_spacing.is_none() {
                element.letter_spacing = current_style.letter_spacing.clone();
            }
        }
        "svg" => {
            element.kind = "frame".to_string();
            element.name = Some("Icon".to_string());
        }
        "path" => {
            element.kind = "path".to_string();
            if let Some(path_data) = node.attributes.get("d") {
                element.geometry = Some(path_data.clone());
            }
            if let Some(fill) = node.attributes.get("fill") {
                if fill == "currentColor" {
                    if let Some(color) = &current_style.color {
                        element.fill = Some(Value::String(color.clone()));
                    }
                } else if fill != "none" {
                    element.fill = Some(Value::String(fill.clone()));
                }
            }
        }
        "img" => {
            element.kind = "image".to_string();
            if let Some(src) = node.attributes.get("src") {
                element.fill = Some(json!({
                    "type": "image",
                    "url": src,
                    "enabled": true
                }));
            }
        }
        _ => {
            element.kind = "frame".to_string();
        }
    }

    apply_positioned_geometry(&mut element, layout.rect, parent_rect);

    let children = if tag_name == "svg" {
        let svg_child_layout = TailwindNodeLayout {
            rect: layout.rect,
            size: Size::new(layout.rect.width, layout.rect.height),
            children: Vec::new(),
        };

        node.children
            .into_iter()
            .map(|child| {
                convert_positioned_node(child, &svg_child_layout, Some(layout.rect), &current_style)
            })
            .collect()
    } else {
        node.children
            .into_iter()
            .zip(layout.children.iter())
            .map(|(child, child_layout)| {
                convert_positioned_node(child, child_layout, Some(layout.rect), &current_style)
            })
            .collect()
    };
    element.children = children;

    clear_layout_inference_fields(&mut element);
    element
}

fn convert_node(node: TailwindNode, parent_style: &TextStyle) -> DesignElement {
    let mut element = DesignElement::default();
    element.id = generate_id();
    element.visible = Some(true);

    let mut current_style = parent_style.clone();

    if let Some(class) = node.attributes.get("class") {
        element.class = Some(class.clone());
        apply_tailwind_classes(&mut element);

        let style = TailwindParser::parse(class);
        let is_flex =
            style.display.as_deref().map(|display| display.contains("flex")).unwrap_or(false);
        if is_flex {
            element.layout = Some(if style.flex_direction.as_deref() == Some("row") {
                "horizontal".to_string()
            } else {
                "vertical".to_string()
            });
        }

        if style.position.as_deref() == Some("absolute") {
            if let Some(top) = style.top {
                element.y = top;
            }
            if let Some(left) = style.left {
                element.x = left;
            }
        }

        if let Some(color) = element.color.clone() {
            current_style.color = Some(color);
        }
        if let Some(font_size) = element.font_size.clone() {
            current_style.font_size = Some(font_size);
        }
        if let Some(font_weight) = element.font_weight.clone() {
            current_style.font_weight = Some(font_weight);
        }
        if let Some(text_align) = element.text_align.clone() {
            current_style.text_align = Some(text_align);
        }
    }

    match node.tag.as_str() {
        "text" => {
            element.kind = "text".to_string();
            element.content = node.text.clone();
            if element.color.is_none() {
                element.color = current_style.color.clone();
            }
            if element.font_size.is_none() {
                element.font_size = current_style.font_size.clone();
            }
            if element.font_weight.is_none() {
                element.font_weight = current_style.font_weight.clone();
            }
            if element.text_align.is_none() {
                element.text_align = current_style.text_align.clone();
            }
        }
        "svg" => {
            element.kind = "frame".to_string();
            element.name = Some("Icon".to_string());
        }
        "path" => {
            element.kind = "path".to_string();
            if let Some(path_data) = node.attributes.get("d") {
                element.geometry = Some(path_data.clone());
            }
            if let Some(fill) = node.attributes.get("fill")
                && fill == "currentColor"
                && let Some(color) = &current_style.color
            {
                element.fill = Some(Value::String(color.clone()));
            }
        }
        "img" => {
            element.kind = "image".to_string();
            if let Some(src) = node.attributes.get("src") {
                element.fill = Some(json!({
                    "type": "image",
                    "url": src,
                    "enabled": true
                }));
            }
        }
        _ => {
            element.kind = "frame".to_string();
        }
    }

    element.children =
        node.children.into_iter().map(|child| convert_node(child, &current_style)).collect();

    element
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod html_tests;
