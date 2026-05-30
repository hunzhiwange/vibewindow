//! Figma 导入模块，负责把 Figma JSON 中的节点、几何、样式和辅助字段转换为设计模型。

use crate::app::views::design::models::DesignElement;
use serde_json::{Map, Value, json};
use std::collections::HashMap;

use super::figma_geometry::read_vector_geometry;
use super::figma_style::{
    first_color_string, map_figma_effects, map_figma_paints, map_figma_stroke,
};
use super::figma_support::{
    clone_number_or_string, first_visible_paint, has_children, has_styled_frame_characteristics,
    is_image_fill, read_clip_value, read_guid_key_from_object, read_guid_key_from_value,
    read_node_type, value_to_f32, value_to_f64, value_to_str,
};
use super::shared::{generate_id, parse_measurement_string};

#[derive(Default)]
/// FigmaImportContext 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct FigmaImportContext {
    raw_symbol_masters: HashMap<String, Value>,
}

impl FigmaImportContext {
    /// 执行 from_raw 对应的设计辅助逻辑。
    ///
    /// 返回值直接交给调用方继续渲染、导入或属性更新。
    pub(super) fn from_raw(raw_json: Option<&Value>) -> Self {
        let mut context = Self::default();
        if let Some(document) = raw_json.and_then(|raw| raw.get("document")) {
            index_raw_symbol_masters(document, &mut context.raw_symbol_masters);
        }
        context
    }
}

fn index_raw_symbol_masters(node: &Value, symbol_masters: &mut HashMap<String, Value>) {
    let Some(object) = node.as_object() else {
        return;
    };

    if read_node_type(object) == Some("SYMBOL")
        && let Some(guid) = read_guid_key_from_object(object, "guid")
    {
        symbol_masters.insert(guid, node.clone());
    }

    if let Some(children) = object.get("children").and_then(Value::as_array) {
        for child in children {
            index_raw_symbol_masters(child, symbol_masters);
        }
    }
}

#[allow(dead_code)]
fn figma_node_to_element(node: &Value) -> Option<DesignElement> {
    figma_node_to_element_with_parent(node, None, 0.0, 0.0, &FigmaImportContext::default())
}

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(super) fn figma_node_to_element_with_parent(
    node: &Value,
    raw_node: Option<&Value>,
    _parent_abs_x: f32,
    _parent_abs_y: f32,
    context: &FigmaImportContext,
) -> Option<DesignElement> {
    let object = node.as_object()?;
    let source_object = raw_node.and_then(Value::as_object).unwrap_or(object);
    let fill_source = object
        .get("fills")
        .or_else(|| object.get("fillPaints"))
        .or_else(|| source_object.get("fills"))
        .or_else(|| source_object.get("fillPaints"));
    let stroke_source = object
        .get("strokes")
        .or_else(|| object.get("strokePaints"))
        .or_else(|| source_object.get("strokes"))
        .or_else(|| source_object.get("strokePaints"));
    let fill = map_figma_paints(fill_source);
    let absolute_x = read_transform_value(source_object, "x");
    let absolute_y = read_transform_value(source_object, "y");
    let mut element = DesignElement {
        id: object
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| read_guid_key_from_object(source_object, "guid"))
            .unwrap_or_else(generate_id),
        kind: figma_kind_for_node(source_object, fill.as_ref()).to_string(),
        name: source_object.get("name").and_then(Value::as_str).map(ToOwned::to_owned),
        x: absolute_x,
        y: absolute_y,
        width: read_size_value(source_object, "x"),
        height: read_size_value(source_object, "y"),
        fill,
        corner_radius: clone_number_or_string(source_object.get("cornerRadius")),
        stroke: map_figma_stroke(source_object, stroke_source),
        effect: map_figma_effects(source_object.get("effects")),
        opacity: source_object.get("opacity").and_then(value_to_f32),
        rotation: source_object.get("rotation").and_then(value_to_f32),
        visible: Some(source_object.get("visible").and_then(Value::as_bool).unwrap_or(true)),
        clip: read_clip_value(source_object),
        enabled: Some(true),
        ..DesignElement::default()
    };

    if let Some(layout) = read_stack_mode(source_object) {
        element.layout = Some(layout);
        element.gap = clone_number_or_string(source_object.get("stackSpacing"));
        element.padding = read_padding(source_object);
    }

    if element.kind == "text" {
        let text_auto_resize = read_text_auto_resize(source_object);
        element.content = read_text_content(source_object);
        element.font_size = read_measurement_value(source_object.get("fontSize"));
        element.font_family = object
            .get("fontName")
            .or_else(|| source_object.get("fontName"))
            .and_then(Value::as_object)
            .and_then(|font| font.get("family"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        element.font_weight = read_font_weight(source_object);
        element.line_height = read_line_height(source_object, element.font_size.as_ref());
        element.letter_spacing = read_measurement_value(source_object.get("letterSpacing"));
        element.text_align = source_object
            .get("textAlignHorizontal")
            .and_then(value_to_str)
            .map(|value| value.to_ascii_lowercase());
        element.text_align_vertical = source_object
            .get("textAlignVertical")
            .and_then(value_to_str)
            .map(|value| value.to_ascii_lowercase());
        element.color = first_color_string(fill_source);
        match text_auto_resize.as_deref() {
            Some("HEIGHT") => {
                if element.width.is_some() {
                    element.text_growth = Some("fixed-width".to_string());
                }
            }
            Some("WIDTH_AND_HEIGHT") => {
                element.width = None;
                element.height = None;
            }
            _ => {}
        }
    }

    if element.kind == "path" && element.geometry.is_none() {
        element.geometry = read_vector_geometry(source_object);
    }

    if element.kind == "path" {
        return Some(element);
    }

    let children = object.get("children").and_then(Value::as_array);
    let raw_children = raw_node.and_then(|raw| raw.get("children")).and_then(Value::as_array);
    element.children = children
        .into_iter()
        .flat_map(|children| children.iter().enumerate())
        .filter_map(|(index, child)| {
            figma_node_to_element_with_parent(
                child,
                raw_children.and_then(|siblings| match_raw_child(siblings, child, index)),
                absolute_x,
                absolute_y,
                context,
            )
        })
        .collect();

    if element.children.is_empty()
        && read_node_type(source_object) == Some("INSTANCE")
        && let Some(resolved_children) = resolve_instance_children(source_object, context)
    {
        element.children = resolved_children
            .iter()
            .filter_map(|child| {
                figma_node_to_element_with_parent(
                    child,
                    Some(child),
                    absolute_x,
                    absolute_y,
                    context,
                )
            })
            .collect();
    }

    Some(element)
}

/// 执行 match_raw_child 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn match_raw_child<'a>(
    raw_children: &'a [Value],
    node: &Value,
    index: usize,
) -> Option<&'a Value> {
    let expected_name = node.get("name").and_then(Value::as_str);
    let expected_x =
        node.as_object().map(|object| read_transform_value(object, "x")).unwrap_or(0.0);
    let expected_y =
        node.as_object().map(|object| read_transform_value(object, "y")).unwrap_or(0.0);

    raw_children
        .get(index)
        .filter(|candidate| {
            let Some(object) = candidate.as_object() else {
                return false;
            };
            object.get("name").and_then(Value::as_str) == expected_name
                && (read_transform_value(object, "x") - expected_x).abs() < 0.01
                && (read_transform_value(object, "y") - expected_y).abs() < 0.01
        })
        .or_else(|| {
            raw_children.iter().find(|candidate| {
                let Some(object) = candidate.as_object() else {
                    return false;
                };
                object.get("name").and_then(Value::as_str) == expected_name
                    && (read_transform_value(object, "x") - expected_x).abs() < 0.01
                    && (read_transform_value(object, "y") - expected_y).abs() < 0.01
            })
        })
}

fn resolve_instance_children(
    object: &Map<String, Value>,
    context: &FigmaImportContext,
) -> Option<Vec<Value>> {
    let symbol_id = object
        .get("symbolData")
        .and_then(Value::as_object)
        .and_then(|symbol_data| symbol_data.get("symbolID"))
        .and_then(read_guid_key_from_value)?;
    let master = context.raw_symbol_masters.get(&symbol_id)?.as_object()?;
    let master_children = master.get("children").and_then(Value::as_array)?;
    let mut resolved_children = master_children.clone();

    if let Some(overrides) = object
        .get("symbolData")
        .and_then(Value::as_object)
        .and_then(|symbol_data| symbol_data.get("symbolOverrides"))
        .and_then(Value::as_array)
    {
        for override_value in overrides {
            apply_instance_override(&mut resolved_children, override_value);
        }
    }

    if let Some(overrides) = object.get("derivedSymbolData").and_then(Value::as_array) {
        for override_value in overrides {
            apply_instance_override(&mut resolved_children, override_value);
        }
    }

    Some(resolved_children)
}

fn apply_instance_override(nodes: &mut [Value], override_value: &Value) {
    let Some(override_object) = override_value.as_object() else {
        return;
    };
    let Some(target_key) = override_object
        .get("guidPath")
        .and_then(Value::as_object)
        .and_then(|guid_path| guid_path.get("guids"))
        .and_then(Value::as_array)
        .and_then(|guids| guids.first())
        .and_then(read_guid_key_from_value)
    else {
        return;
    };

    for node in nodes {
        if apply_instance_override_to_node(node, &target_key, override_object) {
            return;
        }
    }
}

fn apply_instance_override_to_node(
    node: &mut Value,
    target_key: &str,
    override_object: &Map<String, Value>,
) -> bool {
    let Some(object) = node.as_object_mut() else {
        return false;
    };

    if read_guid_key_from_object(object, "overrideKey").as_deref() == Some(target_key) {
        merge_override_fields(object, override_object);
        return true;
    }

    if let Some(children) = object.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if apply_instance_override_to_node(child, target_key, override_object) {
                return true;
            }
        }
    }

    false
}

fn merge_override_fields(target: &mut Map<String, Value>, override_object: &Map<String, Value>) {
    for (key, value) in override_object {
        if matches!(key.as_str(), "guidPath") {
            continue;
        }

        match (target.get_mut(key), value) {
            (Some(Value::Object(target_object)), Value::Object(source_object)) => {
                merge_override_fields(target_object, source_object);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn figma_kind_for_node(object: &Map<String, Value>, fill: Option<&Value>) -> &'static str {
    if let Some(node_type) = read_node_type(object) {
        if node_type != "TEXT" && fill.is_some_and(is_image_fill) {
            return "image";
        }
        return match node_type {
            "TEXT" => "text",
            "ELLIPSE" => "ellipse",
            "LINE" => "line",
            "VECTOR" | "STAR" | "POLYGON" | "BOOLEAN_OPERATION" => "path",
            "RECTANGLE" => "rectangle",
            "GROUP" => "group",
            "FRAME" | "COMPONENT" | "INSTANCE" | "SECTION" | "COMPONENT_SET" => {
                if should_treat_as_group(object, fill) {
                    "group"
                } else {
                    "frame"
                }
            }
            _ => {
                if has_children(object) {
                    "frame"
                } else {
                    "rectangle"
                }
            }
        };
    }

    if read_text_content(object).is_some() {
        return "text";
    }
    if object.get("vectorData").is_some() {
        return "path";
    }
    if fill.is_some_and(is_image_fill) {
        return "image";
    }
    if has_children(object) {
        if has_styled_frame_characteristics(object) { "frame" } else { "group" }
    } else {
        "rectangle"
    }
}

/// 执行 read_transform_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_transform_value(object: &Map<String, Value>, key: &str) -> f32 {
    object
        .get("transform")
        .and_then(Value::as_object)
        .and_then(|transform| {
            transform.get(key).or_else(|| match key {
                "x" => transform.get("m02"),
                "y" => transform.get("m12"),
                _ => None,
            })
        })
        .and_then(value_to_f32)
        .unwrap_or(0.0)
}

fn read_size_value(object: &Map<String, Value>, key: &str) -> Option<Value> {
    object
        .get("size")
        .and_then(Value::as_object)
        .and_then(|size| size.get(key))
        .and_then(|value| clone_number_or_string(Some(value)))
}

fn read_text_content(object: &Map<String, Value>) -> Option<String> {
    object
        .get("textData")
        .and_then(Value::as_object)
        .and_then(|text_data| text_data.get("characters"))
        .and_then(Value::as_str)
        .or_else(|| {
            object
                .get("derivedTextData")
                .and_then(Value::as_object)
                .and_then(|text_data| text_data.get("characters"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned)
}

fn read_text_auto_resize(object: &Map<String, Value>) -> Option<String> {
    object.get("textAutoResize").and_then(value_to_str).map(ToOwned::to_owned)
}

fn read_stack_mode(object: &Map<String, Value>) -> Option<String> {
    match object.get("stackMode").and_then(value_to_str) {
        Some("HORIZONTAL") => Some("horizontal".to_string()),
        Some("VERTICAL") => Some("vertical".to_string()),
        _ => None,
    }
}

fn read_font_weight(object: &Map<String, Value>) -> Option<Value> {
    if let Some(font_weight) =
        object.get("fontWeight").and_then(|value| clone_number_or_string(Some(value)))
    {
        return Some(font_weight);
    }

    let style = object
        .get("fontName")
        .and_then(Value::as_object)
        .and_then(|font| font.get("style"))
        .and_then(value_to_str)?;

    let normalized = match style.to_ascii_lowercase().as_str() {
        "regular" | "normal" | "book" => Value::String("normal".to_string()),
        "medium" => Value::String("500".to_string()),
        "semibold" | "semi bold" | "demibold" => Value::String("600".to_string()),
        "bold" => Value::String("bold".to_string()),
        "extrabold" | "extra bold" | "heavy" => Value::String("800".to_string()),
        "black" => Value::String("900".to_string()),
        "light" => Value::String("300".to_string()),
        "thin" => Value::String("100".to_string()),
        _ => Value::String(style.to_string()),
    };
    Some(normalized)
}

fn read_line_height(object: &Map<String, Value>, font_size: Option<&Value>) -> Option<Value> {
    let line_height = object.get("lineHeight")?;
    if let Some(line_height_object) = line_height.as_object() {
        let number = line_height_object.get("value").and_then(value_to_f64)?;
        let units = line_height_object.get("units").and_then(value_to_str).unwrap_or_default();

        return match units {
            "PERCENT" => {
                if (number - 100.0).abs() < f64::EPSILON {
                    None
                } else {
                    Some(json!(number / 100.0))
                }
            }
            "PIXELS" => {
                if let Some(font_size) = font_size.and_then(value_to_f64)
                    && font_size > 0.0
                {
                    Some(json!(number / font_size))
                } else {
                    Some(json!(number))
                }
            }
            _ => Some(json!(number)),
        };
    }

    if let Some(number) = value_to_f64(line_height) {
        if line_height.is_number() {
            return Some(json!(number));
        }
        if let Some(font_size) = font_size.and_then(value_to_f64)
            && font_size > 0.0
        {
            return Some(json!(number / font_size));
        }
        return Some(json!(number));
    }
    None
}

fn read_measurement_value(value: Option<&Value>) -> Option<Value> {
    match value? {
        Value::Number(number) => Some(Value::Number(number.clone())),
        Value::String(string) => parse_measurement_string(string)
            .map(|number| json!(number))
            .or_else(|| Some(Value::String(string.clone()))),
        Value::Object(object) => {
            let number = object.get("value").and_then(value_to_f64)?;
            let units = object.get("units").and_then(value_to_str).unwrap_or_default();
            match units {
                "PERCENT" => Some(json!(number / 100.0)),
                _ => Some(json!(number)),
            }
        }
        _ => None,
    }
}

fn read_padding(object: &Map<String, Value>) -> Option<Value> {
    let horizontal =
        object.get("stackHorizontalPadding").and_then(|value| clone_number_or_string(Some(value)));
    let vertical =
        object.get("stackVerticalPadding").and_then(|value| clone_number_or_string(Some(value)));
    let top = object
        .get("stackPaddingTop")
        .and_then(|value| clone_number_or_string(Some(value)))
        .or(vertical.clone());
    let right = object
        .get("stackPaddingRight")
        .and_then(|value| clone_number_or_string(Some(value)))
        .or(horizontal.clone());
    let bottom = object
        .get("stackPaddingBottom")
        .and_then(|value| clone_number_or_string(Some(value)))
        .or(vertical);
    let left = object
        .get("stackPaddingLeft")
        .and_then(|value| clone_number_or_string(Some(value)))
        .or(horizontal);

    match (top, right, bottom, left) {
        (None, None, None, None) => None,
        (Some(top), Some(right), Some(bottom), Some(left))
            if top == right && right == bottom && bottom == left =>
        {
            Some(top)
        }
        (Some(top), Some(right), Some(bottom), Some(left)) => {
            Some(Value::Array(vec![top, right, bottom, left]))
        }
        (Some(top), None, None, None) => Some(top),
        _ => None,
    }
}

fn should_treat_as_group(object: &Map<String, Value>, fill: Option<&Value>) -> bool {
    has_children(object)
        && read_clip_value(object) != Some(true)
        && read_stack_mode(object).is_none()
        && object.get("effects").is_none()
        && object.get("backgroundColor").is_none()
        && fill.is_none()
        && object.get("fillPaints").and_then(first_visible_paint).is_none()
}

#[cfg(test)]
#[path = "figma_node_tests.rs"]
mod figma_node_tests;
